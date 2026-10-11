// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! Both translated beginner courses must stay in sync with the real compiler.

use semauri::run_cli;
use std::fs;
use std::path::{Path, PathBuf};

const ES_CONSOLE_LESSONS: [(&str, &str); 9] = [
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

const EN_CONSOLE_LESSONS: [(&str, &str); 9] = [
    ("01_hello_world", "Hello from Semauri\n2026\n"),
    ("02_variables_and_types", "Alex\n18\ntrue\n"),
    ("03_arithmetic", "20\n30\n2\n"),
    ("04_booleans", "true\ntrue\n"),
    ("05_if_otherwise", "Passed\n"),
    ("06_unless_else_if", "Great\n"),
    ("07_lists", "[10, 20, 30]\n[red, green, blue]\n"),
    ("08_loops", "1\n2\n3\nDone\n"),
    ("09_scopes", "inside\noutside\n"),
];

fn course_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("learn_to_code")
}

fn lesson_path(language: &str, lesson: &str) -> PathBuf {
    course_root().join(language).join(lesson)
}

fn check(path: &Path) {
    let result = run_cli(&["check".into(), path.display().to_string()]);
    assert_eq!(result.status, 0, "{}: {}", path.display(), result.stderr);
    assert_eq!(result.stdout, "OK\n");
}

#[test]
fn both_courses_check_and_run_all_console_lessons() {
    for (language, lessons) in [("es", &ES_CONSOLE_LESSONS), ("en", &EN_CONSOLE_LESSONS)] {
        for &(name, expected) in lessons {
            let dir = lesson_path(language, name);
            let readme = fs::read_to_string(dir.join("README.md")).unwrap();
            assert!(readme.contains("main.sema"), "{language}/{name}: missing link");

            let path = dir.join("main.sema");
            check(&path);

            // Console access is explicitly approved; no filesystem or network
            // effects are requested by any of these beginner examples.
            let result = run_cli(&[
                "run".into(),
                "--allow".into(),
                "console_write".into(),
                path.display().to_string(),
            ]);
            assert_eq!(result.status, 0, "{language}/{name}: {}", result.stderr);
            assert_eq!(result.stdout, expected, "{language}/{name}");
        }
    }
}

#[test]
fn both_courses_check_and_build_web_and_schema_lessons() {
    for (language, web_name, schema_name, button, first_field, second_field) in [
        ("es", "10_crea_una_web", "11_esquemas", "Empezar", "Nombre", "Edad"),
        ("en", "10_build_a_website", "11_schemas", "Start", "Name", "Age"),
    ] {
        let web = lesson_path(language, web_name).join("main.sema");
        let schema = lesson_path(language, schema_name).join("main.sema");
        for path in [&web, &schema] {
            assert!(path.parent().unwrap().join("README.md").is_file());
            check(path);
        }

        let html = run_cli(&["build".into(), web.display().to_string()]);
        assert_eq!(html.status, 0, "{language}: {}", html.stderr);
        assert!(html.stdout.contains(&format!("{button}</button>")), "{language}");
        assert!(html.stdout.contains("color: blue"), "{language}");

        let json = run_cli(&["build".into(), schema.display().to_string()]);
        assert_eq!(json.status, 0, "{language}: {}", json.stderr);
        for field in [first_field, second_field] {
            assert!(json.stdout.contains(&format!("\"{field}\"")), "{language}/{field}");
        }
        assert!(json.stdout.contains("\"integer\""), "{language}");
    }
}

#[test]
fn bilingual_language_selector_and_lesson_links_are_complete() {
    let root = course_root();
    let landing = fs::read_to_string(root.join("README.md")).unwrap();
    assert!(landing.contains("./es/README.md"));
    assert!(landing.contains("./en/README.md"));

    for (language, console_lessons, extra_lessons) in [
        ("es", &ES_CONSOLE_LESSONS, ["10_crea_una_web", "11_esquemas"]),
        ("en", &EN_CONSOLE_LESSONS, ["10_build_a_website", "11_schemas"]),
    ] {
        let index = fs::read_to_string(root.join(language).join("README.md")).unwrap();
        assert!(index.contains("../README.md"), "{language}: no language selector");
        assert!(index.contains(if language == "es" { "../en/" } else { "../es/" }));
        for &(name, _) in console_lessons {
            assert!(index.contains(&format!("./{name}/README.md")), "{language}/{name}");
        }
        for name in extra_lessons {
            assert!(index.contains(&format!("./{name}/README.md")), "{language}/{name}");
        }
    }
    assert!(!root.join("01_hola_mundo").exists(), "Spanish lessons must live under es/");
}
