//! Reproducible benchmark harness.
//!
//!   cargo run --release -p benchkit -- --solver exa  --n 200 --seed 1 --depth 9
//!   cargo run --release -p benchkit -- --solver enso --n 100000 --seed 1
//!   cargo run --release -p benchkit -- --solver enso --n 100000 --facelets --batch
//!
//! `--depth 0` (default) means uniformly random solvable states.
//! `--facelets` times parsing the 54-char facelet string plus solving (input is the string).
//! `--batch` times all solves together (excluding generation and verification) and reports
//! the average per solve; without it, each solve is timed individually for p50/p99.
//! Every solution is verified; invalid solutions are counted and reported, never hidden.
//!
//! Owns: input generation, timing, percentile reporting and solution verification.
//! Must NOT: implement a solver or change the measured metric.

use cubecore::{from_facelets, random_alg, to_facelets, CubieCube, Move, Rng, Solver};
use std::hint::black_box;
use std::time::Instant;

fn usage() -> ! {
    eprintln!(
        "usage: benchkit --solver <exa|enso> [--n N] [--seed S] [--depth D] [--facelets] [--batch] [--version]"
    );
    std::process::exit(2)
}

fn pct(sorted: &[u128], p: f64) -> u128 {
    sorted[((sorted.len() - 1) as f64 * p) as usize]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        println!("benchkit {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let (mut name, mut n, mut seed, mut depth) = ("exa".to_string(), 100usize, 1u64, 0usize);
    let (mut facelets, mut batch) = (false, false);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--facelets" => facelets = true,
            "--batch" => batch = true,
            "--solver" | "--n" | "--seed" | "--depth" => {
                let val = args
                    .get(i + 1)
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| usage());
                match args[i].as_str() {
                    "--solver" => name = val.to_string(),
                    "--n" => n = val.parse().unwrap_or_else(|_| usage()),
                    "--seed" => seed = val.parse().unwrap_or_else(|_| usage()),
                    "--depth" => depth = val.parse().unwrap_or_else(|_| usage()),
                    _ => unreachable!(),
                }
                i += 1;
            }
            _ => usage(),
        }
        i += 1;
    }

    let t_start = Instant::now();
    let mut solver: Box<dyn Solver> = match name.as_str() {
        "exa" => Box::new(exa::Exa::new()),
        "enso" => Box::new(enso::Enso::new()),
        _ => usage(),
    };
    let startup_ns = t_start.elapsed().as_nanos();

    let mut rng = Rng::new(seed);
    let mut cubes = Vec::with_capacity(n);
    for _ in 0..n {
        cubes.push(if depth == 0 {
            CubieCube::random(&mut rng)
        } else {
            CubieCube::SOLVED.apply_all(&random_alg(&mut rng, depth))
        });
    }
    let strings: Option<Vec<String>> = if facelets {
        Some(cubes.iter().map(to_facelets).collect())
    } else {
        None
    };

    let (mut times, mut lens) = (Vec::new(), Vec::new());
    let (mut failed, mut invalid) = (0usize, 0usize);
    let mut batch_ns = 0u128;

    if batch {
        let t = Instant::now();
        let solutions: Vec<Option<Vec<Move>>> = match &strings {
            Some(ss) => ss
                .iter()
                .map(|s| {
                    let c = from_facelets(s).expect("bad generated facelet string");
                    black_box(solver.solve(black_box(&c)))
                })
                .collect(),
            None => cubes
                .iter()
                .map(|c| black_box(solver.solve(black_box(c))))
                .collect(),
        };
        batch_ns = t.elapsed().as_nanos();
        for (cube, sol) in cubes.iter().zip(solutions) {
            record(cube, sol, &mut lens, &mut failed, &mut invalid);
        }
    } else {
        for (idx, cube) in cubes.iter().enumerate() {
            let t = Instant::now();
            let sol = match &strings {
                Some(ss) => {
                    let c = from_facelets(&ss[idx]).expect("bad generated facelet string");
                    black_box(solver.solve(black_box(&c)))
                }
                None => black_box(solver.solve(black_box(cube))),
            };
            times.push(t.elapsed().as_nanos());
            record(cube, sol, &mut lens, &mut failed, &mut invalid);
        }
    }

    let solved = lens.len();
    println!(
        "solver: {}  n: {}  seed: {}  depth: {}  input: {}  timing: {}",
        solver.name(),
        n,
        seed,
        depth,
        if facelets { "facelets" } else { "cubie" },
        if batch { "batch" } else { "per-solve" }
    );
    println!("startup (tables) ns: {startup_ns}");
    println!("solved: {solved}  failed: {failed}  INVALID: {invalid}");
    if solved == 0 {
        return;
    }
    let avg_len = lens.iter().sum::<usize>() as f64 / lens.len() as f64;
    println!(
        "avg length: {avg_len:.2}  max length: {}",
        lens.iter().max().unwrap()
    );
    if batch {
        let avg_ns = batch_ns as f64 / n as f64;
        println!("time ns  batch total: {batch_ns}  avg per solve: {avg_ns:.0}");
        println!("throughput: {:.0} solves/s (single thread)", 1e9 / avg_ns);
    } else {
        times.sort_unstable();
        let avg_ns = times.iter().sum::<u128>() as f64 / times.len() as f64;
        println!(
            "time ns  avg: {avg_ns:.0}  p50: {}  p99: {}  max: {}",
            pct(&times, 0.5),
            pct(&times, 0.99),
            times.last().unwrap()
        );
        println!("throughput: {:.0} solves/s (single thread)", 1e9 / avg_ns);
    }
}

fn record(
    cube: &CubieCube,
    sol: Option<Vec<Move>>,
    lens: &mut Vec<usize>,
    failed: &mut usize,
    invalid: &mut usize,
) {
    match sol {
        Some(s) if cube.apply_all(&s).is_solved() => lens.push(s.len()),
        Some(_) => *invalid += 1,
        None => *failed += 1,
    }
}
