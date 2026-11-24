# Documentation index

Start with `README.md` at the repository root and `AGENTS.md` if you are an agent.

| Document | Purpose |
|----------|---------|
| `ARCHITECTURE.md` | current file layout, module ownership and public APIs |
| `DESIGN.md` | load-bearing cube conventions; changing them changes every table and test |
| `ENSO.md` | enso goal, Variant A chain and tuning ideas |
| `EXA.md` | exa optimality goal, IDA* oracle and future engines |
| `BENCHMARKING.md` | rules for honest, reproducible measurements |
| `EXPERIMENTS.md` | hypotheses, results and decisions, newest first |
| `ROADMAP.md` | milestone status |
| `data/` | committed raw benchmark JSON and the fixed 100k cube file |
| `assets/` | generated light/dark SVG charts; regenerate with `python3 scripts/assets.py` |

`ENSO_EXA_INIT.md` at the repository root is a historical extraction snapshot. Do not run
it; it would restore the pre-refactor layout. Current layout is `ARCHITECTURE.md`.
