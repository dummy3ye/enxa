//! Reproducible benchmark harness.
//!
//!   cargo run --release -p benchkit -- --solver exa  --n 200 --seed 1 --depth 9
//!   cargo run --release -p benchkit -- --solver enso --n 100000 --seed 1
//!   cargo run --release -p benchkit -- --solver enso --n 100000 --facelets --batch
//!   JSON, cube dumps, overhead floor and external verification flags are documented in
//!   `docs/BENCHMARKING.md`.
//!
//! `--depth 0` (default) means uniformly random solvable states.
//! `--facelets` times parsing the 54-char facelet string plus solving (input is the string).
//! `--batch` times all solves together (excluding generation and verification) and reports
//! the average per solve; without it, each solve is timed individually for p50/p99.
//! Every solution is verified; invalid solutions are counted and reported, never hidden.
//!
//! Owns: input generation, timing, percentile reporting, JSON/raw dumps and verification.
//! Must NOT: implement a solver or change the measured metric.

mod report;
mod verify;

use cubecore::{from_facelets, random_alg, to_facelets, CubieCube, Move, Rng, Solver};
use enso::{Enso, MAX_SOLUTION_LEN};
use report::Report;
use std::hint::black_box;
use std::time::Instant;

/// Bench-local extension so enso can be timed on facelet strings without parsing to a cube.
/// Default preserves the original parse-then-solve path for every other solver.
trait FaceletSolver: Solver {
    fn solve_facelets(&mut self, s: &str) -> Option<Vec<Move>> {
        let cube = from_facelets(s).expect("bad generated facelet string");
        self.solve(&cube)
    }
}

impl FaceletSolver for exa::Exa {}

impl FaceletSolver for Enso {
    fn solve_facelets(&mut self, s: &str) -> Option<Vec<Move>> {
        let mut buf = [0u8; MAX_SOLUTION_LEN];
        let n = self.solve_facelets_into(s, &mut buf).ok()?;
        Some(buf[..n].iter().map(|&m| Move(m)).collect())
    }
}

fn usage() -> ! {
    eprintln!(
        "usage: benchkit --solver <exa|enso> [--n N] [--seed S] [--depth D] [--facelets] \
         [--batch] [--json PATH] [--dump-cubes PATH] [--cube-file PATH] [--overhead] \
         [--verify-file PATH] [--version]"
    );
    std::process::exit(2)
}

fn pct(sorted: &[u128], p: f64) -> u128 {
    sorted[((sorted.len() - 1) as f64 * p) as usize]
}

type TimingStats = (
    Option<f64>,
    Option<u128>,
    Option<u128>,
    Option<u128>,
    Option<u128>,
    f64,
);

fn write_json(path: Option<&String>, json: String) {
    if let Some(path) = path {
        if let Err(e) = std::fs::write(path, json) {
            eprintln!("cannot write JSON report {path}: {e}");
            std::process::exit(1);
        }
    }
}

fn load_cubes(
    cube_path: Option<&String>,
    n: usize,
    seed: u64,
    depth: usize,
    facelets: &mut bool,
    loaded_depth: &mut usize,
) -> Vec<CubieCube> {
    let Some(path) = cube_path else {
        let mut rng = Rng::new(seed);
        return (0..n)
            .map(|_| {
                if depth == 0 {
                    CubieCube::random(&mut rng)
                } else {
                    CubieCube::SOLVED.apply_all(&random_alg(&mut rng, depth))
                }
            })
            .collect();
    };
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read cube file {path}: {e}");
        std::process::exit(1)
    });
    let loaded: Result<Vec<CubieCube>, String> = text
        .lines()
        .filter(|s| !s.is_empty())
        .map(from_facelets)
        .collect();
    let loaded = loaded.unwrap_or_else(|e| {
        eprintln!("invalid facelets in {path}: {e}");
        std::process::exit(1)
    });
    if loaded.len() != n {
        eprintln!("cube file {path} has {} states, expected {n}", loaded.len());
        std::process::exit(1)
    }
    *facelets = true;
    *loaded_depth = 0;
    loaded
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

fn print_common(report: &Report) {
    println!(
        "solver: {}  n: {}  seed: {}  depth: {}  input: {}  timing: {}",
        report.solver, report.n, report.seed, report.depth, report.input, report.timing
    );
    println!("startup (tables) ns: {}", report.startup_ns);
    println!(
        "solved: {}  failed: {}  INVALID: {}",
        report.solved, report.failed, report.invalid
    );
    println!(
        "avg length: {:.2}  max length: {}",
        report.avg_length, report.max_length
    );
}

