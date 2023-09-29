use egg::{Id, Language, RecExpr};

use crate::cad::Cad;
use crate::cad_struct::{get_num, get_vec3_nums};

pub fn remove_empty(expr: &RecExpr<Cad>, p: Id, out: &mut RecExpr<Cad>) -> Option<Id> {
    let e = expr[p].clone();
    use Cad::*;
    let res = match e {
        Empty => None,
        BlackBox(ref b, args) => {
            let args: Vec<_> = args
                .iter()
                .map(|c| remove_empty(expr, *c, out).unwrap_or_else(|| out.add(Cad::Empty)))
                .collect();
            Some(out.add(BlackBox(b.clone(), args)))
        }
        Hull(args) => {
            let args =
                args.map(|c| remove_empty(expr, c, out).unwrap_or_else(|| out.add(Cad::Empty)));
            Some(out.add(Cad::Hull(args)))
        }
        List(args) => {
            let args: Vec<_> = args
                .iter()
                .map(|c| remove_empty(expr, *c, out).unwrap_or_else(|| out.add(Cad::Empty)))
                .collect();
            Some(out.add(List(args)))
        }
        Cube(args) => {
            let args =
                args.map(|c| remove_empty(expr, c, out).unwrap_or_else(|| out.add(Cad::Empty)));
            let v = get_vec3_nums(out, args[0]);
            if v.0 == 0.0 || v.1 == 0.0 || v.2 == 0.0 {
                None
            } else {
                Some(out.add(Cube(args)))
            }
        }
        Sphere(args) => {
            let args =
                args.map(|c| remove_empty(expr, c, out).unwrap_or_else(|| out.add(Cad::Empty)));
            let r = get_num(out, args[0]);
            if r == 0.0 {
                None
            } else {
                Some(out.add(Sphere(args)))
            }
        }
        Cylinder(args) => {
            let args =
                args.map(|c| remove_empty(expr, c, out).unwrap_or_else(|| out.add(Cad::Empty)));
            let (h, r1, r2) = get_vec3_nums(out, args[0]);
            if h == 0.0 || (r1, r2) == (0.0, 0.0) {
                None
            } else {
                Some(out.add(Cylinder(args)))
            }
        }
        Affine(args) => {
            let args =
                args.map(|c| remove_empty(expr, c, out).unwrap_or_else(|| out.add(Cad::Empty)));
            Some(out.add(Affine(args)))
            // TODO check scale
        }
        Binop(args) => {
            let args = args.map(|c| remove_empty(expr, c, out));
            let bop_id = args[0].expect("op should be valid");
            let bop = out[bop_id].clone();
            let a = args[1];
            let b = args[2];
            match bop {
                Union => match (a, b) {
                    (Some(op1), Some(op2)) => Some(out.add(Binop([bop_id, op1, op2]))),
                    _ => a.or(b),
                },
                Inter => match (a, b) {
                    (Some(op1), Some(op2)) => Some(out.add(Binop([bop_id, op1, op2]))),
                    _ => None,
                },
                Diff => match (a, b) {
                    (Some(op1), Some(op2)) => Some(out.add(Binop([bop_id, op1, op2]))),
                    (Some(op1), None) => Some(op1),
                    (None, Some(op2)) => Some(op2),
                    _ => panic!(
                        "should have at least op available: bop {:?} with op1 {:?} and op2 {:?}",
                        bop, a, b
                    ),
                },
                _ => panic!("unexpected binop: {:?}", bop),
            }
        }
        Fold(args) => {
            let bop = expr[args[0]].clone();
            let list = expr[args[1]].clone();
            assert!(matches!(list, List(_)));
            let listargs = list.children().iter().map(|e| remove_empty(expr, *e, out));
            match bop {
                Union => {
                    let non_empty: Vec<Id> = listargs.flatten().collect();
                    if non_empty.is_empty() {
                        None
                    } else {
                        let listexpr = List(non_empty);
                        let union_expr = out.add(Union);
                        let listexpr = out.add(listexpr);
                        Some(out.add(Fold([union_expr, listexpr])))
                    }
                }
                Inter => {
                    let args: Option<Vec<Id>> = listargs.collect();
                    let listexpr = List(args?);
                    let inter = out.add(Inter);
                    let listexpr = out.add(listexpr);
                    Some(out.add(Fold([inter, listexpr])))
                }
                Diff => {
                    let mut listargs = listargs;
                    // if first is empty, then we are empty
                    let first = listargs.next().unwrap()?;

                    let non_empty: Vec<Id> = listargs.flatten().collect();
                    if non_empty.is_empty() {
                        Some(first)
                    } else {
                        let mut args = vec![first];
                        args.extend(non_empty);
                        let listexpr = List(args);
                        let diff = out.add(Diff);
                        let listexpr = out.add(listexpr);
                        Some(out.add(Fold([diff, listexpr])))
                    }
                }
                _ => panic!("unexpected binop: {:?}", bop),
            }
        }
        _ => {
            let e =
                e.map_children(|id| remove_empty(expr, id, out).unwrap_or_else(|| out.add(Empty)));
            Some(out.add(e))
        }
    };
    if res.is_none() {
        println!("Found empty: {}", expr.pretty(80));
    }
    res
}
