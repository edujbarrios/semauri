// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use semauri::{run_cli, Compiler};
use std::fs;

#[test]
fn ten_thousand_distinct_values_preserve_source_order_and_cardinality() {
    let source = "let values be range from 1 to 10000.
let unique be distinct of values.
assert length of unique is equal to 10000.
assert (unique at 0) is equal to 1.
assert (unique at 9999) is equal to 10000.
assert cardinality of values is equal to 10000.
print length of unique.";
    let temp = std::env::temp_dir().join(format!("semauri-perf-{}-unique.suri", std::process::id()));
    fs::write(&temp, source).unwrap();
    let result = run_cli(&["run".into(), "--allow".into(), "console_write".into(),
        temp.to_string_lossy().into_owned()]);
    fs::remove_file(temp).unwrap();
    assert_eq!(result.status, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "10000\n");
}

#[test]
fn filtering_partitioning_and_mapping_keep_original_values() {
    let compiler = Compiler::new();
    compiler.analyze(r#"let values be a list of "alpha", "beta", "alphabet".
let mapped be map every s in values to (uppercase of s).
let filtered be filter every s in values if (s contains "alpha").
let grouped_values be partition every s in values if (s contains "alpha").
assert length of mapped is equal to 3.
assert (mapped at 0) is equal to "ALPHA".
assert length of filtered is equal to 2.
assert (filtered at 1) is equal to "alphabet".
assert length of grouped_values is equal to 2.
assert (values at 0) is equal to "alpha".
print length of filtered."#).unwrap();
}
