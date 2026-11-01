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

