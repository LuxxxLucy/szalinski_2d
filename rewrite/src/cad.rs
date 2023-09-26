use std::fmt;
use std::str::FromStr;

use log::debug;

use egg::*;

use crate::{
    base::num::{num, Num},
    cost::{Cost, CostFn},
    permute::{Partitioning, Permutation},
};

pub type EGraph = egg::EGraph<Cad, MetaAnalysis>;
pub type EClass = egg::EClass<Cad, MetaAnalysis>;
pub type Rewrite = egg::Rewrite<Cad, MetaAnalysis>;

pub type Vec3 = (Num, Num, Num);

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub struct ListVar(pub &'static str);
impl FromStr for ListVar {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i" => Ok(ListVar("i")),
            "j" => Ok(ListVar("j")),
            "k" => Ok(ListVar("k")),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ListVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
pub struct BlackBox(String);
impl FromStr for BlackBox {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug!("Parsing black box: {}", s);
        Ok(BlackBox(s.to_owned()))
    }
}
impl fmt::Display for BlackBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.replace("\\\"", "\"");
        write!(f, "{}", s)
    }
}

define_language! {
    pub enum Cad {
        "Cube" = Cube([Id; 2]),
        "Sphere" = Sphere([Id; 2]),
        "Cylinder" = Cylinder([Id; 3]),
        "Empty" = Empty,
        "Hull" = Hull([Id; 1]),
        "Nil" = Nil,
        Num(Num),
        Bool(bool),

        // TODO: mapI could be a smallvec
        "MapI" = MapI(Vec<Id>),
        ListVar(ListVar),
        "Repeat" = Repeat([Id; 2]),

        "Trans" = Trans,
        "TransPolar" = TransPolar,
        "Scale" = Scale,
        "Rotate" = Rotate,

        "Union" = Union,
        "Diff" = Diff,
        "Inter" = Inter,

        "Map2" = Map2([Id; 3]),
        "Fold" = Fold([Id; 2]),
        "Affine" = Affine([Id; 3]),
        "Binop" = Binop([Id; 3]),

        "Vec3" = Vec3([Id; 3]),

        "Cons" = Cons([Id; 2]),
        "Concat" = Concat([Id; 1]),
        "List" = List(Vec<Id>),

        "Sort" = Sort([Id; 2]),
        "Unsort" = Unsort([Id; 2]),
        "Part" = Part([Id; 2]),
        "Unpart" = Unpart([Id; 2]),
        "Unpolar" = Unpolar([Id; 3]),

        Permutation(Permutation),
        Partitioning(Partitioning),

        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        BlackBox(BlackBox, Vec<Id>),
    }
}

#[derive(Debug, Default)]
pub struct MetaAnalysis;
#[derive(Debug, Clone)]
pub struct Meta {
    pub list: Option<Vec<Id>>,
    pub cost: Cost,
    pub best: Cad,
}

fn eval(egraph: &EGraph, enode: &Cad) -> Option<Cad> {
    use Cad::*;
    match enode {
        Add(args) => {
            assert_eq!(args.len(), 2);
            match (&egraph[args[0]].data.best, &egraph[args[1]].data.best) {
                (Num(f1), Num(f2)) => Some(Num(num(f1.to_f64() + f2.to_f64()))),
                _ => None,
            }
        }
        Sub(args) => {
            assert_eq!(args.len(), 2);
            match (&egraph[args[0]].data.best, &egraph[args[1]].data.best) {
                (Num(f1), Num(f2)) => Some(Num(num(f1.to_f64() - f2.to_f64()))),
                _ => None,
            }
        }
        Mul(args) => {
            assert_eq!(args.len(), 2);
            match (&egraph[args[0]].data.best, &egraph[args[1]].data.best) {
                (Num(f1), Num(f2)) => Some(Num(num(f1.to_f64() * f2.to_f64()))),
                _ => None,
            }
        }
        Div(args) => {
            assert_eq!(args.len(), 2);
            match (&egraph[args[0]].data.best, &egraph[args[1]].data.best) {
                (Num(f1), Num(f2)) => {
                    let f = f1.to_f64() / f2.to_f64();
                    if f.is_finite() && !f2.is_close(0) {
                        Some(Num(num(f)))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

impl Analysis<Cad> for MetaAnalysis {
    type Data = Meta;

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        let mut did_merge = DidMerge(false, false);
        did_merge.0 |= a.list.is_none() && b.list.is_some();
        did_merge.1 |= a.list.is_some() && b.list.is_none();
        did_merge.0 |= a.cost > b.cost;
        did_merge.1 |= a.cost < b.cost;
        if a.list.is_none() {
            a.list = b.list;
        }

        if a.cost > b.cost {
            a.cost = b.cost;
            a.best = b.best;
        }

        did_merge
    }
    fn make(egraph: &EGraph, enode: &Cad) -> Self::Data {
        let best = eval(egraph, enode).unwrap_or_else(|| enode.clone());

        let cost = CostFn.cost(enode, |id| egraph[id].data.cost);

        let list = match enode {
            Cad::Nil => Some(vec![]),
            Cad::Cons(args) => {
                assert_eq!(args.len(), 2);
                let head = std::iter::once(args[0]);
                let tail_meta = &egraph[args[1]].data;
                tail_meta
                    .list
                    .as_ref()
                    .map(|tail| head.chain(tail.iter().copied()).collect())
            }
            Cad::List(list) => Some(list.clone()),
            _ => None,
        };

        Self::Data { list, best, cost }
    }

    fn modify(egraph: &mut EGraph, id: Id) {
        let eclass = &egraph[id];
        if let Some(list1) = eclass.nodes.iter().find(|n| matches!(n, Cad::List(_))) {
            for list2 in eclass.nodes.iter().filter(|n| matches!(n, Cad::List(_))) {
                assert_eq!(
                    list1.children().len(),
                    list2.children().len(),
                    "at id {}, nodes:\n{:#?}",
                    eclass.id,
                    eclass.nodes
                )
            }
        }

        if let Some(list) = &eclass.data.list {
            let list = list.clone();
            let id2 = egraph.add(Cad::List(list));
            egraph.union(id, id2);
        }
        let eclass = &egraph[id];

        let best = &eclass.data.best;
        if best.is_leaf() {
            let best = best.clone();
            let id2 = egraph.add(best);
            egraph.union(id, id2);
        }
    }
}

pub fn println_cad(egraph: &EGraph, id: Id) {
    pub fn println_cad_impl(egraph: &EGraph, id: Id) {
        let best = &egraph[id].data.best;
        if best.is_leaf() {
            print!("{}", best);
            return;
        }
        print!("(");
        print!("{}", best);
        best.for_each(|i| {
            print!(" ");
            println_cad_impl(egraph, i);
        });
        print!(")");
    }
    println_cad_impl(egraph, id);
    println!();
}
