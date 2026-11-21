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

## Cutting a release

`v0.1.0` is the first numbered pre-release. The process is:

1. Ensure `Cargo.toml` has the intended workspace version, `Cargo.lock` is refreshed, and
   `CHANGELOG.md` has a dated `## [<version>]` section with Added, Changed and Known limits.
2. Run the local dry run: `./scripts/release.sh <version>`. It requires the branch to be `main`
   and the manifest and changelog versions to match, prints `git status`, runs
   `./scripts/check.sh` and the 1M gate, packages both `v2` and `v3`, writes
   `dist/SHA256SUMS`, verifies them, smoke-tests the unpacked binaries, and prints v3 benchmark
   numbers. It does not create or push a tag.
3. Read the dry-run output and `git diff` / `git status --short`, stage any new release files,
   and commit with `git commit -am "release: v<version>"`.
4. Create a signed tag if signing is configured, otherwise an annotated tag:
   `git tag -s v<version> -m "enxa v<version>" || git tag -a v<version> -m "enxa v<version>"`.
5. Push: `git push origin main v<version>`. The tag-triggered workflow verifies, builds both
   flavors, publishes checksums and attestations, and creates a pre-release.
6. Watch the workflow run, open the release page, and confirm it is marked Pre-release. Verify a
   download with `sha256sum -c SHA256SUMS`, `gh attestation verify <file> --repo <owner>/<repo>`
   and `tar xzf <file> && ./enso --version`.
7. If the workflow fails before anyone downloads anything, the tag may be deleted locally and
   remotely, the problem fixed, and the tag recreated. After any download, never move the
   published tag: release `<next-version>` instead.

## Report format

```
Changed:    <files/modules>
Verified:   <commands run + results>
Measured:   <numbers, with command, or "n/a">
Not done / risks: <be specific>
```
