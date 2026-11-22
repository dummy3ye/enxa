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

`v0.2.0` is the current numbered release. The process is:

1. Ensure `Cargo.toml` has the intended workspace version, `Cargo.lock` is refreshed, and
   `CHANGELOG.md` has a dated `## [<version>]` section with Added, Changed and Known limits.
2. Run the local dry run: `./scripts/release.sh <version>`. It requires the branch to be `main`
   and the manifest and changelog versions to match, prints `git status`, runs
   `./scripts/check.sh` and the 1M gate, packages both `v2` and `v3`, writes
   `dist/SHA256SUMS`, verifies them, smoke-tests the unpacked binaries, and prints v3 benchmark
   numbers. It does not create or push a tag.
3. Read the dry-run output and `git diff` / `git status --short`, stage any new release files,
   and commit with `git commit -am "release: v<version>"`. Push the commit with
   `git push origin main`. Do not create or push a local tag.
4. Dispatch the `release` workflow from the Actions tab, or with
   `gh workflow run release.yml -f version=<version>`. It re-runs the health checks (including
   the 1M gate and `python3 scripts/assets.py --check`), builds both portable flavors, verifies
   checksums, smoke-tests the archives, uploads build-provenance attestations, and creates a
   **normal** release marked Latest. The GitHub release API creates the annotated tag; GitHub
   signs it, so no local GPG key is needed.
5. Watch the workflow, then confirm the release page is marked Latest with both tarballs,
   `SHA256SUMS` and attestations. Verify a download with `sha256sum -c SHA256SUMS`,
   `gh attestation verify <file> --repo dummy3ye/enxa` and `tar xzf <file> && ./enso --version`.
   Confirm the tag is verified with
   `gh api repos/dummy3ye/enxa/git/ref/tags/v<version>` followed by a `git/tags/<sha>` query.
6. If the workflow fails before anyone downloads anything, fix the problem and re-dispatch.
   After any download, never move the published tag: release `<next-version>` instead.

## Report format

```
Changed:    <files/modules>
Verified:   <commands run + results>
Measured:   <numbers, with command, or "n/a">
Not done / risks: <be specific>
```
