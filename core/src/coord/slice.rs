//! UD-slice coordinate: the positions of edges 8..11 (C(12,4) values).
//!
//! Owns: `N_SLICE`, `SLICE_SOLVED`, `CubieCube::slice_pos`, `cubie_from_slice`.
//! Must NOT: use move tables, or encode orientation/permutation coordinates.

use super::cnk;
use crate::cubie::CubieCube;

pub const N_SLICE: usize = 495; // C(12,4)
pub const SLICE_SOLVED: usize = 494; // edges FR FL BL BR sit in slots 8..12

pub(crate) fn slice_from_ep(ep: &[u8; 12]) -> usize {
    let mut rank = 0usize;
    let mut k = 0usize;
    for pos in 0..12 {
        if ep[pos] >= 8 {
            rank += cnk(pos, k + 1);
            k += 1;
        }
    }
    rank
}

impl CubieCube {
    /// Combinadic rank of the slots that hold UD-slice edges (edge ids 8..12).
    pub fn slice_pos(&self) -> usize {
        slice_from_ep(&self.ep)
    }
}

pub fn cubie_from_slice(mut rank: usize) -> CubieCube {
    let mut c = CubieCube::SOLVED;
    let mut is_slice = [false; 12];
    for k in (1..=4).rev() {
        let mut p = 11usize;
        while cnk(p, k) > rank {
            p -= 1;
        }
        rank -= cnk(p, k);
        is_slice[p] = true;
    }
    let (mut s, mut o) = (8u8, 0u8);
    for pos in 0..12 {
        if is_slice[pos] {
            c.ep[pos] = s;
            s += 1;
        } else {
            c.ep[pos] = o;
            o += 1;
        }
    }
    c
}
