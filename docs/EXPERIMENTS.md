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

## Summary of attempts

| Attempt | Change | Measured effect (interleaved A/B, seed 1, n=100000) | Decision |
|---------|--------|----------------------------------------------------|----------|
| EXP-001 | Variant A table descent | cubie p50 ~2190-2310 ns, facelets p50 3474-3707 ns; 1M gate 0 invalid | baseline |
| EXP-002 | module structure only | no behavior change, all tests pass | keep |
| EXP-003 step 0 | perf profile | dependent table loads dominate `Enso::solve`; `CubieCube::multiply` second | guide only |
| EXP-003 step 2 | pack `(move,next)` into one u32 | vs published v3: cubie p50 -32.2%, p99 -44.3%; facelets p50 -21.3%, p99 -38.5%; hot tables 130.6 MB -> 7.26 MB | keep |
| EXP-004 | kociemba timing reference | kociemba p50 30,264 ns vs enso 3,088 ns on the same 50 cubes; length 10.74 vs 26.28 | reference only |
| EXP-005 | direct facelet coordinate path with incremental tables | vs step 2: facelets p50 3021.5 -> 2443 ns (-19.1%), cubie unchanged | kept, then refined |
| EXP-006 | rank at boundaries, drop incremental tables | facelets p50 +3.7%, p99 -16.0%; cache misses 44.2 -> 24.2/solve; tables -1.7 MB | iterate |
| EXP-007 | cheap array updates, skip dead arrays | vs EXP-006 facelets p50 -7.8%; vs published v3 p50 -38.8%, p99 -52.0% | keep |
| EXP-008 | validated prebuilt packed tables | startup 426 ms -> 13.2 ms warm; p50 unchanged (+0.1% cubie, +0.5% facelets) | keep |
| EXP-009 step 0 | re-profile after table packing/cache | dependent loads still dominate; prefetch and macro entries tried, huge pages skipped | guide only |
| EXP-010 | prefetch next packed entry | cubie p50 -11.0%, p99 -10.4%; facelets p50 -5.1%, p99 -6.3% | keep, default-on |
| EXP-011 | two-move macro entries | cubie p50 +0.3% (fail), facelets p50 -4.9%; 2.0x table bytes | revert |
| EXP-012 | fair 100k external comparison | measured | measurement |
| EXP-013 | exa depth curve + BFS truth | measured | measurement |
| EXP-014 | signed GitHub release tag without a local key | API/UI cannot sign tags; use GitHub-signed provenance | document limit |

## Log

### EXP-014: signed GitHub release tag without a local key
Date:        2026-11-25
Tool:        release workflow / GitHub APIs
Status:      decision recorded before publishing v0.2.0
Hypothesis:  A `workflow_dispatch` release can create a cryptographically signed annotated tag
             without a local GPG/SSH key, while still producing GitHub-signed build-provenance
             attestations for both tarballs.
Kill if:     GitHub's release or Git-database APIs cannot sign annotated tag objects, or the web
             UI only creates lightweight tags. In that case the unsigned tag is reported as a
             limitation and no "signed tag" claim is made.
Setup:       Inspected the existing `v0.1.0` tag with
             `gh api repos/dummy3ye/enxa/git/ref/tags/v0.1.0` and the corresponding
             `git/tags/<sha>` query; verified action SHAs with `git ls-remote`; consulted the
             GitHub REST docs for `git/tags` and `releases` plus GitHub's community discussion
             on release tags. No v0.2.0 tag was created during this investigation.
Result:      The existing API/local-created annotated tag has
             `verification: {verified: false, reason: "unsigned"}`. GitHub's `POST /git/tags`
             endpoint only creates annotated tag objects and does not sign them; creating a
             release from a new tag does not add a signature. The release UI creates lightweight
             tags. Build provenance from `actions/attest-build-provenance` is GitHub-signed and
             verifiable with `gh attestation verify`.
