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

`enso::chain::DescentStage` is the generic building block (built and unit-tested). Variant A is implemented in `enso::variant_a`: four descent stages with subgroup assertions after each one, and an allocation-free `Enso::solve_into` that writes into a fixed buffer. Measured results are in `docs/EXPERIMENTS.md` (EXP-001).

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
| A: Thistlethwaite chain | 4 (G0>G1>G2>G3>solved) | ~31 measured (max 41) | ~127 MB measured | simplest, most steps; implemented in M3 |
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
- Startup cost (building tables) was expected to be seconds; Variant A measured 465-547 ms. Loading prebuilt tables from disk via mmap is the fix and is part of M5.
- Assembly is a last resort: only for a function the profiler names, only with a scalar fallback and tests against it.

## Correctness gate

On 1,000,000 random solvable cubes: 0 invalid solutions, 0 failures. Always.
