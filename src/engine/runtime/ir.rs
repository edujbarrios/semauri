#[derive(Clone, Debug)]
pub struct Element {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub properties: BTreeMap<String, ValueData>,
    pub property_provenance: BTreeMap<String, SourceSpan>,
}

impl Element {
    fn to_json(&self) -> JsonValue {
        json!({
            "id": self.id,
            "kind": self.kind,
            "label": self.label,
            "properties": self.properties.iter().map(|(key, value)| (key.clone(), value.to_json())).collect::<JsonMap<_,_>>(),
            "property_provenance": self.property_provenance
        })
    }
}

#[derive(Clone, Debug)]
pub struct WebDocument {
    pub title: String,
    pub subject: Option<String>,
    pub title_origin: String,
    pub elements: Vec<Element>,
}

impl WebDocument {
    fn to_json(&self) -> JsonValue {
        json!({
            "type": "web_document",
            "title": self.title,
            "subject": self.subject,
            "title_origin": self.title_origin,
            "elements": self.elements.iter().map(Element::to_json).collect::<Vec<_>>()
        })
    }
}

#[derive(Clone, Debug)]
pub struct SchemaField {
    pub id: String,
    pub label: String,
    pub properties: BTreeMap<String, ValueData>,
    pub property_sources: BTreeMap<String, SourceSpan>,
}

impl SchemaField {
    fn to_json(&self) -> JsonValue {
        json!({
            "id": self.id,
            "kind": "field",
            "label": self.label,
            "properties": self.properties.iter().map(|(key, value)| (key.clone(), value.to_json())).collect::<JsonMap<_,_>>(),
            "property_sources": self.property_sources
        })
    }
}

#[derive(Clone, Debug)]
pub struct SchemaDocument {
    pub title: String,
    pub fields: Vec<SchemaField>,
}

impl SchemaDocument {
    fn to_json(&self) -> JsonValue {
        json!({
            "type": "schema_document",
            "title": self.title,
            "fields": self.fields.iter().map(SchemaField::to_json).collect::<Vec<_>>()
        })
    }
}

#[derive(Clone, Debug)]
pub struct OperationIr {
    pub name: String,
    pub arguments: BTreeMap<String, ValueData>,
    pub effects: Vec<String>,
    pub source_span: SourceSpan,
}

impl OperationIr {
    fn to_json(&self) -> JsonValue {
        json!({
            "name": self.name,
            "arguments": self.arguments.iter().map(|(key, value)| (key.clone(), value.to_json())).collect::<JsonMap<_,_>>(),
            "effects": self.effects,
            "source_span": self.source_span
        })
    }
}

#[derive(Clone, Debug)]
pub struct OperationPlan {
    pub domain: String,
    pub operations: Vec<OperationIr>,
}

impl OperationPlan {
    fn to_json(&self) -> JsonValue {
        json!({
            "domain": self.domain,
            "operations": self.operations.iter().map(OperationIr::to_json).collect::<Vec<_>>()
        })
    }
}

#[derive(Clone, Debug)]
pub struct GenericElement {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub properties: BTreeMap<String, ValueData>,
    pub property_provenance: BTreeMap<String, SourceSpan>,
}

impl GenericElement {
    fn to_json(&self) -> JsonValue {
        json!({
            "id": self.id,
            "kind": self.kind,
            "label": self.label,
            "properties": self.properties.iter().map(|(key, value)| (key.clone(), value.to_json())).collect::<JsonMap<_,_>>(),
            "property_provenance": self.property_provenance
        })
    }
}

#[derive(Clone, Debug)]
pub struct GenericArtifact {
    pub domain: String,
    pub kind: String,
    pub title: String,
    pub subject: Option<String>,
    pub elements: Vec<GenericElement>,
}

impl GenericArtifact {
    fn to_json(&self) -> JsonValue {
        json!({
            "type": "generic_artifact",
            "domain": self.domain,
            "kind": self.kind,
            "title": self.title,
            "subject": self.subject,
            "elements": self.elements.iter().map(GenericElement::to_json).collect::<Vec<_>>()
        })
    }
}

#[derive(Clone, Debug)]
pub enum Artifact {
    Web(WebDocument),
    Schema(SchemaDocument),
    Filesystem(OperationPlan),
    Generic(GenericArtifact),
}

