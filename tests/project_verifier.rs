use semauri::run_cli;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn init() -> (PathBuf, PathBuf) {
    let parent = std::env::temp_dir().join(format!(
        "semauri-verifier-{}-{}", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&parent).unwrap();
    let root = parent.join("project");
    let created = run_cli(&["init".into(), root.to_string_lossy().into_owned()]);
    assert_eq!(created.status, 0, "{}", created.stderr);
    (parent, root)
}

fn verify(root: &Path, options: &[&str]) -> semauri::CliResult {
    let mut args = vec!["verify".to_string(), root.to_string_lossy().into_owned()];
    args.extend(options.iter().map(|value| value.to_string()));
    run_cli(&args)
}

#[test]
fn initialized_project_passes_without_running_entry_program() {
    let (parent, root) = init();
    let result = verify(&root, &[]);
    assert_eq!(result.status, 0, "{} {}", result.stdout, result.stderr);
    assert!(result.stdout.contains("PASS project verification"), "{}", result.stdout);
    assert!(result.stdout.contains("1 passed; 0 failed"), "{}", result.stdout);
    assert!(result.stdout.contains("console_write"), "{}", result.stdout);
    // Application's print statement must not execute during verification.
    assert!(!result.stdout.lines().any(|line| line == "49"), "{}", result.stdout);
    let json = verify(&root, &["--json"]);
    assert_eq!(json.status, 0, "{}", json.stdout);
    let report: serde_json::Value = serde_json::from_str(&json.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["status"], 0);
    assert_eq!(report["source_files"], 3);
    assert_eq!(report["effects"][0], "console_write");
    assert!(report["issues"].as_array().unwrap().is_empty());
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn reports_format_failures_without_mutation() {
    let (parent, root) = init();
    let path = root.join("lib/math.suri");
    let original = fs::read_to_string(&path).unwrap();
    let changed = original.replace("define function", "DEFINE FUNCTION");
    fs::write(&path, &changed).unwrap();
    let result = verify(&root, &["--json"]);
    assert_eq!(result.status, 1, "{}", result.stdout);
    let report: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert!(report["issues"].to_string().contains("non-canonical formatting"),
        "{}", result.stdout);
    assert_eq!(fs::read_to_string(&path).unwrap(), changed);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn detects_compile_errors_and_does_not_execute_tests() {
    let (parent, root) = init();
    fs::write(root.join("lib/math.suri"),
        "define function square taking n as number to (n plus true).\n").unwrap();
    let result = verify(&root, &["--json"]);
    assert_eq!(result.status, 1, "{}", result.stdout);
    let report: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
    assert!(report["issues"].to_string().contains("math.suri"),
        "{}", result.stdout);
    assert!(report["issues"].to_string().contains("Tests were not executed"),
        "{}", result.stdout);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn detects_failing_assertions_and_refuses_unsafe_test_effects() {
    let (parent, root) = init();
    let test_path = root.join("tests/math.suri");
    fs::write(&test_path, "assert false.\n").unwrap();
    let fail = verify(&root, &[]);
    assert_eq!(fail.status, 1, "{}", fail.stdout);
    assert!(fail.stdout.contains("Test suite failed"), "{}", fail.stdout);
    fs::write(&test_path, "print \"hello\".\nassert true.\n").unwrap();
    let denied = verify(&root, &[]);
    assert_eq!(denied.status, 1, "{}", denied.stdout);
    let allowed = verify(&root, &["--allow-console"]);
    assert_eq!(allowed.status, 0, "{}", allowed.stdout);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn reports_invalid_roots_and_unsupported_options() {
    let (parent, root) = init();
    let bad_flag = verify(&root, &["--unknown"]);
    assert_eq!(bad_flag.status, 64, "{}", bad_flag.stderr);
    let no_path = run_cli(&["verify".into()]);
    assert_eq!(no_path.status, 64, "{}", no_path.stderr);
    fs::remove_file(root.join(".semauri-root")).unwrap();
    let invalid = verify(&root, &["--json"]);
    assert_eq!(invalid.status, 64, "{}", invalid.stdout);
    assert_eq!(serde_json::from_str::<serde_json::Value>(&invalid.stdout)
        .unwrap()["ok"], false);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn tests_are_required_and_excessive_sources_are_bounded() {
    let (parent, root) = init();
    fs::remove_file(root.join("tests/math.suri")).unwrap();
    let missing = verify(&root, &[]);
    assert_eq!(missing.status, 1, "{}", missing.stdout);
    assert!(missing.stdout.contains("No .suri assertion tests"), "{}", missing.stdout);
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn verifier_does_not_follow_symlinked_source_files() {
    use std::os::unix::fs::symlink;
    let (parent, root) = init();
    let outside = parent.join("outside.suri");
    fs::write(&outside, "this is not a valid program").unwrap();
    symlink(&outside, root.join("linked.suri")).unwrap();
    let ok = verify(&root, &["--json"]);
    assert_eq!(ok.status, 0, "{}", ok.stdout);
    let report: serde_json::Value = serde_json::from_str(&ok.stdout).unwrap();
    assert_eq!(report["source_files"], 3);
    fs::remove_dir_all(parent).unwrap();
}
