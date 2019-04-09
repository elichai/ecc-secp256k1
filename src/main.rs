mod field;
mod point;
mod secp256k1;

use self::point::*;
use self::field::*;
use self::secp256k1::*;
use rug::Integer;


fn main() {
    let s1: Integer = Integer::u_pow_u(2,32).into();
    let s2: Integer = Integer::u_pow_u(2,31).into();
    let s = s1 + s2;

    let privkey = PrivateKey::new(s);
    let pubkey = privkey.generate_pubkey();
    println!("{}", pubkey);
    println!("{:?}", &pubkey.clone().uncompressed()[..]);
    println!("{:?}", &pubkey.compressed()[..]);
}
