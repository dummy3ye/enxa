//! Coordinates: compact integer encodings of parts of the cube state.

use crate::cubie::CubieCube;

pub const N_TWIST: usize = 2187; // 3^7
pub const N_FLIP: usize = 2048; // 2^11
pub const N_SLICE: usize = 495; // C(12,4): positions of the 4 UD-slice edges
pub const N_CPERM: usize = 40320; // 8!
pub const SLICE_SOLVED: usize = 494; // slice edges FR FL BL BR sit in slots 8..12

/// Binomial coefficient C(n, k); 0 when k > n.
pub fn cnk(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let mut r = 1usize;
    for i in 0..k {
        r = r * (n - i) / (i + 1);
    }
    r
}

impl CubieCube {
    pub fn twist(&self) -> usize {
        let mut t = 0usize;
        for i in 0..7 {
            t = t * 3 + self.co[i] as usize;
        }
        t
    }

    pub fn flip(&self) -> usize {
        let mut f = 0usize;
        for i in 0..11 {
            f = f * 2 + self.eo[i] as usize;
        }
        f
    }

    /// Combinadic rank of the slots that hold UD-slice edges (edge ids 8..12).
    pub fn slice_pos(&self) -> usize {
        let mut rank = 0usize;
        let mut k = 0usize;
        for pos in 0..12 {
            if self.ep[pos] >= 8 {
                rank += cnk(pos, k + 1);
                k += 1;
            }
        }
        rank
    }

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
