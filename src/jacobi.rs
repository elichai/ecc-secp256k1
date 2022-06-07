use num_bigint::BigInt;
use num_integer::Integer;
use std::mem;

#[derive(PartialEq, Eq, Debug)]
pub enum Jacobi {
    Zero,
    One,
    MinusOne,
}
impl Jacobi {
    pub fn flip(&mut self) {
        match *self {
            Jacobi::One => *self = Jacobi::MinusOne,
            Jacobi::MinusOne => *self = Jacobi::One,
            Jacobi::Zero => (),
        }
    }
}

impl From<Jacobi> for i8 {
    fn from(sym: Jacobi) -> i8 {
        match sym {
            Jacobi::One => 1,
            Jacobi::MinusOne => -1,
            Jacobi::Zero => 0,
        }
    }
}

pub fn jacobi_symbol(mut numerator: BigInt, mut denominator: BigInt) -> Jacobi {
    debug_assert!(!denominator.is_even());
    debug_assert_ne!(denominator, BigInt::from(0u32));
    let mut res = Jacobi::One;
    while numerator != BigInt::from(0u32) {
        while numerator.is_even() {
            // As long as it's even we can use the second supplementary law (2/p) and check the symbol.
            numerator /= 2u32;
            let tmp = denominator.mod_floor(&8u32.into());
            if tmp == 3u32.into() || tmp == 5u32.into() {
                res.flip();
            }
        }
        // According to `Legendre's version of quadratic reciprocity`, we can flip them, and if both of them mod 4 equal 3 we just need to negate the Jacobi symbol.
        mem::swap(&mut numerator, &mut denominator);
        let num_mod = numerator.mod_floor(&4u32.into());
        if (num_mod == 3u32.into()) && (denominator.mod_floor(&4u32.into()) == num_mod) {
            res.flip();
        }

        numerator %= &denominator;
    }

    if denominator == 1u32.into() {
        res
    } else {
        Jacobi::Zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, Sign};

    #[test]
    fn test_jacobi() {
        let a: BigInt = 34321421432_u128.into();
        let b: BigInt = 89732432312341_u128.into();

        let jacobi = jacobi_symbol(a.clone(), b.clone());

        assert_eq!(i8::from(jacobi) as i32, -1);
    }

    fn euiler_criterion(a: &BigInt, p: &BigInt) -> i8 {
        let exp = (p - 1u32) / 2u32;
        let res = a.modpow(&exp, p);
        if res == BigInt::from(1u32) {
            1
        } else if res == (p - 1u32) {
            return -1;
        } else {
            unreachable!()
        }
    }
    #[test]
    fn test_failed_jacobi() {
        let a = [
            63_u8, 57, 121, 191, 114, 174, 130, 2, 152, 61, 201, 137, 174, 199, 242, 255, 46, 217, 27, 221, 105, 206, 2, 252, 7, 0,
            202, 16, 14, 89, 221, 243,
        ];
        let a = BigInt::from_bytes_be(Sign::Plus, &a);
        let b = [
            255_u8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 254, 255, 255, 252, 47,
        ];
        let b = BigInt::from_bytes_be(Sign::Plus, &b);

        let my_jacobi = jacobi_symbol(a.clone(), b.clone());
        assert_eq!(i8::from(my_jacobi), euiler_criterion(&a, &b));
    }
}
