//! 54-char facelet strings. Order: U R F D L B, nine stickers each, as in Kociemba's tools.
//! Solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB".
//!
//! Owns: the sticker mapping between 54-character strings and cubies.
//! Must NOT: solve, search, or depend on move tables.

use crate::coord::{cperm_from_cp, flip_from_eo, slice_from_ep, twist_from_co};
use crate::cubie::{verify_arrays, CubieCube};

const CORNER_FACELET: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];
const EDGE_FACELET: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];
// Colors as face indices: U=0 R=1 F=2 D=3 L=4 B=5
const CORNER_COLOR: [[u8; 3]; 8] = [
    [0, 1, 2],
    [0, 2, 4],
    [0, 4, 5],
    [0, 5, 1],
    [3, 2, 1],
    [3, 4, 2],
    [3, 5, 4],
    [3, 1, 5],
];
const EDGE_COLOR: [[u8; 2]; 12] = [
    [0, 1],
    [0, 2],
    [0, 4],
    [0, 5],
    [3, 1],
    [3, 2],
    [3, 4],
    [3, 5],
    [2, 1],
    [2, 4],
    [5, 4],
    [5, 1],
];
const LETTERS: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

/// Cubie arrays decoded from a facelet string, without allocating an intermediate `Vec<char>`.
/// This is the hand-off format for solvers that want coordinates rather than a full cube.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceletParts {
    pub cp: [u8; 8],
    pub co: [u8; 8],
    pub ep: [u8; 12],
    pub eo: [u8; 12],
}

impl FaceletParts {
    pub fn verify(&self) -> Result<(), &'static str> {
        verify_arrays(&self.cp, &self.co, &self.ep, &self.eo)
    }

    pub fn flip(&self) -> usize {
        flip_from_eo(&self.eo)
    }

    pub fn twist(&self) -> usize {
        twist_from_co(&self.co)
    }

    pub fn slice_pos(&self) -> usize {
        slice_from_ep(&self.ep)
    }

    pub fn cperm(&self) -> usize {
        cperm_from_cp(&self.cp)
    }
}

pub fn to_facelets(c: &CubieCube) -> String {
    let mut f = [0u8; 54];
    for (i, slot) in f.iter_mut().enumerate() {
        *slot = (i / 9) as u8;
    }
    for i in 0..8 {
        let p = c.cp[i] as usize;
        let o = c.co[i] as usize;
        for n in 0..3 {
            f[CORNER_FACELET[i][(n + o) % 3]] = CORNER_COLOR[p][n];
        }
    }
    for i in 0..12 {
        let p = c.ep[i] as usize;
        let o = c.eo[i] as usize;
        for n in 0..2 {
            f[EDGE_FACELET[i][(n + o) % 2]] = EDGE_COLOR[p][n];
        }
    }
    f.iter().map(|&x| LETTERS[x as usize]).collect()
}

pub fn from_facelets_parts(s: &str) -> Result<FaceletParts, String> {
    let bytes = s.trim().as_bytes();
    if bytes.len() != 54 {
        return Err(format!("expected 54 facelets, got {}", bytes.len()));
    }
    let mut f = [0u8; 54];
    for (i, &b) in bytes.iter().enumerate() {
        f[i] = match b {
            b'U' => 0,
            b'R' => 1,
            b'F' => 2,
            b'D' => 3,
            b'L' => 4,
            b'B' => 5,
            _ => return Err(format!("bad facelet character '{}'", b as char)),
        };
    }
    for face in 0..6 {
        if f[face * 9 + 4] != face as u8 {
            return Err("center facelets are not in U R F D L B order".into());
        }
    }
    let mut parts = FaceletParts {
        cp: [0; 8],
        co: [0; 8],
        ep: [0; 12],
        eo: [0; 12],
    };
    for i in 0..8 {
        let mut ori = 0usize;
        while ori < 3 {
            let col = f[CORNER_FACELET[i][ori]];
            if col == 0 || col == 3 {
                break;
            }
            ori += 1;
        }
        if ori == 3 {
            return Err(format!("corner slot {i} has no U/D sticker"));
        }
        let c1 = f[CORNER_FACELET[i][(ori + 1) % 3]];
        let c2 = f[CORNER_FACELET[i][(ori + 2) % 3]];
        let mut found = false;
        for j in 0..8 {
            if CORNER_COLOR[j][1] == c1 && CORNER_COLOR[j][2] == c2 {
                parts.cp[i] = j as u8;
                parts.co[i] = ori as u8;
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!("corner slot {i} has an impossible color pair"));
        }
    }
    for i in 0..12 {
        let a = f[EDGE_FACELET[i][0]];
        let b = f[EDGE_FACELET[i][1]];
        let mut found = false;
        for j in 0..12 {
            if a == EDGE_COLOR[j][0] && b == EDGE_COLOR[j][1] {
                parts.ep[i] = j as u8;
                parts.eo[i] = 0;
                found = true;
                break;
            }
            if a == EDGE_COLOR[j][1] && b == EDGE_COLOR[j][0] {
                parts.ep[i] = j as u8;
                parts.eo[i] = 1;
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!("edge slot {i} has an impossible color pair"));
        }
    }
    parts.verify().map_err(|e| e.to_string())?;
    Ok(parts)
}

pub fn from_facelets(s: &str) -> Result<CubieCube, String> {
    let parts = from_facelets_parts(s)?;
    Ok(CubieCube {
        cp: parts.cp,
        co: parts.co,
        ep: parts.ep,
        eo: parts.eo,
    })
}
