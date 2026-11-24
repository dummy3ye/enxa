# enxa

`enxa` contains exactly two tools:

| Tool | Goal | Metric |
|------|------|--------|
| **enso** | Solve a scrambled Rubik's Cube as fast as possible. Any valid solution is acceptable; length is irrelevant. | Latency (median/p99 ns) and batch throughput; startup reported separately. |
| **exa** | Find the fewest possible moves, with a provably HTM-optimal answer. | Solution length in HTM and time to prove optimality. |

Rust, edition 2021, no third-party dependencies in `core`. Randomness is deterministic
through `cubecore::Rng` with an explicit seed.

## Status

**v0.2.0 is published** as the normal Latest release with signed build-provenance
attestations for both tarballs. Milestones **M1/M2/M3** are complete: the core, the exa
reference oracle and the enso four-stage table descent are implemented and verified. exa's
depth curve and independent BFS truth test are measured; the external optimal cross-check is
explicitly unavailable because repeated host restarts cleared the solver's tmpfs tables
(EXP-013). enso is **not** a stub. See `docs/ROADMAP.md` for the remaining milestones.

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
gh attestation verify enxa-0.2.0-x86_64-linux-v3.tar.gz --repo dummy3ye/enxa
```

The files are published on the [v0.2.0 release page](https://github.com/dummy3ye/enxa/releases/tag/v0.2.0).
The GitHub-created release tag is annotated but not cryptographically signed; the tarball
attestations are GitHub-signed (EXP-014).

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

## Results

Every number below is computed from committed JSON in `docs/data/`; the exact commands,
CPU metadata and experiment decisions are in `docs/EXPERIMENTS.md`. See
`docs/BENCHMARKING.md` for the methodology.

| measurement (100,000 seed-1 states) | p50 | p99 | mean length | evidence |
|-------------------------------------|-----|-----|-------------|----------|
| enso, cubie input | 1,607 ns | 3,073 ns | 31 | `docs/data/enso-cubie-100k-seed1-run*.json` |
| enso, facelet input | 3,564 ns | 9,712 ns | 31 | `docs/data/enso-facelets-100k-seed1-run*.json` |
| enso, batch facelets | 2,458 ns/solve | n/a | 31 | `docs/data/enso-facelets-batch-100k-seed1-run*.json` |
| harness floor, no solve | 21 ns | 27–29 ns | n/a | `docs/data/overhead-*-100k-seed1-run*.json` |
| kociemba 1.2.1, max_depth 24 | 15.93 ms | 135.27 ms | 20.77 | `docs/data/kociemba-d24-100k-seed1.json` |
| kociemba 1.2.1, max_depth 21 | 14.69 ms | 526.92 ms | 20.59 | `docs/data/kociemba-d21-100k-seed1.json` |

Startup (table load/build) is 14–18 ms and is reported separately, never folded into
per-solve latency. All solutions were verified: 100,000 solved and 0 invalid for every enso
run, and both kociemba dumps were independently re-verified. kociemba is an external timing
reference only; it produces shorter solutions but is not part of this repository.

![enso latency, light](docs/assets/enso-latency-light.svg#gh-light-mode-only)
![enso latency, dark](docs/assets/enso-latency-dark.svg#gh-dark-mode-only)

![enso vs kociemba, light](docs/assets/enso-vs-kociemba-light.svg#gh-light-mode-only)
![enso vs kociemba, dark](docs/assets/enso-vs-kociemba-dark.svg#gh-dark-mode-only)

![solution length, light](docs/assets/solution-length-light.svg#gh-light-mode-only)
![solution length, dark](docs/assets/solution-length-dark.svg#gh-dark-mode-only)

![exa depth curve, light](docs/assets/exa-depth-curve-light.svg#gh-light-mode-only)
![exa depth curve, dark](docs/assets/exa-depth-curve-dark.svg#gh-dark-mode-only)

![overhead floor, light](docs/assets/overhead-floor-light.svg#gh-light-mode-only)
![overhead floor, dark](docs/assets/overhead-floor-dark.svg#gh-dark-mode-only)

The charts are generated from the raw JSON by `python3 scripts/assets.py`; CI and
`./scripts/check.sh` run `python3 scripts/assets.py --check`.

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
