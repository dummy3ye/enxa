# enxa

`enxa` contains exactly two tools:

| Tool | Goal | Metric |
|------|------|--------|
| **enso** | Solve a scrambled Rubik's Cube as fast as possible. Any valid solution is acceptable; length is irrelevant. | Latency (median/p99 ns) and batch throughput; startup reported separately. |
| **exa** | Find the fewest possible moves, with a provably HTM-optimal answer. | Solution length in HTM and time to prove optimality. |

Rust, edition 2021, no third-party dependencies in `core`. Randomness is deterministic
through `cubecore::Rng` with an explicit seed.

## Status

Milestone **M1** (scaffold + core) and **M2** (exa reference oracle) are complete.
Milestone **M3** (enso Variant A four-stage table descent) is implemented and verified.
enso is **not** a stub. See `docs/ROADMAP.md` for M4–M9.

## Install

Download the release tarball for your CPU from the GitHub release page:

| Flavor | File | Required CPU features | Pick it when |
|--------|------|-----------------------|--------------|
| v2 | `enxa-0.2.0-x86_64-linux-v2.tar.gz` | x86-64-v2 baseline (SSE4.2, SSSE3, POPCNT, CMPXCHG16B, LAHF/SAHF) | Older x86-64 machines or unknown CPUs |
| v3 | `enxa-0.2.0-x86_64-linux-v3.tar.gz` | x86-64-v3 baseline (AVX, AVX2, BMI1/2, FMA, LZCNT, MOVBE, XSAVE) | Modern desktop/server CPUs with AVX2 |

Check for AVX2 on Linux:

```bash
grep -o 'avx2' /proc/cpuinfo | head -1
```

If that prints `avx2`, use the v3 tarball. Otherwise use v2. Verify a download with:

```bash
sha256sum -c SHA256SUMS
gh attestation verify enxa-0.2.0-x86_64-linux-v3.tar.gz --repo <owner>/<repo>
```

The files are published on the [v0.2.0 release page](https://github.com/dummy3ye/enxa/releases/tag/v0.2.0).

## Layout

```
core/     shared, dependency-free cube model, coordinates, move tables and Solver trait
enso/     speed engine: four-stage table descent, private stages and a tiny CLI binary
exa/      optimal solver: IDA* oracle with admissible projections, private search and a tiny CLI binary
bench/    reproducible benchmark harness (crate name: benchkit)
verify/   correctness tests, including the ignored million-cube enso gate
docs/     design, architecture, benchmarking rules, roadmap and experiment log
```

## Quick start

```bash
./scripts/check.sh
cargo test --workspace --release
cargo run --release -p enso -- --scramble "R U R' U' F2 D L"
cargo run --release -p exa -- --scramble "R U R' U'"
cargo run --release -p benchkit -- --solver enso --n 10000 --seed 1
cargo run --release -p benchkit -- --solver exa --n 20 --depth 8 --seed 1
cargo test -p verify --release --test gate -- --ignored --nocapture
```

Every solver solution is verified by applying it to the input cube and checking that the
cube is solved. Benchmarks report invalid and failed solves explicitly; they are never hidden.

## Rules of the project

1. Correct first, fast second. A fast wrong solver is worth nothing.
2. Every claim is a measurement with a reproducible command.
3. Failed experiments are results; log them in `docs/EXPERIMENTS.md`.
4. Rust source filenames contain words only: no `_` and no `-`.
5. No source file exceeds roughly 400 lines.

See `docs/README.md` for documentation and `CONTRIBUTING.md` for the contribution workflow.
Agents must read `AGENTS.md` first.

## License

Licensed under either of MIT (`LICENSE-MIT`) or Apache-2.0 (`LICENSE-APACHE`), at your option.
