//! IDA* search, the reference oracle for HTM-optimality.
//!
//! Heuristic h(state) = max of three admissible lower bounds, each the exact distance to
//! solved in a *projection* of the cube (projections can only be easier than the real cube,
//! so they never overestimate):
//!   - (corner twist, UD-slice position)   2187 * 495 entries
//!   - (edge flip,    UD-slice position)   2048 * 495 entries
//!   - corner permutation                  40320 entries
//!
//! Owns: iterative deepening and DFS. Must NOT: change the heuristic, the move order, or
//! report "optimal" without lower bound equal to the found upper bound.

use crate::heuristic::{Coords, Heuristics};
use cubecore::{CubieCube, Move, MoveTables, Solver};

/// God's number in HTM. No solvable cube needs more.
const GODS_NUMBER: u8 = 20;

pub struct Exa {
    mt: MoveTables,
    heuristics: Heuristics,
    /// Nodes expanded by the last call to `solve` (for benchmarking and experiments).
    pub nodes: u64,
}

impl Exa {
    pub fn new() -> Self {
        let mt = MoveTables::new();
        let heuristics = Heuristics::new(&mt);
        Exa {
            mt,
            heuristics,
            nodes: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs(
        &self,
        cube: &CubieCube,
        c: Coords,
        g: u8,
        limit: u8,
        last: Option<Move>,
        path: &mut Vec<Move>,
        nodes: &mut u64,
    ) -> bool {
        *nodes += 1;
        let h = self.heuristics.h(&c);
        if g + h > limit {
            return false;
        }
        if h == 0 && cube.is_solved() {
            return true;
        }
        if g == limit {
            return false;
        }
        for m in Move::all() {
            if !m.can_follow(last) {
                continue;
            }
            let mi = m.0 as usize;
            let nc = Coords {
                t: self.mt.twist[c.t * 18 + mi] as usize,
                f: self.mt.flip[c.f * 18 + mi] as usize,
                s: self.mt.slice[c.s * 18 + mi] as usize,
                p: self.mt.cperm[c.p * 18 + mi] as usize,
            };
            let next = cube.apply(m);
            path.push(m);
            if self.dfs(&next, nc, g + 1, limit, Some(m), path, nodes) {
                return true;
            }
            path.pop();
        }
        false
    }
}

impl Default for Exa {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver for Exa {
    fn name(&self) -> &str {
        "exa-ida-baseline"
    }

    fn solve(&mut self, cube: &CubieCube) -> Option<Vec<Move>> {
        if cube.verify().is_err() {
            return None;
        }
        let c = Coords {
            t: cube.twist(),
            f: cube.flip(),
            s: cube.slice_pos(),
            p: cube.cperm(),
        };
        let mut nodes = 0u64;
        let mut limit = self.heuristics.h(&c);
        let mut result = None;
        while limit <= GODS_NUMBER {
            let mut path = Vec::with_capacity(limit as usize);
            if self.dfs(cube, c, 0, limit, None, &mut path, &mut nodes) {
                result = Some(path);
                break;
            }
            limit += 1;
        }
        self.nodes = nodes;
        result
    }
}
