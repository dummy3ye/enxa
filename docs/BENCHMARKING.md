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
