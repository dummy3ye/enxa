//! JSON report writing for benchkit. Owns only serialization, not measurement.

use std::fmt::Display;

fn json_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn opt_num<T: Display>(v: Option<T>) -> String {
    v.map(|x| x.to_string())
        .unwrap_or_else(|| "null".to_string())
}

#[derive(Clone)]
pub struct Report {
    pub solver: String,
    pub n: usize,
    pub attempted: usize,
    pub timed_out: bool,
    pub timeout_ms: Option<u64>,
    pub seed: u64,
    pub depth: usize,
    pub input: String,
    pub timing: String,
    pub cube_file: Option<String>,
    pub startup_ns: u128,
    pub solved: usize,
    pub failed: usize,
    pub invalid: usize,
    pub avg_length: f64,
    pub max_length: usize,
    pub avg_ns: Option<f64>,
    pub p50_ns: Option<u128>,
    pub p99_ns: Option<u128>,
    pub max_ns: Option<u128>,
    pub batch_ns: Option<u128>,
    pub throughput: f64,
}

impl Report {
    pub fn to_json(&self) -> String {
        format!(
            "{{\n  \"solver\": {},\n  \"n\": {},\n  \"attempted\": {},\n  \
             \"timed_out\": {},\n  \"timeout_ms\": {},\n  \"seed\": {},\n  \"depth\": {},\n  \
             \"input\": {},\n  \"timing\": {},\n  \"cube_file\": {},\n  \
             \"startup_ns\": {},\n  \"solved\": {},\n  \
             \"failed\": {},\n  \"invalid\": {},\n  \"avg_length\": {:.2},\n  \
             \"max_length\": {},\n  \"avg_ns\": {},\n  \"p50_ns\": {},\n  \"p99_ns\": {},\n  \
             \"max_ns\": {},\n  \"batch_ns\": {},\n  \"throughput\": {:.0}\n}}\n",
            json_string(&self.solver),
            self.n,
            self.attempted,
            self.timed_out,
            opt_num(self.timeout_ms),
            self.seed,
            self.depth,
            json_string(&self.input),
            json_string(&self.timing),
            self.cube_file
                .as_deref()
                .map(json_string)
                .unwrap_or_else(|| "null".to_string()),
            self.startup_ns,
            self.solved,
            self.failed,
            self.invalid,
            self.avg_length,
            self.max_length,
            opt_num(self.avg_ns),
            opt_num(self.p50_ns),
            opt_num(self.p99_ns),
            opt_num(self.max_ns),
            opt_num(self.batch_ns),
            self.throughput
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_report_has_traceable_fields_and_nulls_for_unused_timing() {
        let report = Report {
            solver: "enso-variant-a".to_string(),
            n: 2,
            attempted: 2,
            timed_out: false,
            timeout_ms: Some(1000),
            seed: 1,
            depth: 0,
            input: "facelets".to_string(),
            timing: "per-solve".to_string(),
            cube_file: Some("cubes.txt".to_string()),
            startup_ns: 12,
            solved: 2,
            failed: 0,
            invalid: 0,
            avg_length: 31.34,
            max_length: 41,
            avg_ns: Some(100.5),
            p50_ns: Some(90),
            p99_ns: Some(110),
            max_ns: Some(120),
            batch_ns: None,
            throughput: 9_950_248.0,
        };
        let json = report.to_json();
        for field in [
            "\"solver\": \"enso-variant-a\"",
            "\"n\": 2",
            "\"attempted\": 2",
            "\"timed_out\": false",
            "\"timeout_ms\": 1000",
            "\"seed\": 1",
            "\"cube_file\": \"cubes.txt\"",
            "\"p50_ns\": 90",
            "\"p99_ns\": 110",
            "\"batch_ns\": null",
            "\"avg_length\": 31.34",
        ] {
            assert!(json.contains(field), "missing {field}");
        }
        assert!(json.starts_with('{') && json.ends_with("}\n"));
        assert_eq!(json_string("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }
}
