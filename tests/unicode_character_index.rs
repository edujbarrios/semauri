use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let file = std::env::temp_dir().join(format!("semauri-character-at-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&file, source).unwrap();
    let r = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        file.to_string_lossy().into_owned()]);
    fs::remove_file(file).unwrap();
    r
}

#[test]
fn unicode_indexing_counts_scalars_not_utf8_bytes() {
    let result = run(include_str!("../examples/unicode_character_index.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "🙂\n");
}

#[test]
fn handles_ascii_boundaries_and_nested_expressions() {
    let source = r#"let text be "café".
assert (character at 0 in text) is equal to "c".
assert (character at 3 in text) is equal to "é".
assert (codepoint of (character at 3 in text)) is equal to 233."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn reports_invalid_numeric_values_with_a_source_span() {
    for expr in [
        "character at minus 1 in \"abc\"",
        "character at 0.5 in \"abc\"",
        "character at 3 in \"abc\"",
        "character at 0 in \"\"",
    ] {
        let result = run(&format!("print {expr}."));
        assert_eq!(result.status, 65, "{expr}: {}", result.stderr);
        assert!(result.stderr.contains("S411"), "{expr}: {}", result.stderr);
    }
}

#[test]
fn type_mismatch_is_caught_in_hir() {
    let compiler = Compiler::new();
    for expr in ["character at true in \"abc\"",
        "character at 0 in false", "character at \"1\" in \"abc\""] {
        let error = compiler.hir(&format!("print {expr}.")).unwrap_err();
        assert_eq!(error.code, "S410", "{expr}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn formatter_folding_effects_and_identifiers() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT CHARACTER AT 1 IN \"café\".").unwrap(),
        "print character at 1 in \"café\".");
    let optimized = compiler.optimized_hir("print character at 1 in \"café\".")
        .unwrap().to_json().to_string();
    assert!(optimized.contains("a"), "{optimized}");
    assert!(!optimized.contains("character_at"), "{optimized}");
    assert_eq!(compiler.effect_analysis("print character at 1 in \"café\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
    let result = run(r#"let character be "ok".
assert character is equal to "ok"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}
