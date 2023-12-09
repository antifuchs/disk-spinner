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
        // Fill the tail with random data:
        let buf_length = size;
        let mut buf = Vec::with_capacity(buf_length);
        buf.resize(buf_length, 0);
        let mut rng = thread_rng();
        rng.fill_bytes(&mut buf[Self::OFFSET..buf_length]);

        // Compute the hash:
        let mut hasher = Hasher::new();
        hasher.update(&buf[Self::OFFSET..buf_length]);
        let hash = hasher.finalize();
        let hash_bytes = hash.as_bytes();
        buf[0..(hash_bytes.len())].copy_from_slice(hash_bytes);

        Self { buf }
    }

    /// Writes the block to the given stream, computing the hash for the buffer first.
    pub(crate) fn write_to(self, out: &mut impl io::Write) -> io::Result<()> {
        out.write_all(&self.buf)
    }
}
