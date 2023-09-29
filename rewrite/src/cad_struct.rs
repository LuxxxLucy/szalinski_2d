use crate::cad::Cad;
use egg::{Id, RecExpr};

pub fn get_num(expr: &RecExpr<Cad>, p: Id) -> f64 {
    match expr[p] {
        Cad::Num(num) => num.to_f64(),
        _ => panic!("Not a num"), // is panic the right thing?
    }
}

pub fn get_vec3_nums(expr: &RecExpr<Cad>, p: Id) -> (f64, f64, f64) {
    match expr[p] {
        Cad::Vec3(arg) => (
            get_num(expr, arg[0]),
            get_num(expr, arg[1]),
            get_num(expr, arg[2]),
        ),
        _ => panic!("Not a vec3"), // is panic the right thing?
    }
}
