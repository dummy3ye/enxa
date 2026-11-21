//! enso: minimum time from scrambled state to *a* valid solution. Solution length is irrelevant.
//!
//! STATUS: milestone M3 — Variant A (Thistlethwaite chain) implemented in `variant_a`.
//! Design: docs/ENSO.md.

#![allow(clippy::needless_range_loop)]

pub mod chain;
pub mod variant_a;

pub use variant_a::{Enso, MAX_SOLUTION_LEN};
