/// Scad
use std::fmt;

use egg::{Id, Language, RecExpr};

use crate::cad_struct::get_vec3_nums;

use crate::cad::Cad;

use crate::eval::eval;

pub struct Scad<'a>(pub &'a RecExpr<Cad>, pub Id);

impl<'a> Scad<'a> {
    pub fn new(expr: &'a RecExpr<Cad>) -> Scad<'a> {
        Scad(expr, (expr.as_ref().len() - 1).into())
    }
}

impl<'a> fmt::Display for Scad<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt_impl = |p: Id, out: &RecExpr<Cad>| -> fmt::Result {
            let expr = &out[p];
            let arg = |i: usize| expr.children()[i];
            let child = |i: usize| Scad(out, expr.children()[i]);
            match expr {
                Cad::Num(float) => write!(f, "{}", float),
                Cad::Bool(b) => write!(f, "{}", b),
                Cad::Vec3(children) => write!(f, "[{}, {}, {}]", children[0], children[1], children[2]),
                Cad::Add(children) => write!(f, "{} + {}", children[0], children[1]),
                Cad::Sub(children) => write!(f, "{} - {}", children[0], children[1]),
                Cad::Mul(children) => write!(f, "{} * {}", children[0], children[1]),
                Cad::Div(children) => write!(f, "{} / {}", children[0], children[1]),
                Cad::Empty => writeln!(f, "sphere(r=0);"),
                Cad::Cube(_) => writeln!(f, "cube({}, center={});", child(0), child(1)),
                Cad::Sphere(_) => writeln!(
                    f,
                    "sphere(r = {}, $fn = {}, $fa = {}, $fs = {});",
                    child(0),
                    get_vec3_nums(out, arg(1)).0,
                    get_vec3_nums(out, arg(1)).1,
                    get_vec3_nums(out, arg(1)).2
                ),
                Cad::Cylinder(_) => writeln!(
                    f,
                    "cylinder(h = {}, r1 = {}, r2 = {}, $fn = {}, $fa = {}, $fs = {}, center = {});",
                    get_vec3_nums(out, arg(0)).0,
                    get_vec3_nums(out, arg(0)).1,
                    get_vec3_nums(out, arg(0)).2,
                    get_vec3_nums(out, arg(1)).0,
                    get_vec3_nums(out, arg(1)).1,
                    get_vec3_nums(out, arg(1)).2,
                    child(2),
                ),
                Cad::Hull(_) => {
                    write!(f, "hull() {{")?;
                    for cad in out[arg(0)].children() {
                        write!(f, "  {}", Scad(out, *cad))?;
                    }
                    write!(f, "}}")
                }

                Cad::Trans => write!(f, "translate"),
                Cad::Scale => write!(f, "scale"),
                Cad::Rotate => write!(f, "rotate"),
                Cad::Affine(_) => write!(f, "{} ({}) {}", child(0), child(1), child(2)),

                Cad::Union => write!(f, "union"),
                Cad::Inter => write!(f, "intersection"),
                Cad::Diff => write!(f, "difference"),
                Cad::Fold(_) => {
                    writeln!(f, "{} () {{", child(0))?;
                    for cad in out[arg(1)].children() {
                        write!(f, "  {}", Scad(out, *cad))?;
                    }
                    write!(f, "}}")
                }
                Cad::BlackBox(b, _) => {
                    writeln!(f, "{} {{", b)?;
                    for cad in expr.children().iter() {
                        write!(f, "  {}", Scad(out, *cad))?;
                    }
                    write!(f, "}}")
                }
                cad => panic!("TODO: {:?}", cad),
            }
        };
        // may need to shrink expr to match self.1
        let mut normalform = RecExpr::from(vec![]);
        let p = eval(None, self.0, self.1, &mut normalform);
        fmt_impl(p, &normalform)
    }
}
