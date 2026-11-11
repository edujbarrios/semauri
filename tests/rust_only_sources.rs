// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Keep all project-owned internal sources, tooling and tests Rust-only.
use std::fs;
use std::path::Path;

fn scan(directory: &Path) {
    for item in fs::read_dir(directory).unwrap() {
        let path = item.unwrap().path();
        if path.is_dir() {
            // Compilation artifacts are not repository-owned source files.
            if path.file_name().is_some_and(|name| name == "target") { continue; }
            scan(&path);
        } else {
            assert_ne!(path.extension().and_then(|e| e.to_str()), Some("py"),
                "Python source is not allowed: {}", path.display());
            assert_ne!(path.file_name().and_then(|e| e.to_str()), Some("pyproject.toml"),
                "The Semauri Jupyter adapter must be installed from its Cargo project");
        }
    }
}
#[test]
fn owned_integrations_and_tooling_have_no_python_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for folder in ["benches", "scripts", "integrations/jupyter"] {
        scan(&root.join(folder));
    }
}
