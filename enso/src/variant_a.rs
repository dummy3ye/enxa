//! enso Variant A: a four-stage Thistlethwaite-style chain solved by pure table descent.
//!
//! G0 --(all 18 moves)--> G1: edge orientation solved
//! G1 --<U,D,L,R,F2,B2>--> G2: edge + corner orientation solved, UD-slice edges in slice
//! G2 --<U,D,L2,R2,F2,B2>--> G3: paired corner/edge coset solved
//! G3 --<U2,D2,L2,R2,F2,B2>--> solved
//!
//! The G2→G3 generator set is *not* <U,D,L,R,F2,B2>: L/R quarter turns move UD-slice
//! edges out of the slice and would destroy the G2 invariant. This was derived and
//! verified independently before implementation (see docs/EXPERIMENTS.md, EXP-001).
//!
//! All solve-path work is table lookups into fixed allocations owned by `Enso`. The only
//! allocation in `Solver::solve` is the `Vec<Move>` the trait signature requires; the
//! internal descent writes into a caller-supplied fixed-size buffer.

use cubecore::coord::{cnk, cubie_from_cperm, N_CPERM, N_FLIP, N_SLICE, N_TWIST, SLICE_SOLVED};
use cubecore::cubie::{perm_parity, MOVES};
use cubecore::solver::Solver;
use cubecore::tables::MoveTables;
use cubecore::{CubieCube, Move};
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::chain::{DescentStage, TableIndex};

/// Upper bound on emitted moves. Every stage's BFS depth is far below this.
pub const MAX_SOLUTION_LEN: usize = 128;

/// Tetrad preserved by the half-turn group (and its complement).
pub const TETRA: [u8; 4] = [0, 2, 5, 7];
/// Edge position orbit preserved by the half-turn group (and {1,3,5,7}).
pub const EORB: [u8; 4] = [0, 2, 4, 6];

const ALL: [u8; 18] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
/// <U,D,L,R,F2,B2> — preserves G1.
const G1_MOVES: [u8; 14] = [0, 1, 2, 3, 4, 5, 7, 9, 10, 11, 12, 13, 14, 16];
/// <U,D,L2,R2,F2,B2> — preserves G2.
const G2_MOVES: [u8; 10] = [0, 1, 2, 4, 7, 9, 10, 11, 13, 16];
/// <U2,D2,L2,R2,F2,B2> — the G3 subgroup itself.
const HALF: [u8; 6] = [1, 4, 7, 10, 13, 16];

const COORD3: usize = 70 * 70 * 6; // 29,400
const COORD4: usize = 96 * 6912; // 663,552
const EDGE_RANK: usize = 24 * 24 * 24; // 13,824

// ---------------------------------------------------------------------------
// Subgroup predicates (used by tests and by debug assertions in the solve path)
// ---------------------------------------------------------------------------

#[inline]
pub fn in_g1(c: &CubieCube) -> bool {
    c.eo.iter().all(|&x| x == 0)
}

#[inline]
pub fn in_g2(c: &CubieCube) -> bool {
    in_g1(c) && c.co.iter().all(|&x| x == 0) && c.slice_pos() == SLICE_SOLVED
}

#[inline]
fn rank_bits<const N: usize>(f: impl Fn(usize) -> bool) -> usize {
    let (mut rank, mut k) = (0usize, 0usize);
    for pos in 0..N {
        if f(pos) {
            rank += cnk(pos, k + 1);
            k += 1;
        }
    }
    rank
}

#[inline]
pub fn tet_pattern(cp: &[u8; 8]) -> usize {
    rank_bits::<8>(|i| TETRA.contains(&cp[i]))
}

#[inline]
pub fn eor_pattern(ep: &[u8; 12]) -> usize {
    rank_bits::<8>(|i| EORB.contains(&ep[i]))
}

#[inline]
pub fn perm_rank8(p: &[u8]) -> usize {
    let (mut r, mut fact) = (0usize, 1usize);
    for i in 1..p.len() {
        let mut s = 0usize;
        for j in 0..i {
            if p[j] > p[i] {
                s += 1;
            }
        }
        r += s * fact;
        fact *= i + 1;
    }
    r
}

#[inline]
fn rank4(p: &[u8; 4]) -> usize {
    let (mut r, mut f) = (0usize, 1usize);
    for i in 1..4 {
        let mut s = 0usize;
        for j in 0..i {
            if p[j] > p[i] {
                s += 1;
            }
        }
        r += s * f;
        f *= i + 1;
    }
    r
}

