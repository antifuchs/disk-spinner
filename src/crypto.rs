//! Routines for generating an infinite amount of deterministic garbage.

use std::io;
use aes::cipher::{KeyIvInit, StreamCipher};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

type ActiveCipher = ctr::Ctr128LE<aes::Aes128>;

/// A generator for deterministically random-looking garbage data.
#[derive(Clone)]
pub(crate) struct GarbageGenerator<P: Fn(u64)> {
    buf: Vec<u8>,
    cipher: ActiveCipher,
    progress: P,
}

impl<P: Fn(u64)> GarbageGenerator<P> {
    /// Generate a new garbage generator for a block size from a random seed.
    pub(crate) fn new(block_size: usize, seed: u64, progress: P) -> Self {
        let mut buf = Vec::new();
        buf.resize(block_size, 0);

        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut key = [0; 16];
        let mut iv = [0; 16];
        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut iv);
        let cipher = ActiveCipher::new(&key.into(), &iv.into());

        Self { buf, cipher, progress }
    }
}

/// GarbageGenerator implements Read in order to supply the write test
/// with random data that can be copied to disk.
impl<P: Fn(u64)> io::Read for GarbageGenerator<P> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut done = 0;
        for chunk in buf.chunks_exact_mut(self.buf.len()) {
            self.cipher
                .apply_keystream_b2b(&self.buf, chunk)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("crypto error {:?}", e)))?;
            done+=chunk.len();
        }
        (self.progress)(done.try_into().unwrap());
        Ok(done)
    }
}
