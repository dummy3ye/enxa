use cubecore::rng::Rng;
use cubecore::solver::Solver;
use cubecore::CubieCube;
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

/// The M3 correctness gate: 1,000,000 seeded random cubes, every solution verified.
/// Run explicitly with `cargo test -p verify --release --test enso -- --ignored --nocapture`.
#[test]
#[ignore = "million-cube gate; run explicitly"]
fn million_random_cubes_all_valid() {
    let enso = Enso::new();
    let mut rng = Rng::new(1);
    let mut total = 0u64;
    let mut max_len = 0usize;
    let mut buf = [0u8; MAX_SOLUTION_LEN];
    for i in 0..1_000_000u64 {
        let cube = CubieCube::random(&mut rng);
        let n = match enso.solve_into(&cube, &mut buf) {
            Ok(n) => n,
            Err(e) => panic!("cube {i}: solve failed: {e}"),
        };
        let sol: Vec<cubecore::Move> = buf[..n].iter().map(|&m| cubecore::Move(m)).collect();
        assert!(
            cube.apply_all(&sol).is_solved(),
            "cube {i}: solution does not solve the cube"
        );
        total += n as u64;
        max_len = max_len.max(n);
    }
    println!(
        "1,000,000 cubes: 0 invalid, 0 failed, avg length {:.3}, max length {max_len}",
        total as f64 / 1_000_000.0
    );
}
