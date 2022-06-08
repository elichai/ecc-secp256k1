use crate::field::FieldElement;
use crate::hash::{HashDigest, HashTrait};
use crate::hmac_sha2::{HmacSha256, HmacSha256Drbg};
use crate::jacobi;
use crate::jacobi::Jacobi;
use crate::point::{Group, Point};
use num_bigint::{BigInt, Sign};
use std::{
    fmt,
    io::{BufReader, Read},
    ops::Deref,
    sync::Once,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Secp256k1 {
    pub modulo: BigInt,
    pub order: BigInt,
    generator: Point,
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
        let x: BigInt = Self::Gx.parse().unwrap();
        let y: BigInt = Self::Gy.parse().unwrap();
        let p: BigInt = Self::p.parse().unwrap();
        let n: BigInt = Self::n.parse().unwrap();
        let a = BigInt::from(Self::a);
        let b = BigInt::from(Self::b);
        let group = Group { a, b };
        let point = Point::new_with_group(x, y, p.clone(), group).unwrap();
        Secp256k1 { generator: point, modulo: p, order: n }
    }

    pub fn generator(&self) -> Point {
        self.generator.clone()
    }

    pub fn get_fe(&self, num: &[u8]) -> FieldElement {
        FieldElement::from_serialize(num, self.modulo.clone())
    }

    pub fn get_pubkey(&self, x: &[u8], y: &[u8]) -> PublicKey {
        let x = FieldElement::from_serialize(x, self.modulo.clone());
        let y = FieldElement::from_serialize(y, self.modulo.clone());
        let point = Point { x, y, group: self.generator.group.clone() };
        if !point.is_on_curve() {
            unimplemented!();
        }
        PublicKey { point }
    }

    // TODO: Hard code this.
    pub fn serialized_order(&self) -> [u8; 32] {
        let mut res = [0u8; 32];
        let (sign, serialized) = self.order.to_bytes_be();
        assert_ne!(sign, Sign::Minus);
        if serialized.len() > 32 {
            unimplemented!();
        }
        res[32 - serialized.len()..].copy_from_slice(&serialized);
        res
    }
}

pub struct PrivateKey {
    scalar: BigInt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PublicKey {
    point: Point,
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
        let x = self.point.x.serialize_num();
        result[1 + (32 - x.len())..].copy_from_slice(&x);
        result[0] = if self.point.y.is_even() { 0x02 } else { 0x03 };
        result
    }

    pub fn from_uncompressed(ser: &[u8]) -> PublicKey {
        let secp = get_context();
        if ser[0] != 0x04 {
            unimplemented!()
        }
        let x = FieldElement::from_serialize(&ser[1..33], secp.modulo.clone());
        let y = FieldElement::from_serialize(&ser[33..65], secp.modulo.clone());
        let point = Point { x, y, group: secp.generator.group.clone() };
        if !point.is_on_curve() {
            unimplemented!();
        }
        PublicKey { point }
    }

    pub fn from_compressed(ser: &[u8]) -> Result<PublicKey, &'static str> {
        let secp = get_context();
        let x = FieldElement::from_serialize(&ser[1..33], secp.modulo.clone());
        let mut y = secp.generator.group.get_y(&x);
        let is_even = y.is_even();
        if (ser[0] == 0x02 && !is_even) || (ser[0] == 0x03 && is_even) {
            y.reflect();
        } else if ser[0] != 0x02 && ser[0] != 0x03 {
            return Err("A compressed public key should start with 0x02/0x03");
        }
        let point = Point { x, y, group: secp.generator.group.clone() };
        if !point.is_on_curve() {
            return Err("The public key is not on the point"); // Could it even happen assuming I got the y?;
        }
        Ok(PublicKey { point })
    }

    // TODO: Maxwell's trick: https://github.com/bitcoin-core/secp256k1/blob/abe2d3e/src/ecdsa_impl.h#L238-L253
    #[allow(non_snake_case)]
    pub(crate) fn verify_raw(&self, z: FieldElement, r: FieldElement, s: FieldElement) -> bool {
        let G = get_context().generator();
        let u1 = z / &s;
        let u2 = r.clone() / &s;
        let point: Point = (u1.num * G) + (u2.num * self.point.clone());
        point.x.num == r.num // Sometimes r.num is only 31 bytes. need to take a closer look.
    }

    pub fn verify(&self, msg: &[u8], sig: Signature, to_hash: bool) -> bool {
        let order = &get_context().order;
        let msg_hash = get_hashed_message_if(msg, to_hash);
        let z = FieldElement::from_serialize(&msg_hash, order.clone());
        let r = FieldElement::from_serialize(&sig.r.0, order.clone());
        let s = FieldElement::from_serialize(&sig.s.0, order.clone());
        self.verify_raw(z, r, s)
    }

    #[allow(non_snake_case)]
    pub fn verify_schnorr(&self, msg: &[u8], sig: SchnorrSignature, to_hash: bool) -> bool {
        let m = get_hashed_message_if(msg, to_hash);
        let order = &get_context().order;
        let r = FieldElement::from_serialize(&sig.0.r.0, order.clone());
        let s = FieldElement::from_serialize(&sig.0.s.0, order.clone());

        let e = get_e(r.clone(), self.clone(), m);

        self.verify_schnorr_raw(e, r, s)
    }

    #[allow(non_snake_case)]
    pub(crate) fn verify_schnorr_raw(&self, mut e: FieldElement, r: FieldElement, s: FieldElement) -> bool {
        let G = get_context().generator();
        let p = &get_context().modulo;

        e.reflect();
        let R = (s.num * G) + e.num * &self.point;
        if R.is_on_infinity() {
            return false;
        }

        if jacobi::jacobi_symbol(R.y.num, p.clone()) != Jacobi::One {
            return false;
        }
        R.x.num == r.num
    }
}

