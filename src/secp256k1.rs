use rug::{Integer, integer::Order};
use crate::field::FieldElement;
use crate::point::{Point, Group};
use crate::hash::{HashDigest, HashTrait};
use std::{fmt, ops::Deref, io::Read};
use rand::rngs::OsRng;
use rand::Rng;

#[derive(Clone, PartialEq, Debug)]
pub struct Secp256k1 {
    pub modulo: Integer,
    pub order: Integer,
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
        let n: Integer = Self::n.parse().unwrap();
        let a = Integer::from(Self::a);
        let b = Integer::from(Self::b);
        let group = Group { a, b };
        let point = Point::new_with_group(x, y, &p, group).unwrap();
        Secp256k1 { generator: point, modulo: p, order: n }
    }

    pub fn generator(&self) -> Point {
        self.generator.clone()
    }

    pub fn get_fe(&self, num: &[u8]) -> FieldElement {
        FieldElement::from_serialize(&num, &self.modulo)
    }

    pub fn get_pubkey(&self, x: &[u8], y: &[u8]) -> PublicKey {
        let x = FieldElement::from_serialize(x, &self.modulo);
        let y = FieldElement::from_serialize(y, &self.modulo);
        let point = Point { x, y, group: self.generator.group.clone() };
        if !point.is_on_curve() {
            unimplemented!();
        }
        PublicKey { point, _secp: self.clone() }
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
        if !point.is_on_curve() {
            unimplemented!();
        }
        PublicKey { point, _secp: secp }
    }

    pub fn from_compressed(ser: &[u8]) -> PublicKey {
        let secp = Secp256k1::new();
        let x = FieldElement::from_serialize(&ser[1..33], &secp.modulo);
        let mut y = secp.generator.group.get_y(&x);
        let is_even = y.is_even();
        if (ser[0] == 0x02 && !is_even) || (ser[0] == 0x03 && is_even) {
            y = secp.modulo.clone() - y;
        } else if ser[0] != 0x02 && ser[0] != 0x03 {
            unimplemented!()
        }
        let point = Point { x,y, group: secp.generator.group.clone() };
        PublicKey { point, _secp: secp }
    }

    // TODO: Maxwell's trick: https://github.com/bitcoin-core/secp256k1/blob/abe2d3e/src/ecdsa_impl.h#L238-L253
    #[allow(non_snake_case)]
    pub(crate) fn verify_raw(&self, z: FieldElement, r: FieldElement, s: FieldElement) -> bool {
        let G = self._secp.generator();
        let u1 = z / &s;
        let u2 = r.clone() / &s;
        let point: Point = (u1.num * G) + (u2.num * self.point.clone());
        point.x.num == r.num // Sometimes r.num is only 31 bytes. need to take a closer look.   
    }

    pub fn verify(&self, msg: &[u8], sig: Signature) -> bool {
        let order = &self._secp.order;
        let msg_hash = msg.hash_digest();
        println!("{:?}", msg_hash);
        let z = FieldElement::from_serialize(&msg_hash, order);
        let r = FieldElement::from_serialize(&sig.r.0, order);
        let s = FieldElement::from_serialize(&sig.s.0, order);
        self.verify_raw(z, r, s)
    }
}

