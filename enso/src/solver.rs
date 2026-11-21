//! The `Enso` solver and its allocation-free descent path.
//!
//! Owns: fixed-buffer descent, stage sequencing, subgroup assertions and the `Solver` impl.
//! Must NOT: build tables, parse CLI arguments, or use search or `dyn` dispatch.

use crate::chain::{DescentStage, TableIndex};
use crate::stages::{orient, r#final, slicecorner, tetrad};
use crate::subgroup::{edge_rank, in_g1, in_g2};
use crate::tables::Tables;
use cubecore::{CubieCube, Move, Solver};
use std::time::Duration;

/// Upper bound on emitted moves. Every stage's BFS depth is far below this.
pub const MAX_SOLUTION_LEN: usize = 128;

fn descend<T: TableIndex>(
    st: &DescentStage,
    table: &[T],
    mut x: usize,
    goal: usize,
    c: &mut CubieCube,
    out: &mut [u8; MAX_SOLUTION_LEN],
    n: &mut usize,
) -> Result<(), &'static str> {
    while x != goal {
        let (m, nx) = st.step(x, table).ok_or("stuck before stage goal")?;
        if *n >= out.len() {
            return Err("solution buffer overflow");
        }
        out[*n] = m.0;
        *n += 1;
        *c = c.apply(m);
        x = nx;
    }
    Ok(())
}

pub struct Enso {
    tables: Tables,
}

impl Enso {
    /// Builds all stage tables. This is startup cost, reported separately from solve latency.
    pub fn new() -> Self {
        Enso {
            tables: Tables::build(),
        }
    }

    pub fn table_build_time(&self) -> Duration {
        self.tables.build_time
    }

    /// Allocation-free descent into a caller-provided fixed-size buffer.
    pub fn solve_into(
        &self,
        cube: &CubieCube,
        out: &mut [u8; MAX_SOLUTION_LEN],
    ) -> Result<usize, &'static str> {
        if cube.verify().is_err() {
            return Err("invalid cube");
        }
        let t = &self.tables;
        let mut c = *cube;
        let mut n = 0usize;

        descend(
            &t.st1,
            &t.t1,
            orient::coord(&c),
            orient::GOAL,
            &mut c,
            out,
            &mut n,
        )?;
        debug_assert!(in_g1(&c));

        descend(
            &t.st2,
            &t.t2,
            slicecorner::coord(&c),
            slicecorner::GOAL,
            &mut c,
            out,
            &mut n,
        )?;
        debug_assert!(in_g2(&c));

        descend(
            &t.st3,
            &t.t3,
            tetrad::encode_g2(&c, &t.corner_class),
            t.goal3,
            &mut c,
            out,
            &mut n,
        )?;
        debug_assert!(t.in_g3(&c));

        // Guard before indexing the 663,552-entry table: a bad stage-3 outcome must
        // return an error, never panic on cube data.
        let ci = t.cp_idx[c.cperm()];
        let ei = t.ep_idx[edge_rank(&c.ep)];
        if ci == u16::MAX || ei == u16::MAX {
            return Err("stage 3 did not reach G3");
        }
        descend(
            &t.st4,
            &t.t4,
            ci as usize * r#final::NEP + ei as usize,
            r#final::GOAL,
            &mut c,
            out,
            &mut n,
        )?;

        if !c.is_solved() {
            return Err("descent finished on an unsolved cube");
        }
        Ok(n)
    }
}

impl Default for Enso {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver for Enso {
    fn name(&self) -> &str {
        "enso-variant-a"
    }

    fn solve(&mut self, cube: &CubieCube) -> Option<Vec<Move>> {
        let mut buf = [0u8; MAX_SOLUTION_LEN];
        let n = self.solve_into(cube, &mut buf).ok()?;
        Some(buf[..n].iter().map(|&m| Move(m)).collect())
    }
}

impl Tables {
    /// Test-only: run the chain while asserting each subgroup invariant.
    #[cfg(test)]
    fn solve_checkpoints(&self, cube: &CubieCube, out: &mut [u8; MAX_SOLUTION_LEN]) -> usize {
        let mut c = *cube;
        let mut n = 0;
        descend(
            &self.st1,
            &self.t1,
            orient::coord(&c),
            orient::GOAL,
            &mut c,
            out,
            &mut n,
        )
        .unwrap();
        assert!(in_g1(&c));
        descend(
            &self.st2,
            &self.t2,
            slicecorner::coord(&c),
            slicecorner::GOAL,
            &mut c,
            out,
            &mut n,
        )
        .unwrap();
        assert!(in_g2(&c));
        descend(
            &self.st3,
            &self.t3,
            tetrad::encode_g2(&c, &self.corner_class),
            self.goal3,
            &mut c,
            out,
            &mut n,
        )
        .unwrap();
        assert!(self.in_g3(&c));
        descend(
            &self.st4,
            &self.t4,
            self.cp_idx[c.cperm()] as usize * r#final::NEP + self.ep_idx[edge_rank(&c.ep)] as usize,
            r#final::GOAL,
            &mut c,
            out,
            &mut n,
        )
        .unwrap();
        assert!(c.is_solved());
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::tables;
    use cubecore::{random_alg, Rng};

    #[test]
    fn each_stage_lands_in_its_subgroup_and_the_cube_is_solved() {
        let t = tables();
        let mut rng = Rng::new(105);
        for _ in 0..2000 {
            let alg = random_alg(&mut rng, 40);
            let cube = CubieCube::SOLVED.apply_all(&alg);
            let mut buf = [0u8; MAX_SOLUTION_LEN];
            let n = t.solve_checkpoints(&cube, &mut buf);
            assert!(cube
                .apply_all(&buf[..n].iter().map(|&m| Move(m)).collect::<Vec<_>>())
                .is_solved());
        }
    }
}
