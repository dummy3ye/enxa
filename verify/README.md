# verify

Correctness tests for the whole workspace. This crate owns tests, never solver logic.

| File | Scope |
|------|-------|
| `tests/cube.rs` | cubie operations, move orders, inversion, superflip |
| `tests/coords.rs` | coordinate round-trips and move-table agreement |
| `tests/facelets.rs` | facelet strings and algorithm parsing/formatting |
| `tests/enso.rs` | enso solver API correctness |
| `tests/exa.rs` | exa oracle correctness on short scrambles |
| `tests/gate.rs` | ignored 1,000,000-cube enso gate |
| `tests/truth.rs` | ignored BFS-to-depth-6 exact-length oracle for exa |

## Commands

Run from the repository root:

```bash
cargo test --workspace --release
cargo test -p verify --release --test enso
cargo test -p verify --release --test gate -- --ignored --nocapture
cargo test -p verify --release --test truth -- --ignored --nocapture
```

The gate is ignored by default so ordinary test runs stay fast. It remains a required
check for changes that can affect enso correctness.
