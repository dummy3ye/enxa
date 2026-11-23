# exa: design

## Goal

The fewest possible moves, **provably**. HTM metric.

## Reality check (so effort goes to the right place)

- God's number is 20 in HTM (proven 2010). exa can never need more.
- A random state needs about 17.7 moves on average; the vast majority need 16-19.
- So "fewer moves than ever" is bounded. The open frontier is **how cheaply optimality can be proven**.

## Baseline (M2, written): IDA* with admissible projections

`exa::Exa` (implemented in `exa/src/ida.rs`). The pruning tables live in
`exa/src/heuristic.rs`; CLI work lives in `exa/src/cli.rs`. Heuristic = max of exact
distances in three projections: (twist, slice), (flip, slice), (corner perm). Exact,
simple, correct, slow on deep states. It is the oracle every other engine is tested against.

## Ideas to try (each is a hypothesis with a kill criterion)

### 1. Bound squeeze
Run an *upper-bound* finder (fast two-phase-style search returning some solution of length U) and a *lower-bound* prover (admissible tables showing nothing shorter than L exists). Stop when U = L.
- Exactness lives entirely in the lower-bound side; heuristic/ML guidance may only reorder the upper-bound search.
- **Kill criterion:** if on 1,000 random states the average time to certify is not at least 2x better than the oracle plus plain IDA* on the same tables, drop it.

### 2. Meet-in-the-middle with symmetry reduction
Count of states within d moves (HTM): d=8: 1.33x10^9 at exactly 8; cumulative through 9: about 1.9x10^10. Dividing by the 96-element (48 symmetries x inversion) group gives roughly 2x10^8 canonical states: storable as hashed fingerprints in a few GB. Then search about 10-11 moves from the scramble and look up hits.
- **Caveat, stated plainly:** searching 11 deep from the scramble is ~10^12 nodes without good pruning. This is only viable with strong pruning and/or a GPU. That is the experiment.
- **Kill criterion:** if the table build plus lookup cost exceeds the oracle's time on states up to depth 16, it's not worth it below that depth.

### 3. Coset-based admissible heuristics
Distances to cosets of H = <U, D, R2, L2, F2, B2> are much stronger lower bounds than classic corner/edge PDBs. Stack them with symmetry reduction.
- **Kill criterion:** if average h at the root is not at least 1.5 higher than the baseline projections on random states, drop it.

## Verification protocol

- Oracle agreement: for every new engine, same length as `Exa` on a shared seed set (`verify/tests/`).
- Known results: superflip = 20 moves exactly; any scramble of k moves has optimal length <= k.
- Emit `(lower_bound, upper_bound, path)`; optimal iff equal.
- Cross-check a sample against an external solver (nissy, Cube Explorer) before publishing numbers.

## Speed expectations (baseline)

Fast up to about 10-12 moves deep, then it falls off a cliff. That cliff is what the ideas above attack. Measure it with `benchkit --depth N` for N = 8..14 and record in `docs/EXPERIMENTS.md`.

## Measured depth curve (2026-11-23)

Command for each depth (n=20, seed 1, 300 s deadline, v3 binary):
`target/release/benchkit --solver exa --n 20 --seed 1 --depth D --timeout-ms 300000 --json docs/data/exa-depth-D.json`,
followed by `scripts/bench_meta.py`. Raw JSON is committed in `docs/data/exa-depth-*.json`.
Hardware metadata is in each JSON; the CPU was pinned to `performance`, 3.4 GHz.

| depth | attempted | solved | p50 | p99 | mean solution length |
|-------|-----------|--------|-----|-----|----------------------|
| 8 | 20 | 20 | 70,350 ns | 382,166 ns | 8.00 |
| 9 | 20 | 20 | 421,863 ns | 4,333,489 ns | 9.00 |
| 10 | 20 | 20 | 4,355,446 ns | 11,059,659 ns | 10.00 |
| 11 | 20 | 20 | 51,537,915 ns | 237,088,908 ns | 11.00 |
| 12 | 20 | 20 | 499,405,563 ns | 923,991,882 ns | 12.00 |
| 13 | 20 | 20 | 6.53 s | 14.17 s | 13.00 |
| 14 | 6 | 6 | 37.75 s | 101.57 s | 14.00 |

Depth 14 hit the 300 s deadline after 6 of 20 solves; the JSON records `timed_out: true` and
`attempted: 6`. No failed or invalid solve occurred at any depth. The jump from depth 12 to 13
is about 13x at p50, and depth 14 is slower still: the baseline falls off a cliff exactly where
the docs predicted.

## Independent exact-length truth

`cargo test -p verify --release --test truth -- --ignored --nocapture` BFSes the full cube graph
to depth 6 (layer counts 1, 18, 243, 3,240, 43,239, 574,908, 7,618,438), then checks exa's
solution length against BFS distance on all 46,741 states through depth 4 plus 128 deterministic
samples at each of depths 5 and 6. Output: `checked 46997 states against BFS truth`, test passed
in 3.96 s. This validates exact lengths without using exa's heuristic as its own oracle.

## External optimal cross-check: unavailable

RubikOptimal 1.1.0 with PyPy 3.12 was installed successfully under `/tmp/opencode/rubikoptimal`
on 2026-11-23, but its ~963 MB table set could not be generated before the host/agent restarted
twice and cleared that tmpfs; available RAM also fell to about 1.2 GiB, below the roughly 2 GiB
the solver needs. **No external optimal lengths were produced and none are claimed.** The BFS
truth test above and exa's admissible lower-bound proof remain the correctness evidence; see
EXP-013 in `docs/EXPERIMENTS.md` for the full attempt record.
