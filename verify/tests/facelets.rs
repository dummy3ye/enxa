//! Facelet and algorithm-string tests.
//! Owns: 54-character parsing/formatting and algorithm round-trips. Must NOT: test solvers.

use cubecore::{format_alg, from_facelets, parse_alg, to_facelets, CubieCube, Rng};

fn alg(s: &str) -> CubieCube {
    CubieCube::SOLVED.apply_all(&parse_alg(s).unwrap())
}

#[test]
fn parse_and_format_roundtrip() {
    let s = "R U R' U' F2 D L' B";
    assert_eq!(format_alg(&parse_alg(s).unwrap()), s);
    assert!(parse_alg("X").is_err());
    assert!(parse_alg("R3").is_err());
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
