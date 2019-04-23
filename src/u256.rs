use std::ops::*;
#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
struct U256 {
    h: u128,
    l: u128,
}

macro_rules! impl_add_mul {
    ($Lhs:ty, $Rhs:ty) => (
        impl Add<$Rhs> for $Lhs {
            type Output = U256;
            #[inline(always)]
            fn add(self, other: $Rhs) -> U256 {
                let ref a = self;
                let ref b = other;
                let mut res = U256::default();
                let (res0, is_overflown) = a.h.overflowing_add(b.h);
                res.h = res0;
                res.l = is_overflown as u128;
                res.l += a.l + b.l;
                res
            }
        }

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
    );
}

macro_rules! impl_add_assign {
    ($Lhs:ty, $Rhs:ty) => (
        impl AddAssign<$Rhs> for $Lhs {
            fn add_assign(&mut self, other: $Rhs) {
                let me = *self;
                *self = me + other;
            }
        }
    );
}
impl_add_mul! {U256, U256}
impl_add_mul! {U256, &U256}
impl_add_mul! {&U256, U256}
impl_add_mul! {&U256, &U256}

impl_add_assign! {U256, U256}
impl_add_assign! {U256, &U256}

fn mul_u128(a: u128, b: u128) -> U256 {
    let (a_h, a_l) = split(a);
    let (b_h, b_l) = split(b);

    let mut res_l = a_l * b_l;
    res_l += (a_l * b_h) << 64;
    res_l += (a_h * b_l) << 64;

    let res_h = a_h * b_h;
    U256{h: res_h, l: res_l}
}


fn split(input: u128) -> (u128, u128) {
    let l: u128 = input as u64 as u128;
    let h: u128 = (input >> 64) as u64 as u128;
    (h,l)
}