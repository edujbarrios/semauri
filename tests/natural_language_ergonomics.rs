// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Natural language ergonomics without any new runtime opcodes or effects.
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let file = std::env::temp_dir().join(format!(
        "semauri-ergonomics-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::write(&file, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        file.to_string_lossy().into_owned()
    ]);
    fs::remove_file(file).unwrap();
    result
}

#[test]
fn negated_ordering_supports_strict_inclusive_and_no_spellings() {
    let output = run(r#"print 2 is not greater than 3.
print 2 is not greater than 2.
print 3 is not less than 2.
print 3 is not less than 3.
print 2 is not greater than or equal to 2.
print 3 is not less than or equal to 3.
print 1 is not greater than or equal to 2.
print 4 is not less than or equal to 3.
print 3 is no greater than 3.
print 3 is no less than 3.
print 9223372036854775807 is not less than 9223372036854775806."#);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout,
        "true\ntrue\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn negated_comparisons_preserve_type_errors_and_contextual_names() {
    let compiler = Compiler::new();
    for source in [
        r#"print "two" is not greater than 3."#,
        "print true is no less than 0.",
        r#"print 7 is not less than "seven"."#,
    ] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S314", "{source}: {error}");
        assert!(error.span.is_some());
    }
    let output = run(r#"let no be 2.
let greater be 7.
assert no is no greater than greater.
print "ok"."#);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "ok\n");
    let missing_than = compiler.parse("print 1 is not greater 2.").unwrap_err();
    assert_eq!(missing_than.code, "S222");
}

#[test]
fn through_ranges_share_inclusive_range_semantics() {
    let output = run(r#"let through be 9.
print through.
print range from 1 through 4.
print range from 3 through 1.
print range from 7 through 7.
print range from minus 2 through 1."#);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout,
        "9\n[1, 2, 3, 4]\n[3, 2, 1]\n[7]\n[-2, -1, 0, 1]\n");
    let compiler = Compiler::new();
    let error = compiler.hir("print range from 1 through true.").unwrap_err();
    assert_eq!(error.code, "S367");
    assert!(compiler.optimized_hir("print range from 1 through 4.").is_ok());
    assert_eq!(compiler.effect_analysis("print range from 1 through 4.").unwrap().effects(),
        vec!["console_write".to_string()]);
}

#[test]
fn trailing_commas_work_in_lists_and_function_calls() {
    let output = run(r#"define function combine taking x as number, y as number to (x plus y).
define function triple taking x as number, y as number, z as number to ((x plus y) plus z).
let values be a list of 1, 2, 3,.
assert length of values is equal to 3.
print values.
print invoke combine to (4, 5,).
print invoke triple to (1, 2, 3,)."#);
    assert_eq!(output.status, 0, "{}", output.stderr);
    assert_eq!(output.stdout, "[1, 2, 3]\n9\n6\n");
}

#[test]
fn malformed_comma_sequences_and_argument_arity_still_fail() {
    let compiler = Compiler::new();
    for source in [
        "print a list of 1,, 2.",
        "define function add taking x as number, y as number to (x plus y). print invoke add to (1,, 2).",
    ] {
        assert!(compiler.parse(source).is_err(), "{source}");
    }
    let error = compiler.hir(
        "define function add taking x as number, y as number to (x plus y). print invoke add to (1,)."
    ).unwrap_err();
    assert_eq!(error.code, "S374");
}

#[test]
fn formatter_ast_and_optimizer_handle_new_spellings() {
    let compiler = Compiler::new();
    for (source, expected) in [
        ("PRINT 3 IS NOT GREATER THAN 4.", "print 3 is not greater than 4."),
        ("PRINT 3 IS NO LESS THAN 2.", "print 3 is no less than 2."),
        ("PRINT RANGE FROM 1 THROUGH 4.", "print range from 1 through 4."),
        ("PRINT INVOKE combine TO (1, 2,).", "print invoke combine to (1, 2,)."),
        ("PRINT A LIST OF 1, 2,.", "print a list of 1, 2,."),
        ("LET No BE 2. PRINT No.", "let No be 2. print No."),
        ("LET Through BE 2. PRINT Through.", "let Through be 2. print Through."),
    ] {
        assert_eq!(compiler.format_source(source).unwrap(), expected, "{source}");
    }
    let ast = compiler.parse("print 1 is not greater than 3.").unwrap().to_json().to_string();
    assert!(ast.contains("less_than_or_equal"), "{ast}");
    let range = compiler.parse("print range from 1 through 3.").unwrap().to_json().to_string();
    assert!(range.contains("range"), "{range}");
    assert!(compiler.optimized_hir("print 4 is no less than 3.").is_ok());
}
