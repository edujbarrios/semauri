// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);

fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!("semauri-natural-{}-{}.suri",
        std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn joins_strings_with_correct_precedence_and_chaining() {
    let result = run(r#"let name be "Ada".
print "Hello, " joined with name.
print "one" joined with "-" joined with "two".
print "a" plus "b" joined with "c" plus "d".
let joined be "saved".
print joined joined with "!"."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Hello, Ada\none-two\nabcd\nsaved!\n");
}

#[test]
fn rejects_non_string_joins_at_type_check_time() {
    let compiler = Compiler::new();
    for source in [r#"print 1 joined with "x"."#, r#"print "x" joined with true."#,
                   "print 3 joined with 4."] {
        let error = compiler.hir(source).unwrap_err();
        assert_eq!(error.code, "S343", "{source}: {error}");
    }
}

#[test]
fn when_and_different_from_support_conditional_chains() {
    let result = run(r#"when 3 is different from 4:
  print "first".
otherwise when true:
  print "second".
otherwise:
  print "third".
end.
when false:
  print "wrong".
otherwise when 5 is different from 5:
  print "also wrong".
otherwise:
  print "last".
end."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "first\nlast\n");
}

#[test]
fn inequality_keeps_strict_typing() {
    let err = Compiler::new().hir(r#"when 1 is different from "1": print "no". end."#).unwrap_err();
    assert_eq!(err.code, "S315");
}

#[test]
fn ast_optimizer_formatter_and_effects_support_new_syntax() {
    let c = Compiler::new();
    let ast = c.parse(r#"print "A" joined with "B"."#).unwrap().to_json().to_string();
    assert!(ast.contains("concat"), "{ast}");
    let optimized = c.optimized_hir(r#"print "A" joined with "B"."#).unwrap().to_json().to_string();
    assert!(optimized.contains("AB"), "{optimized}");
    assert_eq!(c.effect_analysis(r#"when true: print "Hi". end."#).unwrap().effects(),
        vec!["console_write".to_string()]);
    assert_eq!(c.format_source(r#"WHEN true: PRINT "Hi" JOINED WITH "there". END."#).unwrap(),
        r#"when true: print "Hi" joined with "there". end."#);
}

#[test]
fn example_compiles_and_runs() {
    let result = run(include_str!("../examples/natural_syntax.suri"));
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Hello, Ada\n");
}

#[test]
fn elif_then_and_do_support_readable_control_flow() {
    let source = r#"let numbers be a list of 1, 2, 3.
when 2 is in numbers then:
    print "first".
elif 3 not in numbers then:
    print "second".
else:
    print "third".
end.
if false then:
    print "never".
elif 2 in numbers then:
    print "elif".
else:
    print "not reached".
end.
for each n in numbers do:
    assert n is greater than 0.
end."#;
    let result = run(source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "first\nelif\n");
}

#[test]
fn existing_else_if_and_for_syntax_stays_compatible() {
    let result = run(r#"if false:
    print "never".
otherwise if true:
    print "old spelling".
end.
let values be a list of 2, 4.
for n in values:
    assert n is greater than 0.
end."#);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "old spelling\n");
}
