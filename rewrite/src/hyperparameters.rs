// cad.rs
pub const COST_BIG_VALUE: f64 = 100_000_000.0;
pub const COST_SMALL_VALUE: f64 = 0.001;

// num.rs
pub const ABS_EPSILON: f64 = 0.0001;
pub const REL_EPSILON: f64 = 0.0001;

// rules.rs
pub const CAD_IDENTS: bool = true;
pub const INV_TRANS: bool = true;
pub const PARTITIONING: bool = true;
pub const PARTITIONING_MAX: usize = 5;
pub const AFFINE_SIGNATURE_MAX_LEN: usize = 10;
pub const STRUCTURE_MATCH_LIMIT: usize = 1000;

// solve.rs
pub const SOLVE_ROUND: f64 = 0.01;
