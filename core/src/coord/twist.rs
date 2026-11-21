//! Corner twist coordinate (3^7 values) and its decomposition.
//!
//! Owns: `N_TWIST`, `CubieCube::twist`, `cubie_from_twist`.
//! Must NOT: encode flips, slices, permutations, or depend on move tables.

use crate::cubie::CubieCube;

pub const N_TWIST: usize = 2187; // 3^7

pub(crate) fn twist_from_co(co: &[u8; 8]) -> usize {
    let mut t = 0usize;
    for i in 0..7 {
        t = t * 3 + co[i] as usize;
    }
    t
}

impl CubieCube {
    pub fn twist(&self) -> usize {
        twist_from_co(&self.co)
    }
}

pub fn cubie_from_twist(mut t: usize) -> CubieCube {
    let mut c = CubieCube::SOLVED;
    let mut sum = 0u8;
    for i in (0..7).rev() {
        let d = (t % 3) as u8;
        c.co[i] = d;
        sum += d;
        t /= 3;
    }
    c.co[7] = (3 - sum % 3) % 3;
    c
}