/// Compact index of an edge permutation inside the half-turn group's three position orbits.
#[inline]
pub fn edge_rank(ep: &[u8; 12]) -> usize {
    let (mut a, mut b, mut c) = ([0u8; 4], [0u8; 4], [0u8; 4]);
    for i in 0..4 {
        a[i] = ep[i * 2] / 2;
        b[i] = (ep[i * 2 + 1] - 1) / 2;
        c[i] = ep[8 + i] - 8;
    }
    rank4(&a) * 576 + rank4(&b) * 24 + rank4(&c)
}

fn nth_perm8(mut r: usize) -> [u8; 8] {
    let mut inv = [0usize; 8];
    for (i, slot) in inv.iter_mut().enumerate().skip(1) {
        *slot = r % (i + 1);
        r /= i + 1;
    }
    let mut avail: Vec<u8> = (0..8).collect();
    let mut out = [0u8; 8];
    for i in (0..8).rev() {
        out[i] = avail.remove(i - inv[i]);
    }
    out
}

// ---------------------------------------------------------------------------
// Precomputed tables (startup only; never touched by the hot path except reads)
// ---------------------------------------------------------------------------

pub struct Tables {
    st1: DescentStage,
    t1: Vec<u16>,
    st2: DescentStage,
    t2: Vec<u32>,
    st3: DescentStage,
    t3: Vec<u16>,
    st4: DescentStage,
    t4: Vec<u32>,
    corner_class: Vec<u8>,
    cp_idx: Vec<u16>,
    ep_idx: Vec<u16>,
    goal3: usize,
    pub build_time: Duration,
}

impl Tables {
    /// True iff the cube is in the half-turn subgroup G3 (exact membership test).
    pub fn in_g3(&self, c: &CubieCube) -> bool {
        in_g2(c) && self.cp_idx[c.cperm()] != u16::MAX && self.ep_idx[edge_rank(&c.ep)] != u16::MAX
    }

