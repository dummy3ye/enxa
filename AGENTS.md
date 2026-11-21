# AGENTS.md: instructions for AI coding agents

You are working in `enxa`, a repo with **two tools and only two**:

- **enso**: solves a scrambled Rubik's Cube *as fast as possible*. Solution length does not matter.
- **exa**: finds the *fewest possible moves* (provably optimal, HTM metric).

Do not add a third tool. Do not turn either tool into the other.

## Step 0: always, before anything else

```bash
cargo build --workspace --release
cargo test  --workspace --release
```

The scaffold was written without a compiler. If this fails, **fixing it is your first and only task**. Fix compile errors with minimal edits; never "fix" a failing correctness test by changing the test or the cube conventions. If a test fails, the code is wrong until proven otherwise.

## Commands

| Task | Command |
|------|---------|
| Everything healthy? | `./scripts/check.sh` |
| Tests | `cargo test --workspace --release` |
| Solve with exa | `cargo run --release -p exa -- --scramble "R U R' U'"` |
| Benchmark | `cargo run --release -p benchkit -- --solver exa --n 100 --seed 1 --depth 8` |

After the first clean compile, make clippy clean and then change `scripts/check.sh` and CI to `-D warnings`.

## Hard rules (violating any of these is a failure)

1. **Correctness before speed.** Every solver output is verified by applying it to the input cube and checking `is_solved()`. Never skip this in tests or benchmarks.
2. **Never change the conventions** (cubie layout, move order U R F D L B, move index `face*3+power-1`, HTM metric, `a.multiply(b)` = a then b) without updating `docs/DESIGN.md` and every dependent test in the same change.
3. **No benchmark cheating.** Forbidden: caching or precomputing answers for the benchmark scrambles, special-casing seeds, timing only the easy part, hiding failures, changing the metric after seeing results, comparing a release build to a debug build. Table generation time is reported separately and honestly; it is never silently excluded from "startup" numbers.
4. **Optimality claims need proof.** exa may only output "optimal" when lower bound == upper bound by an admissible argument. Heuristics that are not provably admissible must never prune in exa. (They may *guide ordering*.)
5. **`unsafe` needs a comment** stating the invariant that makes it sound, a safe reference implementation behind a test, and a benchmark showing the gain. No gain, no `unsafe`.
6. **Core stays dependency-free.** No third-party crates in `core`. Dependencies in other crates need a written justification in the PR/commit message.
7. **No hot-path allocation, no `dyn` dispatch, no panics-on-data in enso's solve path** once M3 exists.
8. **Don't weaken tests.** You may add tests freely. Removing or loosening one requires an explanation in `docs/EXPERIMENTS.md`.
9. **Be honest in reports.** If an idea failed, say so plainly with numbers. If something is untested, say it is untested.

## Experiment protocol (this project is research, not just engineering)

The goal is to find approaches better than the standard ones, and some will fail. Failing is allowed; hiding it is not.

1. Write the hypothesis in `docs/EXPERIMENTS.md` *before* coding (what you expect, and what result would kill the idea).
2. Implement it behind a cargo feature or a separate module; the baseline must stay runnable.
3. Run the benchmark against the current best on the same seeds and the same machine. Record CPU model, flags, commit.
4. Keep, or delete, based on the numbers. Record the outcome either way.

## Conventions

- Rust stable, edition 2021, `cargo fmt` clean.
- Metric: **HTM** (all 18 face turns cost 1). Max optimal length is 20 (God's number).
- Faces in order **U R F D L B**. Move `Move(face*3 + power-1)`; power 1 = clockwise, 2 = half, 3 = counter-clockwise.
- Coordinates are `usize` at the API, `u16` in tables. Table layout: `table[coord * 18 + move]`.
- Solvers implement `cubecore::solver::Solver`.
- Determinism: all randomness goes through `cubecore::rng::Rng` with an explicit seed.

## Where things live

| You want to... | Edit |
|----------------|------|
| change cube model, moves, coordinates, tables | `core/` (and update `verify/tests/core.rs`) |
| work on the speed engine | `enso/` (read `enso/AGENTS.md`) |
| work on the optimal solver | `exa/` (read `exa/AGENTS.md`) |
| change measurement | `bench/` (and `docs/BENCHMARKING.md`) |
| add a correctness test | `verify/tests/` |
| record an idea or result | `docs/EXPERIMENTS.md` |

## Definition of done (any change)

- [ ] `./scripts/check.sh` passes
- [ ] new behavior has a test in `verify/`
- [ ] if performance-related: before/after numbers recorded with the exact command
- [ ] docs updated if a convention, format, or milestone status changed
- [ ] `docs/ROADMAP.md` checkboxes updated

## Report format when you finish a task

```
Changed:    <files/modules>
Verified:   <commands run + results>
Measured:   <numbers, with command, or "n/a">
Not done / risks: <be specific>
```

## If you are stuck

Prefer a smaller verified step over a bigger unverified one. Write a failing test that pins the bug, then fix it. If the roadmap and reality disagree, update the roadmap and say so.
