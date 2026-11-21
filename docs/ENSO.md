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

`DescentStage` in `enso/src/chain.rs` is the generic building block (built and unit-tested).
Variant A lives in `enso/src/solver.rs`, `subgroup.rs`, `tables.rs`, `tablefile.rs` and
`enso/src/stages/`: four descent stages with subgroup assertions after each one, and an
allocation-free `Enso::solve_into` that writes into a fixed buffer. `tablefile.rs` stores the
packed tables in a validated cache file (magic, format version, expected sizes, payload
checksum); a missing or mismatched file is reported and rebuilt in-process. The path is
`$ENSO_TABLES`, otherwise `$XDG_CACHE_HOME/enxa/` or `$HOME/.cache/enxa/`, otherwise the system
temp directory. `enso --build-tables` forces a rewrite. CLI work is isolated in
`enso/src/cli.rs`; `main.rs` only calls it. Measured results are in `docs/EXPERIMENTS.md`
(EXP-001, EXP-008).

### Variant A stage chain (derived and verified, not assumed)

| Stage | Moves | Coordinate / goal | States |
|-------|-------|-------------------|--------|
| G0→G1 | all 18 | flip → 0 | 2,048 |
| G1→G2 | U,D,L,R,F2,B2 | (twist, slice) → (0, 494) | 1,082,565 |
| G2→G3 | U,D,L2,R2,F2,B2 | tetrad pattern × edge-orbit pattern × corner left-coset class → goal class | 29,400 |
| G3→solved | U2,D2,L2,R2,F2,B2 | (corner perm, edge perm) → identity | 663,552 |

The third stage cannot use L/R quarter turns: they move UD-slice edges out of the slice and break G2. Its coordinate was derived from the half-turn group's corner tetrads `{0,2,5,7}/{1,3,4,6}` and edge orbits `{0,2,4,6}/{1,3,5,7}/{8,9,10,11}`.

## Three variants to build and compare

| Variant | Stages | Expected length | Table size | Notes |
|---------|--------|-----------------|------------|-------|
| A: Thistlethwaite chain | 4 (G0>G1>G2>G3>solved) | ~31 measured (max 41) | 7,258,868 packed bytes | simplest, most steps; implemented in M3, packed in EXP-003 |
| B: two-phase, first solution only | 2 | ~25-30 | ~100s of MB | phase 1 via descent, phase 2 via descent, no length optimization |
| C: fat tables | 2-3 | 25-30 | GBs | fewest steps, DRAM-latency bound |

**The knob is table size vs. step count.** The experiment is to find the knee for the target machine, measured, not guessed.

## Things most likely to actually matter

1. **Input handling.** If input is a 54-char facelet string, parsing it into coordinates may cost more than the solve. Report both numbers. enso now parses once into `FaceletParts` and descends on coordinates without building a `CubieCube` (EXP-005/006/007); a SIMD coordinate extractor is a separate potential win.
2. **Memory latency.** A dependent-load chain is latency-bound. Entry packing is implemented
   (EXP-003), and the next coordinate returned by each load is prefetched while the cube arrays
   are updated (EXP-010, default-on with `--no-default-features` as the scalar fallback). Huge
   pages were skipped because the re-profile showed only ~0.3 dTLB misses per solve (EXP-009).
3. **Macro entries.** Instead of one move per lookup, let a stage entry hold a whole short move sequence (2-4 moves) when it fits. Fewer dependent loads, bigger tables. Measure.
4. **Output.** Writing moves into a fixed `[u8; N]` buffer, no `Vec`.
5. **Batch mode.** Many cubes in flight at once hides load latency (interleave N independent descents per thread; SIMD gathers; GPU later). Throughput is a separate benchmark from latency.

## Honest expectations

- Single-solve latency in the low microseconds is plausible; sub-microsecond is a *hypothesis* to be earned with measurements, not a promise.
- Startup cost (building tables) was expected to be seconds; Variant A measured 465-547 ms. A validated prebuilt file now reduces warm startup to ~13 ms (EXP-008); a cold build+write is ~400-470 ms and is reported separately. mmap is still an unimplemented M5 idea.
- Assembly is a last resort: only for a function the profiler names, only with a scalar fallback and tests against it.

## Correctness gate

On 1,000,000 random solvable cubes: 0 invalid solutions, 0 failures. Always.
