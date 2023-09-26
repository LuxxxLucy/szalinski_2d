/// Compile a source file into a fully layouted document.
///
///
use std::fmt;
use std::str::FromStr;

use crate::hyperparameters::{ABS_EPSILON, REL_EPSILON};

/// A basic data structure abstraction for floaing point number
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct Num(ordered_float::NotNan<f64>);

pub fn num(n: impl Into<Num>) -> Num {
    n.into()
}

impl Num {
    pub fn to_f64(self) -> f64 {
        self.0.into_inner()
    }

    pub fn is_close(self, other: impl Clone + Into<Num>) -> bool {
        let a = self.to_f64();
        let b = other.into().to_f64();

        let diff = (a - b).abs();
        (diff <= ABS_EPSILON)
            .then_some(())
            .or_else(|| {
                let max = a.abs().max(b.abs());
                (diff <= max * REL_EPSILON).then_some(())
            })
            .is_some()
    }
}

// conversions
impl From<f64> for Num {
    fn from(f: f64) -> Num {
        Num(f.into())
    }
}

impl From<usize> for Num {
    fn from(u: usize) -> Num {
        let f = u as f64;
        f.into()
    }
}

impl From<i32> for Num {
    fn from(i: i32) -> Num {
        let f = i as f64;
        f.into()
    }
}

// core traits
impl FromStr for Num {
    type Err = ordered_float::ParseNotNanError<std::num::ParseFloatError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f: ordered_float::NotNan<f64> = s.parse()?;
        Ok(f.into_inner().into())
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let float = self.to_f64();
        write!(f, "{}", float)
    }
}

impl fmt::Debug for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Num({})", self.to_f64())
    }
}
