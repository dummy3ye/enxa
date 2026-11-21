//! Table construction for all four Variant A stages.
//!
//! Owns: the `Tables` container, startup BFS/table generation, and exact G3 membership.
//! Must NOT: run a solve, emit moves, or perform I/O. The hot path only reads these tables.

use crate::chain::DescentStage;
use crate::stages::{orient, r#final, slicecorner, tetrad};
use crate::subgroup::{
    edge_rank, eor_pattern, in_g2, nth_perm8, perm_rank8, tet_pattern, EDGE_RANK, HALF,
};
use cubecore::{
    cubie_from_cperm, perm_parity, CubieCube, Move, MoveTables, MOVES, N_CPERM, N_SLICE,
};
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub(crate) struct Tables {
    pub(crate) st1: DescentStage,
    pub(crate) t1: Vec<u16>,
    pub(crate) st2: DescentStage,
    pub(crate) t2: Vec<u32>,
    pub(crate) st3: DescentStage,
    pub(crate) t3: Vec<u16>,
    pub(crate) st4: DescentStage,
    pub(crate) t4: Vec<u32>,
    pub(crate) corner_class: Vec<u8>,
    pub(crate) cp_idx: Vec<u16>,
    pub(crate) ep_idx: Vec<u16>,
    pub(crate) goal3: usize,
    pub(crate) build_time: Duration,
}

impl Tables {
    /// True iff the cube is in the half-turn subgroup G3 (exact membership test).
    pub(crate) fn in_g3(&self, c: &CubieCube) -> bool {
        in_g2(c) && self.cp_idx[c.cperm()] != u16::MAX && self.ep_idx[edge_rank(&c.ep)] != u16::MAX
    }

    pub(crate) fn build() -> Self {
        let t0 = Instant::now();
        let all: Vec<Move> = orient::MOVES.iter().map(|&m| Move(m)).collect();
        let g1: Vec<Move> = slicecorner::MOVES.iter().map(|&m| Move(m)).collect();
        let g2: Vec<Move> = tetrad::MOVES.iter().map(|&m| Move(m)).collect();
        let half: Vec<Move> = r#final::MOVES.iter().map(|&m| Move(m)).collect();

        // Stage 1: flip coordinate.
        let mt = MoveTables::new();
        let st1 = DescentStage::build(orient::N, &all, &mt.flip, &[orient::GOAL]);

        // Stage 2: (twist, slice) product coordinate under G1 moves.
        let t2 = slicecorner::combine(&mt.twist, &mt.slice, N_SLICE);
        let st2 = DescentStage::build(slicecorner::N, &g1, &t2, &[slicecorner::GOAL]);

        // Group building blocks shared by stages 3 and 4.
        let hcp = build_hcp();
        let hep = build_hep();
        let (corner_class, rep_cp, rep_ep) = build_g2_reps(&hcp);
        let mut cp_idx = vec![u16::MAX; N_CPERM];
        for (i, &p) in hcp.iter().enumerate() {
            cp_idx[perm_rank8(&p)] = i as u16;
        }
        let mut ep_idx = vec![u16::MAX; EDGE_RANK];
        for (i, &e) in hep.iter().enumerate() {
            ep_idx[edge_rank(&e)] = i as u16;
        }

        // Stage 3: 29,400 G2/G3 cosets.
        let goal3 = tetrad::goal(&corner_class);
        let mut t3 = vec![0u16; tetrad::N * 18];
        for x in 0..tetrad::N {
            let c = tetrad::decode_g2(x, &rep_cp, &rep_ep);
            for &m in &tetrad::MOVES {
                let n = c.apply(Move(m));
                t3[x * 18 + m as usize] = tetrad::encode_g2(&n, &corner_class) as u16;
            }
        }
        let st3 = DescentStage::build(tetrad::N, &g2, &t3, &[goal3]);

        // Stage 4: full BFS of the 663,552-state half-turn subgroup.
        let mut t4 = vec![0u32; r#final::N * 18];
        for x in 0..r#final::N {
            let c = CubieCube {
                cp: hcp[x / r#final::NEP],
                co: [0; 8],
                ep: hep[x % r#final::NEP],
                eo: [0; 12],
            };
            for &m in &r#final::MOVES {
                let n = c.apply(Move(m));
                let ci = cp_idx[n.cperm()];
                let ei = ep_idx[edge_rank(&n.ep)];
                debug_assert!(ci != u16::MAX && ei != u16::MAX);
                t4[x * 18 + m as usize] = (ci as usize * r#final::NEP + ei as usize) as u32;
            }
        }
        let st4 = DescentStage::build(r#final::N, &half, &t4, &[r#final::GOAL]);

        Tables {
            st1,
            t1: mt.flip,
            st2,
            t2,
            st3,
            t3,
            st4,
            t4,
            corner_class,
            cp_idx,
            ep_idx,
            goal3,
            build_time: t0.elapsed(),
        }
    }
}

/// Corner permutations of the half-turn group, indexed 0..95 (identity = 0).
fn build_hcp() -> Vec<[u8; 8]> {
    let mut seen: HashSet<[u8; 8]> = HashSet::new();
    let mut list: Vec<[u8; 8]> = Vec::new();
    let id = [0u8, 1, 2, 3, 4, 5, 6, 7];
    seen.insert(id);
    list.push(id);
    let mut i = 0;
    while i < list.len() {
        let cur = list[i];
        i += 1;
        for &m in &HALF {
            let mv = MOVES[m as usize].cp;
            let mut n = [0u8; 8];
            for p in 0..8 {
                n[p] = cur[mv[p] as usize];
            }
            if seen.insert(n) {
                list.push(n);
            }
        }
    }
    assert_eq!(list.len(), 96);
    list
}

/// Edge permutations of the half-turn group, indexed 0..6911 (identity = 0).
fn build_hep() -> Vec<[u8; 12]> {
    let mut seen: HashSet<[u8; 12]> = HashSet::new();
    let mut list: Vec<[u8; 12]> = Vec::new();
    let mut id = [0u8; 12];
    for i in 0..12 {
        id[i] = i as u8;
    }
    seen.insert(id);
    list.push(id);
    let mut i = 0;
    while i < list.len() {
        let cur = list[i];
        i += 1;
        for &m in &HALF {
            let mv = MOVES[m as usize].ep;
            let mut n = [0u8; 12];
            for p in 0..12 {
                n[p] = cur[mv[p] as usize];
            }
            if seen.insert(n) {
                list.push(n);
            }
        }
    }
    assert_eq!(list.len(), 6912);
    list
}

/// Left-coset classes of corner permutations under G3, re-indexed per tetrad pattern,
/// plus one representative cube per (tetrad, class) and (edge orbit, parity) pair.
fn build_g2_reps(hcp: &[[u8; 8]]) -> (Vec<u8>, Vec<[u8; 8]>, Vec<[u8; 12]>) {
    let mut global = vec![u16::MAX; N_CPERM];
    let mut tet_of = vec![0usize; N_CPERM];
    let mut gid = 0u16;
    for r in 0..N_CPERM {
        if global[r] != u16::MAX {
            continue;
        }
        let cp = cubie_from_cperm(r).cp;
        let tet = tet_pattern(&cp);
        let mut queue = vec![cp];
        let mut i = 0;
        while i < queue.len() {
            let cur = queue[i];
            i += 1;
            let ri = perm_rank8(&cur);
            if global[ri] != u16::MAX {
                continue;
            }
            global[ri] = gid;
            tet_of[ri] = tet;
            for h in hcp {
                let mut n = [0u8; 8];
                for p in 0..8 {
                    n[p] = h[cur[p] as usize];
                }
                queue.push(n);
            }
        }
        gid += 1;
    }
    assert_eq!(gid, 420);

    let mut sets: Vec<Vec<u16>> = vec![Vec::new(); 70];
    for r in 0..N_CPERM {
        sets[tet_of[r]].push(global[r]);
    }
    let mut local = vec![255u8; 70 * gid as usize];
    for (tet, set) in sets.iter_mut().enumerate() {
        set.sort_unstable();
        set.dedup();
        assert_eq!(set.len(), 6, "tetrad pattern {tet}");
        for (i, &g) in set.iter().enumerate() {
            local[tet * gid as usize + g as usize] = i as u8;
        }
    }
    let mut corner_class = vec![0u8; N_CPERM];
    for r in 0..N_CPERM {
        corner_class[r] = local[tet_of[r] * gid as usize + global[r] as usize];
    }

    let mut rep_cp = [[0u8; 8]; 420];
    let mut seen_cp = [false; 420];
    for r in 0..N_CPERM {
        let cp = cubie_from_cperm(r).cp;
        let k = tet_pattern(&cp) * 6 + corner_class[r] as usize;
        if !seen_cp[k] {
            rep_cp[k] = cp;
            seen_cp[k] = true;
        }
    }
    assert!(seen_cp.iter().all(|&x| x));

    let mut rep_ep = [[0u8; 12]; 140];
    let mut seen_ep = [false; 140];
    for r in 0..40320 {
        let p = nth_perm8(r);
        let mut ep = [0u8; 12];
        ep[..8].copy_from_slice(&p);
        ep[8..].copy_from_slice(&[8, 9, 10, 11]);
        let par = perm_parity(&ep) as usize;
        let k = eor_pattern(&ep) * 2 + par;
        if !seen_ep[k] {
            rep_ep[k] = ep;
            seen_ep[k] = true;
        }
    }
    assert!(seen_ep.iter().all(|&x| x));

    (corner_class, rep_cp.to_vec(), rep_ep.to_vec())
}

#[cfg(test)]
pub(crate) fn tables() -> &'static Tables {
    static TABLES: std::sync::OnceLock<Tables> = std::sync::OnceLock::new();
    TABLES.get_or_init(Tables::build)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subgroup::walk;
    use cubecore::Rng;

    #[test]
    fn half_turns_preserve_g3_and_membership_is_exact() {
        let t = tables();
        let mut rng = Rng::new(103);
        assert!(t.in_g3(&CubieCube::SOLVED));
        let mut c = CubieCube::SOLVED;
        for _ in 0..500 {
            c = walk(&c, &HALF, &mut rng, 15);
            assert!(t.in_g3(&c));
        }
        assert_eq!(build_hcp().len(), 96);
        assert_eq!(build_hep().len(), 6912);
        // A state outside G2 cannot be in G3.
        assert!(!t.in_g3(&CubieCube::SOLVED.apply(Move(6))));
    }
}
