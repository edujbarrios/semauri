use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-text-slice-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}
#[test]
fn unicode_slicing_respects_scalars_not_bytes() {
    let result = run(include_str!("../examples/unicode_take_drop.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "café\n");
}
#[test]
fn boundaries_and_nesting_are_deterministic() {
    let result = run(r#"let source be "ab🍵é".
assert (take 2 from (drop 1 from source)) is equal to "b🍵".
assert take 0 from source is equal to "".
assert drop 0 from source is equal to source.
assert take 100 from source is equal to source.
assert drop 100 from source is equal to ""."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}
#[test]
fn count_must_be_a_nonnegative_integer() {
    for source in ["print take minus 1 from \"abc\".", "print drop 1.5 from \"abc\"."] {
        let result = run(source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S364"), "{source}: {}", result.stderr);
    }
}
#[test]
fn type_checks_reject_wrong_operands() {
    let compiler = Compiler::new();
    for source in ["print take \"two\" from \"abc\".", "print drop 2 from true."] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S363", "{source}: {error}");
    }
}
#[test]
fn lists_and_optimizer_remain_consistent() {
    let result = run(r#"let numbers be a list of 1, 2, 3.
assert length of (take 2 from numbers) is equal to 2.
assert length of (drop 1 from numbers) is equal to 2."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    let compiler = Compiler::new();
    let optimized = compiler.optimized_hir("print take 2 from \"café\".")
        .unwrap().to_json().to_string();
    assert!(optimized.contains("ca"), "{optimized}");
    assert!(!optimized.contains("take"), "{optimized}");
    assert_eq!(compiler.effect_analysis("print drop 1 from \"abc\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
}
