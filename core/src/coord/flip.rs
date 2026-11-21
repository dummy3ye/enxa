//! Edge flip coordinate (2^11 values) and its decomposition.
//!
//! Owns: `N_FLIP`, `CubieCube::flip`, `cubie_from_flip`.
//! Must NOT: encode twists, slices, permutations, or depend on move tables.

use crate::cubie::CubieCube;

pub const N_FLIP: usize = 2048; // 2^11

pub(crate) fn flip_from_eo(eo: &[u8; 12]) -> usize {
    let mut f = 0usize;
    for i in 0..11 {
        f = f * 2 + eo[i] as usize;
    }
    f
}

impl CubieCube {
    pub fn flip(&self) -> usize {
        flip_from_eo(&self.eo)
    }
}

pub fn cubie_from_flip(mut f: usize) -> CubieCube {
    let mut c = CubieCube::SOLVED;
    let mut sum = 0u8;
    for i in (0..11).rev() {
        let d = (f % 2) as u8;
        c.eo[i] = d;
        sum += d;
        f /= 2;
    }
    c.eo[11] = sum % 2;
    c
}
