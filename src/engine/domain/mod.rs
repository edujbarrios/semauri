#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainCategory {
    Artifact,
    Element,
    Property,
    Action,
}

impl DomainCategory {
    fn token_kind(&self) -> TokenKind {
        match self {
            DomainCategory::Artifact => TokenKind::DomainArtifact,
            DomainCategory::Element => TokenKind::DomainElement,
            DomainCategory::Property => TokenKind::DomainProperty,
            DomainCategory::Action => TokenKind::DomainAction,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            DomainCategory::Artifact => "artifact",
            DomainCategory::Element => "element",
            DomainCategory::Property => "property",
            DomainCategory::Action => "action",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainTerm {
    pub domain: String,
    pub category: DomainCategory,
    pub kind: String,
}

impl DomainTerm {
    fn to_json(&self) -> JsonValue {
        json!({
            "domain": self.domain,
            "category": self.category.as_str(),
            "kind": self.kind
        })
    }
}

#[derive(Clone, Debug)]
pub enum PatternSegment {
    Literal(String),
    Slot { name: String, ty: Option<Type> },
}

impl PatternSegment {
    pub fn literal(word: impl Into<String>) -> Self {
        Self::Literal(word.into().to_lowercase())
    }

    pub fn slot(name: impl Into<String>, ty: Type) -> Self {
        Self::Slot {
            name: name.into(),
            ty: Some(ty),
        }
    }

    pub fn untyped_slot(name: impl Into<String>) -> Self {
        Self::Slot {
            name: name.into(),
            ty: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OperationSpec {
    pub name: String,
    pub verbs: Vec<String>,
    pub pattern: Vec<PatternSegment>,
    pub return_type: Type,
    pub effects: Vec<String>,
}

impl OperationSpec {
    pub fn new(
        name: impl Into<String>,
        verbs: impl IntoIterator<Item = impl Into<String>>,
        pattern: Vec<PatternSegment>,
        return_type: Type,
        effects: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            verbs: verbs
                .into_iter()
                .map(|verb| verb.into().to_lowercase())
                .collect(),
            pattern,
            return_type,
            effects: effects.into_iter().map(Into::into).collect(),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let pattern = self
            .pattern
            .iter()
            .map(|segment| match segment {
                PatternSegment::Literal(word) => json!({"literal": word}),
                PatternSegment::Slot { name, ty } => json!({
                    "slot": name,
                    "kind": "expression",
                    "type": ty.as_ref().map(Type::to_json)
                }),
            })
            .collect::<Vec<_>>();
        json!({
            "name": self.name,
            "verbs": self.verbs,
            "pattern": pattern,
            "returns": self.return_type.to_json(),
            "effects": self.effects
        })
    }
}

#[derive(Clone, Debug)]
pub struct DomainSpec {
    pub name: String,
    pub default_backend: Option<String>,
    pub artifacts: BTreeMap<String, String>,
    pub elements: BTreeMap<String, String>,
    pub properties: BTreeMap<String, (String, Type)>,
    pub types: Vec<NominalType>,
    pub operations: Vec<OperationSpec>,
}

impl DomainSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default_backend: None,
            artifacts: BTreeMap::new(),
            elements: BTreeMap::new(),
            properties: BTreeMap::new(),
            types: Vec::new(),
            operations: Vec::new(),
        }
    }

    pub fn with_default_backend(mut self, backend: impl Into<String>) -> Self {
        self.default_backend = Some(backend.into());
        self
    }

    pub fn with_artifact(
        mut self,
        surface: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        self.artifacts
            .insert(surface.into().to_lowercase(), kind.into());
        self
    }

    pub fn with_element(
        mut self,
        surface: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        self.elements
            .insert(surface.into().to_lowercase(), kind.into());
        self
    }

    pub fn with_property(
        mut self,
        surface: impl Into<String>,
        kind: impl Into<String>,
        ty: Type,
    ) -> Self {
        self.properties
            .insert(surface.into().to_lowercase(), (kind.into(), ty));
        self
    }

    pub fn with_type(mut self, ty: NominalType) -> Self {
        assert_eq!(
            ty.domain, self.name,
            "nominal type '{}' belongs to '{}', expected '{}'",
            ty.name, ty.domain, self.name
        );
        self.types.push(ty);
        self
    }

    pub fn with_operation(mut self, operation: OperationSpec) -> Self {
        self.operations.push(operation);
        self
    }

    pub fn operation(&self, name: &str) -> Option<&OperationSpec> {
        self.operations.iter().find(|operation| operation.name == name)
    }

    pub fn to_json(&self) -> JsonValue {
        let properties = self
            .properties
            .iter()
            .map(|(surface, (kind, _))| (surface.clone(), json!(kind)))
            .collect::<JsonMap<_, _>>();
        let actions = self
            .operations
            .iter()
            .flat_map(|operation| {
                operation
                    .verbs
                    .iter()
                    .map(move |verb| (verb.clone(), json!(operation.name)))
            })
            .collect::<JsonMap<_, _>>();
        json!({
            "name": self.name,
            "default_backend": self.default_backend,
            "artifacts": self.artifacts,
            "elements": self.elements,
            "properties": properties,
            "actions": actions,
            "types": self.types.iter().map(NominalType::to_json).collect::<Vec<_>>(),
            "operations": self.operations.iter().map(OperationSpec::to_json).collect::<Vec<_>>()
        })
    }
}

#[derive(Clone, Debug)]
pub struct DomainRegistry {
    domains: Vec<DomainSpec>,
}

impl Default for DomainRegistry {
    fn default() -> Self {
        Self::builtins()
    }
}

impl DomainRegistry {
    pub fn new() -> Self {
        Self { domains: Vec::new() }
    }

    pub fn builtins() -> Self {
        let mut registry = Self::new();
        for domain in [
            web_domain(),
            structured_data_domain(),
            filesystem_domain(),
            ml_domain(),
        ] {
            registry
                .register(domain)
                .expect("built-in semantic domains must be valid");
        }
        registry
    }

    pub fn register(
        &mut self,
        domain: DomainSpec,
    ) -> std::result::Result<&mut Self, String> {
        if self.domains.iter().any(|existing| existing.name == domain.name) {
            return Err(format!("Domain '{}' is already registered", domain.name));
        }

        let pending = domain_surface_entries(&domain)?;
        for (word, is_action) in &pending {
            if keyword_kind(word).is_some() || is_color(word) {
                return Err(format!(
                    "Semantic domain term '{}' conflicts with reserved English vocabulary",
                    word
                ));
            }

            for existing in &self.domains {
                for (existing_word, existing_is_action) in domain_surface_entries(existing)? {
                    if existing_word != *word {
                        continue;
                    }
                    if *is_action && existing_is_action {
                        continue;
                    }
                    return Err(format!(
                        "Domain term '{}' conflicts between '{}' and '{}'",
                        word, existing.name, domain.name
                    ));
                }
            }
        }

        self.domains.push(domain);
        Ok(self)
    }

    pub fn with_domain(
        mut self,
        domain: DomainSpec,
    ) -> std::result::Result<Self, String> {
        self.register(domain)?;
        Ok(self)
    }

    pub fn words(&self) -> Vec<String> {
        let mut words = self
            .domains
            .iter()
            .flat_map(|domain| domain_surface_entries(domain).unwrap_or_default())
            .map(|(word, _)| word)
            .collect::<Vec<_>>();
        words.sort();
        words.dedup();
        words
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = self
            .domains
            .iter()
            .map(|domain| domain.name.clone())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    pub fn fetch(&self, name: &str) -> Result<&DomainSpec> {
        self.domains
            .iter()
            .find(|domain| domain.name == name)
            .ok_or_else(|| {
                SemauriError::semantic(
                    "S326",
                    format!("Unknown semantic domain '{name}'"),
                    None,
                    None,
                )
            })
    }

    pub fn classify(&self, word: &str) -> Option<TokenLiteral> {
        let key = word.to_lowercase();
        let mut action_candidates = Vec::new();

        for domain in &self.domains {
            if let Some(kind) = domain.artifacts.get(&key) {
                return Some(TokenLiteral::DomainTerm(DomainTerm {
                    domain: domain.name.clone(),
                    category: DomainCategory::Artifact,
                    kind: kind.clone(),
                }));
            }
            if let Some(kind) = domain.elements.get(&key) {
                return Some(TokenLiteral::DomainTerm(DomainTerm {
                    domain: domain.name.clone(),
                    category: DomainCategory::Element,
                    kind: kind.clone(),
                }));
            }
            if let Some((kind, _)) = domain.properties.get(&key) {
                return Some(TokenLiteral::DomainTerm(DomainTerm {
                    domain: domain.name.clone(),
                    category: DomainCategory::Property,
                    kind: kind.clone(),
                }));
            }
            for operation in &domain.operations {
                if operation.verbs.iter().any(|verb| verb == &key) {
                    action_candidates.push(DomainTerm {
                        domain: domain.name.clone(),
                        category: DomainCategory::Action,
                        kind: operation.name.clone(),
                    });
                }
            }
        }

        if action_candidates.is_empty() {
            None
        } else {
            Some(TokenLiteral::ActionCandidates(action_candidates))
        }
    }

    fn infer_property_for_type(&self, value_type: &Type) -> Option<(String, String)> {
        let mut candidates = Vec::new();
        for domain in &self.domains {
            for (_surface, (kind, ty)) in &domain.properties {
                if ty == value_type {
                    let candidate = (domain.name.clone(), kind.clone());
                    if !candidates.contains(&candidate) {
                        candidates.push(candidate);
                    }
                }
            }
        }
        if candidates.len() == 1 {
            candidates.into_iter().next()
        } else {
            None
        }
    }

    fn property_type(&self, domain_name: &str, property: &str) -> Option<Type> {
        self.domains
            .iter()
            .find(|domain| domain.name == domain_name)
            .and_then(|domain| {
                domain
                    .properties
                    .values()
                    .find(|(kind, _)| kind == property)
                    .map(|(_, ty)| ty.clone())
            })
    }

    pub fn to_json(&self) -> JsonValue {
        let mut domains = self.domains.iter().collect::<Vec<_>>();
        domains.sort_by(|a, b| a.name.cmp(&b.name));
        json!({
            "domains": domains.into_iter().map(DomainSpec::to_json).collect::<Vec<_>>()
        })
    }
}

fn domain_surface_entries(
    domain: &DomainSpec,
) -> std::result::Result<Vec<(String, bool)>, String> {
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();

    for surface in domain
        .artifacts
        .keys()
        .chain(domain.elements.keys())
        .chain(domain.properties.keys())
    {
        let word = surface.to_lowercase();
        if !seen.insert(word.clone()) {
            return Err(format!(
                "Duplicate term '{}' in domain '{}'",
                word, domain.name
            ));
        }
        entries.push((word, false));
    }

    for operation in &domain.operations {
        if operation.verbs.is_empty() {
            return Err(format!(
                "Operation '{}' requires at least one verb",
                operation.name
            ));
        }
        let mut slot_names = BTreeSet::new();
        for segment in &operation.pattern {
            if let PatternSegment::Slot { name, .. } = segment {
                if !slot_names.insert(name.clone()) {
                    return Err(format!(
                        "Operation '{}' contains duplicate argument slots",
                        operation.name
                    ));
                }
            }
        }
        for verb in &operation.verbs {
            let word = verb.to_lowercase();
            if word.is_empty() {
                return Err(format!(
                    "Operation '{}' contains an empty verb",
                    operation.name
                ));
            }
            if !seen.insert(word.clone()) {
                return Err(format!(
                    "Duplicate term '{}' in domain '{}'",
                    word, domain.name
                ));
            }
            entries.push((word, true));
        }
    }

    Ok(entries)
}

fn nominal(domain: &str, name: &str, base: Type, promote_from_base: bool) -> Type {
    Type::Nominal(NominalType::new(domain, name, base, promote_from_base))
}

fn filesystem_path_type() -> Type {
    nominal("filesystem", "path", Type::String, true)
}

fn ml_type(name: &str, base: Type) -> Type {
    nominal("ml", name, base, false)
}

fn web_domain() -> DomainSpec {
    DomainSpec {
        name: "web".to_string(),
        default_backend: Some("html".to_string()),
        artifacts: [
            ("web", "web"),
            ("website", "web"),
            ("webpage", "web"),
            ("page", "web"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect(),
        elements: [("button", "button"), ("image", "image"), ("picture", "image")]
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
        properties: [("color", ("color", Type::Color))]
            .into_iter()
            .map(|(surface, (kind, ty))| {
                (surface.to_string(), (kind.to_string(), ty))
            })
            .collect(),
        types: vec![],
        operations: vec![],
    }
}

fn structured_data_domain() -> DomainSpec {
    DomainSpec {
        name: "structured_data".to_string(),
        default_backend: Some("json-schema".to_string()),
        artifacts: [("schema", "schema")]
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
        elements: [("field", "field")]
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect(),
        properties: [
            ("datatype", ("datatype", Type::String)),
            ("required", ("required", Type::Boolean)),
        ]
        .into_iter()
        .map(|(surface, (kind, ty))| {
            (surface.to_string(), (kind.to_string(), ty))
        })
        .collect(),
        types: vec![],
        operations: vec![],
    }
}

fn slot(name: &str, ty: Type) -> PatternSegment {
    PatternSegment::Slot {
        name: name.to_string(),
        ty: Some(ty),
    }
}

fn lit(word: &str) -> PatternSegment {
    PatternSegment::Literal(word.to_string())
}

fn op(
    name: &str,
    verbs: &[&str],
    pattern: Vec<PatternSegment>,
    return_type: Type,
    effects: &[&str],
) -> OperationSpec {
    OperationSpec {
        name: name.to_string(),
        verbs: verbs.iter().map(|value| value.to_string()).collect(),
        pattern,
        return_type,
        effects: effects.iter().map(|value| value.to_string()).collect(),
    }
}

fn filesystem_domain() -> DomainSpec {
    let path = filesystem_path_type();
    DomainSpec {
        name: "filesystem".to_string(),
        default_backend: Some("posix-sh".to_string()),
        artifacts: BTreeMap::new(),
        elements: BTreeMap::new(),
        properties: BTreeMap::new(),
        types: match &path {
            Type::Nominal(value) => vec![value.clone()],
            _ => vec![],
        },
        operations: vec![
            op(
                "write",
                &["write"],
                vec![slot("content", Type::String), lit("to"), slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "append",
                &["append"],
                vec![slot("content", Type::String), lit("to"), slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "copy",
                &["copy"],
                vec![slot("source", path.clone()), lit("to"), slot("destination", path.clone())],
                Type::Unit,
                &["filesystem_read", "filesystem_write"],
            ),
            op(
                "move",
                &["move"],
                vec![slot("source", path.clone()), lit("to"), slot("destination", path.clone())],
                Type::Unit,
                &["filesystem_read", "filesystem_write"],
            ),
            op(
                "make_directory",
                &["mkdir"],
                vec![slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "touch",
                &["touch"],
                vec![slot("path", path.clone())],
                Type::Unit,
                &["filesystem_write"],
            ),
            op(
                "delete",
                &["delete", "remove"],
                vec![slot("path", path)],
                Type::Unit,
                &["filesystem_write"],
            ),
        ],
    }
}

fn ml_domain() -> DomainSpec {
    let dataset = ml_type("dataset", Type::String);
    let model = ml_type("model", Type::String);
    let device = ml_type("device", Type::String);
    let resource_budget = ml_type("resource_budget", Type::Opaque);
    let training_config = ml_type("training_config", Type::Opaque);
    let training_run = ml_type("training_run", Type::Opaque);
    let inference_run = ml_type("inference_run", Type::Opaque);
    let evaluation_run = ml_type("evaluation_run", Type::Opaque);

    let types = [
        &dataset,
        &model,
        &device,
        &resource_budget,
        &training_config,
        &training_run,
        &inference_run,
        &evaluation_run,
    ]
    .iter()
    .filter_map(|ty| match ty {
        Type::Nominal(value) => Some(value.clone()),
        _ => None,
    })
    .collect();

    DomainSpec {
        name: "ml".to_string(),
        default_backend: None,
        artifacts: BTreeMap::new(),
        elements: BTreeMap::new(),
        properties: BTreeMap::new(),
        types,
        operations: vec![
            op(
                "open_dataset",
                &["open"],
                vec![lit("dataset"), slot("source", Type::String)],
                dataset.clone(),
                &["filesystem_read"],
            ),
            op(
                "transform_dataset",
                &["transform"],
                vec![slot("dataset", dataset.clone()), lit("using"), slot("transform", Type::String)],
                dataset.clone(),
                &[],
            ),
            op(
                "split_dataset",
                &["split"],
                vec![
                    slot("dataset", dataset.clone()),
                    lit("ratio"),
                    slot("ratio", Type::Number),
                    lit("seed"),
                    slot("seed", Type::Number),
                ],
                dataset.clone(),
                &[],
            ),
            op(
                "load_model",
                &["load"],
                vec![lit("model"), slot("identifier", Type::String)],
                model.clone(),
                &["model_load"],
            ),
            op(
                "build_cnn",
                &["build"],
                vec![
                    lit("cnn"),
                    lit("for"),
                    slot("classes", Type::Number),
                    lit("classes"),
                    lit("input"),
                    lit("channels"),
                    slot("input_channels", Type::Number),
                ],
                model.clone(),
                &[],
            ),
            op(
                "freeze_component",
                &["freeze"],
                vec![
                    slot("model", model.clone()),
                    lit("component"),
                    slot("component", Type::String),
                ],
                model.clone(),
                &[],
            ),
            op(
                "apply_lora",
                &["apply"],
                vec![
                    lit("lora"),
                    lit("to"),
                    slot("model", model.clone()),
                    lit("rank"),
                    slot("rank", Type::Number),
                    lit("alpha"),
                    slot("alpha", Type::Number),
                ],
                model.clone(),
                &[],
            ),
            op(
                "apply_qlora",
                &["adapt"],
                vec![
                    slot("model", model.clone()),
                    lit("with"),
                    lit("qlora"),
                    lit("rank"),
                    slot("rank", Type::Number),
                    lit("alpha"),
                    slot("alpha", Type::Number),
                    lit("quantization"),
                    slot("quantization_bits", Type::Number),
                    lit("bits"),
                    lit("targets"),
                    slot("targets", Type::String),
                ],
                model.clone(),
                &[],
            ),
            op(
                "select_device",
                &["select"],
                vec![lit("device"), slot("name", Type::String)],
                device.clone(),
                &[],
            ),
            op(
                "budget_resources",
                &["budget"],
                vec![
                    lit("resources"),
                    lit("memory"),
                    slot("memory_gb", Type::Number),
                    lit("gb"),
                    lit("workers"),
                    slot("workers", Type::Number),
                ],
                resource_budget.clone(),
                &[],
            ),
            op(
                "configure_training",
                &["configure"],
                vec![
                    lit("training"),
                    lit("for"),
                    slot("epochs", Type::Number),
                    lit("epochs"),
                    lit("using"),
                    lit("optimizer"),
                    slot("optimizer", Type::String),
                    lit("learning"),
                    lit("rate"),
                    slot("learning_rate", Type::Number),
                    lit("batch"),
                    lit("size"),
                    slot("batch_size", Type::Number),
                    lit("seed"),
                    slot("seed", Type::Number),
                ],
                training_config.clone(),
                &[],
            ),
            op(
                "plan_training",
                &["plan"],
                vec![
                    lit("training"),
                    lit("for"),
                    slot("epochs", Type::Number),
                    lit("epochs"),
                    lit("using"),
                    lit("optimizer"),
                    slot("optimizer", Type::String),
                    lit("learning"),
                    lit("rate"),
                    slot("learning_rate", Type::Number),
                    lit("batch"),
                    lit("size"),
                    slot("batch_size", Type::Number),
                    lit("seed"),
                    slot("seed", Type::Number),
                    lit("precision"),
                    slot("precision", Type::String),
                    lit("accumulate"),
                    slot("gradient_accumulation", Type::Number),
                    lit("steps"),
                    lit("checkpoint"),
                    lit("every"),
                    slot("checkpoint_every", Type::Number),
                    lit("steps"),
                ],
                training_config.clone(),
                &[],
            ),
            op(
                "train",
                &["train"],
                vec![
                    slot("model", model.clone()),
                    lit("using"),
                    slot("dataset", dataset.clone()),
                    lit("on"),
                    slot("device", device.clone()),
                    lit("for"),
                    slot("epochs", Type::Number),
                    lit("epochs"),
                ],
                training_run.clone(),
                &["compute", "model_training"],
            ),
            op(
                "fit",
                &["fit"],
                vec![
                    slot("model", model.clone()),
                    lit("using"),
                    slot("dataset", dataset.clone()),
                    lit("on"),
                    slot("device", device.clone()),
                    lit("with"),
                    slot("config", training_config.clone()),
                ],
                training_run.clone(),
                &["compute", "model_training"],
            ),
            op(
                "fit_budgeted",
                &["allocate"],
                vec![
                    slot("budget", resource_budget.clone()),
                    lit("to"),
                    lit("fit"),
                    slot("model", model.clone()),
                    lit("using"),
                    slot("dataset", dataset.clone()),
                    lit("on"),
                    slot("device", device.clone()),
                    lit("with"),
                    slot("config", training_config.clone()),
                ],
                training_run,
                &["compute", "model_training"],
            ),
            op(
                "infer",
                &["infer"],
                vec![
                    slot("model", model.clone()),
                    lit("on"),
                    slot("dataset", dataset.clone()),
                    lit("using"),
                    slot("device", device.clone()),
                ],
                inference_run,
                &["compute", "model_inference"],
            ),
            op(
                "evaluate",
                &["evaluate"],
                vec![
                    slot("model", model),
                    lit("on"),
                    slot("dataset", dataset),
                    lit("using"),
                    slot("device", device),
                    lit("metric"),
                    slot("metric", Type::String),
                ],
                evaluation_run,
                &["compute", "model_evaluation"],
            ),
        ],
    }
}

