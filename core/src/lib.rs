//! cubecore: the shared foundation of `enso` and `exa`.
//!
//! Conventions (do not change without updating docs/DESIGN.md and every test):
//! - Kociemba cubie layout and move definitions.
//! - Metric: HTM (face-turn metric). Every one of the 18 moves costs 1.
//! - Face order everywhere: U, R, F, D, L, B. Move index = face * 3 + (power - 1).
//!
//! Owns: cube conventions and re-exports of the primitives shared by both solvers.
//! Must NOT: solve, search, benchmark, or parse CLI arguments.
//!
//! This is the crate's entire public API. Implementation modules are private; only the
//! primitives needed by `enso`, `exa`, `benchkit` and `verify` are re-exported here.

#![allow(clippy::needless_range_loop)]

mod coord;
mod cubie;
mod facelet;
mod moves;
mod rng;
mod solver;
mod tables;

pub use coord::{
    cnk, cubie_from_cperm, cubie_from_flip, cubie_from_slice, cubie_from_twist, N_CPERM, N_FLIP,
    N_SLICE, N_TWIST, SLICE_SOLVED,
};
pub use cubie::{corner, edge, perm_parity, CubieCube, MOVES};
pub use facelet::{from_facelets, to_facelets};
pub use moves::{format_alg, parse_alg, random_alg, Move};
pub use rng::Rng;
pub use solver::Solver;
pub use tables::MoveTables;
