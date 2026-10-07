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

