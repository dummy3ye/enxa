//! Admissible pruning tables used by the IDA* search.
//!
//! Owns: the four-coordinate snapshot, the three projection tables, and `h()`.
//! Must NOT: search, mutate state, or use a non-admissible heuristic to prune.

use cubecore::{MoveTables, N_CPERM, N_FLIP, N_SLICE, N_TWIST, SLICE_SOLVED};

/// The four coordinate values that determine the search state.
#[derive(Copy, Clone)]
pub(crate) struct Coords {
    pub(crate) t: usize,
    pub(crate) f: usize,
    pub(crate) s: usize,
    pub(crate) p: usize,
}

pub(crate) struct Heuristics {
    p_ts: Vec<u8>,
    p_fs: Vec<u8>,
    p_cp: Vec<u8>,
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

impl Heuristics {
    pub(crate) fn new(mt: &MoveTables) -> Self {
        let p_ts = build_prune(N_TWIST, &mt.twist, N_SLICE, &mt.slice, 0, SLICE_SOLVED);
        let p_fs = build_prune(N_FLIP, &mt.flip, N_SLICE, &mt.slice, 0, SLICE_SOLVED);
        let trivial = vec![0u16; 18];
        let p_cp = build_prune(N_CPERM, &mt.cperm, 1, &trivial, 0, 0);
        Heuristics { p_ts, p_fs, p_cp }
    }

    /// Exact distance to solved in each projection; the maximum is a lower bound for the
    /// full cube because a projection can only make a state easier, never harder.
    #[inline]
    pub(crate) fn h(&self, c: &Coords) -> u8 {
        let a = self.p_ts[c.t * N_SLICE + c.s];
        let b = self.p_fs[c.f * N_SLICE + c.s];
        let d = self.p_cp[c.p];
        a.max(b).max(d)
    }
}
