# Contributing

Read `AGENTS.md` before making changes. There are exactly two tools, `enso` and `exa`;
do not add a third tool or turn one into the other.

## Before a change

```bash
./scripts/check.sh
```

Record the baseline if the change can affect performance or correctness. Use the same machine,
seed and command before and after; include CPU model, rustc version and flags in the report.

## Workflow

1. Write the hypothesis in `docs/EXPERIMENTS.md` before implementing an experiment.
2. Keep the baseline runnable, ideally behind a cargo feature or in a separate module.
3. Add or preserve a correctness test in `verify/tests/`.
4. Keep public APIs small, documented and re-exported from each crate's `lib.rs`.
5. Keep every module under roughly 400 lines and split by responsibility.

## Naming

Rust source filenames use lowercase words joined together. Underscores and hyphens are
rejected by `scripts/namelint.sh`, which runs in `scripts/check.sh` and CI. Directories may
use ordinary names; crate names and Rust identifiers are unaffected.

## Definition of done

- `./scripts/check.sh` passes.
- `cargo test --workspace --release` is green with no lost tests.
- The ignored gate still runs: `cargo test -p verify --release --test gate -- --ignored --nocapture`.
- Performance-sensitive changes include before/after numbers with the exact command.
- Docs are updated when layout, conventions or milestone status change.

## Report format

```
Changed:    <files/modules>
Verified:   <commands run + results>
Measured:   <numbers, with command, or "n/a">
Not done / risks: <be specific>
```
