# enso: agent instructions (read the root `AGENTS.md` first)

**Objective:** minimum wall-clock time from a scrambled cube to *a* valid solution. Solution length is irrelevant, trade it away freely.

## The metric (do not change it)

- Latency: median and p99 ns per solve, single thread, input already in `CubieCube` form.
- Also report latency including facelet-string input, because parsing can dominate a sub-microsecond solve.
- Throughput: solves/second in batch mode (SIMD lanes / threads / GPU later).
- Startup (table build/load) is reported separately, never hidden.

## Architecture direction (see `docs/ENSO.md`)

Replace *search* with *table descent*: a chain of stages, each a precomputed best-move table over a coordinate (`src/chain.rs::DescentStage` is the building block). A solve is a few dozen dependent loads, no tree search, no branching on cube data except stage transitions.

Current layout: `src/solver.rs` owns `Enso` and the descent; `src/tables.rs` builds tables;
`src/subgroup.rs` owns subgroup predicates and generator sets; `src/stages/` owns the four
stage definitions; `src/cli.rs` owns argument parsing; `src/main.rs` is only an entry point.
The G2→G3 generator set is `<U,D,L2,R2,F2,B2>`, not L/R quarter turns.

The 1M gate lives in `../verify/tests/gate.rs`:
`cargo test -p verify --release --test gate -- --ignored --nocapture`.

## Rules specific to enso

- The hot path (`Solver::solve` internals) must not allocate, must not use `dyn`, must not panic on valid input.
- Pack hot-path table entries so one load yields both "move" and "next coordinate" where possible.
- Measure cache behavior (L1/L2/L3/DRAM) before and after any table-layout change; record in `docs/EXPERIMENTS.md`.
- SIMD/asm goes behind the `asm` feature with a safe scalar fallback that tests compare against on 1M random cubes.
- A "faster" change that makes any solution invalid is a bug, regardless of the speedup.

## Milestones: see `docs/ROADMAP.md` (M3 to M6)
