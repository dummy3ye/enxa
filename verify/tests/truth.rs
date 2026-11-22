//! Heavy independent truth table for exa. Ignored by default because it BFSes the cube
//! graph to depth 6. Owns: BFS ground truth and exact-length checks. Must NOT: weaken
//! the optimality claim or use exa's heuristic as ground truth.

use cubecore::{CubieCube, Move, Solver};
use exa::Exa;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[test]
#[ignore = "heavy BFS to depth 6; run explicitly"]
fn bfs_to_depth_six_matches_exa_distances() {
    // Known HTM counts at exactly 0..=6 moves from solved. They validate the BFS itself.
    let expected = [1u64, 18, 243, 3240, 43239, 574908, 7_618_438];
    let mut dist: HashMap<CubieCube, u8> = HashMap::with_capacity(8_300_000);
    dist.insert(CubieCube::SOLVED, 0);
    let mut frontier = vec![CubieCube::SOLVED];
    let mut counts = [0u64; 7];
    counts[0] = 1;
    // Full truth for 0..=4; a deterministic prefix sample for 5 and 6 keeps exa's checks
    // bounded while the BFS still builds the complete distance table.
    let mut checks: Vec<(CubieCube, u8)> = vec![(CubieCube::SOLVED, 0)];
    let (mut sampled5, mut sampled6) = (0usize, 0usize);

    for depth in 1..=6u8 {
        let mut next = Vec::new();
        for &cube in &frontier {
            for m in Move::all() {
                let neighbour = cube.apply(m);
                if let Entry::Vacant(entry) = dist.entry(neighbour) {
                    entry.insert(depth);
                    next.push(neighbour);
                    if depth <= 4 {
                        checks.push((neighbour, depth));
                    } else if depth == 5 && sampled5 < 128 {
                        checks.push((neighbour, depth));
                        sampled5 += 1;
                    } else if depth == 6 && sampled6 < 128 {
                        checks.push((neighbour, depth));
                        sampled6 += 1;
                    }
                }
            }
        }
        counts[depth as usize] = next.len() as u64;
        eprintln!("depth {depth}: {} states", next.len());
        frontier = next;
    }

    assert_eq!(
        counts, expected,
        "BFS layer counts do not match published HTM counts"
    );
    assert_eq!(sampled5, 128);
    assert_eq!(sampled6, 128);
    drop(frontier);

    let mut exa = Exa::new();
    for (index, (cube, truth)) in checks.iter().enumerate() {
        let solution = exa
            .solve(cube)
            .expect("exa returned no solution for a reachable cube");
        assert!(
            cube.apply_all(&solution).is_solved(),
            "invalid solution at check {index}"
        );
        assert_eq!(
            solution.len(),
            *truth as usize,
            "exa returned {} moves for a {truth}-move state at check {index}",
            solution.len()
        );
    }
    eprintln!("checked {} states against BFS truth", checks.len());
}
