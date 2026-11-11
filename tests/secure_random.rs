// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FILE: AtomicU64 = AtomicU64::new(1);

fn run_source(source: &str, allow_random: bool, allow_console: bool) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-secure-random-{}-{}.suri",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, source).unwrap();
    let mut args = vec!["run".into(), path.to_string_lossy().into_owned()];
    if allow_random {
        args.extend(["--allow".into(), "secure_random".into()]);
    }
    if allow_console {
        args.extend(["--allow".into(), "console_write".into()]);
    }
    let result = run_cli(&args);
    fs::remove_file(&path).unwrap();
    result
}

#[test]
fn password_generation_uses_a_secure_default_alphabet() {
    let source = r#"
let generated_password be password of 32 characters.
print generated_password.
"#;
    let compiler = Compiler::new();
    let effect_analysis = compiler.effect_analysis(source).unwrap();
    assert_eq!(
        effect_analysis.effects(),
        vec!["console_write".to_string(), "secure_random".to_string()]
    );
    let plan = compiler.runtime_plan(source).unwrap();
    assert!(plan.dataflow_graph().is_ok());
    assert_eq!(plan.operations[0].domain, "random");
    assert_eq!(plan.operations[0].name, "password");
    assert_eq!(plan.operations[0].effects, vec!["secure_random"]);
    for _ in 0..8 {
        let outcome = run_source(source, true, true);
        assert_eq!(outcome.status, 0, "{}", outcome.stderr);
        let password = outcome.stdout.trim_end_matches('\n');
        assert_eq!(password.chars().count(), 32);
        assert!(password.chars().all(|character|
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}?"
                .contains(character)));
    }
}

#[test]
fn custom_alphabet_is_unicode_aware_and_bounded() {
    let source = r#"
let token be secure random string of 17 characters from "🍏ñ🍋".
print token.
"#;
    for _ in 0..6 {
        let result = run_source(source, true, true);
        assert_eq!(result.status, 0, "{}", result.stderr);
        let token = result.stdout.trim_end_matches('\n');
        assert_eq!(token.chars().count(), 17);
        assert!(token.chars().all(|ch| ['🍏', 'ñ', '🍋'].contains(&ch)));
    }
}

#[test]
fn secure_integer_uses_inclusive_bounds_and_character_pick_is_typed() {
    let source = r#"
let die be draw random integer from 1 to 6.
let chosen be pick random character from "ABC".
let constant be draw random integer from 42 to 42.
print die.
print chosen.
print constant.
"#;
    for _ in 0..20 {
        let outcome = run_source(source, true, true);
        assert_eq!(outcome.status, 0, "{}", outcome.stderr);
        let lines = outcome.stdout.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 3, "{}", outcome.stdout);
        let number = lines[0].parse::<i64>().unwrap();
        assert!((1..=6).contains(&number));
        assert!(["A", "B", "C"].contains(&lines[1]));
        assert_eq!(lines[2], "42");
    }
}

#[test]
fn large_integer_ranges_preserve_signed_i64_precision() {
    let result = run_source(
        r#"let rolled_value be draw random integer from 9007199254740993 to 9007199254740995. print rolled_value."#,
        true, true,
    );
    assert_eq!(result.status, 0, "{}", result.stderr);
    let value = result.stdout.trim().parse::<i64>().unwrap();
    assert!((9007199254740993..=9007199254740995).contains(&value));
}

#[test]
fn rejects_length_and_alphabet_errors_without_secret_echo() {
    let invalid = [
        "let token be password of 0 characters. print token.",
        "let token be password of 4097 characters. print token.",
        "let token be password of (minus 2) characters. print token.",
        "let token be password of 1.5 characters. print token.",
        r#"let token be secure random string of 8 characters from "". print token."#,
        r#"let token be secure random string of 8 characters from "abca". print token."#,
        r#"let token be pick random character from "aa". print token."#,
        "let rolled_value be draw random integer from 20 to 10. print rolled_value.",
        "let rolled_value be draw random integer from 1.5 to 2. print rolled_value.",
    ];
    for source in invalid {
        let outcome = run_source(source, true, true);
        assert_eq!(outcome.status, 70, "{source}: {}", outcome.stderr);
        assert!(outcome.stderr.contains("S503"), "{source}: {}", outcome.stderr);
        assert!(outcome.stdout.is_empty(), "{source}: {}", outcome.stdout);
    }
}

#[test]
fn type_checker_rejects_incorrect_random_arguments() {
    let compiler = Compiler::new();
    for source in [
        r#"let output be password of "16" characters."#,
        r#"let output be secure random string of 20 characters from 123."#,
        r#"let output be draw random integer from "1" to 9."#,
        r#"let output be pick random character from 10."#,
    ] {
        assert!(compiler.hir(source).is_err(), "{source}");
    }
}

#[test]
fn random_effect_cannot_be_executed_without_permission() {
    let source = r#"let secret be password of 24 characters. print secret."#;
    let without_random = run_source(source, false, true);
    assert_eq!(without_random.status, 65);
    assert!(without_random.stderr.contains("S334"), "{}", without_random.stderr);
    assert!(without_random.stdout.is_empty());
    let without_console = run_source(source, true, false);
    assert_eq!(without_console.status, 65);
    assert!(without_console.stderr.contains("S334"), "{}", without_console.stderr);
    assert!(without_console.stdout.is_empty());
    let allowed = run_source(source, true, true);
    assert_eq!(allowed.status, 0, "{}", allowed.stderr);
    assert_eq!(allowed.stdout.trim().chars().count(), 24);
}

#[test]
fn formatter_and_checked_example_stay_valid() {
    let source = include_str!("../examples/secure_password.suri");
    let compiler = Compiler::new();
    compiler.hir(source).unwrap();
    let first = compiler.format_source(source).unwrap();
    let second = compiler.format_source(&first).unwrap();
    assert_eq!(first, second);
    let outcome = run_source(source, true, true);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let lines = outcome.stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].chars().count(), 20);
    assert!(lines[1].chars().all(|ch| "abcdef0123456789".contains(ch)));
    assert_eq!(lines[1].chars().count(), 32);
    assert!((100000..=999999).contains(&lines[2].parse::<i64>().unwrap()));
    assert!(["A", "B", "C", "D"].contains(&lines[3]));
}
