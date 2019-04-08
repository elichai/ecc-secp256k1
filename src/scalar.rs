use std::{ops::*, fmt};
use rug::Integer;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Scalar {
    num: Integer,
    pub modulo: Integer,
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.num)
    }
}

impl Scalar {
    pub fn new<I: Into<Integer>, T: Into<Integer>>(num: I, modulo: T) -> Scalar {
        Scalar { num: num.into(), modulo: modulo.into() }
    }

    pub fn infinity<I: Into<Integer>>(modulo: I) -> Scalar {
        Self { num: 0.into(), modulo: modulo.into() } // Right now I'm representing Infinity as (0,0).
    }
    pub fn is_infinity(&self) -> bool {
        self.num == Self::infinity(&self.modulo)
    }
    pub fn is_zero(&self) -> bool {
        self.num == 0
    }

    pub fn pow_u<I: Into<Integer>>(self, other: I) -> Scalar {
        let num = (self.num.pow_mod(&other.into(), &self.modulo)).unwrap();
        Scalar { num, modulo: self.modulo }
    }

    #[inline(always)]
    pub fn pow(self, other: Scalar) -> Scalar {
        self.same_modulo(&other);
        let num = (self.num.pow_mod(&other.num, &self.modulo)).unwrap();
        Scalar { num, modulo: self.modulo }
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
}

#[inline(always)]
fn mod_and_new(num: Integer, modulo: &Integer) -> Scalar {
    let num = num % modulo;
    let mut res = Scalar { num, modulo: modulo.clone() };
    res.round_mod();
    res
}

impl Add for Scalar {
    type Output = Scalar;
    #[inline(always)]
    fn add(self, other: Self) -> Scalar {
        self.same_modulo(&other);
        let num = (self.num + other.num) % &self.modulo;
        let mut res = Scalar { num, modulo: self.modulo };
        res.round_mod();
        res
    }
}
impl Add<&Scalar> for Scalar {
    type Output = Scalar;
    #[inline(always)]
    fn add(self, other: &Self) -> Scalar {
        self.same_modulo(&other);
        let num = (self.num + &other.num) % &self.modulo;
        let mut res = Scalar { num, modulo: self.modulo };
        res.round_mod();
        res
    }
}

impl Sub for Scalar {
    type Output = Scalar;
    #[inline(always)]
    fn sub(self, other: Self) -> Scalar {
        self.same_modulo(&other);
        let num = self.num - other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Sub<&Scalar> for Scalar {
    type Output = Scalar;
    #[inline(always)]
    fn sub(self, other: &Self) -> Scalar {
        self.same_modulo(&other);
        let num = self.num - &other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Mul for Scalar {
    type Output = Scalar;
    #[inline(always)]
    fn mul(self, other: Self) -> Scalar {
        self.same_modulo(&other);
        let num = self.num * other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Mul<&Scalar> for Scalar {
    type Output = Scalar;
    #[inline(always)]
    fn mul(self, other: &Self) -> Scalar {
        self.same_modulo(&other);
        let num = self.num * &other.num;
        mod_and_new(num, &self.modulo)
    }
}

impl Div for Scalar {
    type Output = Scalar;
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline(always)]
    fn div(self, other: Self) -> Scalar {
        let mut other = other;
        self.same_modulo(&other);
        let p = &self.modulo - Integer::from(2);
        other.num = other.num.pow_mod(&p, &self.modulo).unwrap();
        let mut res = self * other;
        res.round_mod();
        res

    }
}


impl Div<&Scalar> for Scalar {
    type Output = Scalar;
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline(always)]
    fn div(self, other: &Self) -> Scalar {
        let mut other = other.clone();
        self.same_modulo(&other);
        let p = &self.modulo - Integer::from(2);
        other.num = other.num.pow_mod(&p, &self.modulo).unwrap();
        let mut res = self * other;
        res.round_mod();
        res

    }
}

macro_rules! mul_impl_scalar {
    ($($t:ty)*) => ($(
        impl Mul<$t> for Scalar {
        type Output = Scalar;
            #[inline]
            fn mul(self, other: $t) -> Scalar {
                let other: Integer = other.into();
                let num = self.num * other;
                mod_and_new(num, &self.modulo)
            }
        }

        impl Mul<&$t> for Scalar {
        type Output = Scalar;

            #[inline]
            fn mul(self, other: &$t) -> Scalar {
                let other: Integer = Integer::from(other.clone());
                let num = self.num * other;
                mod_and_new(num, &self.modulo)
            }
        }

         impl Mul<Scalar> for $t {
            type Output = Scalar;

            #[inline]
            fn mul(self, other: Scalar) -> Scalar {
                other.mul(self)
            }
        }

         impl Mul<&Scalar> for $t {
            type Output = Scalar;

            #[inline]
            fn mul(self, other: &Scalar) -> Scalar {
                let s: Integer = self.into();
                let num = &other.num * s;
                mod_and_new(num, &other.modulo)
            }
        }
    )*)
}

macro_rules! add_impl_scalar {
    ($($t:ty)*) => ($(
       impl Add<$t> for Scalar {
       type Output = Scalar;
            fn add(self, other: $t) -> Scalar {
                let other = Scalar::new(other, &self.modulo);
                self + other
            }
        }
       impl Add<Scalar> for $t {
       type Output = Scalar;
            fn add(self, other: Scalar) -> Scalar {
                other + self
            }
        }
       impl Add<&$t> for Scalar {
       type Output = Scalar;
            fn add(self, other: &$t) -> Scalar {
                let other = Scalar::new(other.clone(), &self.modulo);
                self + other
            }
        }
       impl Add<&Scalar> for $t {
       type Output = Scalar;
            fn add(self, other: &Scalar) -> Scalar {
                let s = Scalar::new(self, &other.modulo);
                s + other
            }
        }
    )*)
}

macro_rules! sub_impl_scalar {
    ($($t:ty)*) => ($(
       impl Sub<$t> for Scalar {
       type Output = Scalar;
            fn sub(self, other: $t) -> Scalar {
                let other = Scalar::new(other, &self.modulo);
                self - other
             }
        }
       impl Sub<Scalar> for $t {
       type Output = Scalar;
            fn sub(self, other: Scalar) -> Scalar {
                let s = Scalar::new(self, &other.modulo);
                s - other
            }
        }
       impl Sub<&$t> for Scalar {
       type Output = Scalar;
            fn sub(self, other: &$t) -> Scalar {
                let other = Scalar::new(other.clone(), &self.modulo);
                self - other
            }
        }
       impl Sub<&Scalar> for $t {
       type Output = Scalar;
            fn sub(self, other: &Scalar) -> Scalar {
                let s = Scalar::new(self, &other.modulo);
                s - other
            }
        }
    )*)
}

macro_rules! div_impl_scalar {
    ($($t:ty)*) => ($(
       impl Div<$t> for Scalar {
       type Output = Scalar;
            fn div(self, other: $t) -> Scalar {
                let other = Scalar::new(other, &self.modulo);
                self / other
            }
        }
        impl Div<&$t> for Scalar {
        type Output = Scalar;
            fn div(self, other: &$t) -> Scalar {
                let other = Scalar::new(other.clone(), &self.modulo);
                self / other
            }
        }
       impl Div<Scalar> for $t {
       type Output = Scalar;
            fn div(self, other: Scalar) -> Scalar {
                let s = Scalar::new(self, &other.modulo);
                s / other
            }
        }
       impl Div<&Scalar> for $t {
       type Output = Scalar;
            fn div(self, other: &Scalar) -> Scalar {
                let s = Scalar::new(self, &other.modulo);
                s / other
            }
        }
    )*)
}

mul_impl_scalar! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }
add_impl_scalar! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }
sub_impl_scalar! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }
div_impl_scalar! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 Integer }