fn timing_stats(times: &mut [u128], batch_ns: u128, n: usize, batch: bool) -> TimingStats {
    if batch {
        let avg = if n == 0 {
            0.0
        } else {
            batch_ns as f64 / n as f64
        };
        println!("time ns  batch total: {batch_ns}  avg per solve: {avg:.0}");
        let throughput = if avg > 0.0 { 1e9 / avg } else { 0.0 };
        if avg > 0.0 {
            println!("throughput: {throughput:.0} solves/s (single thread)");
        }
        return (Some(avg), None, None, None, Some(batch_ns), throughput);
    }
    if times.is_empty() {
        return (None, None, None, None, None, 0.0);
    }
    times.sort_unstable();
    let avg = times.iter().sum::<u128>() as f64 / times.len() as f64;
    let p50 = pct(times, 0.5);
    let p99 = pct(times, 0.99);
    let max = *times.last().unwrap();
    println!("time ns  avg: {avg:.0}  p50: {p50}  p99: {p99}  max: {max}");
    let throughput = if avg > 0.0 { 1e9 / avg } else { 0.0 };
    if avg > 0.0 {
        println!("throughput: {throughput:.0} solves/s (single thread)");
    }
    (Some(avg), Some(p50), Some(p99), Some(max), None, throughput)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        println!("benchkit {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let (mut name, mut n, mut seed, mut depth) = ("exa".to_string(), 100usize, 1u64, 0usize);
    let (mut facelets, mut batch, mut overhead) = (false, false, false);
    let (mut json_path, mut dump_path, mut cube_path, mut verify_path) = (None, None, None, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--facelets" => facelets = true,
            "--batch" => batch = true,
            "--overhead" => overhead = true,
            "--solver" | "--n" | "--seed" | "--depth" | "--json" | "--dump-cubes"
            | "--cube-file" | "--verify-file" => {
                let val = args
                    .get(i + 1)
                    .map(String::as_str)
                    .unwrap_or_else(|| usage());
                match args[i].as_str() {
                    "--solver" => name = val.to_string(),
                    "--n" => n = val.parse().unwrap_or_else(|_| usage()),
                    "--seed" => seed = val.parse().unwrap_or_else(|_| usage()),
                    "--depth" => depth = val.parse().unwrap_or_else(|_| usage()),
                    "--json" => json_path = Some(val.to_string()),
                    "--dump-cubes" => dump_path = Some(val.to_string()),
                    "--cube-file" => cube_path = Some(val.to_string()),
                    "--verify-file" => verify_path = Some(val.to_string()),
                    _ => unreachable!(),
                }
                i += 1;
            }
            _ => usage(),
        }
        i += 1;
    }

    if let Some(solution_path) = verify_path.as_deref() {
        let Some(cube_file) = cube_path.as_deref() else {
            usage()
        };
        let report = verify::verify_solutions_file(cube_file, solution_path).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1)
        });
        println!(
            "verified {} solutions: {} valid, {} unparseable, {} invalid",
            report.n, report.solved, report.failed, report.invalid
        );
        write_json(json_path.as_ref(), report.to_json());
        return;
    }

    let t_start = Instant::now();
    let mut solver: Box<dyn FaceletSolver> = match name.as_str() {
        "exa" => Box::new(exa::Exa::new()),
        "enso" => Box::new(enso::Enso::new()),
        _ => usage(),
    };
    let startup_ns = t_start.elapsed().as_nanos();

    let cube_file_report = cube_path.clone();
    let cubes = load_cubes(
        cube_path.as_ref(),
        n,
        seed,
        depth,
        &mut facelets,
        &mut depth,
    );
    let mut strings: Option<Vec<String>> = if facelets {
        Some(cubes.iter().map(to_facelets).collect())
    } else {
        None
    };

    if let Some(path) = dump_path {
        let lines = strings
            .take()
            .unwrap_or_else(|| cubes.iter().map(to_facelets).collect());
        let mut text = lines.join("\n");
        text.push('\n');
        if let Err(e) = std::fs::write(&path, text) {
            eprintln!("cannot write cube dump {path}: {e}");
            std::process::exit(1)
        }
        println!("dumped {} cubes to {}", n, path);
        return;
    }

    let (mut times, mut lens) = (Vec::new(), Vec::new());
    let (mut failed, mut invalid) = (0usize, 0usize);
    let mut batch_ns = 0u128;

    if overhead {
        if batch {
            let t = Instant::now();
            match &strings {
                Some(ss) => {
                    for s in ss {
                        black_box(s.as_str());
                    }
                }
                None => {
                    for cube in &cubes {
                        black_box(cube);
                    }
                }
            }
            batch_ns = t.elapsed().as_nanos();
        } else {
            for idx in 0..n {
                let t = Instant::now();
                match &strings {
                    Some(ss) => {
                        black_box(ss[idx].as_str());
                    }
                    None => {
                        black_box(&cubes[idx]);
                    }
                }
                times.push(t.elapsed().as_nanos());
            }
        }
    } else if batch {
        let t = Instant::now();
        let solutions: Vec<Option<Vec<Move>>> = match &strings {
            Some(ss) => ss
                .iter()
                .map(|s| black_box(solver.solve_facelets(black_box(s.as_str()))))
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
                Some(ss) => black_box(solver.solve_facelets(black_box(ss[idx].as_str()))),
                None => black_box(solver.solve(black_box(cube))),
            };
            times.push(t.elapsed().as_nanos());
            record(cube, sol, &mut lens, &mut failed, &mut invalid);
        }
    }

    let input = if facelets { "facelets" } else { "cubie" };
    let timing = if batch { "batch" } else { "per-solve" };
    let solver_name = if overhead {
        "overhead-loop".to_string()
    } else {
        solver.name().to_string()
    };
    let solved = lens.len();
    let avg_length = if lens.is_empty() {
        0.0
    } else {
        lens.iter().sum::<usize>() as f64 / lens.len() as f64
    };
    let max_length = lens.iter().max().copied().unwrap_or(0);
    let (avg_ns, p50_ns, p99_ns, max_ns, report_batch_ns, throughput) =
        timing_stats(&mut times, batch_ns, n, batch);

    let report = Report {
        solver: solver_name,
        n,
        seed,
        depth,
        input: input.to_string(),
        timing: timing.to_string(),
        cube_file: cube_file_report,
        startup_ns,
        solved,
        failed,
        invalid,
        avg_length,
        max_length,
        avg_ns,
        p50_ns,
        p99_ns,
        max_ns,
        batch_ns: report_batch_ns,
        throughput,
    };
    print_common(&report);
    write_json(json_path.as_ref(), report.to_json());
}
