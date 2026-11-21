//! Stage G1→G2: solve corner orientation and put UD-slice edges in the slice.
//!
//! Owns: the (twist, slice) product coordinate, its goal, its <U,D,L,R,F2,B2> generators,
//! and the packing of the two base move tables. Must NOT: use L/R quarter turns or touch
//! other stages' coordinates.

use crate::subgroup::G1_MOVES;
use cubecore::CubieCube;
use cubecore::{N_SLICE, N_TWIST, SLICE_SOLVED};

pub(crate) const MOVES: [u8; 14] = G1_MOVES;
pub(crate) const N: usize = N_TWIST * N_SLICE;
pub(crate) const GOAL: usize = SLICE_SOLVED;

#[inline]
pub(crate) fn coord(c: &CubieCube) -> usize {
    c.twist() * N_SLICE + c.slice_pos()
}

/// Pack two independent move tables into one coordinate table.
pub(crate) fn combine(a: &[u16], b: &[u16], nb: usize) -> Vec<u32> {
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
