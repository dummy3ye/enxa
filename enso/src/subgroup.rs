//! Subgroup predicates and the invariants the Variant A chain relies on.
//!
//! Owns: generator sets, tetrad/orbit definitions, membership tests and coordinate
//! ranking helpers shared by the stage modules.
//! Must NOT: build tables, run a descent, or perform I/O.

use cubecore::CubieCube;
use cubecore::{cnk, SLICE_SOLVED};

/// Tetrad preserved by the half-turn group (and its complement).
pub(crate) const TETRA: [u8; 4] = [0, 2, 5, 7];
/// Edge position orbit preserved by the half-turn group (and {1,3,5,7}).
pub(crate) const EORB: [u8; 4] = [0, 2, 4, 6];

pub(crate) const ALL: [u8; 18] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
/// <U,D,L,R,F2,B2> — preserves G1.
pub(crate) const G1_MOVES: [u8; 14] = [0, 1, 2, 3, 4, 5, 7, 9, 10, 11, 12, 13, 14, 16];
/// <U,D,L2,R2,F2,B2> — preserves G2.
pub(crate) const G2_MOVES: [u8; 10] = [0, 1, 2, 4, 7, 9, 10, 11, 13, 16];
/// <U2,D2,L2,R2,F2,B2> — the G3 subgroup itself.
pub(crate) const HALF: [u8; 6] = [1, 4, 7, 10, 13, 16];

pub(crate) const COORD3: usize = 70 * 70 * 6; // 29,400
pub(crate) const COORD4: usize = 96 * 6912; // 663,552
pub(crate) const EDGE_RANK: usize = 24 * 24 * 24; // 13,824

#[inline]
pub(crate) fn in_g1(c: &CubieCube) -> bool {
    c.eo.iter().all(|&x| x == 0)
}

#[inline]
pub(crate) fn in_g2(c: &CubieCube) -> bool {
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
pub(crate) fn tet_pattern(cp: &[u8; 8]) -> usize {
    rank_bits::<8>(|i| TETRA.contains(&cp[i]))
}

#[inline]
pub(crate) fn eor_pattern(ep: &[u8; 12]) -> usize {
    rank_bits::<8>(|i| EORB.contains(&ep[i]))
}

#[inline]
pub(crate) fn perm_rank8(p: &[u8]) -> usize {
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
pub(crate) fn edge_rank(ep: &[u8; 12]) -> usize {
    let (mut a, mut b, mut c) = ([0u8; 4], [0u8; 4], [0u8; 4]);
    for i in 0..4 {
        a[i] = ep[i * 2] / 2;
        b[i] = (ep[i * 2 + 1] - 1) / 2;
        c[i] = ep[8 + i] - 8;
    }
    rank4(&a) * 576 + rank4(&b) * 24 + rank4(&c)
}

/// Nth permutation of 8 items, in the same Lehmer order as `perm_rank8`.
pub(crate) fn nth_perm8(mut r: usize) -> [u8; 8] {
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

#[cfg(test)]
pub(crate) fn walk(
    c: &CubieCube,
    moves: &[u8],
    rng: &mut cubecore::Rng,
    steps: usize,
) -> CubieCube {
    let mut c = *c;
    for _ in 0..steps {
        c = c.apply(cubecore::Move(
            moves[rng.below(moves.len() as u64) as usize],
        ));
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use cubecore::Move;
    use cubecore::Rng;

    #[test]
    fn g1_is_a_subgroup_and_g1_moves_preserve_edge_orientation() {
        let mut rng = Rng::new(101);
        assert!(in_g1(&CubieCube::SOLVED));
        let mut c = CubieCube::SOLVED;
        for _ in 0..200 {
            c = walk(&c, &G1_MOVES, &mut rng, 20);
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
            c = walk(&c, &G2_MOVES, &mut rng, 20);
            assert!(in_g2(&c));
        }
        // L quarter turn is NOT allowed in G2: it ejects UD-slice edges from the slice.
        let l = CubieCube::SOLVED.apply(Move(12));
        assert!(!in_g2(&l), "L quarter turn must leave G2");
        let l2 = CubieCube::SOLVED.apply(Move(13));
        assert!(in_g2(&l2), "L2 must preserve G2");
    }
}
