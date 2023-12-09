//! Block-handling routines for validating that everything that's written can be read.

use blake3::Hasher;
use rand::prelude::*;
use std::io;

/// A block of random data, prefixed by a blake3 hash of its contents.
pub(crate) struct Block {
    buf: Vec<u8>,
}

impl Block {
    const OFFSET: usize = blake3::OUT_LEN;

    pub(crate) fn new(size: usize) -> Self {
        let buf_length = size;
        let mut buf = Vec::with_capacity(buf_length);
        buf.resize(buf_length, 0);
        let mut rng = thread_rng();
        rng.fill_bytes(&mut buf[Self::OFFSET..buf_length]);
        Self { buf }
    }

    /// Writes the block to the given stream, computing the hash for the buffer first.
    pub(crate) fn write_to(self, out: &mut impl io::Write) -> io::Result<()> {
        let mut hasher = Hasher::new();
        let len = self.buf.len();
        hasher.update(&self.buf[Self::OFFSET..len]);
        let hash = hasher.finalize();
        let hash_bytes = hash.as_bytes();
        let mut buf = self.buf;
        buf[0..(hash_bytes.len())].copy_from_slice(hash_bytes);
        out.write_all(&buf)
    }
}