impl PrivateKey {
    pub fn new<I: Into<BigInt>>(key: I) -> Self {
        PrivateKey { scalar: key.into() }
    }

    pub fn generate_pubkey(&self) -> PublicKey {
        let point = &self.scalar * get_context().generator();
        PublicKey { point }
    }

    pub fn ecdh(&self, pubkey: &PublicKey) -> [u8; 32] {
        let point: Point = &self.scalar * pubkey.point.clone();
        let x = point.x.serialize_num();
        let y = if point.y.is_even() { 0x02 } else { 0x03 };
        let mut hash = HashDigest::default();
        hash.input(&[y]);
        hash.input(&x);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash.result());
        result
    }

    pub(crate) fn sign_raw(d: &BigInt, k: FieldElement, z: FieldElement) -> Signature {
        let secp = get_context();
        let k_point: Point = &k.num * secp.generator();
        let order = &secp.order;
        let mut r = k_point.x;
        r.modulo = order.clone();
        r.mod_num().round_mod();
        let mut s: FieldElement = (z + (r.clone() * d)) / k;
        if s.num > order >> 1 {
            s = order - s;
        }
        if r.is_zero() || s.is_zero() {
            unimplemented!();
        }

        Signature::new(&r.serialize_num(), &s.serialize_num())
    }

    // TODO: Recovery ID
    pub fn sign(&self, msg: &[u8], to_hash: bool) -> Signature {
        let secp = get_context();
        let msg_hash = get_hashed_message_if(msg, to_hash);

        let k = self.deterministic_k_ecdsa(msg_hash);
        let z = FieldElement::from_serialize(&msg_hash, secp.order.clone());
        Self::sign_raw(&self.scalar, k, z)
    }

    fn deterministic_k_ecdsa(&self, m: [u8; 32]) -> FieldElement {
        let order = get_context().serialized_order();
        let mut state = HmacSha256Drbg::new(&self.serialize(), Some(&m));
        let mut nonce = [0u8; 32];
        state.generate(&mut nonce);

        while nonce >= order || nonce == [0u8; 32] {
            let mut tmp = HmacSha256::new(&state.k);
            tmp.input(&state.v);
            tmp.input(&[0]);
            state.k = tmp.finalize();
            state.v = HmacSha256::quick(&state.k, &state.v);

            state.generate(&mut nonce);
        }

        FieldElement::from_serialize(&nonce, get_context().order.clone())
    }

    #[allow(non_snake_case)]
    pub fn sign_schnorr(&self, msg: &[u8], to_hash: bool) -> SchnorrSignature {
        let m = get_hashed_message_if(msg, to_hash);
        let G = &get_context().generator;
        let order = &get_context().order;
        let p = &get_context().modulo;
        // Deterministic k, could be random.
        let mut k = self.deterministic_k_schnorr(m);
        let R = &k.num * G;
        if jacobi::jacobi_symbol(R.y.num.clone(), p.clone()) != Jacobi::One {
            k = order - k;
        }
        let e = get_e(R.x.clone(), self.generate_pubkey(), m);

        Self::sign_schnorr_raw(&self.scalar, k, e, Some(R))
    }

    fn deterministic_k_schnorr(&self, m: [u8; 32]) -> FieldElement {
        let order = &get_context().order;
        let d = self.serialize();
        let mut k = HashDigest::new();
        k.input(&d);
        k.input(&m);
        let k = k.result();
        let mut k = FieldElement::from_serialize(&k, order.clone());
        k.mod_num();
        // TODO: Check the Jacobi symbol and if not 1 subtract by the group order (https://en.wikipedia.org/wiki/Jacobi_symbol)
        if k.is_zero() {
            unimplemented!();
        }
        k
    }

    // TODO: Pass Rx instead of R.
    #[allow(non_snake_case)]
    pub(crate) fn sign_schnorr_raw(d: &BigInt, k: FieldElement, e: FieldElement, R: Option<Point>) -> SchnorrSignature {
        let R = R.unwrap_or_else(|| &k.num * get_context().generator());

        let s = k + e * d;
        let s = s.serialize_num();
        let r = R.x.serialize_num();
        SchnorrSignature::new(&r, &s)
    }

    fn serialize(&self) -> [u8; 32] {
        let mut res = [0u8; 32];
        let (sign, serialized) = self.scalar.to_bytes_be();
        assert_ne!(sign, Sign::Minus);
        if serialized.len() > 32 {
            unimplemented!();
        }
        res[32 - serialized.len()..].copy_from_slice(&serialized);
        res
    }

    // TODO: make [u8;32]
    pub fn from_serialized(ser: &[u8]) -> PrivateKey {
        let i = BigInt::from_bytes_be(Sign::Plus, ser);
        PrivateKey::new(i)
    }
}

