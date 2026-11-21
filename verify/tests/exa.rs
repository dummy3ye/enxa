//! exa integration tests: optimality oracle behavior on short scrambles.
//! Owns: exa-level validity checks. Must NOT: benchmark or weaken the optimality claim.

use cubecore::{random_alg, CubieCube, Rng, Solver};
use exa::Exa;

#[test]
fn solves_solved_cube_with_zero_moves() {
    let mut exa = Exa::new();
    assert_eq!(exa.solve(&CubieCube::SOLVED).unwrap().len(), 0);
}

#[test]
fn solutions_are_valid_and_never_longer_than_the_scramble() {
    let mut exa = Exa::new();
    let mut rng = Rng::new(42);
    for len in 1..=7 {
        for _ in 0..5 {
            let scramble = random_alg(&mut rng, len);
            let cube = CubieCube::SOLVED.apply_all(&scramble);
            let sol = exa.solve(&cube).expect("no solution");
            assert!(cube.apply_all(&sol).is_solved());
            assert!(
                sol.len() <= len,
                "found {} moves for a {}-move scramble",
                sol.len(),
                len
            );
        }
    }
}

#[test]
fn rejects_unsolvable_cubes() {
    let mut exa = Exa::new();
    let mut c = CubieCube::SOLVED;
    c.co[0] = 1; // single twisted corner: physically impossible
    assert!(exa.solve(&c).is_none());
}
