//! Stage G0→G1: solve edge orientation with all 18 moves.
//!
//! Owns: the flip coordinate, the goal (0) and the generator set.
//! Must NOT: touch later stages' coordinates, tables, or the solver.

use crate::subgroup::ALL;
use cubecore::CubieCube;
use cubecore::N_FLIP;

pub(crate) const MOVES: [u8; 18] = ALL;
pub(crate) const N: usize = N_FLIP;
pub(crate) const GOAL: usize = 0;

#[inline]
pub(crate) fn coord(c: &CubieCube) -> usize {
    c.flip()
}
