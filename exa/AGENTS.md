# exa: agent instructions (read the root `AGENTS.md` first)

**Objective:** fewest possible moves. Output must be provably HTM-optimal.

## Facts to keep straight

- God's number is 20 (HTM). exa can never need more than 20. The frontier is *how fast optimality can be proven*, not shrinking below 20.
- Average optimal length of a random state is about 17.7 moves.

## The reference oracle

`exa::Exa` (IDA* with three admissible projections) is the **oracle**. The search is in
`src/ida.rs`, pruning tables in `src/heuristic.rs`, CLI in `src/cli.rs`, and `src/main.rs`
is only an entry point. Every faster engine must, on a shared test set, return a solution
of exactly the oracle's length. Disagreement = the new engine is wrong (or, rarely, the
oracle is; investigate with a third opinion before touching it).

## Rules specific to exa

- A heuristic used for pruning must be provably admissible (never overestimates). Learned or empirical heuristics may only reorder search, never prune.
- Never report "optimal" without lower bound == upper bound. Emit both numbers.
- Any new engine is introduced as a separate module implementing `Solver`, benchmarked against the oracle on the same seeds.
- Respect the kill criteria in `docs/EXA.md`: if an idea fails them, log it and move on.

## Milestones: see `docs/ROADMAP.md` (M2, M7 to M9)
