use crate::hash::HashDigest;
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
    const IPAD: [u8; 64] = [0x36; 64];
    const OPAD: [u8; 64] = [0x5C; 64];
    const IPAD_XOR_OPAD: [u8; 64] = [0x36^0x5C; 64];
    const BLOCK_SIZE: usize = 64*8;

    pub fn new(key: &[u8]) -> Result<Self, &'static str> {
        if key.len() > Self::BLOCK_SIZE {
            return Err("Max block size  is 512 bytes");
        }
        let mut k = [0u8; 64];
        k[..key.len()].copy_from_slice(key);

        let mut inner = HashDigest::new();
        xor(&mut k, &Self::IPAD);
        inner.input(&k);

        let mut outer = HashDigest::new();
        xor(&mut k, &Self::IPAD_XOR_OPAD);
        outer.input(&k);

        Ok(Self{inner, outer})
    }

    pub fn new_exact(key: &[u8; 32]) -> Self {
        Self::new(key).unwrap()
    }

    pub fn input(&mut self, text: &[u8]) {
        self.inner.input(text)
    }

    pub fn finalize(self) -> [u8; 32] {
        let Self{inner, mut outer} = self;
        outer.input(&inner.result());
        outer.result()
    }

    #[inline]
    pub fn quick(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
        let mut res = Self::new_exact(key);
        res.input(data);
        res.finalize()
    }
}

impl HmacSha256Drbg {
    pub fn new(seed: &[u8], seed2: Option<&[u8]>) -> Self {
        let k_start = [0u8; 32];
        let v_start = [1u8; 32];

        let mut hmac = HmacSha256::new_exact(&k_start); // k.len() == 32 so can never fail.
        hmac.input(&v_start);
        hmac.input(&[0]);
        hmac.input(seed);
        if let Some(seed2) = seed2 {
            hmac.input(seed2);
        }
        let k = hmac.finalize();

        let v = HmacSha256::quick(&k, &v_start);

        let mut hmac = HmacSha256::new_exact(&k);
        hmac.input(&v_start);
        hmac.input(&[1]);
        hmac.input(seed);
        if let Some(seed2) = seed2 {
            hmac.input(seed2);
        }
        let k = hmac.finalize();
        let v = HmacSha256::quick(&k, &v);

        Self {k, v}
    }

    pub fn generate(&mut self, mut out: &mut [u8]) {
        let len = out.len();
        for i in (0..=(len/32)).rev() {
            self.v = HmacSha256::quick(&self.k, &self.v);
            let amount = out.write(&self.v).unwrap();
            if i > 0 && amount != 32 {
                println!("amout: {:?} max len: {}, i: {}", amount, len, i);
                unimplemented!("Something wrong with random generator");
            } else if i==0 && amount > 0 {
                debug_assert_eq!(amount, len%32);
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