    pub fn build() -> Self {
        let t0 = Instant::now();
        let all: Vec<Move> = ALL.iter().map(|&m| Move(m)).collect();
        let g1: Vec<Move> = G1_MOVES.iter().map(|&m| Move(m)).collect();
        let g2: Vec<Move> = G2_MOVES.iter().map(|&m| Move(m)).collect();
        let half: Vec<Move> = HALF.iter().map(|&m| Move(m)).collect();

        // Stage 1: flip coordinate.
        let mt = MoveTables::new();
        let st1 = DescentStage::build(N_FLIP, &all, &mt.flip, &[0]);

        // Stage 2: (twist, slice) product coordinate under G1 moves.
        let t2 = combine(&mt.twist, &mt.slice, N_SLICE);
        let st2 = DescentStage::build(N_TWIST * N_SLICE, &g1, &t2, &[SLICE_SOLVED]);

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
        let goal3 = encode_g2(&CubieCube::SOLVED, &corner_class);
        let mut t3 = vec![0u16; COORD3 * 18];
        for x in 0..COORD3 {
            let c = decode_g2(x, &rep_cp, &rep_ep);
            for &m in &G2_MOVES {
                let n = c.apply(Move(m));
                t3[x * 18 + m as usize] = encode_g2(&n, &corner_class) as u16;
            }
        }
        let st3 = DescentStage::build(COORD3, &g2, &t3, &[goal3]);

        // Stage 4: full BFS of the 663,552-state half-turn subgroup.
        let mut t4 = vec![0u32; COORD4 * 18];
        for x in 0..COORD4 {
            let c = CubieCube {
                cp: hcp[x / 6912],
                co: [0; 8],
                ep: hep[x % 6912],
                eo: [0; 12],
            };
            for &m in &HALF {
                let n = c.apply(Move(m));
                let ci = cp_idx[n.cperm()];
                let ei = ep_idx[edge_rank(&n.ep)];
                debug_assert!(ci != u16::MAX && ei != u16::MAX);
                t4[x * 18 + m as usize] = (ci as usize * 6912 + ei as usize) as u32;
            }
        }
        let st4 = DescentStage::build(COORD4, &half, &t4, &[0]);

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

/// Pack two independent move tables into one coordinate table.
fn combine(a: &[u16], b: &[u16], nb: usize) -> Vec<u32> {
    let na = a.len() / 18;
    let mut t = vec![0u32; na * nb * 18];
    for x in 0..na {
        for y in 0..nb {
            for m in 0..18 {
                t[(x * nb + y) * 18 + m] = a[x * 18 + m] as u32 * nb as u32 + b[y * 18 + m] as u32;
            }
        }
    }
    t
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

#[inline]
fn encode_g2(c: &CubieCube, corner_class: &[u8]) -> usize {
    (tet_pattern(&c.cp) * 70 + eor_pattern(&c.ep)) * 6 + corner_class[c.cperm()] as usize
}

#[inline]
fn decode_g2(x: usize, rep_cp: &[[u8; 8]], rep_ep: &[[u8; 12]]) -> CubieCube {
    let (tet, eor, cls) = (x / 420, (x / 6) % 70, x % 6);
    let cp = rep_cp[tet * 6 + cls];
    let ep = rep_ep[eor * 2 + perm_parity(&cp) as usize];
    CubieCube {
        cp,
        co: [0; 8],
        ep,
        eo: [0; 12],
    }
}

// ---------------------------------------------------------------------------
// The solver
// ---------------------------------------------------------------------------

fn descend<T: TableIndex>(
    st: &DescentStage,
    table: &[T],
    mut x: usize,
    goal: usize,
    c: &mut CubieCube,
    out: &mut [u8; MAX_SOLUTION_LEN],
    n: &mut usize,
) -> Result<(), &'static str> {
    while x != goal {
        let (m, nx) = st.step(x, table).ok_or("stuck before stage goal")?;
        if *n >= out.len() {
            return Err("solution buffer overflow");
        }
        out[*n] = m.0;
        *n += 1;
        *c = c.apply(m);
        x = nx;
    }
    Ok(())
}

pub struct Enso {
    tables: Tables,
}

impl Enso {
    /// Builds all stage tables. This is startup cost, reported separately from solve latency.
    pub fn new() -> Self {
        Enso {
            tables: Tables::build(),
        }
    }

    pub fn table_build_time(&self) -> Duration {
        self.tables.build_time
    }

    /// Allocation-free descent into a caller-provided fixed-size buffer.
    pub fn solve_into(
        &self,
        cube: &CubieCube,
        out: &mut [u8; MAX_SOLUTION_LEN],
    ) -> Result<usize, &'static str> {
        if cube.verify().is_err() {
            return Err("invalid cube");
        }
        let t = &self.tables;
        let mut c = *cube;
        let mut n = 0usize;

        descend(&t.st1, &t.t1, c.flip(), 0, &mut c, out, &mut n)?;
        debug_assert!(in_g1(&c));

        descend(
            &t.st2,
            &t.t2,
            c.twist() * N_SLICE + c.slice_pos(),
            SLICE_SOLVED,
            &mut c,
            out,
            &mut n,
        )?;
        debug_assert!(in_g2(&c));

        descend(
            &t.st3,
            &t.t3,
            encode_g2(&c, &t.corner_class),
            t.goal3,
            &mut c,
            out,
            &mut n,
        )?;
        debug_assert!(t.in_g3(&c));

        // Guard before indexing the 663,552-entry table: a bad stage-3 outcome must
        // return an error, never panic on cube data.
        let ci = t.cp_idx[c.cperm()];
        let ei = t.ep_idx[edge_rank(&c.ep)];
        if ci == u16::MAX || ei == u16::MAX {
            return Err("stage 3 did not reach G3");
        }
        descend(
            &t.st4,
            &t.t4,
            ci as usize * 6912 + ei as usize,
            0,
            &mut c,
            out,
            &mut n,
        )?;

        if !c.is_solved() {
            return Err("descent finished on an unsolved cube");
        }
        Ok(n)
    }
}

impl Default for Enso {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver for Enso {
    fn name(&self) -> &str {
        "enso-variant-a"
    }

    fn solve(&mut self, cube: &CubieCube) -> Option<Vec<Move>> {
        let mut buf = [0u8; MAX_SOLUTION_LEN];
        let n = self.solve_into(cube, &mut buf).ok()?;
        Some(buf[..n].iter().map(|&m| Move(m)).collect())
    }
}

// ---------------------------------------------------------------------------
// Tests: subgroup definitions, preservation, coordinate consistency, end to end
// ---------------------------------------------------------------------------

impl Tables {
    /// Test-only: run the chain while asserting each subgroup invariant.
    #[cfg(test)]
    fn solve_checkpoints(&self, cube: &CubieCube, out: &mut [u8; MAX_SOLUTION_LEN]) -> usize {
        let mut c = *cube;
        let mut n = 0;
        descend(&self.st1, &self.t1, c.flip(), 0, &mut c, out, &mut n).unwrap();
        assert!(in_g1(&c));
        descend(
            &self.st2,
            &self.t2,
            c.twist() * N_SLICE + c.slice_pos(),
            SLICE_SOLVED,
            &mut c,
            out,
            &mut n,
        )
        .unwrap();
        assert!(in_g2(&c));
        descend(
            &self.st3,
            &self.t3,
            encode_g2(&c, &self.corner_class),
            self.goal3,
            &mut c,
            out,
            &mut n,
        )
        .unwrap();
        assert!(self.in_g3(&c));
        descend(
            &self.st4,
            &self.t4,
            self.cp_idx[c.cperm()] as usize * 6912 + self.ep_idx[edge_rank(&c.ep)] as usize,
            0,
            &mut c,
            out,
            &mut n,
        )
        .unwrap();
        assert!(c.is_solved());
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cubecore::moves::random_alg;
    use cubecore::rng::Rng;
    use std::sync::OnceLock;

    static TABLES: OnceLock<Tables> = OnceLock::new();
    fn tables() -> &'static Tables {
        TABLES.get_or_init(Tables::build)
    }

