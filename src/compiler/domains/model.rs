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
