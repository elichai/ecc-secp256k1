use std::cmp::Ordering;
use std::ops::*;

#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
struct U256 {
    h: u128,
    l: u128,
}

impl U256 {
    //    pub fn pow(self, n: Self) -> Self {
    //        let mut res = self;
    //        while n > 0 {
    //            res *= res;
    //        }
    //        res
    //    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let mut h = [0u8; 16];
        let mut l = [0u8; 16];
        h.copy_from_slice(&bytes[0..16]);
        l.copy_from_slice(&bytes[16..32]);

        U256 { h: u128::from_be_bytes(h), l: u128::from_be_bytes(l) }
    }

    pub fn into_be_bytes(self) -> [u8; 32] {
        let mut res = [0u8; 32];
        res[0..16].copy_from_slice(dbg!(&self.h.to_be_bytes()));
        res[16..32].copy_from_slice(dbg!(&self.l.to_be_bytes()));
        res
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut res = [0u8; 32];
        res[0..16].copy_from_slice(&self.h.to_be_bytes());
        res[16..32].copy_from_slice(&self.l.to_be_bytes());
        res
    }

    //    pub fn mul_u64(mut self, other: u64) -> U256 {
    //        let mut ret = U256::default();
    //        {
    //            let (mut lower_h, mut lower_l) = split(self.l);
    //
    //            lower_l *= other as u128;
    //            lower_h *= other as u128;
    //            self.h += split(lower_h).0;
    //            ret += lower_l;
    //            ret += lower_h << 64;
    //        }
    //
    //        let (mut higher_h, mut higher_l) = split(self.h);
    //
    //        higher_l *= other as u128;
    //        higher_h *= other as u128;
    //        ret += higher_l;
    //        ret += higher_h << 64;
    //        ret
    //    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &U256) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &U256) -> Ordering {
        let high_order = self.h.cmp(&other.h);
        match high_order {
            Ordering::Equal => self.l.cmp(&other.l),
            Ordering::Greater | Ordering::Less => high_order,
        }
    }
}

macro_rules! impl_from_less_than_128 {
    ($($t:ty),*) => {$(
        impl From<$t> for U256 {
            fn from(l: $t) -> Self {
                U256 {
                    l: l as u128,
                    h: 0,
                }
            }
        }

    )+}
}

macro_rules! ops_less_than_128 {
    ($($t:ty),*) => {$(
        impl Add<$t> for U256 {
            type Output = U256;
            #[inline(always)]
            fn add(self, other: $t) -> U256 {
                let a = &self;
                let b = &U256::from(other);
                a + b
            }
        }
        impl Add<U256> for $t {
            type Output = U256;
            #[inline(always)]
            fn add(self, other: U256) -> U256 {
                let a = &U256::from(self);
                let b = &other;
                a + b
            }
        }

        impl Add<&U256> for $t {
            type Output = U256;
            #[inline(always)]
            fn add(self, other: &U256) -> U256 {
                let a = U256::from(self);
                let b = other;
                a + b
            }
        }

         impl AddAssign<$t> for U256 {
            fn add_assign(&mut self, other: $t) {
                let me = *self;
                *self = me + other;
            }
        }

        impl AddAssign<&$t> for U256 {
            fn add_assign(&mut self, other: &$t) {
                let me = *self;
                *self = me + *other;
            }
        }
    )+}
}

impl_from_less_than_128! { u128, u64, u32, u16, u8 }
ops_less_than_128! { u128, u64, u32, u16, u8 }

