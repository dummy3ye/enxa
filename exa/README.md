# exa

Fewest-move solutions in the HTM metric, with a provably optimal answer. exa is the
reference oracle; it is not the speed tool.

## Layout

- `src/main.rs` — tiny binary entry point.
- `src/cli.rs` — argument parsing and output verification.
- `src/ida.rs` — iterative-deepening search and the `Exa` solver.
- `src/heuristic.rs` — admissible projection pruning tables.

The public API is just `exa::Exa`. A heuristic is never used to prune unless it is
provably admissible.

## Commands

Run from the repository root:

```bash
cargo run --release -p exa -- --scramble "R U R' U'"
cargo run --release -p exa -- --depth 8 --seed 1
cargo test -p verify --release --test exa
cargo run --release -p benchkit -- --solver exa --n 20 --depth 8 --seed 1
```

## Measured v0.2.0 numbers

Depth curve on 20 seed-1 scrambles per depth, v3 build, 300 s deadline:
p50 rises from 70,350 ns at depth 8 to 499 ms at depth 12, 6.53 s at depth 13 and
37.75 s at depth 14 (which timed out after 6 of 20 solves). Every completed solve was
valid and had the exact scramble depth as its HTM length. Raw JSON:
`docs/data/exa-depth-{8..14}.json`. The ignored BFS truth test checks 46,997 states
against exact distances through depth 6. No external optimal-solver cross-check was
obtained; see `docs/EXA.md` and EXP-013 in `docs/EXPERIMENTS.md`.

See `docs/EXA.md` for the oracle design and `docs/ROADMAP.md` for M7–M9.
