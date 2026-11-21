//! Coordinates: compact integer encodings of parts of the cube state.
//!
//! Owns: coordinate constants, ranking/decomposition between cubies and integers, and the
//! shared binomial helper. Each individual coordinate lives in its own submodule.
//! Must NOT: depend on move tables, solvers, or any crate outside `cubecore`.

mod cperm;
mod flip;
mod slice;
mod twist;

pub use cperm::{cubie_from_cperm, N_CPERM};
pub use flip::{cubie_from_flip, N_FLIP};
pub use slice::{cubie_from_slice, N_SLICE, SLICE_SOLVED};
pub use twist::{cubie_from_twist, N_TWIST};

/// Binomial coefficient C(n, k); 0 when k > n.
pub fn cnk(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let mut r = 1usize;
    for i in 0..k {
        r = r * (n - i) / (i + 1);
    }
    r
}
