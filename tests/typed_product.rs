// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-product-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let r=run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}

#[test]
fn multiplies_exact_signed_integer_lists() {
    let r=run(r#"let data be a list of 2, 3, 4.
let total be product of data.
assert total is equal to 24.
assert (data at 0) is equal to 2.
print total.
print product of (reverse of data).
print product of a list of minus 2, 3, 4."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"24\n24\n-24\n");
}
#[test]
fn decimal_products_and_zero() {
    let r=run("print product of a list of 1.5, 2.5. print product of a list of 0, 99.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"3.75\n0\n");
}
#[test]
fn empty_typed_list_returns_one() {
    let r=run(r#"let empty be take 0 from a list of 4, 5.
print product of empty.
print length of empty."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"1\n0\n");
}
#[test]
fn no_loss_of_precision_above_f64_integer_limit() {
    let r=run("print product of a list of 9007199254740993, 1.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"9007199254740993\n");
}
#[test]
fn overflow_returns_located_error_not_silent_wrap() {
    let r=run("print product of a list of 9223372036854775807, 2.");
    assert_eq!(r.status,65,"{}",r.stderr);
    assert!(r.stderr.contains("S387"),"{}",r.stderr);
}
#[test]
fn static_type_checks_reject_wrong_list() {
    for s in ["print product of 2.","print product of a list of true, false.","print product of \"hi\"."] {
        let e=Compiler::new().hir(s).unwrap_err();
        assert_eq!(e.code,"S353","{s}: {e}");
        assert!(e.span.is_some());
    }
}
#[test]
fn contextual_variables_and_formatter() {
    let r=run("let product be 5. print product. print product of a list of 2, 3.");
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"5\n6\n");
    let c=Compiler::new();
    assert_eq!(c.format_source("PRINT PRODUCT OF a LIST OF 2, 3.").unwrap(),
        "print product of a list of 2, 3.");
    assert_eq!(c.format_source("LET Product BE 7. PRINT Product.").unwrap(),
        "let Product be 7. print Product.");
}
#[test]
fn integrates_with_functions_map_and_hir_apis() {
    let r=run(r#"define function multiply_list taking xs as list of number to (product of xs).
let numbers be a list of 2, 3, 4.
print invoke multiply_list to numbers."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"24\n");
    let c=Compiler::new();
    let source="print product of a list of 2, 3.";
    assert!(c.hir(source).unwrap().to_json().to_string().contains("product"));
    assert!(c.optimized_hir(source).is_ok());
    assert_eq!(c.effect_analysis(source).unwrap().effects(),vec!["console_write".to_owned()]);
}
