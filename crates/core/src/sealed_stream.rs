use zeroize::Zeroizing;

use crate::crypto::{SecretBox, random_bytes};
use crate::{AppError, AppResult};

pub const CHUNK_SIZE: usize = 1024 * 1024;
const MAGIC: &[u8; 4] = b"FDS1";
const PREFIX_LEN: usize = 7;
const HEADER_LEN: usize = MAGIC.len() + PREFIX_LEN;
const TAG_LEN: usize = 16;
const LENGTH_LEN: usize = 4;
const MAX_SEALED_CHUNK: usize = CHUNK_SIZE + TAG_LEN;

fn invalid() -> AppError {
    AppError::validation("BACKUP_ARCHIVE_INVALID")
}

fn nonce(header: &[u8; HEADER_LEN], counter: u32, last: bool) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..PREFIX_LEN].copy_from_slice(&header[MAGIC.len()..]);
    nonce[PREFIX_LEN..PREFIX_LEN + 4].copy_from_slice(&counter.to_be_bytes());
    nonce[11] = u8::from(last);
    nonce
}

#[derive(Debug)]
pub struct StreamSealer<'a> {
    secrets: &'a SecretBox,
    header: [u8; HEADER_LEN],
    counter: u32,
    finished: bool,
}

impl<'a> StreamSealer<'a> {
    pub fn new(secrets: &'a SecretBox) -> Self {
        let mut header = [0u8; HEADER_LEN];
        header[..MAGIC.len()].copy_from_slice(MAGIC);
        header[MAGIC.len()..].copy_from_slice(&random_bytes::<PREFIX_LEN>());
        Self {
            secrets,
            header,
            counter: 0,
            finished: false,
        }
    }

    pub fn header(&self) -> &[u8] {
        &self.header
    }

    pub fn seal(&mut self, chunk: &[u8], last: bool) -> AppResult<Vec<u8>> {
        if self.finished || chunk.len() > CHUNK_SIZE {
            return Err(AppError::validation("ENCRYPTION_FAILED"));
        }
        let sealed = self.secrets.seal_chunk(
            nonce(&self.header, self.counter, last),
            &self.header,
            chunk,
        )?;
        self.counter = self
            .counter
            .checked_add(1)
            .ok_or_else(|| AppError::validation("ENCRYPTION_FAILED"))?;
        self.finished = last;
        let length = u32::try_from(sealed.len()).map_err(AppError::internal)?;
        let mut framed = Vec::with_capacity(LENGTH_LEN + sealed.len());
        framed.extend_from_slice(&length.to_be_bytes());
        framed.extend_from_slice(&sealed);
        Ok(framed)
    }
}

#[derive(Debug)]
pub struct StreamOpener<'a> {
    secrets: &'a SecretBox,
    header: Option<[u8; HEADER_LEN]>,
    pending: Vec<u8>,
    counter: u32,
    finished: bool,
}

impl<'a> StreamOpener<'a> {
    pub fn new(secrets: &'a SecretBox) -> Self {
        Self {
            secrets,
            header: None,
            pending: Vec::new(),
            counter: 0,
            finished: false,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> AppResult<Vec<Zeroizing<Vec<u8>>>> {
        self.pending.extend_from_slice(bytes);
        let mut opened = Vec::new();
        while let Some(chunk) = self.next_chunk()? {
            opened.push(chunk);
        }
        Ok(opened)
    }

    pub fn finish(&self) -> AppResult<()> {
        if self.finished && self.pending.is_empty() {
            Ok(())
        } else {
            Err(invalid())
        }
    }

    fn take_header(&mut self) -> AppResult<Option<[u8; HEADER_LEN]>> {
        if let Some(header) = self.header {
            return Ok(Some(header));
        }
        if self.pending.len() < HEADER_LEN {
            return Ok(None);
        }
        let header: [u8; HEADER_LEN] = self.pending[..HEADER_LEN]
            .try_into()
            .map_err(|_| invalid())?;
        if &header[..MAGIC.len()] != MAGIC {
            return Err(invalid());
        }
        self.pending.drain(..HEADER_LEN);
        self.header = Some(header);
        Ok(Some(header))
    }

    fn next_chunk(&mut self) -> AppResult<Option<Zeroizing<Vec<u8>>>> {
        let Some(header) = self.take_header()? else {
            return Ok(None);
        };
        let Some(length) = self.pending.get(..LENGTH_LEN) else {
            return Ok(None);
        };
        let length = u32::from_be_bytes(length.try_into().map_err(|_| invalid())?) as usize;
        if self.finished || !(TAG_LEN..=MAX_SEALED_CHUNK).contains(&length) {
            return Err(invalid());
        }
        let Some(sealed) = self.pending.get(LENGTH_LEN..LENGTH_LEN + length) else {
            return Ok(None);
        };
        let open = |last| {
            self.secrets
                .open_chunk(nonce(&header, self.counter, last), &header, sealed)
        };
        let (plaintext, last) = match open(false) {
            Some(plaintext) => (plaintext, false),
            None => (open(true).ok_or_else(invalid)?, true),
        };
        self.pending.drain(..LENGTH_LEN + length);
        self.counter = self.counter.checked_add(1).ok_or_else(invalid)?;
        self.finished = last;
        Ok(Some(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sealed(secrets: &SecretBox, chunks: &[&[u8]]) -> AppResult<Vec<u8>> {
        let mut sealer = StreamSealer::new(secrets);
        let mut out = sealer.header().to_vec();
        for (index, chunk) in chunks.iter().enumerate() {
            out.extend(sealer.seal(chunk, index + 1 == chunks.len())?);
        }
        Ok(out)
    }

    fn opened(secrets: &SecretBox, bytes: &[u8], step: usize) -> AppResult<Vec<u8>> {
        let mut opener = StreamOpener::new(secrets);
        let mut plain = Vec::new();
        for piece in bytes.chunks(step) {
            for chunk in opener.push(piece)? {
                plain.extend_from_slice(&chunk);
            }
        }
        opener.finish()?;
        Ok(plain)
    }

    #[test]
    fn round_trips_in_any_read_size() -> AppResult<()> {
        let secrets = SecretBox::from_key(&[3u8; 32])?;
        let bytes = sealed(&secrets, &[b"hello ", b"stream", b""])?;
        for step in [1, 5, 64, bytes.len()] {
            assert_eq!(opened(&secrets, &bytes, step)?, b"hello stream");
        }
        Ok(())
    }

    #[test]
    fn rejects_truncation_reordering_and_tampering() -> AppResult<()> {
        let secrets = SecretBox::from_key(&[3u8; 32])?;
        let bytes = sealed(&secrets, &[b"first", b"second"])?;
        let first_end = HEADER_LEN + LENGTH_LEN + 5 + TAG_LEN;
        assert!(opened(&secrets, &bytes[..first_end], 64).is_err());
        let mut swapped = bytes[..HEADER_LEN].to_vec();
        swapped.extend_from_slice(&bytes[first_end..]);
        swapped.extend_from_slice(&bytes[HEADER_LEN..first_end]);
        assert!(opened(&secrets, &swapped, 64).is_err());
        let mut flipped = bytes.clone();
        flipped[HEADER_LEN + LENGTH_LEN] ^= 1;
        assert!(opened(&secrets, &flipped, 64).is_err());
        let other = SecretBox::from_key(&[4u8; 32])?;
        assert!(opened(&other, &bytes, 64).is_err());
        let mut trailing = bytes;
        trailing.push(0);
        assert!(opened(&secrets, &trailing, 64).is_err());
        Ok(())
    }
}
