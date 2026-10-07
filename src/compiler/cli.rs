#[derive(Clone, Debug)]
pub struct CliResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_cli(args: &[String]) -> CliResult {
    let compiler = Compiler::new();
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut current_source: Option<String> = None;

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
            | Some("explain") => {
                let source = read_single_source(&args[1..])?;
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
            Some("check") => {
                let mut allowed: Option<Vec<String>> = None;
                let mut path = None;
                let mut index = 1;
                while index < args.len() {
                    match args[index].as_str() {
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
                stdout.push_str("OK\n");
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
                let console_only = !result.runtime_plan.operations.is_empty()
                    && result
                        .runtime_plan
                        .operations
                        .iter()
                        .all(|operation| operation.domain == "console");
                if console_only {
                    if dry_run {
                        stdout.push_str(&pretty_json(&result.runtime_plan.to_json()));
                        return Ok(0);
                    }
                    CapabilityPolicy::new(allowed).validate(&result.effects)?;
                    stdout.push_str(&execute_console_plan(&result.runtime_plan)?);
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
            stderr.push_str(&error.diagnostic(current_source.as_deref()));
            stderr.push('\n');
            CliResult {
                status,
                stdout,
                stderr,
            }
        }
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
    "Usage: semauri <command> [options] [FILE]\n\nCommands:\n  domains                  Print registered semantic domains as JSON\n  tokens FILE              Print lexer tokens as JSON\n  ast FILE                 Print the parsed syntax AST as JSON\n  hir FILE                 Print typed HIR before optimization\n  optimize FILE            Print optimized HIR and pass statistics\n  symbols FILE             Print semantic symbols as JSON\n  effects FILE             Print statically required effects/capabilities\n  plan FILE                Print the lowered runtime operation plan as JSON\n  explain FILE             Explain semantic decisions\n  check FILE [--allow ...] Validate source and optionally enforce capabilities\n  build FILE [-o PATH]     Compile one or more build-time domain outputs\n  run FILE --allow EFFECT  Execute console or filesystem effects after authorization\n  version                  Print version\n"
        .to_string()
}