macro_rules! reg_ops {
    ($Lhs:ty, $Rhs:ty) => {
        impl Add<$Rhs> for $Lhs {
            type Output = U256;
            #[inline(always)]
            fn add(self, other: $Rhs) -> U256 {
                let a = &self;
                let b = &other;
                let mut res = U256::default();
                let (res0, is_overflown) = dbg!(a.l.overflowing_add(b.l));
                res.l = res0;
                res.h += is_overflown as u128;
                res.h += a.h + b.h;
                res
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl Mul<$Rhs> for $Lhs {
            type Output = U256;
            #[inline(always)]
            fn mul(self, other: $Rhs) -> U256 {
                let (ref a, ref b) = (self, other);
                let mut low: U256;
                //                let mut high: U256 = U256::default();
                low = mul_u128(a.l, b.l);
                let al_bh = mul_u128(a.l, b.h);
                let ah_bl = mul_u128(a.h, b.l);
                low.h += al_bh.l + ah_bl.l;
                //                high.l += al_bh.h + ah_bl.h;
                //                high = high + mul(a.h, b.h);
                low
            }
        }
    };
}

macro_rules! assign_ops {
    ($Lhs:ty, $Rhs:ty) => {
        impl AddAssign<$Rhs> for $Lhs {
            fn add_assign(&mut self, other: $Rhs) {
                let me = *self;
                *self = me + other;
            }
        }
        impl MulAssign<$Rhs> for $Lhs {
            fn mul_assign(&mut self, other: $Rhs) {
                let me = *self;
                *self = me * other;
            }
        }
    };
}
reg_ops! {U256, U256}
reg_ops! {U256, &U256}
reg_ops! {&U256, U256}
reg_ops! {&U256, &U256}

assign_ops! {U256, U256}
assign_ops! {U256, &U256}

fn mul_u128(a: u128, b: u128) -> U256 {
    let mut res = U256::default();
    let (a_h, a_l) = dbg!(split(a));
    let (b_h, b_l) = dbg!(split(b));

    res.l = a_l * b_l;
    res += (a_l * b_h) << 64;
    res += (a_h * b_l) << 64;

    if a_h == 0 || b_h == 0 {
        res.h += a_h + b_h;
    } else {
        res.h += a_h * b_h;
    }
    res
}

fn split(input: u128) -> (u128, u128) {
    let l: u128 = input as u64 as u128;
    let h: u128 = (input >> 64) as u64 as u128;
    (h, l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use numext_fixed_uint::U256 as T_U256;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_from_to() {
        let mut rng = thread_rng();
        for _ in 0..15 {
            let bytes: [u8; 32] = rng.gen();
            let u256 = U256::from_bytes(bytes);
            assert_eq!(bytes, u256.into_be_bytes());
        }
    }

    #[test]
    fn test_be() {
        let mut rng = thread_rng();
        let num: u128 = rng.gen();
        let (my, test) = get_uints(num);
        assert_eq!(test.to_be_bytes(), my.into_be_bytes());
    }
    #[test]
    fn test_be_mul() {
        let mut rng = thread_rng();
        let a: u128 = rng.gen();
        let b = u64::max_value() as u128;
        let (a_my, a_test) = get_uints(a);
        let (b_my, b_test) = get_uints(b);
        dbg!(a);
        dbg!(b);

        let my = a_my * b_my;
        let test = a_test * b_test;
        dbg!(&my);
        dbg!(&test);
        assert_eq!(test.to_be_bytes(), my.into_be_bytes());
    }

    #[test]
    fn test_specific() {
        let real_be = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 187, 38, 228, 19, 155, 141, 172, 191, 166, 217, 27, 236, 100, 114, 83, 63,
            158,
        ];
        let a = 3452343528210635210850_u128;
        let b = 18446744073709551615_u128;
        let res = mul_u128(a, b);

        assert_eq!(res.into_be_bytes(), real_be);
    }

    #[test]
    fn test_other_specific() {
        let real_be = [
            0, 0, 0, 0, 0, 0, 0, 0, 205, 24, 175, 194, 81, 32, 160, 12, 252, 10, 19, 243, 165, 227, 59, 3, 54, 221, 60, 74, 8, 252,
            36, 239,
        ];
        let a = 272619919077564483872665775724761963281_u128;
        let b = 18446744073709551615_u128;
        let res = mul_u128(a, b);
        assert_eq!(res.into_be_bytes(), real_be);
    }

    fn slice_to_u128(s: &[u8]) -> u128 {
        let mut shit = [0u8; 16];
        shit.copy_from_slice(s);
        u128::from_be_bytes(shit)
    }

    fn get_uints(num: u128) -> (U256, T_U256) {
        (U256::from(num), T_U256::from(num))
    }
}
