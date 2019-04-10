
use rug::Integer;
use crate::{Point, Group, FieldElement};
use std::fmt;
use sha2::{Sha256, Digest};

#[derive(Clone, PartialEq, Debug)]
struct Secp256k1 {
    pub modulo: Integer,
    generator: Point
}

impl Secp256k1 {
    #![allow(non_upper_case_globals)]
    const Gx: &'static str = "55066263022277343669578718895168534326250603453777594175500187360389116729240";
    const Gy: &'static str = "32670510020758816978083085130507043184471273380659243275938904335757337482424";
    pub const p: &'static str = "115792089237316195423570985008687907853269984665640564039457584007908834671663";

    const a: u8 = 0;
    const b: u8 = 7;
    const n: &'static str = "115792089237316195423570985008687907852837564279074904382605163141518161494337";

    #[allow(clippy::many_single_char_names)]
    pub fn new() -> Secp256k1 {
        let x: Integer = Self::Gx.parse().unwrap();
        let y: Integer = Self::Gy.parse().unwrap();
        let p: Integer = Self::p.parse().unwrap();
        let a = Integer::from(Self::a);
        let b = Integer::from(Self::b);
        let group = Group { a, b };
        let point = Point::new_with_group(x, y, &p, group).unwrap();
        Secp256k1 { generator: point, modulo: p }
    }

    pub fn generator(&self) -> Point {
        self.generator.clone()
    }
}

pub struct PrivateKey {
    scalar: Integer,
    _secp : Secp256k1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PublicKey {
    point: Point,
    _secp: Secp256k1,
}

impl PublicKey {
    pub fn uncompressed(self) -> [u8; 65] {
        let mut result = [0u8; 65];
        result[0] = 0x04;
        result[1..33].copy_from_slice(&self.point.x.serialize_num());
        result[33..65].copy_from_slice(&self.point.y.serialize_num());
        result
    }

    pub fn compressed(self) -> [u8; 33] {
        let mut result = [0u8; 33];
        result[1..].copy_from_slice(&self.point.x.serialize_num());
        result[0] = if self.point.y.is_even() {
            0x02
        } else {
            0x03
        };
        result
    }

    pub fn from_uncompressed(ser: &[u8]) -> PublicKey {
        let secp = Secp256k1::new();
        if ser[0] != 0x04 {
            unimplemented!()
        }
        let x = FieldElement::from_serialize(&ser[1..33], &secp.modulo);
        let y = FieldElement::from_serialize(&ser[33..65], &secp.modulo);
        let point = Point { x, y, group: secp.generator.group.clone() };
        PublicKey { point, _secp: secp }
    }

    pub fn from_compressed(ser: &[u8]) -> PublicKey {
        let secp = Secp256k1::new();
        let x = FieldElement::from_serialize(&ser[1..33], &secp.modulo);
        let mut y = secp.generator.group.get_y(&x);
        let is_even = y.is_even();
        if (ser[0] == 0x02 && !is_even) || (ser[0] == 0x03 && is_even) {
            println!("fdsfdsf");
            y = secp.modulo.clone() - y;
        } else if ser[0] != 0x02 && ser[0] != 0x03 {
            unimplemented!()
        }
        let point = Point { x,y, group: secp.generator.group.clone() };
        dbg!(point.is_on_curve());
        PublicKey { point, _secp: secp }

    }
}

impl PrivateKey {
    pub fn new<I: Into<Integer>>(key: I) -> Self {
        PrivateKey {scalar: key.into(), _secp: Default::default() }
    }

    pub fn generate_pubkey(&self) -> PublicKey {
        let point = &self.scalar * self._secp.generator();
        PublicKey { point, _secp: self._secp.clone() }

    }

    pub fn ecdh(&self, pubkey: &PublicKey) -> [u8; 32] {
        let point: Point = &self.scalar * pubkey.point.clone();
        let x = point.x.serialize_num();
        let y = if point.y.is_even() {
            0x02
        } else {
            0x03
        };
        let mut hash = Sha256::default();
        hash.input(&[y]);
        hash.input(&x);
        let mut result = [0u8; 32];
        result.copy_from_slice(&mut hash.result());
        result
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Public: {{ X: {:#X}, Y: {:#X} }}", self.point.x.inner(), self.point.y.inner())
    }
}

impl fmt::Display for Secp256k1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Generator {{ X: {:#X}, Y: {:#X} }}", self.generator.x.inner(), self.generator.y.inner())
    }
}

impl From<Point> for PublicKey {
    fn from(point: Point) -> PublicKey {
        PublicKey{ point, _secp: Default::default() }
    }
}


impl Default for Secp256k1 {
    fn default() -> Secp256k1 {
        Secp256k1::new()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compress_pubkey() {
        let privkey = PrivateKey::new(32432432);
        let pubkey = privkey.generate_pubkey();
        let compress = pubkey.clone().compressed();
        assert_eq!(PublicKey::from_compressed(&compress), pubkey);
    }

    #[test]
    fn test_uncompressed_pubkey() {
        let privkey = PrivateKey::new(32432432);
        let pubkey = privkey.generate_pubkey();
        let compress = pubkey.clone().uncompressed();
        assert_eq!(PublicKey::from_uncompressed(&compress), pubkey);
    }

    #[test]
    fn test_ecdh() {
        let priv_key1 = PrivateKey::new(8764321234_u128);
        let pub_key1 = priv_key1.generate_pubkey();
        let priv_key2 = PrivateKey::new(49234078927865834890_u128);
        let pub_key2 = priv_key2.generate_pubkey();

        let ecdh1 = priv_key1.ecdh(&pub_key2);
        let ecdh2 = priv_key2.ecdh(&pub_key1);
        assert_eq!(ecdh1, ecdh2);
    }
}