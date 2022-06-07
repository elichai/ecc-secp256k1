#![cfg_attr(feature = "nightly", feature(test))]

mod field;
mod hash;
pub mod internal;
mod jacobi;
mod point;
mod secp256k1;
// mod u256;
mod ffi;
#[cfg(test)]
mod test_vectors;

pub use crate::secp256k1::{PrivateKey, PublicKey, SchnorrSignature, Signature};
pub use hash::*;

pub use crate::ffi::{ecdsa::*, schnorr::*, *};

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn it_works() {
        let s1: BigInt = BigInt::from(2u32).pow(32);
        let s2: BigInt = BigInt::from(2u32).pow(31);
        let s = s1 + s2;

        let privkey = PrivateKey::new(s);
        let pubkey = privkey.generate_pubkey();
        println!("{}", pubkey);
        println!("{:?}", &pubkey.clone().uncompressed()[..]);
        println!("{:?}", &pubkey.compressed()[..]);
    }
}
