pub type Cost = f64;

use egg::{Id, Language};

use crate::{
    cad::Cad,
    hyperparameters::{COST_BIG_VALUE, COST_SMALL_VALUE},
};

pub struct CostFn;
impl egg::CostFunction<Cad> for CostFn {
    type Cost = Cost;

    fn cost<C>(&mut self, enode: &Cad, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        use Cad::*;
        let cost = match enode {
            Num(n) => {
                let s = format!("{}", n);
                1.0 + (0.000001 * s.len() as Cost)
            }
            Bool(_) | ListVar(_) => COST_SMALL_VALUE,
            Add(_args) | Sub(_args) | Mul(_args) | Div(_args) => COST_SMALL_VALUE,

            BlackBox(..) => 1.0,
            Cube(_) | Empty | Nil | Sphere(_) | Cylinder(_) | Hull(_) => 1.0,

            Trans | TransPolar | Scale | Rotate => 1.0,

            Union | Diff | Inter => 1.0,

            Repeat(_) => 0.99,
            MapI(_) => 1.0,
            Fold(_) => 1.0,
            Map2(_) => 1.0,
            Affine(_) => 1.0,
            Binop(_) => 1.0,

            Concat(_) => 1.0,
            Cons(_) => 1.0,
            List(_) => 1.0,
            Vec3(_) => 1.0,

            Unpolar(_) => COST_BIG_VALUE,
            Sort(_) | Unsort(_) | Part(_) | Unpart(_) => COST_BIG_VALUE,
            Partitioning(_) => COST_BIG_VALUE,
            Permutation(_) => COST_BIG_VALUE,
        };

        enode.fold(cost, |sum, i| sum + costs(i))
    }
}
