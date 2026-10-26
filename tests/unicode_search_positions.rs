use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-positions-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn unicode_scalar_positions_and_rightmost_match() {
    let result = run(include_str!("../examples/unicode_search_positions.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Unicode positions passed\n");
}

#[test]
fn overlapping_and_nonoverlapping_matches_are_precise() {
    let result = run(r#"let found be positions of "aa" in "aaaaa".
assert length of found is equal to 2.
assert (found at 0) is equal to 0.
assert (found at 1) is equal to 2.
assert last position of "aa" in "aaaaa" is equal to 3.
assert last position of "" in "" is equal to 0.
assert last position of "no" in "abc" is equal to minus 1."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn empty_needle_and_excessive_matches_have_located_diagnostics() {
    let bad = run("print positions of \"\" in \"abc\".");
    assert_eq!(bad.status, 65, "{}", bad.stderr);
    assert!(bad.stderr.contains("S415"), "{}", bad.stderr);
    let too_many = "a".repeat(10_001);
    let source = format!("print positions of \"a\" in \"{too_many}\".");
    let err = run(&source);
    assert_eq!(err.status, 65, "{}", err.stderr);
    assert!(err.stderr.contains("S415"), "{}", err.stderr);
}

#[test]
fn operands_are_statically_checked() {
    let compiler = Compiler::new();
    for source in [
        "print positions of 2 in \"abc\".",
        "print positions of \"a\" in false.",
        "print last position of 2 in \"abc\".",
        "print last position of \"a\" in 17.",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S414", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn optimizer_formatter_and_contextual_names() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT POSITIONS OF \"é\" IN \"café\".").unwrap(),
        "print positions of \"é\" in \"café\".");
    assert_eq!(compiler.format_source("PRINT LAST POSITION OF \"a\" IN \"aba\".").unwrap(),
        "print last position of \"a\" in \"aba\".");
    let positions = compiler.optimized_hir(
        "print positions of \"a\" in \"aba\".").unwrap().to_json().to_string();
    assert!(positions.contains("list"), "{positions}");
    assert!(!positions.contains("text_positions"), "{positions}");
    let last = compiler.optimized_hir(
        "print last position of \"a\" in \"aba\".").unwrap().to_json().to_string();
    assert!(last.contains("2"), "{last}");
    assert!(!last.contains("text_last_position"), "{last}");
    let effects = compiler.effect_analysis(
        "print positions of \"a\" in \"aba\".").unwrap();
    assert_eq!(effects.effects(), vec!["console_write".to_string()]);
    let result = run(r#"let positions be "fine".
let last be "ok".
assert positions is equal to "fine".
assert last is equal to "ok"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}
