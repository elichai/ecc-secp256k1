use sha2::{Digest, Sha256};

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
        let mut res = [0u8; 32];
        res.copy_from_slice(&self.h.result());
        res
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
        let mut result = [0u8; 32];
        result.copy_from_slice(&hasher.result());
        result
    }
}
