use std::time::{Duration, Instant};

use log::*;
use serde::Serialize;

use egg::*;
use std::default::Default;
use szalinski_egg::cad::{Cad, Cost, CostFn, MetaAnalysis};
use szalinski_egg::eval::{remove_empty, Scad};
use szalinski_egg::sz_param;


#[derive(Serialize)]
pub struct RunResult {
    pub initial_expr: String,
    pub initial_cost: Cost,
    pub iterations: Vec<Iteration<MyIterData>>,
    pub final_expr: String,
    pub final_cost: Cost,
    pub extract_time: f64,
    pub final_scad: String,
    pub stop_reason: StopReason,

    // metrics
    pub ast_size: usize,
    pub ast_depth: usize,
    pub n_mapis: usize,
    pub depth_under_mapis: usize,
}

fn ast_size_impl(expr: &RecExpr<Cad>, id: Id) -> usize {
    let e = &expr[id];
    let sum_children: usize = e.children().iter().map(|e| ast_size_impl(expr, *e)).sum();
    match e {
        Cad::Vec3(_) => 1,
        _ => 1 + sum_children,
    }
}

fn ast_size(e: &RecExpr<Cad>) -> usize {
    ast_size_impl(e, (e.as_ref().len() - 1).into())
}

fn ast_depth_impl(expr: &RecExpr<Cad>, id: Id) -> usize {
    let e = &expr[id];
    let max_children = e
        .children()
        .iter()
        .map(|e| ast_depth_impl(expr, *e))
        .max()
        .unwrap_or(0);
    match e {
        Cad::Vec3(_) => 1,
        _ => 1 + max_children,
    }
}

fn ast_depth(e: &RecExpr<Cad>) -> usize {
    ast_depth_impl(e, (e.as_ref().len() - 1).into())
}

fn n_mapis_impl(expr: &RecExpr<Cad>, id: Id) -> usize {
    let e = &expr[id];
    let sum_children: usize = e.children().iter().map(|e| n_mapis_impl(expr, *e)).sum();
    sum_children
        + match e {
            Cad::MapI(_) => 1,
            _ => 0,
        }
}
fn n_mapis(e: &RecExpr<Cad>) -> usize {
    n_mapis_impl(e, (e.as_ref().len() - 1).into())
}

fn depth_under_mapis(e: &RecExpr<Cad>) -> usize {
    fn depth_under_mapis_impl(expr: &RecExpr<Cad>, id: Id) -> usize {
        let e = &expr[id];
        match e {
            Cad::MapI(_) => ast_depth_impl(expr, id) - 1,
            _ => e.children().iter().map(|e| n_mapis_impl(expr, *e)).sum(),
        }
    }
    depth_under_mapis_impl(e, (e.as_ref().len() - 1).into())
}

#[derive(Serialize)]
pub struct MyIterData {
    best_cost: Cost,
}

impl IterationData<Cad, MetaAnalysis> for MyIterData {
    fn make(runner: &MyRunner) -> Self {
        let root = runner.roots[0];
        // let best_cost = Extractor::new(&runner.egraph, CostFn).find_best(root).0;
        let best_cost = runner.egraph[root].data.cost;
        MyIterData { best_cost }
    }
}

type MyRunner = egg::Runner<Cad, MetaAnalysis, MyIterData>;

sz_param!(PRE_EXTRACT: bool);

fn main() {
    let _ = env_logger::builder().is_test(false).try_init();
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        panic!("Usage: optimize <input> <output>")
    }
    let input = std::fs::read_to_string(&args[1]).expect("failed to read input");

    sz_param!(ITERATIONS: usize);
    sz_param!(NODE_LIMIT: usize);
    sz_param!(TIMEOUT: f64);

    let initial_expr: RecExpr<_> = input.parse().expect("Couldn't parse input");

    // remove empty
    let n = (initial_expr.as_ref().len() - 1).into();
    let mut out = RecExpr::from(vec![]);
    remove_empty(&initial_expr, n, &mut out).expect("input was empty");
    let initial_expr = out;
    // yz: i want to write this
    // initial_expr.compact();

    let initial_cost = CostFn.cost_rec(&initial_expr);

    let initial_expr = if *PRE_EXTRACT {
        let pre_rules = szalinski_egg::rules::pre_rules();
        let runner = MyRunner::new(MetaAnalysis::default())
            .with_iter_limit(*ITERATIONS)
            .with_node_limit(*NODE_LIMIT)
            .with_time_limit(Duration::from_secs_f64(1.0))
            .with_expr(&initial_expr)
            .run(&pre_rules);
        Extractor::new(&runner.egraph, CostFn)
            .find_best(runner.roots[0])
            .1
    } else {
        initial_expr
    };

    let rules = szalinski_egg::rules::rules();
    let runner = MyRunner::new(MetaAnalysis::default())
        .with_iter_limit(*ITERATIONS)
        .with_node_limit(*NODE_LIMIT)
        .with_time_limit(Duration::from_secs_f64(*TIMEOUT))
        .with_scheduler(
            BackoffScheduler::default()
                .with_ban_length(5)
                .with_initial_match_limit(1_000_00),
        )
        .with_expr(&initial_expr)
        .run(&rules);

    info!(
        "Stopping after {} iters: {:?}",
        runner.iterations.len(),
        runner.stop_reason
    );

    runner.print_report();

    let root = runner.roots[0];
    let extract_time = Instant::now();
    let best = Extractor::new(&runner.egraph, CostFn).find_best(root);
    let extract_time = extract_time.elapsed().as_secs_f64();

    println!("Best ({}): {}", best.0, best.1.pretty(80));

    let report = RunResult {
        initial_expr: initial_expr.pretty(80),
        initial_cost,
        iterations: runner.iterations,
        final_cost: best.0,
        final_expr: best.1.pretty(80),
        extract_time,
        final_scad: format!("{}", Scad::new(&best.1)),
        stop_reason: runner.stop_reason.unwrap(),
        ast_size: ast_size(&best.1),
        ast_depth: ast_depth(&best.1),
        n_mapis: n_mapis(&best.1),
        depth_under_mapis: depth_under_mapis(&best.1),
    };

    let out_file = std::fs::File::create(&args[2]).expect("failed to open output");
    serde_json::to_writer_pretty(out_file, &report).unwrap();
}
