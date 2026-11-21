# enso

Minimum wall-clock time from a scrambled cube to **any** valid solution. Solution length
does not matter. enso is the speed tool; it is not an optimal solver.

## Layout

- `src/main.rs` — tiny binary entry point.
- `src/cli.rs` — argument parsing and output verification.
- `src/solver.rs` — `Enso`, the `Solver` implementation and fixed-buffer descent.
- `src/subgroup.rs` — subgroup predicates, generator sets and ranking helpers.
- `src/tables.rs` — startup table construction and exact G3 membership.
- `src/stages/` — `orient`, `slicecorner`, `tetrad` and `final` stage definitions.
- `src/chain.rs` — reusable stage BFS used by table construction.

The G2→G3 generator set is `<U,D,L2,R2,F2,B2>`. L/R quarter turns would leave G2.

## Commands

Run from the repository root:

```bash
cargo run --release -p enso -- --scramble "R U R' U' F2 D L"
cargo test -p verify --release --test enso
cargo test -p verify --release --test gate -- --ignored --nocapture
cargo run --release -p benchkit -- --solver enso --n 10000 --seed 1
```

The ignored gate is the 1,000,000-cube correctness check. Startup table construction is
reported separately from per-solve latency. See `docs/ENSO.md` and `docs/EXPERIMENTS.md`.