Decision:    Keep the workflow-created annotated tag, verify attestation signatures before
             publishing, and state plainly that the tag itself is unsigned. A signed tag would
             require a key registered to a GitHub account, which the user asked us to stop
             pursuing locally.
             The published v0.2.0 tag object is `03800ccb7123b2f41ee6e854db98296b26fc49f9`,
             annotated, pointing at commit `a33284b5...`, and its verification fields are both
             `null`; GitHub reports no signature rather than a verified signature. Local
             `gh attestation verify` on both downloaded tarballs passed and listed both artifact
             names as attestation subjects.
Notes:       This is a release-integrity limitation, not a substitution: no claim of a signed
             tag is made. `docs/CHANGELOG.md` and `CONTRIBUTING.md` carry the same limitation.

### EXP-013: exa depth curve and BFS truth
Date:        2026-11-22
Tool:        exa
Status:      hypothesis recorded before the depth-curve measurements
Hypothesis:  The baseline IDA* oracle will prove depth 8-11 scrambles quickly on seed 1, then
             fall off sharply because its three projections are weak. A full BFS to depth 6 will
             give exact distances that exa matches on every state through depth 4 and a sampled
             prefix at depths 5-6, pinning correctness independently of the heuristic.
Kill if:     BFS layer counts differ from the published HTM counts, exa returns a different length
             for any BFS-truth state, or an external optimal solver cannot be installed/used; in
             the last case the external cross-check is reported as unavailable, not substituted.
Setup:       Planned: `benchkit --solver exa --seed 1 --depth D` for D=8..14 with an explicit
             wall-clock timeout and partial JSON on timeout; `verify/tests/truth.rs` run with
             `--ignored`; external `nissy` attempted outside the workspace. Exact commands,
             per-depth counts and raw JSON will be added before any claim is made.
Result:      Depth curve measured 2026-11-23 on the pinned Intel Core i3-1005G1 at 3.4 GHz
             (same metadata policy as EXP-012). Command per depth:
             `target/release/benchkit --solver exa --n 20 --seed 1 --depth D --timeout-ms 300000 --json docs/data/exa-depth-D.json`
             plus `scripts/bench_meta.py`. p50 by depth 8..14: 70,350 ns, 421,863 ns,
             4,355,446 ns, 51,537,915 ns, 499,405,563 ns, 6.53 s, 37.75 s. Depth 14 timed out
             after 6 of 20 attempted solves; all other depths completed 20/20. Failed=0 and
             invalid=0 everywhere; each depth's mean solution length equalled the scramble depth
             (8.00 through 14.00). Raw JSON: `docs/data/exa-depth-{8..14}.json`.

             BFS truth: `cargo test -p verify --release --test truth -- --ignored --nocapture`
             produced layer counts 1, 18, 243, 3,240, 43,239, 574,908, 7,618,438 and
             `checked 46997 states against BFS truth`; the test passed in 3.96 s. Every checked
             exa solution was valid and had exactly the BFS distance.

             External cross-check attempt: PyPy 3.12 plus RubikOptimal 1.1.0 installed under
             `/tmp/opencode/rubikoptimal` (download and pip install both succeeded), but the
             ~963 MB table generation did not finish: two host/agent restarts cleared that tmpfs
             mid-generation, and free memory fell to about 1.2 GiB versus the ~2 GiB the solver
             needs. No external optimal length was obtained. This is reported as unavailable,
             not substituted with kociemba (which is not an optimal solver).
Decision:    Keep the baseline exa as the oracle. The depth cliff is real and is now measured;
             BFS-to-depth-6 is the independent exact-length check. The external optimal
             cross-check remains an open task, explicitly blocked by environment restarts and
             memory pressure, and no external-optimal claim is made for v0.2.0.
Notes:       exa may only print `optimal` when lower bound equals upper bound. The BFS test
             checks exact lengths, not just solution validity. The timed-out depth-14 JSON keeps
             its partial counts instead of being hidden or rescaled.

### EXP-012: fair 100k external comparison and harness floor
Date:        2026-11-22
Tool:        enso + external kociemba (timing reference only)
Status:      hypothesis recorded before the external harness was written
Hypothesis:  On 100,000 uniform random seed-1 states dumped to one committed facelet file, enso's
             cubie and facelet paths will both stay well below the two-phase reference at p50 and
             p99, while kociemba will return shorter valid solutions. Reporting the harness loop
             floor and two max-depth settings prevents a wrapper or one tuning setting from being
             mistaken for solver performance.
