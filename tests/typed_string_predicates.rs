// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-predicates-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn checks_unicode_prefix_suffix_and_empty_patterns() {
    let source = r#"let message be "Café Ñandú".
assert message starts with "Café".
assert message ends with "Ñandú".
assert message starts with "".
assert message ends with "".
assert not (message starts with "café").
assert not (message ends with "ñandú").
assert "" starts with "".
assert "" ends with "".
print "ok"."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "ok\n");
}

#[test]
fn identifiers_remain_usable_and_predicates_compose() {
    let source = r#"let starts be "report.json".
let ends be ".json".
assert (starts ends with ends) and (starts starts with "report").
assert ends starts with "."."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
}

#[test]
fn mismatched_operands_have_stable_diagnostic() {
    let compiler = Compiler::new();
    for source in [
        "assert 1 starts with \"1\".",
        "assert \"one\" ends with 2.",
        "let numbers be a list of 1, 2. assert numbers starts with \"one\".",
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S391", "{source}: {error}");
        assert!(error.span.is_some());
    }
}

#[test]
fn formatter_and_optimizer_recognize_contextual_predicates() {
    let compiler = Compiler::new();
    assert_eq!(compiler.format_source("ASSERT \"abc\" STARTS WITH \"a\".").unwrap(),
        "assert \"abc\" starts with \"a\".");
    assert_eq!(compiler.format_source("ASSERT \"abc\" ENDS WITH \"c\".").unwrap(),
        "assert \"abc\" ends with \"c\".");
    let optimized = compiler.optimized_hir("print \"hello\" starts with \"he\".")
        .unwrap().to_json().to_string();
    assert!(optimized.contains("true"), "{optimized}");
    assert_eq!(compiler.effect_analysis("print \"hello\" ends with \"o\".")
        .unwrap().effects(), vec!["console_write".to_string()]);
}
