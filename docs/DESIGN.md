# Design and conventions

These are load-bearing. Changing any of them means changing `core/`, every table, and
`verify/tests/{cube,coords,facelets}.rs` together.

## Cube model (cubie level, Kociemba conventions)

- Corners `URF UFL ULB UBR DFR DLF DBL DRB` = 0..7. Edges `UR UF UL UB DR DF DL DB FR FL BL BR` = 0..11.
- `cp[slot]` = which corner cubie is in that slot; `co[slot]` = its twist (0..3). Same for `ep` / `eo` (flip 0..2).
- `a.multiply(&b)` = apply `a`, then `b`. A move is applied as `cube.multiply(&MOVES[m])`.
- Solvable iff: both permutations valid, sum(co) % 3 == 0, sum(eo) % 2 == 0, parity(cp) == parity(ep).
- `MOVES: [CubieCube; 18]` is computed at compile time from the six basic quarter turns.

## Moves and metric

- Faces `U R F D L B` = 0..5. `Move(face * 3 + power - 1)`, power 1/2/3 = `X` / `X2` / `X'`.
- Metric is **HTM**: all 18 moves cost 1. God's number = 20.
- Redundancy pruning (`Move::can_follow`): no repeated face; opposite faces only in canonical order (U before D, R before L, F before B).

## Coordinates

| Coordinate | Size | Meaning |
|------------|------|---------|
| twist | 3^7 = 2187 | corner orientations (last is implied) |
| flip | 2^11 = 2048 | edge orientations (last is implied) |
| slice | C(12,4) = 495 | which 4 slots hold UD-slice edges (FR FL BL BR); solved = 494 |
| cperm | 8! = 40320 | corner permutation (Lehmer rank) |

Move tables: `table[coord * 18 + move] -> coord`, `u16`.

## Facelets

54 chars, order `U R F D L B`, nine each. Solved = `UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB`. After `R` from solved: `UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB`.

## Validation history

Before the first compile, these properties were verified with an independent Python port of the same tables: every move has order 4; `R U R' U'` has order 6; the 20-move superflip algorithm yields exactly "all edges flipped"; facelet round-trips; coordinate encode/decode round-trips; move tables agree with real cube moves; `inverse()` is a true inverse. The Rust tests in `verify/tests/{cube,coords,facelets}.rs` re-assert all of these.

## Additional coordinates implemented for M3

- Edge permutation pieces and the half-turn edge-rank used by enso stage 4.
- Tetrad/edge-orbit patterns and corner left-coset classes for enso stage 3.
- Exact G3 membership indices used by `Enso::solve_into`.

## Planned (not yet implemented)

- 48-element symmetry group (and inverse) with symmetry-reduced coordinates: needed for exa's meet-in-the-middle tables.
- UD-slice sorted coordinate.