Kill if:     The external tool cannot be installed or cannot consume the same facelet file; any
             solution fails independent verification; the same file cannot be used for enso and
             the reference; or an obvious input/format mismatch makes the comparison meaningless.
             In that case the comparison is reported as unavailable, not replaced by a substitute.
Setup:       Planned: 100,000 states from `cubecore::Rng::new(1)` and `CubieCube::random`,
             written by `benchkit --dump-cubes` to `docs/data/cubes-100k-seed1.txt`; the same file
             is read by `benchkit --cube-file` and by an isolated external Python driver. Exact
             commands, CPU metadata, commit and raw JSON will be added to `docs/data/` and linked
             here before any claim is made.
Result:      Measured on 2026-11-22/23 with Intel Core i3-1005G1 @ 1.20 GHz (4 logical CPUs),
             pinned to `performance`, 3,400,000 kHz min/max and EPP `performance`; Linux
             7.2.14-arch1-1, rustc 1.97.1. The v3 binary was built with
             `RUSTFLAGS="-C target-cpu=x86-64-v3 --remap-path-prefix=/home/raven/enxa=." cargo build --release -p benchkit`.
             The fixed file is `docs/data/cubes-100k-seed1.txt` (SHA-256
             `94b1f7accc7eee0196e6c7a01a17d99dba65e87f22e947adbbefe862e4e905b7`).
             Exact commands (each repeated five times for enso, with `scripts/bench_meta.py`):
             `target/release/benchkit --solver enso --cube-file docs/data/cubes-100k-seed1.txt --n 100000 [--cubie|--batch] --json docs/data/<name>.json`;
             the floor used the same command plus `--overhead` (and `--cubie` where labelled).
             External commands:
             `/tmp/opencode/enxa-kociemba-venv/bin/python scripts/external_kociemba.py --cubes docs/data/cubes-100k-seed1.txt --max-depth 24 --solutions /tmp/opencode/enxa-perf/kociemba-d24-100k-seed1.txt --json docs/data/kociemba-d24-100k-seed1.json --command external-kociemba-d24`
             and the same with 21; verification used
             `target/release/benchkit --cube-file docs/data/cubes-100k-seed1.txt --verify-file /tmp/opencode/enxa-perf/kociemba-d<21|24>-100k-seed1.txt --json docs/data/kociemba-d<21|24>-verify.json`.

             Median of five runs, per solve unless noted:
             | path | p50 | p99 | avg | startup |
             |------|-----|-----|-----|---------|
             | enso cubie | 1,607 ns | 3,073 ns | 1,809 ns | 16.8 ms |
             | enso facelets | 3,564 ns | 9,712 ns | 3,808 ns | 18.1 ms |
             | enso facelets batch | 2,458 ns/solve (`batch_ns`/n) | n/a | n/a | 14.1 ms |
             | overhead cubie (no solve) | 21 ns | 29 ns | 22 ns | 15.9 ms |
             | overhead facelets (no solve) | 21 ns | 27 ns | 21 ns | 14.9 ms |
             All 15 enso runs: 100,000 solved, 0 failed, 0 invalid; mean length 31 moves, max 41.
             The harness floor is about 1.3% of the cubie p50, so it does not explain the result.

             kociemba 1.2.1, same 100,000 states:
             | max_depth | p50 | p99 | avg | mean length | max | solved/failed/invalid/over_depth |
             |-----------|-----|-----|-----|-------------|-----|----------------------------------|
             | 24 | 15,934,020 ns | 135,265,492 ns | 25,161,305 ns | 20.77 | 23 | 100000/0/0/0 |
             | 21 | 14,686,947 ns | 526,922,688 ns | 48,016,556 ns | 20.59 | 21 | 100000/0/0/0 |
             Independent Rust verification: 100,000 valid, 0 unparseable, 0 invalid for both
             settings. Depth 21 is faster at p50 but has a 527 ms p99; the previously tried
             depth 20 had a 2.98 s max on 20 cubes, so 21 and 24 are the reported settings.
             enso's cubie p50 is about 9,900x faster than kociemba d24 (about 9,100x vs d21);
             facelets p50 about 4,500x faster (about 4,100x vs d21). That speed buys valid but
             longer solutions: enso averages 31 moves versus kociemba's 20.6-20.8.
             Raw JSON: `docs/data/enso-*-run?.json`, `docs/data/overhead-*-run?.json`,
             `docs/data/kociemba-d{21,24}-100k-seed1.json` and the two `*-verify.json` files.
