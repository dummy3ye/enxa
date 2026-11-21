//! Argument parsing and output for the `enso` binary.
//!
//! Owns: CLI flags, cube construction from flags, verification of the emitted solution.
//! Must NOT: implement the solver, stage coordinates, or cube conventions.

use cubecore::{format_alg, from_facelets, parse_alg, random_alg, CubieCube, Rng, Solver};
use enso::Enso;
use std::time::Instant;

fn usage() -> ! {
    eprintln!(
        "enso: fast table-descent solver (solution length is irrelevant)\n\n\
         usage:\n  \
         enso --scramble \"R U R' U'\"\n  \
         enso --facelets <54 chars, order U R F D L B>\n  \
         enso --depth N [--seed S]\n  \
         enso --version\n\n\
         note: every emitted solution is verified against the input cube."
    );
    std::process::exit(2)
}

fn arg(args: &[String], i: usize) -> &str {
    args.get(i).map(|s| s.as_str()).unwrap_or_else(|| usage())
}

pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        println!("enso {}", env!("CARGO_PKG_VERSION"));
        return;
    }
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
    let mut solver = Enso::new();
    println!("tables built in {:.2?}", solver.table_build_time());
    let _ = t0;

    let t1 = Instant::now();
    match solver.solve(&cube) {
        Some(sol) => {
            assert!(
                cube.apply_all(&sol).is_solved(),
                "BUG: solution does not solve the cube"
            );
            println!("solution ({} moves, HTM): {}", sol.len(), format_alg(&sol));
            println!("time: {:.2?}", t1.elapsed());
        }
        None => {
            eprintln!("no solution found (invalid cube or descent failure)");
            std::process::exit(1);
        }
    }
}
