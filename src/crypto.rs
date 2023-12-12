//! Routines for generating an infinite amount of deterministic garbage.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::io;

/// A generator for deterministically random-looking garbage data.
#[derive(Clone)]
pub(crate) struct GarbageGenerator<P: Fn(u64)> {
    buf: Vec<u8>,
    hasher: blake3::Hasher,
    lba: usize,
    progress: P,
}

impl<P: Fn(u64)> GarbageGenerator<P> {
    /// Generate a new garbage generator for a block size from a random seed.
    pub(crate) fn new(block_size: usize, seed: u64, progress: P) -> Self {
        let mut buf = Vec::new();
        buf.resize(block_size, 0);

        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut key = [0; 32];
        rng.fill_bytes(&mut key);
        let hasher = blake3::Hasher::new_keyed(&key);

        Self {
            buf,
            hasher,
            lba: 0,
            progress,
        }
    }
}

/// GarbageGenerator implements Read in order to supply the write test
/// with random data that can be copied to disk.
impl<P: Fn(u64)> io::Read for GarbageGenerator<P> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut done = 0;
        for chunk in buf.chunks_exact_mut(self.buf.len()) {
            let length = chunk.len();
            let mut chunk = io::Cursor::new(chunk);
            self.hasher.update(&self.lba.to_le_bytes());
            let reader = self.hasher.finalize_xof();
            io::copy(&mut reader.take(length.try_into().unwrap()), &mut chunk)?;
            self.hasher.reset();
            done += length;
            self.lba += 1;
        }
        (self.progress)(done.try_into().unwrap());
        Ok(done)
    }
}
