//! cubecore: the shared foundation of `enso` and `exa`.
//!
//! Conventions (do not change without updating docs/DESIGN.md and every test):
//! - Kociemba cubie layout and move definitions.
//! - Metric: HTM (face-turn metric). Every one of the 18 moves costs 1.
//! - Face order everywhere: U, R, F, D, L, B. Move index = face * 3 + (power - 1).

#![allow(clippy::needless_range_loop)]

pub mod coord;
pub mod cubie;
pub mod facelet;
pub mod moves;
pub mod rng;
pub mod solver;
pub mod tables;

pub use cubie::CubieCube;
pub use moves::Move;