    fn in_group(c: &CubieCube, moves: &[u8], rng: &mut Rng, steps: usize) -> CubieCube {
        let mut c = *c;
        for _ in 0..steps {
            c = c.apply(Move(moves[rng.below(moves.len() as u64) as usize]));
        }
        c
    }

    #[test]
    fn g1_is_a_subgroup_and_g1_moves_preserve_edge_orientation() {
        let mut rng = Rng::new(101);
        assert!(in_g1(&CubieCube::SOLVED));
        let mut c = CubieCube::SOLVED;
        for _ in 0..200 {
            c = in_group(&c, &G1_MOVES, &mut rng, 20);
            assert!(in_g1(&c));
            assert!(c.verify().is_ok());
        }
        // A quarter turn of F leaves G1, which is why G1 excludes F/F'/B/B'.
        assert!(!in_g1(&CubieCube::SOLVED.apply(Move(6))));
    }

    #[test]
    fn g2_is_a_subgroup_and_g2_moves_preserve_its_invariants() {
        let mut rng = Rng::new(102);
        assert!(in_g2(&CubieCube::SOLVED));
        let mut c = CubieCube::SOLVED;
        for _ in 0..200 {
            c = in_group(&c, &G2_MOVES, &mut rng, 20);
            assert!(in_g2(&c));
        }
        // L quarter turn is NOT allowed in G2: it ejects UD-slice edges from the slice.
        let l = CubieCube::SOLVED.apply(Move(12));
        assert!(!in_g2(&l), "L quarter turn must leave G2");
        let l2 = CubieCube::SOLVED.apply(Move(13));
        assert!(in_g2(&l2), "L2 must preserve G2");
    }

    #[test]
    fn half_turns_preserve_g3_and_membership_is_exact() {
        let t = tables();
        let mut rng = Rng::new(103);
        assert!(t.in_g3(&CubieCube::SOLVED));
        let mut c = CubieCube::SOLVED;
        for _ in 0..500 {
            c = in_group(&c, &HALF, &mut rng, 15);
            assert!(t.in_g3(&c));
        }
        assert_eq!(hcp_len(), 96);
        assert_eq!(hep_len(), 6912);
        // A state outside G2 cannot be in G3.
        assert!(!t.in_g3(&CubieCube::SOLVED.apply(Move(6))));
    }

    fn hcp_len() -> usize {
        build_hcp().len()
    }
    fn hep_len() -> usize {
        build_hep().len()
    }

    #[test]
    fn stage3_move_table_matches_recomputed_coordinate() {
        let t = tables();
        let mut rng = Rng::new(104);
        let all: Vec<Move> = ALL.iter().map(|&m| Move(m)).collect();
        for _ in 0..2000 {
            let mut c = CubieCube::SOLVED;
            for _ in 0..60 {
                c = c.apply(Move(G1_MOVES[rng.below(14) as usize]));
            }
            if !in_g2(&c) {
                continue;
            }
            let x = encode_g2(&c, &t.corner_class);
            for m in all.iter().copied().filter(|m| G2_MOVES.contains(&m.0)) {
                let n = c.apply(m);
                assert_eq!(
                    t.t3[x * 18 + m.0 as usize] as usize,
                    encode_g2(&n, &t.corner_class)
                );
            }
        }
    }

    #[test]
    fn each_stage_lands_in_its_subgroup_and_the_cube_is_solved() {
        let t = tables();
        let mut rng = Rng::new(105);
        for _ in 0..2000 {
            let alg = random_alg(&mut rng, 40);
            let cube = CubieCube::SOLVED.apply_all(&alg);
            let mut buf = [0u8; MAX_SOLUTION_LEN];
            let n = t.solve_checkpoints(&cube, &mut buf);
            assert!(cube
                .apply_all(&buf[..n].iter().map(|&m| Move(m)).collect::<Vec<_>>())
                .is_solved());
        }
    }
}
