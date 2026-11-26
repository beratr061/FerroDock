use std::collections::VecDeque;

pub const REPLAY_CAPACITY: usize = 64 * 1024;

#[derive(Debug)]
pub struct ReplayBuffer {
    bytes: VecDeque<u8>,
    capacity: usize,
}

impl ReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            bytes: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, chunk: &[u8]) {
        let kept = &chunk[chunk.len().saturating_sub(self.capacity)..];
        let overflow = (self.bytes.len() + kept.len()).saturating_sub(self.capacity);
        self.bytes.drain(..overflow);
        self.bytes.extend(kept);
    }

    pub fn snapshot(&self) -> Vec<u8> {
        self.bytes.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_newest_bytes() {
        let mut replay = ReplayBuffer::new(8);
        replay.push(b"hello ");
        replay.push(b"world");
        assert_eq!(replay.snapshot(), b"lo world");
        replay.push(b"0123456789");
        assert_eq!(replay.snapshot(), b"23456789");
        replay.push(b"");
        assert_eq!(replay.snapshot().len(), 8);
    }
}
