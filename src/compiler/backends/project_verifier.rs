// A deliberate, non-mutating project gate for local development and CI.
// It checks all sources and runs only the restricted assertion test runner;
// main.suri is compiled/inspected, never executed.
const MAX_VERIFY_FILES: usize = 256;
const MAX_VERIFY_BYTES: u64 = 4 * 1024 * 1024;

fn scan_verify_sources(directory: &Path, collected: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(directory)?;
    if metadata.file_type().is_symlink() { return Ok(()); }
    if metadata.is_dir() {
        for item in fs::read_dir(directory)? {
            let entry = item?;
            let filename = entry.file_name();
            if [".git", "target", "node_modules", ".venv", ".semauri-cache"]
                .iter().any(|excluded| filename == *excluded) {
                continue;
            }
            scan_verify_sources(&entry.path(), collected)?;
        }
    } else if metadata.is_file()
        && directory.extension().is_some_and(|ext| ext == "suri")
        && directory.file_name().is_some_and(|name| name != ".config.suri")
    {
        if collected.len() >= MAX_VERIFY_FILES {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                "Project contains more than 256 .suri sources"));
        }
        collected.push(directory.to_path_buf());
    }
    Ok(())
}

// Prepare every source before modifying anything, to avoid partial changes
// when one file has invalid lexical content. Symlinks and generated cache
// directories are excluded by the shared bounded project scanner.
fn format_source_directory(
    directory: &Path, compiler: &Compiler, check: bool, write: bool,
) -> Result<(String, i32)> {
    if !check && !write {
        return Err(usage_error(
            "Directory formatting requires --check or --write; usage: semauri fmt DIRECTORY --check|--write"
        ));
    }
    let directory = fs::canonicalize(directory)
        .map_err(|error| usage_error(format!("Cannot open directory: {error}")))?;
    let mut sources = Vec::new();
    scan_verify_sources(&directory, &mut sources)
        .map_err(|error| usage_error(format!("Cannot scan directory: {error}")))?;
    sources.sort();
    if sources.is_empty() {
        return Err(usage_error(format!(
            "No .suri source files under {}", directory.display()
        )));
    }
    let mut changes = Vec::new();
    let mut total_bytes = 0u64;
    for path in &sources {
        let source = fs::read_to_string(path)
            .map_err(|error| usage_error(format!("Cannot read {}: {error}", path.display())))?;
        total_bytes = total_bytes.saturating_add(source.len() as u64);
        if total_bytes > MAX_VERIFY_BYTES {
            return Err(usage_error("Source directory exceeds the 4 MiB format limit"));
        }
        let formatted = compiler.format_source(&source)?;
        if source != formatted {
            changes.push((path.to_path_buf(), formatted));
        }
    }
    let total = sources.len();
    let changed = changes.len();
    if check {
        let mut report = changes.iter()
            .map(|(path, _)| format!("{}: formatting required\n", path.display()))
            .collect::<String>();
        report.push_str(&format!("{total} files checked; {changed} need formatting\n"));
        return Ok((report, if changed == 0 { 0 } else { 1 }));
    }
    for (path, content) in changes {
        fs::write(&path, content).map_err(|error|
            usage_error(format!("Cannot write {}: {error}", path.display())))?;
    }
    Ok((format!("Formatted {changed} of {total} .suri files\n"), 0))
}

fn verification_usage() -> String {
    "Usage: semauri verify PROJECT [--json] [--allow-console]".to_string()
}

fn verify_response(
    status: i32,
    json_mode: bool,
    project: &Path,
    source_count: usize,
    effects: &[String],
    test_report: &str,
    issues: &[String],
) -> CliResult {
    let ok = status == 0;
    let stdout = if json_mode {
        pretty_json(&json!({
            "ok": ok,
            "status": status,
            "project": project.display().to_string(),
            "source_files": source_count,
            "effects": effects,
            "tests": test_report,
            "issues": issues,
        }))
    } else {
        let mut display = format!("Semauri project verification: {}\n",
            project.display());
        display.push_str(&format!("Sources: {source_count}\n"));
        if !effects.is_empty() {
            display.push_str(&format!("Entry-point effects: {}\n", effects.join(", ")));
        }
        if !test_report.is_empty() {
            display.push_str(test_report);
            if !test_report.ends_with('\n') { display.push('\n'); }
        }
        for issue in issues {
            display.push_str(&format!("FAIL {issue}\n"));
        }
        if ok { display.push_str("PASS project verification\n"); }
        else { display.push_str("FAIL project verification\n"); }
        display
    };
    CliResult { status, stdout, stderr: String::new() }
}

