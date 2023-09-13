use std::ffi::OsStr;
use std::io::{self, Write};

extern crate walkdir;
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

extern crate rayon;
use rayon::iter::{ParallelBridge, ParallelIterator};

const RAW_DIR: &str = "raw";
const PROGRAM_DIR: &str = "program";
const REF_DIR: &str = "ref";
const REPORT_DIR: &str = "report";

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
    let trim_a = a.trim();
    let trim_b = b.trim();
    trim_a.eq(trim_b)
}

fn run(test_case: TestCase, world: &TestWorld) -> bool {
    println!("run test {}", world.id);
    let mut ok = true;

    // todo: change the program_dir into a dir_path in test case.
    let name = &test_case.name;
    // let program_path = &test_case.program_path;
    let program_path = &test_case.ref_path;
    let ref_program_path = &test_case.ref_path;

    let res_program = std::fs::read_to_string(program_path).expect("Unable to read file");
    let ref_program = std::fs::read_to_string(ref_program_path).expect("Unable to read file");

    println!("source is {}", res_program);
    println!("reference is {}", ref_program);

    if !compare(&res_program, &ref_program) {
        println!("is not equal!");
        ok = false;
    }

    let mut stdout = std::io::stdout().lock();
    // stdout.write_all(name.to_string_lossy().as_bytes()).unwrap();
    stdout.write_all(name.as_bytes()).unwrap();
    if ok {
        writeln!(stdout, " ✔").unwrap();
    } else {
        writeln!(stdout, " ❌").unwrap();
    }
    ok
}

fn main() {
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

            Some(src_path)
            // TODO: add matching and arg
            // if args.matches(&src_path) {
            //     Some(src_path)
            // } else {
            //     None
            // }
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
