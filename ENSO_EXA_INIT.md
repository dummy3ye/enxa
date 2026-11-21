# enxa: historical initialization snapshot

> **Do not run this file.** It is preserved only as a record of the original bootstrap.
> Its embedded source, status claims and unpack instructions are stale: they describe the
> pre-M3 scaffold and would overwrite the current split layout. The current layout is
> `docs/ARCHITECTURE.md`; current status is in `README.md` and `docs/ROADMAP.md`.
>
> This snapshot says "enso is a stub" and "Rust has not been compiled". Both were true
> when it was written and are false now. EXP-001 records a clean build, 21 passing tests,
> the 1,000,000-cube gate and measured enso/exa results.

## 1. Historical unpack instructions (do not run)

The original file contained a Python extractor here. It has been removed deliberately:
running it would restore `core/src/coord.rs`, `verify/tests/core.rs`, CLI code inside both
`main.rs` files, and the old stub status. Nothing in this document should be executed.

Current healthy checks are:

```bash
./scripts/check.sh
cargo test -p verify --release --test gate -- --ignored --nocapture
cargo run --release -p exa -- --scramble "R U R' U' F2 D L"
cargo run --release -p benchkit -- --solver exa --n 50 --depth 8 --seed 1
```

## 2. Current box (historical snapshot content follows)

```
enxa/
├── README.md  AGENTS.md  CLAUDE.md          project + agent instructions
├── Cargo.toml  rust-toolchain.toml  .cargo/config.toml  rustfmt.toml  .editorconfig
├── core/      cubecore: cubies, moves, coord/, facelets, RNG, Solver trait
├── exa/       optimal solver: ida.rs, heuristic.rs, cli.rs, tiny main.rs
├── enso/      speed engine: solver.rs, subgroup.rs, tables.rs, chain.rs, stages/, cli.rs, tiny main.rs
├── bench/     benchkit: reproducible benchmark harness
├── verify/    tests: cube, coords, facelets, exa, enso, gate
├── docs/      README, ARCHITECTURE, DESIGN, ENSO, EXA, ROADMAP, BENCHMARKING, EXPERIMENTS
├── scripts/check.sh  scripts/namelint.sh
└── .github/workflows/ci.yml
```

## 3. Kickoff prompts for agents

**Agent 1: make it build (historical prompt; M0 is now complete)**
```
The original prompt said the scaffold was written without a compiler. That is no longer true:
`cargo build --workspace --release` and `cargo test --workspace --release` are green, and
clippy already runs with `-D warnings` in scripts/check.sh and CI. Current layout: docs/ARCHITECTURE.md.
```

**Agent 2: exa baseline curve (M2)**
```
Read AGENTS.md and exa/AGENTS.md. Using benchkit, measure the IDA* baseline for depth 8 through 14
(n=50, seed=1, release). Record the numbers, CPU, flags and exact commands in docs/EXPERIMENTS.md.
Then cross-check 100 scrambles against an external optimal solver if one is available. Report where
the speed cliff is. Do not optimize yet.
```

**Agent 3: enso v1 (M3)**
```
Read AGENTS.md, enso/AGENTS.md and docs/ENSO.md. Implement Variant A (Thistlethwaite chain) using
enso::chain::DescentStage. First write the coordinate definitions and a test proving each stage's
invariant, then the stages, then Enso::solve with zero allocation. Gate: 1,000,000 random cubes,
0 invalid, 0 failed. Then record latency and length numbers per docs/BENCHMARKING.md.
```

## 4. Files
| File | Lines |
|---|---|
| `README.md` | 50 |
| `AGENTS.md` | 90 |
| `CLAUDE.md` | 4 |
| `enso/AGENTS.md` | 24 |
| `exa/AGENTS.md` | 21 |
| `Cargo.toml` | 22 |
| `rust-toolchain.toml` | 3 |
| `.cargo/config.toml` | 4 |
| `.gitignore` | 5 |
| `.github/workflows/ci.yml` | 13 |
| `scripts/check.sh` | 7 |
| `core/Cargo.toml` | 10 |
| `core/src/lib.rs` | 19 |
| `core/src/rng.rs` | 26 |
| `core/src/moves.rs` | 106 |
| `core/src/cubie.rs` | 256 |
| `core/src/coord.rs` (historical; now `core/src/coord/`) | 134 |
| `core/src/tables.rs` | 45 |
| `core/src/facelet.rs` | 148 |
| `core/src/solver.rs` | 11 |
| `exa/Cargo.toml` | 14 |
| `exa/src/lib.rs` | 11 |
| `exa/src/ida.rs` | 159 |
| `exa/src/main.rs` | 87 |
| `enso/Cargo.toml` | 15 |
| `enso/src/lib.rs` | 31 |
| `enso/src/chain.rs` | 97 |
| `enso/src/main.rs` | 5 |
| `bench/Cargo.toml` | 12 |
| `bench/src/main.rs` | 86 |
| `verify/Cargo.toml` | 12 |
| `verify/src/lib.rs` | 1 |
| `verify/tests/core.rs` (historical; now `cube/coords/facelets.rs`) | 125 |
| `verify/tests/exa.rs` | 34 |
| `docs/DESIGN.md` | 41 |
| `docs/ENSO.md` | 50 |
| `docs/EXA.md` | 42 |
| `docs/ROADMAP.md` | 53 |
| `docs/BENCHMARKING.md` | 12 |
| `docs/EXPERIMENTS.md` | 21 |

---

## Project and agent instructions

### `README.md`

`````markdown
# enso + exa

Two tools. Exactly two.

| Tool | Goal | Metric |
|------|------|--------|
| **enso** | Solve a scrambled Rubik's Cube as fast as physically possible. Any valid solution is fine, length is irrelevant. | wall-clock time from state to solution (latency), plus batch throughput |
| **exa** | Find the fewest possible moves for a given state: a provably optimal solution. | moves (HTM), and time-to-prove-optimal |

Language: **Rust**, zero third-party dependencies in `core`. Hand-tuned SIMD/asm only where a profiler proves it matters (`enso` feature `asm`).

## Status

**Current status:** M1, M2 and M3 are complete. enso is **not** a stub. See `docs/ROADMAP.md`.

> Historical note: the original scaffold was authored without a Rust toolchain. It was later
> compiled and tested; the first clean build is recorded in `docs/EXPERIMENTS.md` (EXP-001).
> Start with `./scripts/check.sh` and the current layout in `docs/ARCHITECTURE.md`.

## Layout

```
core/     shared cube model: cubies, moves, coord/, move tables, facelets, RNG, Solver trait
enso/     speed engine: solver.rs, subgroup.rs, tables.rs, chain.rs, stages/, cli.rs, tiny main.rs
exa/      optimal solver: ida.rs, heuristic.rs, cli.rs, tiny main.rs
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
`````

### `AGENTS.md`

`````markdown
# AGENTS.md: instructions for AI coding agents

You are working on a repo with **two tools and only two**:

- **enso**: solves a scrambled Rubik's Cube *as fast as possible*. Solution length does not matter.
- **exa**: finds the *fewest possible moves* (provably optimal, HTM metric).

Do not add a third tool. Do not turn either tool into the other.

## Step 0: always, before anything else

```bash
cargo build --workspace --release
cargo test  --workspace --release
```

The scaffold was originally written without a toolchain, but now compiles cleanly. If this step ever fails, fixing it is your first and only task. Fix compile errors with minimal edits; never "fix" a failing correctness test by changing the test or the cube conventions. If a test fails, the code is wrong until proven otherwise.

## Commands

| Task | Command |
|------|---------|
| Everything healthy? | `./scripts/check.sh` |
| Tests | `cargo test --workspace --release` |
| enso 1M gate | `cargo test -p verify --release --test gate -- --ignored --nocapture` |
| Solve with enso | `cargo run --release -p enso -- --scramble "R U R' U'"` |
| Solve with exa | `cargo run --release -p exa -- --scramble "R U R' U'"` |
| Benchmark | `cargo run --release -p benchkit -- --solver exa --n 100 --seed 1 --depth 8` |

Clippy is already clean with `-D warnings` in both `scripts/check.sh` and CI.

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
- Solvers implement `cubecore::Solver`.
- Determinism: all randomness goes through `cubecore::Rng` with an explicit seed.

## Where things live

