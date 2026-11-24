# Changelog

All notable changes are recorded here. Format follows Keep a Changelog-style headings.

## [Unreleased]

No unreleased changes.

## [0.2.0] - 2026-11-25

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
- The release workflow is manually dispatched with a version input, builds both portable
  tarballs, verifies their signed build-provenance attestations before publishing, creates an
  annotated tag via the GitHub API, and creates a normal Latest release. The API-created tag is
  annotated but unsigned; a cryptographically signed tag would require a key registered to a
  GitHub account (EXP-014).

### Measured v0.2.0 results

Every number below comes from committed JSON in `docs/data/`; commands, CPU metadata and
decisions are in `docs/EXPERIMENTS.md` (EXP-012/013).

| measurement | p50 | p99 | mean length | evidence |
|-------------|-----|-----|-------------|----------|
| enso, cubie input, 5-run median | 1,607 ns | 3,073 ns | 31 | `docs/data/enso-cubie-100k-seed1-run*.json` |
| enso, facelet input, 5-run median | 3,564 ns | 9,712 ns | 31 | `docs/data/enso-facelets-100k-seed1-run*.json` |
| enso, batch facelets, 5-run median | 2,458 ns/solve | n/a | 31 | `docs/data/enso-facelets-batch-100k-seed1-run*.json` |
| harness floor, no solve | 21 ns | 27–29 ns | n/a | `docs/data/overhead-*-100k-seed1-run*.json` |
| kociemba 1.2.1, max_depth 24 | 15,934,020 ns | 135,265,492 ns | 20.77 | `docs/data/kociemba-d24-100k-seed1.json` |
| kociemba 1.2.1, max_depth 21 | 14,686,947 ns | 526,922,688 ns | 20.59 | `docs/data/kociemba-d21-100k-seed1.json` |
| exa depth 8 → 14 (20 scrambles each) | 70,350 ns → 37.75 s | n/a | equal to scramble depth | `docs/data/exa-depth-{8..14}.json` |

All 100,000 states were solved in every enso and kociemba run with 0 invalid; both kociemba
dumps were independently re-verified. exa's depth-14 run timed out after 6 of 20 solves and the
partial JSON was retained. Startup is 14–18 ms and is reported separately from per-solve time.

### Known limits

- enso returns valid but non-optimal solutions; exa is optimal but not tuned for deep random
  states. They remain two distinct tools.
- The external HTM-optimal cross-check for exa is unavailable: RubikOptimal/PyPy installed but
  table generation was cleared twice by host restarts, so no external optimal lengths are claimed
  (EXP-013).
- GitHub's release APIs cannot produce a cryptographically signed annotated tag without a signing
  key registered to an account. The v0.2.0 tag is GitHub-created and annotated, but unsigned;
  tarball build-provenance attestations are GitHub-signed and are verified before publishing
  (EXP-014).
- Human visual confirmation of the generated SVG/PNG previews was not obtained after two tool
  interruptions. XML validity, file-size limits and rasterization were checked machine-side only.

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
