//! enso Variant A: a four-stage Thistlethwaite-style chain solved by pure table descent.
//!
//! G0 --(all 18 moves)--> G1: edge orientation solved
//! G1 --<U,D,L,R,F2,B2>--> G2: edge + corner orientation solved, UD-slice edges in slice
//! G2 --<U,D,L2,R2,F2,B2>--> G3: paired corner/edge coset solved
//! G3 --<U2,D2,L2,R2,F2,B2>--> solved
//!
//! The G2→G3 generator set is *not* <U,D,L,R,F2,B2>: L/R quarter turns move UD-slice
//! edges out of the slice and would destroy the G2 invariant. This was derived and
//! verified independently before implementation (see docs/EXPERIMENTS.md, EXP-001).
//!
//! Owns: the four stage definitions (`orient`, `slicecorner`, `tetrad`, `final`).
//! Must NOT: build tables, run a descent, or perform I/O.
//!
//! All solve-path work is table lookups into fixed allocations owned by `Enso`. The only
//! allocation in `Solver::solve` is the `Vec<Move>` the trait signature requires; the
//! internal descent writes into a caller-supplied fixed-size buffer.

pub(crate) mod r#final;
pub(crate) mod orient;
pub(crate) mod slicecorner;
pub(crate) mod tetrad;
