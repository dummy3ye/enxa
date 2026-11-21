//! Cubie-level model. Kociemba conventions.
//!
//! Corners: URF UFL ULB UBR DFR DLF DBL DRB (0..8)
//! Edges:   UR UF UL UB DR DF DL DB FR FL BL BR (0..12)
//!
//! `a.multiply(&b)` means "apply a, then apply b".
//!
//! Owns: cubie arrays, multiplication, inversion, verification, random states.
//! Must NOT: parse strings, encode coordinates, or know about solvers.

use crate::moves::Move;
use crate::rng::Rng;

pub mod corner {
    pub const URF: u8 = 0;
    pub const UFL: u8 = 1;
    pub const ULB: u8 = 2;
    pub const UBR: u8 = 3;
    pub const DFR: u8 = 4;
    pub const DLF: u8 = 5;
    pub const DBL: u8 = 6;
    pub const DRB: u8 = 7;
}

pub mod edge {
    pub const UR: u8 = 0;
    pub const UF: u8 = 1;
    pub const UL: u8 = 2;
    pub const UB: u8 = 3;
    pub const DR: u8 = 4;
    pub const DF: u8 = 5;
    pub const DL: u8 = 6;
    pub const DB: u8 = 7;
    pub const FR: u8 = 8;
    pub const FL: u8 = 9;
    pub const BL: u8 = 10;
    pub const BR: u8 = 11;
}

use self::corner::*;
use self::edge::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubieCube {
    /// Which corner cubie sits in each corner slot.
    pub cp: [u8; 8],
    /// Corner twist 0..3 per slot.
    pub co: [u8; 8],
    /// Which edge cubie sits in each edge slot.
    pub ep: [u8; 12],
    /// Edge flip 0..2 per slot.
    pub eo: [u8; 12],
}

impl CubieCube {
    pub const SOLVED: CubieCube = CubieCube {
        cp: [0, 1, 2, 3, 4, 5, 6, 7],
        co: [0; 8],
        ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        eo: [0; 12],
    };

    /// Group multiplication: apply `self`, then `b`.
    pub const fn multiply(&self, b: &CubieCube) -> CubieCube {
        let mut r = CubieCube::SOLVED;
        let mut i = 0;
        while i < 8 {
            let j = b.cp[i] as usize;
            r.cp[i] = self.cp[j];
            r.co[i] = (self.co[j] + b.co[i]) % 3;
            i += 1;
        }
        let mut i = 0;
        while i < 12 {
            let j = b.ep[i] as usize;
            r.ep[i] = self.ep[j];
            r.eo[i] = (self.eo[j] + b.eo[i]) % 2;
            i += 1;
        }
        r
    }

    pub fn inverse(&self) -> CubieCube {
        let mut r = CubieCube::SOLVED;
        for j in 0..8 {
            let p = self.cp[j] as usize;
            r.cp[p] = j as u8;
            r.co[p] = (3 - self.co[j]) % 3;
        }
        for j in 0..12 {
            let p = self.ep[j] as usize;
            r.ep[p] = j as u8;
            r.eo[p] = self.eo[j];
        }
        r
    }

    #[inline]
    pub fn apply(&self, m: Move) -> CubieCube {
        self.multiply(&MOVES[m.0 as usize])
    }

    pub fn apply_all(&self, moves: &[Move]) -> CubieCube {
        let mut c = *self;
        for &m in moves {
            c = c.apply(m);
        }
        c
    }

    #[inline]
    pub fn is_solved(&self) -> bool {
        *self == CubieCube::SOLVED
    }

    /// All 12 edges flipped, everything else solved. Known to need exactly 20 HTM moves.
    pub fn superflip() -> CubieCube {
        let mut c = CubieCube::SOLVED;
        c.eo = [1; 12];
        c
    }

