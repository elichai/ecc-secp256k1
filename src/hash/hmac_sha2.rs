use crate::hash::{HashDigest, HashTrait};
use std::io::Write;

pub struct HmacSha256 {
    inner: HashDigest,
    outer: HashDigest,
}

pub struct HmacSha256Drbg {
    pub(crate) k: [u8; 32],
    pub(crate) v: [u8; 32],
}

impl HmacSha256 {
    #[allow(dead_code)]
    const OPAD: [u8; 64] = [0x5C; 64];
    const IPAD: [u8; 64] = [0x36; 64];
    const IPAD_XOR_OPAD: [u8; 64] = [0x36 ^ 0x5C; 64];
    const BLOCK_SIZE: usize = 64;

    pub fn new(key: &[u8]) -> Self {
        let mut k = [0u8; 64];
        if key.len() > Self::BLOCK_SIZE {
            let key = key.hash_digest();
            k[..key.len()].copy_from_slice(&key);
        } else {
            k[..key.len()].copy_from_slice(key);
        }

        let mut inner = HashDigest::new();
        xor(&mut k, &Self::IPAD);
        inner.input(&k);

        let mut outer = HashDigest::new();
        xor(&mut k, &Self::IPAD_XOR_OPAD);
        outer.input(&k);

        Self { inner, outer }
    }

    pub fn input(&mut self, text: &[u8]) {
        self.inner.input(text)
    }

    pub fn finalize(self) -> [u8; 32] {
        let Self { inner, mut outer } = self;
        outer.input(&inner.result());
        outer.result()
    }

    #[inline]
    pub fn quick(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
        let mut res = Self::new(key);
        res.input(data);
        res.finalize()
    }
}

impl HmacSha256Drbg {
    pub fn new(seed: &[u8], seed2: Option<&[u8]>) -> Self {
        let k = [0u8; 32];
        let v = [1u8; 32];

        let mut hmac = HmacSha256::new(&k);
        hmac.input(&v);
        hmac.input(&[0]);
        hmac.input(seed);
        if let Some(seed2) = seed2 {
            hmac.input(seed2);
        }
        let k = hmac.finalize();

        let v = HmacSha256::quick(&k, &v);

        let mut hmac = HmacSha256::new(&k);
        hmac.input(&v);
        hmac.input(&[1]);
        hmac.input(seed);
        if let Some(seed2) = seed2 {
            hmac.input(seed2);
        }
        let k = hmac.finalize();
        let v = HmacSha256::quick(&k, &v);

        Self { k, v }
    }

    pub fn generate(&mut self, mut out: &mut [u8]) {
        let len = out.len();
        for i in (0..=(len / 32)).rev() {
            self.v = HmacSha256::quick(&self.k, &self.v);
            let amount = out.write(&self.v).unwrap();
            if i > 0 && amount != 32 {
                println!("amout: {:?} max len: {}, i: {}", amount, len, i);
                unimplemented!("Something wrong with random generator");
            } else if i == 0 && amount > 0 {
                debug_assert_eq!(amount, len % 32);
            }
        }
    }
}

#[inline(always)]
fn xor(lhs: &mut [u8], rhs: &[u8]) {
    debug_assert!(lhs.len() <= rhs.len());
    for i in 0..lhs.len() {
        lhs[i] ^= rhs[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::FromHex;

    #[test]
    fn test_hmac_test_vectors() {
        assert!(test_vector(
            hex("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"),
            b"Hi There",
            hex("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"),
            None
        ));
        assert!(test_vector(
            b"Jefe",
            b"what do ya want for nothing?",
            hex("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"),
            None
        ));

        assert!(test_vector(
            [0xAA; 20],
            &[0xDD; 50][..],
            hex("773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe"),
            None
        ));
        assert!(test_vector(
            hex("0102030405060708090a0b0c0d0e0f10111213141516171819"),
            &[0xCD_u8; 50][..],
            hex("82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b"),
            None
        ));
        assert!(test_vector([0x0C_u8; 20], b"Test With Truncation", hex("a3b6167473100ee06e0c796c2955552b"), Some(16)));
        assert!(test_vector(
            &[0xAA; 131][..],
            &b"Test Using Larger Than Block-Size Key - Hash Key First"[..],
            hex("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"),
            None
        ));
        assert!(test_vector(
            &[0xAA; 131][..],
            &b"This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm."[..],
            hex("9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2"),
            None
        ));
    }

    fn test_vector<A: AsRef<[u8]>, B: AsRef<[u8]>, C: AsRef<[u8]>>(key: A, data: B, res: C, len: Option<usize>) -> bool {
        let key = key.as_ref();
        let data = data.as_ref();
        let res = res.as_ref();
        let len = len.unwrap_or(32);

        let mut hmac = HmacSha256::new(key);
        hmac.input(data);

        &hmac.finalize()[..len] == res
    }

    fn hex(hex: &str) -> Vec<u8> {
        hex.from_hex().unwrap()
    }
}
