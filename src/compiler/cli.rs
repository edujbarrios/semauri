#[derive(Clone, Debug)]
pub struct CliResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_cli(args: &[String]) -> CliResult {
    if args.first().map(String::as_str) == Some("config") {
        return run_config_cli(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("test") {
        return run_test_cli(&args[1..]);
    }
    let compiler = Compiler::new();
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut current_source: Option<String> = None;
    let mut json_diagnostics = false;

    let outcome = (|| -> Result<i32> {
        let command = args.first().map(String::as_str);
        match command {
            None | Some("help") | Some("--help") | Some("-h") => {
                stdout.push_str(&usage());
                Ok(0)
            }
            Some("version") | Some("--version") | Some("-v") => {
                stdout.push_str(&format!("Semauri {VERSION}\n"));
                Ok(0)
            }
            Some("domains") => {
                if args.len() != 1 {
                    return Err(usage_error(format!(
                        "Unexpected arguments: {}",
                        args[1..].join(" ")
                    )));
                }
                stdout.push_str(&pretty_json(&compiler.domains().to_json()));
                Ok(0)
            }
            Some("tokens")
            | Some("ast")
            | Some("hir")
            | Some("optimize")
            | Some("symbols")
            | Some("effects")
            | Some("plan")
            | Some("graph")
            | Some("explain") => {
                // Inspection commands already print JSON on success. The flag
                // opts in to JSON diagnostics without wrapping their results.
                let mut source_args = Vec::new();
                for arg in &args[1..] {
                    if arg == "--json" && command != Some("explain") {
                        json_diagnostics = true;
                    } else {
                        source_args.push(arg.clone());
                    }
                }
                let source = read_single_source(&source_args)?;
                current_source = Some(source.clone());
                match command.unwrap() {
                    "tokens" => {
                        let tokens = compiler.tokenize(&source)?;
                        stdout.push_str(&pretty_json(&JsonValue::Array(
                            tokens.iter().map(Token::to_json).collect(),
                        )));
                    }
                    "ast" => stdout.push_str(&pretty_json(&compiler.parse(&source)?.to_json())),
                    "hir" => stdout.push_str(&pretty_json(&compiler.hir(&source)?.to_json())),
                    "optimize" => {
                        stdout.push_str(&pretty_json(&compiler.optimized_hir(&source)?.to_json()))
                    }
                    "symbols" => {
                        let (_, semantic) = compiler.analyze(&source)?;
                        stdout.push_str(&pretty_json(&JsonValue::Array(
                            semantic.symbols.iter().map(Symbol::to_json).collect(),
                        )));
                    }
                    "effects" => {
                        stdout.push_str(&pretty_json(&compiler.effect_analysis(&source)?.to_json()))
                    }
                    "plan" => {
                        stdout.push_str(&pretty_json(&compiler.runtime_plan(&source)?.to_json()))
                    }
                    "graph" => {
                        stdout.push_str(&pretty_json(&compiler.runtime_graph(&source)?))
                    }
                    "explain" => {
                        let (_, semantic) = compiler.analyze(&source)?;
                        for (index, explanation) in semantic.explanations.iter().enumerate() {
                            stdout.push_str(&format!("{}. {}\n", index + 1, explanation));
                        }
                    }
                    _ => unreachable!(),
                }
                Ok(0)
            }
            Some("audit") => {
                let mut strict = false;
                let mut path = None;
                for arg in &args[1..] {
                    match arg.as_str() {
                        "--strict" => strict = true,
                        option if option.starts_with('-') => {
                            return Err(usage_error(format!("Unknown audit option: {option}")));
                        }
                        value if path.is_none() => path = Some(value.to_string()),
                        value => {
                            return Err(usage_error(format!(
                                "Unexpected audit argument: {value}"
                            )));
                        }
                    }
                }
                let path = path.ok_or_else(|| usage_error(
                    "Usage: semauri audit FILE [--strict]"
                ))?;
                let source = fs::read_to_string(&path)
                    .map_err(|error| usage_error(format!("S001: {error}")))?;
                current_source = Some(source.clone());
                let report = compiler.ml_reproducibility_audit(&source)?;
                let warnings = report["warning_count"].as_u64().unwrap_or(0);
                stdout.push_str(&pretty_json(&report));
                Ok(if strict && warnings > 0 { 2 } else { 0 })
            }
            Some("fmt") => {
                let mut check = false;
                let mut write = false;
                let mut path: Option<String> = None;
                for arg in &args[1..] {
                    match arg.as_str() {
                        "--check" => check = true,
                        "--write" => write = true,
                        value if value.starts_with('-') => {
                            return Err(usage_error(format!("Unknown fmt option: {value}")))
                        }
                        value if path.is_none() => path = Some(value.to_string()),
                        value => {
                            return Err(usage_error(format!(
                                "Unexpected fmt argument: {value}"
                            )))
                        }
                    }
                }
                if check && write {
                    return Err(usage_error(
                        "fmt --check and --write cannot be used together",
                    ));
                }
                let path = path.ok_or_else(|| usage_error(
                    "Usage: semauri fmt FILE [--check | --write]"
                ))?;
                let source = fs::read_to_string(&path)
                    .map_err(|error| usage_error(format!("S001: {error}")))?;
                current_source = Some(source.clone());
                let formatted = compiler.format_source(&source)?;
                if check {
                    if source != formatted {
                        stdout.push_str(&format!("{path}: lowercase formatting required\n"));
                        return Ok(1);
                    }
                } else if write {
                    if source != formatted {
                        fs::write(&path, formatted)
                            .map_err(|error| usage_error(format!("S001: {error}")))?;
                    }
                } else {
                    stdout.push_str(&formatted);
                }
                Ok(0)
            }
            Some("check") => {
                let mut allowed: Option<Vec<String>> = None;
                let mut path = None;
                let mut index = 1;
                while index < args.len() {
                    match args[index].as_str() {
                        "--json" => {
                            json_diagnostics = true;
                            index += 1;
                        }
                        "--allow" => {
                            let effect = args.get(index + 1).ok_or_else(|| {
                                usage_error("--allow requires an effect")
                            })?;
                            allowed.get_or_insert_with(Vec::new).push(effect.clone());
                            index += 2;
                        }
                        "--allow-none" => {
                            allowed = Some(Vec::new());
                            index += 1;
                        }
                        value if value.starts_with('-') => {
                            return Err(usage_error(format!("Unknown option: {value}")))
                        }
                        value => {
                            if path.is_some() {
                                return Err(usage_error(format!(
                                    "Unexpected arguments: {}",
                                    args[index..].join(" ")
                                )));
                            }
                            path = Some(value.to_string());
                            index += 1;
                        }
                    }
                }
                let path = path.ok_or_else(|| usage_error(usage()))?;
                let source = fs::read_to_string(&path)
                    .map_err(|error| usage_error(format!("S001: {error}")))?;
                current_source = Some(source.clone());
                compiler.analyze(&source)?;
                if let Some(allowed) = allowed {
                    compiler.validate_capabilities(&source, &CapabilityPolicy::new(allowed))?;
                }
                if json_diagnostics {
                    stdout.push_str(&pretty_json(&json!({
                        "ok": true,
                        "status": 0,
                        "diagnostics": [],
                    })));
                } else {
                    stdout.push_str("OK\n");
                }
                Ok(0)
            }
            Some("build") => {
                let mut backend = None;
                let mut output = None;
                let mut path = None;
                let mut index = 1;
                while index < args.len() {
                    match args[index].as_str() {
                        "-o" | "--output" => {
                            output = Some(
                                args.get(index + 1)
                                    .ok_or_else(|| usage_error("--output requires a path"))?
                                    .clone(),
                            );
                            index += 2;
                        }
                        "--backend" => {
                            backend = Some(
                                args.get(index + 1)
                                    .ok_or_else(|| usage_error("--backend requires a name"))?
                                    .clone(),
                            );
                            index += 2;
                        }
                        value if value.starts_with('-') => {
                            return Err(usage_error(format!("Unknown option: {value}")))
                        }
                        value => {
                            if path.is_some() {
                                return Err(usage_error(format!(
                                    "Unexpected arguments: {}",
                                    args[index..].join(" ")
                                )));
                            }
                            path = Some(value.to_string());
                            index += 1;
                        }
                    }
                }
                let path = path.ok_or_else(|| usage_error(usage()))?;
                let source = fs::read_to_string(&path)
                    .map_err(|error| usage_error(format!("S001: {error}")))?;
                current_source = Some(source.clone());
                let result = compiler.compile(&source, backend.as_deref())?;
                if result.outputs.is_empty() && result.runtime() {
                    return Err(SemauriError::backend(
                        "S406",
                        "Program produces a runtime plan but no build-time backend output",
                        Some("Inspect it with 'semauri plan FILE'.".to_string()),
                    ));
                }
                if result.outputs.len() > 1 {
                    if let Some(output_dir) = output {
                        fs::create_dir_all(&output_dir).map_err(|error| {
                            usage_error(format!("S001: {error}"))
                        })?;
                        for domain_output in &result.outputs {
                            let path = Path::new(&output_dir).join(domain_output.filename());
                            fs::write(&path, &domain_output.content)
                                .map_err(|error| usage_error(format!("S001: {error}")))?;
                            stdout.push_str(&format!(
                                "Built {} for {} with {}\n",
                                path.display(),
                                domain_output.domain,
                                domain_output.backend
                            ));
                        }
                    } else {
                        for (index, domain_output) in result.outputs.iter().enumerate() {
                            if index > 0 {
                                stdout.push('\n');
                            }
                            stdout.push_str(&format!(
                                "=== {} [{}] ===\n{}",
                                domain_output.domain,
                                domain_output.backend,
                                domain_output.content
                            ));
                            if !domain_output.content.ends_with('\n') {
                                stdout.push('\n');
                            }
                        }
                    }
                } else if let Some(domain_output) = result.outputs.first() {
                    if let Some(output_path) = output {
                        fs::write(&output_path, &domain_output.content)
                            .map_err(|error| usage_error(format!("S001: {error}")))?;
                        stdout.push_str(&format!(
                            "Built {} with {}\n",
                            output_path, domain_output.backend
                        ));
                    } else {
                        stdout.push_str(&domain_output.content);
                    }
                }
                Ok(0)
            }
            Some("run") => {
                let mut allowed = Vec::new();
                let mut cwd: Option<PathBuf> = None;
                let mut dry_run = false;
                let mut path = None;
                let mut index = 1;
                while index < args.len() {
                    match args[index].as_str() {
                        "--allow" => {
                            allowed.push(
                                args.get(index + 1)
                                    .ok_or_else(|| usage_error("--allow requires an effect"))?
                                    .clone(),
                            );
                            index += 2;
                        }
                        "--cwd" => {
                            cwd = Some(PathBuf::from(
                                args.get(index + 1)
                                    .ok_or_else(|| usage_error("--cwd requires a path"))?,
                            ));
                            index += 2;
                        }
                        "--dry-run" => {
                            dry_run = true;
                            index += 1;
                        }
                        value if value.starts_with('-') => {
                            return Err(usage_error(format!("Unknown option: {value}")))
                        }
                        value => {
                            if path.is_some() {
                                return Err(usage_error(format!(
                                    "Unexpected arguments: {}",
                                    args[index..].join(" ")
                                )));
                            }
                            path = Some(value.to_string());
                            index += 1;
                        }
                    }
                }
                let path = path.ok_or_else(|| usage_error(usage()))?;
                let source = fs::read_to_string(&path)
                    .map_err(|error| usage_error(format!("S001: {error}")))?;
                current_source = Some(source.clone());
                let result = compiler.compile(&source, None)?;
                let console_only = !result.semantic.runtime_plan.operations.is_empty()
                    && result
                        .semantic
                        .runtime_plan
                        .operations
                        .iter()
                        .all(|operation| operation.domain == "console");
                if console_only {
                    if dry_run {
                        stdout.push_str(&pretty_json(&result.semantic.runtime_plan.to_json()));
                        return Ok(0);
                    }
                    CapabilityPolicy::new(allowed).validate(&result.effects)?;
                    stdout.push_str(&execute_console_plan(&result.semantic.runtime_plan)?);
                    return Ok(0);
                }
                if result.outputs.len() != 1
                    || result.outputs[0].domain != "filesystem"
                    || result.outputs[0].backend != "posix-sh"
                {
                    return Err(SemauriError::backend(
                        "S407",
                        "run currently supports a console program or one filesystem program rendered by posix-sh",
                        Some(
                            "Use 'semauri build' or 'semauri plan' for other domains until their execution runtimes are available."
                                .to_string(),
                        ),
                    ));
                }
                if dry_run {
                    stdout.push_str(&result.outputs[0].content);
                    return Ok(0);
                }
                CapabilityPolicy::new(allowed).validate(&result.effects)?;
                let Some(Artifact::Filesystem(plan)) =
                    result.semantic.program_ir.artifact("filesystem")
                else {
                    return Err(SemauriError::backend(
                        "S407",
                        "run requires a filesystem operation plan",
                        None,
                    ));
                };
                execute_filesystem_plan(plan, cwd.as_deref()).map_err(|error| {
                    SemauriError::new(
                        ErrorKind::Runtime,
                        "S500",
                        error.to_string(),
                        None,
                        None,
                    )
                })?;
                Ok(0)
            }
            Some(other) => {
                stderr.push_str(&format!("Unknown command: {other}\n"));
                stderr.push_str(&usage());
                Ok(64)
            }
        }
    })();

    match outcome {
        Ok(status) => CliResult {
            status,
            stdout,
            stderr,
        },
        Err(error) => {
            let status = match error.kind {
                ErrorKind::Usage => 64,
                ErrorKind::Runtime => 70,
                _ => 65,
            };
            if json_diagnostics {
                stdout.clear();
                stdout.push_str(&pretty_json(&json!({
                    "ok": false,
                    "status": status,
                    "diagnostics": [{
                        "code": error.code,
                        "kind": format!("{:?}", error.kind).to_lowercase(),
                        "message": error.message,
                        "span": error.span,
                        "hint": error.hint,
                    }],
                })));
            } else {
                stderr.push_str(&error.diagnostic(current_source.as_deref()));
                stderr.push('\n');
            }
            CliResult {
                status,
                stdout,
                stderr,
            }
        }
    }
}

fn run_config_cli(args: &[String]) -> CliResult {
    let mut path = ".config.suri".to_string();
    let mut path_seen = false;
    let mut check_environment = false;
    let mut overrides = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--check-env" => {
                check_environment = true;
                index += 1;
            }
            "--set" => {
                let Some(override_value) = args.get(index + 1) else {
                    return config_cli_error("--set requires NAME=JSON_VALUE");
                };
                overrides.push(override_value.clone());
                index += 2;
            }
            value if value.starts_with('-') => {
                return config_cli_error(format!("unknown config option: {value}"));
            }
            value if !path_seen => {
                path = value.to_string();
                path_seen = true;
                index += 1;
            }
            value => return config_cli_error(format!("unexpected config argument: {value}")),
        }
    }

    let mut config = match crate::ProgramConfig::from_file(&path) {
        Ok(config) => config,
        Err(error) => return config_cli_error(error.to_string()),
    };
    for item in overrides {
        let Some((name, raw_value)) = item.split_once('=') else {
            return config_cli_error(format!("invalid override '{item}'; expected NAME=JSON_VALUE"));
        };
        let value = match serde_json::from_str(raw_value) {
            Ok(value) => value,
            Err(_) => return config_cli_error(format!("override '{name}' is not valid JSON")),
        };
        if let Err(error) = config.set_parameter(name, value) {
            return config_cli_error(error.to_string());
        }
    }
    if check_environment {
        if let Err(error) = config.resolve() {
            return config_cli_error(error.to_string());
        }
    }

    CliResult {
        status: 0,
        stdout: pretty_json(&config.summary_json(|name| std::env::var_os(name).is_some())),
        stderr: String::new(),
    }
}

