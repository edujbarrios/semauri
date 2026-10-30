// Symbol IDs are internal sequential u64s, not attacker-controlled keys.
// An identity hash avoids SipHash on every loop-iteration binding.
#[derive(Default)]
struct SymbolIdHasher(u64);

impl std::hash::Hasher for SymbolIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = self.0.wrapping_mul(0x100000001b3) ^ u64::from(*byte);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}

type Environment = HashMap<u64, BoundValue, std::hash::BuildHasherDefault<SymbolIdHasher>>;

macro_rules! trace_explanation {
    ($this:expr, $message:expr) => {
        if $this.collect_explanations {
            $this.explanations.push($message);
        }
    };
}

struct Lowerer<'a> {
    domains: &'a DomainRegistry,
    symbols: Vec<Symbol>,
    environments: Vec<Environment>,
    recycled_environments: Vec<Environment>,
    program: ProgramIr,
    runtime_plan: RuntimePlan,
    focus_domain: Option<String>,
    domain_scope_stack: Vec<String>,
    entities: HashMap<String, Vec<EntityRecord>>,
    kind_counts: HashMap<(String, String), usize>,
    explanations: Vec<String>,
    collect_explanations: bool,
}

impl<'a> Lowerer<'a> {
    fn new(domains: &'a DomainRegistry, symbols: &[Symbol]) -> Self {
        Self {
            domains,
            symbols: symbols.to_vec(),
            environments: vec![Environment::default()],
            recycled_environments: Vec::new(),
            program: ProgramIr::default(),
            runtime_plan: RuntimePlan::default(),
            focus_domain: None,
            domain_scope_stack: Vec::new(),
            entities: HashMap::new(),
            kind_counts: HashMap::new(),
            explanations: Vec::new(),
            collect_explanations: true,
        }
    }

