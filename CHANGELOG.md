# Changelog

All notable changes are recorded here. Format follows Keep a Changelog-style headings;
the project does not follow semantic versioning yet.

## [Unreleased]

### Added

- Structural refactor commit: module ownership docs, root and per-crate READMEs,
  `docs/README.md`, `docs/ARCHITECTURE.md`, license files, `.editorconfig`,
  `rustfmt.toml`, `scripts/namelint.sh`, and CI enforcement of the Rust filename rule.

### Changed

- `enso` Variant A was split from one 696-line file into `stages/`, `subgroup.rs`,
  `tables.rs`, `solver.rs`, `cli.rs` and a tiny `main.rs`.
- `exa` search and pruning were split into `ida.rs`, `heuristic.rs`, `cli.rs` and a tiny
  `main.rs`.
- `core` coordinates were split into `coord/{twist,flip,slice,cperm}.rs` behind
  `coord/mod.rs`.
- `verify/tests/core.rs` was split into `cube.rs`, `coords.rs`, `facelets.rs`, `exa.rs`,
  `enso.rs` and `gate.rs`. The ignored million-cube gate moved from `enso.rs` to
  `gate.rs`; run it with `--test gate`.
- Stale status and layout documentation was corrected; enso is no longer described as a stub.

### Verified

No behavior, algorithm, test assertion, benchmark metric or cube convention changed.
Baseline and post-refactor results are recorded in `docs/EXPERIMENTS.md` (EXP-002).
