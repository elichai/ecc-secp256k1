use std::{io::Read, mem, slice, fmt};

const U32_SIZE: usize = mem::size_of::<u32>();
const U32_ALIGN: usize = mem::align_of::<u32>();
const BLOCK_SIZE: usize = 64;
const BLOCK_SIZE_BITS: u64 = BLOCK_SIZE as u64 * 8;

pub struct Sha256 {
    hash: [u32; 8],
    curr: Vec64,
    len: u64,
}

impl Sha256 {
    pub fn process_block(&mut self, block: [u32; 16]) {
        let mut W = [0u32; BLOCK_SIZE];
        W[..16].copy_from_slice(&block);

        for t in 16..BLOCK_SIZE {
            W[t] = s_sigma1(W[t - 2]).wrapping_add(W[t - 7]).wrapping_add(s_sigma0(W[t-15])).wrapping_add(W[t - 16]);
        }
        let H = &mut self.hash;
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (H[0], H[1], H[2], H[3], H[4], H[5], H[6], H[7]);


        for t in 0..BLOCK_SIZE {
            let T1 = h.wrapping_add(b_sigma1(e)).wrapping_add(choose(e, f, g)).wrapping_add(K[t]).wrapping_add(W[t]);
            let T2 = b_sigma0(a).wrapping_add(majority(a, b, c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(T1);
            d = c;
            c = b;
            b = a;
            a = T1.wrapping_add(T2);
        }
        H[0].wrapping_add_mut(a);
        H[1].wrapping_add_mut(b);
        H[2].wrapping_add_mut(c);
        H[3].wrapping_add_mut(d);
        H[4].wrapping_add_mut(e);
        H[5].wrapping_add_mut(f);
        H[6].wrapping_add_mut(g);
        H[7].wrapping_add_mut(h);
    }

    pub const fn new() -> Self {
        Self {
            hash: [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19],
            curr: Vec64::empty(),
            len: 0,
        }
    }

    pub fn input(&mut self, data: &[u8]) {
        // TODO: This can probably be optimized
        for &byte in data {
            if self.curr.is_full() {
                self.process_current_block();
            }
            self.curr.push(byte);
        }
        self.len += 8 * data.len() as u64;
    }

    #[inline(always)]
    fn finalize_internal(mut self) -> [u32; 8] {
        let zeroes = [0u8; BLOCK_SIZE_BITS as usize - BLOCK_SIZE - 1];
        let len: u64 = self.len;
        let last_block_len: u64 =  BLOCK_SIZE_BITS - BLOCK_SIZE as u64;
        let mut how_many_zeros: u64 = last_block_len.wrapping_sub(8).wrapping_sub(len) % BLOCK_SIZE_BITS;
        self.input(&[0b10000000]);
        if how_many_zeros != 0 {
            self.input(&zeroes[..(how_many_zeros/8) as usize]);
        }
        self.input(&len.to_be_bytes());
        if self.curr.pos as usize == BLOCK_SIZE {
            self.process_current_block();
        }
        debug_assert!(self.curr.is_empty());
        self.hash
    }

    pub fn finalize(mut self) -> [u8; 32] {

        let mut hash = self.finalize_internal();

        debug_assert_eq!(mem::size_of_val(&hash), mem::size_of::<[u8; 32]>());

        memory_le_to_be(&mut hash);
        let mut res = unsafe { *(hash.as_ptr() as *const u8 as *const [u8; 32]) };
        res
    }

    pub fn process_current_block(&mut self) {
        debug_assert!(self.curr.is_full());
        self.process_block(self.curr.to_data());
        self.curr.clear();
    }
}

#[inline(always)]
pub const fn b_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline(always)]
pub const fn b_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}
#[inline(always)]
pub const fn s_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}
#[inline(always)]
pub const fn s_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}
#[inline(always)]
pub const fn choose(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}
#[inline(always)]
pub const fn majority(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

struct Vec64 {
    data: [u8; 64],
    pos: u8,
}



impl Vec64 {
    const BUF_SIZE: u8 = 64;

    #[inline]
    pub fn push(&mut self, byte: u8) -> bool {
        if self.is_full() {
            unreachable!();
            false
        } else {
            self.data[self.pos as usize] = byte;
            self.pos += 1;
            true
        }
    }

    pub fn to_data(&self) -> [u32; 16] {
        let ptr = self.data.as_ptr();
        assert_eq!(ptr as usize % U32_ALIGN, 0);
        let mut res = unsafe { *(ptr as *const u32 as *const [u32; 16]) };
        memory_le_to_be(&mut res);
        res
    }

    pub const fn empty() -> Self {
        Self { data: [0u8; 64], pos: 0 }
    }

    pub fn clear(&mut self) {
        *self = Self::empty();
    }

    pub fn is_full(&self) -> bool {
        self.pos == Self::BUF_SIZE
    }
    pub fn is_empty(&self) -> bool {
        self.pos == 0
    }
}

#[inline(always)]
pub fn memory_le_to_be(_slice: &mut [u32]) {
    #[cfg(target_endian = "little")]
    {
        for byte in _slice.iter_mut() {
            *byte = byte.to_be();
        }
    }
}

impl Default for Vec64 {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Debug for Vec64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_trait_builder = f.debug_struct("Vec64");
        debug_trait_builder.field("data", &(&self.data[..]));
        debug_trait_builder.field("pos", &(self.pos));
        debug_trait_builder.finish()
    }
}

