//! Stage G2→G3: solve the paired corner/edge coset with <U,D,L2,R2,F2,B2>.
//!
//! Owns: the 29,400-state (tetrad pattern × edge-orbit pattern × corner left-coset)
//! coordinate and its encoding/decoding. Must NOT: use L/R quarter turns, which would
//! eject UD-slice edges from the slice.

use crate::subgroup::{eor_pattern, tet_pattern, COORD3, G2_MOVES};
use cubecore::perm_parity;
use cubecore::CubieCube;

pub(crate) const MOVES: [u8; 10] = G2_MOVES;
pub(crate) const N: usize = COORD3;

#[inline]
pub(crate) fn goal(corner_class: &[u8]) -> usize {
    encode_g2(&CubieCube::SOLVED, corner_class)
}

#[inline]
pub(crate) fn encode_g2(c: &CubieCube, corner_class: &[u8]) -> usize {
    (tet_pattern(&c.cp) * 70 + eor_pattern(&c.ep)) * 6 + corner_class[c.cperm()] as usize
}

#[inline]
pub(crate) fn decode_g2(x: usize, rep_cp: &[[u8; 8]], rep_ep: &[[u8; 12]]) -> CubieCube {
    let (tet, eor, cls) = (x / 420, (x / 6) % 70, x % 6);
    let cp = rep_cp[tet * 6 + cls];
    let ep = rep_ep[eor * 2 + perm_parity(&cp) as usize];
    CubieCube {
        cp,
        co: [0; 8],
        ep,
        eo: [0; 12],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subgroup::{in_g2, ALL, G1_MOVES};
    use crate::tables::{build_g2_reps, build_hcp, tables};
    use cubecore::Move;
    use cubecore::Rng;

    #[test]
    fn stage3_coordinate_roundtrips_and_g2_moves_stay_in_domain() {
        let t = tables();
        let (_, rep_cp, rep_ep) = build_g2_reps(&build_hcp());
        let mut rng = Rng::new(104);
        let all: Vec<Move> = ALL.iter().map(|&m| Move(m)).collect();
        for _ in 0..2000 {
            let mut c = CubieCube::SOLVED;
            for _ in 0..60 {
                c = c.apply(Move(G1_MOVES[rng.below(14) as usize]));
            }
            if !in_g2(&c) {
                continue;
            }
            let x = encode_g2(&c, &t.corner_class);
            let decoded = decode_g2(x, &rep_cp, &rep_ep);
            assert_eq!(encode_g2(&decoded, &t.corner_class), x);
            for m in all.iter().copied().filter(|m| G2_MOVES.contains(&m.0)) {
                let n = decoded.apply(m);
                let nx = encode_g2(&n, &t.corner_class);
                assert_eq!(
                    encode_g2(&decode_g2(nx, &rep_cp, &rep_ep), &t.corner_class),
                    nx
                );
            }
        }
    }
}