Decision:    Keep the comparison as the honest reference point for v0.2.0. enso is reported as the
             speed tool with its 31-move solutions; kociemba is reported only as an external
             timing reference with shorter solutions, never as a repo dependency or second goal.
Notes:       The earlier 50-cube EXP-004 comparison used depth-8 scrambles and is not reused as a
             fair 100k result. This experiment supersedes it for the final report. A metadata
             bug in the first `bench_meta.py` version wrote `cpu_model: "0"` into the d24 JSON;
             that single field was corrected to the observed model. No measurement field or
             timestamp was changed. The d24 measurement was taken before, and d21 after, the
             commit that fixed `bench_meta.py`; each JSON records its own commit.

### EXP-009 step 0: re-profile the packed descent path
Date:        2026-11-22
Tool:        enso
Status:      profiling recorded before any further optimization attempt
Hypothesis:  With startup moved to a validated file, the remaining per-solve bottleneck should
             still be dependent loads in the packed stages. If cache/DTLB events per solve are
             small or do not dominate, prefetch, huge pages and macro entries are not worth
             trying on this machine.
Kill if:     This is a guide-only profile, not an optimization. It is invalid if the counters
             cannot be read at `perf_event_paranoid=2` or if the n=0 baseline includes solves.
Setup:       `main` after the EXP-008 merge, v3 benchkit binary, `perf stat` with cache-misses,
             cache-references, LLC-load-misses, L1-dcache-load-misses and dTLB-load-misses,
             n=0 versus n=100000 with seed 1 and a warm `ENSO_TABLES` file. Exact output will be
             pasted below.
Result:      `perf_event_paranoid=2`. n=0 baseline: 82,558 cache misses, 3,980 LLC misses,
             234,041 L1 misses, 341 dTLB misses. n=100000 cubie: 1,910,733 cache misses,
             827,327 LLC misses, 3,213,591 L1 misses, 30,019 dTLB misses. Per solve (after
             subtracting n=0): 18.3 cache misses, 8.2 LLC misses, 29.8 L1 misses, 0.30 dTLB
             misses. Facelet n=100000: 2,347,758 cache misses, 946,396 LLC misses, 3,628,696
             L1 misses, 35,719 dTLB misses; per solve 22.6 / 9.4 / 34.0 / 0.35. The profile
             still shows a dependent-load chain (tens of L1 misses and ~8-9 LLC misses per
             solve), but dTLB pressure is negligible.
Decision:    guide only
Notes:       No code change was made from profiling alone. Prefetch and macro entries are
             justified by the dependent loads; huge pages are not, because dTLB misses are only
             ~0.3 per solve. Any later attempt needs its own hypothesis, interleaved A/B with
             p50 AND p99, and the two-consecutive-<3% stop rule.

### EXP-010: prefetch the next packed entry
Date:        2026-11-22
Tool:        enso
Status:      implemented, benchmarked, kept; enabled by default
Hypothesis:  Each packed lookup yields the next coordinate, so issuing one L2/L3 prefetch for
             `entry[next]` while updating the cube arrays may hide part of the ~8-9 LLC misses
             per solve without changing the solution or table layout.
Kill if:     Interleaved A/B shows p50 or p99 regressing, either metric improves by less than 3%,
             any solution is invalid, or the 1M gate fails. Two consecutive attempts below 3%
             stop this optimization line.
Setup:       Feature `prefetch` on top of the EXP-008 `main`; A = baseline without the feature,
             B = same binary built with it. v3 flags, seed 1, n=100000, cubie and `--facelets`,
             at least three A/B/B/A pairs. Safe scalar fallback remains the default.
