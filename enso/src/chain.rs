//! Building block for enso: a *descent stage*.
//!
//! Given a coordinate space, a move table and a set of goal coordinates, precompute (by BFS)
//! the distance to the goal and the best move for every coordinate. Solving the stage is then
//! pure table lookups: `x -> best[x] -> table[x*18 + best[x]] -> ...` until a goal is reached.
//! No search, no branching on the hot path. Chain several stages to solve a whole cube.
//!
//! Owns: the reusable stage BFS (`DescentStage`), its packed hot-path form (`PackedStage`)
//! and the table element abstraction.
//! Must NOT: define stages, build cube tables, or know about the CLI.

use cubecore::Move;

/// Element type of a move table. Implemented for `u16` (small coordinates) and `u32`
/// (coordinates larger than 65,535). Kept local so no third-party trait is needed.
pub trait TableIndex: Copy {
    fn index(self) -> usize;
}

impl TableIndex for u16 {
    #[inline]
    fn index(self) -> usize {
        self as usize
    }
}

impl TableIndex for u32 {
    #[inline]
    fn index(self) -> usize {
        self as usize
    }
}

const MOVE_SHIFT: u32 = 24;
const NEXT_MASK: u32 = 0x00ff_ffff;
const NO_MOVE: u8 = u8::MAX;

/// One-load descent table: high byte is the move, low 24 bits are the next coordinate.
/// Built from the same BFS as `DescentStage`, so it makes no independent correctness claim.
pub struct PackedStage {
    entry: Vec<u32>,
}

impl PackedStage {
    /// Build packed entries from a standard `n * 18` move table and BFS goals.
    pub fn build<T: TableIndex>(n: usize, allowed: &[Move], table: &[T], goals: &[usize]) -> Self {
        assert!(n <= 1 << 24, "coordinate does not fit in 24 bits");
        let stage = DescentStage::build(n, allowed, table, goals);
        let mut entry = vec![0u32; n];
        for x in 0..n {
            match stage.step(x, table) {
                Some((m, next)) => {
                    debug_assert!(next <= NEXT_MASK as usize);
                    entry[x] = ((m.0 as u32) << MOVE_SHIFT) | (next as u32 & NEXT_MASK);
                }
                None => entry[x] = (NO_MOVE as u32) << MOVE_SHIFT,
            }
        }
        PackedStage { entry }
    }

    /// One dependent load yields both the move and the next coordinate.
    #[inline]
    pub fn step(&self, x: usize) -> Option<(Move, usize)> {
        let e = *self.entry.get(x)?;
        let m = (e >> MOVE_SHIFT) as u8;
        if m == NO_MOVE {
            return None;
        }
        Some((Move(m), (e & NEXT_MASK) as usize))
    }
}

pub struct DescentStage {
    /// Kept for the BFS invariant check in this module's tests; not needed by descent.
    #[allow(dead_code)]
    dist: Vec<u8>,
    best: Vec<u8>,
}

impl DescentStage {
    /// `table` is a standard move table (`n * 18` entries). `allowed` must be closed under
    /// inversion (true for any generator set of the form "all powers of a set of faces"),
    /// which makes distance-from-goal equal distance-to-goal.
    pub fn build<T: TableIndex>(n: usize, allowed: &[Move], table: &[T], goals: &[usize]) -> Self {
        let mut dist = vec![u8::MAX; n];
        let mut frontier = Vec::new();
        for &g in goals {
            dist[g] = 0;
            frontier.push(g);
        }
        let mut d = 0u8;
        while !frontier.is_empty() {
            let mut next = Vec::new();
            for &x in &frontier {
                for m in allowed {
                    let y = table[x * 18 + m.0 as usize].index();
                    if dist[y] == u8::MAX {
                        dist[y] = d + 1;
                        next.push(y);
                    }
                }
            }
            frontier = next;
            d += 1;
        }
        let mut best = vec![u8::MAX; n];
        for x in 0..n {
            if dist[x] == 0 || dist[x] == u8::MAX {
                continue;
            }
            for m in allowed {
                let y = table[x * 18 + m.0 as usize].index();
                if dist[y] != u8::MAX && dist[y] as u16 + 1 == dist[x] as u16 {
                    best[x] = m.0;
                    break;
                }
            }
        }
        DescentStage { dist, best }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn dist(&self, x: usize) -> u8 {
        self.dist[x]
    }

    /// One descent step: the move to play and the next coordinate. `None` at a goal or if unreachable.
    #[inline]
    pub fn step<T: TableIndex>(&self, x: usize, table: &[T]) -> Option<(Move, usize)> {
        let b = self.best[x];
        if b == u8::MAX {
            return None;
        }
        Some((Move(b), table[x * 18 + b as usize].index()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cubecore::{MoveTables, N_TWIST};

    #[test]
    fn packed_stage_matches_descent_stage() {
        let mt = MoveTables::new();
        let all: Vec<Move> = Move::all().collect();
        let plain = DescentStage::build(N_TWIST, &all, &mt.twist, &[0]);
        let packed = PackedStage::build(N_TWIST, &all, &mt.twist, &[0]);
        for x in 0..N_TWIST {
            let a = plain.step(x, &mt.twist).map(|(m, n)| (m.0, n));
            let b = packed.step(x).map(|(m, n)| (m.0, n));
            assert_eq!(a, b, "packed step differs at {x}");
        }
    }

    #[test]
    fn twist_descent_reaches_goal_in_dist_steps() {
        let mt = MoveTables::new();
        let all: Vec<Move> = Move::all().collect();
        let st = DescentStage::build(N_TWIST, &all, &mt.twist, &[0]);
        for x in 0..N_TWIST {
            assert_ne!(st.dist(x), u8::MAX, "coordinate {x} unreachable");
            let mut cur = x;
            let mut steps = 0u8;
            while cur != 0 {
                let (_, nx) = st.step(cur, &mt.twist).expect("stuck before goal");
                cur = nx;
                steps += 1;
                assert!(steps <= 20);
            }
            assert_eq!(steps, st.dist(x));
        }
    }
}
