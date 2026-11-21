# Architecture

This document describes the current layout after the pure structural refactor. There are
exactly two tools: `enso` (fastest valid solve) and `exa` (HTM-optimal solve).

## Source layout

```
core/src/
  lib.rs              public re-exports and crate-level conventions
  coord/mod.rs        coordinate API and re-exports
  coord/twist.rs      corner twist coordinate
  coord/flip.rs       edge flip coordinate
  coord/slice.rs      UD-slice coordinate
  coord/cperm.rs      corner permutation coordinate
  cubie.rs            cubie arrays, multiplication, inversion, validation
  facelet.rs          54-character sticker mapping
  moves.rs            18 HTM moves, parsing, formatting, legality
  rng.rs              seeded deterministic RNG
  solver.rs           Solver trait
  tables.rs           basic coordinate move tables

enso/src/
  lib.rs              public API: Enso, MAX_SOLUTION_LEN
  main.rs             tiny binary entry point
  cli.rs              CLI parsing and solution verification
  chain.rs            reusable descent-stage BFS
  solver.rs           Enso and the allocation-free descent path
  subgroup.rs         subgroup predicates, generator sets, ranking helpers
  tables.rs           startup construction of all four stage tables
  stages/mod.rs       stage module declarations and chain documentation
  stages/orient.rs    G0→G1 edge orientation
  stages/slicecorner.rs G1→G2 twist + slice
  stages/tetrad.rs    G2→G3 paired corner/edge coset
  stages/final.rs     G3→solved half-turn subgroup

exa/src/
  lib.rs              public API: Exa
  main.rs             tiny binary entry point
  cli.rs              CLI parsing and solution verification
  ida.rs              iterative-deepening search
  heuristic.rs        admissible projection tables

verify/tests/
  cube.rs             cubie model tests
  coords.rs           coordinate tests
  facelets.rs         facelet and algorithm-string tests
  exa.rs              exa integration tests
  enso.rs             enso integration tests
  gate.rs             ignored million-cube gate
```

## Ownership rules

Every module documents what it owns and what it must not do. Keep those boundaries:

- `core` owns conventions and primitives; it never solves, searches or benchmarks.
- `enso` owns speed; its solve path performs table lookups into fixed allocations, emits
  into a caller-supplied buffer, uses no `dyn` and never allocates on the hot path.
- `exa` owns optimality; pruning tables must be provably admissible, and optimality is
  claimed only when lower bound equals upper bound.
- `bench` owns measurement; it never caches answers or changes the metric.
- `verify` owns correctness assertions; it never implements solver logic.

## Public APIs

| Crate | Root API |
|-------|----------|
| `cubecore` | cube/move/coordinate primitives and the `Solver` trait, re-exported from `lib.rs` |
| `enso` | `Enso`, `MAX_SOLUTION_LEN` |
| `exa` | `Exa` |
| `benchkit` | binary only |
| `verify` | binary/test crate only |

Internal modules are private where possible. The `coord` module keeps its submodules
private and re-exports each coordinate's constants and decoders from `coord/mod.rs`.

## Data flow

A scramble enters as cubies or a 54-character facelet string. `benchkit` and the CLIs
construct a `CubieCube`, call the selected `Solver`, then apply the returned moves to the
original cube and require `is_solved()`. Startup table construction is timed separately
from the solve.

enso's four stages are G0→G1 (all 18 moves), G1→G2 (`<U,D,L,R,F2,B2>`), G2→G3
(`<U,D,L2,R2,F2,B2>`), and G3→solved (`<U2,D2,L2,R2,F2,B2>`). The G2→G3 set is not a
typo: L/R quarter turns move UD-slice edges out of the slice and would break G2.
