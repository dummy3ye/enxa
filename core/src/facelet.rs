//! 54-char facelet strings. Order: U R F D L B, nine stickers each, as in Kociemba's tools.
//! Solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB".

use crate::cubie::CubieCube;

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

pub fn from_facelets(s: &str) -> Result<CubieCube, String> {
    let chars: Vec<char> = s.trim().chars().collect();
    if chars.len() != 54 {
        return Err(format!("expected 54 facelets, got {}", chars.len()));
    }
    let mut f = [0u8; 54];
    for (i, ch) in chars.iter().enumerate() {
        f[i] = LETTERS
            .iter()
            .position(|l| l == ch)
            .ok_or_else(|| format!("bad facelet character '{ch}'"))? as u8;
    }
    for face in 0..6 {
        if f[face * 9 + 4] != face as u8 {
            return Err("center facelets are not in U R F D L B order".into());
        }
    }
    let mut c = CubieCube::SOLVED;
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
                c.cp[i] = j as u8;
                c.co[i] = ori as u8;
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
                c.ep[i] = j as u8;
                c.eo[i] = 0;
                found = true;
                break;
            }
            if a == EDGE_COLOR[j][1] && b == EDGE_COLOR[j][0] {
                c.ep[i] = j as u8;
                c.eo[i] = 1;
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!("edge slot {i} has an impossible color pair"));
        }
    }
    c.verify().map_err(|e| e.to_string())?;
    Ok(c)
}
