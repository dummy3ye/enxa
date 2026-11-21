# Roadmap

Status key: [x] done, [ ] todo. Update this file in the same change that finishes a task.

## M0: scaffold
- [x] workspace, crates, CI, agent instructions
- [x] **first compile + `cargo test --workspace --release` green** (the scaffold was written without a toolchain; this was fixed before M1)
- [x] pure structural refactor into module ownership layout (EXP-002)

## M1: core (written)
- [x] cubie model, 18 moves (compile-time), inverse, verify, random
- [x] coordinates: twist, flip, slice, cperm (+ decoders)
- [x] move tables
- [x] facelet parse/format
- [x] RNG, Solver trait
- [ ] 48 symmetries + inverse (needed for M8)
- [x] more coordinates (edge perm parts, coset coordinates) — added for M3

## M2: exa baseline
- [x] IDA* with 3 admissible projections (reference oracle)
- [ ] benchmark depth curve 8..14, record in EXPERIMENTS.md
- [ ] cross-check against an external optimal solver on 100 scrambles

## M3: enso v1 (Variant A, Thistlethwaite chain)
- [x] `DescentStage` building block + test
- [x] stage definitions G0>G1>G2>G3>solved with correct coordinates
      (G2→G3 uses `<U,D,L2,R2,F2,B2>`, not L/R quarters — see EXP-001)
- [x] `Enso::solve`, zero allocation on the descent path (`solve_into` with fixed buffer)
- [x] 1M random cubes: 0 invalid, 0 failed (avg 31.343, max 41)
- [x] benchmark: p50/p99, avg/max length, startup separated, facelet input (EXP-001)

## M4: baselines to beat
- [x] two-phase reference (external kociemba, EXP-004)
- [x] benchmark report: enso vs baseline, same machine, same seeds

## M5: enso tuning
- [x] packed entries (EXP-003) and next-entry prefetch (EXP-010); huge pages untested
- [x] validated prebuilt table file with in-process fallback (EXP-008)
- [ ] mmap-able table files
- [x] facelet to coordinate fast path (EXP-005/006/007)
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
