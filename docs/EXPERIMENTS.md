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

### EXP-005: direct facelets -> stage coordinates
Date:        2026-11-21
Tool:        enso
Status:      hypothesis only; no source change yet
Hypothesis:  The remaining ~1.3 us facelet overhead is caused by parsing a 54-char string into a
             full `CubieCube`, then updating that cube after every emitted move. If facelets are
             validated once into the four stage coordinates and each descent step advances the
             coordinates through move tables, the hot path can skip cube construction and
             `CubieCube::multiply` entirely for facelet input. This should lower facelet p50/p99
             while leaving cubie input unchanged.
Kill if:     Interleaved A/B against the current Step 2 candidate shows no reproducible facelet
             p50/p99 improvement beyond run-to-run drift, or cubie latency regresses, or any
             solution fails verification, or the 1M gate fails. An invalid solution is a bug
             regardless of speed.
Setup:       Branch `perf`, Step 2 packed-stage candidate. Baseline for this experiment is the
             current candidate binary built with the published v3 flags; the published v3 tarball
             remains the fixed external baseline for the final report. Same machine, seed 1,
             n=100000, cubie and `--facelets`, alternating A/B order. Current Step 2 medians:
             cubie p50 1565 ns / p99 3621.5 ns, facelets p50 2861 ns / p99 5935.5 ns.
Decision:    pending
Notes:       This is Step 3 from the plan. It is distinct from Step 2 and must be benchmarked
             separately; the cubie path is reported separately from the facelet path.

### EXP-004: external two-phase timing reference (kociemba)
Date:        2026-11-21
Tool:        enso (timing reference only)
Hypothesis:  A published two-phase solver should be much slower per solve than enso because it
             searches for short solutions, even though both are given the same cubes.
Kill if:     The reference cannot be installed or cannot consume the same seeded facelet
             strings; in that case it is skipped rather than replaced by a substitute.
Setup:       `kociemba` 1.2.1 installed in `/tmp/opencode/enxa-kociemba-venv` (isolated venv,
             no repo dependency). The 50 seeded cubes were generated with a scratch Rust
             program that uses `cubecore::Rng::new(1)`, `random_alg(&mut rng, 8)` and
             `to_facelets`, i.e. the same generator and seed as
             `benchkit --solver enso --n 50 --depth 8 --seed 1`. The Python driver did one
             warm-up solve, then timed `kociemba.solve(facelets)` with `time.perf_counter_ns`.
             Same machine and CPU flavor as EXP-003; v3 binaries from the published tarball
             and the Step 2 candidate.
Result:      kociemba: 50 solved, 0 failed; avg solution length 10.74 HTM, max 18;
             p50 30,264 ns, p99 1,073,880 ns, avg 141,454 ns, 7,069 solves/s.
             Same 50 cubes, published-v3 enso: avg length 26.28, p50 5,062 ns, p99 11,024 ns,
             0 failed/INVALID. Step 2 candidate enso: avg length 26.28, p50 3,088 ns,
             p99 6,265 ns, 0 failed/INVALID. The candidate is ~9.8x faster at the median
             than kociemba on identical inputs, at ~2.4x the solution length.
Decision:    keep as a timing reference only; no source or dependency change
Notes:       This is not an apples-to-apples quality comparison: kociemba returns near-optimal
             two-phase solutions while enso returns any valid solution, and the Python/C
             wrapper overhead is included in kociemba's time. Commands and inputs are in
             `/tmp/opencode/enxa-perf/`. No kociemba code was copied into the repository.

### EXP-003: enso hot-path table packing and input path
Date:        2026-11-21
Tool:        enso
Status:      Step 2 implemented and kept; Step 3 next
Hypothesis:  The ~2.3 us solve is limited substantially by dependent loads over ~127 MB of
             move tables. Each descent step currently loads `best[x]` (u8) and then
             `table[x * 18 + best[x]]` (u16/u32), with the second load depending on the first.
             Packing `(move, next_coord)` into one u32 per state should cut hot table bytes by
             ~18x, remove one dependent load per step, and lower p50/p99 and cache misses.
             Facelet input adds parsing time that can be attacked separately.
