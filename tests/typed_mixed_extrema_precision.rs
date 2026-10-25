// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-extrema-precision-{}-{}.sema",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let r = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    r
}

#[test]
fn mixed_float_and_integer_extrema_preserve_53_bit_boundary() {
    let r=run(r#"let values be a list of 9007199254740992.0, 9007199254740993.
print minimum of values.
print maximum of values.
let reverse_values be reverse of values.
print minimum of reverse_values.
print maximum of reverse_values."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"9007199254740992.0\n9007199254740993\n9007199254740992.0\n9007199254740993\n");
}

#[test]
fn signed_large_boundaries_are_ordered_exactly() {
    let r=run(r#"let values be a list of minus 9007199254740993, minus 9007199254740992.0.
print minimum of values.
print maximum of values.
let others be a list of 9223372036854775807, 9223372036854775808.0.
print minimum of others.
print maximum of others."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"-9007199254740993\n-9007199254740992.0\n9223372036854775807\n9223372036854776000.0\n");
}

#[test]
fn integer_only_extrema_remain_exact() {
    let r=run(r#"let xs be a list of 9007199254740995, 9007199254740993, 9007199254740994.
print minimum of xs.
print maximum of xs."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"9007199254740993\n9007199254740995\n");
}

#[test]
fn extrema_consistent_with_sorted_and_median() {
    let r=run(r#"let xs be a list of 9007199254740993, 9007199254740992.0, 9007199254740995.
let sorted_xs be sorted of xs.
assert (minimum of xs) is equal to (first of sorted_xs).
assert (maximum of xs) is equal to (last of sorted_xs).
print median of xs."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"9007199254740993\n");
}

#[test]
fn compiler_effects_and_hir_still_see_pure_extrema() {
    let c=Compiler::new();
    let src="let xs be a list of 9007199254740992.0, 9007199254740993. print maximum of xs.";
    assert!(c.hir(src).unwrap().to_json().to_string().contains("maximum"));
    assert_eq!(c.effect_analysis(src).unwrap().effects(), vec!["console_write".to_owned()]);
}
