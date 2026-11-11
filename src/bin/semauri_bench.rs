// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Reproducible, Python-free end-to-end Semauri benchmarking utilities.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const CASES: [(&str, &str); 5] = [
    ("hello", "Hello, Semauri!\n"),
    ("map_reduce", "333383335000\n"),
    ("filter_count", "4666\n"),
    ("sort", "5000.5\n"),
    ("text", "10000\n"),
];
const OLS_EXPECTED: &str = concat!(
    "Linear regression (ordinary least squares)\n",
    "Training observations:\n5\n",
    "Learned slope:\n2.0\n",
    "Learned intercept:\n1\n",
    "Prediction for x=6:\n13\n"
);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Timing {
    median_ms: f64,
    min_ms: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Report {
    label: String,
    os: String,
    cpu: String,
    n: usize,
    warmups: usize,
    results: BTreeMap<String, BTreeMap<String, Timing>>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn default_binary() -> PathBuf {
    root().join("target").join("release").join(if cfg!(windows) { "semauri.exe" } else { "semauri" })
}
fn positive(value: &str, option: &str, min: usize) -> Result<usize, String> {
    let n = value.parse::<usize>().map_err(|_| format!("{option} must be an integer"))?;
    if n < min { return Err(format!("{option} must be at least {min}")); }
    Ok(n)
}
fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 { sorted[mid] } else { (sorted[mid - 1] + sorted[mid]) / 2.0 }
}
fn timing(values: &[f64]) -> Timing {
    Timing {
        median_ms: (median(values) * 1000.0).round() / 1000.0,
        min_ms: (values.iter().copied().fold(f64::INFINITY, f64::min) * 1000.0).round() / 1000.0,
    }
}
fn cpu_name() -> String {
    if let Ok(info) = fs::read_to_string("/proc/cpuinfo") {
        for line in info.lines() {
            if let Some((name, value)) = line.split_once(':') {
                if name.trim() == "model name" { return value.trim().to_string(); }
            }
        }
    }
    env::consts::ARCH.to_string()
}
fn measured(binary: &Path, source: &Path, expected: &str) -> Result<f64, String> {
    let start = Instant::now();
    let output = Command::new(binary)
        .args(["run", "--allow", "console_write"])
        .arg(source)
        .output()
        .map_err(|error| format!("Unable to launch {}: {error}", binary.display()))?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    if !output.status.success() || output.stdout != expected.as_bytes() || !output.stderr.is_empty() {
        return Err(format!(
            "{} on {}: exit={}, stdout={:?}, stderr={:?}, expected={expected:?}",
            binary.display(), source.display(), output.status,
            String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(elapsed)
}
fn run_suite(args: &[String]) -> Result<(), String> {
    let n = match args.first() { Some(value) => positive(value, "samples", 1)?, None => 50 };
    if args.len() > 1 { return Err("Usage: semauri_bench suite [SAMPLES]".into()); }
    let warmups = positive(&env::var("BENCH_WARMUPS").unwrap_or_else(|_| "3".into()), "warmups", 0)?;
    let binary = env::var_os("SEMAURI_BIN").map(PathBuf::from).unwrap_or_else(default_binary);
    let mut results = BTreeMap::new();
    println!("| benchmark | Semauri median (ms) | Semauri min (ms) |");
    println!("|---|---:|---:|");
    for (case, expected) in CASES {
        let source = root().join("benches").join(format!("{case}.suri"));
        let mut measurements = Vec::with_capacity(n);
        for iteration in 0..(n + warmups) {
            let elapsed = measured(&binary, &source, expected)?;
            if iteration >= warmups { measurements.push(elapsed); }
        }
        let value = timing(&measurements);
        println!("| {case} | {:.3} | {:.3} |", value.median_ms, value.min_ms);
        results.insert(case.into(), BTreeMap::from([("Semauri".into(), value)]));
    }
    let report = Report {
        label: env::var("BENCH_LABEL").unwrap_or_else(|_| "candidate".into()),
        os: format!("{}-{}", env::consts::OS, env::consts::ARCH),
        cpu: cpu_name(), n, warmups, results,
    };
    println!("Machine: {}, CPU: {}. {n} timed runs after {warmups} warmups.", report.os, report.cpu);
    if let Ok(output) = env::var("BENCH_RESULTS") {
        fs::write(&output, format!("{}\n", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?))
            .map_err(|e| format!("{output}: {e}"))?;
    }
    Ok(())
}
fn read_report(path: &str) -> Result<Report, String> {
    let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}
fn check_same_machine(reports: &[Report]) -> Result<(), String> {
    let first = reports.first().ok_or("No reports to compare")?;
    if reports.iter().any(|r| (r.cpu.as_str(), r.os.as_str(), r.n, r.warmups)
        != (first.cpu.as_str(), first.os.as_str(), first.n, first.warmups)) {
        return Err("Cannot compare results from different machines or sample configurations".into());
    }
    Ok(())
}
fn semauri_median(report: &Report, case: &str) -> Result<f64, String> {
    report.results.get(case).and_then(|r| r.get("Semauri"))
        .map(|r| r.median_ms)
        .ok_or_else(|| format!("Missing Semauri benchmark: {case}"))
}
fn compare(args: &[String]) -> Result<(), String> {
    if args.len() != 2 { return Err("Usage: semauri_bench compare BEFORE.json AFTER.json".into()); }
    let reports = args.iter().map(|a| read_report(a)).collect::<Result<Vec<_>, _>>()?;
    check_same_machine(&reports)?;
    println!("CPU: {}\nOS: {}\nRuns: {}, warmups: {}", reports[0].cpu, reports[0].os, reports[0].n, reports[0].warmups);
    println!("| benchmark | before (ms) | after (ms) | speedup |");
    println!("|---|---:|---:|---:|");
    for case in reports[0].results.keys() {
        let before = semauri_median(&reports[0], case)?;
        let after = semauri_median(&reports[1], case)?;
        println!("| {case} | {before:.3} | {after:.3} | {:.2}x |", before / after);
    }
    Ok(())
}
fn phases(args: &[String]) -> Result<(), String> {
    if args.len() != 4 { return Err("Usage: semauri_bench phases PHASE0.json PHASE1.json PHASE2.json PHASE3.json".into()); }
    let reports = args.iter().map(|a| read_report(a)).collect::<Result<Vec<_>, _>>()?;
    check_same_machine(&reports)?;
    println!("Compiler phases on: {} / {}", reports[0].cpu, reports[0].os);
    println!("Samples: {}, warmups: {}", reports[0].n, reports[0].warmups);
    println!("| benchmark | phase0 (ms) | phase1 (ms) | phase2 (ms) | phase3 (ms) | total speedup |");
    println!("|---|---:|---:|---:|---:|---:|");
    for case in reports[0].results.keys() {
        let times = reports.iter().map(|r| semauri_median(r, case)).collect::<Result<Vec<_>, _>>()?;
        println!("| {case} | {:.3} | {:.3} | {:.3} | {:.3} | {:.2}x |",
            times[0], times[1], times[2], times[3], times[0] / times[3]);
    }
    for pair in reports.windows(2) {
        println!("Phase transition: {} -> {}", pair[0].label, pair[1].label);
        for case in pair[0].results.keys() {
            println!("  {case}: {:.2}x", semauri_median(&pair[0], case)? / semauri_median(&pair[1], case)?);
        }
    }
    Ok(())
}
struct TempWorkloads(PathBuf);
impl TempWorkloads {
    fn new() -> Result<Self, String> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
        let path = env::temp_dir().join(format!("semauri-benchmark-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
    fn source(&self, name: &str, text: &str) -> Result<PathBuf, String> {
        let path = self.0.join(format!("{name}.suri"));
        fs::write(&path, text).map_err(|e| e.to_string())?;
        Ok(path)
    }
}
impl Drop for TempWorkloads {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}
fn delta(args: &[String]) -> Result<(), String> {
    if args.len() < 2 { return Err("Usage: semauri_bench delta BEFORE_BINARY AFTER_BINARY [--runs N] [--warmups N]".into()); }
    let before = Path::new(&args[0]); let after = Path::new(&args[1]);
    let mut runs = 30; let mut warmups = 3;
    let mut options = args[2..].chunks_exact(2);
    for chunk in &mut options {
        match chunk[0].as_str() {
            "--runs" => runs = positive(&chunk[1], "--runs", 1)?,
            "--warmups" => warmups = positive(&chunk[1], "--warmups", 0)?,
            bad => return Err(format!("Unknown delta option: {bad}")),
        }
    }
    if !options.remainder().is_empty() { return Err("Delta options require a value".into()); }
    let temp = TempWorkloads::new()?;
    let cases = [
        ("distinct_unique", "let values be range from 1 to 10000.\nprint cardinality of values.\n", "10000\n"),
        ("distinct_repeated", "let values be range from 1 to 10000.\nlet residues be map every n in values to (n modulo 100).\nprint count distinct of residues.\n", "100\n"),
        ("map_reduce", include_str!("../../benches/map_reduce.suri"), "333383335000\n"),
        ("text", include_str!("../../benches/text.suri"), "10000\n"),
    ];
    println!("Paired CLI timings: {runs} samples, {warmups} warmups; process startup + compilation + execution.");
    println!("| Workload | Base median (ms) | Candidate median (ms) | Speedup |");
    println!("|---|---:|---:|---:|");
    for (case, source, expected) in cases {
        let file = temp.source(case, source)?;
        let mut values = [Vec::new(), Vec::new()];
        for iteration in 0..(runs + warmups) {
            let order = if iteration % 2 == 0 { [0, 1] } else { [1, 0] };
            for side in order {
                let binary = if side == 0 { before } else { after };
                let elapsed = measured(binary, &file, expected)?;
                if iteration >= warmups { values[side].push(elapsed); }
            }
        }
        let old = median(&values[0]); let new = median(&values[1]);
        println!("| {case} | {old:.3} | {new:.3} | {:.2}x |", old / new);
    }
    Ok(())
}
fn linear(args: &[String]) -> Result<(), String> {
    let mut binary = default_binary(); let mut source = root().join("examples/linear_regression.suri");
    let mut samples = 101;
    let mut options = args.chunks_exact(2);
    for pair in &mut options {
        match pair[0].as_str() {
            "--binary" => binary = PathBuf::from(&pair[1]),
            "--source" => source = PathBuf::from(&pair[1]),
            "--samples" => samples = positive(&pair[1], "--samples", 3)?,
            bad => return Err(format!("Unknown linear option: {bad}")),
        }
    }
    if !options.remainder().is_empty() { return Err("Linear options require a value".into()); }
    let first = measured(&binary, &source, OLS_EXPECTED)?;
    for _ in 0..3 { measured(&binary, &source, OLS_EXPECTED)?; }
    let mut values = Vec::with_capacity(samples);
    for _ in 0..samples { values.push(measured(&binary, &source, OLS_EXPECTED)?); }
    let value = median(&values);
    println!("$ semauri run --allow console_write examples/linear_regression.suri");
    print!("{OLS_EXPECTED}");
    println!("First execution (wall): {first:.3} ms");
    println!("Median CLI execution (wall): {value:.3} ms");
    println!("Measurement: {samples} runs, 3 warmups, release build, build excluded");
    println!("Machine: {} {}", env::consts::OS, env::consts::ARCH);
    if let Ok(path) = env::var("GITHUB_STEP_SUMMARY") {
        use std::io::Write;
        let mut file = fs::OpenOptions::new().append(true).create(true).open(path).map_err(|e| e.to_string())?;
        writeln!(file, "## Semauri: linear regression from scratch\n\n**Actual program output**\n\n\x60\x60\x60text\n{OLS_EXPECTED}\x60\x60\x60\n\n**Measured median CLI wall time:** {value:.3} ms ({samples} runs, 3 warmups).\n\n**Scope:** process startup + parse/type-check + program execution; shared runner, not a universal speed guarantee.").map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("suite") => run_suite(&args[1..]),
        Some("compare") => compare(&args[1..]),
        Some("phases") => phases(&args[1..]),
        Some("delta") => delta(&args[1..]),
        Some("linear") => linear(&args[1..]),
        _ => Err("Usage: semauri_bench <suite|compare|phases|delta|linear> [OPTIONS]".into()),
    };
    if let Err(error) = result { eprintln!("{error}"); std::process::exit(2); }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn medians_and_minimums_are_deterministic() {
        assert_eq!(median(&[4.0, 1.0, 3.0]), 3.0);
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), 2.5);
        assert_eq!(timing(&[2.01234, 3.0]).min_ms, 2.012);
    }
    #[test]
    fn comparisons_require_identical_hardware_and_sampling() {
        let empty = Report { label: "candidate".into(), os: "linux".into(), cpu: "cpu".into(), n: 50, warmups: 3, results: BTreeMap::new() };
        assert!(check_same_machine(&[empty.clone(), empty.clone()]).is_ok());
        let mut other = empty.clone();
        other.n = 10;
        assert!(check_same_machine(&[empty, other]).is_err());
    }
}