| You want to... | Edit |
|----------------|------|
| change cube model, moves, facelets, solver trait | `core/src/` (update `verify/tests/{cube,coords,facelets}.rs`) |
| change a coordinate encoding | `core/src/coord/<name>.rs` and `coord/mod.rs` re-exports |
| work on the speed engine | `enso/src/{solver,tables,subgroup,chain,stages}/` (read `enso/AGENTS.md`) |
| change enso CLI | `enso/src/cli.rs`; keep `enso/src/main.rs` tiny |
| work on the optimal solver | `exa/src/{ida,heuristic}.rs` (read `exa/AGENTS.md`) |
| change exa CLI | `exa/src/cli.rs`; keep `exa/src/main.rs` tiny |
| change measurement | `bench/src/main.rs` and `docs/BENCHMARKING.md` |
| add a correctness test | `verify/tests/{cube,coords,facelets,enso,exa,gate}.rs` |
| record an idea or result | `docs/EXPERIMENTS.md` |
| understand the layout | `docs/ARCHITECTURE.md` |

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
`````

### `CLAUDE.md`

`````markdown
# CLAUDE.md

Follow `AGENTS.md` in this directory. It is the single source of truth for how to work in this repo.
Nested `AGENTS.md` files in `enso/` and `exa/` add tool-specific rules.
`````

### `enso/AGENTS.md`

`````markdown
# enso: agent instructions (read the root `AGENTS.md` first)

**Objective:** minimum wall-clock time from a scrambled cube to *a* valid solution. Solution length is irrelevant, trade it away freely.

## The metric (do not change it)

- Latency: median and p99 ns per solve, single thread, input already in `CubieCube` form.
- Also report latency including facelet-string input, because parsing can dominate a sub-microsecond solve.
- Throughput: solves/second in batch mode (SIMD lanes / threads / GPU later).
- Startup (table build/load) is reported separately, never hidden.

## Architecture direction (see `docs/ENSO.md`)

Replace *search* with *table descent*: a chain of stages, each a precomputed best-move table over a coordinate (`chain::DescentStage` is the building block). A solve is a few dozen dependent loads, no tree search, no branching on cube data except stage transitions.

## Rules specific to enso

- The hot path (`Solver::solve` internals) must not allocate, must not use `dyn`, must not panic on valid input.
- Pack hot-path table entries so one load yields both "move" and "next coordinate" where possible.
- Measure cache behavior (L1/L2/L3/DRAM) before and after any table-layout change; record in `docs/EXPERIMENTS.md`.
- SIMD/asm goes behind the `asm` feature with a safe scalar fallback that tests compare against on 1M random cubes.
- A "faster" change that makes any solution invalid is a bug, regardless of the speedup.

## Milestones: see `docs/ROADMAP.md` (M3 to M6)
`````

### `exa/AGENTS.md`

`````markdown
# exa: agent instructions (read the root `AGENTS.md` first)

**Objective:** fewest possible moves. Output must be provably HTM-optimal.

## Facts to keep straight

- God's number is 20 (HTM). exa can never need more than 20. The frontier is *how fast optimality can be proven*, not shrinking below 20.
- Average optimal length of a random state is about 17.7 moves.

## The reference oracle

`exa::ida::Exa` (IDA* with three admissible projections) is the **oracle**. Every faster engine must, on a shared test set, return a solution of exactly the oracle's length. Disagreement = the new engine is wrong (or, rarely, the oracle is; investigate with a third opinion before touching it).

## Rules specific to exa

- A heuristic used for pruning must be provably admissible (never overestimates). Learned or empirical heuristics may only reorder search, never prune.
- Never report "optimal" without lower bound == upper bound. Emit both numbers.
- Any new engine is introduced as a separate module implementing `Solver`, benchmarked against the oracle on the same seeds.
- Respect the kill criteria in `docs/EXA.md`: if an idea fails them, log it and move on.

## Milestones: see `docs/ROADMAP.md` (M2, M7 to M9)
`````


---

## Workspace config, CI, scripts

### `Cargo.toml`

`````toml
[workspace]
resolver = "2"
members = ["core", "enso", "exa", "bench", "verify"]

[workspace.package]
version = "0.0.1"
edition = "2021"
license = "MIT OR Apache-2.0"
publish = false

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
debug = false

[profile.dev]
opt-level = 1

[profile.test]
opt-level = 3
`````

### `rust-toolchain.toml`

`````toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
`````

### `.cargo/config.toml`

`````toml
# Native codegen: AVX2/AVX-512/BMI2 are used when the CPU has them.
# Binaries built this way are NOT portable to older CPUs. That is intentional.
[build]
rustflags = ["-C", "target-cpu=native"]
`````

### `.gitignore`

`````
/target
*.tbl
*.bin
*.log
.DS_Store
`````

### `.github/workflows/ci.yml`

`````yaml
name: ci
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets
      - run: cargo test --workspace --release
`````

### `scripts/check.sh`

`````bash
#!/usr/bin/env bash
# One command to know whether the repo is healthy.
set -euo pipefail
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace --release
echo "OK"
`````


---

## core: shared cube foundation

### `core/Cargo.toml`

`````toml
[package]
name = "cubecore"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true
description = "Shared cube model, moves, coordinates and tables for enso and exa"

[dependencies]
# Intentionally empty. Zero third-party dependencies in core.
`````

### `core/src/lib.rs`

`````rust
//! cubecore: the shared foundation of `enso` and `exa`.
//!
//! Conventions (do not change without updating docs/DESIGN.md and every test):
//! - Kociemba cubie layout and move definitions.
//! - Metric: HTM (face-turn metric). Every one of the 18 moves costs 1.
//! - Face order everywhere: U, R, F, D, L, B. Move index = face * 3 + (power - 1).

#![allow(clippy::needless_range_loop)]

pub mod coord;
pub mod cubie;
pub mod facelet;
pub mod moves;
pub mod rng;
pub mod solver;
pub mod tables;

pub use cubie::CubieCube;
pub use moves::Move;
`````

### `core/src/rng.rs`

`````rust
//! Tiny deterministic RNG (xorshift64*). Deterministic seeds make benchmarks reproducible.

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform-ish integer in `0..n` (multiply-shift; bias is negligible for small n).
    #[inline]
    pub fn below(&mut self, n: u64) -> u64 {
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }
}
`````

### `core/src/moves.rs`

`````rust
//! The 18 HTM moves.

use crate::rng::Rng;
use std::fmt;

/// A move, encoded as `face * 3 + (power - 1)`.
/// Faces: U=0, R=1, F=2, D=3, L=4, B=5. Power: 1 = clockwise, 2 = half turn, 3 = counter-clockwise.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Move(pub u8);

impl Move {
    pub const COUNT: usize = 18;
    pub const FACES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

    #[inline]
    pub fn new(face: u8, power: u8) -> Move {
        debug_assert!(face < 6 && (1..=3).contains(&power));
        Move(face * 3 + power - 1)
    }

    #[inline]
    pub fn face(self) -> u8 {
        self.0 / 3
    }

    #[inline]
    pub fn power(self) -> u8 {
        self.0 % 3 + 1
    }

    #[inline]
    pub fn inverse(self) -> Move {
        Move::new(self.face(), 4 - self.power())
    }

    pub fn all() -> impl Iterator<Item = Move> {
        (0..18u8).map(Move)
    }