Result:      The first six-pair run was invalidated when the CPU was unplugged and its frequency
             governor changed mid-run; those numbers were discarded. After the user replugged it
             and all cores were pinned to `performance` with min=max=current 3,400,000 kHz and
             EPP `performance`, the rerun gave cubie p50 2160.5 -> 1922 ns (-11.0%) and p99
             6164 -> 5520 ns (-10.4%); facelets p50 2888 -> 2739.5 ns (-5.1%) and p99
             8444.5 -> 7912.5 ns (-6.3%). All 24 runs solved 100,000 cubes with 0 failed /
             0 INVALID. `cargo test --workspace --release --features enso/prefetch` passed;
             `cargo test -p verify --release --test gate --features prefetch -- --ignored
             --nocapture` reported 0 invalid / 0 failed, avg 31.343, max 41. Logs:
             `/tmp/opencode/enxa-perf/ab_exp010_{cubie,facelets}.log`.
Decision:    keep, enabled by default
Notes:       The prefetch call is behind the `prefetch` feature with an inline SAFETY comment and
             a non-x86 no-op; `--no-default-features` is the tested scalar reference and its 1M
             gate also reported 0 invalid / 0 failed. This is an x86_64 intrinsic, not a new table
             format, so committed table files remain valid.

### EXP-011: two-move macro entries
Date:        2026-11-22
Tool:        enso
Status:      implemented, benchmarked, reverted
Hypothesis:  A stage entry that stores two consecutive `(move, next)` pairs halves the number of
             dependent table loads per stage. The extra bytes may stay within LLC, reducing p50
             and p99 together.
Kill if:     Interleaved A/B shows p50 or p99 regressing, either metric improves by less than 3%,
             any solution is invalid, or the 1M gate fails. Two consecutive attempts below 3%
             stop this optimization line.
Setup:       A = default EXP-010 build (prefetch on, macro off), B = same tree with
             `--features macrosteps`. Both v3 flags, seed 1, n=100000, cubie and `--facelets`,
             three A/B/B/A pairs with all cores pinned to performance/3.40 GHz. Logs:
             `/tmp/opencode/enxa-perf/ab_exp011_{cubie,facelets}.log`.
Result:      Cubie p50 2002 -> 2008.5 ns (+0.3%, no gain), p99 5157 -> 4781.5 ns (-7.3%);
             facelets p50 2938 -> 2795 ns (-4.9%), p99 8078 -> 6932.5 ns (-14.2%). The facelet
             `perf stat` run showed cache misses 4.63M -> 6.17M, LLC misses 0.50M -> 1.39M, L1
             misses 4.43M -> 6.78M and dTLB misses 69k -> 177k. Macro tables add 4 bytes per
             packed entry (7,110,260 bytes), taking in-memory hot tables from 7,258,868 to
             14,369,128 bytes. All 24 runs solved 100,000 cubes with 0 failed / 0 INVALID.
Decision:    revert
Notes:       The keep rule requires p50 AND p99 to improve; cubie p50 did not. This is one
             sub-3% attempt and the optimization line is stopped here rather than iterated:
             the extra random-access array increases LLC/DTLB pressure. The default build and
             all committed table files remain one-move packed stages.

### EXP-008: validated prebuilt packed tables
Date:        2026-11-22
Tool:        enso
Status:      implemented, benchmarked, kept
Hypothesis:  The packed tables are deterministic and take ~7.26 MB, so loading a validated file
             should cut enso startup (currently ~475 ms cold, ~482 ms warm) to a few
             milliseconds without changing solve latency. A header with magic, format version,
             expected sizes and a checksum over the payload can reject wrong or corrupt files
             instead of silently using them; an in-process build fallback keeps the file
             optional.
Kill if:     A valid file fails verification, a mismatched/corrupt file is accepted, the
             serialized bytes do not round-trip to the in-process tables, solve latency changes,
             or the 1M gate reports any invalid/failed solution.
Setup:       Branch `perf`, A = detached worktree at 1f8e1c5, B = current tree. Both built with
             `-C target-cpu=x86-64-v3 --remap-path-prefix`, seed 1, n=100000, three A/B/B/A pairs
             each for cubie and `--facelets`. Cache path overridden with `ENSO_TABLES` to a fresh
             file under `/tmp/opencode/enxa-perf/tableexp/`. CPU: Intel Core i3-1005G1 @ 1.20 GHz,
             Linux 7.2.14-arch1-1, rustc 1.97.1. Logs: `/tmp/opencode/enxa-perf/ab_exp008_*.log`.