    /// Is this a physically reachable cube?
    pub fn verify(&self) -> Result<(), &'static str> {
        let mut seen = [false; 8];
        for &c in &self.cp {
            if c >= 8 || seen[c as usize] {
                return Err("invalid corner permutation");
            }
            seen[c as usize] = true;
        }
        let mut seen = [false; 12];
        for &e in &self.ep {
            if e >= 12 || seen[e as usize] {
                return Err("invalid edge permutation");
            }
            seen[e as usize] = true;
        }
        if self.co.iter().any(|&x| x >= 3)
            || self.co.iter().map(|&x| x as u32).sum::<u32>() % 3 != 0
        {
            return Err("corner twist is not solvable");
        }
        if self.eo.iter().any(|&x| x >= 2)
            || self.eo.iter().map(|&x| x as u32).sum::<u32>() % 2 != 0
        {
            return Err("edge flip is not solvable");
        }
        if perm_parity(&self.cp) != perm_parity(&self.ep) {
            return Err("permutation parity mismatch");
        }
        Ok(())
    }

    /// A uniformly random solvable cube.
    pub fn random(rng: &mut Rng) -> CubieCube {
        let mut c = CubieCube::SOLVED;
        shuffle(&mut c.cp, rng);
        shuffle(&mut c.ep, rng);
        if perm_parity(&c.cp) != perm_parity(&c.ep) {
            c.ep.swap(0, 1);
        }
        let mut s = 0u8;
        for i in 0..7 {
            c.co[i] = rng.below(3) as u8;
            s += c.co[i];
        }
        c.co[7] = (3 - s % 3) % 3;
        let mut s = 0u8;
        for i in 0..11 {
            c.eo[i] = rng.below(2) as u8;
            s += c.eo[i];
        }
        c.eo[11] = s % 2;
        c
    }
}

fn shuffle<const N: usize>(a: &mut [u8; N], rng: &mut Rng) {
    for i in (1..N).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        a.swap(i, j);
    }
}

pub fn perm_parity(p: &[u8]) -> u8 {
    let mut c = 0u8;
    for i in 0..p.len() {
        for j in 0..i {
            if p[j] > p[i] {
                c ^= 1;
            }
        }
    }
    c
}

// ---- the six basic clockwise face turns (U R F D L B) ----

const BASE: [CubieCube; 6] = [
    // U
    CubieCube {
        cp: [UBR, URF, UFL, ULB, DFR, DLF, DBL, DRB],
        co: [0; 8],
        ep: [UB, UR, UF, UL, DR, DF, DL, DB, FR, FL, BL, BR],
        eo: [0; 12],
    },
    // R
    CubieCube {
        cp: [DFR, UFL, ULB, URF, DRB, DLF, DBL, UBR],
        co: [2, 0, 0, 1, 1, 0, 0, 2],
        ep: [FR, UF, UL, UB, BR, DF, DL, DB, DR, FL, BL, UR],
        eo: [0; 12],
    },
    // F
    CubieCube {
        cp: [UFL, DLF, ULB, UBR, URF, DFR, DBL, DRB],
        co: [1, 2, 0, 0, 2, 1, 0, 0],
        ep: [UR, FL, UL, UB, DR, FR, DL, DB, UF, DF, BL, BR],
        eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
    },
    // D
    CubieCube {
        cp: [URF, UFL, ULB, UBR, DLF, DBL, DRB, DFR],
        co: [0; 8],
        ep: [UR, UF, UL, UB, DF, DL, DB, DR, FR, FL, BL, BR],
        eo: [0; 12],
    },
    // L
    CubieCube {
        cp: [URF, ULB, DBL, UBR, DFR, UFL, DLF, DRB],
        co: [0, 1, 2, 0, 0, 2, 1, 0],
        ep: [UR, UF, BL, UB, DR, DF, FL, DB, FR, UL, DL, BR],
        eo: [0; 12],
    },
    // B
    CubieCube {
        cp: [URF, UFL, UBR, DRB, DFR, DLF, ULB, DBL],
        co: [0, 0, 1, 2, 0, 0, 2, 1],
        ep: [UR, UF, UL, BR, DR, DF, DL, BL, FR, FL, UB, DB],
        eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
    },
];

const fn build_moves() -> [CubieCube; 18] {
    let mut out = [CubieCube::SOLVED; 18];
    let mut f = 0;
    while f < 6 {
        let b = BASE[f];
        let mut cur = b;
        let mut p = 0;
        while p < 3 {
            out[f * 3 + p] = cur;
            cur = cur.multiply(&b);
            p += 1;
        }
        f += 1;
    }
    out
}

/// All 18 moves as cubes, indexed by `Move.0`. Computed at compile time.
pub const MOVES: [CubieCube; 18] = build_moves();