Kill if:     A same-machine interleaved A/B against the published v3 tarball shows no
             reproducible p50/p99 improvement outside run-to-run drift, or any solution
             fails verification, or the 1M gate fails. Attempts that do not help are reverted.
Setup:       Branch `perf`, based on v0.1.0 (commit 690f5df). A/B baseline is the published
             `enxa-0.1.0-x86_64-linux-v3.tar.gz` downloaded once to
             `/tmp/opencode/enxa-user-verify/`; no worktrees. Baseline binary:
             `/tmp/opencode/enxa-user-verify/unpacked-v3/benchkit`.
             CPU Intel Core i3-1005G1 @ 1.20GHz (max 3.4GHz), 4 logical CPUs, Linux
             7.2.14-arch1-1 x86_64, rustc 1.97.1, `-C target-cpu=x86-64-v3` with
             `--remap-path-prefix` and fat LTO. perf 7.3.5-1, `perf_event_paranoid` temporarily
             raised from 2 to 1 for sampling and restored afterwards.
             Commands:
               `<base>/benchkit --solver enso --n 100000 --seed 1`
               `<base>/benchkit --solver enso --n 100000 --seed 1 --facelets`
               `<base>/benchkit --solver enso --n 0 --seed 1`  (startup only)
               `perf stat -e cycles,instructions,cache-misses,LLC-load-misses,L1-dcache-load-misses,dTLB-load-misses ...`
               `perf record -F 999 --call-graph=dwarf -- <base>/benchkit --solver enso --n 500000 --seed 1`
Result:      BASELINE (published v3 tarball, 3 runs each):
               cubie p50 2396 / 2289 / 2240 ns, p99 6071 / 6911 / 6736 ns,
                     startup 450.3 / 498.6 / 433.8 ms, 0 failed, 0 INVALID, avg len 31.34.
               facelets p50 3549 / 3557 / 3474 ns, p99 9006 / 9351 / 9154 ns,
                        startup 438.7 / 464.2 / 442.8 ms, 0 failed, 0 INVALID.
               cold startup (first n=0 process) 444.7 ms; warm 471.3 / 447.1 ms.
                 There are no prebuilt tables yet: every process rebuilds them.
               perf counts, 3-run averages:
                 startup-only: cycles 1,390,077,895; instructions 3,550,669,270;
                   cache-misses 13,298,123; LLC-load-misses 3,321,221;
                   L1-dcache-load-misses 25,839,320; dTLB-load-misses 1,205,720.
                 cubie n=100000: cycles 2,548,268,749; instructions 5,315,370,074;
                   cache-misses 17,251,205; LLC-load-misses 5,137,398;
                   L1-dcache-load-misses 32,717,169; dTLB-load-misses 2,586,687.
                 facelets n=100000: cycles 3,071,568,460; instructions 5,934,420,996;
                   cache-misses 17,992,028; LLC-load-misses 5,350,012;
                   L1-dcache-load-misses 33,339,109; dTLB-load-misses 2,601,128.
                 Subtracting startup gives ~39.5 cache-misses/solve and ~18.2
                 LLC-load-misses/solve for cubie input, and ~46.9 / ~20.3 for
                 facelets. These are whole-process counts, so they also include input
                 generation and solution verification; they are upper bounds.
               perf record (500k cubie solves, 2367 samples):
                 Enso::solve 46.75% self, CubieCube::multiply 24.88% self,
                 Tables::build 6.44%, DescentStage::build<u32> 6.56%,
                 benchkit::record 2.98%. Within solve, instruction samples are
                 concentrated in stage-2 u32 move-table loads (23.88%), stage-4 u32
                 loads (28.17%), stage-3 u16 loads (9.86%), stage-1 u16 loads (3.06%),
                 and best[] loads (7.51% / 10.70%).
               Table size at v0.1.0: 130,552,194 bytes in the `Tables` struct, of which
                 128,630,117 bytes are the four move tables plus best[] arrays used by
                 descent. Stage-2 move table alone is 77,944,680 bytes and stage-4 is
                 47,775,744 bytes.
               Verdict on the hypothesis: partially agrees. Dependent loads over large
                 tables dominate solve self-samples, especially stages 2 and 4, but cube
                 bookkeeping is the second-largest cost. Proceed with Step 2 packing,
                 then re-profile before deciding on later steps.

