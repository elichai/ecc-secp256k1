use crate::field::*;
use num_bigint::BigInt;
use std::{fmt, ops::*};

#[derive(Clone, PartialEq)]
pub struct Group {
    pub a: BigInt,
    pub b: BigInt,
}

impl Group {
    pub fn new<I: Into<BigInt>, T: Into<BigInt>>(a: I, b: T) -> Self {
        Self { a: a.into(), b: b.into() }
    }

    pub fn get_y(&self, x: &FieldElement) -> FieldElement {
        let mut y2 = x.clone().pow_u(3u32) + (self.a.clone() * x) + &self.b; // Y^2 = X^3 + ax + b
        y2.sqrt();
        y2
    }
}

#[derive(PartialEq, Clone)]
pub struct Point {
    pub x: FieldElement,
    pub y: FieldElement,
    pub group: Group,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point {{ x: {}, y: {} }}", self.x.clone(), self.y.clone())
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Point {
    pub fn new<I, T, V>(x: I, y: T, modulo: V) -> Result<Self, ()>
    where
        I: Into<BigInt>,
        T: Into<BigInt>,
        V: Into<BigInt>,
    {
        let group = Group::new(0u32, 7u32);
        Self::new_with_group(x, y, modulo, group)
    }

    pub fn new_with_group<I, T, V>(x: I, y: T, modulo: V, group: Group) -> Result<Self, ()>
    where
        I: Into<BigInt>,
        T: Into<BigInt>,
        V: Into<BigInt>,
    {
        let x = FieldElement::new(x, modulo);
        let y = FieldElement::new(y, x.modulo.clone());
        let point = Self { x, y, group };
        if !point.is_on_curve() {
            Err(())
        } else {
            Ok(point)
        }
    }

    pub fn new_serialized_with_group<I>(x: &[u8], y: &[u8], modulo: I, group: Group) -> Result<Self, ()>
    where
        I: Into<BigInt>,
    {
        let x = FieldElement::from_serialize(x, modulo);
        let y = FieldElement::from_serialize(y, x.modulo.clone());
        let point = Self { x, y, group };
        if !point.is_on_curve() {
            Err(())
        } else {
            Ok(point)
        }
    }

    pub fn gen_zero(&self) -> Self {
        let x = FieldElement::new(0u32, self.x.modulo.clone());
        let y = FieldElement::new(0u32, self.x.modulo.clone());
        Self { x, y, group: self.group.clone() }
    }

    #[inline(always)]
    pub fn is_on_curve(&self) -> bool {
        self.y.clone().pow_u(2u32) == (self.x.clone().pow_u(3u32) + self.group.a.clone() * self.x.clone() + &self.group.b)
        // Y^2 = X^3 + ax + b
    }

    #[inline(always)]
    pub fn is_on_infinity(&self) -> bool {
        self.x.is_infinity() || self.y.is_infinity()
    }

    #[inline(always)]
    fn get_slope(&self, other: &Point) -> FieldElement {
        if self.x != other.x {
            (self.y.clone() - other.y.clone()) / (self.x.clone() - other.x.clone())
        } else {
            (3u32 * self.x.clone().pow_u(2u32) + &self.group.a) / (2u32 * self.y.clone())
        }
    }
}

impl Add for Point {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        let inf = FieldElement::infinity(self.x.modulo.clone());
        if self.x.is_infinity() {
            other
        } else if other.x.is_infinity() {
            self
        } else if self.x == other.x && self.y != other.y {
            Self { x: inf.clone(), y: inf, group: self.group }
        } else if self == other && self.y.is_zero() {
            Self { x: inf.clone(), y: inf, group: self.group }
        } else {
            let m = self.get_slope(&other); // Returns the slope of the line
            let x = m.clone().pow_u(2u32) - &self.x - other.x; // takes the slope to the power of 2 minus both X's
            let y = m * (self.x - &x) - self.y; // negative of y-y1=m(x-x1) - Simple line equation
            Self { x, y, group: self.group }
        }
    }
}
macro_rules! mul_impl_point {
    ($($t:ty)*) => ($(
       impl Mul<$t> for Point {
            type Output = Point;
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline(always)]
            fn mul(self, mut other: $t) -> Self {
                let mut result = self.gen_zero();
                let mut adding = self.clone();
                while other != <$t>::from(0u8) {
                    if (other.clone() & <$t>::from(1u8)) == <$t>::from(1u8) {
                        result = result + adding.clone();
                    }
                    adding = adding.clone() + adding;
                    other >>= 1;
                }
                result
            }
        }
        impl Mul<&$t> for Point {
            type Output = Point;
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline(always)]
            fn mul(self, other: &$t) -> Self {
                self.mul(other.clone())
            }
        }
        impl Mul<Point> for $t {
            type Output = Point;
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline(always)]
            fn mul(self, other: Point) -> Point {
                other.mul(self)
            }
        }
        impl Mul<Point> for &$t {
            type Output = Point;
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline(always)]
            fn mul(self, other: Point) -> Point {
                other.mul(self)
            }
        }
        impl Mul<&Point> for $t {
            type Output = Point;
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline(always)]
            fn mul(mut self, other: &Point) -> Point {
                let mut result = other.gen_zero();
                let mut adding = other.clone();
                while self != <$t>::from(0u8) {
                    if (self.clone() & <$t>::from(1u8)) == <$t>::from(1u8) {
                        result = result.clone() + adding.clone();
                    }
                    adding = adding.clone() + adding;
                    self >>= 1;
                }
                result
            }
        }
        impl Mul<&Point> for &$t {
            type Output = Point;
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline(always)]
            fn mul(self, other: &Point) -> Point {
                let mut s = self.clone();
                let mut result = other.gen_zero();
                let mut adding = other.clone();
                while s != <$t>::from(0u8) {
                    if (s.clone() & <$t>::from(1u8)) == <$t>::from(1u8) {
                        result = result.clone() + adding.clone();
                    }
                    adding = adding.clone() + adding;
                    s >>= 1;
                }
                result
            }
        }
        )*)
}

mul_impl_point! { usize u8 u16 u32 u64 u128 BigInt }
