pub mod hmac_sha2;
mod sha2;

use sha2::Sha256;

#[derive(Default)]
pub(crate) struct HashDigest {
    h: Sha256,
}

impl HashDigest {
    pub fn new() -> HashDigest {
        HashDigest::default()
    }

    pub fn input(&mut self, input: &[u8]) {
        self.h.input(input)
    }

    pub fn result(self) -> [u8; 32] {
        self.h.finalize()
    }
}

pub trait HashTrait<T> {
    fn hash_digest(&self) -> T
    where
        T: Sized;
}

impl HashTrait<[u8; 32]> for [u8] {
    fn hash_digest(&self) -> [u8; 32] {
        let mut hasher = HashDigest::new();
        hasher.input(&self);
        hasher.result()
    }
}
