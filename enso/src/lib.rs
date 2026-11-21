//! enso: minimum time from scrambled state to *a* valid solution. Solution length is irrelevant.
//!
//! STATUS: milestone M3 — Variant A (Thistlethwaite chain) implemented as `stages`.
//! Design: docs/ENSO.md. Public API: [`Enso`] and [`MAX_SOLUTION_LEN`].
//!
//! Owns: the curated public surface of the speed solver.
//! Must NOT: expose stage internals, implement CLI parsing, or change cube conventions.

#![allow(clippy::needless_range_loop)]

mod chain;
mod solver;
mod stages;
mod subgroup;
mod tablefile;
mod tables;

pub use solver::{build_tables, Enso, MAX_SOLUTION_LEN};
