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

## Measured v0.2.0 numbers

On the fixed 100,000-state seed-1 facelet file (`docs/data/cubes-100k-seed1.txt`), v3 build,
the five-run medians are 1,607 ns p50 / 3,073 ns p99 for cubie input, 3,564 ns p50 /
9,712 ns p99 for facelet input, and 2,458 ns/solve in batch mode. Startup is 14–18 ms and is
reported separately. Every run solved all 100,000 states with 0 invalid; mean length is 31
moves. Raw JSON: `docs/data/enso-{cubie,facelets,batch}-100k-seed1-run*.json`. See
`docs/EXPERIMENTS.md` (EXP-012) for commands, metadata and the external kociemba comparison.

The ignored gate is the 1,000,000-cube correctness check. Startup table construction is
reported separately from per-solve latency. See `docs/ENSO.md` and `docs/EXPERIMENTS.md`.