    /// Redundancy pruning: no same-face repeats; opposite faces only in one canonical order
    /// (U before D, R before L, F before B), because they commute.
    #[inline]
    pub fn can_follow(self, last: Option<Move>) -> bool {
        match last {
            None => true,
            Some(l) => {
                let (f, lf) = (self.face(), l.face());
                f != lf && !(f % 3 == lf % 3 && f < lf)
            }
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = Self::FACES[self.face() as usize];
        match self.power() {
            1 => write!(f, "{c}"),
            2 => write!(f, "{c}2"),
            _ => write!(f, "{c}'"),
        }
    }
}

/// Parse e.g. "R U R' U' F2".
pub fn parse_alg(s: &str) -> Result<Vec<Move>, String> {
    let mut out = Vec::new();
    for tok in s.split_whitespace() {
        let mut chars = tok.chars();
        let face_ch = chars.next().ok_or("empty token")?;
        let face = Move::FACES
            .iter()
            .position(|&c| c == face_ch)
            .ok_or_else(|| format!("bad face in token '{tok}'"))? as u8;
        let power = match chars.as_str() {
            "" => 1,
            "2" => 2,
            "'" => 3,
            _ => return Err(format!("bad suffix in token '{tok}'")),
        };
        out.push(Move::new(face, power));
    }
    Ok(out)
}

pub fn format_alg(moves: &[Move]) -> String {
    moves
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A random move sequence of `len` moves that respects `can_follow`.
pub fn random_alg(rng: &mut Rng, len: usize) -> Vec<Move> {
    let mut out = Vec::with_capacity(len);
    let mut last = None;
    while out.len() < len {
        let m = Move(rng.below(18) as u8);
        if m.can_follow(last) {
            out.push(m);
            last = Some(m);
        }
    }
    out
}
`````

### `core/src/cubie.rs`

`````rust
//! Cubie-level model. Kociemba conventions.
//!
//! Corners: URF UFL ULB UBR DFR DLF DBL DRB (0..8)
//! Edges:   UR UF UL UB DR DF DL DB FR FL BL BR (0..12)
//!
//! `a.multiply(&b)` means "apply a, then apply b".

use crate::moves::Move;
use crate::rng::Rng;

pub mod corner {
    pub const URF: u8 = 0;
    pub const UFL: u8 = 1;
    pub const ULB: u8 = 2;
    pub const UBR: u8 = 3;
    pub const DFR: u8 = 4;
    pub const DLF: u8 = 5;
    pub const DBL: u8 = 6;
    pub const DRB: u8 = 7;
}

pub mod edge {
    pub const UR: u8 = 0;
    pub const UF: u8 = 1;
    pub const UL: u8 = 2;
    pub const UB: u8 = 3;
    pub const DR: u8 = 4;
    pub const DF: u8 = 5;
    pub const DL: u8 = 6;
    pub const DB: u8 = 7;
    pub const FR: u8 = 8;
    pub const FL: u8 = 9;
    pub const BL: u8 = 10;
    pub const BR: u8 = 11;
}

use self::corner::*;
use self::edge::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubieCube {
    /// Which corner cubie sits in each corner slot.
    pub cp: [u8; 8],
    /// Corner twist 0..3 per slot.
    pub co: [u8; 8],
    /// Which edge cubie sits in each edge slot.
    pub ep: [u8; 12],
    /// Edge flip 0..2 per slot.
    pub eo: [u8; 12],
}

impl CubieCube {
    pub const SOLVED: CubieCube = CubieCube {
        cp: [0, 1, 2, 3, 4, 5, 6, 7],
        co: [0; 8],
        ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        eo: [0; 12],
    };

    /// Group multiplication: apply `self`, then `b`.
    pub const fn multiply(&self, b: &CubieCube) -> CubieCube {
        let mut r = CubieCube::SOLVED;
        let mut i = 0;
        while i < 8 {
            let j = b.cp[i] as usize;
            r.cp[i] = self.cp[j];
            r.co[i] = (self.co[j] + b.co[i]) % 3;
            i += 1;
        }
        let mut i = 0;
        while i < 12 {
            let j = b.ep[i] as usize;
            r.ep[i] = self.ep[j];
            r.eo[i] = (self.eo[j] + b.eo[i]) % 2;
            i += 1;
        }
        r
    }

    pub fn inverse(&self) -> CubieCube {
        let mut r = CubieCube::SOLVED;
        for j in 0..8 {
            let p = self.cp[j] as usize;
            r.cp[p] = j as u8;
            r.co[p] = (3 - self.co[j]) % 3;
        }
        for j in 0..12 {
            let p = self.ep[j] as usize;
            r.ep[p] = j as u8;
            r.eo[p] = self.eo[j];
        }
        r
    }

    #[inline]
    pub fn apply(&self, m: Move) -> CubieCube {
        self.multiply(&MOVES[m.0 as usize])
    }

    pub fn apply_all(&self, moves: &[Move]) -> CubieCube {
        let mut c = *self;
        for &m in moves {
            c = c.apply(m);
        }
        c
    }

    #[inline]
    pub fn is_solved(&self) -> bool {
        *self == CubieCube::SOLVED
    }

    /// All 12 edges flipped, everything else solved. Known to need exactly 20 HTM moves.
    pub fn superflip() -> CubieCube {
        let mut c = CubieCube::SOLVED;
        c.eo = [1; 12];
        c
    }

    /// Is this a physically reachable cube?
    pub fn verify(&self) -> Result<(), &'static str> {
        let mut seen = [false; 8];
        for &c in &self.cp {
            if c >= 8 || seen[c as usize] {
                return Err("invalid corner permutation");
            }
            seen[c as usize] = true;
        }
        let mut seen = [false; 12];
        for &e in &self.ep {
            if e >= 12 || seen[e as usize] {
                return Err("invalid edge permutation");
            }
            seen[e as usize] = true;
        }
        if self.co.iter().any(|&x| x >= 3) || self.co.iter().map(|&x| x as u32).sum::<u32>() % 3 != 0 {
            return Err("corner twist is not solvable");
        }
        if self.eo.iter().any(|&x| x >= 2) || self.eo.iter().map(|&x| x as u32).sum::<u32>() % 2 != 0 {
            return Err("edge flip is not solvable");
        }
        if perm_parity(&self.cp) != perm_parity(&self.ep) {
            return Err("permutation parity mismatch");
        }
        Ok(())
    }

    /// A uniformly random solvable cube.
    pub fn random(rng: &mut Rng) -> CubieCube {
        let mut c = CubieCube::SOLVED;
        shuffle(&mut c.cp, rng);
        shuffle(&mut c.ep, rng);
        if perm_parity(&c.cp) != perm_parity(&c.ep) {
            c.ep.swap(0, 1);
        }
        let mut s = 0u8;
        for i in 0..7 {
            c.co[i] = rng.below(3) as u8;
            s += c.co[i];
        }
        c.co[7] = (3 - s % 3) % 3;
        let mut s = 0u8;
        for i in 0..11 {
            c.eo[i] = rng.below(2) as u8;
            s += c.eo[i];
        }
        c.eo[11] = s % 2;
        c
    }
}

fn shuffle<const N: usize>(a: &mut [u8; N], rng: &mut Rng) {
    for i in (1..N).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        a.swap(i, j);
    }
}

pub fn perm_parity(p: &[u8]) -> u8 {
    let mut c = 0u8;
    for i in 0..p.len() {
        for j in 0..i {
            if p[j] > p[i] {
                c ^= 1;
            }
        }
    }
    c
}

// ---- the six basic clockwise face turns (U R F D L B) ----

const BASE: [CubieCube; 6] = [
    // U
    CubieCube {
        cp: [UBR, URF, UFL, ULB, DFR, DLF, DBL, DRB],
        co: [0; 8],
        ep: [UB, UR, UF, UL, DR, DF, DL, DB, FR, FL, BL, BR],
        eo: [0; 12],
    },
    // R
    CubieCube {
        cp: [DFR, UFL, ULB, URF, DRB, DLF, DBL, UBR],
        co: [2, 0, 0, 1, 1, 0, 0, 2],
        ep: [FR, UF, UL, UB, BR, DF, DL, DB, DR, FL, BL, UR],
        eo: [0; 12],
    },
    // F
    CubieCube {
        cp: [UFL, DLF, ULB, UBR, URF, DFR, DBL, DRB],
        co: [1, 2, 0, 0, 2, 1, 0, 0],
        ep: [UR, FL, UL, UB, DR, FR, DL, DB, UF, DF, BL, BR],
        eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
    },
    // D
    CubieCube {
        cp: [URF, UFL, ULB, UBR, DLF, DBL, DRB, DFR],
        co: [0; 8],
        ep: [UR, UF, UL, UB, DF, DL, DB, DR, FR, FL, BL, BR],
        eo: [0; 12],
    },
    // L
    CubieCube {
        cp: [URF, ULB, DBL, UBR, DFR, UFL, DLF, DRB],
        co: [0, 1, 2, 0, 0, 2, 1, 0],
        ep: [UR, UF, BL, UB, DR, DF, FL, DB, FR, UL, DL, BR],
        eo: [0; 12],
    },
    // B
    CubieCube {
        cp: [URF, UFL, UBR, DRB, DFR, DLF, ULB, DBL],
        co: [0, 0, 1, 2, 0, 0, 2, 1],
        ep: [UR, UF, UL, BR, DR, DF, DL, BL, FR, FL, UB, DB],
        eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
    },
];

const fn build_moves() -> [CubieCube; 18] {
    let mut out = [CubieCube::SOLVED; 18];
    let mut f = 0;
    while f < 6 {
        let b = BASE[f];
        let mut cur = b;
        let mut p = 0;
        while p < 3 {
            out[f * 3 + p] = cur;
            cur = cur.multiply(&b);
            p += 1;
        }
        f += 1;
    }
    out
}

/// All 18 moves as cubes, indexed by `Move.0`. Computed at compile time.
pub const MOVES: [CubieCube; 18] = build_moves();
`````

### `core/src/coord.rs` (historical; now split into `core/src/coord/`)

`````rust
//! Coordinates: compact integer encodings of parts of the cube state.

use crate::cubie::CubieCube;

pub const N_TWIST: usize = 2187; // 3^7
pub const N_FLIP: usize = 2048; // 2^11
pub const N_SLICE: usize = 495; // C(12,4): positions of the 4 UD-slice edges
pub const N_CPERM: usize = 40320; // 8!
pub const SLICE_SOLVED: usize = 494; // slice edges FR FL BL BR sit in slots 8..12

/// Binomial coefficient C(n, k); 0 when k > n.
pub fn cnk(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let mut r = 1usize;
    for i in 0..k {
        r = r * (n - i) / (i + 1);
    }
    r
}

impl CubieCube {
    pub fn twist(&self) -> usize {
        let mut t = 0usize;
        for i in 0..7 {
            t = t * 3 + self.co[i] as usize;
        }
        t
    }

    pub fn flip(&self) -> usize {
        let mut f = 0usize;
        for i in 0..11 {
            f = f * 2 + self.eo[i] as usize;
        }
        f
    }

    /// Combinadic rank of the slots that hold UD-slice edges (edge ids 8..12).
    pub fn slice_pos(&self) -> usize {
        let mut rank = 0usize;
        let mut k = 0usize;
        for pos in 0..12 {
            if self.ep[pos] >= 8 {
                rank += cnk(pos, k + 1);
                k += 1;
            }
        }
        rank
    }

    /// Lehmer-style rank of the corner permutation.
    pub fn cperm(&self) -> usize {
        let p = &self.cp;
        let mut r = 0usize;
        let mut fact = 1usize;
        for i in 1..8 {
            let mut s = 0usize;
            for j in 0..i {
                if p[j] > p[i] {
                    s += 1;
                }
            }
            r += s * fact;
            fact *= i + 1;
        }
        r
    }
}

pub fn cubie_from_twist(mut t: usize) -> CubieCube {
    let mut c = CubieCube::SOLVED;
    let mut sum = 0u8;
    for i in (0..7).rev() {
        let d = (t % 3) as u8;
        c.co[i] = d;
        sum += d;
        t /= 3;
    }
    c.co[7] = (3 - sum % 3) % 3;
    c
}

pub fn cubie_from_flip(mut f: usize) -> CubieCube {
    let mut c = CubieCube::SOLVED;
    let mut sum = 0u8;
    for i in (0..11).rev() {
        let d = (f % 2) as u8;
        c.eo[i] = d;
        sum += d;
        f /= 2;
    }
    c.eo[11] = sum % 2;
    c
}

pub fn cubie_from_slice(mut rank: usize) -> CubieCube {
    let mut c = CubieCube::SOLVED;
    let mut is_slice = [false; 12];
    for k in (1..=4).rev() {
        let mut p = 11usize;
        while cnk(p, k) > rank {
            p -= 1;
        }
        rank -= cnk(p, k);
        is_slice[p] = true;
    }
    let (mut s, mut o) = (8u8, 0u8);
    for pos in 0..12 {
        if is_slice[pos] {
            c.ep[pos] = s;
            s += 1;
        } else {
            c.ep[pos] = o;
            o += 1;
        }
    }
    c
}

pub fn cubie_from_cperm(mut r: usize) -> CubieCube {
    let mut inv = [0usize; 8];
    for (i, slot) in inv.iter_mut().enumerate().skip(1) {
        *slot = r % (i + 1);
        r /= i + 1;
    }
    let mut avail: Vec<u8> = (0..8).collect();
    let mut c = CubieCube::SOLVED;
    for i in (0..8).rev() {
        c.cp[i] = avail.remove(i - inv[i]);
    }
    c
}
`````

### `core/src/tables.rs`

`````rust
//! Move tables: `table[coord * 18 + move] -> next coord`.
//! A table is generated by decoding a coordinate to a representative cube,
//! applying the move, and re-encoding.

use crate::coord::*;
use crate::cubie::{CubieCube, MOVES};

pub struct MoveTables {
    pub twist: Vec<u16>,
    pub flip: Vec<u16>,
    pub slice: Vec<u16>,
    pub cperm: Vec<u16>,
}

fn build(
    n: usize,
    decode: impl Fn(usize) -> CubieCube,
    encode: impl Fn(&CubieCube) -> usize,
) -> Vec<u16> {
    let mut t = vec![0u16; n * 18];
    for x in 0..n {
        let c = decode(x);
        for (m, mv) in MOVES.iter().enumerate() {
            t[x * 18 + m] = encode(&c.multiply(mv)) as u16;
        }
    }
    t
}

impl MoveTables {
    pub fn new() -> Self {
        MoveTables {
            twist: build(N_TWIST, cubie_from_twist, |c| c.twist()),
            flip: build(N_FLIP, cubie_from_flip, |c| c.flip()),
            slice: build(N_SLICE, cubie_from_slice, |c| c.slice_pos()),
            cperm: build(N_CPERM, cubie_from_cperm, |c| c.cperm()),
        }
    }
}

impl Default for MoveTables {
    fn default() -> Self {
        Self::new()
    }
}
`````

### `core/src/facelet.rs`

`````rust
//! 54-char facelet strings. Order: U R F D L B, nine stickers each, as in Kociemba's tools.
//! Solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB".

use crate::cubie::CubieCube;

const CORNER_FACELET: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];
const EDGE_FACELET: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];
// Colors as face indices: U=0 R=1 F=2 D=3 L=4 B=5
const CORNER_COLOR: [[u8; 3]; 8] = [
    [0, 1, 2],
    [0, 2, 4],
    [0, 4, 5],
    [0, 5, 1],
    [3, 2, 1],
    [3, 4, 2],
    [3, 5, 4],
    [3, 1, 5],
];
const EDGE_COLOR: [[u8; 2]; 12] = [
    [0, 1],
    [0, 2],
    [0, 4],
    [0, 5],
    [3, 1],
    [3, 2],
    [3, 4],
    [3, 5],
    [2, 1],
    [2, 4],
    [5, 4],
    [5, 1],
];
const LETTERS: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

pub fn to_facelets(c: &CubieCube) -> String {
    let mut f = [0u8; 54];
    for (i, slot) in f.iter_mut().enumerate() {
        *slot = (i / 9) as u8;
    }
    for i in 0..8 {
        let p = c.cp[i] as usize;
        let o = c.co[i] as usize;
        for n in 0..3 {
            f[CORNER_FACELET[i][(n + o) % 3]] = CORNER_COLOR[p][n];
        }
    }
    for i in 0..12 {
        let p = c.ep[i] as usize;
        let o = c.eo[i] as usize;
        for n in 0..2 {
            f[EDGE_FACELET[i][(n + o) % 2]] = EDGE_COLOR[p][n];
        }
    }
    f.iter().map(|&x| LETTERS[x as usize]).collect()
}

pub fn from_facelets(s: &str) -> Result<CubieCube, String> {
    let chars: Vec<char> = s.trim().chars().collect();
    if chars.len() != 54 {
        return Err(format!("expected 54 facelets, got {}", chars.len()));
    }
    let mut f = [0u8; 54];
    for (i, ch) in chars.iter().enumerate() {
        f[i] = LETTERS
            .iter()
            .position(|l| l == ch)
            .ok_or_else(|| format!("bad facelet character '{ch}'"))? as u8;
    }
    for face in 0..6 {
        if f[face * 9 + 4] != face as u8 {
            return Err("center facelets are not in U R F D L B order".into());
        }
    }
    let mut c = CubieCube::SOLVED;
    for i in 0..8 {
        let mut ori = 0usize;
        while ori < 3 {
            let col = f[CORNER_FACELET[i][ori]];
            if col == 0 || col == 3 {
                break;
            }
            ori += 1;
        }
        if ori == 3 {
            return Err(format!("corner slot {i} has no U/D sticker"));
        }
        let c1 = f[CORNER_FACELET[i][(ori + 1) % 3]];
        let c2 = f[CORNER_FACELET[i][(ori + 2) % 3]];
        let mut found = false;
        for j in 0..8 {
            if CORNER_COLOR[j][1] == c1 && CORNER_COLOR[j][2] == c2 {
                c.cp[i] = j as u8;
                c.co[i] = ori as u8;
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!("corner slot {i} has an impossible color pair"));
        }
    }
    for i in 0..12 {
        let a = f[EDGE_FACELET[i][0]];
        let b = f[EDGE_FACELET[i][1]];
        let mut found = false;
        for j in 0..12 {
            if a == EDGE_COLOR[j][0] && b == EDGE_COLOR[j][1] {
                c.ep[i] = j as u8;
                c.eo[i] = 0;
                found = true;
                break;
            }
            if a == EDGE_COLOR[j][1] && b == EDGE_COLOR[j][0] {
                c.ep[i] = j as u8;
                c.eo[i] = 1;
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!("edge slot {i} has an impossible color pair"));
        }
    }
    c.verify().map_err(|e| e.to_string())?;
    Ok(c)
}
`````

### `core/src/solver.rs`

`````rust
//! The one trait every solver implements, so `bench` and `verify` can treat them uniformly.

use crate::{CubieCube, Move};

pub trait Solver {
    fn name(&self) -> &str;

    /// Return a move sequence (HTM) that takes `cube` to solved, or `None` if the solver
    /// cannot (unsolvable input, not implemented, or search limit reached).
    fn solve(&mut self, cube: &CubieCube) -> Option<Vec<Move>>;
}
`````


---

## exa: optimal solver

### `exa/Cargo.toml`

`````toml
[package]
name = "exa"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true
description = "exa: fewest-move (optimal) Rubik's Cube solver"

[dependencies]
cubecore = { path = "../core" }

[features]
default = []
# Future: gpu = []   (CUDA/wgpu meet-in-the-middle backend)
`````

### `exa/src/lib.rs`

`````rust
//! exa: find the fewest-move (HTM-optimal) solution.
//!
//! Milestone M2 (this file set): a correct, simple IDA* with admissible pattern-database
//! style pruning. It is the *reference oracle*: slow on deep states, but exact.
//! Every later exa engine (bound squeeze, meet-in-the-middle, GPU) must agree with it.

#![allow(clippy::needless_range_loop)]

pub mod ida;

pub use ida::Exa;
`````

### `exa/src/ida.rs`

`````rust
//! IDA* baseline.
//!
//! Heuristic h(state) = max of three admissible lower bounds, each the exact distance to
//! solved in a *projection* of the cube (projections can only be easier than the real cube,
//! so they never overestimate):
//!   - (corner twist, UD-slice position)   2187 * 495 entries
//!   - (edge flip,    UD-slice position)   2048 * 495 entries
//!   - corner permutation                  40320 entries

use cubecore::coord::{N_CPERM, N_FLIP, N_SLICE, N_TWIST, SLICE_SOLVED};
use cubecore::solver::Solver;
use cubecore::tables::MoveTables;
use cubecore::{CubieCube, Move};

/// God's number in HTM. No solvable cube needs more.
const GODS_NUMBER: u8 = 20;

#[derive(Copy, Clone)]
struct Coords {
    t: usize,
    f: usize,
    s: usize,
    p: usize,
}

pub struct Exa {
    mt: MoveTables,
    p_ts: Vec<u8>,
    p_fs: Vec<u8>,
    p_cp: Vec<u8>,
    /// Nodes expanded by the last call to `solve` (for benchmarking and experiments).
    pub nodes: u64,
}

/// BFS over the product of two coordinates, starting from the solved pair.
fn build_prune(na: usize, ma: &[u16], nb: usize, mb: &[u16], goal_a: usize, goal_b: usize) -> Vec<u8> {
    let mut dist = vec![u8::MAX; na * nb];
    let start = goal_a * nb + goal_b;
    dist[start] = 0;
    let mut frontier = vec![start];
    let mut depth: u8 = 0;
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for &idx in &frontier {
            let (a, b) = (idx / nb, idx % nb);
            for m in 0..18 {
                let a2 = ma[a * 18 + m] as usize;
                let b2 = mb[b * 18 + m] as usize;
                let i2 = a2 * nb + b2;
                if dist[i2] == u8::MAX {
                    dist[i2] = depth + 1;
                    next.push(i2);
                }
            }
        }
        frontier = next;
        depth += 1;
    }
    dist
}

impl Exa {
    pub fn new() -> Self {
        let mt = MoveTables::new();
        let p_ts = build_prune(N_TWIST, &mt.twist, N_SLICE, &mt.slice, 0, SLICE_SOLVED);
        let p_fs = build_prune(N_FLIP, &mt.flip, N_SLICE, &mt.slice, 0, SLICE_SOLVED);
        let trivial = vec![0u16; 18];
        let p_cp = build_prune(N_CPERM, &mt.cperm, 1, &trivial, 0, 0);
        Exa { mt, p_ts, p_fs, p_cp, nodes: 0 }
    }

    #[inline]
    fn h(&self, c: &Coords) -> u8 {
        let a = self.p_ts[c.t * N_SLICE + c.s];
        let b = self.p_fs[c.f * N_SLICE + c.s];
        let d = self.p_cp[c.p];
        a.max(b).max(d)
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs(
        &self,
        cube: &CubieCube,
        c: Coords,
        g: u8,
        limit: u8,
        last: Option<Move>,
        path: &mut Vec<Move>,
        nodes: &mut u64,
    ) -> bool {
        *nodes += 1;
        let h = self.h(&c);
        if g + h > limit {
            return false;
        }
        if h == 0 && cube.is_solved() {
            return true;
        }
        if g == limit {
            return false;
        }
        for m in Move::all() {
            if !m.can_follow(last) {
                continue;
            }
            let mi = m.0 as usize;
            let nc = Coords {
                t: self.mt.twist[c.t * 18 + mi] as usize,
                f: self.mt.flip[c.f * 18 + mi] as usize,
                s: self.mt.slice[c.s * 18 + mi] as usize,
                p: self.mt.cperm[c.p * 18 + mi] as usize,
            };
            let next = cube.apply(m);
            path.push(m);
            if self.dfs(&next, nc, g + 1, limit, Some(m), path, nodes) {
                return true;
            }
            path.pop();
        }
        false
    }
}

impl Default for Exa {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver for Exa {
    fn name(&self) -> &str {
        "exa-ida-baseline"
    }

    fn solve(&mut self, cube: &CubieCube) -> Option<Vec<Move>> {
        if cube.verify().is_err() {
            return None;
        }
        let c = Coords {
            t: cube.twist(),
            f: cube.flip(),
            s: cube.slice_pos(),
            p: cube.cperm(),
        };
        let mut nodes = 0u64;
        let mut limit = self.h(&c);
        let mut result = None;
        while limit <= GODS_NUMBER {
            let mut path = Vec::with_capacity(limit as usize);
            if self.dfs(cube, c, 0, limit, None, &mut path, &mut nodes) {
                result = Some(path);
                break;
            }
            limit += 1;
        }
        self.nodes = nodes;
        result
    }
}
`````

### `exa/src/main.rs`

`````rust
use cubecore::facelet::from_facelets;
use cubecore::moves::{format_alg, parse_alg, random_alg};
use cubecore::rng::Rng;
use cubecore::solver::Solver;
use cubecore::CubieCube;
use exa::Exa;
use std::time::Instant;

fn usage() -> ! {
    eprintln!(
        "exa: optimal (fewest-move, HTM) solver\n\n\
         usage:\n  \
         exa --scramble \"R U R' U'\"\n  \
         exa --facelets <54 chars, order U R F D L B>\n  \
         exa --depth N [--seed S]      random-walk scramble of N moves\n\n\
         note: the baseline IDA* is exact but slow beyond ~12-13 moves (see docs/EXA.md)."
    );
    std::process::exit(2)
}

fn arg(args: &[String], i: usize) -> &str {
    args.get(i).map(|s| s.as_str()).unwrap_or_else(|| usage())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let mut cube: Option<CubieCube> = None;
    let mut depth: Option<usize> = None;
    let mut seed = 1u64;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--scramble" => {
                i += 1;
                let alg = parse_alg(arg(&args, i)).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    usage()
                });
                cube = Some(CubieCube::SOLVED.apply_all(&alg));
            }
            "--facelets" => {
                i += 1;
                cube = Some(from_facelets(arg(&args, i)).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    usage()
                }));
            }
            "--depth" => {
                i += 1;
                depth = Some(arg(&args, i).parse().unwrap_or_else(|_| usage()));
            }
            "--seed" => {
                i += 1;
                seed = arg(&args, i).parse().unwrap_or_else(|_| usage());
            }
            _ => usage(),
        }
        i += 1;
    }
    if let Some(d) = depth {
        let mut rng = Rng::new(seed);
        let alg = random_alg(&mut rng, d);
        println!("scramble: {}", format_alg(&alg));
        cube = Some(CubieCube::SOLVED.apply_all(&alg));
    }
    let cube = cube.unwrap_or_else(|| usage());

    let t0 = Instant::now();
    let mut solver = Exa::new();
    println!("tables built in {:.2?}", t0.elapsed());

    let t1 = Instant::now();
    match solver.solve(&cube) {
        Some(sol) => {
            assert!(cube.apply_all(&sol).is_solved(), "BUG: solution does not solve the cube");
            println!("optimal ({} moves, HTM): {}", sol.len(), format_alg(&sol));
            println!("nodes: {}  time: {:.2?}", solver.nodes, t1.elapsed());
        }
        None => {
            eprintln!("no solution found (invalid cube or search limit)");
            std::process::exit(1);
        }
    }
}
`````


---

## enso: speed engine

### `enso/Cargo.toml`

`````toml
[package]
name = "enso"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true
description = "enso: absolute-fastest Rubik's Cube solver (any valid solution, minimum latency)"

[dependencies]
cubecore = { path = "../core" }

[features]
default = []
# Opt-in hand-written SIMD / asm kernels. Only add after profiling proves a hot spot.
asm = []
`````

### `enso/src/lib.rs`

`````rust
//! enso: minimum time from scrambled state to *a* valid solution. Solution length is irrelevant.
//!
//! STATUS: scaffold. `Enso::solve` returns `None` until milestone M3 lands.
//! Design: docs/ENSO.md.

#![allow(clippy::needless_range_loop)]

pub mod chain;

use cubecore::solver::Solver;
use cubecore::{CubieCube, Move};

#[derive(Default)]
pub struct Enso;

impl Enso {
    pub fn new() -> Self {
        Enso
    }
}

impl Solver for Enso {
    fn name(&self) -> &str {
        "enso"
    }

    // TODO(M3): stage chain = table descent, no search.
    fn solve(&mut self, _cube: &CubieCube) -> Option<Vec<Move>> {
        None
    }
}
`````

### `enso/src/chain.rs`

`````rust
//! Building block for enso: a *descent stage*.
//!
//! Given a coordinate space, a move table and a set of goal coordinates, precompute (by BFS)
//! the distance to the goal and the best move for every coordinate. Solving the stage is then
//! pure table lookups: `x -> best[x] -> table[x*18 + best[x]] -> ...` until a goal is reached.
//! No search, no branching on the hot path. Chain several stages to solve a whole cube.

use cubecore::Move;

pub struct DescentStage {
    dist: Vec<u8>,
    best: Vec<u8>,
}

impl DescentStage {
    /// `table` is a standard move table (`n * 18` entries). `allowed` must be closed under
    /// inversion (true for any generator set of the form "all powers of a set of faces"),
    /// which makes distance-from-goal equal distance-to-goal.
    pub fn build(n: usize, allowed: &[Move], table: &[u16], goals: &[usize]) -> Self {
        let mut dist = vec![u8::MAX; n];
        let mut frontier = Vec::new();
        for &g in goals {
            dist[g] = 0;
            frontier.push(g);
        }
        let mut d = 0u8;
        while !frontier.is_empty() {
            let mut next = Vec::new();
            for &x in &frontier {
                for m in allowed {
                    let y = table[x * 18 + m.0 as usize] as usize;
                    if dist[y] == u8::MAX {
                        dist[y] = d + 1;
                        next.push(y);
                    }
                }
            }
            frontier = next;
            d += 1;
        }
        let mut best = vec![u8::MAX; n];
        for x in 0..n {
            if dist[x] == 0 || dist[x] == u8::MAX {
                continue;
            }
            for m in allowed {
                let y = table[x * 18 + m.0 as usize] as usize;
                if dist[y] != u8::MAX && dist[y] as u16 + 1 == dist[x] as u16 {
                    best[x] = m.0;
                    break;
                }
            }
        }
        DescentStage { dist, best }
    }

    #[inline]
    pub fn dist(&self, x: usize) -> u8 {
        self.dist[x]
    }

