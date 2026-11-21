# cubecore

The shared, dependency-free foundation for `enso` and `exa`. It owns the Kociemba cubie
model, the 18 HTM moves, coordinate encodings, basic move tables, facelet strings, the
seeded RNG and the `Solver` trait.

It must NOT solve cubes, search, benchmark or parse CLI arguments.

## Public API

The crate root re-exports the primitives used by the two solvers. Implementation modules
stay internal where possible; the coordinate module keeps stable re-exports for its
individual coordinate files.

## Commands

Run from the repository root:

```bash
cargo test -p cubecore --release
cargo test -p verify --release --test cube
cargo test -p verify --release --test coords
cargo test -p verify --release --test facelets
```

Conventions are load-bearing; see `docs/DESIGN.md`.
