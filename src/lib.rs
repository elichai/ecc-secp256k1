mod field;
mod hash;
pub mod internal;
mod point;
mod secp256k1;
mod u256;

mod test_rust_secp256k1;

pub use crate::secp256k1::{PrivateKey, PublicKey, Signature};

#[cfg(test)]
mod tests {
    use super::*;
    use rug::Integer;

    #[test]
    fn it_works() {
        let s1: Integer = Integer::u_pow_u(2, 32).into();
        let s2: Integer = Integer::u_pow_u(2, 31).into();
        let s = s1 + s2;

        let privkey = PrivateKey::new(s);
        let pubkey = privkey.generate_pubkey();
        println!("{}", pubkey);
        println!("{:?}", &pubkey.clone().uncompressed()[..]);
        println!("{:?}", &pubkey.compressed()[..]);
    }
}