#[rustfmt::skip]
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
];


pub trait MutArithmetics {
    fn wrapping_add_mut(&mut self, rhs: u32);
}

impl MutArithmetics for u32 {
    #[inline(always)]
    fn wrapping_add_mut(&mut self, rhs: u32) {
        *self = self.wrapping_add(rhs);
    }
}

impl Default for Sha256 {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha2_test_vectors() {
        assert!(test_vec(b"abc", [0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61, 0xf20015ad]));
        assert!(test_vec(b"", [0xe3b0c442, 0x98fc1c14, 0x9afbf4c8, 0x996fb924, 0x27ae41e4, 0x649b934c, 0xa495991b, 0x7852b855]));
        assert!(test_vec(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", [0x248d6a61, 0xd20638b8, 0xe5c02693, 0x0c3e6039, 0xa33ce459, 0x64ff2167, 0xf6ecedd4, 0x19db06c1]));
        assert!(test_vec(b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
                         [0xcf5b16a7, 0x78af8380, 0x036ce59e, 0x7b049237, 0x0b249b11, 0xe8f07a51, 0xafac4503, 0x7afee9d1]));
        assert!(test_vec(&[b'a'; 1_000_000], [0xcdc76e5c, 0x9914fb92, 0x81a1c7e2, 0x84d73e67, 0xf1809a48, 0xa497200e, 0x046d39cc, 0xc7112cd0]));

        let mut hash = Sha256::new();
        for _ in 0..16_777_216 {
            hash.input(b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno");
        }
        assert_eq!(hash.finalize_internal(), [0x50e72a0e, 0x26442fe2, 0x552dc393, 0x8ac58658, 0x228c0cbf, 0xb1d2ca87, 0x2ae43526, 0x6fcd055e])
    }

    fn test_vec(input: &[u8], res: [u32; 8]) -> bool {
        let mut hash = Sha256::new();
        hash.input(input);
        let input = hash.finalize_internal();
        input == res
    }
}



#[cfg(all(test, feature = "nightly"))]
mod benches {
    extern crate test;
    use self::test::{black_box, Bencher};
    use super::*;
    const MiB: usize = 1024 * 1024;

    #[bench]
    pub fn my_sha2(bh: &mut Bencher) {
        let mut data = [01u8; MiB];
        bh.iter(|| {
            let mut hash = Sha256::new();
            hash.input(&data);
            black_box(hash.finalize());
        });
    }
}