fn run_verify_cli(args: &[String]) -> CliResult {
    let mut json_mode = false;
    let mut allow_console = false;
    let mut path: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json_mode = true,
            "--allow-console" => allow_console = true,
            option if option.starts_with('-') => {
                return CliResult {
                    status: 64, stdout: String::new(),
                    stderr: format!("Unknown verify option: {option}\n{}\n", verification_usage()),
                };
            }
            value if path.is_none() => path = Some(value),
            value => return CliResult {
                status: 64, stdout: String::new(),
                stderr: format!("Unexpected verify argument: {value}\n{}\n", verification_usage()),
            },
        }
    }
    let Some(path) = path else {
        return CliResult {
            status: 64, stdout: String::new(),
            stderr: format!("{}\n", verification_usage()),
        };
    };
    let project = match fs::canonicalize(path) {
        Ok(project) => project,
        Err(error) => return CliResult {
            status: 64, stdout: String::new(),
            stderr: format!("Cannot open project '{path}': {error}\n"),
        },
    };
    let marker = project.join(".semauri-root");
    let marker_ok = fs::symlink_metadata(&marker)
        .is_ok_and(|metadata| metadata.file_type().is_file());
    if !project.is_dir() || !marker_ok
        || !project.join("main.suri").is_file() || !project.join("tests").is_dir()
    {
        return verify_response(64, json_mode, &project, 0, &[], "", &[
            "Expected a directory with a regular .semauri-root, main.suri, and tests/".into()
        ]);
    }
    let mut files = Vec::new();
    if let Err(error) = scan_verify_sources(&project, &mut files) {
        return verify_response(1, json_mode, &project, files.len(), &[], "", &[
            format!("Cannot enumerate project sources: {error}")
        ]);
    }
    files.sort();
    let tests_dir = project.join("tests");
    if !files.iter().any(|file| file.starts_with(&tests_dir)) {
        return verify_response(1, json_mode, &project, files.len(), &[], "", &[
            "No .suri assertion tests found under tests/".into()
        ]);
    }
    let compiler = Compiler::new();
    let mut issues: Vec<String> = Vec::new();
    let mut total_bytes = 0u64;
    let mut syntax_ok = true;
    for file in &files {
        let text = match fs::read_to_string(file) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!("{}: cannot read UTF-8 source: {error}", file.display()));
                syntax_ok = false;
                continue;
            }
        };
        total_bytes = total_bytes.saturating_add(text.len() as u64);
        if total_bytes > MAX_VERIFY_BYTES {
            issues.push("Project .suri files exceed the 4 MiB verification limit".into());
            syntax_ok = false;
            break;
        }
        match compiler.format_source(&text) {
            Ok(formatted) if formatted != text => issues.push(format!(
                "{}: non-canonical formatting; run semauri fmt FILE --write",
                file.display()
            )),
            Ok(_) => {},
            Err(error) => {
                issues.push(format!("{}: {error}", file.display()));
                syntax_ok = false;
            }
        }
        // Reusable modules may contain only function definitions. HIR typing
        // validates them without the executable-output requirement (S301).
        // Entry points and assertion tests still undergo full CLI compilation.
        if file == &project.join("main.suri") || file.starts_with(&tests_dir) {
            let checked = run_cli(&["check".into(), file.to_string_lossy().into_owned()]);
            if checked.status != 0 {
                issues.push(format!("{}: {}", file.display(), checked.stderr.trim()));
                syntax_ok = false;
            }
        } else {
            match load_module_source(file)
                .and_then(|loaded| compiler.hir(&loaded.source))
            {
                Ok(_) => {},
                Err(error) => {
                    issues.push(format!("{}: {error}", file.display()));
                    syntax_ok = false;
                }
            }
        }
    }
    let mut effects = Vec::new();
    if syntax_ok {
        match load_module_source(&project.join("main.suri"))
            .and_then(|source| compiler.effect_analysis(&source.source))
        {
            Ok(analysis) => effects = analysis.effects(),
            Err(error) => {
                issues.push(format!("main.suri: {error}"));
                syntax_ok = false;
            }
        }
    }
    let mut test_report = String::new();
    if syntax_ok {
        let mut test_args = vec![tests_dir.to_string_lossy().into_owned()];
        if allow_console { test_args.push("--allow-console".to_string()); }
        let tests = run_test_cli(&test_args);
        test_report = tests.stdout;
        if tests.status != 0 {
            issues.push(format!("Test suite failed: {}",
                if tests.stderr.is_empty() { test_report.trim() } else { tests.stderr.trim() }));
        }
    } else {
        issues.push("Tests were not executed because source validation failed".into());
    }
    let status = if issues.is_empty() { 0 } else { 1 };
    verify_response(status, json_mode, &project, files.len(),
        &effects, &test_report, &issues)
}
