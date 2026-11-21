//! Corner permutation coordinate (8! values) and its decomposition.
//!
//! Owns: `N_CPERM`, `CubieCube::cperm`, `cubie_from_cperm`.
//! Must NOT: encode orientation/slice coordinates or depend on move tables.

use crate::cubie::CubieCube;

pub const N_CPERM: usize = 40320; // 8!

impl CubieCube {
    /// Lehmer-style rank of the corner permutation.
    pub fn cperm(&self) -> usize {
        let p = &self.cp;
        let mut r = 0usize;
        let mut fact = 1usize;
        for i in 1..8 {
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
}

pub fn cubie_from_cperm(mut r: usize) -> CubieCube {
    let mut inv = [0usize; 8];
    for (i, slot) in inv.iter_mut().enumerate().skip(1) {
        *slot = r % (i + 1);
        r /= i + 1;
    }
    let mut avail: Vec<u8> = (0..8).collect();
    let mut c = CubieCube::SOLVED;
    for i in (0..8).rev() {
        c.cp[i] = avail.remove(i - inv[i]);
    }
    c
}
