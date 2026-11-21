//! IDA* baseline.
//!
//! Heuristic h(state) = max of three admissible lower bounds, each the exact distance to
//! solved in a *projection* of the cube (projections can only be easier than the real cube,
//! so they never overestimate):
//!   - (corner twist, UD-slice position)   2187 * 495 entries
//!   - (edge flip,    UD-slice position)   2048 * 495 entries
//!   - corner permutation                  40320 entries

use cubecore::coord::{N_CPERM, N_FLIP, N_SLICE, N_TWIST, SLICE_SOLVED};
use cubecore::solver::Solver;
use cubecore::tables::MoveTables;
use cubecore::{CubieCube, Move};

/// God's number in HTM. No solvable cube needs more.
const GODS_NUMBER: u8 = 20;

#[derive(Copy, Clone)]
struct Coords {
    t: usize,
    f: usize,
    s: usize,
    p: usize,
}

pub struct Exa {
    mt: MoveTables,
    p_ts: Vec<u8>,
    p_fs: Vec<u8>,
    p_cp: Vec<u8>,
    /// Nodes expanded by the last call to `solve` (for benchmarking and experiments).
    pub nodes: u64,
}

/// BFS over the product of two coordinates, starting from the solved pair.
fn build_prune(
    na: usize,
    ma: &[u16],
    nb: usize,
    mb: &[u16],
    goal_a: usize,
    goal_b: usize,
) -> Vec<u8> {
    let mut dist = vec![u8::MAX; na * nb];
    let start = goal_a * nb + goal_b;
    dist[start] = 0;
    let mut frontier = vec![start];
    let mut depth: u8 = 0;
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for &idx in &frontier {
            let (a, b) = (idx / nb, idx % nb);
            for m in 0..18 {
                let a2 = ma[a * 18 + m] as usize;
                let b2 = mb[b * 18 + m] as usize;
                let i2 = a2 * nb + b2;
                if dist[i2] == u8::MAX {
                    dist[i2] = depth + 1;
                    next.push(i2);
                }
            }
        }
        frontier = next;
        depth += 1;
    }
    dist
}

impl Exa {
    pub fn new() -> Self {
        let mt = MoveTables::new();
        let p_ts = build_prune(N_TWIST, &mt.twist, N_SLICE, &mt.slice, 0, SLICE_SOLVED);
        let p_fs = build_prune(N_FLIP, &mt.flip, N_SLICE, &mt.slice, 0, SLICE_SOLVED);
        let trivial = vec![0u16; 18];
        let p_cp = build_prune(N_CPERM, &mt.cperm, 1, &trivial, 0, 0);
        Exa {
            mt,
            p_ts,
            p_fs,
            p_cp,
            nodes: 0,
        }
    }

    #[inline]
    fn h(&self, c: &Coords) -> u8 {
        let a = self.p_ts[c.t * N_SLICE + c.s];
        let b = self.p_fs[c.f * N_SLICE + c.s];
        let d = self.p_cp[c.p];
        a.max(b).max(d)
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
        let h = self.h(&c);
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
        let mut limit = self.h(&c);
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
