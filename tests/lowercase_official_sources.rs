// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! Public code examples ship with canonical lowercase language commands.
//! The bilingual lessons keep their original natural-language text and
//! user-visible values; only Semauri source syntax is normalized.

use semauri::Compiler;
use std::fs;
use std::path::Path;

fn visit(path: &Path, compiler: &Compiler, visited: &mut usize) {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            visit(&entry.unwrap().path(), compiler, visited);
        }
        return;
    }
    if path.extension().is_none_or(|extension| extension != "sema") {
        return;
    }
    let source = fs::read_to_string(path).unwrap();
    let formatted = compiler.format_source(&source)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    assert_eq!(
        source, formatted,
        "{}: example is not in canonical lowercase style; run 'semauri fmt {} --write'",
        path.display(), path.display(),
    );
    compiler.parse(&source)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    *visited += 1;
}

#[test]
fn official_examples_and_bilingual_learning_programs_are_lowercase() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let compiler = Compiler::new();
    let mut visited = 0;
    for folder in ["examples", "learn_to_code/en", "learn_to_code/es"] {
        visit(&root.join(folder), &compiler, &mut visited);
    }
    assert!(visited >= 41, "Expected all 19 examples and 22 lessons; found {visited}");
}
