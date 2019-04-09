use std::{ops::*, fmt};
use rug::{Integer, integer::Order, ops::NegAssign};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FieldElement {
    pub num: Integer,
    pub modulo: Integer,
}

impl fmt::Display for FieldElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.num)
    }
}

impl FieldElement {
    const ORDER: Order = Order::MsfLe;
    pub fn new<I: Into<Integer>, T: Into<Integer>>(num: I, modulo: T) -> FieldElement {
        FieldElement { num: num.into(), modulo: modulo.into() }
    }

    pub fn infinity<I: Into<Integer>>(modulo: I) -> FieldElement {
        Self { num: 0.into(), modulo: modulo.into() } // Right now I'm representing Infinity as (0,0).
    }
    pub fn is_infinity(&self) -> bool {
        self == &Self::infinity(&self.modulo)
    }
    pub fn is_zero(&self) -> bool {
        self.num == 0
    }

    pub fn pow_u<I: Into<Integer>>(self, other: I) -> FieldElement {
        let num = (self.num.pow_mod(&other.into(), &self.modulo)).unwrap();
        FieldElement { num, modulo: self.modulo }
    }

    #[inline(always)]
    pub fn pow(self, other: FieldElement) -> FieldElement {
        self.same_modulo(&other);
        let num = (self.num.pow_mod(&other.num, &self.modulo)).unwrap();
        FieldElement { num, modulo: self.modulo }
    }

    #[inline(always)]
    pub fn sqrt(&mut self) {
        let mut p: Integer = self.modulo.clone() + 1;
        p.div_exact_mut(&4.into());
        self.num.pow_mod_mut(&p, &self.modulo).unwrap();
    }

    #[inline(always)]
    fn same_modulo(&self, other: &Self) {
        if self.modulo != other.modulo {
            unimplemented!();
        }
    }
    #[inline(always)]
    fn round_mod(&mut self) {
        if self.num < 0 {
            self.num += &self.modulo;
        }
    }
    #[inline(always)]
    pub fn inner(&self) -> &Integer {
        &self.num
    }

    #[inline(always)]
    pub fn reflect(&mut self) {
        self.num.neg_assign();
        self.round_mod();
        self.num = self.num.clone() & &self.modulo;
    }

    pub fn serialize_num(self) -> Vec<u8> {
        self.num.to_digits(Self::ORDER)
    }

    pub fn from_serialize<I: Into<Integer>>(ser: &[u8], modulo: I) -> FieldElement {
        let num = Integer::from_digits(ser, Self::ORDER);
        FieldElement { num, modulo: modulo.into() }
    }

    pub fn is_even(&self) -> bool {
        self.num.is_even()
    }
}

#[inline(always)]
pub fn mod_and_new(num: Integer, modulo: &Integer) -> FieldElement {
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
        self.same_modulo(&other);
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
        self.same_modulo(&other);
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
        self.same_modulo(&other);
        let num = self.num * &other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Div for FieldElement {
    type Output = FieldElement;
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline(always)]
    fn div(self, other: Self) -> FieldElement {
        let mut other = other;
        self.same_modulo(&other);
        let p = &self.modulo - Integer::from(2);
        other.num = other.num.pow_mod(&p, &self.modulo).unwrap();
        let mut res = self * other;
        res.round_mod();
        res

    }
}


impl Div<&FieldElement> for FieldElement {
    type Output = FieldElement;
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline(always)]
    fn div(self, other: &Self) -> FieldElement {
        let mut other = other.clone();
        self.same_modulo(&other);
        let p = &self.modulo - Integer::from(2);
        other.num = other.num.pow_mod(&p, &self.modulo).unwrap();
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
                let other: Integer = other.into();
                let num = self.num * other;
                mod_and_new(num, &self.modulo)
            }
        }

        impl Mul<&$t> for FieldElement {
        type Output = FieldElement;

            #[inline]
            fn mul(self, other: &$t) -> FieldElement {
                let other: Integer = Integer::from(other.clone());
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
                let s: Integer = self.into();
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
                let other = FieldElement::new(other, &self.modulo);
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
                let other = FieldElement::new(other.clone(), &self.modulo);
                self + other
            }
        }
       impl Add<&FieldElement> for $t {
       type Output = FieldElement;
            fn add(self, other: &FieldElement) -> FieldElement {
                let s = FieldElement::new(self, &other.modulo);
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
                let other = FieldElement::new(other, &self.modulo);
                self - other
             }
        }
       impl Sub<FieldElement> for $t {
       type Output = FieldElement;
            fn sub(self, other: FieldElement) -> FieldElement {
                let s = FieldElement::new(self, &other.modulo);
                s - other
            }
        }
       impl Sub<&$t> for FieldElement {
       type Output = FieldElement;
            fn sub(self, other: &$t) -> FieldElement {
                let other = FieldElement::new(other.clone(), &self.modulo);
                self - other
            }
        }
       impl Sub<&FieldElement> for $t {
       type Output = FieldElement;
            fn sub(self, other: &FieldElement) -> FieldElement {
                let s = FieldElement::new(self, &other.modulo);
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
                let other = FieldElement::new(other, &self.modulo);
                self / other
            }
        }
        impl Div<&$t> for FieldElement {
        type Output = FieldElement;
            fn div(self, other: &$t) -> FieldElement {
                let other = FieldElement::new(other.clone(), &self.modulo);
                self / other
            }
        }
       impl Div<FieldElement> for $t {
       type Output = FieldElement;
            fn div(self, other: FieldElement) -> FieldElement {
                let s = FieldElement::new(self, &other.modulo);
                s / other
            }
        }
       impl Div<&FieldElement> for $t {
       type Output = FieldElement;
            fn div(self, other: &FieldElement) -> FieldElement {
                let s = FieldElement::new(self, &other.modulo);
                s / other
            }
        }
    )*)
}

mul_impl_field! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }
add_impl_field! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }
sub_impl_field! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }
div_impl_field! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }