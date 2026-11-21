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

### EXP-002: structural refactor with no behavior change
Date:        2026-11-21
Tool:        workspace
Hypothesis:  Splitting the large source files into the required modules, curating each crate's
             root API and moving the ignored gate to its own test target will preserve all
             21 tests, the 1,000,000-cube gate result and both benchmark results within
             machine noise.
Kill if:     Any test is lost or weakened, the gate output changes, or a same-machine
             interleaved A/B shows a reproducible >5% slowdown for either solver.
Setup:       Baseline commit 869172a; post-refactor working tree. Same machine and flags as
             EXP-001. Commands: `./scripts/check.sh`,
             `cargo test -p verify --release --test gate -- --ignored --nocapture`,
             `cargo run --release -p benchkit -- --solver enso --n 100000 --seed 1`,
             `cargo run --release -p benchkit -- --solver exa --n 50 --depth 8 --seed 1`.
             Baseline numbers were recorded first in `/tmp/opencode/refactor_baseline.md`.
Result:      Tests: 21 passed, 1 ignored before and after. Gate before and after:
             1,000,000 cubes, 0 invalid, 0 failed, avg length 31.343, max 41.
             Baseline enso (single run): avg 2232 ns, p50 2144 ns, p99 3880 ns,
             447999 solves/s. Final post-refactor enso (three runs): p50 2249 / 2212 /
             2309 ns, p99 4568 / 4009 / 4456 ns, 420514 / 434270 / 410672 solves/s.
             Baseline exa (single run): avg 121157 ns, p50 61216 ns, p99 393820 ns,
             8254 solves/s. Final post-refactor exa (three runs): p50 63073 / 61123 /
             58095 ns, p99 464118 / 369496 / 366438 ns, 8014 / 8430 / 8969 solves/s.
             An early apparent enso slowdown was machine drift: a clean baseline worktree
             measured at the same time was equal or slower in interleaved A/B runs
             (baseline p50 2229-2467 ns; refactor p50 2212-2309 ns). Lengths and validity
             were identical in every run.
Decision:    keep
Notes:       Only module ownership, visibility and file layout changed. The four-stage
             chain, generator sets, search rules, cube conventions, test assertions and
             benchmark metric are unchanged. The ignored gate target is now `--test gate`.

### EXP-001: enso Variant A — pure table descent (Thistlethwaite chain)
Date:        2026-11-21
Tool:        enso
Hypothesis:  A four-stage chain (flip → twist+slice → paired coset coordinate → half-turn
             subgroup) can be solved entirely by precomputed best-move lookups. Solution
             length will be ~30-45 HTM; single-solve latency will be a few microseconds;
             table build will be seconds. Correctness is the gate, not length.
Kill if:     Any stage's descent leaves a state outside the next subgroup, any of 1,000,000
             seeded random cubes fails or yields an invalid solution, or a required table
             needs more than a few hundred MB.
Setup:       rustc 1.97.1 (8bab26f4f), Linux 7.2.14-arch1-1 x86_64,
             Intel Core i3-1005G1 @ 1.20GHz (max 3.4GHz, 4 logical CPUs), 4 cores,
             `.cargo/config.toml`: `-C target-cpu=native`.
             Baseline commit 869172a (pre-refactor snapshot).
             Seeds: 1 (bench + gate), extra unit-test seeds 101-105 and 31337.
Result:      Correctness gate (exact command):
               `cargo test -p verify --release --test gate -- --ignored --nocapture`
               → 1,000,000 cubes, 0 invalid, 0 failed, avg length 31.343, max 41,
               3.98 s clean repeat (first run, 6.16 s, had the throwaway Python prototype
               from the derivation step still competing for CPU; it was then stopped).
             Full health check: `./scripts/check.sh` → fmt, clippy -D warnings,
               21 release tests passed, `OK`.
             Latency, 5 repeats of
               `cargo run --release -p benchkit -- --solver enso --n 100000 --seed 1`
               cubie input, per-solve timing: p50 = 2166 / 2304 / 2191 / 2257 ns
               (first of the five, 4928 ns, was discarded: a throwaway Python prototype
               from an earlier derivation step was still running and was then stopped);
               p99 = 3925 / 5715 / 4165 / 5215 ns; avg length 31.34, max 41;
               0 failed, 0 INVALID on all runs. Batch timing (`--batch`, clean run):
               2932 ns/solve cubie.
             Facelet-string input (parse + solve):
               `cargo run --release -p benchkit -- --solver enso --n 100000 --seed 1 --facelets`
               → p50 3707 ns, p99 9587 ns; `--facelets --batch` → 3697 ns/solve.
             Startup, reported separately by benchkit for every run above:
               465-547 ms for all four stage tables (~127 MB of move tables).
             Fair comparison, identical inputs
               `cargo run --release -p benchkit -- --solver <exa|enso> --n 200 --depth 8 --seed 1`
               exa: avg 8.00 moves, p50 66,009 ns, 0 failed/INVALID
               enso: avg 26.56 moves, p50 3,337 ns, 0 failed/INVALID
               → ~20x lower median latency at ~3.3x the solution length; the tools
               optimise different objectives, as intended.
Decision:    keep
Notes:       Derived, not assumed: G2→G3 cannot use L/R quarter turns (they move UD-slice
             edges out of the slice). The correct generator set is <U,D,L2,R2,F2,B2>.
             G3's half turns preserve corner tetrads {0,2,5,7}/{1,3,4,6} and edge orbits
             {0,2,4,6}/{1,3,5,7}/{8,9,10,11}; its order is 663,552.
             The G2/G3 coset space has 29,400 states: tetrad pattern (70) × edge orbit
             pattern (70) × corner left-coset class (6). A Python model verified all
             29,400 coordinates are reachable (max depth 13) and that 20,000/20,000
             random G2 descents land exactly in G3 before any Rust code was written.
