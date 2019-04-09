mod field;
mod point;

use self::point::*;



fn main() {
    let _group = Group::new(0,7);

    let point = Point::new(47, 71, 223).unwrap();
    let mut current = Point::new(0, 0, 223).unwrap();
//    for _ in 0..22 {
//        current = dbg!(current + point.clone());
//    }
    dbg!(point+current);
}
