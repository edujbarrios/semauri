// Assertion-based test discovery is intentionally local and effect-restricted.
// It does not execute arbitrary project hooks or shell scripts.

// Keep test discovery bounded just like project verification. Ignore generated
// files and symlinks: test execution must never walk into build artifacts.
const MAX_TEST_FILES: usize = 256;
const MAX_TEST_DEPTH: usize = 32;

fn collect_suri_test_files(
    path: &Path, output: &mut Vec<PathBuf>, depth: usize,
) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if path.extension().is_some_and(|ext| ext == "suri")
            && path.file_name().is_some_and(|name| name != ".config.suri")
        {
            if output.len() >= MAX_TEST_FILES {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Test suite exceeds 256 .suri files",
                ));
            }
            output.push(path.to_path_buf());
        }
    } else if metadata.is_dir() {
        if depth > MAX_TEST_DEPTH {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Test directory nesting exceeds 32 levels",
            ));
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            if [".git", "target", "node_modules", ".venv", ".semauri-cache"]
                .iter().any(|excluded| name == *excluded)
            {
                continue;
            }
            collect_suri_test_files(&entry.path(), output, depth + 1)?;
        }
    }
    Ok(())
}

fn run_test_cli(args: &[String]) -> CliResult {
    let mut allow_console = false;
    let mut path: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "--allow-console" => allow_console = true,
            value if value.starts_with('-') => {
                return CliResult {
                    status: 64,
                    stdout: String::new(),
                    stderr: format!("Unknown test option: {value}\nUsage: semauri test PATH [--allow-console]\n"),
                };
            }
            value if path.is_none() => path = Some(value),
            value => {
                return CliResult {
                    status: 64,
                    stdout: String::new(),
                    stderr: format!("Unexpected test argument: {value}\nUsage: semauri test PATH [--allow-console]\n"),
                };
            }
        }
    }

    let Some(path) = path else {
        return CliResult {
            status: 64,
            stdout: String::new(),
            stderr: "Usage: semauri test PATH [--allow-console]\n".to_string(),
        };
    };
    let target = Path::new(path);
    let mut files = Vec::new();
    if let Err(error) = collect_suri_test_files(target, &mut files, 0) {
        return CliResult {
            status: 64,
            stdout: String::new(),
            stderr: format!("Cannot read test path '{}': {error}\n", target.display()),
        };
    }
    files.sort();
    if files.is_empty() {
        return CliResult {
            status: 64,
            stdout: String::new(),
            stderr: format!("No .suri test files found in '{}'\n", target.display()),
        };
    }

    let compiler = Compiler::new();
    let mut output = String::new();
    let mut failures = 0usize;

    for file in &files {
        let label = file.display();
        let outcome = (|| -> std::result::Result<(), String> {
            let source = load_module_source(file)
                .map_err(|error| error.to_string())?.source;
            let compiled = compiler
                .compile(&source, None)
                .map_err(|error| error.to_string())?;
            if !compiled.semantic.program_ir.units.is_empty() {
                return Err("Tests must not generate build artifacts or filesystem plans".to_string());
            }
            if compiled.effects.effects().iter().any(|effect| effect != "console_write") {
                return Err("Tests may not require filesystem, network or other non-console capabilities".to_string());
            }
            let operations = &compiled.semantic.runtime_plan.operations;
            if !operations.iter().any(|op| op.domain == "console" && op.name == "assert") {
                return Err("Test file needs at least one executable Assert".to_string());
            }
            if operations.iter().any(|op| {
                op.domain != "console" || !matches!(op.name.as_str(), "assert" | "print")
            }) {
                return Err("Test runner only supports console assertions and optional prints".to_string());
            }

            // Reuse the public run path: it enforces all authorization checks.
            let mut arguments = vec!["run".to_string()];
            if allow_console {
                arguments.extend(["--allow".to_string(), "console_write".to_string()]);
            }
            arguments.push(file.to_string_lossy().into_owned());
            let result = run_cli(&arguments);
            if result.status == 0 {
                Ok(())
            } else {
                Err(result.stderr.trim().to_string())
            }
        })();

        match outcome {
            Ok(()) => output.push_str(&format!("PASS {label}\n")),
            Err(reason) => {
                failures += 1;
                output.push_str(&format!("FAIL {label}\n  {}\n", reason.replace('\n', "\n  ")));
            }
        }
    }

    output.push_str(&format!(
        "\n{} passed; {} failed\n",
        files.len() - failures,
        failures
    ));
    CliResult {
        status: if failures == 0 { 0 } else { 1 },
        stdout: output,
        stderr: String::new(),
    }
}
