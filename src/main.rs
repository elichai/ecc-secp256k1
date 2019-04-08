mod scalar;
mod point;

use self::point::*;



fn main() {
    let _group = Group::new(0,7);
//    let a = Point::new(73, 128, 137).unwrap();
//    let b = Point::new(46, 22, 137).unwrap();
//    let c = dbg!(a+b);
    let _c = dbg!(addition((192, 105), (17, 56)));
    let _c = dbg!(addition((47, 71), (117, 141)));
    let _c = dbg!(addition((143, 98), (76, 66)));

    let a = Point::new(73,128,137).unwrap();
    dbg!(a.clone()+a);
}

fn addition(a: (u128, u128), b: (u128, u128)) -> Point {
    Point::new(a.0,a.1, 223).unwrap() +
        Point::new(b.0,b.1, 223).unwrap()
}