impl PrivateKey {
    const ORDER: Order = Order::MsfLe;
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
        let mut hash = HashDigest::default();
        hash.input(&[y]);
        hash.input(&x);
        let mut result = [0u8; 32];
        result.copy_from_slice(&mut hash.result());
        result
    }

    pub(crate) fn sign_raw(&self, mut k: FieldElement, z: FieldElement) -> Signature {
        let k_point: Point = k.num.clone() * self._secp.generator();
        let order = &self._secp.order;
        let mut r = k_point.x;
        r.modulo = order.clone();
        k.modulo = order.clone();
        r.mod_num().round_mod();
        let mut s: FieldElement = (z + (r.clone()*&self.scalar)) / k;
        if s.num > Integer::from(order/2) {
            s = order - s;
        }
        if r.is_zero() || s.is_zero() {
            unimplemented!();
        }

        Signature::new(&r.serialize_num(), &s.serialize_num())

    }

    // TODO: Recovery ID
    pub fn sign(&self, msg: &[u8]) -> Signature {
        let k: [u8; 32] = OsRng::new().unwrap().gen();
        let msg_hash = msg.hash_digest();
        let k = FieldElement::from_serialize(&k, &self._secp.modulo);
        let z = FieldElement::from_serialize(&msg_hash, &self._secp.order);
        self.sign_raw(k, z)
    }

    pub fn from_serialized(ser: &[u8]) -> PrivateKey {
        let i = Integer::from_digits(ser, Order::MsfLe);
        PrivateKey::new(i)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Signature {
    r: Scalar,
    s: Scalar,
}

impl Signature {
    const START: u8 = 0x30;
    const MARKER: u8 = 0x02;
    pub(crate) fn new(r: &[u8], s: &[u8]) -> Signature {
        Signature {
            r: Scalar::new(r),
            s: Scalar::new(s),
        }
    }

    pub fn serialize(&self) -> [u8; 64] {
        let mut result = [0u8; 64];
        result[..32].copy_from_slice(&self.r.0);
        result[32..].copy_from_slice(&self.s.0);
        result
    }

    pub fn parse(sig: [u8; 64]) -> Signature {
        Signature {
            r: Scalar::new(&sig[..32]),
            s: Scalar::new(&sig[32..]),
        }
    }

    pub fn serialize_der(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(72);
        let r_start = self.r.iter().position(|x| *x!=0).unwrap();
        let s_start = self.s.iter().position(|x| *x!=0).unwrap();
        let r = &self.r[r_start..];
        let s = &self.s[s_start..];
        let data_length = r.len() + s.len() + 4; // 4 =  2 markers + 2 lengths. (res.len() - start - data_length)

        res.push(Self::START);
        res.push(data_length as u8);

        res.push(Self::MARKER);
        res.push(r.len() as u8);
        res.extend_from_slice(r);

        res.push(Self::MARKER);
        res.push(s.len() as u8);
        res.extend_from_slice(s);
        res
    }

    pub fn parse_der(sig: &[u8]) -> Signature {
        fn take<R: Read>(reader: &mut R) -> u8 {
            let mut b = [0];
            reader.read(&mut b).unwrap();
            b[0]
        }
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        let mut reader = sig;
        if take(&mut reader) != Self::START {
            unimplemented!();
        }
        let data_length = take(&mut reader) as usize;

        if take(&mut reader) != Self::MARKER {
            unimplemented!();
        }
        let r_length = take(&mut reader) as usize;
        reader.read_exact(&mut r[32-r_length..]).unwrap();

        if take(&mut reader) != Self::MARKER {
            unimplemented!();
        }
        let s_length = take(&mut reader) as usize;
        reader.read_exact(&mut s[32-s_length..]).unwrap();

        if data_length != r_length + s_length + 4 {
            unimplemented!();
        }

        Signature {
            r: Scalar(r),
            s: Scalar(s),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
struct Scalar(pub [u8;32]);


impl Scalar {
    pub fn new(slice: &[u8]) -> Scalar {
        let mut res = Scalar::default();
        res.0.copy_from_slice(slice);
        res
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

impl Deref for Scalar {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Scalar {
    fn as_ref(&self) -> &[u8] {
        &self.0
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

    #[test]
    fn test_sign_verify() {
        let priv_key = PrivateKey::new(8764321234_u128);
        let pub_key = priv_key.generate_pubkey();

        let msg = b"Liberta!";
        let sig = priv_key.sign(msg);
        assert!(pub_key.verify(msg, sig));
    }

    #[test]
    fn test_sign_der() {
        let priv_key = PrivateKey::new(8764321234_u128);
        let msg = b"Liberta!";
        let sig = priv_key.sign(msg);
        let der = sig.serialize_der();
        assert_eq!(sig, Signature::parse_der(&der));
    }
}