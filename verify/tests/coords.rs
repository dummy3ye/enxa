//! Coordinate tests: ranking round-trips and agreement between move tables and cubies.
//! Owns: coordinate correctness only. Must NOT: test facelets or solvers.

use cubecore::*;

#[test]
fn coordinates_roundtrip() {
    for x in 0..N_TWIST {
        assert_eq!(cubie_from_twist(x).twist(), x);
    }
    for x in 0..N_FLIP {
        assert_eq!(cubie_from_flip(x).flip(), x);
    }
    for x in 0..N_SLICE {
        assert_eq!(cubie_from_slice(x).slice_pos(), x);
    }
    for x in (0..N_CPERM).step_by(7).chain([N_CPERM - 1]) {
        assert_eq!(cubie_from_cperm(x).cperm(), x);
    }
    let s = CubieCube::SOLVED;
    assert_eq!(
        (s.twist(), s.flip(), s.slice_pos(), s.cperm()),
        (0, 0, SLICE_SOLVED, 0)
    );
}

#[test]
fn move_tables_match_real_cube_moves() {
    let mt = MoveTables::new();
    let mut rng = Rng::new(3);
    for _ in 0..300 {
        let c = CubieCube::random(&mut rng);
        for m in Move::all() {
            let n = c.apply(m);
            let mi = m.0 as usize;
            assert_eq!(mt.twist[c.twist() * 18 + mi] as usize, n.twist());
            assert_eq!(mt.flip[c.flip() * 18 + mi] as usize, n.flip());
            assert_eq!(mt.slice[c.slice_pos() * 18 + mi] as usize, n.slice_pos());
            assert_eq!(mt.cperm[c.cperm() * 18 + mi] as usize, n.cperm());
        }
    }
}