STEP 2 RESULT — packed `(move, next_coord)` u32 per state (kept):
             Implementation: `PackedStage` in `enso/src/chain.rs` is built from the same BFS
               as `DescentStage` and stores `(move << 24) | next` in one u32. `Tables` now
               owns only the four packed stages; the 18-wide move tables and best[] arrays
               are dropped after startup. The stage-3 move-table test is preserved by a
               `#[cfg(test)]` field holding the pre-packed table.
             Correctness: `cargo test --workspace --release` 22 passed, 0 failed (new test
               `packed_stage_matches_descent_stage` compares every step with the old BFS).
               Gate: 1,000,000 cubes, 0 invalid, 0 failed, avg 31.343, max 41.
             Interleaved A/B, 6 pairs, A = published v3 tarball, B = candidate, alternating
               AB/BA order, same machine, seed 1, n=100000:
               cubie p50 median: A 2308.5 ns -> B 1565 ns (-32.2%).
               cubie p99 median: A 6503 ns -> B 3621.5 ns (-44.3%).
               facelets p50 median: A 3637 ns -> B 2861 ns (-21.3%).
               facelets p99 median: A 9656 ns -> B 5935.5 ns (-38.5%).
               All 12 runs in each mode: 0 failed, 0 INVALID, avg length 31.34, max 41.
             Startup: cold (first n=0 process) A 444.7 ms -> B 457.7 ms; warm A
               471.3/447.1 ms -> B 509.5/514.3 ms. Packaging A/B ranges 447-513 ms vs
               470-527 ms. The extra packing pass makes startup ~3-5% slower; per-solve
               latency improves far more, and startup remains reported separately.
             Table size: hot descent tables 128,630,117 -> 7,110,260 bytes (-94.5%);
               total `Tables` struct 130,552,194 -> 7,258,868 bytes (-94.4%). Stage 2
               alone 79,027,245 -> 4,330,260 bytes; stage 4 48,439,296 -> 2,654,208.
             perf stat (user-space counts at paranoid=2, 3-run averages, whole process;
               per-solve figures subtract the matching n=0 startup run):
               cubie cache-misses 39.5 -> 16.5/solve, LLC-load-misses 18.2 -> 6.1/solve,
               L1-dcache-load-misses 68.8 -> 30.5/solve, dTLB-load-misses 13.8 -> 0.8/solve.
               facelets cache-misses 46.9 -> 24.5/solve, LLC-load-misses 20.3 -> 10.4/solve,
               L1-dcache-load-misses 75.0 -> 36.9/solve, dTLB-load-misses 13.9 -> 1.4/solve.
               Instructions are essentially unchanged (~17,650 -> ~17,510 per cubie solve);
               cycles fall from ~11,580 to ~8,459 per solve, confirming that latency, not
               instruction count, was the bottleneck.
             perf record after (500k solves): Enso::solve 46.75% -> 26.98% self;
               CubieCube::multiply 24.88% -> 35.55% self. The old best[] loads and the
               second dependent table load are gone; three packed u32 loads remain.
Decision:    keep
Notes:       Logs/perf data: /tmp/opencode/enxa-perf/. `perf_event_paranoid` was raised to 1
             only for `perf record` and restored to 2 afterwards; all reported `perf stat`
             counts were collected at 2 so A and B are directly comparable. The next
             bottleneck is cube bookkeeping, so Step 3 (facelets -> coordinates) and a
             cheaper per-step cube update are the candidates to test next.

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
