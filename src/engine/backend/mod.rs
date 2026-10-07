#[derive(Clone, Debug)]
pub struct DomainOutput {
    pub domain: String,
    pub backend: String,
    pub content: String,
}

impl DomainOutput {
    pub fn filename(&self) -> String {
        let sanitized = self
            .backend
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                    ch
                } else {
                    '-'
                }
            })
            .collect::<String>();
        format!("{}.{}", self.domain, sanitized)
    }
}

#[derive(Clone, Debug)]
pub struct CompilationResult {
    pub output: Option<String>,
    pub backend: Option<String>,
    pub outputs: Vec<DomainOutput>,
    pub ast: Program,
    pub hir: HirResult,
    pub optimized_hir: OptimizedHir,
    pub semantic: SemanticResult,
    pub effects: EffectAnalysis,
}

impl CompilationResult {
    pub fn multi_domain(&self) -> bool {
        self.outputs.len() > 1
    }

    pub fn runtime(&self) -> bool {
        self.semantic.runtime()
    }
}

pub trait BackendRenderer: Send + Sync {
    fn render(&self, artifact: &Artifact) -> Result<String>;
}

impl<F> BackendRenderer for F
where
    F: Fn(&Artifact) -> Result<String> + Send + Sync,
{
    fn render(&self, artifact: &Artifact) -> Result<String> {
        self(artifact)
    }
}

#[derive(Clone)]
pub struct BackendRegistry {
    renderers: BTreeMap<String, Arc<dyn BackendRenderer>>,
}

impl fmt::Debug for BackendRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackendRegistry")
            .field("names", &self.names())
            .finish()
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::builtins()
    }
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self {
            renderers: BTreeMap::new(),
        }
    }

    pub fn builtins() -> Self {
        let mut registry = Self::new();
        registry.register("html", render_html);
        registry.register("json-schema", render_json_schema);
        registry.register("posix-sh", render_posix_shell);
        registry
    }

    pub fn register<R>(&mut self, name: impl Into<String>, renderer: R) -> &mut Self
    where
        R: BackendRenderer + 'static,
    {
        self.renderers.insert(name.into(), Arc::new(renderer));
        self
    }

    pub fn names(&self) -> Vec<String> {
        self.renderers.keys().cloned().collect()
    }

    pub fn render(&self, name: &str, artifact: &Artifact) -> Result<String> {
        self.renderers
            .get(name)
            .ok_or_else(|| {
                SemauriError::backend(
                    "S402",
                    format!("Unknown backend '{name}'"),
                    None,
                )
            })?
            .render(artifact)
    }
}

