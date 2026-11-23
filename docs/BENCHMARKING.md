# Benchmarking rules

1. **Release build only**: use `./scripts/package.sh <version> <v2|v3>` for portable release binaries, or `RUSTFLAGS="-C target-cpu=native" cargo build --release` for local native experiments. Record CPU model, OS, rustc version, commit and the exact target-cpu flavor. Release builds never use `native`.
2. **Fixed seeds.** Same seed = same cubes. Compare solvers on identical inputs.
3. **Warm up**, then measure. Report median and p99, not just the mean. Report failures and invalid solutions.
4. **Verify every solution** (the harness does). A faster solver that returns a bad solution has no time.
5. **Separate** startup (table build/load) from per-solve time. Report both.
6. **Sub-microsecond timing:** one `Instant::now()` pair costs tens of ns. For enso, also time batches (N solves, divide) and report the batch method; `benchkit --batch` is available.
7. **Latency vs throughput** are different benchmarks. Label which one a number is.
8. **Pin the machine:** close other loads, fix CPU frequency if you can, repeat 5 times, report the spread.
9. **Compare fairly:** same metric, same input form (cubie vs facelets), same build flags class.
10. No number enters a doc or README without the command that produced it.

## Harness flags

`benchkit` writes machine-readable output with `--json PATH`. It can dump the exact seeded
states with `--dump-cubes PATH --n N --seed S`, then reload the same file with
`--cube-file PATH --n N`. By default a cube file is timed as facelet strings; add `--cubie`
to time `CubieCube` solves on the same file. These paths are reported separately and must
never be mixed into one latency number.

`--overhead` times the identical input loop without calling the solver. This is the harness
floor for timer, `black_box`, indexing and input access; it is not a solve. `--batch` reports
throughput and excludes per-solve `Instant` pairs. `--timeout-ms N` stops the per-solve loop
after the deadline and writes `attempted`, `timed_out` and `timeout_ms` into the JSON; a
process-level `timeout` command is still used as a backstop for a single hung solve.

`--verify-file PATH` reads one whitespace-separated move sequence per line and checks it
against `--cube-file` with `cubecore`, independently of the solver that produced it. The
result is written as JSON when `--json` is supplied.

`scripts/bench_meta.py` adds commit, dirty flag, CPU model/flags/frequency, OS, rustc,
cube-file SHA-256 and the exact command to a JSON report. Raw JSON lives in `docs/data/`.

## Fair 100k comparison

The fixed input file is `docs/data/cubes-100k-seed1.txt`: 100,000 uniformly random solvable
states from `CubieCube::random` with `Rng::new(1)`, one 54-char URFDLB facelet string per
line. It is generated once and reused by every solver, so no implementation gets easier
scrambles. Commands and results for the final run are recorded in `docs/EXPERIMENTS.md`
(EXP-012) and the committed JSON files it names.

External solvers are timing references only. They are installed outside the repository,
never added as workspace dependencies. `scripts/external_kociemba.py` consumes the same cube
file, writes every returned solution to a dump for independent Rust verification, records
the no-op call floor, and emits raw JSON with per-solve lengths and times. Two `max_depth`
settings are measured; a shallow depth that causes pathological search is reported as a
failed attempt rather than quietly chosen as the headline number.

### Result (2026-11-22/23)

Hardware: Intel Core i3-1005G1 @ 1.20 GHz, 4 logical CPUs pinned to `performance`,
3,400,000 kHz min/max and EPP `performance`; Linux 7.2.14-arch1-1; rustc 1.97.1; v3 build.
Every number below comes from the committed JSON named in EXP-012 in
`docs/EXPERIMENTS.md`; medians are across five runs where applicable.

| measurement | p50 | p99 | mean length | solved / invalid |
|-------------|-----|-----|-------------|------------------|
| enso, cubie input | 1,607 ns | 3,073 ns | 31 | 100,000 / 0 |
| enso, facelet input | 3,564 ns | 9,712 ns | 31 | 100,000 / 0 |
| enso, batch facelets | 2,458 ns/solve | n/a | 31 | 100,000 / 0 |
| harness floor, no solve | 21 ns | 27-29 ns | n/a | n/a |
| kociemba 1.2.1, max_depth 24 | 15.93 ms | 135.27 ms | 20.77 | 100,000 / 0 |
| kociemba 1.2.1, max_depth 21 | 14.69 ms | 526.92 ms | 20.59 | 100,000 / 0 |

enso's startup (table load/build) is 14-18 ms and is reported separately, never folded into
per-solve latency. Both kociemba solution dumps were independently re-verified with
`benchkit --verify-file`: 100,000 valid, 0 unparseable, 0 invalid at each depth. The 2,458
ns/solve batch figure comes from the median `batch_ns` of 245,842,930 ns divided by 100,000;
batch timing deliberately omits per-solve `Instant` pairs.
