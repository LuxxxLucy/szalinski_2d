use std::ffi::OsStr;

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
    program_path: PathBuf,
    raw_path: PathBuf,
    ref_path: PathBuf,
    report_path: PathBuf,
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
            let path = src_path.strip_prefix(PROGRAM_DIR).unwrap();

            println!("the name is {}", path.display());
            let program_path = Path::new(PROGRAM_DIR).join(path);
            let raw_path = Path::new(RAW_DIR).join(path);
            let ref_path = Path::new(REF_DIR).join(path);
            let report_path = Path::new(REPORT_DIR).join(path);

            TestCase {
                program_path,
                raw_path,
                ref_path,
                report_path,
            }
        })
        .collect::<Vec<_>>();

    // TODO: change to actual testing function in the map_with and remove
    // the printing the below iter map for ok.
    for r in results.iter() {
        println!("test case {:?}", r);
    }

    let len = results.len();
    let ok = results.iter().map(|_| 1).sum::<usize>();
    if len > 1 {
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
