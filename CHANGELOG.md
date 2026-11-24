# Changelog

All notable changes are recorded here. Format follows Keep a Changelog-style headings.

## [Unreleased]

No unreleased changes.

## [0.2.0] - 2026-11-22

### Added

- Validated prebuilt enso table cache with magic/version/size/checksum, automatic rebuild on
  mismatch, `enso --build-tables`, and a byte-identical roundtrip test.
- Next-entry prefetch in the enso descent, enabled by default.
- benchkit JSON reports plus `--cube-file`, `--dump-cubes`, `--cubie`, `--lengths`,
  `--overhead`, `--verify-file` and `--timeout-ms`; machine metadata annotator and an external
  kociemba harness that writes raw JSON and solution dumps for independent verification.
- Ignored heavy BFS-to-depth-6 exa truth test in `verify/tests/truth.rs`.
- `docs/data/cubes-100k-seed1.txt`: fixed 100,000 uniformly random facelet states used by every
  fair comparison.
- Ten generated light/dark SVG charts from `docs/data/` via `python3 scripts/assets.py`, with
  `--check` enforced by `scripts/check.sh` and CI.
- Committed exa depth-8..14 JSON reports and kociemba depth-21/24 verification JSON.

### Changed

- Packed descent table entries and direct facelet coordinate descent in enso; EXPs 001-011 in
  `docs/EXPERIMENTS.md` record measured outcomes, including the rejected two-move macro and
  huge-page attempts.
- Benchmark rules and methodology rewritten in `docs/BENCHMARKING.md`; raw JSON reports live in
  `docs/data/`.
- Workspace package version is now `0.2.0`; `Cargo.lock` was refreshed with it.
- Root and crate READMEs now carry measured numbers traced to `docs/data/*.json`.
- The release workflow is manually dispatched with a version input, builds and attests both
  portable tarballs via GitHub Actions, and creates a normal Latest release. GitHub creates and
  signs the annotated tag, so no local GPG key is needed.

### Known limits

- enso returns valid but non-optimal solutions; exa is optimal but not tuned for deep random
  states. They remain two distinct tools.
- The external HTM-optimal cross-check for exa is unavailable: RubikOptimal/PyPy installed but
  table generation was cleared twice by host restarts, so no external optimal lengths are claimed
  (EXP-013).

## [0.1.0] - 2026-11-21

### Added

- First numbered pre-release of enxa with exactly two tools: `enso` (fastest valid solve) and
  `exa` (provably HTM-optimal solve).
- `--version` on `enso`, `exa` and `benchkit`, printing the binary name and package version.
- Reproducible x86-64 Linux release builds in two flavors: `v2` and `v3`.
- `scripts/package.sh`, `scripts/releasenotes.sh`, `scripts/release.sh`, `dist/SHA256SUMS`,
  and a tag-triggered GitHub Actions release workflow.
- Root and per-crate READMEs, architecture documentation, license files, `.editorconfig`,
  `rustfmt.toml`, `scripts/namelint.sh` and CI enforcement of the Rust filename rule.

### Changed

- Workspace package version is now `0.1.0`; `Cargo.lock` was refreshed with it.
- The Rust toolchain is pinned to the exact tested version, `1.97.1`.
- Release binaries are built with explicit portable `target-cpu=x86-64-v2` or
  `x86-64-v3` flags and `--locked`; they no longer inherit `target-cpu=native`.
  Native dev builds remain available with `RUSTFLAGS="-C target-cpu=native" cargo ...`.
- `enso` Variant A was split into `stages/`, `subgroup.rs`, `tables.rs`, `solver.rs`,
  `cli.rs` and a tiny `main.rs`; `exa` search/pruning/CLI were split similarly.
- `core` coordinates were split into `coord/{twist,flip,slice,cperm}.rs` behind `coord/mod.rs`.
- `verify/tests/core.rs` was split into `cube.rs`, `coords.rs`, `facelets.rs`, `exa.rs`,
  `enso.rs` and `gate.rs`. The ignored million-cube gate now runs with `--test gate`.

### Known limits

- This is a pre-release; APIs, CLI flags and artifact names may still change.
- No external two-phase solver comparison has been measured yet.
- The exa depth curve beyond depth 8 has not been measured yet.
- enso builds its stage tables at startup; prebuilt mmap-loaded tables are not implemented.
- The release workflow itself can only be exercised by pushing a tag; local dry runs cover
  packaging, checksums, smoke tests and benchmarks.
