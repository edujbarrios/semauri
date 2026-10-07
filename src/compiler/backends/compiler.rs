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

