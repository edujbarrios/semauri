use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let file = std::env::temp_dir().join(format!("semauri-remove-substrings-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&file, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        file.to_string_lossy().into_owned()]);
    fs::remove_file(file).unwrap();
    result
}

#[test]
fn removes_unicode_and_overlapping_literal_matches() {
    let result = run(include_str!("../examples/remove_substrings.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "caf caf\n");
}

#[test]
fn rejects_empty_search_text_at_evaluation() {
    for code in [
        "print remove all of \"\" from \"hello\".",
        "print remove all of \"\" from \"\".",
    ] {
        let result = run(code);
        assert_eq!(result.status, 65, "{code}: {}", result.stderr);
        assert!(result.stderr.contains("S413"), "{code}: {}", result.stderr);
    }
}

#[test]
fn requires_two_string_operands_statically() {
    let compiler = Compiler::new();
    for code in [
        "print remove all of 4 from \"abcd\".",
        "print remove all of \"a\" from true.",
        "print remove all of false from 7.",
    ] {
        let error = compiler.hir(code).unwrap_err();
        assert_eq!(error.code, "S412", "{code}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn folds_literals_and_preserves_reserved_action_keywords() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT REMOVE ALL OF \"a\" FROM \"banana\".").unwrap(),
        "print remove all of \"a\" from \"banana\".");
    let h = compiler.optimized_hir("print remove all of \"a\" from \"banana\".")
        .unwrap().to_json().to_string();
    assert!(h.contains("bnn"), "{h}");
    assert!(!h.contains("text_remove_all"), "{h}");
    assert_eq!(compiler.effect_analysis("print remove all of \"x\" from \"abc\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
    let result = run(r#"let removable be "ok".
assert removable is equal to "ok"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}
