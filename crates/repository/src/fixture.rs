//! Builders the unit tests share: bytes of a git clone made in memory,
//! with no `git` binary, and a seeded generator for the corruption
//! loops.

use crate::inflate::adler32;

/// The bytes `text` spells in hexadecimal digits.
pub(crate) fn hex(text: &str) -> Vec<u8> {
    let digits: Vec<u8> = text
        .bytes()
        .filter(u8::is_ascii_hexdigit)
        .map(|b| match b {
            b'0'..=b'9' => b - b'0',
            _ => b.to_ascii_lowercase() - b'a' + 10,
        })
        .collect();
    digits.chunks(2).map(|p| p[0] << 4 | p[1]).collect()
}

/// `data` as a zlib stream of stored blocks.
pub(crate) fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut blocks = data.chunks(0xffff).peekable();
    if blocks.peek().is_none() {
        out.extend([1, 0, 0, 0xff, 0xff]);
    }
    while let Some(block) = blocks.next() {
        let len = u16::try_from(block.len()).unwrap();
        out.push(u8::from(blocks.peek().is_none()));
        out.extend(len.to_le_bytes());
        out.extend((!len).to_le_bytes());
        out.extend(block);
    }
    out.extend(adler32(data).to_be_bytes());
    out
}

/// A small deterministic generator (xorshift64*), so a corruption loop
/// replays from its seed.
pub(crate) struct Prng(u64);

impl Prng {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// A number below `bound`, which must not be zero.
    pub(crate) fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next_u64() % u64::try_from(bound).unwrap()).unwrap()
    }

    pub(crate) fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_u64().to_le_bytes()[3]).collect()
    }

    /// `data` with some bytes replaced, flipped or cut off.
    pub(crate) fn corrupt(&mut self, data: &[u8]) -> Vec<u8> {
        let mut out = data.to_vec();
        for _ in 0..=self.below(4) {
            let at = self.below(out.len().max(1));
            match (self.below(3), out.get_mut(at)) {
                (0, Some(byte)) => *byte ^= 1 << self.below(8),
                (1, Some(byte)) => *byte = self.next_u64().to_le_bytes()[0],
                _ => out.truncate(at),
            }
        }
        out
    }
}
