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

`--json PATH` writes machine-readable reports; `scripts/bench_meta.py` adds commit, CPU and
cube-file metadata. `--cube-file PATH` reuses the fixed 100k facelet file, `--cubie` times
`CubieCube` input, `--batch` measures throughput, `--overhead` measures the timer/input floor,
`--lengths` emits per-solve solution lengths, `--timeout-ms N` stops a long run with partial
counts, and `--verify-file PATH` independently checks solutions produced by an external tool.

The committed 100k results and their commands are in `docs/data/` and
`docs/EXPERIMENTS.md` (EXP-012/013). Startup is printed separately. See
`docs/BENCHMARKING.md` before recording any number.
