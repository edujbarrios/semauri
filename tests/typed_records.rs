// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Immutable, structurally typed records and checked field lookup.
use semauri::{run_cli, Compiler};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(1);

fn source_file(source: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "semauri-records-{}-{}.suri", std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&path, source).unwrap();
    path
}

fn command(verb: &str, source: &str) -> semauri::CliResult {
    let file = source_file(source);
    let result = if verb == "run" {
        run_cli(&["run".into(), "--allow".into(), "console_write".into(),
            file.to_string_lossy().into_owned()])
    } else {
        run_cli(&[verb.into(), file.to_string_lossy().into_owned()])
    };
    fs::remove_file(file).unwrap();
    result
}

#[test]
fn records_infer_field_types_and_print_json() {
    let source = r#"let person be record of ("name": "Ada", "age": 37, "active": true).
assert (field "age" of person) is equal to 37.
assert (field "active" of person) is equal to true.
print field "name" of person.
print person.
"#;
    for verb in ["check", "hir", "ast", "effects"] {
        let result = command(verb, source);
        assert_eq!(result.status, 0, "{verb}: {}", result.stderr);
    }
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Ada\n{\"active\":true,\"age\":37,\"name\":\"Ada\"}\n");
    let hir = Compiler::new().hir(source).unwrap().to_json();
    assert!(hir.to_string().contains("\"record\""));
}

#[test]
fn nested_and_empty_records_work() {
    let source = r#"let person be a record of ("name": "Ada",).
let wrapper be record of ("person": person, "empty": record of ()).
assert (field "name" of field "person" of wrapper) is equal to "Ada".
print field "empty" of wrapper.
"#;
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "{}\n");
}

#[test]
fn record_field_names_are_case_sensitive_and_types_are_static() {
    for (source, expected) in [
        (r#"let person be record of ("age": 37).
print field "Age" of person.
"#, "S419"),
        (r#"let person be record of ("age": 37).
print field "age" of 42.
"#, "S419"),
        (r#"let person be record of ("x": 1, "x": 2).
print person.
"#, "S418"),
        (r#"let person be record of ("": 1).
print person.
"#, "S418"),
        (r#"let person be record of ("x" 1).
print person.
"#, "S418"),
    ] {
        let result = command("check", source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains(expected), "{source}: {}", result.stderr);
    }
}

#[test]
fn records_can_appear_in_pure_function_bodies_and_list_values() {
    let source = r#"define function build_profile taking n as number to record of ("value": n, "double": (n times 2)).
let result be invoke build_profile to 6.
assert (field "double" of result) is equal to 12.
let people be a list of record of ("name": "Ada"), record of ("name": "Grace").
print field "value" of result.
"#;
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "6\n");
}

#[test]
fn records_do_not_add_runtime_effects() {
    let source = r#"let input be record of ("n": 7).
assert field "n" of input is equal to 7.
"#;
    let effects = command("effects", source);
    assert_eq!(effects.status, 0, "{}", effects.stderr);
    assert!(!effects.stdout.contains("filesystem_write"), "{}", effects.stdout);
    assert!(!effects.stdout.contains("network"), "{}", effects.stdout);
}

#[test]
fn record_field_limits_are_checked() {
    let fields = (0..33).map(|i| format!("\"f{i}\": {i}")).collect::<Vec<_>>().join(", ");
    let source = format!("let r be record of ({fields}).\nprint r.\n");
    let result = command("check", &source);
    assert_eq!(result.status, 65, "{}", result.stderr);
    assert!(result.stderr.contains("S418"), "{}", result.stderr);
}

#[test]
fn structural_record_parameters_are_typechecked() {
    let source = r#"define function display_age taking person as record of ("name": string, "age": number) to field "age" of person.
let person be record of ("age": 37, "name": "Ada").
print invoke display_age to person.
"#;
    let check = command("check", source);
    assert_eq!(check.status, 0, "{}", check.stderr);
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "37\n");
}

#[test]
fn nested_record_parameter_types_work_with_lists() {
    let source = r#"define function extract taking profile as record of ("address": record of ("city": string), "tags": list of string) to field "city" of field "address" of profile.
let entry be record of ("tags": a list of "a", "b", "address": record of ("city": "Barcelona")).
print invoke extract to entry.
"#;
    let result = command("run", source);
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "Barcelona\n");
}

#[test]
fn structural_record_parameters_reject_incompatible_fields() {
    let prefix = r#"define function display_age taking person as record of ("name": string, "age": number) to field "age" of person.
"#;
    for value in [
        r#"record of ("name": "Ada", "age": "old")"#,
        r#"record of ("name": "Ada")"#,
        r#"record of ("name": "Ada", "age": 37, "extra": true)"#,
    ] {
        let source = format!("{prefix}let person be {value}.\nprint invoke display_age to person.\n");
        let result = command("check", &source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S370"), "{source}: {}", result.stderr);
    }
}

#[test]
fn invalid_record_parameter_schemas_report_s418() {
    for schema in [
        r#"record of ("name": string, "name": number)"#,
        r#"record of ("": string)"#,
        r#"record of ("age" number)"#,
    ] {
        let source = format!("define function get_value taking value as {schema} to value.\nprint 1.\n");
        let result = command("check", &source);
        assert_eq!(result.status, 65, "{source}: {}", result.stderr);
        assert!(result.stderr.contains("S418"), "{source}: {}", result.stderr);
    }
}
