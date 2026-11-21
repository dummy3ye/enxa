//! The `Enso` solver and its allocation-free descent path.
//!
//! Owns: fixed-buffer descent, stage sequencing, subgroup assertions and the `Solver` impl.
//! Must NOT: build tables, parse CLI arguments, or use search or `dyn` dispatch.

use crate::chain::PackedStage;
use crate::stages::{orient, r#final, slicecorner, tetrad};
use crate::subgroup::{edge_rank, eor_pattern, in_g1, in_g2, tet_pattern};
use crate::tablefile::{self, Source};
use crate::tables::Tables;
use cubecore::{from_facelets_parts, CubieCube, Move, Solver, MOVES, N_SLICE};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Upper bound on emitted moves. Every stage's BFS depth is far below this.
pub const MAX_SOLUTION_LEN: usize = 128;

/// Rebuild and rewrite the table cache. Returns startup time and the written path.
pub fn build_tables() -> (Duration, Option<PathBuf>) {
    let startup = tablefile::force_build();
    (
        startup.elapsed,
        startup.source.path().map(Path::to_path_buf),
    )
}

fn descend(
    st: &PackedStage,
    mut x: usize,
    goal: usize,
    c: &mut CubieCube,
    out: &mut [u8; MAX_SOLUTION_LEN],
    n: &mut usize,
) -> Result<(), &'static str> {
    while x != goal {
        let (m, nx) = st.step(x).ok_or("stuck before stage goal")?;
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

/// Apply one move to the cubie arrays that can still be read by a later stage.
///
/// `eo` is never needed after the initial flip and `co` only until the stage-2 coordinate is
/// ranked, so callers pass `track_co = true` only in stage 1. `track_co` is a literal at every
/// call site and this function is inlined, so the branch is eliminated in the hot loop.
#[inline]
fn apply_parts(parts: &mut cubecore::FaceletParts, m: u8, track_co: bool) {
    let mv = &MOVES[m as usize];
    let old_cp = parts.cp;
    for i in 0..8 {
        parts.cp[i] = old_cp[mv.cp[i] as usize];
    }
    if track_co {
        let old_co = parts.co;
        for i in 0..8 {
            let s = old_co[mv.cp[i] as usize] + mv.co[i];
            // Both operands are 0..2, so one conditional subtract is exactly `% 3`.
            parts.co[i] = if s >= 3 { s - 3 } else { s };
        }
    }
    let old_ep = parts.ep;
    for i in 0..12 {
        parts.ep[i] = old_ep[mv.ep[i] as usize];
    }
}

pub struct Enso {
    tables: Tables,
    startup_elapsed: Duration,
    startup_source: Source,
}

impl Enso {
    /// Loads validated prebuilt tables, rebuilding the cache in-process when needed.
    /// Startup time is reported separately from solve latency.
    pub fn new() -> Self {
        let startup = tablefile::load();
        Enso {
            tables: startup.tables,
            startup_elapsed: startup.elapsed,
            startup_source: startup.source,
        }
    }

    /// Time spent loading or building tables at construction.
    pub fn table_build_time(&self) -> Duration {
        self.startup_elapsed
    }

    /// `true` when tables came from a validated file rather than an in-process build.
    pub fn tables_loaded_from_file(&self) -> bool {
        self.startup_source.is_loaded()
    }

    /// Table file used or written, if the cache path was writable.
    pub fn table_path(&self) -> Option<&Path> {
        self.startup_source.path()
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

        descend(&t.st1, orient::coord(&c), orient::GOAL, &mut c, out, &mut n)?;
        debug_assert!(in_g1(&c));

        descend(
            &t.st2,
            slicecorner::coord(&c),
            slicecorner::GOAL,
            &mut c,
            out,
            &mut n,
        )?;
        debug_assert!(in_g2(&c));

        descend(
            &t.st3,
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

    /// Facelet input path: parse once into cubie arrays, update them with each emitted move,
    /// and rank the three stage coordinates only at their boundaries. No `CubieCube` is
    /// built and no large random-access move tables are read on this path.
    pub fn solve_facelets_into(
        &self,
        s: &str,
        out: &mut [u8; MAX_SOLUTION_LEN],
    ) -> Result<usize, String> {
        let mut parts = from_facelets_parts(s)?;
        let t = &self.tables;
        let mut n = 0usize;

        macro_rules! emit {
            ($m:expr) => {{
                if n >= out.len() {
                    return Err("solution buffer overflow".to_string());
                }
                out[n] = $m.0;
                n += 1;
            }};
        }

        // Stage 1: flip only; the packed step already yields the next flip coordinate.
        let mut flip = parts.flip();
        while flip != orient::GOAL {
            let (m, nx) = t
                .st1
                .step(flip)
                .ok_or_else(|| "stuck before stage goal".to_string())?;
            emit!(m);
            apply_parts(&mut parts, m.0, true);
            flip = nx;
        }

        // Stage 2: (twist, slice), both ranked once from the arrays updated by stage 1.
        let mut x2 = parts.twist() * N_SLICE + parts.slice_pos();
        while x2 != slicecorner::GOAL {
            let (m, nx) = t
                .st2
                .step(x2)
                .ok_or_else(|| "stuck before stage goal".to_string())?;
            emit!(m);
            apply_parts(&mut parts, m.0, false);
            x2 = nx;
        }

        // Stage 3: (tet, eor, corner class), also ranked once at the G2 boundary.
        let tet = tet_pattern(&parts.cp);
        let eor = eor_pattern(&parts.ep);
        let mut x3 = (tet * 70 + eor) * 6 + t.corner_class[parts.cperm()] as usize;
        while x3 != t.goal3 {
            let (m, nx) = t
                .st3
                .step(x3)
                .ok_or_else(|| "stuck before stage goal".to_string())?;
            emit!(m);
            apply_parts(&mut parts, m.0, false);
            x3 = nx;
        }

        // Guard before indexing the full half-turn table: bad stage-3 output is an error.
        let ci = t.cp_idx[parts.cperm()];
        let ei = t.ep_idx[edge_rank(&parts.ep)];
        if ci == u16::MAX || ei == u16::MAX {
            return Err("stage 3 did not reach G3".to_string());
        }
        let mut x4 = ci as usize * r#final::NEP + ei as usize;
        while x4 != r#final::GOAL {
            let (m, nx) = t
                .st4
                .step(x4)
                .ok_or_else(|| "stuck before stage goal".to_string())?;
            emit!(m);
            x4 = nx;
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
