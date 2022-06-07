use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use std::{fmt, ops::*};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FieldElement {
    pub num: BigInt,
    pub modulo: BigInt,
}

impl fmt::Display for FieldElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.num)
    }
}

impl FieldElement {
    pub fn new<I: Into<BigInt>, T: Into<BigInt>>(num: I, modulo: T) -> FieldElement {
        let mut res = FieldElement { num: num.into(), modulo: modulo.into() };
        res.mod_num();
        res
    }

    pub fn infinity(modulo: BigInt) -> FieldElement {
        Self { num: 0u32.into(), modulo } // Right now I'm representing Infinity as (0,0).
    }

    pub fn is_infinity(&self) -> bool {
        self == &Self::infinity(self.modulo.clone())
    }
    pub fn is_zero(&self) -> bool {
        self.num == BigInt::from(0u32)
    }

    pub fn pow_u<I: Into<BigInt>>(self, other: I) -> FieldElement {
        let num = self.num.modpow(&other.into(), &self.modulo);
        FieldElement { num, modulo: self.modulo }
    }

    #[inline(always)]
    pub fn pow(self, other: FieldElement) -> FieldElement {
        self.same_modulo(&other);
        let num = self.num.modpow(&other.num, &self.modulo);
        FieldElement { num, modulo: self.modulo }
    }

    #[inline(always)]
    pub fn sqrt(&mut self) {
        let mut p: BigInt = self.modulo.clone() + 1u32;
        p /= 4u32;
        self.num = self.num.modpow(&p, &self.modulo);
    }

    #[inline(always)]
    fn same_modulo(&self, other: &Self) {
        if self.modulo != other.modulo {
            unimplemented!();
        }
    }
    #[inline(always)]
    pub fn round_mod(&mut self) {
        if self.num.sign() == Sign::Minus {
            self.num += &self.modulo;
        }
    }

    pub fn mod_num(&mut self) -> &mut Self {
        self.num = self.num.mod_floor(&self.modulo);
        self
    }

    #[inline(always)]
    pub fn inner(&self) -> &BigInt {
        &self.num
    }

    #[inline(always)]
    pub fn reflect(&mut self) {
        self.num = &self.modulo - &self.num;
    }

    pub fn serialize_num(self) -> [u8; 32] {
        let mut res = [0u8; 32];
        let (sign, serialized) = self.num.to_bytes_be();
        assert_ne!(sign, Sign::Minus);
        if serialized.len() > 32 {
            unimplemented!();
        }
        res[32 - serialized.len()..].copy_from_slice(&serialized);
        res
    }

    pub fn from_serialize<I: Into<BigInt>>(ser: &[u8], modulo: I) -> FieldElement {
        let num = BigInt::from_bytes_be(Sign::Plus, ser);
        FieldElement { num, modulo: modulo.into() }
    }

    pub fn is_even(&self) -> bool {
        self.num.is_even()
    }
}

#[inline(always)]
pub fn mod_and_new(num: BigInt, modulo: &BigInt) -> FieldElement {
    let num = num % modulo;
    let mut res = FieldElement { num, modulo: modulo.clone() };
    res.round_mod();
    res
}

impl Add for FieldElement {
    type Output = FieldElement;
    #[inline(always)]
    fn add(self, other: Self) -> FieldElement {
        self.same_modulo(&other);
        let num = (self.num + other.num) % &self.modulo;
        let mut res = FieldElement { num, modulo: self.modulo };
        res.round_mod();
        res
    }
}
impl Add<&FieldElement> for FieldElement {
    type Output = FieldElement;
    #[inline(always)]
    fn add(self, other: &Self) -> FieldElement {
        self.same_modulo(other);
        let num = (self.num + &other.num) % &self.modulo;
        let mut res = FieldElement { num, modulo: self.modulo };
        res.round_mod();
        res
    }
}

