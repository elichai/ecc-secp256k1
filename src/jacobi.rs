use rug::Integer;
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

pub fn jacobi_symbol(mut numerator: Integer, mut denominator: Integer) -> Jacobi {
    debug_assert!(!denominator.is_even());
    debug_assert_ne!(denominator, 0);
    let mut res = Jacobi::One;
    while numerator != 0 {
        while numerator.is_even() {
            // As long as it's even we can use the second supplementary law (2/p) and check the symbol.
            numerator /= 2;
            let tmp = denominator.mod_u(8);
            if tmp == 3 || tmp == 5 {
                res.flip();
            }
        }
        // According to `Legendre's version of quadratic reciprocity`, we can flip them, and if both of them mod 4 equal 3 we just need to negate the Jacobi symbol.
        mem::swap(&mut numerator, &mut denominator);
        let num_mod = numerator.mod_u(4);
        if (num_mod == 3) && (denominator.mod_u(4) == num_mod) {
            res.flip();
        }

        numerator %= &denominator;
    }

    if denominator == 1 {
        res
    } else {
        Jacobi::Zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rug::{integer::Order, Integer};

    #[test]
    fn test_jacobi() {
        let a: Integer = 34321421432_u128.into();
        let b: Integer = 89732432312341_u128.into();

        let jacobi = jacobi_symbol(a.clone(), b.clone());

        let other = a.jacobi(&b);
        assert_eq!(i8::from(jacobi) as i32, other);
    }

    #[test]
    fn test_failed_jacobi() {
        let a = [
            63_u8, 57, 121, 191, 114, 174, 130, 2, 152, 61, 201, 137, 174, 199, 242, 255, 46, 217, 27, 221, 105, 206, 2, 252, 7, 0,
            202, 16, 14, 89, 221, 243,
        ];
        let a = Integer::from_digits(&a, Order::MsfLe);
        let b = [
            255_u8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 254, 255, 255, 252, 47,
        ];
        let b = Integer::from_digits(&b, Order::MsfLe);

        let my_jacobi = jacobi_symbol(a.clone(), b.clone());
        assert_eq!(i8::from(my_jacobi) as i32, a.jacobi(&b));
    }
}
