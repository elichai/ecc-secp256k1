use std::{ops::*, fmt};
use rug::Integer;
use crate::field::*;

#[derive(Clone, PartialEq)]
pub struct Group {
    pub a: Integer,
    pub b: Integer,
}

impl Group {
    pub fn new<I: Into<Integer>, T: Into<Integer>>(a: I, b: T) -> Self {
        Self{ a: a.into(), b: b.into() }
    }
}

#[derive(PartialEq, Clone)]
pub struct Point {
    x: FieldElement,
    y: FieldElement,
    group: Group,
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
    pub fn new<I,T,V>(x: I, y: T, modulo: V) -> Result<Self, ()>
    where I: Into<Integer>, T: Into<Integer>, V: Into<Integer> {
        let group = Group::new(0,7);
        Self::new_with_group(x,y,modulo,group)
    }

    pub fn new_with_group<I,T,V>(x: I, y: T, modulo: V, group: Group) -> Result<Self, ()>
        where I: Into<Integer>, T: Into<Integer>, V: Into<Integer> {
        let x = FieldElement::new(x, modulo);
        let y = FieldElement::new(y, x.modulo.clone());
        let point = Self { x, y, group };
        if !point.is_on_curve() {
            Err(())
        } else {
            Ok(point)
        }
    }

    #[inline(always)]
    fn is_on_curve(&self) -> bool {
        self.y.clone().pow_u(2) == (self.x.clone().pow_u(3) + self.group.a.clone() * self.x.clone() + &self.group.b) // Y^2 = X^3 + ax + b
    }

    #[inline(always)]
    fn get_slope(&self, other: &Point) -> FieldElement {
        if self.x != other.x {
            (self.y.clone() - other.y.clone()) / (self.x.clone() - other.x.clone())
        } else {
            (3 * self.x.clone().pow_u(2) + &self.group.a) /
                (2 * self.y.clone())
        }
    }
}

impl Add for Point {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        let inf = FieldElement::infinity(&self.x.modulo);
        if self.x.is_infinity() {
            other
        } else if other.x.is_infinity() {
            self
        } else if self.x == other.x && self.y != other.y {
            let inf = FieldElement::infinity(&self.x.modulo);
            Self { x: inf.clone(), y: inf.clone(), group: self.group }
        } else if self == other && self.y.is_zero() {
            Self { x: inf.clone(), y: inf.clone(), group: self.group }
        } else {
            let m = self.get_slope(&other);
            let x = m.clone().pow_u(2) - self.x.clone() - other.x;
            let y = m*(self.x-x.clone())-self.y; // negative of y-y1=m(x-x1)
            Self {x,y, group: self.group}
        }
    }
}