#[derive(Clone, Debug)]
pub struct Compiler {
    domains: DomainRegistry,
    backends: BackendRegistry,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            domains: DomainRegistry::builtins(),
            backends: BackendRegistry::builtins(),
        }
    }

    pub fn with_registries(domains: DomainRegistry, backends: BackendRegistry) -> Self {
        Self { domains, backends }
    }

    pub fn with_domains(domains: DomainRegistry) -> Self {
        Self {
            domains,
            backends: BackendRegistry::builtins(),
        }
    }

    pub fn domains(&self) -> &DomainRegistry {
        &self.domains
    }

    pub fn tokenize(&self, source: &str) -> Result<Vec<Token>> {
        Lexer::new(source, &self.domains).tokens()
    }

    pub fn parse(&self, source: &str) -> Result<Program> {
        let tokens = self.tokenize(source)?;
        Parser::new(tokens, &self.domains).parse()
    }

    pub fn hir(&self, source: &str) -> Result<HirResult> {
        let ast = self.parse(source)?;
        HirBuilder::new(&self.domains).build(&ast)
    }

    pub fn optimized_hir(&self, source: &str) -> Result<OptimizedHir> {
        Ok(optimize_hir(&self.hir(source)?))
    }

    pub fn effect_analysis(&self, source: &str) -> Result<EffectAnalysis> {
        Ok(analyze_effects(&self.hir(source)?))
    }

    pub fn validate_capabilities(
        &self,
        source: &str,
        policy: &CapabilityPolicy,
    ) -> Result<()> {
        policy.validate(&self.effect_analysis(source)?)
    }

    pub fn runtime_plan(&self, source: &str) -> Result<RuntimePlan> {
        let optimized = self.optimized_hir(source)?;
        Ok(Lowerer::new(&self.domains, &optimized.result.symbols)
            .lower(&optimized.result)?
            .runtime_plan)
    }

    pub fn analyze(&self, source: &str) -> Result<(Program, SemanticResult)> {
        let ast = self.parse(source)?;
        let hir = HirBuilder::new(&self.domains).build(&ast)?;
        let semantic = Lowerer::new(&self.domains, &hir.symbols).lower(&hir)?;
        Ok((ast, semantic))
    }

    pub fn compile(&self, source: &str, backend: Option<&str>) -> Result<CompilationResult> {
        self.compile_with_policy(source, backend, None)
    }

    pub fn compile_with_policy(
        &self,
        source: &str,
        backend: Option<&str>,
        policy: Option<&CapabilityPolicy>,
    ) -> Result<CompilationResult> {
        let ast = self.parse(source)?;
        let hir = HirBuilder::new(&self.domains).build(&ast)?;
        let effects = analyze_effects(&hir);
        if let Some(policy) = policy {
            policy.validate(&effects)?;
        }
        let optimized_hir = optimize_hir(&hir);
        let semantic =
            Lowerer::new(&self.domains, &optimized_hir.result.symbols).lower(&optimized_hir.result)?;
        let outputs = self.render_program(&semantic.program_ir, backend)?;
        let (output, selected_backend) = if outputs.len() == 1 {
            (
                Some(outputs[0].content.clone()),
                Some(outputs[0].backend.clone()),
            )
        } else {
            (None, None)
        };
        Ok(CompilationResult {
            output,
            backend: selected_backend,
            outputs,
            ast,
            hir,
            optimized_hir,
            semantic,
            effects,
        })
    }

    fn render_program(
        &self,
        program: &ProgramIr,
        backend: Option<&str>,
    ) -> Result<Vec<DomainOutput>> {
        if backend.is_some() && program.units.len() > 1 {
            return Err(SemauriError::backend(
                "S405",
                "A single backend override cannot render a multi-domain program",
                Some(
                    "Build without --backend so each semantic domain uses its own default backend."
                        .to_string(),
                ),
            ));
        }
        let mut outputs = Vec::new();
        for unit in &program.units {
            let backend_name = if let Some(backend) = backend {
                backend.to_string()
            } else {
                self.domains
                    .fetch(&unit.domain)?
                    .default_backend
                    .clone()
                    .ok_or_else(|| {
                        SemauriError::backend(
                            "S404",
                            format!(
                                "Semantic domain '{}' has no default backend",
                                unit.domain
                            ),
                            None,
                        )
                    })?
            };
            let content = self.backends.render(&backend_name, &unit.artifact)?;
            outputs.push(DomainOutput {
                domain: unit.domain.clone(),
                backend: backend_name,
                content,
            });
        }
        Ok(outputs)
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_html(artifact: &Artifact) -> Result<String> {
    let Artifact::Web(program) = artifact else {
        return Err(SemauriError::backend(
            "S401",
            "HTML backend cannot render this semantic IR",
            None,
        ));
    };
    let title = html_escape(&program.title);
    let mut body = Vec::new();
    for element in &program.elements {
        let style = if let Some(ValueData::Color(color) | ValueData::String(color)) =
            element.properties.get("color")
        {
            format!(" style=\"color: {}\"", html_escape(color))
        } else {
            String::new()
        };
        let label = html_escape(&element.label);
        let rendered = match element.kind.as_str() {
            "button" => format!("<button{style}>{label}</button>"),
            "image" => format!(
                "<figure{style} data-semauri-kind=\"image\" aria-label=\"{label}\">{label}</figure>"
            ),
            kind => {
                return Err(SemauriError::backend(
                    "S403",
                    format!("HTML backend cannot render element kind '{kind}'"),
                    None,
                ))
            }
        };
        body.push(format!("    {rendered}"));
    }
    let body_text = if body.is_empty() {
        String::new()
    } else {
        format!("\n{}", body.join("\n"))
    };
    Ok(format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>{title}</title>\n</head>\n<body>\n  <h1>{title}</h1>{body_text}\n</body>\n</html>\n"
    ))
}

fn render_json_schema(artifact: &Artifact) -> Result<String> {
    let Artifact::Schema(program) = artifact else {
        return Err(SemauriError::backend(
            "S401",
            "JSON Schema backend cannot render this semantic IR",
            None,
        ));
    };
    let mut properties = JsonMap::new();
    let mut required = Vec::new();
    for field in &program.fields {
        let datatype = match field.properties.get("datatype") {
            Some(ValueData::String(value)) => value.clone(),
            _ => "string".to_string(),
        };
        properties.insert(field.label.clone(), json!({"type": datatype}));
        if matches!(field.properties.get("required"), Some(ValueData::Boolean(true))) {
            required.push(field.label.clone());
        }
    }
    let mut document = JsonMap::new();
    document.insert(
        "$schema".to_string(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    document.insert("title".to_string(), json!(program.title));
    document.insert("type".to_string(), json!("object"));
    document.insert("properties".to_string(), JsonValue::Object(properties));
    if !required.is_empty() {
        document.insert("required".to_string(), json!(required));
    }
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&JsonValue::Object(document)).unwrap()
    ))
}

