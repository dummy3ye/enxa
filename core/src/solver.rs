//! The one trait every solver implements, so `bench` and `verify` can treat them uniformly.
//!
//! Owns: the solver interface only. Must NOT: implement a solver or depend on bench/verify.

use crate::{CubieCube, Move};

pub trait Solver {
    fn name(&self) -> &str;

    /// Return a move sequence (HTM) that takes `cube` to solved, or `None` if the solver
    /// cannot (unsolvable input, not implemented, or search limit reached).
    fn solve(&mut self, cube: &CubieCube) -> Option<Vec<Move>>;
}