fn config_cli_error(message: impl AsRef<str>) -> CliResult {
    CliResult {
        status: 64,
        stdout: String::new(),
        stderr: format!(
            "S001: {}\nUsage: semauri config [PATH] [--set NAME=JSON_VALUE] [--check-env]\n",
            message.as_ref()
        ),
    }
}

fn pretty_json(value: &JsonValue) -> String {
    format!("{}\n", serde_json::to_string_pretty(value).unwrap())
}

fn read_single_source(args: &[String]) -> Result<String> {
    if args.len() != 1 {
        return Err(usage_error(usage()));
    }
    fs::read_to_string(&args[0]).map_err(|error| usage_error(format!("S001: {error}")))
}

fn usage_error(message: impl Into<String>) -> SemauriError {
    SemauriError::new(ErrorKind::Usage, "S001", message, None, None)
}

pub fn usage() -> String {
    "Usage: semauri <command> [options] [FILE]\n\nCommands:\n  config [PATH]            Validate .config.suri parameters and environment bindings\n  domains                  Print registered semantic domains as JSON\n  tokens FILE              Print lexer tokens as JSON\n  ast FILE                 Print the parsed syntax AST as JSON\n  hir FILE                 Print typed HIR before optimization\n  optimize FILE            Print optimized HIR and pass statistics\n  symbols FILE             Print semantic symbols as JSON\n  effects FILE             Print statically required effects/capabilities\n  plan FILE                Print the lowered runtime operation plan as JSON\n  graph FILE               Print validated SSA runtime dependencies as JSON\n  audit FILE [--strict]     Report ML provenance/seed risks; strict mode fails CI\n  explain FILE             Explain semantic decisions\n  fmt FILE [--check|--write]  Lowercase source syntax while preserving strings\n  check FILE [--allow ...] [--json] Validate source with optional JSON diagnostics\n  test PATH [--allow-console] Execute assertion-based .suri test files\n  build FILE [-o PATH]     Compile one or more build-time domain outputs\n  run FILE --allow EFFECT  Execute console or filesystem effects after authorization\n  version                  Print version\n"
        .to_string()
}

