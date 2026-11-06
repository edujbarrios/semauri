// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
use semauri::{run_cli, Compiler};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
fn run(source: &str) -> semauri::CliResult {
    let path = std::env::temp_dir().join(format!(
        "semauri-integer-radix-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        path.to_string_lossy().into_owned()]);
    fs::remove_file(path).unwrap();
    result
}

#[test]
fn formats_binary_octal_hex_exactly_with_sign_and_zero() {
    let source = r#"print binary of 42.
print octal of 64.
print hex of 255.
print hexadecimal of (minus 42).
print binary of 0.
print hex of 9007199254740993.
print hex of 9223372036854775807.
let minimum be (minus 9223372036854775807) minus 1.
print hex of minimum.
print binary of (minus 5).
print hex of 42.0."#;
    let r = run(source);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "101010\n100\nff\n-2a\n0\n20000000000001\n7fffffffffffffff\n-8000000000000000\n-101\n2a\n");
}

#[test]
fn rejects_wrong_types_and_undefined_integer_conversions() {
    let c = Compiler::new();
    for s in ["print binary of true.", r#"print octal of "text"."#,
              "print hex of (a list of 1, 2)."] {
        let err = c.hir(s).unwrap_err();
        assert_eq!(err.code, "S468", "{s}: {err}");
        assert!(err.span.is_some());
    }
    for s in ["print binary of 2.5.", "print hex of (minus 0.5).",
              r#"print octal of (parse number from "9223372036854775808.0")."#] {
        let r=run(s);
        assert_eq!(r.status, 65, "{s}: {}", r.stderr);
        assert!(r.stderr.contains("S469"), "{s}: {}", r.stderr);
    }
}

#[test]
fn typed_list_transformations_formatter_and_compiler_passes() {
    let c=Compiler::new();
    let r=run(r#"let numbers be a list of 10, 15, 255.
let codes be map every n in numbers to (hex of n).
print length of codes.
print codes at 0.
print codes at 2.
let binary be "name".
print binary."#);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "3\na\nff\nname\n");
    for name in ["binary", "octal", "hex", "hexadecimal"] {
        let source = format!("PRINT {} OF 42.", name.to_uppercase());
        assert_eq!(c.format_source(&source).unwrap(), format!("print {name} of 42."));
        let ast=c.parse(&format!("print {name} of 42.")).unwrap().to_json().to_string();
        assert!(ast.contains(if name == "hexadecimal" { "hex" } else { name }), "{ast}");
        assert!(c.optimized_hir(&format!("print {name} of 42.")).is_ok());
        assert_eq!(c.effect_analysis(&format!("print {name} of 42.")).unwrap().effects(),
            vec!["console_write".to_string()]);
    }
}

#[test]
fn official_example_executes() {
    let r=run(include_str!("../examples/integer_radices.suri"));
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "Integer radix functions OK\n");
}
