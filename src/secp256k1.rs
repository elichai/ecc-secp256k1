
use rug::Integer;
use crate::{Point, Group, FieldElement};
use std::fmt;


struct Secp256k1 {
    generator: Point
}

impl Secp256k1 {
    #![allow(non_upper_case_globals)]
    const Gx: &'static str = "55066263022277343669578718895168534326250603453777594175500187360389116729240";
    const Gy: &'static str = "32670510020758816978083085130507043184471273380659243275938904335757337482424";
    const p: &'static str = "115792089237316195423570985008687907853269984665640564039457584007908834671663";

    const a: u8 = 0;
    const b: u8 = 7;
    const n: &'static str = "115792089237316195423570985008687907852837564279074904382605163141518161494337";

    pub fn new() -> Secp256k1 {
        let x: Integer = Self::Gx.parse().unwrap();
        let y: Integer = Self::Gy.parse().unwrap();
        let p: Integer= Self::p.parse().unwrap();
        let a = Integer::from(Self::a);
        let b = Integer::from(Self::b);
        let group = Group { a, b };
        let point = Point::new_with_group(x, y, p, group).unwrap();
        Secp256k1 { generator: point }
    }

    pub fn generator(&self) -> Point {
        self.generator.clone()
    }
}

pub struct PrivateKey {
    scalar: Integer,
}


pub struct PublicKey {
    point: Point,
}

impl PrivateKey {
    pub fn new<I: Into<Integer>>(key: I) -> Self {
        PrivateKey {scalar: key.into() }
    }

    pub fn generate_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        println!("{}", secp);
        let point = &self.scalar * secp.generator();
        PublicKey { point }

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
        PublicKey{ point }
    }
}