impl Artifact {
    pub fn to_json(&self) -> JsonValue {
        match self {
            Artifact::Web(value) => value.to_json(),
            Artifact::Schema(value) => value.to_json(),
            Artifact::Filesystem(value) => value.to_json(),
            Artifact::Generic(value) => value.to_json(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProgramUnit {
    pub domain: String,
    pub artifact: Artifact,
}

#[derive(Clone, Debug, Default)]
pub struct ProgramIr {
    pub units: Vec<ProgramUnit>,
}

impl ProgramIr {
    fn include(&self, domain: &str) -> bool {
        self.units.iter().any(|unit| unit.domain == domain)
    }

    fn artifact(&self, domain: &str) -> Option<&Artifact> {
        self.units
            .iter()
            .find(|unit| unit.domain == domain)
            .map(|unit| &unit.artifact)
    }

    fn put(&mut self, domain: &str, artifact: Artifact) {
        if let Some(unit) = self.units.iter_mut().find(|unit| unit.domain == domain) {
            unit.artifact = artifact;
        } else {
            self.units.push(ProgramUnit {
                domain: domain.to_string(),
                artifact,
            });
        }
    }

    fn to_json(&self) -> JsonValue {
        json!({
            "kind": "program_ir",
            "domains": self.units.iter().map(|unit| unit.domain.clone()).collect::<Vec<_>>(),
            "units": self.units.iter().map(|unit| json!({
                "domain": unit.domain,
                "artifact": unit.artifact.to_json()
            })).collect::<Vec<_>>()
        })
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeValueRef {
    pub id: String,
    pub ty: Type,
    pub producer_id: usize,
    pub source_span: SourceSpan,
}

impl RuntimeValueRef {
    fn to_json(&self) -> JsonValue {
        json!({
            "kind": "runtime_value_ref",
            "id": self.id,
            "type": self.ty.to_json(),
            "producer_id": self.producer_id,
            "source_span": self.source_span
        })
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeArgument {
    Value(ValueData),
    Ref(RuntimeValueRef),
}

impl RuntimeArgument {
    fn to_json(&self) -> JsonValue {
        match self {
            RuntimeArgument::Value(value) => value.to_json(),
            RuntimeArgument::Ref(reference) => reference.to_json(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeOperation {
    pub id: usize,
    pub domain: String,
    pub name: String,
    pub arguments: BTreeMap<String, RuntimeArgument>,
    pub return_type: Type,
    pub result: Option<RuntimeValueRef>,
    pub effects: Vec<String>,
    pub source_span: SourceSpan,
}

impl RuntimeOperation {
    fn to_json(&self) -> JsonValue {
        json!({
            "id": self.id,
            "domain": self.domain,
            "name": self.name,
            "arguments": self.arguments.iter().map(|(key, value)| (key.clone(), value.to_json())).collect::<JsonMap<_,_>>(),
            "return_type": self.return_type.to_json(),
            "result": self.result.as_ref().map(RuntimeValueRef::to_json),
            "effects": self.effects,
            "source_span": self.source_span
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuntimePlan {
    pub operations: Vec<RuntimeOperation>,
}

impl RuntimePlan {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "kind": "runtime_plan",
            "operations": self.operations.iter().map(RuntimeOperation::to_json).collect::<Vec<_>>()
        })
    }
}

#[derive(Clone, Debug)]
enum BoundValue {
    Compile { ty: Type, data: ValueData },
    Runtime(RuntimeValueRef),
}

impl BoundValue {
    fn ty(&self) -> Type {
        match self {
            BoundValue::Compile { ty, .. } => ty.clone(),
            BoundValue::Runtime(reference) => reference.ty.clone(),
        }
    }

    fn describe(&self) -> String {
        match self {
            BoundValue::Compile { ty, data } => data.describe(ty),
            BoundValue::Runtime(reference) => {
                format!("runtime {} {}", reference.ty, reference.id)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct EntityRecord {
    id: String,
    kind: String,
    label: String,
}

#[derive(Clone, Debug)]
pub struct SemanticResult {
    pub program_ir: ProgramIr,
    pub runtime_plan: RuntimePlan,
    pub explanations: Vec<String>,
    pub symbols: Vec<Symbol>,
}

impl SemanticResult {
    pub fn runtime(&self) -> bool {
        !self.runtime_plan.operations.is_empty()
    }

    pub fn multi_domain(&self) -> bool {
        self.program_ir.units.len() > 1
    }

    pub fn domains(&self) -> Vec<String> {
        self.program_ir
            .units
            .iter()
            .map(|unit| unit.domain.clone())
            .collect()
    }

    pub fn program_json(&self) -> JsonValue {
        if self.program_ir.units.len() == 1 {
            self.program_ir.units[0].artifact.to_json()
        } else {
            self.program_ir.to_json()
        }
    }
}

struct Lowerer<'a> {
    domains: &'a DomainRegistry,
    symbols: Vec<Symbol>,
    environments: Vec<HashMap<u64, BoundValue>>,
    program: ProgramIr,
    runtime_plan: RuntimePlan,
    focus_domain: Option<String>,
    domain_scope_stack: Vec<String>,
    entities: HashMap<String, Vec<EntityRecord>>,
    kind_counts: HashMap<(String, String), usize>,
    explanations: Vec<String>,
}

impl<'a> Lowerer<'a> {
    fn new(domains: &'a DomainRegistry, symbols: &[Symbol]) -> Self {
        Self {
            domains,
            symbols: symbols.to_vec(),
            environments: vec![HashMap::new()],
            program: ProgramIr::default(),
            runtime_plan: RuntimePlan::default(),
            focus_domain: None,
            domain_scope_stack: Vec::new(),
            entities: HashMap::new(),
            kind_counts: HashMap::new(),
            explanations: Vec::new(),
        }
    }

    fn lower(mut self, hir: &HirResult) -> Result<SemanticResult> {
        self.lower_statements(&hir.statements)?;
        if self.program.units.is_empty() && self.runtime_plan.operations.is_empty() {
            return Err(SemauriError::semantic(
                "S301",
                "Program does not create or produce semantic output",
                None,
                None,
            ));
        }
        Ok(SemanticResult {
            program_ir: self.program,
            runtime_plan: self.runtime_plan,
            explanations: self.explanations,
            symbols: self.symbols,
        })
    }

    fn lower_statements(&mut self, statements: &[HirStmt]) -> Result<()> {
        for statement in statements {
            self.lower_statement(statement)?;
        }
        Ok(())
    }

    fn lower_statement(&mut self, statement: &HirStmt) -> Result<()> {
        match statement {
            HirStmt::DomainScope { domain, body, .. } => {
                self.domains.fetch(domain)?;
                self.domain_scope_stack.push(domain.clone());
                self.explanations
                    .push(format!("Entered semantic domain scope '{domain}'."));
                self.push_environment();
                let result = self.lower_statements(body);
                self.pop_environment();
                self.domain_scope_stack.pop();
                result
            }
            HirStmt::Let {
                symbol_id,
                source_name,
                value,
                span,
                ..
            } => {
                let value = self.lower_runtime_value(value)?;
                let symbol = self.symbol(*symbol_id, *span)?;
                if symbol.ty != value.ty() {
                    return Err(SemauriError::semantic(
                        "S324",
                        format!(
                            "HIR symbol #{} expects {}, received {}",
                            symbol.id,
                            symbol.ty,
                            value.ty()
                        ),
                        Some(*span),
                        None,
                    ));
                }
                self.environments
                    .last_mut()
                    .unwrap()
                    .insert(*symbol_id, value.clone());
                self.explanations.push(format!(
                    "Bound '{}' as symbol #{} to {}.",
                    source_name,
                    symbol_id,
                    value.describe()
                ));
                Ok(())
            }
            HirStmt::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                let condition_span = condition.span();
                let condition = self.evaluate(condition)?;
                let BoundValue::Compile {
                    ty: Type::Boolean,
                    data: ValueData::Boolean(value),
                } = condition
                else {
                    return Err(SemauriError::semantic(
                        "S316",
                        "If condition must evaluate to boolean",
                        Some(condition_span),
                        None,
                    ));
                };
                self.explanations.push(format!(
                    "If condition evaluated to {}; selected {} branch.",
                    value,
                    if value {
                        "consequence"
                    } else {
                        "alternative"
                    }
                ));
                if value {
                    self.push_environment();
                    let result = self.lower_statements(consequence);
                    self.pop_environment();
                    result
                } else if let Some(alternative) = alternative {
                    self.push_environment();
                    let result = self.lower_statements(alternative);
                    self.pop_environment();
                    result
                } else {
                    Ok(())
                }
            }
            HirStmt::ForEach {
                iterator_symbol_id,
                iterator_source_name,
                iterable,
                body,
                span,
                ..
            } => {
                let iterable_value = self.evaluate(iterable)?;
                let (element_type, values) = match iterable_value {
                    BoundValue::Compile {
                        ty: Type::List(element_type),
                        data: ValueData::List(values),
                    } => (*element_type, values),
                    BoundValue::Compile { ty, .. } => {
                        return Err(SemauriError::semantic(
                            "S322",
                            format!("For every requires a list, received {ty}"),
                            Some(*span),
                            None,
                        ))
                    }
                    BoundValue::Runtime(_) => {
                        return Err(self.runtime_value_error(*span));
                    }
                };
                let symbol = self.symbol(*iterator_symbol_id, *span)?;
                self.explanations.push(format!(
                    "For every '{}' iterates over {} {} value(s) as symbol #{}.",
                    iterator_source_name,
                    values.len(),
                    element_type,
                    iterator_symbol_id
                ));
                for item in values {
                    self.push_environment();
                    self.environments.last_mut().unwrap().insert(
                        *iterator_symbol_id,
                        BoundValue::Compile {
                            ty: symbol.ty.clone(),
                            data: item,
                        },
                    );
                    let result = self.lower_statements(body);
                    self.pop_environment();
                    result?;
                }
                Ok(())
            }
            HirStmt::CreateArtifact {
                domain,
                artifact_kind,
                subject,
                title,
                span,
            } => self.lower_create_artifact(
                domain,
                artifact_kind,
                subject.clone(),
                title.clone(),
                *span,
            ),
            HirStmt::DomainOperation {
                domain,
                operation,
                arguments,
                effects,
                return_type,
                span,
            } => {
                let expr = HirExpr::DomainOperation {
                    ty: return_type.clone(),
                    domain: domain.clone(),
                    operation: operation.clone(),
                    arguments: arguments.clone(),
                    effects: effects.clone(),
                    span: *span,
                };
                if *return_type != Type::Unit || self.runtime_dependent_arguments(arguments)? {
                    let _ = self.append_runtime_operation(&expr)?;
                    Ok(())
                } else {
                    self.lower_compile_time_operation(
                        domain,
                        operation,
                        arguments,
                        effects,
                        *span,
                    )
                }
            }
            HirStmt::SetTitle { title, span } => {
                let domain = self.focus_domain.clone().ok_or_else(|| {
                    SemauriError::semantic(
                        "S303",
                        "Cannot add a title before creating an artifact",
                        Some(*span),
                        Some("Create an artifact first.".to_string()),
                    )
                })?;
                let artifact = self.program.artifact(&domain).cloned().ok_or_else(|| {
                    SemauriError::semantic(
                        "S303",
                        "Cannot add a title before creating an artifact",
                        Some(*span),
                        Some("Create an artifact first.".to_string()),
                    )
                })?;
                let updated = match artifact {
                    Artifact::Web(mut web) => {
                        web.title = title.clone();
                        web.title_origin = "explicit".to_string();
                        Artifact::Web(web)
                    }
                    Artifact::Schema(mut schema) => {
                        schema.title = title.clone();
                        Artifact::Schema(schema)
                    }
                    Artifact::Generic(mut generic) => {
                        generic.title = title.clone();
                        Artifact::Generic(generic)
                    }
                    Artifact::Filesystem(_) => {
                        return Err(SemauriError::semantic(
                            "S303",
                            "Cannot add a title to this domain",
                            Some(*span),
                            None,
                        ))
                    }
                };
                self.program.put(&domain, updated);
                self.explanations
                    .push(format!("Title explicitly set to '{title}'."));
                Ok(())
            }
            HirStmt::AddElement {
                domain,
                element_kind,
                label,
                span,
            } => self.lower_add_element(domain, element_kind, label.clone(), *span),
            HirStmt::SetProperty {
                domain,
                target,
                property,
                value,
                span,
            } => self.lower_set_property(domain, target, property, value, *span),
        }
    }

    fn lower_create_artifact(
        &mut self,
        domain: &str,
        kind: &str,
        subject: Option<String>,
        explicit_title: Option<String>,
        span: SourceSpan,
    ) -> Result<()> {
        if self.program.include(domain) {
            return Err(SemauriError::semantic(
                "S302",
                "Semauri currently supports one artifact/plan per semantic domain in a source file",
                Some(span),
                Some(format!(
                    "Reuse the existing {domain} domain output or split independent {domain} artifacts into separate .sema files."
                )),
            ));
        }
        let artifact = match domain {
            "web" if kind == "web" => {
                let (title, origin) = if let Some(title) = explicit_title.clone() {
                    (title, "explicit".to_string())
                } else if let Some(subject) = subject.clone() {
                    (subject, "subject_default".to_string())
                } else {
                    ("Untitled".to_string(), "fallback".to_string())
                };
                self.explanations
                    .push("'web' resolved to an HTML web document (default web backend).".to_string());
                if let Some(subject) = &subject {
                    self.explanations
                        .push(format!("Subject resolved to '{subject}'."));
                }
                self.explanations.push(match origin.as_str() {
                    "explicit" => format!("Title explicitly set to '{title}'."),
                    "subject_default" => format!(
                        "No title was provided, so the web title defaults to its subject: '{title}'."
                    ),
                    _ => "No title or subject was provided, so the web title defaults to 'Untitled'."
                        .to_string(),
                });
                Artifact::Web(WebDocument {
                    title,
                    subject,
                    title_origin: origin,
                    elements: Vec::new(),
                })
            }
            "structured_data" if kind == "schema" => {
                let title = explicit_title
                    .clone()
                    .or(subject.clone())
                    .unwrap_or_else(|| "Untitled Schema".to_string());
                self.explanations
                    .push("'schema' resolved to a structured-data schema.".to_string());
                self.explanations
                    .push(format!("Schema title resolved to '{title}'."));
                Artifact::Schema(SchemaDocument {
                    title,
                    fields: Vec::new(),
                })
            }
            _ => {
                let specification = self.domains.fetch(domain)?;
                if !specification.artifacts.values().any(|candidate| candidate == kind) {
                    return Err(SemauriError::semantic(
                        "S326",
                        format!("Unsupported artifact '{kind}' in domain '{domain}'"),
                        Some(span),
                        None,
                    ));
                }
                let title = explicit_title
                    .clone()
                    .or(subject.clone())
                    .unwrap_or_else(|| format!("Untitled {}", capitalize(kind)));
                self.explanations.push(format!(
                    "Created generic '{}' artifact for semantic domain '{}'.",
                    kind, domain
                ));
                Artifact::Generic(GenericArtifact {
                    domain: domain.to_string(),
                    kind: kind.to_string(),
                    title,
                    subject,
                    elements: Vec::new(),
                })
            }
        };
        self.program.put(domain, artifact);
        self.focus_domain = Some(domain.to_string());
        self.entities.entry(domain.to_string()).or_default();
        Ok(())
    }

    fn lower_add_element(
        &mut self,
        domain: &str,
        kind: &str,
        label: Option<String>,
        span: SourceSpan,
    ) -> Result<()> {
        let artifact = self.program.artifact(domain).cloned().ok_or_else(|| {
            SemauriError::semantic(
                "S307",
                format!(
                    "Cannot use a {domain} domain operation before that domain has produced an artifact/plan"
                ),
                Some(span),
                Some(format!("Create a {domain} artifact first.")),
            )
        })?;
        let count = self
            .kind_counts
            .entry((domain.to_string(), kind.to_string()))
            .and_modify(|count| *count += 1)
            .or_insert(1);
        let id = format!("{kind}-{count}");
        let label = label.unwrap_or_else(|| capitalize(kind));
        let updated = match artifact {
            Artifact::Web(mut web) => {
                let element = Element {
                    id: id.clone(),
                    kind: kind.to_string(),
                    label: label.clone(),
                    properties: BTreeMap::new(),
                    property_provenance: BTreeMap::new(),
                };
                web.elements.push(element);
                Artifact::Web(web)
            }
            Artifact::Schema(mut schema) if kind == "field" => {
                let mut properties = BTreeMap::new();
                properties.insert("datatype".to_string(), ValueData::String("string".to_string()));
                properties.insert("required".to_string(), ValueData::Boolean(false));
                schema.fields.push(SchemaField {
                    id: id.clone(),
                    label: label.clone(),
                    properties,
                    property_sources: BTreeMap::new(),
                });
                Artifact::Schema(schema)
            }
            Artifact::Generic(mut generic) => {
                let specification = self.domains.fetch(domain)?;
                if !specification.elements.values().any(|candidate| candidate == kind) {
                    return Err(SemauriError::semantic(
                        "S307",
                        format!("Domain '{domain}' does not support element '{kind}'"),
                        Some(span),
                        None,
                    ));
                }
                generic.elements.push(GenericElement {
                    id: id.clone(),
                    kind: kind.to_string(),
                    label: label.clone(),
                    properties: BTreeMap::new(),
                    property_provenance: BTreeMap::new(),
                });
                Artifact::Generic(generic)
            }
            _ => {
                return Err(SemauriError::semantic(
                    "S307",
                    format!("Domain '{domain}' does not support element '{kind}'"),
                    Some(span),
                    None,
                ))
            }
        };
        self.program.put(domain, updated);
        self.focus_domain = Some(domain.to_string());
        self.entities
            .entry(domain.to_string())
            .or_default()
            .push(EntityRecord {
                id: id.clone(),
                kind: kind.to_string(),
                label: label.clone(),
            });
        self.explanations
            .push(format!("Added {kind} '{label}' as {id}."));
        Ok(())
    }

    fn lower_set_property(
        &mut self,
        domain: &str,
        reference: &HirReference,
        property: &str,
        value: &HirExpr,
        span: SourceSpan,
    ) -> Result<()> {
        let artifact = self.program.artifact(domain).cloned().ok_or_else(|| {
            SemauriError::semantic(
                "S307",
                format!(
                    "Cannot use a {domain} domain operation before that domain has produced an artifact/plan"
                ),
                Some(span),
                Some(format!("Create a {domain} artifact first.")),
            )
        })?;
        let target = self.resolve_reference(domain, reference)?;
        let evaluated = self.evaluate(value)?;
        let (value_type, value_data) = match evaluated {
            BoundValue::Compile { ty, data } => (ty, data),
            BoundValue::Runtime(_) => return Err(self.runtime_value_error(value.span())),
        };

        if let Some(expected) = self.domains.property_type(domain, property) {
            if expected != value_type {
                return Err(SemauriError::semantic(
                    "S313",
                    format!(
                        "Property '{}' in domain '{}' expects {}, but received {}",
                        property, domain, expected, value_type
                    ),
                    Some(value.span()),
                    Some(format!(
                        "Use a {} literal or a variable containing a {}.",
                        expected, expected
                    )),
                ));
            }
        }

        let updated = match artifact {
            Artifact::Web(mut web) => {
                let element = web
                    .elements
                    .iter_mut()
                    .find(|element| element.id == target.id)
                    .ok_or_else(|| {
                        SemauriError::semantic(
                            "S309",
                            format!("No {} called '{}' exists", target.kind, target.label),
                            Some(reference.span()),
                            None,
                        )
                    })?;
                element
                    .properties
                    .insert(property.to_string(), value_data.clone());
                element
                    .property_provenance
                    .insert(property.to_string(), span);
                Artifact::Web(web)
            }
            Artifact::Schema(mut schema) => {
                if property == "datatype" {
                    if let ValueData::String(datatype) = &value_data {
                        if !matches!(
                            datatype.as_str(),
                            "string" | "number" | "integer" | "boolean" | "object" | "array"
                        ) {
                            return Err(SemauriError::semantic(
                                "S328",
                                format!("Unsupported schema datatype '{datatype}'"),
                                Some(span),
                                Some(
                                    "Use one of: string, number, integer, boolean, object, array."
                                        .to_string(),
                                ),
                            ));
                        }
                    }
                }
                let field = schema
                    .fields
                    .iter_mut()
                    .find(|field| field.id == target.id)
                    .ok_or_else(|| {
                        SemauriError::semantic(
                            "S309",
                            format!("No field called '{}' exists", target.label),
                            Some(reference.span()),
                            None,
                        )
                    })?;
                field
                    .properties
                    .insert(property.to_string(), value_data.clone());
                field
                    .property_sources
                    .insert(property.to_string(), span);
                Artifact::Schema(schema)
            }
            Artifact::Generic(mut generic) => {
                let element = generic
                    .elements
                    .iter_mut()
                    .find(|element| element.id == target.id)
                    .ok_or_else(|| {
                        SemauriError::semantic(
                            "S309",
                            format!("No {} called '{}' exists", target.kind, target.label),
                            Some(reference.span()),
                            None,
                        )
                    })?;
                element
                    .properties
                    .insert(property.to_string(), value_data.clone());
                element
                    .property_provenance
                    .insert(property.to_string(), span);
                Artifact::Generic(generic)
            }
            Artifact::Filesystem(_) => {
                return Err(SemauriError::semantic(
                    "S307",
                    "Filesystem plans do not support properties",
                    Some(span),
                    None,
                ))
            }
        };
        self.program.put(domain, updated);
        self.explanations.push(match reference {
            HirReference::Pronoun { pronoun, .. } => format!(
                "'{}' resolved to {} '{}' ({}).",
                pronoun, target.kind, target.label, target.id
            ),
            HirReference::Named { .. } => format!(
                "Explicit reference resolved to {} '{}' ({}).",
                target.kind, target.label, target.id
            ),
        });
        self.explanations.push(format!(
            "Set {}.{} to {}.",
            target.id,
            property,
            inspect_value(&value_data)
        ));
        Ok(())
    }

    fn resolve_reference(&self, domain: &str, reference: &HirReference) -> Result<EntityRecord> {
        let entities = self.entities.get(domain).cloned().unwrap_or_default();
        match reference {
            HirReference::Pronoun { pronoun, span } => {
                if entities.is_empty() {
                    return Err(SemauriError::semantic(
                        "S304",
                        format!("Pronoun '{pronoun}' has no object to refer to"),
                        Some(*span),
                        Some("Add an element before referring to it.".to_string()),
                    ));
                }
                if entities.len() > 1 {
                    let candidates = entities
                        .iter()
                        .map(|entity| format!("{} '{}'", entity.kind, entity.label))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(SemauriError::semantic(
                        "S305",
                        format!("Pronoun '{pronoun}' is ambiguous"),
                        Some(*span),
                        Some(format!(
                            "Possible references: {candidates}. Use an explicit reference such as 'the button called Buy'."
                        )),
                    ));
                }
                Ok(entities[0].clone())
            }
            HirReference::Named {
                domain: reference_domain,
                kind,
                label,
                span,
            } => {
                if reference_domain != domain {
                    return Err(SemauriError::semantic(
                        "S327",
                        format!(
                            "Reference belongs to domain '{}', expected '{}'",
                            reference_domain, domain
                        ),
                        Some(*span),
                        None,
                    ));
                }
                let candidates = entities
                    .into_iter()
                    .filter(|entity| {
                        entity.kind == *kind && entity.label.eq_ignore_ascii_case(label)
                    })
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    return Err(SemauriError::semantic(
                        "S309",
                        format!("No {kind} called '{label}' exists"),
                        Some(*span),
                        None,
                    ));
                }
                if candidates.len() > 1 {
                    return Err(SemauriError::semantic(
                        "S310",
                        format!("Reference to {kind} '{label}' is ambiguous"),
                        Some(*span),
                        Some(
                            "Give elements unique names before referring to them explicitly."
                                .to_string(),
                        ),
                    ));
                }
                Ok(candidates[0].clone())
            }
        }
    }

    fn lower_compile_time_operation(
        &mut self,
        domain: &str,
        operation: &str,
        arguments: &BTreeMap<String, HirExpr>,
        effects: &[String],
        span: SourceSpan,
    ) -> Result<()> {
        if let Some(scope) = self.domain_scope_stack.last() {
            if scope != domain {
                return Err(SemauriError::semantic(
                    "S332",
                    format!(
                        "Operation '{}' belongs to domain '{}', but the active semantic scope is '{}'",
                        operation, domain, scope
                    ),
                    Some(span),
                    None,
                ));
            }
        }
        if domain != "filesystem" {
            let expr = HirExpr::DomainOperation {
                ty: Type::Unit,
                domain: domain.to_string(),
                operation: operation.to_string(),
                arguments: arguments.clone(),
                effects: effects.to_vec(),
                span,
            };
            let _ = self.append_runtime_operation(&expr)?;
            return Ok(());
        }
        let mut resolved = BTreeMap::new();
        for (name, value) in arguments {
            match self.evaluate(value)? {
                BoundValue::Compile { data, .. } => {
                    resolved.insert(name.clone(), data);
                }
                BoundValue::Runtime(_) => return Err(self.runtime_value_error(value.span())),
            }
        }
        let mut plan = match self.program.artifact("filesystem").cloned() {
            Some(Artifact::Filesystem(plan)) => plan,
            Some(_) => {
                return Err(SemauriError::semantic(
                    "S331",
                    "Invalid filesystem domain artifact",
                    Some(span),
                    None,
                ))
            }
            None => OperationPlan {
                domain: "filesystem".to_string(),
                operations: Vec::new(),
            },
        };
        plan.operations.push(OperationIr {
            name: operation.to_string(),
            arguments: resolved,
            effects: effects.to_vec(),
            source_span: span,
        });
        self.program.put("filesystem", Artifact::Filesystem(plan));
        self.explanations.push(format!(
            "Planned filesystem operation '{}' with effects {}.",
            operation,
            effects.join(", ")
        ));
        Ok(())
    }

    fn lower_runtime_value(&mut self, expression: &HirExpr) -> Result<BoundValue> {
        match expression {
            HirExpr::DomainOperation { .. } => self.append_runtime_operation(expression),
            HirExpr::SymbolRef {
                symbol_id,
                source_name,
                span,
                ..
            } => {
                let value = self.resolve_bound(*symbol_id, *span)?;
                let symbol = self.symbol(*symbol_id, *span)?;
                self.explanations.push(format!(
                    "Variable '{}' resolved to symbol #{} ({}).",
                    source_name,
                    symbol.id,
                    value.describe()
                ));
                Ok(value)
            }
            HirExpr::Promote {
                ty, value, span, ..
            } => {
                let value = self.lower_runtime_value(value)?;
                match value {
                    BoundValue::Runtime(mut reference) => {
                        reference.ty = ty.clone();
                        reference.source_span = *span;
                        Ok(BoundValue::Runtime(reference))
                    }
                    BoundValue::Compile { data, .. } => Ok(BoundValue::Compile {
                        ty: ty.clone(),
                        data,
                    }),
                }
            }
            _ => self.evaluate(expression),
        }
    }

    fn append_runtime_operation(&mut self, expression: &HirExpr) -> Result<BoundValue> {
        let HirExpr::DomainOperation {
            ty,
            domain,
            operation,
            arguments,
            effects,
            span,
        } = expression
        else {
            return self.evaluate(expression);
        };
        if let Some(scope) = self.domain_scope_stack.last() {
            if scope != domain {
                return Err(SemauriError::semantic(
                    "S332",
                    format!(
                        "Operation '{}' belongs to domain '{}', but the active semantic scope is '{}'",
                        operation, domain, scope
                    ),
                    Some(*span),
                    None,
                ));
            }
        }
        let mut runtime_arguments = BTreeMap::new();
        for (name, value) in arguments {
            let lowered = self.lower_runtime_value(value)?;
            runtime_arguments.insert(
                name.clone(),
                match lowered {
                    BoundValue::Compile { data, .. } => RuntimeArgument::Value(data),
                    BoundValue::Runtime(reference) => RuntimeArgument::Ref(reference),
                },
            );
        }
        let id = self.runtime_plan.operations.len() + 1;
        let result = if *ty == Type::Unit {
            None
        } else {
            Some(RuntimeValueRef {
                id: format!("%{id}"),
                ty: ty.clone(),
                producer_id: id,
                source_span: *span,
            })
        };
        self.runtime_plan.operations.push(RuntimeOperation {
            id,
            domain: domain.clone(),
            name: operation.clone(),
            arguments: runtime_arguments,
            return_type: ty.clone(),
            result: result.clone(),
            effects: effects.clone(),
            source_span: *span,
        });
        if let Some(reference) = result {
            self.explanations.push(format!(
                "Planned runtime operation '{}.{}' as operation #{}; result is {} ({}).",
                domain, operation, id, reference.id, reference.ty
            ));
            Ok(BoundValue::Runtime(reference))
        } else {
            self.explanations.push(format!(
                "Planned runtime operation '{}.{}' as operation #{}.",
                domain, operation, id
            ));
            Ok(BoundValue::Compile {
                ty: Type::Unit,
                data: ValueData::Unit,
            })
        }
    }

    fn runtime_dependent_arguments(
        &self,
        arguments: &BTreeMap<String, HirExpr>,
    ) -> Result<bool> {
        for value in arguments.values() {
            if self.runtime_dependent(value)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn runtime_dependent(&self, expression: &HirExpr) -> Result<bool> {
        Ok(match expression {
            HirExpr::DomainOperation { .. } => true,
            HirExpr::SymbolRef {
                symbol_id, span, ..
            } => matches!(self.resolve_bound(*symbol_id, *span)?, BoundValue::Runtime(_)),
            HirExpr::Promote { value, .. } | HirExpr::Unary { operand: value, .. } => {
                self.runtime_dependent(value)?
            }
            HirExpr::List { items, .. } => {
                for item in items {
                    if self.runtime_dependent(item)? {
                        return Ok(true);
                    }
                }
                false
            }
            HirExpr::Binary { left, right, .. } => {
                self.runtime_dependent(left)? || self.runtime_dependent(right)?
            }
            HirExpr::Literal { .. } => false,
        })
    }

    fn evaluate(&mut self, expression: &HirExpr) -> Result<BoundValue> {
        match expression {
            HirExpr::Literal { ty, value, .. } => Ok(BoundValue::Compile {
                ty: ty.clone(),
                data: match value {
                    LiteralValue::Number(value) => ValueData::Number(value.clone()),
                    LiteralValue::Boolean(value) => ValueData::Boolean(*value),
                    LiteralValue::String(value) => ValueData::String(value.clone()),
                    LiteralValue::Color(value) => ValueData::Color(value.clone()),
                },
            }),
            HirExpr::List { ty, items, .. } => {
                let mut values = Vec::new();
                for item in items {
                    match self.evaluate(item)? {
                        BoundValue::Compile { data, .. } => values.push(data),
                        BoundValue::Runtime(_) => return Err(self.runtime_value_error(item.span())),
                    }
                }
                Ok(BoundValue::Compile {
                    ty: ty.clone(),
                    data: ValueData::List(values),
                })
            }
            HirExpr::SymbolRef {
                symbol_id,
                source_name,
                span,
                ..
            } => {
                let value = self.resolve_bound(*symbol_id, *span)?;
                if matches!(value, BoundValue::Runtime(_)) {
                    return Err(self.runtime_value_error(*span));
                }
                let symbol = self.symbol(*symbol_id, *span)?;
                self.explanations.push(format!(
                    "Variable '{}' resolved to symbol #{} ({}).",
                    source_name,
                    symbol.id,
                    value.describe()
                ));
                Ok(value)
            }
            HirExpr::Promote { ty, value, .. } => {
                let value = self.evaluate(value)?;
                match value {
                    BoundValue::Compile { data, .. } => Ok(BoundValue::Compile {
                        ty: ty.clone(),
                        data,
                    }),
                    BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
                }
            }
            HirExpr::Unary {
                operator: UnaryOperator::Not,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Boolean,
                    data: ValueData::Boolean(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Boolean,
                    data: ValueData::Boolean(!value),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S319",
                    format!("Logical 'not' requires a boolean operand, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Binary {
                left,
                operator,
                right,
                ..
            } => self.evaluate_binary(expression.span(), left, operator, right),
            HirExpr::DomainOperation { .. } => Err(self.runtime_value_error(expression.span())),
        }
    }

    fn evaluate_binary(
        &mut self,
        span: SourceSpan,
        left: &HirExpr,
        operator: &BinaryOperator,
        right: &HirExpr,
    ) -> Result<BoundValue> {
        let left_value = self.evaluate(left)?;
        if let BoundValue::Compile {
            ty: Type::Boolean,
            data: ValueData::Boolean(value),
        } = &left_value
        {
            if matches!(operator, BinaryOperator::And) && !value {
                return Ok(BoundValue::Compile {
                    ty: Type::Boolean,
                    data: ValueData::Boolean(false),
                });
            }
            if matches!(operator, BinaryOperator::Or) && *value {
                return Ok(BoundValue::Compile {
                    ty: Type::Boolean,
                    data: ValueData::Boolean(true),
                });
            }
        }

        let right_value = self.evaluate(right)?;
        let (left_ty, left_data) = match left_value {
            BoundValue::Compile { ty, data } => (ty, data),
            BoundValue::Runtime(_) => return Err(self.runtime_value_error(left.span())),
        };
        let (right_ty, right_data) = match right_value {
            BoundValue::Compile { ty, data } => (ty, data),
            BoundValue::Runtime(_) => return Err(self.runtime_value_error(right.span())),
        };
        let result_type = binary_type(operator, &left_ty, &right_ty, span)?;

        let result = match (operator, left_data, right_data) {
            (BinaryOperator::Add, ValueData::Number(a), ValueData::Number(b)) => {
                ValueData::Number(NumberValue::from_f64(a.as_f64() + b.as_f64()))
            }
            (BinaryOperator::Subtract, ValueData::Number(a), ValueData::Number(b)) => {
                ValueData::Number(NumberValue::from_f64(a.as_f64() - b.as_f64()))
            }
            (BinaryOperator::Multiply, ValueData::Number(a), ValueData::Number(b)) => {
                ValueData::Number(NumberValue::from_f64(a.as_f64() * b.as_f64()))
            }
            (BinaryOperator::Divide, ValueData::Number(a), ValueData::Number(b)) => {
                if b.is_zero() {
                    return Err(SemauriError::semantic(
                        "S317",
                        "Division by zero",
                        Some(right.span()),
                        None,
                    ));
                }
                ValueData::Number(NumberValue::Float(a.as_f64() / b.as_f64()))
            }
            (BinaryOperator::GreaterThan, ValueData::Number(a), ValueData::Number(b)) => {
                ValueData::Boolean(a.as_f64() > b.as_f64())
            }
            (BinaryOperator::LessThan, ValueData::Number(a), ValueData::Number(b)) => {
                ValueData::Boolean(a.as_f64() < b.as_f64())
            }
            (
                BinaryOperator::GreaterThanOrEqual,
                ValueData::Number(a),
                ValueData::Number(b),
            ) => ValueData::Boolean(a.as_f64() >= b.as_f64()),
            (
                BinaryOperator::LessThanOrEqual,
                ValueData::Number(a),
                ValueData::Number(b),
            ) => ValueData::Boolean(a.as_f64() <= b.as_f64()),
            (BinaryOperator::Equal, a, b) => ValueData::Boolean(a == b),
            (BinaryOperator::And, ValueData::Boolean(a), ValueData::Boolean(b)) => {
                ValueData::Boolean(a && b)
            }
            (BinaryOperator::Or, ValueData::Boolean(a), ValueData::Boolean(b)) => {
                ValueData::Boolean(a || b)
            }
            _ => {
                return Err(SemauriError::semantic(
                    "S325",
                    format!("Unsupported HIR operator '{}'", operator.as_str()),
                    Some(span),
                    None,
                ))
            }
        };

        Ok(BoundValue::Compile {
            ty: result_type,
            data: result,
        })
    }

    fn resolve_bound(&self, symbol_id: u64, span: SourceSpan) -> Result<BoundValue> {
        for environment in self.environments.iter().rev() {
            if let Some(value) = environment.get(&symbol_id) {
                return Ok(value.clone());
            }
        }
        Err(SemauriError::semantic(
            "S324",
            format!("Unknown HIR symbol #{symbol_id}"),
            Some(span),
            None,
        ))
    }

    fn symbol(&self, id: u64, span: SourceSpan) -> Result<Symbol> {
        self.symbols
            .iter()
            .find(|symbol| symbol.id == id)
            .cloned()
            .ok_or_else(|| {
                SemauriError::semantic(
                    "S324",
                    format!("Unknown HIR symbol #{id}"),
                    Some(span),
                    None,
                )
            })
    }

    fn push_environment(&mut self) {
        self.environments.push(HashMap::new());
    }

    fn pop_environment(&mut self) {
        self.environments.pop();
    }

    fn runtime_value_error(&self, span: SourceSpan) -> SemauriError {
        SemauriError::semantic(
            "S335",
            "Runtime values cannot be evaluated during compilation",
            Some(span),
            Some(
                "Use runtime values only as direct operation inputs for now. Runtime arithmetic and control flow require the planned CFG/SSA lowering stage."
                    .to_string(),
            ),
        )
    }
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn inspect_value(value: &ValueData) -> String {
    match value {
        ValueData::String(value) | ValueData::Color(value) => format!("{:?}", value),
        ValueData::Number(value) => value.to_string(),
        ValueData::Boolean(value) => value.to_string(),
        ValueData::List(values) => format!("{:?}", values.iter().map(ValueData::to_json).collect::<Vec<_>>()),
        ValueData::Unit => "nil".to_string(),
    }
}

