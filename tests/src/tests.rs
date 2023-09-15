#[warn(non_snake_case)] // TODO: remove it

use std::ffi::OsStr;
use std::io::{self, Write};

// For Arg
extern crate clap;
use clap::Parser;

// For looping over of the file system.
extern crate walkdir;
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

extern crate rayon;
use rayon::iter::{ParallelBridge, ParallelIterator};

extern crate szalinski_egg;
// use szalinski-egg::interface::optimize;


// ==============================================================
extern crate log;
extern crate serde;
extern crate egg;

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

pub fn optimize(input: &str) -> String {

    let ITERATIONS = 50000;
    let NODE_LIMIT=3000000;
    let TIMEOUT=10;
    let PRE_EXTRACT=true;

    println!("input is {}", input);
    let initial_expr: RecExpr<_> = input.parse().expect("Couldn't parse input");

    // remove empty
    let n = (initial_expr.as_ref().len() - 1).into();
    let mut out = RecExpr::from(vec![]);
    remove_empty(&initial_expr, n, &mut out).expect("input was empty");
    let initial_expr = out;

    let initial_cost = CostFn.cost_rec(&initial_expr);

    let initial_expr = if PRE_EXTRACT {
        let pre_rules = szalinski_egg::rules::pre_rules();
        let runner = MyRunner::new(MetaAnalysis::default())
            .with_iter_limit(ITERATIONS)
            .with_node_limit(NODE_LIMIT)
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
        .with_iter_limit(ITERATIONS)
        .with_node_limit(NODE_LIMIT)
        .with_time_limit(Duration::from_secs_f64(TIMEOUT as f64))
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

    // TODO: output report
    // let out_file = std::fs::File::create(&args[2]).expect("failed to open output");
    // serde_json::to_writer_pretty(out_file, &report).unwrap();
    // String::from("run result okay")
    best.1.pretty(80)
}
// ============================================


const RAW_DIR: &str = "raw";
const PROGRAM_DIR: &str = "program";
const REF_DIR: &str = "ref";
const REPORT_DIR: &str = "report";

#[derive(Debug, Clone, Parser)]
#[clap(name = "bin-test", author)]
struct Args {
    filter: Vec<String>,
    /// runs only the specified subtest
    #[arg(long, default_value_t = false)]
    update: bool,
}

impl Args {
    fn matches(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        self.filter.is_empty() || self.filter.iter().any(|v| path.contains(v))
    }
}

/// A world that provides access to the tests environment.
#[derive(Clone)]
struct TestWorld {
    id: usize,
}

impl TestWorld {
    fn new() -> Self {
        Self {
            id: 0,
        }
    }

}

#[derive(Debug, Clone)]
pub struct TestCase {
    name: String,
    program_path: PathBuf,
    raw_path: PathBuf,
    ref_path: PathBuf,
    report_path: PathBuf,
}

fn compare(a: &str, b: &str) -> bool {
    let a = String::from(a);
    let b = String::from(b);
    let trim = |s: String| s.replace(&['\n', ' '][..], "");
    let trim_a = trim(a);
    let trim_b = trim(b);
    trim_a.eq(&trim_b)
}

fn run(test_case: TestCase, world: &TestWorld) -> bool {
    println!("run test {}", world.id);
    let mut ok = true;

    // todo: change the program_dir into a dir_path in test case.
    let name = &test_case.name;
    let program_path = &test_case.program_path;
    let ref_program_path = &test_case.ref_path;

    let src_program = std::fs::read_to_string(program_path).expect("Unable to read file");
    let res_program = optimize(&src_program);
    let ref_program = std::fs::read_to_string(ref_program_path).expect("Unable to read file");

    println!("source is {}", src_program);
    println!("result is {}", res_program);
    println!("reference is {}", ref_program);

    if !compare(&res_program, &ref_program) {
        println!("is not equal!");
        ok = false;
    }

    let mut stdout = std::io::stdout().lock();
    stdout.write_all(name.as_bytes()).unwrap();
    if ok {
        writeln!(stdout, " ✔").unwrap();
    } else {
        writeln!(stdout, " ❌").unwrap();
    }
    ok
}

fn main() {
    let args = Args::parse();

    let mut world = TestWorld::new();

    println!("Running tests...");
    let results = WalkDir::new("program")
        .into_iter()
        .par_bridge()
        .filter_map(|entry| {
            let entry = entry.unwrap();

            let src_path = entry.into_path();
            if src_path.extension() != Some(OsStr::new("txt")) {
                return None;
            }

            // TODO: add matching and arg
            if args.matches(&src_path) {
                Some(src_path)
            } else {
                None
            }
        })
        .map_with(world, |world, src_path| {
            // TODO: wrap into Ok.
            let path = src_path.strip_prefix(PROGRAM_DIR).unwrap();

            println!("the name is {}", path.display());
            let program_path = Path::new(PROGRAM_DIR).join(path);
            let raw_path = Path::new(RAW_DIR).join(path);
            let ref_path = Path::new(REF_DIR).join(path);
            let report_path = Path::new(REPORT_DIR).join(path);

            let test_case = TestCase {
                name: path.display().to_string(),
                program_path,
                raw_path,
                ref_path,
                report_path,
            };
            run(test_case, world)
        })
        .collect::<Vec<_>>();

    let len = results.len();
    let ok = results.iter().map(|_| 1).sum::<usize>();
    if len >= 1 {
        println!("{ok} / {len} tests passed.");
    } else {
        println!("{len} tests found matching the given pattern {0:#?}", args.filter);
    }

    if ok != len {
        println!(
            "Set the UPDATE_EXPECT environment variable or pass the \
             --update flag to update the reference image(s)."
        );
    }

    if ok < len {
        std::process::exit(1);
    }
}