Result:      A vs B startup median 426.38 -> 13.18 ms (cubie runs) and 427.89 -> 13.25 ms
             (facelet runs); first cold build+write on a fresh path 399.11 ms. Forced rebuild via
             `enso --build-tables` completed in 464.20 ms. A truncated file and a file with one
             flipped payload byte were both reported on stderr and rebuilt; unit tests confirm
             encode/decode byte identity and reject wrong magic, version, size and checksum.
             Solve-path p50: cubie 1535 -> 1537 ns (+0.1%), facelets 2216 -> 2227 ns (+0.5%).
             p99: cubie 3680.5 -> 3464 ns (-5.9%), facelets 4825.5 -> 5008 ns (+3.8%, drift).
             All 24 benchmark runs solved 100,000 cubes with 0 failed / 0 INVALID.
             `./scripts/check.sh` passed; both 1M gates 0 invalid / 0 failed, avg 31.343, max 41.
Decision:    keep
Notes:       Startup is reported separately; no solve-path table layout was changed. The p50
             results are within run-to-run noise; the p99 changes are not used as a performance
             claim for the solve path. The stage-3 construction-only `t3` field was removed and
             its test replaced by an encode/decode/closure check over the same G2 coordinate
             domain, so loaded tables can be decoded without unavailable construction inputs.

### EXP-007: cheap array updates for the facelet coordinate path
Date:        2026-11-21
Tool:        enso
Status:      implemented, benchmarked, kept
Hypothesis:  EXP-006 removed the large random-access move tables but reintroduced per-move
             `% 3` / `% 2` divisions and updates to `eo` after stage 1 and `co` after stage 2,
             even though those arrays are never read again. Replacing the divisions with a
             conditional subtract / bitmask and updating each array only while it is still
             needed should recover the ~3.7% facelet p50 regression while keeping the cache
             and table-size win.
Kill if:     Interleaved A/B against EXP-006 shows no reproducible facelet p50 improvement, or
             cubie latency regresses, or cache misses rise back toward EXP-005 levels, or any
             solution fails verification, or the 1M gate fails.
Setup:       Branch `perf`, EXP-006 candidate. A = EXP-006 binary (sha256 bd15082c is the
             EXP-005 A binary; A here is the EXP-006 B binary). Same v3 flags, seed 1,
             n=100000, cubie and `--facelets`, six interleaved pairs. EXP-006 medians: cubie
             p50 1582.5 / p99 3721 ns; facelets p50 2482 / p99 4868 ns; facelet cache misses
             24.2/solve, hot tables 7,258,868 bytes.
Result:      Six interleaved pairs (`ab_{cubie,facelets}_exp007.log`), A = EXP-006, B = this
             variant: facelets p50 median 2495.5 -> 2300.5 ns (-7.8%), p99 4625.5 -> 4676.5 ns
             (+1.1%, noise), avg 2724 -> 2537 ns (-6.9%). Cubie p50 1600 -> 1627.5 ns and
             p99 3721.5 -> 3432 ns are run-to-run drift: the cubie solve code is unchanged.
             Facelet cache misses stay at 23.7/solve (EXP-006 24.2, EXP-005 44.2), LLC 9.4,
             L1 35.7, dTLB 1.1. Hot tables remain 7,258,868 bytes.
             Final A/B against the published v3 tarball, six interleaved pairs
             (`ab_cubie_final_step3.log`, `ab_facelets_final_step3.log`):
             cubie p50 2291 -> 1559.5 ns (-31.9%), p99 6716 -> 3306.5 ns (-50.8%);
             facelets p50 3646 -> 2232.5 ns (-38.8%), p99 9191 -> 4406.5 ns (-52.0%).
             Every one of the 12 final runs solved 100,000 cubes with 0 failed/INVALID.
             `./scripts/check.sh` passed; both 1M gates 0 invalid/0 failed, avg 31.343, max 41.