    /// One descent step: the move to play and the next coordinate. `None` at a goal or if unreachable.
    #[inline]
    pub fn step(&self, x: usize, table: &[u16]) -> Option<(Move, usize)> {
        let b = self.best[x];
        if b == u8::MAX {
            return None;
        }
        Some((Move(b), table[x * 18 + b as usize] as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cubecore::coord::N_TWIST;
    use cubecore::tables::MoveTables;

    #[test]
    fn twist_descent_reaches_goal_in_dist_steps() {
        let mt = MoveTables::new();
        let all: Vec<Move> = Move::all().collect();
        let st = DescentStage::build(N_TWIST, &all, &mt.twist, &[0]);
        for x in 0..N_TWIST {
            assert_ne!(st.dist(x), u8::MAX, "coordinate {x} unreachable");
            let mut cur = x;
            let mut steps = 0u8;
            while cur != 0 {
                let (_, nx) = st.step(cur, &mt.twist).expect("stuck before goal");
                cur = nx;
                steps += 1;
                assert!(steps <= 20);
            }
            assert_eq!(steps, st.dist(x));
        }
    }
}
`````

### `enso/src/main.rs`

`````rust
fn main() {
    // Historical stub content. Current enso CLI lives in enso/src/cli.rs; main.rs
    // only calls cli::run() and returns its status. See enso/src/solver.rs for the solver.
}
`````


---

## bench: benchmark harness

### `bench/Cargo.toml`

`````toml
[package]
name = "benchkit"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true
description = "Reproducible benchmark harness for enso and exa"

[dependencies]
cubecore = { path = "../core" }
enso = { path = "../enso" }
exa = { path = "../exa" }
`````

### `bench/src/main.rs`

`````rust
//! Reproducible benchmark harness.
//!
//!   cargo run --release -p benchkit -- --solver exa  --n 200 --seed 1 --depth 9
//!   cargo run --release -p benchkit -- --solver enso --n 100000 --seed 1
//!
//! `--depth 0` (default) means uniformly random solvable states.
//! Every solution is verified; invalid solutions are counted and reported, never hidden.

use cubecore::moves::random_alg;
use cubecore::rng::Rng;
use cubecore::solver::Solver;
use cubecore::CubieCube;
use std::hint::black_box;
use std::time::Instant;

fn usage() -> ! {
    eprintln!("usage: benchkit --solver <exa|enso> [--n N] [--seed S] [--depth D]");
    std::process::exit(2)
}

fn pct(sorted: &[u128], p: f64) -> u128 {
    sorted[((sorted.len() - 1) as f64 * p) as usize]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut name, mut n, mut seed, mut depth) = ("exa".to_string(), 100usize, 1u64, 0usize);
    let mut i = 0;
    while i < args.len() {
        let val = args.get(i + 1).map(|s| s.as_str()).unwrap_or_else(|| usage());
        match args[i].as_str() {
            "--solver" => name = val.to_string(),
            "--n" => n = val.parse().unwrap_or_else(|_| usage()),
            "--seed" => seed = val.parse().unwrap_or_else(|_| usage()),
            "--depth" => depth = val.parse().unwrap_or_else(|_| usage()),
            _ => usage(),
        }
        i += 2;
    }

    let mut solver: Box<dyn Solver> = match name.as_str() {
        "exa" => Box::new(exa::Exa::new()),
        "enso" => Box::new(enso::Enso::new()),
        _ => usage(),
    };

    let mut rng = Rng::new(seed);
    let (mut times, mut lens) = (Vec::new(), Vec::new());
    let (mut failed, mut invalid) = (0usize, 0usize);

    for _ in 0..n {
        let cube = if depth == 0 {
            CubieCube::random(&mut rng)
        } else {
            CubieCube::SOLVED.apply_all(&random_alg(&mut rng, depth))
        };
        let t = Instant::now();
        let sol = black_box(solver.solve(black_box(&cube)));
        let ns = t.elapsed().as_nanos();
        match sol {
            Some(s) if cube.apply_all(&s).is_solved() => {
                times.push(ns);
                lens.push(s.len());
            }
            Some(_) => invalid += 1,
            None => failed += 1,
        }
    }

    println!("solver: {}  n: {}  seed: {}  depth: {}", solver.name(), n, seed, depth);
    println!("solved: {}  failed: {}  INVALID: {}", times.len(), failed, invalid);
    if times.is_empty() {
        return;
    }
    times.sort_unstable();
    let avg_len = lens.iter().sum::<usize>() as f64 / lens.len() as f64;
    let avg_ns = times.iter().sum::<u128>() as f64 / times.len() as f64;
    println!("avg length: {avg_len:.2}  max length: {}", lens.iter().max().unwrap());
    println!(
        "time ns  avg: {avg_ns:.0}  p50: {}  p99: {}  max: {}",
        pct(&times, 0.5),
        pct(&times, 0.99),
        times.last().unwrap()
    );
    println!("throughput: {:.0} solves/s (single thread)", 1e9 / avg_ns);
}
`````


---

## verify: correctness tests

### `verify/Cargo.toml`

`````toml
[package]
name = "verify"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true
description = "Correctness tests: the gate every change must pass"

[dependencies]
cubecore = { path = "../core" }
exa = { path = "../exa" }
enso = { path = "../enso" }
`````

### `verify/src/lib.rs`

`````rust
//! Intentionally empty. All tests live in `tests/`.
`````

### `verify/tests/core.rs` (historical; now `cube.rs`, `coords.rs`, `facelets.rs`)

`````rust
use cubecore::coord::*;
use cubecore::cubie::MOVES;
use cubecore::facelet::{from_facelets, to_facelets};
use cubecore::moves::{format_alg, parse_alg, random_alg};
use cubecore::rng::Rng;
use cubecore::tables::MoveTables;
use cubecore::{CubieCube, Move};

fn alg(s: &str) -> CubieCube {
    CubieCube::SOLVED.apply_all(&parse_alg(s).unwrap())
}

#[test]
fn every_move_has_order_4_and_half_turns_have_order_2() {
    for face in 0..6u8 {
        let q = MOVES[(face * 3) as usize];
        let mut c = CubieCube::SOLVED;
        for _ in 0..4 {
            c = c.multiply(&q);
        }
        assert!(c.is_solved(), "face {face}^4 != identity");
        assert!(!q.multiply(&q).is_solved());
        let h = MOVES[(face * 3 + 1) as usize];
        assert!(h.multiply(&h).is_solved());
    }
}

#[test]
fn sexy_move_has_order_6() {
    let mut c = CubieCube::SOLVED;
    for i in 1..=6 {
        c = c.apply_all(&parse_alg("R U R' U'").unwrap());
        assert_eq!(c.is_solved(), i == 6, "after {i} repetitions");
    }
}

#[test]
fn superflip_algorithm_is_correct_and_20_moves() {
    let a = parse_alg("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
    assert_eq!(a.len(), 20);
    assert_eq!(CubieCube::SOLVED.apply_all(&a), CubieCube::superflip());
    assert!(CubieCube::superflip().verify().is_ok());
}

#[test]
fn parse_and_format_roundtrip() {
    let s = "R U R' U' F2 D L' B";
    assert_eq!(format_alg(&parse_alg(s).unwrap()), s);
    assert!(parse_alg("X").is_err());
    assert!(parse_alg("R3").is_err());
}

#[test]
fn inverse_and_move_inverse() {
    let mut rng = Rng::new(7);
    for _ in 0..500 {
        let c = CubieCube::random(&mut rng);
        assert!(c.verify().is_ok());
        assert!(c.multiply(&c.inverse()).is_solved());
        assert!(c.inverse().multiply(&c).is_solved());
    }
    for m in Move::all() {
        assert!(CubieCube::SOLVED.apply(m).apply(m.inverse()).is_solved());
    }
}

#[test]
fn moves_preserve_solvability() {
    let mut rng = Rng::new(11);
    let c = CubieCube::SOLVED.apply_all(&random_alg(&mut rng, 200));
    assert!(c.verify().is_ok());
}

#[test]
fn coordinates_roundtrip() {
    for x in 0..N_TWIST {
        assert_eq!(cubie_from_twist(x).twist(), x);
    }
    for x in 0..N_FLIP {
        assert_eq!(cubie_from_flip(x).flip(), x);
    }
    for x in 0..N_SLICE {
        assert_eq!(cubie_from_slice(x).slice_pos(), x);
    }
    for x in (0..N_CPERM).step_by(7).chain([N_CPERM - 1]) {
        assert_eq!(cubie_from_cperm(x).cperm(), x);
    }
    let s = CubieCube::SOLVED;
    assert_eq!((s.twist(), s.flip(), s.slice_pos(), s.cperm()), (0, 0, SLICE_SOLVED, 0));
}

#[test]
fn move_tables_match_real_cube_moves() {
    let mt = MoveTables::new();
    let mut rng = Rng::new(3);
    for _ in 0..300 {
        let c = CubieCube::random(&mut rng);
        for m in Move::all() {
            let n = c.apply(m);
            let mi = m.0 as usize;
            assert_eq!(mt.twist[c.twist() * 18 + mi] as usize, n.twist());
            assert_eq!(mt.flip[c.flip() * 18 + mi] as usize, n.flip());
            assert_eq!(mt.slice[c.slice_pos() * 18 + mi] as usize, n.slice_pos());
            assert_eq!(mt.cperm[c.cperm() * 18 + mi] as usize, n.cperm());
        }
    }
}

#[test]
fn facelets_roundtrip_and_known_strings() {
    assert_eq!(
        to_facelets(&CubieCube::SOLVED),
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
    );
    assert_eq!(
        to_facelets(&alg("R")),
        "UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB"
    );
    let mut rng = Rng::new(5);
    for _ in 0..300 {
        let c = CubieCube::random(&mut rng);
        assert_eq!(from_facelets(&to_facelets(&c)).unwrap(), c);
    }
    assert!(from_facelets("short").is_err());
}
`````

### `verify/tests/exa.rs`

`````rust
use cubecore::moves::random_alg;
use cubecore::rng::Rng;
use cubecore::solver::Solver;
use cubecore::CubieCube;
use exa::Exa;

#[test]
fn solves_solved_cube_with_zero_moves() {
    let mut exa = Exa::new();
    assert_eq!(exa.solve(&CubieCube::SOLVED).unwrap().len(), 0);
}

#[test]
fn solutions_are_valid_and_never_longer_than_the_scramble() {
    let mut exa = Exa::new();
    let mut rng = Rng::new(42);
    for len in 1..=7 {
        for _ in 0..5 {
            let scramble = random_alg(&mut rng, len);
            let cube = CubieCube::SOLVED.apply_all(&scramble);
            let sol = exa.solve(&cube).expect("no solution");
            assert!(cube.apply_all(&sol).is_solved());
            assert!(sol.len() <= len, "found {} moves for a {}-move scramble", sol.len(), len);
        }
    }
}

#[test]
fn rejects_unsolvable_cubes() {
    let mut exa = Exa::new();
    let mut c = CubieCube::SOLVED;
    c.co[0] = 1; // single twisted corner: physically impossible
    assert!(exa.solve(&c).is_none());
}
`````


---

## docs

### `docs/DESIGN.md`

`````markdown
# Design and conventions

These are load-bearing. Changing any of them means changing `core/`, every table, and
`verify/tests/{cube,coords,facelets}.rs` together.

## Cube model (cubie level, Kociemba conventions)

- Corners `URF UFL ULB UBR DFR DLF DBL DRB` = 0..7. Edges `UR UF UL UB DR DF DL DB FR FL BL BR` = 0..11.
- `cp[slot]` = which corner cubie is in that slot; `co[slot]` = its twist (0..3). Same for `ep` / `eo` (flip 0..2).
- `a.multiply(&b)` = apply `a`, then `b`. A move is applied as `cube.multiply(&MOVES[m])`.
- Solvable iff: both permutations valid, sum(co) % 3 == 0, sum(eo) % 2 == 0, parity(cp) == parity(ep).
- `MOVES: [CubieCube; 18]` is computed at compile time from the six basic quarter turns.

## Moves and metric

- Faces `U R F D L B` = 0..5. `Move(face * 3 + power - 1)`, power 1/2/3 = `X` / `X2` / `X'`.
- Metric is **HTM**: all 18 moves cost 1. God's number = 20.
- Redundancy pruning (`Move::can_follow`): no repeated face; opposite faces only in canonical order (U before D, R before L, F before B).

## Coordinates

| Coordinate | Size | Meaning |
|------------|------|---------|
| twist | 3^7 = 2187 | corner orientations (last is implied) |
| flip | 2^11 = 2048 | edge orientations (last is implied) |
| slice | C(12,4) = 495 | which 4 slots hold UD-slice edges (FR FL BL BR); solved = 494 |
| cperm | 8! = 40320 | corner permutation (Lehmer rank) |

Move tables: `table[coord * 18 + move] -> coord`, `u16`.

## Facelets

54 chars, order `U R F D L B`, nine each. Solved = `UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB`. After `R` from solved: `UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB`.

## Validation history

Before the first compile, these properties were verified with an independent Python port of the same tables: every move has order 4; `R U R' U'` has order 6; the 20-move superflip algorithm yields exactly "all edges flipped"; facelet round-trips; coordinate encode/decode round-trips; move tables agree with real cube moves; `inverse()` is a true inverse. The Rust tests in `verify/tests/{cube,coords,facelets}.rs` re-assert all of these.

## Additional coordinates implemented for M3

Edge permutation pieces, tetrad/edge-orbit patterns, corner left-coset classes and exact
G3 membership indices are implemented for enso stage 3 and stage 4.

## Planned (not yet implemented)

- 48-element symmetry group (and inverse) with symmetry-reduced coordinates: needed for exa's meet-in-the-middle tables.
- More coordinates: edge permutation pieces, UD-slice sorted, coset coordinates for H = <U, D, R2, L2, F2, B2>.
`````

### `docs/ENSO.md`

`````markdown
# enso: design

## Goal

Minimum latency from scrambled state to **any** valid solution. Length is free currency: spend it to buy speed.

## Baseline to beat

Two-phase / Kociemba-style solvers (min2phase etc.) take on the order of a millisecond and return about 20-22 moves, because they search for *short* solutions. enso doesn't need short, so it shouldn't search.

## Core idea: table descent instead of search

For each stage, precompute (BFS from the goal) the best move for every coordinate value. Solving a stage is then:

```
x = coord(cube)
while x not goal:  m = best[x];  emit m;  x = move_table[x][m]
```

Each step is a couple of dependent loads and no branching on cube data. A whole solve is a few dozen steps. The chain is fixed; stages must keep the earlier stages' invariants (the standard group-chain argument, as in Thistlethwaite's algorithm).

`enso::chain::DescentStage` is the generic building block (built and unit-tested). The stage definitions are the work of M3.

## Three variants to build and compare

| Variant | Stages | Expected length | Table size | Notes |
|---------|--------|-----------------|------------|-------|
| A: Thistlethwaite chain | 4 (G0>G1>G2>G3>solved) | ~30-45 | tens of MB | simplest, tiny tables, most steps |
| B: two-phase, first solution only | 2 | ~25-30 | ~100s of MB | phase 1 via descent, phase 2 via descent, no length optimization |
| C: fat tables | 2-3 | 25-30 | GBs | fewest steps, DRAM-latency bound |

**The knob is table size vs. step count.** The experiment is to find the knee for the target machine, measured, not guessed.

## Things most likely to actually matter

1. **Input handling.** If input is a 54-char facelet string, parsing it into coordinates may cost more than the solve. Report both numbers. A SIMD coordinate extractor is a real, separate win.
2. **Memory latency.** A dependent-load chain is latency-bound. Pack `(move, next_coord)` into one entry so each step is one load. Use huge pages. Prefetch the next stage's table entries while finishing the current one.
3. **Macro entries.** Instead of one move per lookup, let a stage entry hold a whole short move sequence (2-4 moves) when it fits. Fewer dependent loads, bigger tables. Measure.
4. **Output.** Writing moves into a fixed `[u8; N]` buffer, no `Vec`.
5. **Batch mode.** Many cubes in flight at once hides load latency (interleave N independent descents per thread; SIMD gathers; GPU later). Throughput is a separate benchmark from latency.

## Honest expectations

- Single-solve latency in the low microseconds is plausible; sub-microsecond is a *hypothesis* to be earned with measurements, not a promise.
- Startup cost (building tables) will be seconds; loading prebuilt tables from disk via mmap is the fix and is part of M5.
- Assembly is a last resort: only for a function the profiler names, only with a scalar fallback and tests against it.

## Correctness gate

On 1,000,000 random solvable cubes: 0 invalid solutions, 0 failures. Always.
`````

### `docs/EXA.md`

`````markdown
# exa: design

## Goal

The fewest possible moves, **provably**. HTM metric.

## Reality check (so effort goes to the right place)

- God's number is 20 in HTM (proven 2010). exa can never need more.
- A random state needs about 17.7 moves on average; the vast majority need 16-19.
- So "fewer moves than ever" is bounded. The open frontier is **how cheaply optimality can be proven**.

## Baseline (M2, written): IDA* with admissible projections

`exa::ida::Exa`. Heuristic = max of exact distances in three projections: (twist, slice), (flip, slice), (corner perm). Exact, simple, correct, slow on deep states. It is the oracle every other engine is tested against.

## Ideas to try (each is a hypothesis with a kill criterion)

### 1. Bound squeeze
Run an *upper-bound* finder (fast two-phase-style search returning some solution of length U) and a *lower-bound* prover (admissible tables showing nothing shorter than L exists). Stop when U = L.
- Exactness lives entirely in the lower-bound side; heuristic/ML guidance may only reorder the upper-bound search.
- **Kill criterion:** if on 1,000 random states the average time to certify is not at least 2x better than the oracle plus plain IDA* on the same tables, drop it.

### 2. Meet-in-the-middle with symmetry reduction
Count of states within d moves (HTM): d=8: 1.33x10^9 at exactly 8; cumulative through 9: about 1.9x10^10. Dividing by the 96-element (48 symmetries x inversion) group gives roughly 2x10^8 canonical states: storable as hashed fingerprints in a few GB. Then search about 10-11 moves from the scramble and look up hits.
- **Caveat, stated plainly:** searching 11 deep from the scramble is ~10^12 nodes without good pruning. This is only viable with strong pruning and/or a GPU. That is the experiment.
- **Kill criterion:** if the table build plus lookup cost exceeds the oracle's time on states up to depth 16, it's not worth it below that depth.

### 3. Coset-based admissible heuristics
Distances to cosets of H = <U, D, R2, L2, F2, B2> are much stronger lower bounds than classic corner/edge PDBs. Stack them with symmetry reduction.
- **Kill criterion:** if average h at the root is not at least 1.5 higher than the baseline projections on random states, drop it.

## Verification protocol

- Oracle agreement: for every new engine, same length as `Exa` on a shared seed set (`verify/tests/`).
- Known results: superflip = 20 moves exactly; any scramble of k moves has optimal length <= k.
- Emit `(lower_bound, upper_bound, path)`; optimal iff equal.
- Cross-check a sample against an external solver (nissy, Cube Explorer) before publishing numbers.

## Speed expectations (baseline)

Fast up to about 10-12 moves deep, then it falls off a cliff. That cliff is what the ideas above attack. Measure it with `benchkit --depth N` for N = 8..14 and record in `docs/EXPERIMENTS.md`.
`````

### `docs/ROADMAP.md`

`````markdown
# Roadmap

Status key: [x] done, [ ] todo. Update this file in the same change that finishes a task.

## M0: scaffold
- [x] workspace, crates, CI, agent instructions
- [x] **first compile + `cargo test --workspace --release` green** (the scaffold was written without a toolchain; this was fixed before M1)

## M1: core (written)
- [x] cubie model, 18 moves (compile-time), inverse, verify, random
- [x] coordinates: twist, flip, slice, cperm (+ decoders)
- [x] move tables
- [x] facelet parse/format
- [x] RNG, Solver trait
- [ ] 48 symmetries + inverse (needed for M8)
- [ ] more coordinates (edge perm parts, coset coordinates)

## M2: exa baseline
- [x] IDA* with 3 admissible projections (reference oracle)
- [ ] benchmark depth curve 8..14, record in EXPERIMENTS.md
- [ ] cross-check against an external optimal solver on 100 scrambles

## M3: enso v1 (Variant A, Thistlethwaite chain)
- [x] `DescentStage` building block + test
- [ ] stage definitions G0>G1>G2>G3>solved with correct coordinates
- [ ] `Enso::solve`, zero allocation
- [ ] 1M random cubes: 0 invalid

## M4: baselines to beat
- [ ] two-phase reference (own implementation or external binary) for speed comparison
- [ ] benchmark report: enso vs baseline, same machine, same seeds

## M5: enso tuning
- [ ] packed entries, huge pages, prefetch
- [ ] mmap-able table files
- [ ] facelet to coordinate fast path
- [ ] Variant B/C comparison, find the table-size knee

## M6: enso batch / SIMD
- [ ] interleaved descents, SIMD gathers
- [ ] `asm` feature only where profiled

## M7: exa lower-bound engine
- [ ] coset heuristics (idea 3)
- [ ] bound squeeze (idea 1)

## M8: exa meet-in-the-middle
- [ ] symmetry-reduced table (idea 2)
- [ ] CPU implementation, then GPU

## M9: publish
- [ ] reproducible benchmark report, honest comparison with nissy / min2phase
- [ ] failed ideas documented
`````

### `docs/BENCHMARKING.md`

`````markdown
# Benchmarking rules

1. **Release build only**: `cargo build --release`, flags from `.cargo/config.toml` (`target-cpu=native`). Record CPU model, OS, rustc version, commit.
2. **Fixed seeds.** Same seed = same cubes. Compare solvers on identical inputs.
3. **Warm up**, then measure. Report median and p99, not just the mean. Report failures and invalid solutions.
4. **Verify every solution** (the harness does). A faster solver that returns a bad solution has no time.
5. **Separate** startup (table build/load) from per-solve time. Report both.
6. **Sub-microsecond timing:** one `Instant::now()` pair costs tens of ns. For enso, also time batches (N solves, divide) and report the batch method; `benchkit --batch` is available.
7. **Latency vs throughput** are different benchmarks. Label which one a number is.
8. **Pin the machine:** close other loads, fix CPU frequency if you can, repeat 5 times, report the spread.
9. **Compare fairly:** same metric, same input form (cubie vs facelets), same build flags class.
10. No number enters a doc or README without the command that produced it.
`````

### `docs/EXPERIMENTS.md`

`````markdown
# Experiment log

Newest first. Write the hypothesis and kill criterion **before** coding. Record the result either way.

## Template

```
### EXP-NNN: <short title>
Date:        YYYY-MM-DD
Tool:        enso | exa
Hypothesis:  <what you expect and why>
Kill if:     <result that means the idea is dead>
Setup:       <commit, CPU, flags, exact command, seeds>
Result:      <numbers>
Decision:    keep | revert | iterate
Notes:       <what you learned>
```

## Log

*(empty: first entries should be the M2 baseline depth curve and the first enso variant A numbers)*
`````
