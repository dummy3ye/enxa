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

See `docs/EXA.md` for the oracle design and `docs/ROADMAP.md` for M7–M9.