#[allow(non_snake_case)]
fn get_e(xR: FieldElement, pubkey: PublicKey, msg: [u8; 32]) -> FieldElement {
    let secp = get_context();
    let mut e = HashDigest::new();
    e.input(&xR.serialize_num());
    e.input(&pubkey.compressed());
    e.input(&msg);
    FieldElement::from_serialize(&e.result(), secp.order.clone())
}

fn get_hashed_message_if(msg: &[u8], to_hash: bool) -> [u8; 32] {
    let mut msg_hash = [0u8; 32];
    if to_hash {
        msg_hash = msg.hash_digest();
    } else if msg.len() != 32 {
        unimplemented!();
    } else {
        msg_hash.copy_from_slice(msg);
    }
    msg_hash
}

#[derive(Debug, PartialEq, Eq)]
pub struct Signature {
    r: Scalar,
    s: Scalar,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SchnorrSignature(pub(crate) Signature);

impl SchnorrSignature {
    pub(crate) fn new(r: &[u8], s: &[u8]) -> SchnorrSignature {
        SchnorrSignature(Signature::new(r, s))
    }

    pub fn serialize(&self) -> [u8; 64] {
        self.0.serialize()
    }

    pub fn parse(sig: [u8; 64]) -> SchnorrSignature {
        SchnorrSignature(Signature::parse(sig))
    }

    pub fn parse_slice(sig: &[u8]) -> SchnorrSignature {
        SchnorrSignature(Signature::parse_slice(sig))
    }
}

impl Signature {
    const START: u8 = 0x30;
    const MARKER: u8 = 0x02;
    pub(crate) fn new(r: &[u8], s: &[u8]) -> Signature {
        Signature { r: Scalar::new(r), s: Scalar::new(s) }
    }

    pub fn serialize(&self) -> [u8; 64] {
        let mut result = [0u8; 64];
        result[..32].copy_from_slice(&self.r.0);
        result[32..].copy_from_slice(&self.s.0);
        result
    }

    pub fn parse(sig: [u8; 64]) -> Signature {
        Signature { r: Scalar::new(&sig[..32]), s: Scalar::new(&sig[32..]) }
    }

    pub fn parse_slice(sig: &[u8]) -> Signature {
        if sig.len() != 64 {
            panic!("Wrong sig length");
        }
        Signature { r: Scalar::new(&sig[..32]), s: Scalar::new(&sig[32..]) }
    }

    pub fn serialize_der(&self) -> Vec<u8> {
        fn generate_33_leading_zeros(a: &[u8]) -> [u8; 33] {
            let mut res = [0u8; 33];
            res[1..].copy_from_slice(a);
            res
        }
        let mut res = Vec::with_capacity(72);
        let r = generate_33_leading_zeros(&self.r);
        let s = generate_33_leading_zeros(&self.s);
        let mut r_start = r.iter().position(|x| *x != 0).unwrap();
        let mut s_start = s.iter().position(|x| *x != 0).unwrap();
        if r[r_start] >= 128 {
            r_start -= 1;
        }
        if s[s_start] >= 128 {
            s_start -= 1;
        }
        let r = &r[r_start..];
        let s = &s[s_start..];
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
            assert_eq!(reader.read(&mut b).unwrap(), 1);
            b[0]
        }
        let mut sum_size = 4;

        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        let mut reader = BufReader::new(sig);
        if take(&mut reader) != Self::START {
            unimplemented!();
        }
        let data_length = take(&mut reader) as usize;

