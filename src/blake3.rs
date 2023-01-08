use bytes::BufMut;
use crypto_common::{Output, OutputSizeUser};
use rsa::signature::digest::{FixedOutput, HashMarker, Update};
use std::io;
use std::io::Read;
use typenum::U128;

pub struct Blake3Digest {
    hasher: blake3::Hasher,
}

impl Blake3Digest {
    fn new() -> Self {
        let hasher = blake3::Hasher::new();
        Self { hasher }
    }
}

impl Update for Blake3Digest {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }
}

impl OutputSizeUser for Blake3Digest {
    type OutputSize = U128;
}

impl FixedOutput for Blake3Digest {
    fn finalize_into(self, out: &mut Output<Self>) {
        let length = out.len();
        let mut output_reader = self.hasher.finalize_xof().take(length as u64);
        let mut writer = out.writer();
        io::copy(&mut output_reader, &mut writer).unwrap();
    }
}

impl HashMarker for Blake3Digest {}

impl Default for Blake3Digest {
    fn default() -> Self {
        Self::new()
    }
}
