use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-scalars-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(),
        "console_write".into(), path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn unicode_scalar_round_trip_and_lists() {
    let result = run(include_str!("../examples/unicode_characters.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "🙂\n");
}

#[test]
fn empty_text_and_unicode_scalars_work() {
    let result = run(r#"let glyphs be characters of "".
let numbers be codepoints of "".
assert length of glyphs is equal to 0.
assert length of numbers is equal to 0.
assert (character from 0) is equal to (character from 0).
assert codepoint of "é" is equal to 233.
assert codepoint of "🙂" is equal to 128578."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn invalid_scalar_values_fail_with_source_aware_diagnostics() {
    for code in [
        "print character from 55296.",
        "print character from 1114112.",
        "print character from minus 1.",
        "print character from 0.5.",
        "print codepoint of \"\".",
        "print codepoint of \"ab\".",
    ] {
        let r = run(code);
        assert_eq!(r.status, 65, "{code}: {}", r.stderr);
        assert!(r.stderr.contains("S409"), "{code}: {}", r.stderr);
    }
}

#[test]
fn collections_enforce_bounded_expansion() {
    let large = "a".repeat(10_001);
    for op in ["characters", "codepoints"] {
        let code = format!("print {op} of \"{large}\".");
        let r = run(&code);
        assert_eq!(r.status, 65, "{op}: {}", r.stderr);
        assert!(r.stderr.contains("S409"), "{op}: {}", r.stderr);
    }
}

#[test]
fn invalid_operand_types_are_caught_statically() {
    let compiler = Compiler::new();
    for code in [
        "print characters of 123.",
        "print codepoints of false.",
        "print codepoint of 7.",
        "print character from \"65\".",
    ] {
        let error = compiler.hir(code).unwrap_err();
        assert_eq!(error.code, "S408", "{code}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn optimizer_formatter_and_effects_agree() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("PRINT CHARACTER FROM 65.").unwrap(),
        "print character from 65.");
    assert_eq!(compiler.format_source("PRINT CODEPOINT OF \"A\".").unwrap(),
        "print codepoint of \"A\".");
    let folded = compiler.optimized_hir("print codepoint of \"A\".")
        .unwrap().to_json().to_string();
    assert!(folded.contains("65"), "{folded}");
    assert!(!folded.contains("codepoint\""), "{folded}");
    let list = compiler.optimized_hir("print characters of \"Aé\".")
        .unwrap().to_json().to_string();
    assert!(list.contains("list"), "{list}");
    assert_eq!(compiler.effect_analysis("print character from 65.")
        .unwrap().effects(), vec!["console_write".to_string()]);
    let result = run(r#"let characters be "good".
let codepoint be codepoint of "A".
assert characters is equal to "good".
assert codepoint is equal to 65."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
}