        if take(&mut reader) != Self::MARKER {
            unimplemented!();
        }

        let mut r_length = take(&mut reader) as usize;
        sum_size += r_length;
        if r_length == 33 {
            assert_eq!(take(&mut reader), 0);
            r_length -= 1;
        }
        reader.read_exact(&mut r[32 - r_length..]).unwrap();

        if take(&mut reader) != Self::MARKER {
            unimplemented!();
        }

        let mut s_length = take(&mut reader) as usize;
        sum_size += s_length;
        if s_length == 33 {
            assert_eq!(take(&mut reader), 0);
            s_length -= 1;
        }
        reader.read_exact(&mut s[32 - s_length..]).unwrap();

        if data_length != sum_size {
            unimplemented!();
        }

        Signature { r: Scalar(r), s: Scalar(s) }
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
struct Scalar(pub [u8; 32]);

impl Scalar {
    pub fn new(slice: &[u8]) -> Scalar {
        let mut res = Scalar::default();
        let res_len = res.len();
        res.0[res_len - slice.len()..].copy_from_slice(slice);
        res
    }
}

static mut CONTEXT: Option<Secp256k1> = None;

pub fn get_context() -> &'static Secp256k1 {
    static INIT_CONTEXT: Once = Once::new();
    INIT_CONTEXT.call_once(|| unsafe {
        CONTEXT = Some(Default::default());
    });
    unsafe { CONTEXT.as_ref().unwrap() }
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
        PublicKey { point }
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
    use crate::test_vectors::{TestMode, TestVector, SCHNORR_VECTORS};

    #[test]
    fn test_compress_pubkey() {
        let privkey = PrivateKey::new(32432432u32);
        let pubkey = privkey.generate_pubkey();
        let compress = pubkey.clone().compressed();
        assert_eq!(PublicKey::from_compressed(&compress).unwrap(), pubkey);
    }

    #[test]
    fn test_uncompressed_pubkey() {
        let privkey = PrivateKey::new(32432432u32);
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
        let sig = priv_key.sign(msg, true);
        assert!(pub_key.verify(msg, sig, true));
    }

    #[test]
    fn test_sign_der() {
        let priv_key = PrivateKey::new(8764321234_u128);
        let msg = b"Liberta!";
        let sig = priv_key.sign(msg, true);
        let der = sig.serialize_der();
        assert_eq!(sig, Signature::parse_der(&der));
    }

    #[test]
    fn test_sign_verify_schnorr() {
        let priv_key = PrivateKey::new(532557312_u128);
        let pub_key = priv_key.generate_pubkey();

        let msg = b"HODL!";
        let sig = priv_key.sign_schnorr(msg, true);
        assert!(pub_key.verify_schnorr(msg, sig, true));
    }

    #[test]
    fn test_schnorr_vectors() {
        fn verify_only(test: &TestVector) {
            let pubkey = match PublicKey::from_compressed(&test.pk) {
                Ok(k) => k,
                Err(_) => {
                    assert_eq!(test.verify_result, false);
                    return;
                }
            };
            let msg = test.msg;
            let sig = SchnorrSignature::parse(test.sig);
            assert_eq!(test.verify_result, pubkey.verify_schnorr(&msg, sig, false));
        }
        fn sign_and_verify(test: &TestVector) {
            let privkey = PrivateKey::from_serialized(&test.sk);
            let m = test.msg;
            let sig = privkey.sign_schnorr(&m, false);

            let pubkey = match PublicKey::from_compressed(&test.pk) {
                Ok(k) => k,
                Err(_) => {
                    assert_eq!(test.verify_result, false);
                    return;
                }
            };
            let othersig = SchnorrSignature::parse(test.sig);

            assert_eq!(sig, othersig);
            assert_eq!(test.verify_result, pubkey.verify_schnorr(&m, othersig, false));
        }
        fn parse_pubkey_only(test: &TestVector) {
            assert_eq!(test.verify_result, PublicKey::from_compressed(&test.pk).is_ok());
        }

        for vec in &SCHNORR_VECTORS {
            match vec.mode {
                TestMode::All => sign_and_verify(vec),
                TestMode::VerifyOnly => verify_only(vec),
                TestMode::ParsePubkeyOnly => parse_pubkey_only(vec),
            };
        }
    }
}
