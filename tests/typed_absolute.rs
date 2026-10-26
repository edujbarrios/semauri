// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-absolute-{}-{}.sema", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let r=run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}

#[test]
fn preserves_i64_precision_above_two_to_the_53rd() {
    let r=run(r#"print absolute of (minus 9007199254740993).
print absolute of 9007199254740995."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"9007199254740993\n9007199254740995\n");
}
#[test]
fn handles_decimals_zeros_and_composition() {
    let r=run(r#"print absolute of (minus 2.75).
print absolute of 0.
let values be a list of minus 2, 3, minus 4.
print map every x in values to (absolute of x)."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"2.75\n0\n[2, 3, 4]\n");
}
#[test]
fn reports_signed_integer_minimum_overflow() {
    let r=run("let lowest be (minus 9223372036854775807) minus 1. print absolute of lowest.");
    assert_eq!(r.status,65,"{}",r.stderr);
    assert!(r.stderr.contains("S388"),"{}",r.stderr);
}
#[test]
fn non_numeric_operands_get_located_type_error() {
    for source in ["print absolute of true.","print absolute of \"foo\".","print absolute of a list of 1, 2."] {
        let e=Compiler::new().hir(source).unwrap_err();
        assert_eq!(e.code,"S386","{source}: {e}");
        assert!(e.span.is_some());
    }
}
#[test]
fn contextual_identifier_and_formatter_are_backwards_compatible() {
    let c=Compiler::new();
    let r=run("let absolute be 3. print absolute. print absolute of (minus 5).");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"3\n5\n");
    assert_eq!(c.format_source("PRINT ABSOLUTE OF (MINUS 2).").unwrap(),"print absolute of (minus 2).");
    assert_eq!(c.format_source("LET Absolute BE 4. PRINT Absolute.").unwrap(),"let Absolute be 4. print Absolute.");
}
#[test]
fn hir_optimizer_and_effect_analysis_include_absolute() {
    let c=Compiler::new();
    let s="print absolute of (minus 9007199254740993).";
    assert!(c.hir(s).unwrap().to_json().to_string().contains("absolute"));
    assert!(c.optimized_hir(s).is_ok());
    assert_eq!(c.effect_analysis(s).unwrap().effects(),vec!["console_write".to_owned()]);
}