Decision:    keep
Notes:       Pure hot-path refinement; no table layout or stage behavior changes. Startup is
             still reported separately: cold n=0 A 462.9 -> B 474.4 ms; warm medians A 454.2 ->
             B 481.6 ms. The ~5% startup increase comes from Step 2's packing pass, not from
             the facelet path. Logs/perf data: `/tmp/opencode/enxa-perf/`.

### EXP-006: drop incremental permutation tables from the facelet path
Date:        2026-11-21
Tool:        enso
Status:      implemented, benchmarked; iterating
Hypothesis:  EXP-005 kept 1.45 MB cperm + pattern move tables to avoid recomputing stage-3
             coordinates after every move. Those random lookups increased facelet cache misses
             from ~24.5 to ~44.2 per solve and added startup work. Packed stages already yield
             the next composite coordinate; the three factors are only needed at stage-3 entry
             and stage-4 entry. Tracking cp/ep arrays (20 bytes) and ranking at those two
             boundaries should cut table bytes and cache misses without losing latency.
Kill if:     Interleaved A/B against the current EXP-005 candidate shows a reproducible p50 or
             p99 regression beyond run-to-run drift, or any solution fails verification, or the
             1M gate fails.
Setup:       Branch `perf`, EXP-005 candidate frozen as A (`/tmp/opencode/enxa-perf/exp005-a-benchkit`),
             EXP-006 candidate as B. Same v3 flags, seed 1, n=100000, cubie and `--facelets`,
             six interleaved pairs. A start medians: cubie p50 1592.5 / p99 3498.5 ns; facelets
             p50 2443 / p99 5587.5 ns; hot tables 8,959,752 bytes.
Result:      Six interleaved pairs (`ab_{cubie,facelets}_exp006.log`). A vs B medians:
             cubie p50 1629 -> 1582.5 ns (-2.9%), p99 3656.5 -> 3721 ns (+1.8%, noise; cubie
             solve code is unchanged); facelets p50 2393.5 -> 2482 ns (+3.7%), p99 5796 -> 4868
             ns (-16.0%), avg 2762.5 -> 2696.5 ns (-2.4%). Hot tables 8,959,752 -> 7,258,868
             bytes (-19.0%). Perf deltas over the n=0 startup baseline, per solve: facelet cache
             misses 44.2 -> 24.2, LLC 19.0 -> 10.1, L1 81.1 -> 35.5, dTLB 2.7 -> 1.3.
             `cargo test --workspace --release` 25 passed; both 1M gates 0 invalid / 0 failed,
             avg length 31.343, max 41.
Decision:    iterate, not kept as-is: the cache/table hypothesis is confirmed, but the facelet
             p50 regression is reproducible and triggers the kill criterion.
Notes:       The per-move `% 3` / `% 2` divisions and unnecessary `eo`/`co` updates are the
             likely cause of the median regression. EXP-007 tests that before choosing a final
             variant. No solution was ever invalid.

### EXP-005: direct facelets -> stage coordinates
Date:        2026-11-21
Tool:        enso
Status:      implemented, benchmarked; refined by EXP-006/007
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
Result:      Six interleaved pairs against the Step 2 candidate (`ab_{cubie,facelets}_step3.log`):
             cubie p50 median 1588.5 -> 1592.5 ns (unchanged), p99 3597 -> 3498.5 ns;
             facelets p50 median 3021.5 -> 2443 ns (-19.1%), p99 5870.5 -> 5587.5 ns (-4.8%).
             Against the published v3 tarball (`ab_{cubie,facelets}_final.log`): cubie p50
             2328.5 -> 1561.5 ns, p99 6853 -> 3439 ns; facelets p50 3587 -> 2489.5 ns,
             p99 8978.5 -> 5399 ns. All runs 0 failed/INVALID. 1M cube gate and new 1M facelet
             gate: 0 invalid, 0 failed, avg 31.343, max 41.
Decision:    kept as the direct-path approach; the incremental move-table implementation was
             then replaced by EXP-006/007 after measuring its cache cost.
Notes:       This is Step 3 from the plan. It is distinct from Step 2 and was benchmarked
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
Status:      Step 2 and Step 3 implemented and kept (EXP-005/006/007)
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
