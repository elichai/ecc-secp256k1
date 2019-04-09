mod field;
mod point;
mod secp256k1;

use self::point::*;
use self::field::*;
use self::secp256k1::*;



fn main() {
    let privkey = PrivateKey::new(999);
    println!("{}",privkey.generate_pubkey());
}
