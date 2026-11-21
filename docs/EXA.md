# exa: design

## Goal

The fewest possible moves, **provably**. HTM metric.

## Reality check (so effort goes to the right place)

- God's number is 20 in HTM (proven 2010). exa can never need more.
- A random state needs about 17.7 moves on average; the vast majority need 16-19.
- So "fewer moves than ever" is bounded. The open frontier is **how cheaply optimality can be proven**.

## Baseline (M2, written): IDA* with admissible projections

`exa::ida::Exa`. Heuristic = max of exact distances in three projections: (twist, slice), (flip, slice), (corner perm). Exact, simple, correct, slow on deep states. It is the oracle every other engine is tested against.

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
