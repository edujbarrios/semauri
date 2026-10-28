use semauri::{run_cli, Compiler};
use std::{fs, sync::atomic::{AtomicU64, Ordering}};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-reciprocal-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn returns_fractional_inverses_for_positive_and_negative_numbers() {
    let r = run(r#"assert (reciprocal of 2) is equal to 0.5.
assert (reciprocal of (minus 4)) is equal to (minus 0.25).
assert (reciprocal of 0.25) is equal to 4.0.
assert (reciprocal of 1) is equal to 1.
let quarter be reciprocal of 4.
assert quarter is greater than 0.
assert quarter is less than 1."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
}
#[test]
fn division_by_zero_produces_located_diagnostics() {
    for source in ["print reciprocal of 0.", "print reciprocal of 0.0.",
                   "print reciprocal of (minus 0.0)."] {
        let r = run(source);
        assert_eq!(r.status, 65, "{source}: {}", r.stderr);
        assert!(r.stderr.contains("S440"), "{source}: {}", r.stderr);
        assert!(r.stderr.contains("1:"), "{source}: {}", r.stderr);
    }
}
#[test]
fn rejects_non_numeric_inputs_at_type_checking() {
    let compiler = Compiler::new();
    for source in ["print reciprocal of true.", "print reciprocal of \"x\".",
                   "print reciprocal of a list of 1, 2."] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S439", "{source}: {err}");
        assert!(err.span.is_some());
    }
}
#[test]
fn stays_contextual_pure_and_composable() {
    let r = run(r#"define function inverse taking n as number to (reciprocal of n).
let entries be a list of 2, 4, 8.
let inverses be map every n in entries to (invoke inverse to n).
assert (inverses at 0) is equal to 0.5.
assert (inverses at 1) is equal to 0.25.
assert (inverses at 2) is equal to 0.125.
let reciprocal be 9.
print reciprocal."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "9\n");
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT RECIPROCAL OF 4.").unwrap(),
               "print reciprocal of 4.");
    assert!(compiler.hir("print reciprocal of 2.").unwrap().to_json().to_string().contains("reciprocal"));
    assert!(compiler.optimized_hir("print reciprocal of 2.").is_ok());
    assert_eq!(compiler.effect_analysis("print reciprocal of 2.").unwrap().effects(),
               vec!["console_write".to_owned()]);
}
