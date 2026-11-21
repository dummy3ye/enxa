# benchkit

Reproducible measurement harness for `enso` and `exa`. The crate is named `benchkit`;
the directory is `bench/`.

It owns input generation, timing, percentile reporting and solution verification. It
must NOT implement a solver, cache answers, special-case seeds or change the HTM metric.
Every emitted solution is verified; failed and invalid solves are counted, not hidden.

## Commands

Run from the repository root:

```bash
cargo run --release -p benchkit -- --solver enso --n 10000 --seed 1
cargo run --release -p benchkit -- --solver exa --n 20 --depth 8 --seed 1
cargo run --release -p benchkit -- --solver enso --n 1000 --seed 1 --facelets
cargo run --release -p benchkit -- --solver enso --n 1000 --seed 1 --batch
```

Startup is printed separately. See `docs/BENCHMARKING.md` before recording any number.