    fn without_explanations(mut self) -> Self {
        self.collect_explanations = false;
        self
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
                trace_explanation!(self, format!("Entered semantic domain scope '{domain}'."));
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
                trace_explanation!(self, format!(
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
                trace_explanation!(self, format!(
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
                let iterator_type = self.symbol(*iterator_symbol_id, *span)?.ty.clone();
                trace_explanation!(self, format!(
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
                            ty: iterator_type.clone(),
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
                trace_explanation!(self, format!("Title explicitly set to '{title}'."));
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
                    "Reuse the existing {domain} domain output or split independent {domain} artifacts into separate .suri files."
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
                trace_explanation!(self, "'web' resolved to an HTML web document (default web backend).".to_string());
                if let Some(subject) = &subject {
                    trace_explanation!(self, format!("Subject resolved to '{subject}'."));
                }
                trace_explanation!(self, match origin.as_str() {
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
                trace_explanation!(self, "'schema' resolved to a structured-data schema.".to_string());
                trace_explanation!(self, format!("Schema title resolved to '{title}'."));
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
                trace_explanation!(self, format!(
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
        trace_explanation!(self, format!("Added {kind} '{label}' as {id}."));
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
        trace_explanation!(self, match reference {
            HirReference::Pronoun { pronoun, .. } => format!(
                "'{}' resolved to {} '{}' ({}).",
                pronoun, target.kind, target.label, target.id
            ),
            HirReference::Named { .. } => format!(
                "Explicit reference resolved to {} '{}' ({}).",
                target.kind, target.label, target.id
            ),
        });
        trace_explanation!(self, format!(
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
        trace_explanation!(self, format!(
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
                trace_explanation!(self, format!(
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
            trace_explanation!(self, format!(
                "Planned runtime operation '{}.{}' as operation #{}; result is {} ({}).",
                domain, operation, id, reference.id, reference.ty
            ));
            Ok(BoundValue::Runtime(reference))
        } else {
            trace_explanation!(self, format!(
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
            } => matches!(self.resolve_bound_ref(*symbol_id, *span)?, BoundValue::Runtime(_)),
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
            HirExpr::Repeat { count, value, .. } =>
                self.runtime_dependent(count)? || self.runtime_dependent(value)?,
            HirExpr::Conditional { condition, consequence, alternative, .. } => {
                self.runtime_dependent(condition)?
                    || self.runtime_dependent(consequence)?
                    || self.runtime_dependent(alternative)?
            },
            HirExpr::FunctionCall { arguments, .. } => {
                let mut dependent = false;
                for argument in arguments {
                    if self.runtime_dependent(argument)? {
                        dependent = true;
                        break;
                    }
                }
                dependent
            },
            HirExpr::Reduce { iterable, initial, .. } =>
                self.runtime_dependent(iterable)? || self.runtime_dependent(initial)?,
            HirExpr::ZipTransform { left_list, right_list, .. } =>
                self.runtime_dependent(left_list)? || self.runtime_dependent(right_list)?,
            HirExpr::ListTransform { iterable, .. } => {
                // The iterator symbol exists only during the actual evaluation.
                // The body is evaluated in its per-element lexical environment.
                self.runtime_dependent(iterable)?
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
            HirExpr::Repeat { ty, count, value, .. } => {
                let BoundValue::Compile {
                    data: ValueData::Number(number), ..
                } = self.evaluate(count)? else {
                    return Err(self.runtime_value_error(count.span()));
                };
                let count = match number {
                    NumberValue::Int(count) if (0..=10_000).contains(&count) => count as usize,
                    _ => return Err(SemauriError::semantic(
                        "S378",
                        "Repeat count must be a non-negative integer not exceeding 10,000",
                        Some(count.span()), None)),
                };
                let element = self.evaluate(value)?;
                let BoundValue::Compile { data, .. } = element else {
                    return Err(self.runtime_value_error(value.span()));
                };
                Ok(BoundValue::Compile {
                    ty: ty.clone(),
                    data: ValueData::List(vec![data; count]),
                })
            }
            HirExpr::Conditional { condition, consequence, alternative, .. } => {
                match self.evaluate(condition)? {
                    BoundValue::Compile { data: ValueData::Boolean(true), .. } =>
                        self.evaluate(consequence),
                    BoundValue::Compile { data: ValueData::Boolean(false), .. } =>
                        self.evaluate(alternative),
                    BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                        "S375", format!("Conditional requires boolean, received {ty}"),
                        Some(condition.span()), None)),
                    BoundValue::Runtime(_) => Err(self.runtime_value_error(condition.span())),
                }
            }
            HirExpr::FunctionCall {
                ty, parameters, arguments, body, ..
            } => {
                let mut values = Vec::with_capacity(arguments.len());
                // Evaluate all arguments once, in source order, before binding
                // parameters so caller scope cannot be shadowed mid-evaluation.
                for (argument, (_, expected)) in arguments.iter().zip(parameters) {
                    let value = self.evaluate(argument)?;
                    if value.ty() != *expected {
                        return Err(SemauriError::semantic(
                            "S370", "Function argument differs from its declared parameter type",
                            Some(argument.span()), None));
                    }
                    values.push(value);
                }
                self.push_environment();
                for ((symbol_id, _), value) in parameters.iter().zip(values) {
                    self.environments.last_mut().unwrap().insert(*symbol_id, value);
                }
                let result = self.evaluate(body);
                self.pop_environment();
                let result = result?;
                if result.ty() != *ty {
                    return Err(SemauriError::semantic(
                        "S370", "Function result differs from its inferred return type",
                        Some(expression.span()), None));
                }
                Ok(result)
            }
            HirExpr::Reduce {
                ty, iterator_symbol_id, accumulator_symbol_id, iterable, initial, body, ..
            } => {
                let BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(items),
                } = self.evaluate(iterable)? else {
                    return Err(self.runtime_value_error(iterable.span()));
                };
                let mut accumulator = self.evaluate(initial)?;
                if accumulator.ty() != *ty {
                    return Err(SemauriError::semantic(
                        "S380", "Reduce seed has incorrect typed value", Some(initial.span()), None));
                }
                for item in items {
                    self.push_environment();
                    let frame = self.environments.last_mut().expect("reduce frame");
                    frame.insert(*iterator_symbol_id, BoundValue::Compile {
                        ty: element.as_ref().clone(),
                        data: item,
                    });
                    frame.insert(*accumulator_symbol_id, accumulator);
                    let next = self.evaluate(body);
                    self.pop_environment();
                    accumulator = next?;
                    if accumulator.ty() != *ty {
                        return Err(SemauriError::semantic(
                            "S380", "Reduce step returned an unexpected type",
                            Some(body.span()), None));
                    }
                }
                Ok(accumulator)
            }
            HirExpr::ZipTransform {
                ty, left_symbol_id, left_list, right_symbol_id, right_list, body, ..
            } => {
                let BoundValue::Compile {
                    ty: Type::List(left_element),
                    data: ValueData::List(left_values),
                } = self.evaluate(left_list)? else {
                    return Err(self.runtime_value_error(left_list.span()));
                };
                let BoundValue::Compile {
                    ty: Type::List(right_element),
                    data: ValueData::List(right_values),
                } = self.evaluate(right_list)? else {
                    return Err(self.runtime_value_error(right_list.span()));
                };
                let mut mapped = Vec::with_capacity(left_values.len().min(right_values.len()));
                for (left_value, right_value) in left_values.into_iter().zip(right_values) {
                    self.push_environment();
                    let frame = self.environments.last_mut().expect("zip frame");
                    frame.insert(*left_symbol_id, BoundValue::Compile {
                        ty: left_element.as_ref().clone(), data: left_value,
                    });
                    frame.insert(*right_symbol_id, BoundValue::Compile {
                        ty: right_element.as_ref().clone(), data: right_value,
                    });
                    let value = self.evaluate(body);
                    self.pop_environment();
                    let BoundValue::Compile { ty: element_ty, data } = value? else {
                        return Err(self.runtime_value_error(body.span()));
                    };
                    let Type::List(expected_ty) = ty else {
                        unreachable!("zip HIR must return a list")
                    };
                    if element_ty != **expected_ty {
                        return Err(SemauriError::semantic(
                            "S382", "Zipped body returned unexpected type",
                            Some(body.span()), None));
                    }
                    mapped.push(data);
                }
                Ok(BoundValue::Compile { ty: ty.clone(), data: ValueData::List(mapped) })
            }
            HirExpr::ListTransform { ty, kind, iterator_symbol_id, iterable, body, .. } => {
                let BoundValue::Compile {
                    ty: Type::List(input_element),
                    data: ValueData::List(input_values),
                } = self.evaluate(iterable)? else {
                    return Err(self.runtime_value_error(iterable.span()));
                };
                let result_element = match ty {
                    Type::List(element) => Some(element.as_ref().clone()),
                    Type::Number if *kind == ListTransformKind::Count => None,
                    Type::Boolean if matches!(kind, ListTransformKind::Any | ListTransformKind::All) => None,
                    _ => unreachable!("type-checked list transformation"),
                };
                let mut selected = Vec::with_capacity(if matches!(kind, ListTransformKind::Count | ListTransformKind::Any | ListTransformKind::All) { 0 } else { input_values.len() });
                let mut rejected = Vec::with_capacity(if *kind == ListTransformKind::Partition {
                    input_values.len()
                } else {
                    0
                });
                let mut matching_count = 0_i64;
                for input in input_values {
                    self.push_environment();
                    self.environments.last_mut().unwrap().insert(
                        *iterator_symbol_id,
                        BoundValue::Compile {
                            ty: input_element.as_ref().clone(),
                            data: input.clone(),
                        },
                    );
                    let result = self.evaluate(body);
                    self.pop_environment();
                    let BoundValue::Compile { ty: body_type, data } = result? else {
                        return Err(self.runtime_value_error(body.span()));
                    };
                    if *kind == ListTransformKind::Map {
                        if Some(&body_type) != result_element.as_ref() {
                            return Err(SemauriError::semantic(
                                "S365",
                                "Mapped element type differs from its typed HIR result",
                                Some(body.span()),
                                None,
                            ));
                        }
                        selected.push(data);
                    } else {
                        match data {
                            ValueData::Boolean(true) => {
                                if *kind == ListTransformKind::Any {
                                    return Ok(BoundValue::Compile { ty: Type::Boolean, data: ValueData::Boolean(true) });
                                }
                                if *kind == ListTransformKind::Count {
                                    matching_count = matching_count.checked_add(1).ok_or_else(|| {
                                        SemauriError::semantic(
                                            "S373",
                                            "Counted list items exceed the signed integer range",
                                            Some(expression.span()), None)
                                    })?;
                                } else if matches!(kind, ListTransformKind::Filter | ListTransformKind::Partition) {
                                    selected.push(input);
                                }
                            },
                            ValueData::Boolean(false) => {
                                if *kind == ListTransformKind::All {
                                    return Ok(BoundValue::Compile { ty: Type::Boolean, data: ValueData::Boolean(false) });
                                }
                                if *kind == ListTransformKind::Partition {
                                    rejected.push(input);
                                }
                            },
                            _ => return Err(SemauriError::semantic(
                                "S366",
                                "Filter predicate must evaluate to boolean",
                                Some(body.span()),
                                None,
                            )),
                        }
                    }
                }
                Ok(BoundValue::Compile {
                    ty: ty.clone(),
                    data: match kind {
                        ListTransformKind::Count => ValueData::Number(NumberValue::Int(matching_count)),
                        ListTransformKind::Any => ValueData::Boolean(false),
                        ListTransformKind::All => ValueData::Boolean(true),
                        ListTransformKind::Map | ListTransformKind::Filter => ValueData::List(selected),
                        ListTransformKind::Partition => ValueData::List(vec![ValueData::List(selected), ValueData::List(rejected)]),
                    },
                })
            }
            HirExpr::List { ty, items, .. } => {
                let mut values = Vec::with_capacity(items.len());
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
                trace_explanation!(self, format!(
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
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Characters | UnaryOperator::Codepoints
                    | UnaryOperator::Codepoint | UnaryOperator::CharacterFrom),
                operand,
                ty,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile { data, .. } => {
                    let result = eval_unicode_scalar(operator, &data).ok_or_else(||
                        SemauriError::semantic(
                            "S409",
                            "Invalid Unicode scalar, character count, or code point",
                            Some(expression.span()),
                            Some("Use exactly one character, a valid integer scalar, or at most 10,000 characters.".into()),
                        ))?;
                    Ok(BoundValue::Compile { ty: ty.clone(), data: result })
                }
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::JsonKeys,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile { data: ValueData::String(source), .. } => {
                    let keys = json_object_keys(&source).ok_or_else(||
                        SemauriError::semantic(
                            "S399", "Expected a JSON object within the allowed size limits",
                            Some(expression.span()), None,
                        ))?;
                    Ok(BoundValue::Compile {
                        ty: Type::List(Box::new(Type::String)),
                        data: ValueData::List(keys.into_iter().map(ValueData::String).collect()),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S398", format!("JSON object inspection needs a string, received {ty}"),
                    Some(expression.span()), None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::DecodeJsonString | UnaryOperator::DecodeJsonNumber
                    | UnaryOperator::DecodeJsonBoolean | UnaryOperator::DecodeJsonStrings
                    | UnaryOperator::DecodeJsonNumbers | UnaryOperator::DecodeJsonBooleans),
                operand,
                ty,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile { data: ValueData::String(source), .. } => {
                    let value = decode_json_text(&source, operator).ok_or_else(||
                        SemauriError::semantic(
                            "S397",
                            format!("Invalid JSON or value does not match requested type '{}'", ty),
                            Some(expression.span()),
                            Some("Provide valid JSON with exactly the requested scalar or homogeneous list type.".to_string()),
                        ))?;
                    Ok(BoundValue::Compile { ty: ty.clone(), data: value })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S396", format!("JSON decoding requires a string, received {ty}"),
                    Some(expression.span()), None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::ParseNumber | UnaryOperator::ParseBoolean),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile { data: ValueData::String(value), .. } => match operator {
                    UnaryOperator::ParseNumber => {
                        let parsed = parse_number_text(&value).ok_or_else(||
                            SemauriError::semantic("S395", "Cannot parse text as a finite number",
                                Some(expression.span()), None))?;
                        Ok(BoundValue::Compile {
                            ty: Type::Number, data: ValueData::Number(parsed),
                        })
                    }
                    UnaryOperator::ParseBoolean => {
                        let parsed = parse_boolean_text(&value).ok_or_else(||
                            SemauriError::semantic("S395", "Cannot parse text as boolean (true or false)",
                                Some(expression.span()), None))?;
                        Ok(BoundValue::Compile {
                            ty: Type::Boolean, data: ValueData::Boolean(parsed),
                        })
                    }
                    _ => unreachable!(),
                },
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S394", format!("Text parsing requires a string, received {ty}"),
                    Some(expression.span()), None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Json,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile { data, .. } => Ok(BoundValue::Compile {
                    ty: Type::String,
                    data: ValueData::String(serde_json::to_string(&data.to_json())
                        .expect("serde_json::Value always serializes")),
                }),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Trim | UnaryOperator::Words | UnaryOperator::Lines),
                operand,
                ty,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::String,
                    data: ValueData::String(value),
                } => {
                    let data = match operator {
                        UnaryOperator::Trim => ValueData::String(value.trim().to_string()),
                        UnaryOperator::Words => ValueData::List(value.split_whitespace()
                            .map(|word| ValueData::String(word.to_string())).collect()),
                        UnaryOperator::Lines => ValueData::List(value.lines()
                            .map(|line| ValueData::String(line.to_string())).collect()),
                        _ => unreachable!(),
                    };
                    Ok(BoundValue::Compile { ty: ty.clone(), data })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S351",
                    format!("Text operation requires a string, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Uppercase | UnaryOperator::Lowercase | UnaryOperator::Capitalize | UnaryOperator::Swapcase | UnaryOperator::Uncapitalize | UnaryOperator::Titlecase),
                operand,
                ..
            } => {
                match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::String,
                    data: ValueData::String(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::String,
                    data: ValueData::String(match operator {
                        UnaryOperator::Uppercase => value.to_uppercase(),
                        UnaryOperator::Lowercase => value.to_lowercase(),
                        UnaryOperator::Capitalize => text_capitalize(&value),
                        UnaryOperator::Swapcase => text_swapcase(&value),
                        UnaryOperator::Uncapitalize => text_uncapitalize(&value),
                        UnaryOperator::Titlecase => text_titlecase(&value),
                        _ => unreachable!(),
                    }),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S351",
                    format!("String case transformation requires a string, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
                }
            }
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Sum | UnaryOperator::Average),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(values),
                } if *element == Type::Number => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(aggregate_numeric_list(
                        &values,
                        *operator == UnaryOperator::Average,
                        expression.span(),
                    )?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S353",
                    format!("Numeric aggregation requires list<number>, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Any | UnaryOperator::All),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(values),
                } if *element == Type::Boolean => {
                    let predicate = |value: &ValueData| {
                        matches!(value, ValueData::Boolean(true))
                    };
                    let result = if *operator == UnaryOperator::Any {
                        values.iter().any(predicate)
                    } else {
                        values.iter().all(predicate)
                    };
                    Ok(BoundValue::Compile {
                        ty: Type::Boolean,
                        data: ValueData::Boolean(result),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S356",
                    format!("Boolean reduction requires list<boolean>, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Median,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(values),
                } if *element == Type::Number => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(median_numeric_list(&values, expression.span())?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S353",
                    format!("Numeric aggregation requires list<number>, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Minimum | UnaryOperator::Maximum),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(values),
                } if *element == Type::Number => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(extrema_numeric_list(
                        &values,
                        *operator == UnaryOperator::Minimum,
                        expression.span(),
                    )?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S353",
                    format!("Numeric aggregation requires list<number>, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::First | UnaryOperator::Last),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(values),
                } => {
                    let value = if *operator == UnaryOperator::First {
                        values.first()
                    } else {
                        values.last()
                    }.cloned().ok_or_else(|| SemauriError::semantic(
                        "S362",
                        format!("'{} of' requires a non-empty list", operator.as_str()),
                        Some(expression.span()),
                        None,
                    ))?;
                    Ok(BoundValue::Compile {
                        ty: *element,
                        data: value,
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S361",
                    format!("'first of' and 'last of' require a list, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Distinct,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: ty @ Type::List(_),
                    data: ValueData::List(values),
                } => {
                    // Preserve source order and element type. Equality deliberately
                    // matches the existing list 'contains' operation.
                    let mut distinct = Vec::with_capacity(values.len());
                    for value in values {
                        if !distinct.contains(&value) {
                            distinct.push(value);
                        }
                    }
                    Ok(BoundValue::Compile {
                        ty,
                        data: ValueData::List(distinct),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S360",
                    format!("'distinct of' requires a list, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Sorted,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(mut values),
                } if matches!(element.as_ref(), Type::Number | Type::String | Type::Boolean) => {
                    match element.as_ref() {
                        Type::Number => {
                            for value in &values {
                                let ValueData::Number(number) = value else {
                                    return Err(SemauriError::semantic(
                                        "S359",
                                        "Sorted numeric list contains a non-number",
                                        Some(expression.span()),
                                        None,
                                    ));
                                };
                                if !number.as_f64().is_finite() {
                                    return Err(SemauriError::semantic(
                                        "S359",
                                        "Sorted numeric list requires finite values",
                                        Some(expression.span()),
                                        None,
                                    ));
                                }
                            }
                            values.sort_by(|left, right| {
                                let (ValueData::Number(left), ValueData::Number(right)) = (left, right) else {
                                    unreachable!("type-checked numeric list")
                                };
                                compare_finite_numbers(left, right)
                            });
                        }
                        Type::String => values.sort_by(|left, right| {
                            let (ValueData::String(left), ValueData::String(right)) = (left, right) else {
                                unreachable!("type-checked string list")
                            };
                            left.cmp(right)
                        }),
                        Type::Boolean => values.sort_by(|left, right| {
                            let (ValueData::Boolean(left), ValueData::Boolean(right)) = (left, right) else {
                                unreachable!("type-checked boolean list")
                            };
                            left.cmp(right)
                        }),
                        _ => unreachable!("type-checked sortable list"),
                    }
                    Ok(BoundValue::Compile {
                        ty: Type::List(element),
                        data: ValueData::List(values),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S359",
                    format!("'sorted of' requires list<number>, list<string> or list<boolean>, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Flatten,
                operand,
                ty,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(outer),
                    data: ValueData::List(groups),
                } if matches!(outer.as_ref(), Type::List(_)) => {
                    let mut output = Vec::new();
                    for group in groups {
                        let ValueData::List(mut entries) = group else {
                            return Err(SemauriError::semantic(
                                "S385", "Flattened list contains a non-list element",
                                Some(expression.span()), None));
                        };
                        output.append(&mut entries);
                    }
                    Ok(BoundValue::Compile {
                        ty: ty.clone(),
                        data: ValueData::List(output),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S385",
                    format!("'flatten of' requires a list of lists, received {ty}"),
                    Some(expression.span()), None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Reverse,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::String,
                    data: ValueData::String(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::String,
                    data: ValueData::String(value.chars().rev().collect()),
                }),
                BoundValue::Compile {
                    ty: ty @ Type::List(_),
                    data: ValueData::List(mut values),
                } => {
                    values.reverse();
                    Ok(BoundValue::Compile {
                        ty,
                        data: ValueData::List(values),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S352",
                    format!("'reverse of' requires a string or list, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Length,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::String,
                    data: ValueData::String(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(NumberValue::Int(
                        i64::try_from(value.chars().count()).map_err(|_| {
                            SemauriError::semantic(
                                "S343",
                                "String length exceeds the supported integer range",
                                Some(expression.span()),
                                None,
                            )
                        })?,
                    )),
                }),
                BoundValue::Compile {
                    ty: Type::List(_),
                    data: ValueData::List(values),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(NumberValue::Int(
                        i64::try_from(values.len()).map_err(|_| {
                            SemauriError::semantic(
                                "S343",
                                "List length exceeds the supported integer range",
                                Some(expression.span()),
                                None,
                            )
                        })?,
                    )),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S343",
                    format!("'Length of' requires a string or list, received {ty}"),
                    Some(expression.span()),
                    None,
                )),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Product,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::List(element),
                    data: ValueData::List(values),
                } if *element == Type::Number => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(product_numeric_list(&values, expression.span())?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S353", format!("'product of' requires list<number>, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Absolute,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(number),
                } => {
                    let value = match number {
                        NumberValue::Int(n) => NumberValue::Int(n.checked_abs().ok_or_else(|| {
                            SemauriError::semantic(
                                "S388", "Absolute value of i64 minimum overflows the signed 64-bit range",
                                Some(expression.span()), None)
                        })?),
                        NumberValue::Float(n) if n.is_finite() => NumberValue::Float(n.abs()),
                        NumberValue::Float(_) => return Err(SemauriError::semantic(
                            "S388", "Absolute value requires a finite number",
                            Some(expression.span()), None)),
                    };
                    Ok(BoundValue::Compile {
                        ty: Type::Number,
                        data: ValueData::Number(value),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S386", format!("'absolute of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Sign,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_sign(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S418", "'sign of' requires a finite number",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S417", format!("'sign of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Floor | UnaryOperator::Ceiling),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_round(
                        value, *operator == UnaryOperator::Ceiling,
                    ).ok_or_else(|| SemauriError::semantic(
                        "S420", "Numeric rounding requires a finite number",
                        Some(expression.span()), None,
                    ))?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S419", format!("Numeric rounding requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Round,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_nearest(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S428", "'round of' requires a finite number",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S427", format!("'round of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Truncate,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_truncate(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S430", "'truncate of' requires a finite number",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S429", format!("'truncate of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Factorial,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_factorial(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S432", "'factorial of' requires an integer from 0 through 20",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S431", format!("'factorial of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Prime,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Boolean,
                    data: ValueData::Boolean(numeric_prime(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S436", "'prime of' requires an integral number within the signed 64-bit range",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S435", format!("'prime of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Cube,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_cube(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S434", "'cube of' overflows the numeric range or is non-finite",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S433", format!("'cube of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Square,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_square(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S422", "'square of' overflows the numeric range or is non-finite",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S421", format!("'square of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Digits,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_digits(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S442", "'digits of' requires an integral number within the signed 64-bit range",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S441", format!("'digits of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Reciprocal,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_reciprocal(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S440", "'reciprocal of' requires a nonzero finite number with a finite result",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S439", format!("'reciprocal of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::CubeRoot,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_cbrt(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S438", "'cbrt of' requires a finite number",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S437", format!("'cbrt of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Exponential | UnaryOperator::NaturalLog
                    | UnaryOperator::Sine | UnaryOperator::Cosine),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_transcendental(value, operator.clone()).ok_or_else(|| {
                        SemauriError::semantic(
                            "S466", "Scientific math requires an input in the real finite domain and a finite result",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S465", format!("Scientific math requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::SquareRoot,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => Ok(BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(numeric_sqrt(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S424", "'sqrt of' requires a finite non-negative number",
                            Some(expression.span()), None,
                        )
                    })?),
                }),
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S423", format!("'sqrt of' requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: operator @ (UnaryOperator::Even | UnaryOperator::Odd),
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => {
                    let even = numeric_is_even(value).ok_or_else(|| {
                        SemauriError::semantic(
                            "S426", "Parity requires a finite integer within the signed 64-bit range",
                            Some(expression.span()), None,
                        )
                    })?;
                    Ok(BoundValue::Compile {
                        ty: Type::Boolean,
                        data: ValueData::Boolean(if *operator == UnaryOperator::Even { even } else { !even }),
                    })
                }
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S425", format!("Parity requires number, received {ty}"),
                    Some(expression.span()), None)),
                BoundValue::Runtime(_) => Err(self.runtime_value_error(expression.span())),
            },
            HirExpr::Unary {
                operator: UnaryOperator::Negate,
                operand,
                ..
            } => match self.evaluate(operand)? {
                BoundValue::Compile {
                    ty: Type::Number,
                    data: ValueData::Number(value),
                } => {
                    let negated = match value {
                        NumberValue::Int(integer) => NumberValue::Int(
                            integer.checked_neg().ok_or_else(|| {
                                SemauriError::semantic(
                                    "S358",
                                    "Negating this integer exceeds the signed 64-bit range",
                                    Some(expression.span()),
                                    None,
                                )
                            })?,
                        ),
                        NumberValue::Float(number) => NumberValue::Float(-number),
                    };
                    Ok(BoundValue::Compile {
                        ty: Type::Number,
                        data: ValueData::Number(negated),
                    })
                },
                BoundValue::Compile { ty, .. } => Err(SemauriError::semantic(
                    "S314",
                    format!("Unary 'minus' requires a number operand, received {ty}"),
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
            (BinaryOperator::Range, ValueData::Number(start), ValueData::Number(end)) => {
                let endpoints = match (start, end) {
                    (NumberValue::Int(a), NumberValue::Int(b)) => Some((a, b)),
                    _ => None,
                }.ok_or_else(|| SemauriError::semantic(
                    "S368",
                    "Range endpoints must be integers; decimals are not permitted",
                    Some(span),
                    None,
                ))?;
                let (start, end) = endpoints;
                let distance = (i128::from(end) - i128::from(start)).abs() + 1;
                // Bounds avoid unbounded compile-time memory or CPU work.
                const MAX_RANGE_ELEMENTS: i128 = 10_000;
                if distance > MAX_RANGE_ELEMENTS {
                    return Err(SemauriError::semantic(
                        "S368",
                        "Range exceeds the 10,000-element compile-time safety limit",
                        Some(span),
                        None,
                    ));
                }
                let step: i128 = if end >= start { 1 } else { -1 };
                let values = (0..distance)
                    .map(|offset| ValueData::Number(NumberValue::Int(
                        (i128::from(start) + step * offset) as i64
                    )))
                    .collect();
                ValueData::List(values)
            }
            (BinaryOperator::Add, ValueData::Number(a), ValueData::Number(b)) => {
                ValueData::Number(NumberValue::from_f64(a.as_f64() + b.as_f64()))
            }
            (BinaryOperator::Add, ValueData::String(a), ValueData::String(b)) => {
                ValueData::String(format!("{a}{b}"))
            }
            (BinaryOperator::Add, ValueData::List(mut a), ValueData::List(b)) => {
                a.extend(b);
                ValueData::List(a)
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
            (BinaryOperator::Modulo, ValueData::Number(a), ValueData::Number(b)) => {
                if b.is_zero() {
                    return Err(SemauriError::semantic(
                        "S317",
                        "Modulo by zero",
                        Some(right.span()),
                        None,
                    ));
                }
                ValueData::Number(NumberValue::from_f64(a.as_f64() % b.as_f64()))
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
            (BinaryOperator::NotEqual, a, b) => ValueData::Boolean(a != b),
            (BinaryOperator::Split, ValueData::String(text), ValueData::String(separator)) => {
                if separator.is_empty() {
                    return Err(SemauriError::semantic(
                        "S393", "Split delimiter must not be empty", Some(right.span()), None,
                    ));
                }
                let parts = text.split(&separator).take(10_001)
                    .map(|part| ValueData::String(part.to_string())).collect::<Vec<_>>();
                if parts.len() > 10_000 {
                    return Err(SemauriError::semantic(
                        "S393", "Split exceeds the 10,000-element safety limit", Some(span), None,
                    ));
                }
                ValueData::List(parts)
            }
            (BinaryOperator::Join, ValueData::List(items), ValueData::String(separator)) => {
                let parts: Vec<String> = items.into_iter().map(|item| match item {
                    ValueData::String(value) => value,
                    _ => unreachable!("join operand type was checked"),
                }).collect();
                ValueData::String(parts.join(&separator))
            }
            (operator @ (BinaryOperator::JsonFieldString | BinaryOperator::JsonFieldNumber
                | BinaryOperator::JsonFieldBoolean | BinaryOperator::JsonFieldStrings
                | BinaryOperator::JsonFieldNumbers | BinaryOperator::JsonFieldBooleans),
                ValueData::String(key), ValueData::String(source)) => {
                extract_json_field(&source, &key, operator).ok_or_else(||
                    SemauriError::semantic(
                        "S401",
                        "JSON field is missing, invalid, or does not match its declared type",
                        Some(span),
                        Some("Check the object, field name, type, and JSON size limits.".into()),
                    ))?
            }
            (BinaryOperator::TextPositions, ValueData::String(needle), ValueData::String(text)) => {
                let positions = text_positions(&text, &needle).ok_or_else(||
                    SemauriError::semantic(
                        "S415",
                        "Search term cannot be empty or exceed 10,000 matching positions",
                        Some(span), None,
                    ))?;
                ValueData::List(positions.into_iter()
                    .map(|index| ValueData::Number(NumberValue::Int(index))).collect())
            }
            (BinaryOperator::TextLastPosition, ValueData::String(needle), ValueData::String(text)) => {
                ValueData::Number(NumberValue::Int(
                    text_last_position(&text, &needle)
                        .expect("string character offsets fit in i64"),
                ))
            }
            (BinaryOperator::TextOccurrences, ValueData::String(needle), ValueData::String(text)) => {
                if needle.is_empty() {
                    return Err(SemauriError::semantic(
                        "S403", "The occurrence search term must not be empty",
                        Some(left.span()), None,
                    ));
                }
                ValueData::Number(NumberValue::Int(text.matches(&needle).count() as i64))
            }
            (BinaryOperator::TextRemoveAll, ValueData::String(needle), ValueData::String(text)) => {
                if needle.is_empty() {
                    return Err(SemauriError::semantic(
                        "S413", "Substring to remove must not be empty", Some(left.span()), None,
                    ));
                }
                ValueData::String(text.replace(&needle, ""))
            }
            (BinaryOperator::CharacterAt, ValueData::Number(index), ValueData::String(text)) => {
                let selected = text_character_at(&index, &text).ok_or_else(|| 
                    SemauriError::semantic(
                        "S411",
                        "Character index must be a non-negative integer within the Unicode text",
                        Some(span), None,
                    ))?;
                ValueData::String(selected)
            }
            (BinaryOperator::TextPosition, ValueData::String(needle), ValueData::String(text)) => {
                let position = text.find(&needle)
                    .map(|byte_offset| text[..byte_offset].chars().count() as i64)
                    .unwrap_or(-1);
                ValueData::Number(NumberValue::Int(position))
            }
            (BinaryOperator::JsonPointer, ValueData::String(pointer), ValueData::String(source)) => {
                let selected = extract_json_pointer_text(&source, &pointer).ok_or_else(||
                    SemauriError::semantic(
                        "S407",
                        "Invalid or missing JSON pointer, invalid JSON document, or size limit exceeded",
                        Some(span),
                        Some("Use an RFC 6901 pointer such as '/users/0/name', with '~1' for '/' and '~0' for '~'.".into()),
                    ))?;
                ValueData::String(selected)
            }
            (BinaryOperator::JsonHasKey, ValueData::String(key), ValueData::String(source)) => {
                let found = json_object_has_key(&source, &key).ok_or_else(||
                    SemauriError::semantic(
                        "S399", "Expected a JSON object within the allowed size limits",
                        Some(span), None,
                    ))?;
                ValueData::Boolean(found)
            }
            (BinaryOperator::StartsWith, ValueData::String(text), ValueData::String(prefix)) => {
                ValueData::Boolean(text.starts_with(&prefix))
            }
            (BinaryOperator::EndsWith, ValueData::String(text), ValueData::String(suffix)) => {
                ValueData::Boolean(text.ends_with(&suffix))
            }
            (BinaryOperator::Contains, ValueData::String(text), ValueData::String(needle)) => {
                ValueData::Boolean(text.contains(&needle))
            }
            (BinaryOperator::Contains, ValueData::List(items), needle) => {
                ValueData::Boolean(items.contains(&needle))
            }
            (
                BinaryOperator::Take | BinaryOperator::Drop,
                ValueData::Number(count),
                ValueData::List(items),
            ) => {
                let count = match count {
                    NumberValue::Int(number) if number >= 0 => usize::try_from(number).ok(),
                    _ => None,
                }.ok_or_else(|| SemauriError::semantic(
                    "S364",
                    "List selection count must be a non-negative integer",
                    Some(left.span()),
                    None,
                ))?;
                let selected = if *operator == BinaryOperator::Take {
                    items.into_iter().take(count).collect()
                } else {
                    items.into_iter().skip(count).collect()
                };
                ValueData::List(selected)
            }
            (BinaryOperator::Take | BinaryOperator::Drop, ValueData::Number(count), ValueData::String(text)) => {
                let count = match count {
                    NumberValue::Int(value) if value >= 0 => usize::try_from(value).ok(),
                    _ => None,
                }.ok_or_else(|| SemauriError::semantic(
                    "S364", "Text selection count must be a non-negative integer",
                    Some(left.span()), None,
                ))?;
                let selected: String = if *operator == BinaryOperator::Take {
                    text.chars().take(count).collect()
                } else {
                    text.chars().skip(count).collect()
                };
                ValueData::String(selected)
            }
            (BinaryOperator::AtIndex, ValueData::List(items), ValueData::Number(index)) => {
                let index = match index {
                    NumberValue::Int(value) if value >= 0 => usize::try_from(value).ok(),
                    _ => None,
                }
                .ok_or_else(|| {
                    SemauriError::semantic(
                        "S346",
                        "List index must be a non-negative integer",
                        Some(right.span()),
                        None,
                    )
                })?;
                items.get(index).cloned().ok_or_else(|| {
                    SemauriError::semantic(
                        "S346",
                        format!(
                            "List index {} is out of bounds for {} item(s)",
                            index,
                            items.len()
                        ),
                        Some(right.span()),
                        None,
                    )
                })?
            }
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

    fn resolve_bound_ref(&self, symbol_id: u64, span: SourceSpan) -> Result<&BoundValue> {
        for environment in self.environments.iter().rev() {
            if let Some(value) = environment.get(&symbol_id) {
                return Ok(value);
            }
        }
        Err(SemauriError::semantic(
            "S324",
            format!("Unknown HIR symbol #{symbol_id}"),
            Some(span),
            None,
        ))
    }

    fn resolve_bound(&self, symbol_id: u64, span: SourceSpan) -> Result<BoundValue> {
        self.resolve_bound_ref(symbol_id, span).cloned()
    }

    fn symbol(&self, id: u64, span: SourceSpan) -> Result<&Symbol> {
        // HIR symbols are assigned sequential IDs starting at one.
        // Index directly rather than scanning and cloning on each lookup.
        usize::try_from(id).ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| self.symbols.get(index))
            .filter(|symbol| symbol.id == id)
            .ok_or_else(|| SemauriError::semantic(
                "S324", format!("Unknown HIR symbol #{id}"), Some(span), None,
            ))
    }

    fn push_environment(&mut self) {
        self.environments.push(
            self.recycled_environments.pop().unwrap_or_default(),
        );
    }

    fn pop_environment(&mut self) {
        // Reuse allocated buckets across iterations, but never leak bindings
        // into a later lexical scope. Retain only empty recycled frames.
        if let Some(mut frame) = self.environments.pop() {
            frame.clear();
            self.recycled_environments.push(frame);
        }
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



/// Exact, transitive comparison for finite ints/floats. Converting an i64
/// into f64 would conflate adjacent integers beyond 2^53 and can make
/// mixed-type sort comparisons inconsistent.
fn compare_finite_numbers(left: &NumberValue, right: &NumberValue) -> std::cmp::Ordering {
    match (left, right) {
        (NumberValue::Int(a), NumberValue::Int(b)) => a.cmp(b),
        (NumberValue::Float(a), NumberValue::Float(b)) => {
            a.partial_cmp(b).expect("finite numeric values")
        }
        (NumberValue::Int(a), NumberValue::Float(b)) => compare_integer_to_float(*a, *b),
        (NumberValue::Float(a), NumberValue::Int(b)) => compare_integer_to_float(*b, *a).reverse(),
    }
}

fn compare_integer_to_float(integer: i64, decimal: f64) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    // 2^63 is exactly representable in f64, but not in i64.
    // Handle out-of-range decimals before using a checked-range cast.
    if decimal >= 9_223_372_036_854_775_808.0 {
        return Ordering::Less;
    }
    if decimal < -9_223_372_036_854_775_808.0 {
        return Ordering::Greater;
    }
    let integral_part = decimal.trunc() as i64;
    match integer.cmp(&integral_part) {
        Ordering::Equal if decimal.fract() > 0.0 => Ordering::Less,
        Ordering::Equal if decimal.fract() < 0.0 => Ordering::Greater,
        other => other,
    }
}

/// Median of concrete numeric data. Integer pairs use i128 to avoid
/// overflowing their sum; sorting pure i64 values does not round via f64.
fn median_numeric_list(values: &[ValueData], span: SourceSpan) -> Result<NumberValue> {
    let mut numbers = Vec::with_capacity(values.len());
    for value in values {
        let ValueData::Number(number) = value else {
            return Err(SemauriError::semantic(
                "S353",
                "Numeric median requires list<number>",
                Some(span),
                None,
            ));
        };
        if !number.as_f64().is_finite() {
            return Err(SemauriError::semantic(
                "S357",
                "Numeric median requires finite values",
                Some(span),
                None,
            ));
        }
        numbers.push(number.clone());
    }
    if numbers.is_empty() {
        return Err(SemauriError::semantic(
            "S357",
            "Numeric median requires a non-empty list",
            Some(span),
            None,
        ));
    }
    numbers.sort_by(compare_finite_numbers);
    let middle = numbers.len() / 2;
    if numbers.len() % 2 == 1 {
        return Ok(numbers[middle].clone());
    }
    match (&numbers[middle - 1], &numbers[middle]) {
        (NumberValue::Int(left), NumberValue::Int(right)) => {
            let sum = i128::from(*left) + i128::from(*right);
            if sum % 2 == 0 {
                Ok(NumberValue::Int((sum / 2) as i64))
            } else {
                Ok(NumberValue::Float(sum as f64 / 2.0))
            }
        }
        (left, right) => {
            let value = left.as_f64() / 2.0 + right.as_f64() / 2.0;
            if !value.is_finite() {
                return Err(SemauriError::semantic(
                    "S357",
                    "Numeric median produced a non-finite result",
                    Some(span),
                    None,
                ));
            }
            Ok(NumberValue::Float(value))
        }
    }
}

/// Return an extremum from a concrete numeric list without mutating it.
/// Integer-only comparisons remain exact beyond the f64 precision limit.
fn extrema_numeric_list(
    values: &[ValueData],
    minimum: bool,
    span: SourceSpan,
) -> Result<NumberValue> {
    let mut best: Option<NumberValue> = None;
    for value in values {
        let ValueData::Number(candidate) = value else {
            return Err(SemauriError::semantic(
                "S353",
                "Numeric aggregation requires list<number>",
                Some(span),
                None,
            ));
        };
        if !candidate.as_f64().is_finite() {
            return Err(SemauriError::semantic(
                "S355",
                "Numeric extrema require finite values",
                Some(span),
                None,
            ));
        }
        // Reuse the exact mixed i64/f64 comparator shared by sorted/median.
        // Casting a large i64 to f64 here could make adjacent integers equal
        // and silently select the wrong minimum or maximum past 2^53.
        let replace = match best.as_ref() {
            None => true,
            Some(current) => {
                let order = compare_finite_numbers(candidate, current);
                if minimum { order.is_lt() } else { order.is_gt() }
            }
        };
        if replace {
            best = Some(candidate.clone());
        }
    }
    best.ok_or_else(|| {
        SemauriError::semantic(
            "S355",
            "Numeric extrema require a non-empty list",
            Some(span),
            None,
        )
    })
}

/// Sum and mean of typed numeric lists. Exact integer sums are computed with
/// checked i64 addition; floats follow the language's f64 number semantics.
/// Overflow and non-finite floating results produce a located diagnostic.
fn aggregate_numeric_list(
    values: &[ValueData],
    average: bool,
    span: SourceSpan,
) -> Result<NumberValue> {
    if average && values.is_empty() {
        return Err(SemauriError::semantic(
            "S354",
            "'average of' requires a non-empty numeric list",
            Some(span),
            None,
        ));
    }

    let mut integer_total = 0_i64;
    let mut floating_total = 0.0_f64;
    let mut all_integers = true;

    for value in values {
        match value {
            ValueData::Number(NumberValue::Int(number)) => {
                if all_integers {
                    integer_total = integer_total.checked_add(*number).ok_or_else(|| {
                        SemauriError::semantic(
                            "S354",
                            "Numeric list sum exceeds the signed 64-bit integer range",
                            Some(span),
                            None,
                        )
                    })?;
                }
                floating_total += *number as f64;
            }
            ValueData::Number(NumberValue::Float(number)) => {
                all_integers = false;
                floating_total += number;
            }
            _ => {
                return Err(SemauriError::semantic(
                    "S354",
                    "Numeric list contains a value that is not a number",
                    Some(span),
                    None,
                ));
            }
        }
        if !floating_total.is_finite() {
            return Err(SemauriError::semantic(
                "S354",
                "Numeric list aggregation produced a non-finite result",
                Some(span),
                None,
            ));
        }
    }

    if average {
        let mean = floating_total / values.len() as f64;
        if !mean.is_finite() {
            return Err(SemauriError::semantic(
                "S354",
                "Numeric list average produced a non-finite result",
                Some(span),
                None,
            ));
        }
        Ok(NumberValue::from_f64(mean))
    } else if all_integers {
        Ok(NumberValue::Int(integer_total))
    } else {
        Ok(NumberValue::from_f64(floating_total))
    }
}

/// Multiplication identity is one for empty typed lists. Exact integer-only
/// products never round through f64. Mixed/decimal products use f64, and all
/// intermediate results must remain finite; no partial result is returned.
fn product_numeric_list(values: &[ValueData], span: SourceSpan) -> Result<NumberValue> {
    let has_decimals = values.iter().any(|v| matches!(v, ValueData::Number(NumberValue::Float(_))));
    if has_decimals {
        let mut result = 1.0_f64;
        for value in values {
            let ValueData::Number(number) = value else {
                return Err(SemauriError::semantic("S353", "Product requires list<number>", Some(span), None));
            };
            let factor = number.as_f64();
            if !factor.is_finite() || !result.is_finite() {
                return Err(SemauriError::semantic("S387", "Product requires finite numeric values", Some(span), None));
            }
            result *= factor;
            if !result.is_finite() {
                return Err(SemauriError::semantic("S387", "Numeric product produced a non-finite result", Some(span), None));
            }
        }
        Ok(NumberValue::from_f64(result))
    } else {
        let mut result = 1_i64;
        for value in values {
            let ValueData::Number(NumberValue::Int(number)) = value else {
                return Err(SemauriError::semantic("S353", "Product requires list<number>", Some(span), None));
            };
            result = result.checked_mul(*number).ok_or_else(|| {
                SemauriError::semantic("S387", "Integer product exceeds signed 64-bit range", Some(span), None)
            })?;
        }
        Ok(NumberValue::Int(result))
    }
}
