//! exa: find the fewest-move (HTM-optimal) solution.
//!
//! Milestone M2 (this file set): a correct, simple IDA* with admissible pattern-database
//! style pruning. It is the *reference oracle*: slow on deep states, but exact.
//! Every later exa engine (bound squeeze, meet-in-the-middle, GPU) must agree with it.
//!
//! Public API: [`Exa`]. Search and pruning live in private modules.
//!
//! Owns: the curated public surface of the optimal solver.
//! Must NOT: expose search internals, implement CLI parsing, or weaken admissibility.

#![allow(clippy::needless_range_loop)]

mod heuristic;
mod ida;

pub use ida::Exa;
