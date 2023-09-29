use std::collections::HashMap;

use egg::{Id, Language, RecExpr};

use crate::base::geom::to_cartesian;
use crate::cad::Cad;
use crate::cad_struct::{get_num, get_vec3_nums};

// Given a sexpr, interpret the Expression. This is essentially from my understanding constant
// folding
pub fn eval(cx: Option<&FunCtx>, expr: &RecExpr<Cad>, p: Id, out: &mut RecExpr<Cad>) -> Id {
    eval_(cx, expr, p, out)
}

type FunCtx = HashMap<&'static str, usize>;

fn mk_vec((x, y, z): (f64, f64, f64), out: &mut RecExpr<Cad>) -> Id {
    let x = out.add(Cad::Num(x.into()));
    let y = out.add(Cad::Num(y.into()));
    let z = out.add(Cad::Num(z.into()));
    out.add(Cad::Vec3([x, y, z]))
}

fn mk_list(exprs: Vec<Id>) -> Cad {
    Cad::List(exprs)
}

fn get_list(expr: &RecExpr<Cad>, list: Id) -> &Vec<Id> {
    match &expr[list] {
        Cad::List(list) => list,
        cad => panic!("expected list, got {:?}", cad),
    }
}

fn eval_list(cx: Option<&FunCtx>, expr: &RecExpr<Cad>, p: Id, out: &mut RecExpr<Cad>) -> Vec<Id> {
    let list = eval(cx, expr, p, out);
    match &out[list] {
        Cad::List(list) => list.clone(),
        cad => panic!("expected list, got {:?}", cad),
    }
}

fn eval_(cx: Option<&FunCtx>, expr: &RecExpr<Cad>, p: Id, out: &mut RecExpr<Cad>) -> Id {
    let e = expr[p].clone();
    match &e {
        Cad::BlackBox(ref b, args) => {
            let args: Vec<_> = args.iter().map(|c| eval(cx, expr, *c, out)).collect();
            out.add(Cad::BlackBox(b.clone(), args))
        }
        // arith
        Cad::Bool(_) => out.add(e),
        Cad::Num(_) => out.add(e),
        Cad::ListVar(v) => {
            let n = cx.unwrap()[v.0];
            out.add(Cad::Num(n.into()))
        }
        Cad::Add(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            let a = get_num(out, args[0]);
            let b = get_num(out, args[1]);
            out.add(Cad::Num((a + b).into()))
        }
        Cad::Sub(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            let a = get_num(out, args[0]);
            let b = get_num(out, args[1]);
            out.add(Cad::Num((a - b).into()))
        }
        Cad::Mul(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            let a = get_num(out, args[0]);
            let b = get_num(out, args[1]);
            out.add(Cad::Num((a * b).into()))
        }
        Cad::Div(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            let a = get_num(out, args[0]);
            let b = get_num(out, args[1]);
            out.add(Cad::Num((a / b).into()))
        }
        // cad
        Cad::Cube(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            out.add(Cad::Cube(args))
        }
        Cad::Sphere(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            out.add(Cad::Sphere(args))
        }
        Cad::Cylinder(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            out.add(Cad::Cylinder(args))
        }
        // Cad::Hexagon => out.add(Cad::Hexagon),
        Cad::Empty => out.add(Cad::Empty),
        Cad::Vec3(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            out.add(Cad::Vec3(args))
        }
        Cad::Hull(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            out.add(Cad::Hull(args))
        }

        Cad::Trans | Cad::Scale | Cad::Rotate | Cad::TransPolar => out.add(e.clone()),

        Cad::Affine(args) => {
            let aff = eval(cx, expr, args[0], out);
            match out[aff] {
                Cad::Trans | Cad::Scale | Cad::Rotate => {
                    let param = eval(cx, expr, args[1], out);
                    let cad = eval(cx, expr, args[2], out);
                    out.add(Cad::Affine([aff, param, cad]))
                }
                Cad::TransPolar => {
                    let param = eval(cx, expr, args[1], out);
                    let cad = eval(cx, expr, args[2], out);
                    let pnums = get_vec3_nums(out, param);
                    let cnums = to_cartesian(pnums);

                    let trans = out.add(Cad::Trans);
                    let cnums = mk_vec(cnums, out);
                    out.add(Cad::Affine([trans, cnums, cad]))
                }
                _ => panic!("expected affine kind, got {:?}", aff),
            }
        }

        Cad::Diff => out.add(Cad::Diff),
        Cad::Inter => out.add(Cad::Inter),
        Cad::Union => out.add(Cad::Union),

        Cad::Fold(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            out.add(Cad::Fold(args))
        }
        Cad::Binop(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            let list = out.add(Cad::List(vec![args[1], args[2]]));
            out.add(Cad::Fold([args[0], list]))
        }

        // lists
        Cad::Nil => out.add(mk_list(vec![])),
        Cad::Cons(args) => {
            let mut list = eval_list(cx, expr, args[1], out);
            list.insert(0, eval(cx, expr, args[0], out));
            out.add(mk_list(list))
        }
        Cad::List(list) => {
            let list = mk_list(list.iter().map(|e| eval(cx, expr, *e, out)).collect());
            out.add(list)
        }
        Cad::Repeat(args) => {
            let args = args.map(|arg| eval(cx, expr, arg, out));
            let n = get_num(out, args[0]);
            let t = args[1];
            out.add(mk_list(vec![t; n as usize]))
        }
        Cad::Concat(args) => {
            let mut vec = Vec::new();
            for list in eval_list(cx, expr, args[0], out) {
                for c in get_list(out, list) {
                    vec.push(*c)
                }
            }
            out.add(mk_list(vec))
        }
        Cad::Map2(args) => {
            let op = out.add(expr[e.children()[0]].clone());
            let params: Vec<_> = eval_list(cx, expr, args[1], out);
            let cads: Vec<_> = eval_list(cx, expr, args[2], out);
            let list = mk_list(
                params
                    .into_iter()
                    .zip(cads)
                    .map(|(p, c)| out.add(Cad::Affine([op, p, c])))
                    .collect(),
            );
            out.add(list)
        }
        Cad::MapI(args) => {
            let body = *args.last().unwrap();
            let bounds: Vec<usize> = args[..args.len() - 1]
                .iter()
                .map(|n| get_num(expr, *n) as usize)
                .collect();
            let mut ctx = HashMap::new();
            let mut vec = Vec::new();
            match bounds.len() {
                1 => {
                    for i in 0..bounds[0] {
                        ctx.insert("i", i);
                        vec.push(eval(Some(&ctx), expr, body, out));
                    }
                }
                2 => {
                    for i in 0..bounds[0] {
                        ctx.insert("i", i);
                        for j in 0..bounds[1] {
                            ctx.insert("j", j);
                            vec.push(eval(Some(&ctx), expr, body, out));
                        }
                    }
                }
                3 => {
                    for i in 0..bounds[0] {
                        ctx.insert("i", i);
                        for j in 0..bounds[1] {
                            ctx.insert("j", j);
                            for k in 0..bounds[2] {
                                ctx.insert("k", k);
                                vec.push(eval(Some(&ctx), expr, body, out));
                            }
                        }
                    }
                }
                _ => unimplemented!(),
            }

            out.add(mk_list(vec))
        }
        cad => panic!("can't eval({:?})", cad),
    }
}
