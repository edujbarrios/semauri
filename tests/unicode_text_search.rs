use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-text-search-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn unicode_positions_are_scalars_not_bytes() {
    let result = run(include_str!("../examples/unicode_text_search.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Unicode text search passed\n");
}

#[test]
fn overlapping_matches_are_not_counted_twice() {
    let result = run(r#"assert occurrences of "aa" in "aaaaa" is equal to 2.
assert occurrences of "AA" in "aaaaa" is equal to 0.
assert position of "aa" in "baaa" is equal to 1.
assert position of "foo" in "bar" is equal to minus 1.
assert position of "" in "bar" is equal to 0."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn rejects_invalid_operands_at_type_check() {
    let compiler = Compiler::new();
    for source in [
        "print occurrences of 2 in \"abc\".",
        "print occurrences of \"a\" in false.",
        "print position of 2 in 3.",
        "print position of \"a\" in 1.",
    ] {
        let err = compiler.hir(source).unwrap_err();
        assert_eq!(err.code, "S402", "{source}: {err}");
        assert!(err.span.is_some());
    }
}

#[test]
fn rejects_empty_occurrence_terms_with_a_located_error() {
    let result = run("print occurrences of \"\" in \"abc\".");
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S403"), "{}", result.stderr);
}

#[test]
fn folding_formatter_and_contextual_names_preserved() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT OCCURRENCES OF \"a\" IN \"aba\".").unwrap(),
        "print occurrences of \"a\" in \"aba\".");
    assert_eq!(compiler.format_source("PRINT POSITION OF \"é\" IN \"café\".").unwrap(),
        "print position of \"é\" in \"café\".");
    let h = compiler.optimized_hir("print position of \"é\" in \"café\".")
        .unwrap().to_json().to_string();
    assert!(h.contains("3"), "{h}");
    assert!(!h.contains("text_position"), "{h}");
    let result = run(r#"let occurrences be "text".
let position be position of "e" in occurrences.
assert position is equal to 1."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(compiler.effect_analysis("print occurrences of \"a\" in \"aba\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
}