impl Sub for FieldElement {
    type Output = FieldElement;
    #[inline(always)]
    fn sub(self, other: Self) -> FieldElement {
        self.same_modulo(&other);
        let num = self.num - other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Sub<&FieldElement> for FieldElement {
    type Output = FieldElement;
    #[inline(always)]
    fn sub(self, other: &Self) -> FieldElement {
        self.same_modulo(other);
        let num = self.num - &other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Mul for FieldElement {
    type Output = FieldElement;
    #[inline(always)]
    fn mul(self, other: Self) -> FieldElement {
        self.same_modulo(&other);
        let num = self.num * other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Mul<&FieldElement> for FieldElement {
    type Output = FieldElement;
    #[inline(always)]
    fn mul(self, other: &Self) -> FieldElement {
        self.same_modulo(other);
        let num = self.num * &other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Div for FieldElement {
    type Output = FieldElement;
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline(always)]
    fn div(self, other: Self) -> FieldElement {
        self.div(&other)
    }
}

impl Div<&FieldElement> for FieldElement {
    type Output = FieldElement;
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline(always)]
    fn div(self, other: &Self) -> FieldElement {
        let mut other = other.clone();
        self.same_modulo(&other);
        let p = &self.modulo - 2u32;
        other.num = other.num.modpow(&p, &self.modulo);
        let mut res = self * other;
        res.round_mod();
        res
    }
}

macro_rules! mul_impl_field {
    ($($t:ty)*) => ($(
        impl Mul<$t> for FieldElement {
        type Output = FieldElement;
            #[inline]
            fn mul(self, other: $t) -> FieldElement {
                let other: BigInt = other.into();
                let num = self.num * other;
                mod_and_new(num, &self.modulo)
            }
        }

        impl Mul<&$t> for FieldElement {
        type Output = FieldElement;

            #[inline]
            fn mul(self, other: &$t) -> FieldElement {
                let other: BigInt = BigInt::from(other.clone());
                let num = self.num * other;
                mod_and_new(num, &self.modulo)
            }
        }

         impl Mul<FieldElement> for $t {
            type Output = FieldElement;

            #[inline]
            fn mul(self, other: FieldElement) -> FieldElement {
                other.mul(self)
            }
        }

         impl Mul<&FieldElement> for $t {
            type Output = FieldElement;

            #[inline]
            fn mul(self, other: &FieldElement) -> FieldElement {
                let s: BigInt = self.into();
                let num = &other.num * s;
                mod_and_new(num, &other.modulo)
            }
        }
    )*)
}

macro_rules! add_impl_field {
    ($($t:ty)*) => ($(
       impl Add<$t> for FieldElement {
       type Output = FieldElement;
            fn add(self, other: $t) -> FieldElement {
                let other = FieldElement::new(other, self.modulo.clone());
                self + other
            }
        }
       impl Add<FieldElement> for $t {
       type Output = FieldElement;
            fn add(self, other: FieldElement) -> FieldElement {
                other + self
            }
        }
       impl Add<&$t> for FieldElement {
       type Output = FieldElement;
            fn add(self, other: &$t) -> FieldElement {
                let other = FieldElement::new(other.clone(), self.modulo.clone());
                self + other
            }
        }
       impl Add<&FieldElement> for $t {
       type Output = FieldElement;
            fn add(self, other: &FieldElement) -> FieldElement {
                let s = FieldElement::new(self, other.modulo.clone());
                s + other
            }
        }
    )*)
}

macro_rules! sub_impl_field {
    ($($t:ty)*) => ($(
       impl Sub<$t> for FieldElement {
       type Output = FieldElement;
            fn sub(self, other: $t) -> FieldElement {
                let other = FieldElement::new(other, self.modulo.clone());
                self - other
             }
        }
       impl Sub<FieldElement> for $t {
       type Output = FieldElement;
            fn sub(self, other: FieldElement) -> FieldElement {
                let s = FieldElement::new(self, other.modulo.clone());
                s - other
            }
        }
       impl Sub<FieldElement> for &$t {
       type Output = FieldElement;
            fn sub(self, other: FieldElement) -> FieldElement {
                let s = FieldElement::new(self.clone(), other.modulo.clone());
                s - other
            }
        }
       impl Sub<&$t> for FieldElement {
       type Output = FieldElement;
            fn sub(self, other: &$t) -> FieldElement {
                let other = FieldElement::new(other.clone(), self.modulo.clone());
                self - other
            }
        }
       impl Sub<&FieldElement> for $t {
       type Output = FieldElement;
            fn sub(self, other: &FieldElement) -> FieldElement {
                let s = FieldElement::new(self, other.modulo.clone());
                s - other
            }
        }
    )*)
}

macro_rules! div_impl_field {
    ($($t:ty)*) => ($(
       impl Div<$t> for FieldElement {
       type Output = FieldElement;
            fn div(self, other: $t) -> FieldElement {
                let other = FieldElement::new(other, self.modulo.clone());
                self / other
            }
        }
        impl Div<&$t> for FieldElement {
        type Output = FieldElement;
            fn div(self, other: &$t) -> FieldElement {
                let other = FieldElement::new(other.clone(), self.modulo.clone());
                self / other
            }
        }
       impl Div<FieldElement> for $t {
       type Output = FieldElement;
            fn div(self, other: FieldElement) -> FieldElement {
                let s = FieldElement::new(self, other.modulo.clone());
                s / other
            }
        }
       impl Div<&FieldElement> for $t {
       type Output = FieldElement;
            fn div(self, other: &FieldElement) -> FieldElement {
                let s = FieldElement::new(self, other.modulo.clone());
                s / other
            }
        }
    )*)
}

mul_impl_field! { usize u8 u16 u32 u64 u128 BigInt }
add_impl_field! { usize u8 u16 u32 u64 u128 BigInt }
sub_impl_field! { usize u8 u16 u32 u64 u128 BigInt }
div_impl_field! { usize u8 u16 u32 u64 u128 BigInt }
