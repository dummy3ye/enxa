# Benchmarking rules

1. **Release build only**: `cargo build --release`, flags from `.cargo/config.toml` (`target-cpu=native`). Record CPU model, OS, rustc version, commit.
2. **Fixed seeds.** Same seed = same cubes. Compare solvers on identical inputs.
3. **Warm up**, then measure. Report median and p99, not just the mean. Report failures and invalid solutions.
4. **Verify every solution** (the harness does). A faster solver that returns a bad solution has no time.
5. **Separate** startup (table build/load) from per-solve time. Report both.
6. **Sub-microsecond timing:** one `Instant::now()` pair costs tens of ns. For enso, time batches (N solves, divide) and report the batch method. TODO: add `--batch` to `benchkit`.
7. **Latency vs throughput** are different benchmarks. Label which one a number is.
8. **Pin the machine:** close other loads, fix CPU frequency if you can, repeat 5 times, report the spread.
9. **Compare fairly:** same metric, same input form (cubie vs facelets), same build flags class.
10. No number enters a doc or README without the command that produced it.
