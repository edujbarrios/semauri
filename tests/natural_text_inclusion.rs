// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-natural-inclusion-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn natural_inclusion_and_exclusion_work_with_lists_and_unicode_text() {
    let src = r#"let values be a list of "Ada", "Grace".
print values includes "Ada".
print values excludes "Alan".
print values excludes "Grace".
print "semauri" includes "aur".
print "semauri" excludes "xyz".
print "mañana" includes "ña".
print "mañana" excludes "XXX"."#;
    let r=run(src);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "true\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn natural_negative_text_predicates_cover_prefix_suffix_and_substrings() {
    let src = r#"print "alphabet" does not start with "beta".
print "alphabet" does not end with "pha".
print "alphabet" does not contain "zzz".
print "alphabet" does not include "zzz".
print "alphabet" does not start with "al".
print "alphabet" does not end with "bet".
print "alphabet" does not contain "pha".
print "alphabet" does not include "pha"."#;
    let r=run(src);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "true\ntrue\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\n");
}

#[test]
fn type_errors_match_original_typed_operators() {
    let c=Compiler::new();
    for s in [r#"print "abc" includes 3."#,
              "print 3 excludes 2.",
              "print (a list of 1, 2) does not contain true."] {
        let e=c.hir(s).unwrap_err();
        assert_eq!(e.code, "S344", "{s}: {e}");
    }
    for s in [r#"print "abc" does not start with 2."#,
              r#"print (a list of 1, 2) does not end with "x"."#] {
        let e=c.hir(s).unwrap_err();
        assert_eq!(e.code, "S391", "{s}: {e}");
    }
    let e=c.parse(r#"print "abc" does not start "a"."#).unwrap_err();
    assert_eq!(e.code, "S225");
}

#[test]
fn formatting_contextual_names_and_optimizer_behave_consistently() {
    let c=Compiler::new();
    for (src, expected) in [
        (r#"PRINT "abc" INCLUDES "a"."#,r#"print "abc" includes "a"."#),
        (r#"PRINT "abc" EXCLUDES "z"."#,r#"print "abc" excludes "z"."#),
        (r#"PRINT "abc" DOES NOT START WITH "z"."#,r#"print "abc" does not start with "z"."#),
        (r#"PRINT "abc" DOES NOT END WITH "z"."#,r#"print "abc" does not end with "z"."#),
        (r#"PRINT "abc" DOES NOT CONTAIN "z"."#,r#"print "abc" does not contain "z"."#),
        (r#"PRINT "abc" DOES NOT INCLUDE "z"."#,r#"print "abc" does not include "z"."#),
    ] { assert_eq!(c.format_source(src).unwrap(), expected); }
    c.compile(r#"let includes be 1.
let excludes be 2.
let does be 3.
assert includes is equal to 1.
assert excludes is equal to 2.
assert does is equal to 3."#, None).unwrap();
    let ast=c.parse(r#"print "abc" does not start with "z"."#).unwrap().to_json().to_string();
    assert!(ast.contains("starts_with") && ast.contains("not"), "{ast}");
    assert!(c.optimized_hir(r#"print "abc" excludes "z"."#).is_ok());
    assert_eq!(c.effect_analysis(r#"print "abc" does not contain "z"."#).unwrap().effects(),
        vec!["console_write".to_string()]);
}

#[test]
fn official_example_runs_without_new_permissions() {
    let r=run(include_str!("../examples/natural_text_inclusion.suri"));
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "Natural text inclusion OK\n");
}
