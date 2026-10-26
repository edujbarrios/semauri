// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn fixture(source: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "semauri-format-{}-{}.suri",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    path
}

#[test]
fn canonicalizes_keywords_and_domain_actions_without_changing_data() {
    let compiler = Compiler::new();
    let source = "# Keep Print Uppercase here.\nLET Message BE \"Hello, Print 🌍!\".\n/* PRINT is a comment */\nPRINT Message.\n";
    let expected = "# Keep Print Uppercase here.\nlet Message be \"Hello, Print 🌍!\".\n/* PRINT is a comment */\nprint Message.\n";
    let formatted = compiler.format_source(source).unwrap();
    assert_eq!(formatted, expected);
    assert_eq!(compiler.format_source(&formatted).unwrap(), expected);

    assert_eq!(
        compiler.runtime_plan(source).unwrap().to_json(),
        compiler.runtime_plan(&formatted).unwrap().to_json()
    );
}

#[test]
fn respects_case_sensitive_titles_labels_and_string_contents() {
    let compiler = Compiler::new();
    let original = "CREATE a WEB called Pet Shop.\nADD a BUTTON called Buy.\nSET the COLOR of the BUTTON called Buy to BLUE.\n";
    let formatted = compiler.format_source(original).unwrap();
    assert_eq!(formatted, "create a web called Pet Shop.\nadd a button called Buy.\nset the color of the button called Buy to blue.\n");
    let old = compiler.compile(original, None).unwrap();
    let new = compiler.compile(&formatted, None).unwrap();
    assert_eq!(old.output, new.output);
    assert!(new.output.unwrap().contains(">Buy</button>"));
}

#[test]
fn lowercase_and_mixed_case_core_expressions_compile_equivalently() {
    let compiler = Compiler::new();
    let source = "LET xs BE a LIST OF 3, 4.\nASSERT (xs AT 1) IS EQUAL TO 4.\nPRINT \"Case PRESERVED\".\n";
    let formatted = compiler.format_source(source).unwrap();
    assert!(formatted.contains("let xs be a list of 3, 4."));
    assert!(formatted.contains("assert (xs at 1) is equal to 4."));
    assert_eq!(
        compiler.runtime_plan(source).unwrap().to_json(),
        compiler.runtime_plan(&formatted).unwrap().to_json()
    );
}

#[test]
fn cli_supports_stdout_check_and_explicit_write() {
    let path = fixture("LET answer BE 42.\nPRINT answer.\n");
    let file = path.display().to_string();

    let displayed = run_cli(&["fmt".into(), file.clone()]);
    assert_eq!(displayed.status, 0, "{}", displayed.stderr);
    assert_eq!(displayed.stdout, "let answer be 42.\nprint answer.\n");
    assert!(fs::read_to_string(&path).unwrap().starts_with("LET"));

    let failed_check = run_cli(&["fmt".into(), "--check".into(), file.clone()]);
    assert_eq!(failed_check.status, 1);
    assert!(failed_check.stdout.contains("lowercase formatting required"));

    let updated = run_cli(&["fmt".into(), "--write".into(), file.clone()]);
    assert_eq!(updated.status, 0, "{}", updated.stderr);
    assert_eq!(fs::read_to_string(&path).unwrap(), displayed.stdout);

    let good_check = run_cli(&["fmt".into(), file.clone(), "--check".into()]);
    assert_eq!(good_check.status, 0);
    assert!(good_check.stdout.is_empty());

    fs::remove_file(path).unwrap();
}

#[test]
fn formatter_rejects_invalid_source_and_conflicting_modes() {
    let path = fixture("LET message BE \"unterminated");
    let file = path.display().to_string();
    let bad = run_cli(&["fmt".into(), file.clone()]);
    assert_eq!(bad.status, 65);
    assert!(bad.stderr.contains("S103"));

    let conflict = run_cli(&["fmt".into(), "--write".into(), "--check".into(), file]);
    assert_eq!(conflict.status, 64);

    fs::remove_file(path).unwrap();
}

#[test]
fn preserves_newlines_comments_and_unicode_offsets() {
    let compiler = Compiler::new();
    let source = "/* Unchanged uppercase: IF */\r\nLET title BE \"línea\\nPRINT\".\r\n// PRINT outside a string\r\nPRINT title.\r\n";
    let result = compiler.format_source(source).unwrap();
    assert_eq!(
        result,
        "/* Unchanged uppercase: IF */\r\nlet title be \"línea\\nPRINT\".\r\n// PRINT outside a string\r\nprint title.\r\n"
    );
}
