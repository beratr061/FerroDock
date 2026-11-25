use crate::{AppError, AppResult};

const MAX_ENTRIES: u32 = 10_000;
const MAX_NAME_LEN: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub data: Vec<u8>,
}

fn corrupted() -> AppError {
    AppError::validation("BACKUP_ARCHIVE_INVALID")
}

pub fn valid_entry_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME_LEN
        && !name.starts_with('/')
        && !name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

pub fn pack(entries: &[Entry]) -> AppResult<Vec<u8>> {
    let count = u32::try_from(entries.len()).map_err(|_| corrupted())?;
    let mut out = Vec::new();
    out.extend_from_slice(&count.to_be_bytes());
    for entry in entries {
        if !valid_entry_name(&entry.name) {
            return Err(corrupted());
        }
        let name_len = u16::try_from(entry.name.len()).map_err(|_| corrupted())?;
        out.extend_from_slice(&name_len.to_be_bytes());
        out.extend_from_slice(entry.name.as_bytes());
        out.extend_from_slice(&(entry.data.len() as u64).to_be_bytes());
        out.extend_from_slice(&entry.data);
    }
    Ok(out)
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> AppResult<&'a [u8]> {
        let (head, rest) = self.bytes.split_at_checked(length).ok_or_else(corrupted)?;
        self.bytes = rest;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> AppResult<[u8; N]> {
        self.take(N)?.try_into().map_err(|_| corrupted())
    }
}

pub fn unpack(bytes: &[u8]) -> AppResult<Vec<Entry>> {
    let mut reader = Reader { bytes };
    let count = u32::from_be_bytes(reader.array()?);
    if count > MAX_ENTRIES {
        return Err(corrupted());
    }
    let mut entries = Vec::new();
    for _ in 0..count {
        let name_len = usize::from(u16::from_be_bytes(reader.array()?));
        let name = String::from_utf8(reader.take(name_len)?.to_vec()).map_err(|_| corrupted())?;
        let data_len =
            usize::try_from(u64::from_be_bytes(reader.array()?)).map_err(|_| corrupted())?;
        if !valid_entry_name(&name) {
            return Err(corrupted());
        }
        entries.push(Entry {
            name,
            data: reader.take(data_len)?.to_vec(),
        });
    }
    if reader.bytes.is_empty() {
        Ok(entries)
    } else {
        Err(corrupted())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rejects_damage() -> AppResult<()> {
        let entries = vec![
            Entry {
                name: "manifest.json".to_owned(),
                data: b"{}".to_vec(),
            },
            Entry {
                name: "proxy/app-1.yml".to_owned(),
                data: vec![0, 1, 2],
            },
        ];
        let packed = pack(&entries)?;
        assert_eq!(unpack(&packed)?, entries);
        assert!(unpack(&packed[..packed.len() - 1]).is_err());
        let mut extended = packed.clone();
        extended.push(0);
        assert!(unpack(&extended).is_err());
        Ok(())
    }

    #[test]
    fn refuses_unsafe_names() {
        for name in ["../etc/passwd", "/abs", "a//b", "a/./b", "", "spa ce"] {
            assert!(!valid_entry_name(name), "{name}");
        }
        assert!(valid_entry_name("ssh/known_hosts"));
    }
}
