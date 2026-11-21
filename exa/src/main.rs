use cubecore::facelet::from_facelets;
use cubecore::moves::{format_alg, parse_alg, random_alg};
use cubecore::rng::Rng;
use cubecore::solver::Solver;
use cubecore::CubieCube;
use exa::Exa;
use std::time::Instant;

fn usage() -> ! {
    eprintln!(
        "exa: optimal (fewest-move, HTM) solver\n\n\
         usage:\n  \
         exa --scramble \"R U R' U'\"\n  \
         exa --facelets <54 chars, order U R F D L B>\n  \
         exa --depth N [--seed S]      random-walk scramble of N moves\n\n\
         note: the baseline IDA* is exact but slow beyond ~12-13 moves (see docs/EXA.md)."
    );
    std::process::exit(2)
}

fn arg(args: &[String], i: usize) -> &str {
    args.get(i).map(|s| s.as_str()).unwrap_or_else(|| usage())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let mut cube: Option<CubieCube> = None;
    let mut depth: Option<usize> = None;
    let mut seed = 1u64;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--scramble" => {
                i += 1;
                let alg = parse_alg(arg(&args, i)).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    usage()
                });
                cube = Some(CubieCube::SOLVED.apply_all(&alg));
            }
            "--facelets" => {
                i += 1;
                cube = Some(from_facelets(arg(&args, i)).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    usage()
                }));
            }
            "--depth" => {
                i += 1;
                depth = Some(arg(&args, i).parse().unwrap_or_else(|_| usage()));
            }
            "--seed" => {
                i += 1;
                seed = arg(&args, i).parse().unwrap_or_else(|_| usage());
            }
            _ => usage(),
        }
        i += 1;
    }
    if let Some(d) = depth {
        let mut rng = Rng::new(seed);
        let alg = random_alg(&mut rng, d);
        println!("scramble: {}", format_alg(&alg));
        cube = Some(CubieCube::SOLVED.apply_all(&alg));
    }
    let cube = cube.unwrap_or_else(|| usage());

    let t0 = Instant::now();
    let mut solver = Exa::new();
    println!("tables built in {:.2?}", t0.elapsed());

    let t1 = Instant::now();
    match solver.solve(&cube) {
        Some(sol) => {
            assert!(
                cube.apply_all(&sol).is_solved(),
                "BUG: solution does not solve the cube"
            );
            println!("optimal ({} moves, HTM): {}", sol.len(), format_alg(&sol));
            println!("nodes: {}  time: {:.2?}", solver.nodes, t1.elapsed());
        }
        None => {
            eprintln!("no solution found (invalid cube or search limit)");
            std::process::exit(1);
        }
    }
}