fn render_posix_shell(artifact: &Artifact) -> Result<String> {
    let Artifact::Filesystem(plan) = artifact else {
        return Err(SemauriError::backend(
            "S403",
            "posix-sh backend expects a filesystem operation plan",
            None,
        ));
    };
    let mut lines = vec!["#!/usr/bin/env sh".to_string(), "set -eu".to_string(), String::new()];
    for operation in &plan.operations {
        let string_arg = |name: &str| -> Result<String> {
            match operation.arguments.get(name) {
                Some(ValueData::String(value) | ValueData::Color(value)) => Ok(value.clone()),
                Some(value) => Ok(match value {
                    ValueData::Number(value) => value.to_string(),
                    ValueData::Boolean(value) => value.to_string(),
                    _ => {
                        return Err(SemauriError::backend(
                            "S405",
                            format!("Unsupported filesystem argument '{name}'"),
                            None,
                        ))
                    }
                }),
                None => Err(SemauriError::backend(
                    "S405",
                    format!("Missing filesystem argument '{name}'"),
                    None,
                )),
            }
        };
        let rendered = match operation.name.as_str() {
            "write" => format!(
                "printf '%s' {} > {}",
                shell_escape(&string_arg("content")?),
                shell_escape(&string_arg("path")?)
            ),
            "append" => format!(
                "printf '%s' {} >> {}",
                shell_escape(&string_arg("content")?),
                shell_escape(&string_arg("path")?)
            ),
            "copy" => format!(
                "cp {} {}",
                shell_escape(&string_arg("source")?),
                shell_escape(&string_arg("destination")?)
            ),
            "move" => format!(
                "mv {} {}",
                shell_escape(&string_arg("source")?),
                shell_escape(&string_arg("destination")?)
            ),
            "make_directory" => format!("mkdir -p {}", shell_escape(&string_arg("path")?)),
            "touch" => format!("touch {}", shell_escape(&string_arg("path")?)),
            "delete" => format!("rm -f {}", shell_escape(&string_arg("path")?)),
            other => {
                return Err(SemauriError::backend(
                    "S405",
                    format!("Unsupported filesystem operation '{other}'"),
                    None,
                ))
            }
        };
        lines.push(rendered);
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn shell_escape(value: &str) -> String {
    if !value.is_empty()
        && value.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, '_' | '@' | '%' | '+' | '=' | ':' | ',' | '.' | '/' | '-')
        })
    {
        return value.to_string();
    }
    if value.is_empty() {
        return "''".to_string();
    }
    let mut escaped = String::new();
    for ch in value.chars() {
        if ch == '\n' {
            escaped.push_str("'\n'");
        } else {
            escaped.push('\\');
            escaped.push(ch);
        }
    }
    escaped
}

fn execute_filesystem_plan(plan: &OperationPlan, cwd: Option<&Path>) -> std::io::Result<()> {
    let root = cwd.unwrap_or_else(|| Path::new("."));
    for operation in &plan.operations {
        let arg = |name: &str| -> String {
            match operation.arguments.get(name) {
                Some(ValueData::String(value) | ValueData::Color(value)) => value.clone(),
                Some(ValueData::Number(value)) => value.to_string(),
                Some(ValueData::Boolean(value)) => value.to_string(),
                _ => String::new(),
            }
        };
        match operation.name.as_str() {
            "write" => fs::write(root.join(arg("path")), arg("content"))?,
            "append" => {
                use std::io::Write;
                let path = root.join(arg("path"));
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                file.write_all(arg("content").as_bytes())?;
            }
            "copy" => {
                fs::copy(root.join(arg("source")), root.join(arg("destination")))?;
            }
            "move" => fs::rename(root.join(arg("source")), root.join(arg("destination")))?,
            "make_directory" => fs::create_dir_all(root.join(arg("path")))?,
            "touch" => {
                let path = root.join(arg("path"));
                let _ = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
            }
            "delete" => {
                let path = root.join(arg("path"));
                if path.exists() {
                    if path.is_dir() {
                        fs::remove_dir_all(path)?;
                    } else {
                        fs::remove_file(path)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

