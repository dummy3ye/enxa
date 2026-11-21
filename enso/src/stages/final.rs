//! Stage G3→solved: the half-turn subgroup, solved by a full BFS over 663,552 states.
//!
//! Owns: the (corner perm, edge perm) coordinate of G3 and its half-turn generators.
//! Must NOT: use quarter turns or touch other stages' coordinates.

use crate::subgroup::{COORD4, HALF};

pub(crate) const MOVES: [u8; 6] = HALF;
pub(crate) const N: usize = COORD4;
pub(crate) const GOAL: usize = 0;
pub(crate) const NEP: usize = 6912;
