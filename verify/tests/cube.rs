//! Cubie-model tests: move orders, inversion, solvability and the known superflip.
//! Owns: cubie operations only. Must NOT: test coordinates, facelets or solvers.

use cubecore::{parse_alg, random_alg, CubieCube, Move, Rng, MOVES};

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
