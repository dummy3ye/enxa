# enxa

The `enxa` project contains exactly two tools: `enso` and `exa`.

| Tool | Goal | Metric |
|------|------|--------|
| **enso** | Solve a scrambled Rubik's Cube as fast as physically possible. Any valid solution is fine, length is irrelevant. | wall-clock time from state to solution (latency), plus batch throughput |
| **exa** | Find the fewest possible moves for a given state: a provably optimal solution. | moves (HTM), and time-to-prove-optimal |

Language: **Rust**, zero third-party dependencies in `core`. Hand-tuned SIMD/asm only where a profiler proves it matters (`enso` feature `asm`).

## Status

**Milestone M1 done (scaffold + core). M2 baseline written. enso is a stub.** See `docs/ROADMAP.md`.

> Read this before anything else: this scaffold was authored in an environment with **no Rust toolchain**, so it has **not been compiled yet**. The cube math (move definitions, coordinates, facelets, inverse) was independently validated by a line-for-line Python port. Your first command should be:
>
> ```
> cargo test --workspace --release
> ```
>
> Fix any compile errors first (expect a few small ones). Nothing else starts before that is green.

## Layout

```
core/     shared cube model: cubies, moves, coordinates, move tables, facelets, RNG, Solver trait
enso/     speed engine (stub + DescentStage building block)
exa/      optimal solver (IDA* baseline = reference oracle)
bench/    reproducible benchmark harness (crate name: benchkit)
verify/   correctness tests, the gate for every change
docs/     design, roadmap, benchmarking rules, experiment log
```

## Quick start

```bash
cargo test  --workspace --release                       # must be green
cargo run   --release -p exa -- --scramble "R U R' U' F2 D L"
cargo run   --release -p exa -- --depth 9 --seed 1
cargo run   --release -p benchkit -- --solver exa --n 50 --depth 8 --seed 1
```

## Rules of the project

1. Correct first, fast second. A fast wrong solver is worth nothing.
2. Every claim is a measurement. No number goes in a doc without a command that reproduces it.
3. Failed experiments are results. Log them in `docs/EXPERIMENTS.md`.

Agents: read `AGENTS.md`.
