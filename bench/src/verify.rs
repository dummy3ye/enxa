//! Independent solution verification for external benchmark output.

use cubecore::{from_facelets, parse_alg, CubieCube};
use std::fs;

pub struct VerifyReport {
    pub n: usize,
    pub solved: usize,
    pub failed: usize,
    pub invalid: usize,
    pub cube_file: String,
    pub solution_file: String,
}

impl VerifyReport {
    pub fn to_json(&self) -> String {
        format!(
            "{{\n  \"n\": {},\n  \"solved\": {},\n  \"failed\": {},\n  \"invalid\": {},\n  \
             \"cube_file\": \"{}\",\n  \"solution_file\": \"{}\"\n}}\n",
            self.n,
            self.solved,
            self.failed,
            self.invalid,
            self.cube_file.replace('\\', "\\\\").replace('"', "\\\""),
            self.solution_file
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
        )
    }
}

/// Verify one whitespace-separated move sequence per cube using cubecore, not the external solver.
pub fn verify_solutions_file(cube_path: &str, solution_path: &str) -> Result<VerifyReport, String> {
    let cube_text = fs::read_to_string(cube_path)
        .map_err(|e| format!("cannot read cube file {cube_path}: {e}"))?;
    let solution_text = fs::read_to_string(solution_path)
        .map_err(|e| format!("cannot read solution file {solution_path}: {e}"))?;
    let cubes: Vec<CubieCube> = cube_text
        .lines()
        .filter(|s| !s.is_empty())
        .map(from_facelets)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("invalid facelets in {cube_path}: {e}"))?;
    let solutions: Vec<&str> = solution_text.lines().filter(|s| !s.is_empty()).collect();
    if cubes.len() != solutions.len() {
        return Err(format!(
            "cube file has {} states but solution file has {} lines",
            cubes.len(),
            solutions.len()
        ));
    }
    let (mut solved, mut failed, mut invalid) = (0usize, 0usize, 0usize);
    for (cube, line) in cubes.iter().zip(solutions) {
        match parse_alg(line) {
            Ok(moves) if cube.apply_all(&moves).is_solved() => solved += 1,
            Ok(_) => invalid += 1,
            Err(_) => failed += 1,
        }
    }
    Ok(VerifyReport {
        n: cubes.len(),
        solved,
        failed,
        invalid,
        cube_file: cube_path.to_string(),
        solution_file: solution_path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cubecore::{format_alg, parse_alg, to_facelets, CubieCube};
    use std::fs;

    #[test]
    fn verifies_a_generated_solution_dump() {
        let dir = std::env::temp_dir().join(format!(
            "enxa-verify-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let cube_path = dir.join("cubes.txt");
        let sol_path = dir.join("solutions.txt");
        let alg = parse_alg("R U R' U'").unwrap();
        let scrambled = CubieCube::SOLVED.apply_all(&alg);
        let inverse: Vec<_> = alg.iter().rev().map(|m| m.inverse()).collect();
        fs::write(&cube_path, format!("{}\n", to_facelets(&scrambled))).unwrap();
        fs::write(&sol_path, format!("{}\n", format_alg(&inverse))).unwrap();
        let report =
            verify_solutions_file(cube_path.to_str().unwrap(), sol_path.to_str().unwrap()).unwrap();
        assert_eq!(report.n, 1);
        assert_eq!(report.solved, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.invalid, 0);
        fs::write(&sol_path, "not a move\n").unwrap();
        let report =
            verify_solutions_file(cube_path.to_str().unwrap(), sol_path.to_str().unwrap()).unwrap();
        assert_eq!(report.solved, 0);
        assert_eq!(report.failed, 1);
        fs::remove_dir_all(&dir).unwrap();
    }
}
