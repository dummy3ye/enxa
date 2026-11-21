//! The 18 HTM moves.

use crate::rng::Rng;
use std::fmt;

/// A move, encoded as `face * 3 + (power - 1)`.
/// Faces: U=0, R=1, F=2, D=3, L=4, B=5. Power: 1 = clockwise, 2 = half turn, 3 = counter-clockwise.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Move(pub u8);

impl Move {
    pub const COUNT: usize = 18;
    pub const FACES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

    #[inline]
    pub fn new(face: u8, power: u8) -> Move {
        debug_assert!(face < 6 && (1..=3).contains(&power));
        Move(face * 3 + power - 1)
    }

    #[inline]
    pub fn face(self) -> u8 {
        self.0 / 3
    }

    #[inline]
    pub fn power(self) -> u8 {
        self.0 % 3 + 1
    }

    #[inline]
    pub fn inverse(self) -> Move {
        Move::new(self.face(), 4 - self.power())
    }

    pub fn all() -> impl Iterator<Item = Move> {
        (0..18u8).map(Move)
    }

    /// Redundancy pruning: no same-face repeats; opposite faces only in one canonical order
    /// (U before D, R before L, F before B), because they commute.
    #[inline]
    pub fn can_follow(self, last: Option<Move>) -> bool {
        match last {
            None => true,
            Some(l) => {
                let (f, lf) = (self.face(), l.face());
                f != lf && !(f % 3 == lf % 3 && f < lf)
            }
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = Self::FACES[self.face() as usize];
        match self.power() {
            1 => write!(f, "{c}"),
            2 => write!(f, "{c}2"),
            _ => write!(f, "{c}'"),
        }
    }
}

/// Parse e.g. "R U R' U' F2".
pub fn parse_alg(s: &str) -> Result<Vec<Move>, String> {
    let mut out = Vec::new();
    for tok in s.split_whitespace() {
        let mut chars = tok.chars();
        let face_ch = chars.next().ok_or("empty token")?;
        let face = Move::FACES
            .iter()
            .position(|&c| c == face_ch)
            .ok_or_else(|| format!("bad face in token '{tok}'"))? as u8;
        let power = match chars.as_str() {
            "" => 1,
            "2" => 2,
            "'" => 3,
            _ => return Err(format!("bad suffix in token '{tok}'")),
        };
        out.push(Move::new(face, power));
    }
    Ok(out)
}

pub fn format_alg(moves: &[Move]) -> String {
    moves
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A random move sequence of `len` moves that respects `can_follow`.
pub fn random_alg(rng: &mut Rng, len: usize) -> Vec<Move> {
    let mut out = Vec::with_capacity(len);
    let mut last = None;
    while out.len() < len {
        let m = Move(rng.below(18) as u8);
        if m.can_follow(last) {
            out.push(m);
            last = Some(m);
        }
    }
    out
}
