//! enso integration tests: validity, rejection of impossible cubes, and the fixed-buffer API.
//! Owns: solver-level enso checks. Must NOT: duplicate the million-cube gate in `gate.rs`.

use cubecore::{to_facelets, CubieCube, Rng, Solver};
use enso::{Enso, MAX_SOLUTION_LEN};

#[test]
fn solves_solved_cube_with_zero_moves() {
    let mut enso = Enso::new();
    let sol = enso.solve(&CubieCube::SOLVED).expect("no solution");
    assert!(sol.is_empty());
}

#[test]
fn rejects_unsolvable_cubes() {
    let mut enso = Enso::new();
    let mut c = CubieCube::SOLVED;
    c.co[0] = 1; // single twisted corner
    assert!(enso.solve(&c).is_none());
}

#[test]
fn facelet_path_agrees_with_cubie_path() {
    let enso = Enso::new();
    let mut rng = Rng::new(31338);
    let mut cubie_buf = [0u8; MAX_SOLUTION_LEN];
    let mut facelet_buf = [0u8; MAX_SOLUTION_LEN];
    for _ in 0..1000 {
        let cube = CubieCube::random(&mut rng);
        let facelets = to_facelets(&cube);
        let n_cubie = enso.solve_into(&cube, &mut cubie_buf).expect("no solution");
        let n_facelets = enso
            .solve_facelets_into(&facelets, &mut facelet_buf)
            .expect("no solution");
        assert_eq!(n_cubie, n_facelets);
        assert_eq!(&cubie_buf[..n_cubie], &facelet_buf[..n_facelets]);
        let sol: Vec<cubecore::Move> = facelet_buf[..n_facelets]
            .iter()
            .map(|&m| cubecore::Move(m))
            .collect();
        assert!(cube.apply_all(&sol).is_solved());
    }
    assert!(enso.solve_facelets_into("short", &mut facelet_buf).is_err());
}

#[test]
fn fixed_buffer_solve_agrees_with_trait_solve() {
    let mut enso = Enso::new();
    let mut rng = Rng::new(31337);
    let mut buf = [0u8; MAX_SOLUTION_LEN];
    for _ in 0..1000 {
        let cube = CubieCube::random(&mut rng);
        let n = enso.solve_into(&cube, &mut buf).expect("no solution");
        let sol = enso.solve(&cube).expect("no solution");
        assert_eq!(n, sol.len());
        assert_eq!(&buf[..n], &sol.iter().map(|m| m.0).collect::<Vec<_>>()[..]);
        assert!(cube.apply_all(&sol).is_solved());
    }
}
