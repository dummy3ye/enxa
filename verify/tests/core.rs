use cubecore::coord::*;
use cubecore::cubie::MOVES;
use cubecore::facelet::{from_facelets, to_facelets};
use cubecore::moves::{format_alg, parse_alg, random_alg};
use cubecore::rng::Rng;
use cubecore::tables::MoveTables;
use cubecore::{CubieCube, Move};

fn alg(s: &str) -> CubieCube {
    CubieCube::SOLVED.apply_all(&parse_alg(s).unwrap())
}

#[test]
fn every_move_has_order_4_and_half_turns_have_order_2() {
    for face in 0..6u8 {
        let q = MOVES[(face * 3) as usize];
        let mut c = CubieCube::SOLVED;
        for _ in 0..4 {
            c = c.multiply(&q);
        }
        assert!(c.is_solved(), "face {face}^4 != identity");
        assert!(!q.multiply(&q).is_solved());
        let h = MOVES[(face * 3 + 1) as usize];
        assert!(h.multiply(&h).is_solved());
    }
}

#[test]
fn sexy_move_has_order_6() {
    let mut c = CubieCube::SOLVED;
    for i in 1..=6 {
        c = c.apply_all(&parse_alg("R U R' U'").unwrap());
        assert_eq!(c.is_solved(), i == 6, "after {i} repetitions");
    }
}

#[test]
fn superflip_algorithm_is_correct_and_20_moves() {
    let a = parse_alg("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
    assert_eq!(a.len(), 20);
    assert_eq!(CubieCube::SOLVED.apply_all(&a), CubieCube::superflip());
    assert!(CubieCube::superflip().verify().is_ok());
}

#[test]
fn parse_and_format_roundtrip() {
    let s = "R U R' U' F2 D L' B";
    assert_eq!(format_alg(&parse_alg(s).unwrap()), s);
    assert!(parse_alg("X").is_err());
    assert!(parse_alg("R3").is_err());
}

#[test]
fn inverse_and_move_inverse() {
    let mut rng = Rng::new(7);
    for _ in 0..500 {
        let c = CubieCube::random(&mut rng);
        assert!(c.verify().is_ok());
        assert!(c.multiply(&c.inverse()).is_solved());
        assert!(c.inverse().multiply(&c).is_solved());
    }
    for m in Move::all() {
        assert!(CubieCube::SOLVED.apply(m).apply(m.inverse()).is_solved());
    }
}

#[test]
fn moves_preserve_solvability() {
    let mut rng = Rng::new(11);
    let c = CubieCube::SOLVED.apply_all(&random_alg(&mut rng, 200));
    assert!(c.verify().is_ok());
}

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

#[test]
fn facelets_roundtrip_and_known_strings() {
    assert_eq!(
        to_facelets(&CubieCube::SOLVED),
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
    );
    assert_eq!(
        to_facelets(&alg("R")),
        "UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB"
    );
    let mut rng = Rng::new(5);
    for _ in 0..300 {
        let c = CubieCube::random(&mut rng);
        assert_eq!(from_facelets(&to_facelets(&c)).unwrap(), c);
    }
    assert!(from_facelets("short").is_err());
}
