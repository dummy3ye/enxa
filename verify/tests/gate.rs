//! The explicitly invoked enso correctness gate.
//! Owns: the ignored 1,000,000-cube validation run. Must NOT: run by default in `cargo test`.

use cubecore::CubieCube;
use cubecore::Rng;
use enso::{Enso, MAX_SOLUTION_LEN};

/// The M3 correctness gate: 1,000,000 seeded random cubes, every solution verified.
/// Run explicitly with `cargo test -p verify --release --test gate -- --ignored --nocapture`.
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
