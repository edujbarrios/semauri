// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-partition-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let result = run_cli(&[
        "run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned(),
    ]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn partitions_in_stable_order_with_typed_inner_lists() {
    let r = run(r#"let xs be a list of 9, 2, 8, 1, 7.
let groups be partition every x in xs if (x is greater than 5).
let positives be groups at 0.
let negatives be groups at 1.
assert length of groups is equal to 2.
assert length of positives is equal to 3.
assert length of negatives is equal to 2.
assert first of positives is equal to 9.
assert first of negatives is equal to 2.
assert length of xs is equal to 5.
print positives.
print negatives."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[9, 8, 7]\n[2, 1]\n");
}

#[test]
fn empty_and_all_true_or_false_inputs_produce_typed_buckets() {
    let r=run(r#"let xs be take 0 from a list of 2, 4.
let empty_groups be partition every x in xs if true.
assert length of (empty_groups at 0) is equal to 0.
assert length of (empty_groups at 1) is equal to 0.
let all_true be partition every n in (a list of 1, 2) if true.
let all_false be partition every n in (a list of 1, 2) if false.
assert length of (all_true at 0) is equal to 2.
assert length of (all_true at 1) is equal to 0.
assert length of (all_false at 0) is equal to 0.
assert length of (all_false at 1) is equal to 2.
print empty_groups."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[[], []]\n");
}

#[test]
fn partitions_string_boolean_and_nested_lists() {
    let r=run(r#"let words be a list of "a", "long", "b", "wide".
let groups be partition every word in words if ((length of word) is greater than 1).
print groups.
let bools be partition every flag in (a list of true, false, true) if flag.
assert length of (bools at 0) is equal to 2.
assert length of (bools at 1) is equal to 1.
let inner be a list of (a list of 1), (a list of 1, 2).
let nested be partition every xs in inner if ((length of xs) is greater than 1).
assert length of (nested at 0) is equal to 1."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert!(r.stdout.starts_with("[[long, wide], [a, b]]"),"{}",r.stdout);
}

#[test]
fn composes_with_user_functions_and_outer_binding() {
    let r=run(r#"let cutoff be 5.
define function above taking n as number, min_value as number to (n is greater than min_value).
let nums be range from 1 to 8.
let groups be partition every n in nums if (invoke above to (n, cutoff)).
print groups at 0.
print groups at 1."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"[6, 7, 8]\n[1, 2, 3, 4, 5]\n");
}

#[test]
fn reports_invalid_list_and_predicate_types_at_hir_time() {
    for (source, code) in [
        ("print partition every x in 3 if true.", "S365"),
        ("print partition every x in (a list of 2, 3) if (x plus 1).", "S366"),
    ] {
        let e=Compiler::new().hir(source).unwrap_err();
        assert_eq!(e.code,code,"{source}: {e}");
        assert!(e.span.is_some());
    }
}

#[test]
fn iterator_does_not_escape_and_keyword_is_contextual() {
    let e=Compiler::new().hir("let grouped be partition every x in (a list of 2, 3) if true. print x.").unwrap_err();
    assert_eq!(e.code,"S312");
    let r=run(r#"let partition be 3.
print partition.
let values be a list of 1, 2.
print length of (partition every n in values if (n is greater than partition))."#);
    assert_eq!(r.status,0,"{}",r.stderr);
    assert_eq!(r.stdout,"3\n2\n");
}

#[test]
fn ast_hir_optimizer_formatter_and_effects_traverse_partition() {
    let c=Compiler::new();
    let source="print partition every n in (a list of 1, 3) if (n is greater than 2).";
    assert!(c.parse(source).unwrap().to_json().to_string().contains("partition"));
    assert!(c.hir(source).unwrap().to_json().to_string().contains("partition"));
    assert!(c.optimized_hir(source).unwrap().to_json().to_string().contains("partition"));
    assert_eq!(c.effect_analysis(source).unwrap().effects(),vec!["console_write".to_owned()]);
    assert_eq!(c.format_source("PRINT PARTITION EVERY n IN (a LIST OF 1, 3) IF (n IS GREATER THAN 2).").unwrap(),
        "print partition every n in (a list of 1, 3) if (n is greater than 2).");
    assert_eq!(c.format_source("LET Partition BE 2. PRINT Partition.").unwrap(),
        "let Partition be 2. print Partition.");
}
