// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! Conformance tests keep the beginner course aligned with the released compiler.

use semauri::run_cli;
use std::fs;
use std::path::PathBuf;

const CONSOLE_LESSONS: [(&str, &str); 9] = [
    ("01_hola_mundo", "Hola desde Semauri\n2026\n"),
    ("02_variables_y_tipos", "Ana\n18\ntrue\n"),
    ("03_aritmetica", "20\n30\n2\n"),
    ("04_booleanos", "true\ntrue\n"),
    ("05_if_otherwise", "Aprobado\n"),
    ("06_unless_y_else_if", "Notable\n"),
    ("07_listas", "[10, 20, 30]\n[red, green, blue]\n"),
    ("08_bucles", "1\n2\n3\nFin\n"),
    ("09_ambitos", "dentro\nfuera\n"),
];

fn lesson_path(lesson: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("learn_to_code")
        .join(lesson)
}

fn check(path: &std::path::Path) {
    let args = vec!["check".into(), path.display().to_string()];
    let result = run_cli(&args);
    assert_eq!(result.status, 0, "{}: {}", path.display(), result.stderr);
    assert_eq!(result.stdout, "OK\n");
}

#[test]
fn every_console_lesson_checks_and_prints_expected_output() {
    for (name, expected) in CONSOLE_LESSONS {
        let dir = lesson_path(name);
        let readme = fs::read_to_string(dir.join("README.md")).unwrap();
        assert!(readme.contains("main.sema"), "{name} has no example link");
        let path = dir.join("main.sema");
        check(&path);

        // No effects happen without an explicit permission. All course programs
        // use console output only, not filesystem or network operations.
        let args = vec![
            "run".into(),
            "--allow".into(),
            "console_write".into(),
            path.display().to_string(),
        ];
        let result = run_cli(&args);
        assert_eq!(result.status, 0, "{name}: {}", result.stderr);
        assert_eq!(result.stdout, expected, "{name}");
    }
}

#[test]
fn build_lessons_check_and_produce_domain_outputs() {
    let web = lesson_path("10_crea_una_web").join("main.sema");
    let schema = lesson_path("11_esquemas").join("main.sema");

    for path in [&web, &schema] {
        assert!(path.parent().unwrap().join("README.md").is_file());
        check(path);
    }

    let html = run_cli(&["build".into(), web.display().to_string()]);
    assert_eq!(html.status, 0, "{}", html.stderr);
    assert!(html.stdout.contains("Empezar</button>"));
    assert!(html.stdout.contains("color: blue"));

    let json = run_cli(&["build".into(), schema.display().to_string()]);
    assert_eq!(json.status, 0, "{}", json.stderr);
    assert!(json.stdout.contains("\"Nombre\""));
    assert!(json.stdout.contains("\"Edad\""));
    assert!(json.stdout.contains("\"integer\""));
}

#[test]
fn course_index_links_to_every_lesson() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("learn_to_code");
    let index = fs::read_to_string(root.join("README.md")).unwrap();
    for (name, _) in CONSOLE_LESSONS {
        assert!(index.contains(name), "missing lesson in index: {name}");
    }
    for name in ["10_crea_una_web", "11_esquemas"] {
        assert!(index.contains(name), "missing lesson in index: {name}");
    }
}
