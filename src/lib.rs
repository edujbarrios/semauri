// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceSpan {
    pub start: Position,
    pub end: Position,
}

impl SourceSpan {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start: Position {
                line: start_line,
                column: start_column,
            },
            end: Position {
                line: end_line,
                column: end_column,
            },
        }
    }

    pub fn point(line: usize, column: usize) -> Self {
        Self::new(line, column, line, column + 1)
    }

    pub fn line(&self) -> usize {
        self.start.line
    }

    pub fn column(&self) -> usize {
        self.start.column
    }

    pub fn end_line(&self) -> usize {
        self.end.line
    }

    pub fn end_column(&self) -> usize {
        self.end.column
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Lex,
    Parse,
    Semantic,
    Backend,
    Usage,
    Runtime,
}

#[derive(Clone, Debug)]
pub struct SemauriError {
    pub kind: ErrorKind,
    pub code: String,
    pub message: String,
    pub span: Option<SourceSpan>,
    pub hint: Option<String>,
}

impl SemauriError {
    fn new(
        kind: ErrorKind,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<SourceSpan>,
        hint: Option<String>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            span,
            hint,
        }
    }

    fn lex(code: &str, message: impl Into<String>, span: SourceSpan) -> Self {
        Self::new(ErrorKind::Lex, code, message, Some(span), None)
    }

    fn parse(
        code: &str,
        message: impl Into<String>,
        span: SourceSpan,
        hint: Option<String>,
    ) -> Self {
        Self::new(ErrorKind::Parse, code, message, Some(span), hint)
    }

    fn semantic(
        code: &str,
        message: impl Into<String>,
        span: Option<SourceSpan>,
        hint: Option<String>,
    ) -> Self {
        Self::new(ErrorKind::Semantic, code, message, span, hint)
    }

    fn backend(code: &str, message: impl Into<String>, hint: Option<String>) -> Self {
        Self::new(ErrorKind::Backend, code, message, None, hint)
    }

    pub fn diagnostic(&self, source: Option<&str>) -> String {
        let location = self
            .span
            .map(|span| format!(" at {}:{}", span.line(), span.column()))
            .unwrap_or_default();
        let header = format!("{}{}: {}", self.code, location, self.message);

        let mut text = if let (Some(source), Some(span)) = (source, self.span) {
            if let Some(source_line) = source.lines().nth(span.line().saturating_sub(1)) {
                let gutter_width = span.line().to_string().len();
                let end_column = if span.end_line() == span.line() {
                    span.end_column()
                } else {
                    source_line.chars().count() + 1
                };
                let width = end_column.saturating_sub(span.column()).max(1);
                let marker = format!(
                    "{}^{}",
                    " ".repeat(span.column().saturating_sub(1)),
                    "~".repeat(width.saturating_sub(1))
                );
                format!(
                    "{}\n{} |\n{} | {}\n{} | {}",
                    header,
                    " ".repeat(gutter_width),
                    span.line().to_string().rjust(gutter_width),
                    source_line,
                    " ".repeat(gutter_width),
                    marker
                )
            } else {
                header
            }
        } else {
            header
        };

        if let Some(hint) = &self.hint {
            text.push_str("\nHint: ");
            text.push_str(hint);
        }
        text
    }
}

trait Rjust {
    fn rjust(self, width: usize) -> String;
}

impl Rjust for String {
    fn rjust(self, width: usize) -> String {
        if self.len() >= width {
            self
        } else {
            format!("{}{}", " ".repeat(width - self.len()), self)
        }
    }
}

impl fmt::Display for SemauriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostic(None))
    }
}

impl std::error::Error for SemauriError {}

pub type Result<T> = std::result::Result<T, SemauriError>;

#[derive(Clone, Debug, PartialEq)]
pub enum NumberValue {
    Int(i64),
    Float(f64),
}

impl NumberValue {
    fn as_f64(&self) -> f64 {
        match self {
            NumberValue::Int(value) => *value as f64,
            NumberValue::Float(value) => *value,
        }
    }

    fn is_zero(&self) -> bool {
        self.as_f64() == 0.0
    }

    fn is_positive(&self) -> bool {
        self.as_f64() > 0.0
    }

    fn is_negative(&self) -> bool {
        self.as_f64() < 0.0
    }

    fn is_integer(&self) -> bool {
        match self {
            NumberValue::Int(_) => true,
            NumberValue::Float(value) => value.fract() == 0.0,
        }
    }

    fn to_json(&self) -> JsonValue {
        match self {
            NumberValue::Int(value) => json!(value),
            NumberValue::Float(value) => json!(value),
        }
    }

    fn from_f64(value: f64) -> Self {
        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            NumberValue::Int(value as i64)
        } else {
            NumberValue::Float(value)
        }
    }
}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberValue::Int(value) => write!(f, "{value}"),
            NumberValue::Float(value) => {
                let mut rendered = value.to_string();
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    rendered.push_str(".0");
                }
                write!(f, "{rendered}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NominalType {
    pub domain: String,
    pub name: String,
    pub base_type: Box<Type>,
    pub promote_from_base: bool,
}

impl NominalType {
    fn new(domain: &str, name: &str, base_type: Type, promote_from_base: bool) -> Self {
        Self {
            domain: domain.to_string(),
            name: name.to_string(),
            base_type: Box::new(base_type),
            promote_from_base,
        }
    }

    fn to_json(&self) -> JsonValue {
        json!({
            "kind": "nominal",
            "domain": self.domain,
            "name": self.name,
            "base": self.base_type.to_json(),
            "promote_from_base": self.promote_from_base
        })
    }
}

impl fmt::Display for NominalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.domain, self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Number,
    Boolean,
    String,
    Color,
    EntityRef,
    Opaque,
    List(Box<Type>),
    Nominal(NominalType),
}

impl Type {
    fn to_json(&self) -> JsonValue {
        match self {
            Type::Unit => json!("unit"),
            Type::Number => json!("number"),
            Type::Boolean => json!("boolean"),
            Type::String => json!("string"),
            Type::Color => json!("color"),
            Type::EntityRef => json!("entity_ref"),
            Type::Opaque => json!("opaque"),
            Type::List(element) => json!({
                "kind": "list",
                "element_type": element.to_json()
            }),
            Type::Nominal(nominal) => nominal.to_json(),
        }
    }

    fn assignment_kind(actual: &Type, expected: &Type) -> Option<AssignmentKind> {
        if actual == expected {
            return Some(AssignmentKind::Exact);
        }
        if let Type::Nominal(nominal) = expected {
            if nominal.promote_from_base && actual == nominal.base_type.as_ref() {
                return Some(AssignmentKind::Promote);
            }
        }
        None
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "unit"),
            Type::Number => write!(f, "number"),
            Type::Boolean => write!(f, "boolean"),
            Type::String => write!(f, "string"),
            Type::Color => write!(f, "color"),
            Type::EntityRef => write!(f, "entity_ref"),
            Type::Opaque => write!(f, "opaque"),
            Type::List(element) => write!(f, "list<{element}>"),
            Type::Nominal(nominal) => write!(f, "{nominal}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssignmentKind {
    Exact,
    Promote,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Number(NumberValue),
    Boolean(bool),
    String(String),
    Color(String),
}

impl LiteralValue {
    fn to_json(&self) -> JsonValue {
        match self {
            LiteralValue::Number(value) => value.to_json(),
            LiteralValue::Boolean(value) => json!(value),
            LiteralValue::String(value) | LiteralValue::Color(value) => json!(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueData {
    Number(NumberValue),
    Boolean(bool),
    String(String),
    Color(String),
    List(Vec<ValueData>),
    Unit,
}

impl ValueData {
    fn to_json(&self) -> JsonValue {
        match self {
            ValueData::Number(value) => value.to_json(),
            ValueData::Boolean(value) => json!(value),
            ValueData::String(value) | ValueData::Color(value) => json!(value),
            ValueData::List(values) => JsonValue::Array(values.iter().map(ValueData::to_json).collect()),
            ValueData::Unit => JsonValue::Null,
        }
    }

    fn describe(&self, ty: &Type) -> String {
        match self {
            ValueData::Number(value) => format!("{ty} {value}"),
            ValueData::Boolean(value) => format!("{ty} {value}"),
            ValueData::String(value) | ValueData::Color(value) => {
                format!("{ty} {:?}", value)
            }
            ValueData::List(values) => format!("{ty} {:?}", values.iter().map(ValueData::to_json).collect::<Vec<_>>()),
            ValueData::Unit => "unit nil".to_string(),
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct OperationSpec {
    pub name: String,
    pub verbs: Vec<String>,
    pub pattern: Vec<PatternSegment>,
    pub return_type: Type,
    pub effects: Vec<String>,
}

impl OperationSpec {
    fn to_json(&self) -> JsonValue {
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
    fn operation(&self, name: &str) -> Option<&OperationSpec> {
        self.operations.iter().find(|operation| operation.name == name)
    }

    fn to_json(&self) -> JsonValue {
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
    pub fn builtins() -> Self {
        Self {
            domains: vec![
                web_domain(),
                structured_data_domain(),
                filesystem_domain(),
                ml_domain(),
            ],
        }
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

    fn classify(&self, word: &str) -> Option<TokenLiteral> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Create,
    Make,
    Let,
    Be,
    Set,
    If,
    Otherwise,
    End,
    For,
    Every,
    In,
    Within,
    List,
    Is,
    Greater,
    Less,
    Than,
    Equal,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Times,
    Divided,
    By,
    Boolean,
    Of,
    To,
    Article,
    Called,
    Add,
    Title,
    Pronoun,
    Color,
    DomainArtifact,
    DomainElement,
    DomainProperty,
    DomainAction,
    Word,
    Number,
    String,
    Dot,
    Comma,
    Colon,
    LParen,
    RParen,
    Eof,
}

impl TokenKind {
    fn as_str(&self) -> &'static str {
        match self {
            TokenKind::Create => "CREATE",
            TokenKind::Make => "MAKE",
            TokenKind::Let => "LET",
            TokenKind::Be => "BE",
            TokenKind::Set => "SET",
            TokenKind::If => "IF",
            TokenKind::Otherwise => "OTHERWISE",
            TokenKind::End => "END",
            TokenKind::For => "FOR",
            TokenKind::Every => "EVERY",
            TokenKind::In => "IN",
            TokenKind::Within => "WITHIN",
            TokenKind::List => "LIST",
            TokenKind::Is => "IS",
            TokenKind::Greater => "GREATER",
            TokenKind::Less => "LESS",
            TokenKind::Than => "THAN",
            TokenKind::Equal => "EQUAL",
            TokenKind::And => "AND",
            TokenKind::Or => "OR",
            TokenKind::Not => "NOT",
            TokenKind::Plus => "PLUS",
            TokenKind::Minus => "MINUS",
            TokenKind::Times => "TIMES",
            TokenKind::Divided => "DIVIDED",
            TokenKind::By => "BY",
            TokenKind::Boolean => "BOOLEAN",
            TokenKind::Of => "OF",
            TokenKind::To => "TO",
            TokenKind::Article => "ARTICLE",
            TokenKind::Called => "CALLED",
            TokenKind::Add => "ADD",
            TokenKind::Title => "TITLE",
            TokenKind::Pronoun => "PRONOUN",
            TokenKind::Color => "COLOR",
            TokenKind::DomainArtifact => "DOMAIN_ARTIFACT",
            TokenKind::DomainElement => "DOMAIN_ELEMENT",
            TokenKind::DomainProperty => "DOMAIN_PROPERTY",
            TokenKind::DomainAction => "DOMAIN_ACTION",
            TokenKind::Word => "WORD",
            TokenKind::Number => "NUMBER",
            TokenKind::String => "STRING",
            TokenKind::Dot => "DOT",
            TokenKind::Comma => "COMMA",
            TokenKind::Colon => "COLON",
            TokenKind::LParen => "LPAREN",
            TokenKind::RParen => "RPAREN",
            TokenKind::Eof => "EOF",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenLiteral {
    None,
    Number(NumberValue),
    String(String),
    DomainTerm(DomainTerm),
    ActionCandidates(Vec<DomainTerm>),
}

impl TokenLiteral {
    fn to_json(&self) -> JsonValue {
        match self {
            TokenLiteral::None => JsonValue::Null,
            TokenLiteral::Number(value) => value.to_json(),
            TokenLiteral::String(value) => json!(value),
            TokenLiteral::DomainTerm(term) => term.to_json(),
            TokenLiteral::ActionCandidates(candidates) => {
                if candidates.len() == 1 {
                    candidates[0].to_json()
                } else {
                    json!({
                        "candidates": candidates.iter().map(DomainTerm::to_json).collect::<Vec<_>>()
                    })
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: TokenLiteral,
    pub span: SourceSpan,
}

impl Token {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "type": self.kind.as_str(),
            "lexeme": self.lexeme,
            "literal": self.literal.to_json(),
            "line": self.span.line(),
            "column": self.span.column(),
            "end_line": self.span.end_line(),
            "end_column": self.span.end_column(),
            "span": self.span
        })
    }
}

pub struct Lexer<'a> {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    domains: &'a DomainRegistry,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &str, domains: &'a DomainRegistry) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
            domains,
        }
    }

    pub fn tokens(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let end = token.kind == TokenKind::Eof;
            tokens.push(token);
            if end {
                return Ok(tokens);
            }
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_ignored();
        if self.eof() {
            return Ok(Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                literal: TokenLiteral::None,
                span: SourceSpan::new(self.line, self.column, self.line, self.column),
            });
        }

        let start_line = self.line;
        let start_column = self.column;
        let current = self.current().unwrap();

        let punctuation = match current {
            '.' => Some(TokenKind::Dot),
            ',' => Some(TokenKind::Comma),
            ':' => Some(TokenKind::Colon),
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            _ => None,
        };
        if let Some(kind) = punctuation {
            self.advance();
            return Ok(Token {
                kind,
                lexeme: current.to_string(),
                literal: TokenLiteral::None,
                span: SourceSpan::new(start_line, start_column, self.line, self.column),
            });
        }

        if current == '"' {
            return self.string_token(start_line, start_column);
        }
        if current.is_ascii_digit() {
            return self.number_token(start_line, start_column);
        }
        if current.is_alphabetic() {
            return self.word_token(start_line, start_column);
        }

        Err(SemauriError::lex(
            "S101",
            format!("Unexpected character {current:?}"),
            SourceSpan::new(start_line, start_column, start_line, start_column + 1),
        ))
    }

    fn word_token(&mut self, line: usize, column: usize) -> Result<Token> {
        let mut text = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || matches!(ch, '_' | '\'' | '-') {
                text.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let normalized = text.to_lowercase();
        let (kind, literal) = if let Some(kind) = keyword_kind(&normalized) {
            (kind, TokenLiteral::None)
        } else if is_color(&normalized) {
            (TokenKind::Color, TokenLiteral::String(normalized.clone()))
        } else if let Some(domain_literal) = self.domains.classify(&normalized) {
            let kind = match &domain_literal {
                TokenLiteral::DomainTerm(term) => term.category.token_kind(),
                TokenLiteral::ActionCandidates(_) => TokenKind::DomainAction,
                _ => TokenKind::Word,
            };
            (kind, domain_literal)
        } else {
            (TokenKind::Word, TokenLiteral::String(text.clone()))
        };

        Ok(Token {
            kind,
            lexeme: text,
            literal,
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn number_token(&mut self, line: usize, column: usize) -> Result<Token> {
        let mut text = String::new();
        while self.current().is_some_and(|ch| ch.is_ascii_digit()) {
            text.push(self.advance().unwrap());
        }
        if self.current() == Some('.')
            && self.peek_char().is_some_and(|ch| ch.is_ascii_digit())
        {
            text.push(self.advance().unwrap());
            while self.current().is_some_and(|ch| ch.is_ascii_digit()) {
                text.push(self.advance().unwrap());
            }
        }
        let value = if text.contains('.') {
            NumberValue::Float(text.parse::<f64>().unwrap())
        } else {
            NumberValue::Int(text.parse::<i64>().unwrap())
        };
        Ok(Token {
            kind: TokenKind::Number,
            lexeme: text,
            literal: TokenLiteral::Number(value),
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn string_token(&mut self, line: usize, column: usize) -> Result<Token> {
        self.advance();
        let mut value = String::new();
        while !self.eof() && self.current() != Some('"') {
            if self.current() == Some('\n') {
                return Err(SemauriError::lex(
                    "S102",
                    "Literal newlines are not allowed in strings; use an escaped newline",
                    SourceSpan::new(line, column, self.line, self.column),
                ));
            }
            if self.current() == Some('\\') {
                let escape_line = self.line;
                let escape_column = self.column;
                self.advance();
                let escaped = self.advance().ok_or_else(|| {
                    SemauriError::lex(
                        "S104",
                        "Unterminated string escape",
                        SourceSpan::new(line, column, self.line, self.column),
                    )
                })?;
                let replacement = match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    other => {
                        return Err(SemauriError::lex(
                            "S104",
                            format!("Unsupported string escape \\{other}"),
                            SourceSpan::new(
                                escape_line,
                                escape_column,
                                self.line,
                                self.column,
                            ),
                        ))
                    }
                };
                value.push(replacement);
            } else {
                value.push(self.advance().unwrap());
            }
        }

        if self.eof() {
            return Err(SemauriError::lex(
                "S103",
                "Unterminated string",
                SourceSpan::new(line, column, self.line, self.column),
            ));
        }
        self.advance();
        Ok(Token {
            kind: TokenKind::String,
            lexeme: value.clone(),
            literal: TokenLiteral::String(value),
            span: SourceSpan::new(line, column, self.line, self.column),
        })
    }

    fn skip_ignored(&mut self) {
        loop {
            while self.current().is_some_and(char::is_whitespace) {
                self.advance();
            }
            if self.current() != Some('#') {
                break;
            }
            while !self.eof() && self.current() != Some('\n') {
                self.advance();
            }
        }
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn eof(&self) -> bool {
        self.index >= self.chars.len()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.index += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }
}

fn keyword_kind(word: &str) -> Option<TokenKind> {
    Some(match word {
        "create" => TokenKind::Create,
        "make" => TokenKind::Make,
        "let" => TokenKind::Let,
        "be" => TokenKind::Be,
        "set" => TokenKind::Set,
        "if" => TokenKind::If,
        "otherwise" | "else" => TokenKind::Otherwise,
        "end" => TokenKind::End,
        "for" => TokenKind::For,
        "every" => TokenKind::Every,
        "in" => TokenKind::In,
        "within" => TokenKind::Within,
        "list" => TokenKind::List,
        "is" => TokenKind::Is,
        "greater" => TokenKind::Greater,
        "less" => TokenKind::Less,
        "than" => TokenKind::Than,
        "equal" => TokenKind::Equal,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        "not" => TokenKind::Not,
        "plus" => TokenKind::Plus,
        "minus" => TokenKind::Minus,
        "times" => TokenKind::Times,
        "divided" => TokenKind::Divided,
        "by" => TokenKind::By,
        "true" | "false" => TokenKind::Boolean,
        "of" => TokenKind::Of,
        "to" => TokenKind::To,
        "a" | "an" | "the" => TokenKind::Article,
        "called" | "named" => TokenKind::Called,
        "add" => TokenKind::Add,
        "title" => TokenKind::Title,
        "it" => TokenKind::Pronoun,
        _ => return None,
    })
}

fn is_color(word: &str) -> bool {
    matches!(
        word,
        "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "orange"
            | "purple"
            | "pink"
            | "gray"
            | "grey"
            | "brown"
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
}

impl UnaryOperator {
    fn as_str(&self) -> &'static str {
        "not"
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    And,
    Or,
}

impl BinaryOperator {
    fn as_str(&self) -> &'static str {
        match self {
            BinaryOperator::Add => "add",
            BinaryOperator::Subtract => "subtract",
            BinaryOperator::Multiply => "multiply",
            BinaryOperator::Divide => "divide",
            BinaryOperator::GreaterThan => "greater_than",
            BinaryOperator::LessThan => "less_than",
            BinaryOperator::GreaterThanOrEqual => "greater_than_or_equal",
            BinaryOperator::LessThanOrEqual => "less_than_or_equal",
            BinaryOperator::Equal => "equal",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal {
        ty: Type,
        value: LiteralValue,
        span: SourceSpan,
    },
    List {
        items: Vec<Expr>,
        span: SourceSpan,
    },
    Variable {
        name: String,
        span: SourceSpan,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expr>,
        span: SourceSpan,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
        span: SourceSpan,
    },
    DomainOperation {
        domain: String,
        operation: String,
        arguments: BTreeMap<String, Expr>,
        span: SourceSpan,
    },
}

impl Expr {
    fn span(&self) -> SourceSpan {
        match self {
            Expr::Literal { span, .. }
            | Expr::List { span, .. }
            | Expr::Variable { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::DomainOperation { span, .. } => *span,
        }
    }

    fn with_span(self, span: SourceSpan) -> Self {
        match self {
            Expr::Literal { ty, value, .. } => Expr::Literal { ty, value, span },
            Expr::List { items, .. } => Expr::List { items, span },
            Expr::Variable { name, .. } => Expr::Variable { name, span },
            Expr::Unary {
                operator, operand, ..
            } => Expr::Unary {
                operator,
                operand,
                span,
            },
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => Expr::Binary {
                left,
                operator,
                right,
                span,
            },
            Expr::DomainOperation {
                domain,
                operation,
                arguments,
                ..
            } => Expr::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            },
        }
    }

    fn to_json(&self) -> JsonValue {
        let span = self.span();
        let mut base = JsonMap::new();
        base.insert("line".into(), json!(span.line()));
        base.insert("column".into(), json!(span.column()));
        base.insert("end_line".into(), json!(span.end_line()));
        base.insert("end_column".into(), json!(span.end_column()));
        base.insert("span".into(), json!(span));

        match self {
            Expr::Literal { ty, value, .. } => {
                base.insert("type".into(), json!("literal"));
                base.insert("value_type".into(), ty.to_json());
                base.insert("value".into(), value.to_json());
            }
            Expr::List { items, .. } => {
                base.insert("type".into(), json!("list_literal"));
                base.insert(
                    "items".into(),
                    JsonValue::Array(items.iter().map(Expr::to_json).collect()),
                );
            }
            Expr::Variable { name, .. } => {
                base.insert("type".into(), json!("variable_reference"));
                base.insert("name".into(), json!(name));
            }
            Expr::Unary {
                operator, operand, ..
            } => {
                base.insert("type".into(), json!("unary_expression"));
                base.insert("operator".into(), json!(operator.as_str()));
                base.insert("operand".into(), operand.to_json());
            }
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                base.insert("type".into(), json!("binary_expression"));
                base.insert("operator".into(), json!(operator.as_str()));
                base.insert("left".into(), left.to_json());
                base.insert("right".into(), right.to_json());
            }
            Expr::DomainOperation {
                domain,
                operation,
                arguments,
                ..
            } => {
                base.insert("type".into(), json!("domain_operation"));
                base.insert("domain".into(), json!(domain));
                base.insert("operation".into(), json!(operation));
                base.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
            }
        }
        JsonValue::Object(base)
    }
}

#[derive(Clone, Debug)]
pub enum Reference {
    Pronoun {
        pronoun: String,
        span: SourceSpan,
    },
    Named {
        domain: String,
        kind: String,
        label: String,
        span: SourceSpan,
    },
}

impl Reference {
    fn span(&self) -> SourceSpan {
        match self {
            Reference::Pronoun { span, .. } | Reference::Named { span, .. } => *span,
        }
    }

    fn to_json(&self) -> JsonValue {
        let span = self.span();
        match self {
            Reference::Pronoun { pronoun, .. } => json!({
                "type": "pronoun_reference",
                "pronoun": pronoun,
                "line": span.line(),
                "column": span.column(),
                "end_line": span.end_line(),
                "end_column": span.end_column(),
                "span": span
            }),
            Reference::Named {
                domain,
                kind,
                label,
                ..
            } => json!({
                "type": "named_reference",
                "domain": domain,
                "kind": kind,
                "label": label,
                "line": span.line(),
                "column": span.column(),
                "end_line": span.end_line(),
                "end_column": span.end_column(),
                "span": span
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: SourceSpan,
}

impl Block {
    fn to_json(&self) -> JsonValue {
        json!({
            "type": "block",
            "statements": self.statements.iter().map(Stmt::to_json).collect::<Vec<_>>(),
            "line": self.span.line(),
            "column": self.span.column(),
            "end_line": self.span.end_line(),
            "end_column": self.span.end_column(),
            "span": self.span
        })
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    CreateArtifact {
        domain: String,
        kind: String,
        subject: Option<String>,
        title: Option<String>,
        span: SourceSpan,
    },
    DomainScope {
        domain: String,
        body: Block,
        span: SourceSpan,
    },
    DomainOperation {
        domain: String,
        operation: String,
        arguments: BTreeMap<String, Expr>,
        span: SourceSpan,
    },
    SetTitle {
        title: String,
        span: SourceSpan,
    },
    AddElement {
        domain: String,
        kind: String,
        label: Option<String>,
        span: SourceSpan,
    },
    SetProperty {
        domain: String,
        target: Reference,
        property: String,
        value: Expr,
        span: SourceSpan,
    },
    Let {
        name: String,
        value: Expr,
        span: SourceSpan,
    },
    If {
        condition: Expr,
        consequence: Block,
        alternative: Option<Block>,
        span: SourceSpan,
    },
    ForEach {
        variable_name: String,
        binding_span: SourceSpan,
        iterable: Expr,
        body: Block,
        span: SourceSpan,
    },
}

impl Stmt {
    fn span(&self) -> SourceSpan {
        match self {
            Stmt::CreateArtifact { span, .. }
            | Stmt::DomainScope { span, .. }
            | Stmt::DomainOperation { span, .. }
            | Stmt::SetTitle { span, .. }
            | Stmt::AddElement { span, .. }
            | Stmt::SetProperty { span, .. }
            | Stmt::Let { span, .. }
            | Stmt::If { span, .. }
            | Stmt::ForEach { span, .. } => *span,
        }
    }

    fn to_json(&self) -> JsonValue {
        let span = self.span();
        let mut object = JsonMap::new();
        object.insert("line".into(), json!(span.line()));
        object.insert("column".into(), json!(span.column()));
        object.insert("end_line".into(), json!(span.end_line()));
        object.insert("end_column".into(), json!(span.end_column()));
        object.insert("span".into(), json!(span));

        match self {
            Stmt::CreateArtifact {
                domain,
                kind,
                subject,
                title,
                ..
            } => {
                object.insert("type".into(), json!("create_artifact"));
                object.insert("domain".into(), json!(domain));
                object.insert("kind".into(), json!(kind));
                object.insert("subject".into(), json!(subject));
                object.insert("title".into(), json!(title));
            }
            Stmt::DomainScope { domain, body, .. } => {
                object.insert("type".into(), json!("domain_scope"));
                object.insert("domain".into(), json!(domain));
                object.insert("body".into(), body.to_json());
            }
            Stmt::DomainOperation {
                domain,
                operation,
                arguments,
                ..
            } => {
                object.insert("type".into(), json!("domain_operation"));
                object.insert("domain".into(), json!(domain));
                object.insert("operation".into(), json!(operation));
                object.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
            }
            Stmt::SetTitle { title, .. } => {
                object.insert("type".into(), json!("set_title"));
                object.insert("title".into(), json!(title));
            }
            Stmt::AddElement {
                domain,
                kind,
                label,
                ..
            } => {
                object.insert("type".into(), json!("add_element"));
                object.insert("domain".into(), json!(domain));
                object.insert("kind".into(), json!(kind));
                object.insert("label".into(), json!(label));
            }
            Stmt::SetProperty {
                domain,
                target,
                property,
                value,
                ..
            } => {
                object.insert("type".into(), json!("set_property"));
                object.insert("domain".into(), json!(domain));
                object.insert("target".into(), target.to_json());
                object.insert("property".into(), json!(property));
                object.insert("value".into(), value.to_json());
            }
            Stmt::Let { name, value, .. } => {
                object.insert("type".into(), json!("let_binding"));
                object.insert("name".into(), json!(name));
                object.insert("value".into(), value.to_json());
            }
            Stmt::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                object.insert("type".into(), json!("if_statement"));
                object.insert("condition".into(), condition.to_json());
                object.insert("consequence".into(), consequence.to_json());
                object.insert(
                    "alternative".into(),
                    alternative
                        .as_ref()
                        .map(Block::to_json)
                        .unwrap_or(JsonValue::Null),
                );
            }
            Stmt::ForEach {
                variable_name,
                binding_span,
                iterable,
                body,
                ..
            } => {
                object.insert("type".into(), json!("for_each"));
                object.insert("variable_name".into(), json!(variable_name));
                object.insert("binding_span".into(), json!(binding_span));
                object.insert("iterable".into(), iterable.to_json());
                object.insert("body".into(), body.to_json());
            }
        }
        JsonValue::Object(object)
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub span: SourceSpan,
}

impl Program {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "type": "program",
            "statements": self.statements.iter().map(Stmt::to_json).collect::<Vec<_>>(),
            "span": self.span
        })
    }
}

struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    domains: &'a DomainRegistry,
    domain_scope_stack: Vec<String>,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, domains: &'a DomainRegistry) -> Self {
        Self {
            tokens,
            current: 0,
            domains,
            domain_scope_stack: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<Program> {
        let mut statements = Vec::new();
        while !self.check(TokenKind::Eof) {
            statements.push(self.statement()?);
        }
        let span = program_span(&statements);
        Ok(Program { statements, span })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches(&[TokenKind::Create]) {
            let start = self.previous().clone();
            return self.create_statement(&start);
        }
        if self.matches(&[TokenKind::Make]) {
            let start = self.previous().clone();
            return self.make_statement(&start);
        }
        if self.matches(&[TokenKind::Add]) {
            let start = self.previous().clone();
            return self.add_statement(&start);
        }
        if self.matches(&[TokenKind::Let]) {
            let start = self.previous().clone();
            return self.let_statement(&start);
        }
        if self.matches(&[TokenKind::Set]) {
            let start = self.previous().clone();
            return self.set_statement(&start);
        }
        if self.matches(&[TokenKind::If]) {
            let start = self.previous().clone();
            return self.if_statement(&start);
        }
        if self.matches(&[TokenKind::For]) {
            let start = self.previous().clone();
            return self.for_statement(&start);
        }
        if self.matches(&[TokenKind::Within]) {
            let start = self.previous().clone();
            return self.domain_scope_statement(&start);
        }
        if self.matches(&[TokenKind::DomainAction]) {
            let start = self.previous().clone();
            let expr = self.parse_registered_domain_operation(&start)?;
            self.consume_optional_dot();
            if let Expr::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            } = expr
            {
                return Ok(Stmt::DomainOperation {
                    domain,
                    operation,
                    arguments,
                    span: SourceSpan::new(
                        start.span.line(),
                        start.span.column(),
                        span.end_line(),
                        if self.previous().kind == TokenKind::Dot {
                            self.previous().span.end_column()
                        } else {
                            span.end_column()
                        },
                    ),
                });
            }
        }

        Err(self.parse_error(
            self.peek(),
            "Expected a core statement or registered domain action",
            "S201",
            None,
        ))
    }

    fn create_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        let artifact = self.consume(
            TokenKind::DomainArtifact,
            &format!("Expected a registered artifact after '{}'", start.lexeme),
            "S202",
        )?;
        let term = self.domain_term(&artifact)?;
        let mut subject = None;
        let mut title = None;

        if self.matches(&[TokenKind::Called]) {
            title = Some(normalize_phrase(&self.phrase_until(&[
                TokenKind::Dot,
                TokenKind::Eof,
            ])?));
        } else if self.matches(&[TokenKind::For]) {
            self.matches(&[TokenKind::Article]);
            subject = Some(normalize_phrase(&self.phrase_until(&[
                TokenKind::Dot,
                TokenKind::Eof,
            ])?));
        }

        self.consume_optional_dot();
        Ok(Stmt::CreateArtifact {
            domain: term.domain,
            kind: term.kind,
            subject,
            title,
            span: self.span_from(start),
        })
    }

    fn domain_scope_statement(&mut self, start: &Token) -> Result<Stmt> {
        let domain_token = self.peek().clone();
        let domain_name = domain_token.lexeme.to_lowercase();
        if !self.domains.names().contains(&domain_name) {
            return Err(self.parse_error(
                &domain_token,
                format!("Unknown semantic domain '{}'", domain_token.lexeme),
                "S240",
                Some(format!(
                    "Available domains: {}.",
                    self.domains.names().join(", ")
                )),
            ));
        }
        self.advance();
        self.consume(
            TokenKind::Colon,
            "Expected ':' after the semantic domain name",
            "S240",
        )?;

        self.domain_scope_stack.push(domain_name.clone());
        let body_result = self.block_until(&[TokenKind::End]);
        self.domain_scope_stack.pop();
        let body = body_result?;

        self.consume(
            TokenKind::End,
            "Expected 'End' to close the semantic domain scope",
            "S240",
        )?;
        self.consume_optional_dot();
        Ok(Stmt::DomainScope {
            domain: domain_name,
            body,
            span: self.span_from(start),
        })
    }

    fn make_statement(&mut self, start: &Token) -> Result<Stmt> {
        if self.check(TokenKind::Pronoun) {
            return self.pronoun_property_statement(start);
        }
        if self.named_reference_ahead() {
            return self.named_property_statement(start);
        }
        self.create_statement(start)
    }

    fn pronoun_property_statement(&mut self, start: &Token) -> Result<Stmt> {
        let pronoun = self.advance().clone();
        let value_token = self.consume(
            TokenKind::Color,
            &format!("Expected a supported value after '{}'", pronoun.lexeme),
            "S206",
        )?;
        let (domain, property) = self.implicit_property_for(&value_token, &Type::Color)?;
        self.consume_optional_dot();
        let value = Expr::Literal {
            ty: Type::Color,
            value: LiteralValue::Color(token_string_literal(&value_token)),
            span: value_token.span,
        };
        Ok(Stmt::SetProperty {
            domain,
            target: Reference::Pronoun {
                pronoun: pronoun.lexeme,
                span: pronoun.span,
            },
            property,
            value,
            span: self.span_from(start),
        })
    }

    fn named_property_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        let element = self.consume(
            TokenKind::DomainElement,
            "Expected a registered element",
            "S207",
        )?;
        let element_term = self.domain_term(&element)?;
        self.consume(
            TokenKind::Called,
            "Expected 'called' or 'named' in an explicit reference",
            "S207",
        )?;
        let reference_tokens = self.tokens_until(&[TokenKind::Dot, TokenKind::Eof]);
        if reference_tokens.len() < 2 {
            return Err(self.parse_error(
                self.peek(),
                "Expected a name and a value",
                "S208",
                None,
            ));
        }
        let value_token = reference_tokens.last().unwrap().clone();
        if value_token.kind != TokenKind::Color {
            return Err(self.parse_error(
                &value_token,
                "Expected a supported value after the referenced element",
                "S208",
                None,
            ));
        }
        let (domain, property) = self.implicit_property_for(&value_token, &Type::Color)?;
        if domain != element_term.domain {
            return Err(self.parse_error(
                &value_token,
                format!(
                    "Implicit property belongs to domain '{}', but the target belongs to '{}'",
                    domain, element_term.domain
                ),
                "S235",
                None,
            ));
        }
        let label_tokens = &reference_tokens[..reference_tokens.len() - 1];
        let label = normalize_phrase(&phrase_from_tokens(label_tokens));
        self.consume_optional_dot();
        let reference_span = span_between_tokens(&element, label_tokens.last().unwrap());
        Ok(Stmt::SetProperty {
            domain: domain.clone(),
            target: Reference::Named {
                domain,
                kind: element_term.kind,
                label,
                span: reference_span,
            },
            property,
            value: Expr::Literal {
                ty: Type::Color,
                value: LiteralValue::Color(token_string_literal(&value_token)),
                span: value_token.span,
            },
            span: self.span_from(start),
        })
    }

    fn let_statement(&mut self, start: &Token) -> Result<Stmt> {
        let name = self.consume(
            TokenKind::Word,
            "Expected a variable name after 'Let'",
            "S209",
        )?;
        self.consume(
            TokenKind::Be,
            "Expected 'be' after the variable name",
            "S210",
        )?;
        let value = self.expression()?;
        self.consume_optional_dot();
        Ok(Stmt::Let {
            name: name.lexeme.to_lowercase(),
            value,
            span: self.span_from(start),
        })
    }

    fn set_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        let property_token = self.consume(
            TokenKind::DomainProperty,
            "Expected a registered property after 'Set'",
            "S211",
        )?;
        let property_term = self.domain_term(&property_token)?;
        self.consume(
            TokenKind::Of,
            "Expected 'of' after the property name",
            "S212",
        )?;
        let target = self.canonical_reference(&property_term.domain)?;
        self.consume(
            TokenKind::To,
            "Expected 'to' before the new value",
            "S213",
        )?;
        let value = self.expression()?;
        self.consume_optional_dot();
        Ok(Stmt::SetProperty {
            domain: property_term.domain,
            target,
            property: property_term.kind,
            value,
            span: self.span_from(start),
        })
    }

    fn if_statement(&mut self, start: &Token) -> Result<Stmt> {
        let condition = self.expression()?;
        self.consume(
            TokenKind::Colon,
            "Expected ':' after the if condition",
            "S218",
        )?;
        let consequence = self.block_until(&[TokenKind::Otherwise, TokenKind::End])?;
        let alternative = if self.matches(&[TokenKind::Otherwise]) {
            self.consume(
                TokenKind::Colon,
                "Expected ':' after 'Otherwise'",
                "S219",
            )?;
            Some(self.block_until(&[TokenKind::End])?)
        } else {
            None
        };
        self.consume(
            TokenKind::End,
            "Expected 'End' to close the If block",
            "S220",
        )?;
        self.consume_optional_dot();
        Ok(Stmt::If {
            condition,
            consequence,
            alternative,
            span: self.span_from(start),
        })
    }

    fn for_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.consume(
            TokenKind::Every,
            "Expected 'every' after 'For'",
            "S228",
        )?;
        let variable = self.consume(
            TokenKind::Word,
            "Expected an iteration variable after 'For every'",
            "S229",
        )?;
        self.consume(
            TokenKind::In,
            "Expected 'in' after the iteration variable",
            "S230",
        )?;
        let iterable = self.expression()?;
        self.consume(
            TokenKind::Colon,
            "Expected ':' after the iterable expression",
            "S231",
        )?;
        let body = self.block_until(&[TokenKind::End])?;
        self.consume(
            TokenKind::End,
            "Expected 'End' to close the For block",
            "S232",
        )?;
        self.consume_optional_dot();
        Ok(Stmt::ForEach {
            variable_name: variable.lexeme.to_lowercase(),
            binding_span: variable.span,
            iterable,
            body,
            span: self.span_from(start),
        })
    }

    fn block_until(&mut self, terminators: &[TokenKind]) -> Result<Block> {
        let mut statements = Vec::new();
        while !terminators.contains(&self.peek().kind) && !self.check(TokenKind::Eof) {
            statements.push(self.statement()?);
        }
        if statements.is_empty() {
            return Err(self.parse_error(
                self.peek(),
                "Expected at least one statement in the block",
                "S221",
                None,
            ));
        }
        let span = program_span(&statements);
        Ok(Block { statements, span })
    }

    fn expression(&mut self) -> Result<Expr> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let mut expression = self.logical_and()?;
        while self.matches(&[TokenKind::Or]) {
            let right = self.logical_and()?;
            let span = span_between_spans(expression.span(), right.span());
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Or,
                right: Box::new(right),
                span,
            };
        }
        Ok(expression)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut expression = self.logical_not()?;
        while self.matches(&[TokenKind::And]) {
            let right = self.logical_not()?;
            let span = span_between_spans(expression.span(), right.span());
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(expression)
    }

    fn logical_not(&mut self) -> Result<Expr> {
        if self.matches(&[TokenKind::Not]) {
            let operator = self.previous().clone();
            let operand = self.logical_not()?;
            let span = span_between_spans(operator.span, operand.span());
            return Ok(Expr::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
                span,
            });
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr> {
        let left = self.additive()?;
        if !self.matches(&[TokenKind::Is]) {
            return Ok(left);
        }

        let operator = if self.matches(&[TokenKind::Greater]) {
            self.consume(
                TokenKind::Than,
                "Expected 'than' after 'greater'",
                "S222",
            )?;
            if self.matches(&[TokenKind::Or]) {
                self.consume(
                    TokenKind::Equal,
                    "Expected 'equal' after 'or' in an inclusive comparison",
                    "S222",
                )?;
                self.consume(
                    TokenKind::To,
                    "Expected 'to' after 'equal' in an inclusive comparison",
                    "S222",
                )?;
                BinaryOperator::GreaterThanOrEqual
            } else {
                BinaryOperator::GreaterThan
            }
        } else if self.matches(&[TokenKind::Less]) {
            self.consume(
                TokenKind::Than,
                "Expected 'than' after 'less'",
                "S223",
            )?;
            if self.matches(&[TokenKind::Or]) {
                self.consume(
                    TokenKind::Equal,
                    "Expected 'equal' after 'or' in an inclusive comparison",
                    "S223",
                )?;
                self.consume(
                    TokenKind::To,
                    "Expected 'to' after 'equal' in an inclusive comparison",
                    "S223",
                )?;
                BinaryOperator::LessThanOrEqual
            } else {
                BinaryOperator::LessThan
            }
        } else if self.matches(&[TokenKind::Equal]) {
            self.consume(
                TokenKind::To,
                "Expected 'to' after 'equal'",
                "S224",
            )?;
            BinaryOperator::Equal
        } else {
            return Err(self.parse_error(
                self.peek(),
                "Expected 'greater than', 'less than', or 'equal to' after 'is'",
                "S225",
                Some(
                    "Ordering comparisons may add 'or equal to', for example 'is greater than or equal to'."
                        .to_string(),
                ),
            ));
        };

        let right = self.additive()?;
        let span = span_between_spans(left.span(), right.span());
        Ok(Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            span,
        })
    }

    fn additive(&mut self) -> Result<Expr> {
        let mut expression = self.multiplicative()?;
        while self.matches(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = if self.previous().kind == TokenKind::Plus {
                BinaryOperator::Add
            } else {
                BinaryOperator::Subtract
            };
            let right = self.multiplicative()?;
            let span = span_between_spans(expression.span(), right.span());
            expression = Expr::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
                span,
            };
        }
        Ok(expression)
    }

    fn multiplicative(&mut self) -> Result<Expr> {
        let mut expression = self.primary()?;
        loop {
            if self.matches(&[TokenKind::Times]) {
                let right = self.primary()?;
                let span = span_between_spans(expression.span(), right.span());
                expression = Expr::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::Multiply,
                    right: Box::new(right),
                    span,
                };
            } else if self.matches(&[TokenKind::Divided]) {
                self.consume(
                    TokenKind::By,
                    "Expected 'by' after 'divided'",
                    "S226",
                )?;
                let right = self.primary()?;
                let span = span_between_spans(expression.span(), right.span());
                expression = Expr::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::Divide,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.peek().clone();
        match token.kind {
            TokenKind::Color => {
                self.advance();
                Ok(Expr::Literal {
                    ty: Type::Color,
                    value: LiteralValue::Color(token_string_literal(&token)),
                    span: token.span,
                })
            }
            TokenKind::String => {
                self.advance();
                Ok(Expr::Literal {
                    ty: Type::String,
                    value: LiteralValue::String(token_string_literal(&token)),
                    span: token.span,
                })
            }
            TokenKind::Number => {
                self.advance();
                let value = match token.literal {
                    TokenLiteral::Number(value) => value,
                    _ => unreachable!(),
                };
                Ok(Expr::Literal {
                    ty: Type::Number,
                    value: LiteralValue::Number(value),
                    span: token.span,
                })
            }
            TokenKind::Boolean => {
                self.advance();
                Ok(Expr::Literal {
                    ty: Type::Boolean,
                    value: LiteralValue::Boolean(token.lexeme.eq_ignore_ascii_case("true")),
                    span: token.span,
                })
            }
            TokenKind::Article => {
                let start = self.advance().clone();
                self.list_literal(&start, false)
            }
            TokenKind::List => {
                let start = self.advance().clone();
                self.list_literal(&start, true)
            }
            TokenKind::Word => {
                self.advance();
                Ok(Expr::Variable {
                    name: token.lexeme.to_lowercase(),
                    span: token.span,
                })
            }
            TokenKind::DomainAction => {
                let start = self.advance().clone();
                let expression = self.parse_registered_domain_operation(&start)?;
                let operation = match &expression {
                    Expr::DomainOperation {
                        domain, operation, ..
                    } => self.domains.fetch(domain)?.operation(operation).cloned(),
                    _ => None,
                }
                .ok_or_else(|| {
                    self.parse_error(
                        &start,
                        "Unknown registered domain operation",
                        "S239",
                        None,
                    )
                })?;
                if operation.return_type == Type::Unit {
                    return Err(self.parse_error(
                        &start,
                        format!("Operation '{}' does not produce a value", operation.name),
                        "S241",
                        Some(format!(
                            "Use '{}' as a statement instead of inside an expression.",
                            start.lexeme
                        )),
                    ));
                }
                Ok(expression)
            }
            TokenKind::LParen => {
                let opening = self.advance().clone();
                let inner = self.expression()?;
                let closing = self.consume(
                    TokenKind::RParen,
                    "Expected ')' after expression",
                    "S227",
                )?;
                Ok(inner.with_span(span_between_tokens(&opening, &closing)))
            }
            _ => Err(self.parse_error(
                &token,
                "Expected a color, string, number, boolean, list, variable or parenthesized expression",
                "S217",
                None,
            )),
        }
    }

    fn list_literal(&mut self, start: &Token, list_consumed: bool) -> Result<Expr> {
        if !list_consumed {
            self.consume(
                TokenKind::List,
                "Expected 'list' after the article in a list literal",
                "S233",
            )?;
        }
        self.consume(
            TokenKind::Of,
            "Expected 'of' after 'list'",
            "S234",
        )?;
        let mut items = vec![self.expression()?];
        while self.matches(&[TokenKind::Comma]) {
            items.push(self.expression()?);
        }
        let span = span_between_spans(start.span, items.last().unwrap().span());
        Ok(Expr::List { items, span })
    }

    fn canonical_reference(&mut self, expected_domain: &str) -> Result<Reference> {
        if self.check(TokenKind::Pronoun) {
            let token = self.advance().clone();
            return Ok(Reference::Pronoun {
                pronoun: token.lexeme,
                span: token.span,
            });
        }

        self.matches(&[TokenKind::Article]);
        let element = self.consume(
            TokenKind::DomainElement,
            "Expected a registered element or 'it' after 'of'",
            "S214",
        )?;
        let term = self.domain_term(&element)?;
        if term.domain != expected_domain {
            return Err(self.parse_error(
                &element,
                format!(
                    "Element belongs to domain '{}', expected '{}'",
                    term.domain, expected_domain
                ),
                "S235",
                None,
            ));
        }
        self.consume(
            TokenKind::Called,
            "Canonical references must name the target with 'called' or 'named'",
            "S215",
        )?;
        let label_tokens = self.tokens_until(&[TokenKind::To, TokenKind::Eof]);
        if label_tokens.is_empty() {
            return Err(self.parse_error(
                self.peek(),
                "Expected the referenced element name",
                "S216",
                None,
            ));
        }
        let label = normalize_phrase(&phrase_from_tokens(&label_tokens));
        let span = span_between_tokens(&element, label_tokens.last().unwrap());
        Ok(Reference::Named {
            domain: term.domain,
            kind: term.kind,
            label,
            span,
        })
    }

    fn add_statement(&mut self, start: &Token) -> Result<Stmt> {
        self.matches(&[TokenKind::Article]);
        if self.matches(&[TokenKind::Title]) {
            self.consume(
                TokenKind::Called,
                "Expected 'called' or 'named' after 'title'",
                "S204",
            )?;
            let title = normalize_phrase(&self.phrase_until(&[
                TokenKind::Dot,
                TokenKind::Eof,
            ])?);
            self.consume_optional_dot();
            return Ok(Stmt::SetTitle {
                title,
                span: self.span_from(start),
            });
        }
        if self.check(TokenKind::DomainElement) {
            let element = self.advance().clone();
            let term = self.domain_term(&element)?;
            let label = if self.matches(&[TokenKind::Called]) {
                Some(normalize_phrase(&self.phrase_until(&[
                    TokenKind::Dot,
                    TokenKind::Eof,
                ])?))
            } else {
                None
            };
            self.consume_optional_dot();
            return Ok(Stmt::AddElement {
                domain: term.domain,
                kind: term.kind,
                label,
                span: self.span_from(start),
            });
        }
        Err(self.parse_error(
            self.peek(),
            "Expected 'title' or a registered domain element after 'Add'",
            "S203",
            None,
        ))
    }

    fn parse_registered_domain_operation(&mut self, start: &Token) -> Result<Expr> {
        let term = self.action_term(start)?;
        let domain = self.domains.fetch(&term.domain)?;
        let operation = domain.operation(&term.kind).cloned().ok_or_else(|| {
            self.parse_error(start, "Unknown domain operation", "S239", None)
        })?;
        let mut arguments = BTreeMap::new();
        for segment in &operation.pattern {
            match segment {
                PatternSegment::Literal(word) => {
                    self.consume_surface(
                        word,
                        &format!(
                            "Expected '{}' in '{}' operation",
                            word, operation.name
                        ),
                        "S238",
                    )?;
                }
                PatternSegment::Slot { name, .. } => {
                    arguments.insert(name.clone(), self.expression()?);
                }
            }
        }
        Ok(Expr::DomainOperation {
            domain: term.domain,
            operation: operation.name,
            arguments,
            span: self.span_from(start),
        })
    }

    fn action_term(&self, token: &Token) -> Result<DomainTerm> {
        let candidates = match &token.literal {
            TokenLiteral::ActionCandidates(values) => values.clone(),
            TokenLiteral::DomainTerm(value) => vec![value.clone()],
            _ => {
                return Err(self.parse_error(
                    token,
                    "Domain token is missing semantic metadata",
                    "S237",
                    None,
                ))
            }
        };

        if let Some(scope) = self.domain_scope_stack.last() {
            if let Some(selected) = candidates
                .iter()
                .find(|candidate| &candidate.domain == scope)
            {
                return Ok(selected.clone());
            }
            let available = candidates
                .iter()
                .map(|candidate| candidate.domain.clone())
                .collect::<BTreeSet<_>>();
            return Err(self.parse_error(
                token,
                format!(
                    "Action '{}' is not available in semantic domain '{}'",
                    token.lexeme, scope
                ),
                "S240",
                Some(format!(
                    "This action is available in: {}.",
                    available.into_iter().collect::<Vec<_>>().join(", ")
                )),
            ));
        }

        if candidates.len() == 1 {
            return Ok(candidates[0].clone());
        }

        let domains = candidates
            .iter()
            .map(|candidate| candidate.domain.clone())
            .collect::<BTreeSet<_>>();
        Err(self.parse_error(
            token,
            format!("Ambiguous domain action '{}'", token.lexeme),
            "S240",
            Some(format!(
                "Qualify it with 'Within <domain>:'; candidates: {}.",
                domains.into_iter().collect::<Vec<_>>().join(", ")
            )),
        ))
    }

    fn implicit_property_for(&self, token: &Token, value_type: &Type) -> Result<(String, String)> {
        self.domains
            .infer_property_for_type(value_type)
            .ok_or_else(|| {
                self.parse_error(
                    token,
                    format!(
                        "Cannot infer a unique property for {}; use 'Set <property> of ... to ...'",
                        value_type
                    ),
                    "S236",
                    None,
                )
            })
    }

    fn domain_term(&self, token: &Token) -> Result<DomainTerm> {
        match &token.literal {
            TokenLiteral::DomainTerm(term) => Ok(term.clone()),
            _ => Err(self.parse_error(
                token,
                "Domain token is missing semantic metadata",
                "S237",
                None,
            )),
        }
    }

    fn named_reference_ahead(&self) -> bool {
        let offset = if self.check(TokenKind::Article) { 1 } else { 0 };
        self.tokens
            .get(self.current + offset)
            .is_some_and(|token| token.kind == TokenKind::DomainElement)
    }

    fn phrase_until(&mut self, terminators: &[TokenKind]) -> Result<String> {
        let tokens = self.tokens_until(terminators);
        if tokens.is_empty() {
            return Err(self.parse_error(
                self.peek(),
                "Expected a name or description",
                "S205",
                None,
            ));
        }
        Ok(phrase_from_tokens(&tokens))
    }

    fn tokens_until(&mut self, terminators: &[TokenKind]) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !terminators.contains(&self.peek().kind) {
            tokens.push(self.advance().clone());
        }
        tokens
    }

    fn consume_optional_dot(&mut self) {
        if self.check(TokenKind::Dot) {
            self.advance();
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str, code: &str) -> Result<Token> {
        if self.check(kind) {
            return Ok(self.advance().clone());
        }
        Err(self.parse_error(self.peek(), message, code, None))
    }

    fn consume_surface(&mut self, word: &str, message: &str, code: &str) -> Result<Token> {
        if self.peek().lexeme.eq_ignore_ascii_case(word) {
            return Ok(self.advance().clone());
        }
        Err(self.parse_error(self.peek(), message, code, None))
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        if kinds.iter().any(|kind| self.check(*kind)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn advance(&mut self) -> &Token {
        if !self.check(TokenKind::Eof) {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }

    fn span_from(&self, start: &Token) -> SourceSpan {
        span_between_tokens(start, self.previous())
    }

    fn parse_error(
        &self,
        token: &Token,
        message: impl Into<String>,
        code: &str,
        hint: Option<String>,
    ) -> SemauriError {
        SemauriError::parse(code, message, token.span, hint)
    }
}

fn token_string_literal(token: &Token) -> String {
    match &token.literal {
        TokenLiteral::String(value) => value.clone(),
        _ => token.lexeme.clone(),
    }
}

fn phrase_from_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|token| match &token.literal {
            TokenLiteral::String(value) => value.clone(),
            _ => token.lexeme.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_phrase(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            if word.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()) {
                word.to_string()
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn span_between_tokens(first: &Token, last: &Token) -> SourceSpan {
    SourceSpan::new(
        first.span.line(),
        first.span.column(),
        last.span.end_line(),
        last.span.end_column(),
    )
}

fn span_between_spans(first: SourceSpan, last: SourceSpan) -> SourceSpan {
    SourceSpan::new(
        first.line(),
        first.column(),
        last.end_line(),
        last.end_column(),
    )
}

fn program_span(statements: &[Stmt]) -> SourceSpan {
    if statements.is_empty() {
        return SourceSpan::point(1, 1);
    }
    span_between_spans(
        statements.first().unwrap().span(),
        statements.last().unwrap().span(),
    )
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub ty: Type,
    pub definition_span: SourceSpan,
}

impl Symbol {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "type": self.ty.to_json(),
            "definition_span": self.definition_span
        })
    }
}

#[derive(Clone, Debug)]
pub enum HirExpr {
    Literal {
        ty: Type,
        value: LiteralValue,
        span: SourceSpan,
    },
    List {
        ty: Type,
        items: Vec<HirExpr>,
        span: SourceSpan,
    },
    SymbolRef {
        ty: Type,
        symbol_id: u64,
        name: String,
        source_name: String,
        span: SourceSpan,
    },
    Promote {
        ty: Type,
        from_type: Type,
        value: Box<HirExpr>,
        span: SourceSpan,
    },
    Unary {
        ty: Type,
        operator: UnaryOperator,
        operand: Box<HirExpr>,
        span: SourceSpan,
    },
    Binary {
        ty: Type,
        left: Box<HirExpr>,
        operator: BinaryOperator,
        right: Box<HirExpr>,
        span: SourceSpan,
    },
    DomainOperation {
        ty: Type,
        domain: String,
        operation: String,
        arguments: BTreeMap<String, HirExpr>,
        effects: Vec<String>,
        span: SourceSpan,
    },
}

impl HirExpr {
    fn ty(&self) -> Type {
        match self {
            HirExpr::Literal { ty, .. }
            | HirExpr::List { ty, .. }
            | HirExpr::SymbolRef { ty, .. }
            | HirExpr::Promote { ty, .. }
            | HirExpr::Unary { ty, .. }
            | HirExpr::Binary { ty, .. }
            | HirExpr::DomainOperation { ty, .. } => ty.clone(),
        }
    }

    fn span(&self) -> SourceSpan {
        match self {
            HirExpr::Literal { span, .. }
            | HirExpr::List { span, .. }
            | HirExpr::SymbolRef { span, .. }
            | HirExpr::Promote { span, .. }
            | HirExpr::Unary { span, .. }
            | HirExpr::Binary { span, .. }
            | HirExpr::DomainOperation { span, .. } => *span,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            HirExpr::Literal { .. } => "literal",
            HirExpr::List { .. } => "list",
            HirExpr::SymbolRef { .. } => "symbol_ref",
            HirExpr::Promote { .. } => "promote",
            HirExpr::Unary { .. } => "unary",
            HirExpr::Binary { .. } => "binary",
            HirExpr::DomainOperation { .. } => "domain_operation",
        }
    }

    fn to_json(&self) -> JsonValue {
        let mut fields = JsonMap::new();
        match self {
            HirExpr::Literal { value, .. } => {
                fields.insert("value".into(), value.to_json());
            }
            HirExpr::List { items, .. } => {
                fields.insert(
                    "items".into(),
                    JsonValue::Array(items.iter().map(HirExpr::to_json).collect()),
                );
            }
            HirExpr::SymbolRef {
                symbol_id,
                name,
                source_name,
                ..
            } => {
                fields.insert("symbol_id".into(), json!(symbol_id));
                fields.insert("name".into(), json!(name));
                fields.insert("source_name".into(), json!(source_name));
            }
            HirExpr::Promote {
                from_type, value, ..
            } => {
                fields.insert("value".into(), value.to_json());
                fields.insert("from_type".into(), from_type.to_json());
            }
            HirExpr::Unary {
                operator, operand, ..
            } => {
                fields.insert("operator".into(), json!(operator.as_str()));
                fields.insert("operand".into(), operand.to_json());
            }
            HirExpr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                fields.insert("operator".into(), json!(operator.as_str()));
                fields.insert("left".into(), left.to_json());
                fields.insert("right".into(), right.to_json());
            }
            HirExpr::DomainOperation {
                domain,
                operation,
                arguments,
                effects,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("operation".into(), json!(operation));
                fields.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
                fields.insert("effects".into(), json!(effects));
            }
        }
        json!({
            "kind": self.kind(),
            "type": self.ty().to_json(),
            "fields": fields,
            "span": self.span()
        })
    }
}

#[derive(Clone, Debug)]
pub enum HirReference {
    Pronoun {
        pronoun: String,
        span: SourceSpan,
    },
    Named {
        domain: String,
        kind: String,
        label: String,
        span: SourceSpan,
    },
}

impl HirReference {
    fn span(&self) -> SourceSpan {
        match self {
            HirReference::Pronoun { span, .. } | HirReference::Named { span, .. } => *span,
        }
    }

    fn to_json(&self) -> JsonValue {
        match self {
            HirReference::Pronoun { pronoun, span } => json!({
                "kind": "pronoun_reference",
                "type": "entity_ref",
                "fields": {"pronoun": pronoun},
                "span": span
            }),
            HirReference::Named {
                domain,
                kind,
                label,
                span,
            } => json!({
                "kind": "named_reference",
                "type": "entity_ref",
                "fields": {"domain": domain, "entity_kind": kind, "label": label},
                "span": span
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum HirStmt {
    DomainScope {
        domain: String,
        body: Vec<HirStmt>,
        span: SourceSpan,
    },
    Let {
        symbol_id: u64,
        name: String,
        source_name: String,
        value: HirExpr,
        span: SourceSpan,
    },
    If {
        condition: HirExpr,
        consequence: Vec<HirStmt>,
        alternative: Option<Vec<HirStmt>>,
        consequence_span: SourceSpan,
        alternative_span: Option<SourceSpan>,
        span: SourceSpan,
    },
    ForEach {
        iterator_symbol_id: u64,
        iterator_name: String,
        iterator_source_name: String,
        iterable: HirExpr,
        body: Vec<HirStmt>,
        body_span: SourceSpan,
        span: SourceSpan,
    },
    CreateArtifact {
        domain: String,
        artifact_kind: String,
        subject: Option<String>,
        title: Option<String>,
        span: SourceSpan,
    },
    DomainOperation {
        domain: String,
        operation: String,
        arguments: BTreeMap<String, HirExpr>,
        effects: Vec<String>,
        return_type: Type,
        span: SourceSpan,
    },
    SetTitle {
        title: String,
        span: SourceSpan,
    },
    AddElement {
        domain: String,
        element_kind: String,
        label: Option<String>,
        span: SourceSpan,
    },
    SetProperty {
        domain: String,
        target: HirReference,
        property: String,
        value: HirExpr,
        span: SourceSpan,
    },
}

impl HirStmt {
    fn span(&self) -> SourceSpan {
        match self {
            HirStmt::DomainScope { span, .. }
            | HirStmt::Let { span, .. }
            | HirStmt::If { span, .. }
            | HirStmt::ForEach { span, .. }
            | HirStmt::CreateArtifact { span, .. }
            | HirStmt::DomainOperation { span, .. }
            | HirStmt::SetTitle { span, .. }
            | HirStmt::AddElement { span, .. }
            | HirStmt::SetProperty { span, .. } => *span,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            HirStmt::DomainScope { .. } => "domain_scope",
            HirStmt::Let { .. } => "let",
            HirStmt::If { .. } => "if",
            HirStmt::ForEach { .. } => "for_each",
            HirStmt::CreateArtifact { .. } => "create_artifact",
            HirStmt::DomainOperation { .. } => "domain_operation",
            HirStmt::SetTitle { .. } => "set_title",
            HirStmt::AddElement { .. } => "add_element",
            HirStmt::SetProperty { .. } => "set_property",
        }
    }

    fn to_json(&self) -> JsonValue {
        let mut fields = JsonMap::new();
        match self {
            HirStmt::DomainScope { domain, body, span } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("body".into(), hir_block_json(body, *span));
            }
            HirStmt::Let {
                symbol_id,
                name,
                source_name,
                value,
                ..
            } => {
                fields.insert("symbol_id".into(), json!(symbol_id));
                fields.insert("name".into(), json!(name));
                fields.insert("source_name".into(), json!(source_name));
                fields.insert("value".into(), value.to_json());
            }
            HirStmt::If {
                condition,
                consequence,
                alternative,
                consequence_span,
                alternative_span,
                ..
            } => {
                fields.insert("condition".into(), condition.to_json());
                fields.insert(
                    "consequence".into(),
                    hir_block_json(consequence, *consequence_span),
                );
                fields.insert(
                    "alternative".into(),
                    alternative
                        .as_ref()
                        .map(|body| hir_block_json(body, alternative_span.unwrap()))
                        .unwrap_or(JsonValue::Null),
                );
            }
            HirStmt::ForEach {
                iterator_symbol_id,
                iterator_name,
                iterator_source_name,
                iterable,
                body,
                body_span,
                ..
            } => {
                fields.insert("iterator_symbol_id".into(), json!(iterator_symbol_id));
                fields.insert("iterator_name".into(), json!(iterator_name));
                fields.insert(
                    "iterator_source_name".into(),
                    json!(iterator_source_name),
                );
                fields.insert("iterable".into(), iterable.to_json());
                fields.insert("body".into(), hir_block_json(body, *body_span));
            }
            HirStmt::CreateArtifact {
                domain,
                artifact_kind,
                subject,
                title,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("artifact_kind".into(), json!(artifact_kind));
                fields.insert("subject".into(), json!(subject));
                fields.insert("title".into(), json!(title));
            }
            HirStmt::DomainOperation {
                domain,
                operation,
                arguments,
                effects,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("operation".into(), json!(operation));
                fields.insert(
                    "arguments".into(),
                    JsonValue::Object(
                        arguments
                            .iter()
                            .map(|(name, value)| (name.clone(), value.to_json()))
                            .collect(),
                    ),
                );
                fields.insert("effects".into(), json!(effects));
            }
            HirStmt::SetTitle { title, .. } => {
                fields.insert("title".into(), json!(title));
            }
            HirStmt::AddElement {
                domain,
                element_kind,
                label,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("element_kind".into(), json!(element_kind));
                fields.insert("label".into(), json!(label));
            }
            HirStmt::SetProperty {
                domain,
                target,
                property,
                value,
                ..
            } => {
                fields.insert("domain".into(), json!(domain));
                fields.insert("target".into(), target.to_json());
                fields.insert("property".into(), json!(property));
                fields.insert("value".into(), value.to_json());
            }
        }
        json!({
            "kind": self.kind(),
            "type": "unit",
            "fields": fields,
            "span": self.span()
        })
    }
}

fn hir_block_json(statements: &[HirStmt], span: SourceSpan) -> JsonValue {
    json!({
        "kind": "block",
        "type": "unit",
        "fields": {
            "statements": statements.iter().map(HirStmt::to_json).collect::<Vec<_>>()
        },
        "span": span
    })
}

#[derive(Clone, Debug)]
pub struct HirResult {
    pub statements: Vec<HirStmt>,
    pub symbols: Vec<Symbol>,
    pub span: SourceSpan,
}

impl HirResult {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "program": {
                "kind": "program",
                "type": "unit",
                "fields": {
                    "statements": self.statements.iter().map(HirStmt::to_json).collect::<Vec<_>>()
                },
                "span": self.span
            },
            "symbols": self.symbols.iter().map(Symbol::to_json).collect::<Vec<_>>()
        })
    }
}

struct HirBuilder<'a> {
    domains: &'a DomainRegistry,
    symbols: Vec<Symbol>,
    scopes: Vec<HashMap<String, u64>>,
}

impl<'a> HirBuilder<'a> {
    fn new(domains: &'a DomainRegistry) -> Self {
        Self {
            domains,
            symbols: Vec::new(),
            scopes: vec![HashMap::new()],
        }
    }

    fn build(mut self, program: &Program) -> Result<HirResult> {
        let statements = self.build_statements(&program.statements)?;
        Ok(HirResult {
            statements,
            symbols: self.symbols,
            span: program.span,
        })
    }

    fn build_statements(&mut self, statements: &[Stmt]) -> Result<Vec<HirStmt>> {
        statements.iter().map(|statement| self.build_stmt(statement)).collect()
    }

    fn build_stmt(&mut self, statement: &Stmt) -> Result<HirStmt> {
        match statement {
            Stmt::DomainScope { domain, body, span } => {
                self.domains.fetch(domain)?;
                self.push_scope();
                let built = self.build_statements(&body.statements);
                self.pop_scope();
                Ok(HirStmt::DomainScope {
                    domain: domain.clone(),
                    body: built?,
                    span: *span,
                })
            }
            Stmt::Let { name, value, span } => {
                let built_value = self.build_expr(value)?;
                let symbol = self.create_symbol(name, "variable", built_value.ty(), *span)?;
                Ok(HirStmt::Let {
                    symbol_id: symbol.id,
                    name: symbol.name.clone(),
                    source_name: name.clone(),
                    value: built_value,
                    span: *span,
                })
            }
            Stmt::If {
                condition,
                consequence,
                alternative,
                span,
            } => {
                let condition = self.build_expr(condition)?;
                if condition.ty() != Type::Boolean {
                    return Err(SemauriError::semantic(
                        "S316",
                        format!(
                            "If condition must evaluate to boolean, received {}",
                            condition.ty()
                        ),
                        Some(condition.span()),
                        None,
                    ));
                }

                self.push_scope();
                let consequence_built = self.build_statements(&consequence.statements);
                self.pop_scope();
                let consequence_built = consequence_built?;

                let alternative_built = if let Some(alternative) = alternative {
                    self.push_scope();
                    let built = self.build_statements(&alternative.statements);
                    self.pop_scope();
                    Some(built?)
                } else {
                    None
                };

                Ok(HirStmt::If {
                    condition,
                    consequence: consequence_built,
                    alternative: alternative_built,
                    consequence_span: consequence.span,
                    alternative_span: alternative.as_ref().map(|block| block.span),
                    span: *span,
                })
            }
            Stmt::ForEach {
                variable_name,
                binding_span,
                iterable,
                body,
                span,
            } => {
                let iterable = self.build_expr(iterable)?;
                let element_type = if let Type::List(element_type) = iterable.ty() {
                    *element_type
                } else {
                    return Err(SemauriError::semantic(
                        "S322",
                        format!("For every requires a list, received {}", iterable.ty()),
                        Some(iterable.span()),
                        None,
                    ));
                };
                self.push_scope();
                let iterator =
                    self.create_symbol(variable_name, "iterator", element_type, *binding_span)?;
                let body_built = self.build_statements(&body.statements);
                self.pop_scope();
                Ok(HirStmt::ForEach {
                    iterator_symbol_id: iterator.id,
                    iterator_name: iterator.name.clone(),
                    iterator_source_name: variable_name.clone(),
                    iterable,
                    body: body_built?,
                    body_span: body.span,
                    span: *span,
                })
            }
            Stmt::CreateArtifact {
                domain,
                kind,
                subject,
                title,
                span,
            } => {
                self.domains.fetch(domain)?;
                Ok(HirStmt::CreateArtifact {
                    domain: domain.clone(),
                    artifact_kind: kind.clone(),
                    subject: subject.clone(),
                    title: title.clone(),
                    span: *span,
                })
            }
            Stmt::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            } => {
                let expression = Expr::DomainOperation {
                    domain: domain.clone(),
                    operation: operation.clone(),
                    arguments: arguments.clone(),
                    span: *span,
                };
                let hir = self.build_expr(&expression)?;
                if let HirExpr::DomainOperation {
                    domain,
                    operation,
                    arguments,
                    effects,
                    ty,
                    span,
                } = hir
                {
                    Ok(HirStmt::DomainOperation {
                        domain,
                        operation,
                        arguments,
                        effects,
                        return_type: ty,
                        span,
                    })
                } else {
                    unreachable!()
                }
            }
            Stmt::SetTitle { title, span } => Ok(HirStmt::SetTitle {
                title: title.clone(),
                span: *span,
            }),
            Stmt::AddElement {
                domain,
                kind,
                label,
                span,
            } => {
                self.domains.fetch(domain)?;
                Ok(HirStmt::AddElement {
                    domain: domain.clone(),
                    element_kind: kind.clone(),
                    label: label.clone(),
                    span: *span,
                })
            }
            Stmt::SetProperty {
                domain,
                target,
                property,
                value,
                span,
            } => {
                let value = self.build_expr(value)?;
                if let Some(expected) = self.domains.property_type(domain, property) {
                    if expected != value.ty() {
                        return Err(SemauriError::semantic(
                            "S313",
                            format!(
                                "Property '{}' in domain '{}' expects {}, but received {}",
                                property,
                                domain,
                                expected,
                                value.ty()
                            ),
                            Some(value.span()),
                            Some(format!(
                                "Use a {} literal or a variable containing a {}.",
                                expected, expected
                            )),
                        ));
                    }
                }
                let target = match target {
                    Reference::Pronoun { pronoun, span } => HirReference::Pronoun {
                        pronoun: pronoun.clone(),
                        span: *span,
                    },
                    Reference::Named {
                        domain,
                        kind,
                        label,
                        span,
                    } => HirReference::Named {
                        domain: domain.clone(),
                        kind: kind.clone(),
                        label: label.clone(),
                        span: *span,
                    },
                };
                Ok(HirStmt::SetProperty {
                    domain: domain.clone(),
                    target,
                    property: property.clone(),
                    value,
                    span: *span,
                })
            }
        }
    }

    fn build_expr(&mut self, expression: &Expr) -> Result<HirExpr> {
        match expression {
            Expr::Literal { ty, value, span } => Ok(HirExpr::Literal {
                ty: ty.clone(),
                value: value.clone(),
                span: *span,
            }),
            Expr::List { items, span } => {
                let items = items
                    .iter()
                    .map(|item| self.build_expr(item))
                    .collect::<Result<Vec<_>>>()?;
                if items.is_empty() {
                    return Err(SemauriError::semantic(
                        "S320",
                        "A list needs at least one item so its type can be inferred",
                        Some(*span),
                        None,
                    ));
                }
                let element_type = items[0].ty();
                if let Some(mismatch) = items.iter().find(|item| item.ty() != element_type) {
                    return Err(SemauriError::semantic(
                        "S321",
                        format!(
                            "List items must have one type, got {} and {}",
                            element_type,
                            mismatch.ty()
                        ),
                        Some(*span),
                        None,
                    ));
                }
                Ok(HirExpr::List {
                    ty: Type::List(Box::new(element_type)),
                    items,
                    span: *span,
                })
            }
            Expr::Variable { name, span } => {
                let symbol = self.resolve_symbol(name, *span)?;
                Ok(HirExpr::SymbolRef {
                    ty: symbol.ty.clone(),
                    symbol_id: symbol.id,
                    name: symbol.name.clone(),
                    source_name: name.clone(),
                    span: *span,
                })
            }
            Expr::Unary {
                operator,
                operand,
                span,
            } => {
                let operand = self.build_expr(operand)?;
                if operand.ty() != Type::Boolean {
                    return Err(SemauriError::semantic(
                        "S319",
                        format!(
                            "Logical 'not' requires a boolean operand, received {}",
                            operand.ty()
                        ),
                        Some(*span),
                        None,
                    ));
                }
                Ok(HirExpr::Unary {
                    ty: Type::Boolean,
                    operator: operator.clone(),
                    operand: Box::new(operand),
                    span: *span,
                })
            }
            Expr::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left = self.build_expr(left)?;
                let right = self.build_expr(right)?;
                let result_type = binary_type(operator, &left.ty(), &right.ty(), *span)?;
                Ok(HirExpr::Binary {
                    ty: result_type,
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: Box::new(right),
                    span: *span,
                })
            }
            Expr::DomainOperation {
                domain,
                operation,
                arguments,
                span,
            } => {
                let domain_spec = self.domains.fetch(domain)?;
                let operation_spec = domain_spec.operation(operation).cloned().ok_or_else(|| {
                    SemauriError::semantic(
                        "S326",
                        format!(
                            "Domain '{}' does not define operation '{}'",
                            domain, operation
                        ),
                        Some(*span),
                        None,
                    )
                })?;
                let mut built_arguments = BTreeMap::new();
                for (name, value) in arguments {
                    built_arguments.insert(name.clone(), self.build_expr(value)?);
                }

                for segment in &operation_spec.pattern {
                    let PatternSegment::Slot { name, ty } = segment else {
                        continue;
                    };
                    let value = built_arguments.get(name).ok_or_else(|| {
                        SemauriError::semantic(
                            "S330",
                            format!(
                                "Operation '{}' is missing argument {:?}",
                                operation_spec.name, name
                            ),
                            Some(*span),
                            None,
                        )
                    })?;
                    if let Some(expected) = ty {
                        if Type::assignment_kind(&value.ty(), expected).is_none() {
                            return Err(SemauriError::semantic(
                                "S329",
                                format!(
                                    "Operation '{}' argument '{}' expects {}, but received {}",
                                    operation_spec.name,
                                    name,
                                    expected,
                                    value.ty()
                                ),
                                Some(value.span()),
                                Some(format!(
                                    "Provide a {} expression for '{}'.",
                                    expected, name
                                )),
                            ));
                        }
                    }
                }

                let mut promoted = BTreeMap::new();
                for (name, value) in built_arguments {
                    let expected = operation_spec.pattern.iter().find_map(|segment| match segment {
                        PatternSegment::Slot { name: slot_name, ty } if slot_name == &name => {
                            ty.clone()
                        }
                        _ => None,
                    });
                    if let Some(expected) = expected {
                        if Type::assignment_kind(&value.ty(), &expected)
                            == Some(AssignmentKind::Promote)
                        {
                            promoted.insert(
                                name,
                                HirExpr::Promote {
                                    ty: expected,
                                    from_type: value.ty(),
                                    span: value.span(),
                                    value: Box::new(value),
                                },
                            );
                            continue;
                        }
                    }
                    promoted.insert(name, value);
                }

                validate_domain_operation(
                    domain,
                    &operation_spec,
                    &promoted,
                    *span,
                )?;

                Ok(HirExpr::DomainOperation {
                    ty: operation_spec.return_type,
                    domain: domain.clone(),
                    operation: operation.clone(),
                    arguments: promoted,
                    effects: operation_spec.effects,
                    span: *span,
                })
            }
        }
    }

    fn create_symbol(
        &mut self,
        name: &str,
        kind: &str,
        ty: Type,
        definition_span: SourceSpan,
    ) -> Result<Symbol> {
        let key = name.to_lowercase();
        if self.scopes.last().unwrap().contains_key(&key) {
            return Err(SemauriError::semantic(
                "S311",
                format!("Variable '{name}' is already defined in this scope"),
                Some(definition_span),
                Some("Choose a different name or reuse the existing variable.".to_string()),
            ));
        }
        let symbol = Symbol {
            id: self.symbols.len() as u64 + 1,
            name: key.clone(),
            kind: kind.to_string(),
            ty,
            definition_span,
        };
        self.symbols.push(symbol.clone());
        self.scopes.last_mut().unwrap().insert(key, symbol.id);
        Ok(symbol)
    }

    fn resolve_symbol(&self, name: &str, span: SourceSpan) -> Result<Symbol> {
        let key = name.to_lowercase();
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(&key) {
                return Ok(self.symbols[*id as usize - 1].clone());
            }
        }
        Err(SemauriError::semantic(
            "S312",
            format!("Unknown variable '{name}'"),
            Some(span),
            Some(format!("Declare it first with 'Let {name} be ...'.")),
        ))
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}

fn binary_type(
    operator: &BinaryOperator,
    left: &Type,
    right: &Type,
    span: SourceSpan,
) -> Result<Type> {
    match operator {
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide => {
            if left == &Type::Number && right == &Type::Number {
                Ok(Type::Number)
            } else {
                Err(SemauriError::semantic(
                    "S314",
                    format!(
                        "Operator '{}' requires number operands, got {} and {}",
                        operator.as_str(),
                        left,
                        right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::GreaterThan
        | BinaryOperator::LessThan
        | BinaryOperator::GreaterThanOrEqual
        | BinaryOperator::LessThanOrEqual => {
            if left == &Type::Number && right == &Type::Number {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S314",
                    format!(
                        "Operator '{}' requires number operands, got {} and {}",
                        operator.as_str(),
                        left,
                        right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::Equal => {
            if left == right {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S315",
                    format!(
                        "Equality requires operands of the same type, got {} and {}",
                        left, right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
        BinaryOperator::And | BinaryOperator::Or => {
            if left == &Type::Boolean && right == &Type::Boolean {
                Ok(Type::Boolean)
            } else {
                Err(SemauriError::semantic(
                    "S319",
                    format!(
                        "Logical operators require boolean operands, got {} and {}",
                        left, right
                    ),
                    Some(span),
                    None,
                ))
            }
        }
    }
}

fn literal_data(expr: &HirExpr) -> Option<ValueData> {
    match expr {
        HirExpr::Literal { value, .. } => Some(match value {
            LiteralValue::Number(value) => ValueData::Number(value.clone()),
            LiteralValue::Boolean(value) => ValueData::Boolean(*value),
            LiteralValue::String(value) => ValueData::String(value.clone()),
            LiteralValue::Color(value) => ValueData::Color(value.clone()),
        }),
        HirExpr::Promote { value, .. } => literal_data(value),
        _ => None,
    }
}

fn validate_domain_operation(
    domain: &str,
    operation: &OperationSpec,
    arguments: &BTreeMap<String, HirExpr>,
    span: SourceSpan,
) -> Result<()> {
    if domain == "filesystem" {
        let path_names: &[&str] = match operation.name.as_str() {
            "copy" | "move" => &["source", "destination"],
            "write" | "append" | "make_directory" | "touch" | "delete" => &["path"],
            _ => &[],
        };
        for name in path_names {
            if let Some(ValueData::String(path)) = arguments.get(*name).and_then(literal_data) {
                if path.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S340",
                        "filesystem path must not be empty",
                        Some(span),
                        Some("Use a non-empty filesystem path.".to_string()),
                    ));
                }
            }
        }
    }

    if domain != "ml" {
        return Ok(());
    }

    let number = |name: &str| -> Option<NumberValue> {
        arguments.get(name).and_then(literal_data).and_then(|value| match value {
            ValueData::Number(value) => Some(value),
            _ => None,
        })
    };
    let string = |name: &str| -> Option<String> {
        arguments.get(name).and_then(literal_data).and_then(|value| match value {
            ValueData::String(value) => Some(value),
            _ => None,
        })
    };

    let positive_integer = |name: &str, display: &str, code: &str| -> Result<()> {
        if let Some(value) = number(name) {
            if !(value.is_positive() && value.is_integer()) {
                return Err(SemauriError::semantic(
                    code,
                    format!("{display} must be a positive integer"),
                    Some(span),
                    None,
                ));
            }
        }
        Ok(())
    };
    let positive_number = |name: &str, display: &str, code: &str| -> Result<()> {
        if let Some(value) = number(name) {
            if !value.is_positive() {
                return Err(SemauriError::semantic(
                    code,
                    format!("{display} must be greater than zero"),
                    Some(span),
                    None,
                ));
            }
        }
        Ok(())
    };
    let non_negative_integer = |name: &str, display: &str, code: &str| -> Result<()> {
        if let Some(value) = number(name) {
            if value.is_negative() || !value.is_integer() {
                return Err(SemauriError::semantic(
                    code,
                    format!("{display} must be a non-negative integer"),
                    Some(span),
                    None,
                ));
            }
        }
        Ok(())
    };

    match operation.name.as_str() {
        "configure_training" | "plan_training" => {
            positive_integer("epochs", "epochs", "S336")?;
            if let Some(optimizer) = string("optimizer") {
                if !matches!(optimizer.to_lowercase().as_str(), "adam" | "adamw" | "sgd") {
                    return Err(SemauriError::semantic(
                        "S336",
                        format!("Unsupported optimizer '{optimizer}'"),
                        Some(span),
                        Some("Supported optimizers: adam, adamw, sgd.".to_string()),
                    ));
                }
            }
            positive_number("learning_rate", "learning rate", "S336")?;
            positive_integer("batch_size", "batch size", "S336")?;
            non_negative_integer("seed", "seed", "S336")?;
            if operation.name == "plan_training" {
                if let Some(precision) = string("precision") {
                    if !matches!(precision.to_lowercase().as_str(), "fp32" | "fp16" | "bf16") {
                        return Err(SemauriError::semantic(
                            "S338",
                            format!("Unsupported training precision '{precision}'"),
                            Some(span),
                            Some("Supported precisions: fp32, fp16, bf16.".to_string()),
                        ));
                    }
                }
                positive_integer(
                    "gradient_accumulation",
                    "gradient accumulation",
                    "S338",
                )?;
                positive_integer("checkpoint_every", "checkpoint interval", "S338")?;
            }
        }
        "budget_resources" => {
            positive_number("memory_gb", "memory budget", "S342")?;
            positive_integer("workers", "worker count", "S342")?;
        }
        "transform_dataset" => {
            if let Some(transform) = string("transform") {
                if transform.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S340",
                        "dataset transform must be a non-empty string",
                        Some(span),
                        None,
                    ));
                }
            }
        }
        "split_dataset" => {
            if let Some(ratio) = number("ratio") {
                if !(ratio.as_f64() > 0.0 && ratio.as_f64() < 1.0) {
                    return Err(SemauriError::semantic(
                        "S340",
                        "dataset split ratio must be greater than zero and less than one",
                        Some(span),
                        None,
                    ));
                }
            }
            non_negative_integer("seed", "dataset split seed", "S340")?;
        }
        "evaluate" => {
            if let Some(metric) = string("metric") {
                if !matches!(
                    metric.to_lowercase().as_str(),
                    "accuracy" | "precision" | "recall" | "f1" | "loss" | "perplexity"
                ) {
                    return Err(SemauriError::semantic(
                        "S341",
                        format!("Unsupported evaluation metric '{metric}'"),
                        Some(span),
                        Some(
                            "Supported metrics: accuracy, precision, recall, f1, loss, perplexity."
                                .to_string(),
                        ),
                    ));
                }
            }
        }
        "build_cnn" => {
            positive_integer("classes", "class count", "S337")?;
            positive_integer("input_channels", "input channel count", "S337")?;
        }
        "freeze_component" => {
            if let Some(component) = string("component") {
                if component.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S337",
                        "component must be a non-empty string",
                        Some(span),
                        None,
                    ));
                }
            }
        }
        "apply_lora" => {
            positive_integer("rank", "LoRA rank", "S337")?;
            positive_number("alpha", "LoRA alpha", "S337")?;
        }
        "apply_qlora" => {
            positive_integer("rank", "QLoRA rank", "S339")?;
            positive_number("alpha", "QLoRA alpha", "S339")?;
            if let Some(bits) = number("quantization_bits") {
                if !matches!(bits, NumberValue::Int(4) | NumberValue::Int(8)) {
                    return Err(SemauriError::semantic(
                        "S339",
                        format!("Unsupported QLoRA quantization '{}' bits", bits),
                        Some(span),
                        Some("Supported quantization widths: 4, 8 bits.".to_string()),
                    ));
                }
            }
            if let Some(targets) = string("targets") {
                if targets.trim().is_empty() {
                    return Err(SemauriError::semantic(
                        "S339",
                        "QLoRA targets must be a non-empty string",
                        Some(span),
                        None,
                    ));
                }
            }
        }
        _ => {}
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct OptimizationPass {
    pub name: String,
    pub changes: usize,
}

#[derive(Clone, Debug)]
pub struct OptimizedHir {
    pub result: HirResult,
    pub changes: usize,
    pub passes: Vec<OptimizationPass>,
}

impl OptimizedHir {
    pub fn to_json(&self) -> JsonValue {
        let mut base = self.result.to_json();
        if let JsonValue::Object(ref mut object) = base {
            object.insert(
                "optimization".into(),
                json!({
                    "total_changes": self.changes,
                    "passes": self.passes.iter().map(|pass| json!({
                        "name": pass.name,
                        "changes": pass.changes
                    })).collect::<Vec<_>>()
                }),
            );
        }
        base
    }
}

fn optimize_hir(hir: &HirResult) -> OptimizedHir {
    let mut constants = HashMap::new();
    let mut constant_changes = 0usize;
    let folded = fold_statements(&hir.statements, &mut constants, &mut constant_changes);

    let mut control_changes = 0usize;
    let controlled = eliminate_dead_control_flow(&folded, &mut control_changes);

    let mut binding_changes = 0usize;
    let bindings = eliminate_dead_bindings(&controlled, &mut binding_changes);

    OptimizedHir {
        result: HirResult {
            statements: bindings,
            symbols: hir.symbols.clone(),
            span: hir.span,
        },
        changes: constant_changes + control_changes + binding_changes,
        passes: vec![
            OptimizationPass {
                name: "constant_folding".to_string(),
                changes: constant_changes,
            },
            OptimizationPass {
                name: "dead_control_flow".to_string(),
                changes: control_changes,
            },
            OptimizationPass {
                name: "dead_binding_elimination".to_string(),
                changes: binding_changes,
            },
        ],
    }
}

fn fold_statements(
    statements: &[HirStmt],
    constants: &mut HashMap<u64, HirExpr>,
    changes: &mut usize,
) -> Vec<HirStmt> {
    statements
        .iter()
        .map(|statement| match statement {
            HirStmt::Let {
                symbol_id,
                name,
                source_name,
                value,
                span,
            } => {
                let value = fold_expr(value, constants, changes);
                if is_constant_expr(&value) {
                    constants.insert(*symbol_id, value.clone());
                }
                HirStmt::Let {
                    symbol_id: *symbol_id,
                    name: name.clone(),
                    source_name: source_name.clone(),
                    value,
                    span: *span,
                }
            }
            HirStmt::If {
                condition,
                consequence,
                alternative,
                consequence_span,
                alternative_span,
                span,
            } => HirStmt::If {
                condition: fold_expr(condition, constants, changes),
                consequence: fold_statements(consequence, constants, changes),
                alternative: alternative
                    .as_ref()
                    .map(|body| fold_statements(body, constants, changes)),
                consequence_span: *consequence_span,
                alternative_span: *alternative_span,
                span: *span,
            },
            HirStmt::ForEach {
                iterator_symbol_id,
                iterator_name,
                iterator_source_name,
                iterable,
                body,
                body_span,
                span,
            } => {
                constants.remove(iterator_symbol_id);
                HirStmt::ForEach {
                    iterator_symbol_id: *iterator_symbol_id,
                    iterator_name: iterator_name.clone(),
                    iterator_source_name: iterator_source_name.clone(),
                    iterable: fold_expr(iterable, constants, changes),
                    body: fold_statements(body, constants, changes),
                    body_span: *body_span,
                    span: *span,
                }
            }
            HirStmt::DomainScope { domain, body, span } => HirStmt::DomainScope {
                domain: domain.clone(),
                body: fold_statements(body, constants, changes),
                span: *span,
            },
            HirStmt::DomainOperation {
                domain,
                operation,
                arguments,
                effects,
                return_type,
                span,
            } => HirStmt::DomainOperation {
                domain: domain.clone(),
                operation: operation.clone(),
                arguments: arguments
                    .iter()
                    .map(|(name, value)| {
                        (name.clone(), fold_expr(value, constants, changes))
                    })
                    .collect(),
                effects: effects.clone(),
                return_type: return_type.clone(),
                span: *span,
            },
            HirStmt::SetProperty {
                domain,
                target,
                property,
                value,
                span,
            } => HirStmt::SetProperty {
                domain: domain.clone(),
                target: target.clone(),
                property: property.clone(),
                value: fold_expr(value, constants, changes),
                span: *span,
            },
            other => other.clone(),
        })
        .collect()
}

fn fold_expr(
    expression: &HirExpr,
    constants: &HashMap<u64, HirExpr>,
    changes: &mut usize,
) -> HirExpr {
    match expression {
        HirExpr::SymbolRef {
            symbol_id, span, ..
        } => {
            if let Some(value) = constants.get(symbol_id) {
                *changes += 1;
                return clone_constant_with_span(value, *span);
            }
            expression.clone()
        }
        HirExpr::Promote {
            ty,
            from_type,
            value,
            span,
        } => HirExpr::Promote {
            ty: ty.clone(),
            from_type: from_type.clone(),
            value: Box::new(fold_expr(value, constants, changes)),
            span: *span,
        },
        HirExpr::List { ty, items, span } => HirExpr::List {
            ty: ty.clone(),
            items: items
                .iter()
                .map(|item| fold_expr(item, constants, changes))
                .collect(),
            span: *span,
        },
        HirExpr::Unary {
            ty,
            operator,
            operand,
            span,
        } => {
            let operand = fold_expr(operand, constants, changes);
            if let (
                UnaryOperator::Not,
                Some(ValueData::Boolean(value)),
            ) = (operator, literal_data(&operand))
            {
                *changes += 1;
                return HirExpr::Literal {
                    ty: Type::Boolean,
                    value: LiteralValue::Boolean(!value),
                    span: *span,
                };
            }
            HirExpr::Unary {
                ty: ty.clone(),
                operator: operator.clone(),
                operand: Box::new(operand),
                span: *span,
            }
        }
        HirExpr::Binary {
            ty,
            left,
            operator,
            right,
            span,
        } => {
            let left = fold_expr(left, constants, changes);
            if matches!(operator, BinaryOperator::And | BinaryOperator::Or) {
                if let Some(ValueData::Boolean(value)) = literal_data(&left) {
                    if matches!(operator, BinaryOperator::And) && !value {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Boolean,
                            value: LiteralValue::Boolean(false),
                            span: *span,
                        };
                    }
                    if matches!(operator, BinaryOperator::Or) && value {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Boolean,
                            value: LiteralValue::Boolean(true),
                            span: *span,
                        };
                    }
                }
            }
            let right = fold_expr(right, constants, changes);
            if matches!(operator, BinaryOperator::Divide)
                && literal_data(&right)
                    .is_some_and(|value| matches!(value, ValueData::Number(number) if number.is_zero()))
            {
                return HirExpr::Binary {
                    ty: ty.clone(),
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: Box::new(right),
                    span: *span,
                };
            }
            if let (Some(left_value), Some(right_value)) =
                (literal_data(&left), literal_data(&right))
            {
                if let Some(value) = eval_constant_binary(operator, &left_value, &right_value) {
                    *changes += 1;
                    return hir_literal_from_data(ty.clone(), value, *span);
                }
            }
            HirExpr::Binary {
                ty: ty.clone(),
                left: Box::new(left),
                operator: operator.clone(),
                right: Box::new(right),
                span: *span,
            }
        }
        HirExpr::DomainOperation {
            ty,
            domain,
            operation,
            arguments,
            effects,
            span,
        } => HirExpr::DomainOperation {
            ty: ty.clone(),
            domain: domain.clone(),
            operation: operation.clone(),
            arguments: arguments
                .iter()
                .map(|(name, value)| {
                    (name.clone(), fold_expr(value, constants, changes))
                })
                .collect(),
            effects: effects.clone(),
            span: *span,
        },
        HirExpr::Literal { .. } => expression.clone(),
    }
}

fn is_constant_expr(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Literal { .. } => true,
        HirExpr::List { items, .. } => items.iter().all(is_constant_expr),
        _ => false,
    }
}

fn clone_constant_with_span(value: &HirExpr, span: SourceSpan) -> HirExpr {
    match value {
        HirExpr::Literal { ty, value, .. } => HirExpr::Literal {
            ty: ty.clone(),
            value: value.clone(),
            span,
        },
        HirExpr::List { ty, items, .. } => HirExpr::List {
            ty: ty.clone(),
            items: items
                .iter()
                .map(|item| clone_constant_with_span(item, item.span()))
                .collect(),
            span,
        },
        _ => value.clone(),
    }
}

fn eval_constant_binary(
    operator: &BinaryOperator,
    left: &ValueData,
    right: &ValueData,
) -> Option<ValueData> {
    match (operator, left, right) {
        (BinaryOperator::Add, ValueData::Number(a), ValueData::Number(b)) => {
            Some(ValueData::Number(NumberValue::from_f64(
                a.as_f64() + b.as_f64(),
            )))
        }
        (BinaryOperator::Subtract, ValueData::Number(a), ValueData::Number(b)) => {
            Some(ValueData::Number(NumberValue::from_f64(
                a.as_f64() - b.as_f64(),
            )))
        }
        (BinaryOperator::Multiply, ValueData::Number(a), ValueData::Number(b)) => {
            Some(ValueData::Number(NumberValue::from_f64(
                a.as_f64() * b.as_f64(),
            )))
        }
        (BinaryOperator::Divide, ValueData::Number(a), ValueData::Number(b))
            if !b.is_zero() =>
        {
            Some(ValueData::Number(NumberValue::Float(
                a.as_f64() / b.as_f64(),
            )))
        }
        (BinaryOperator::GreaterThan, ValueData::Number(a), ValueData::Number(b)) => {
            Some(ValueData::Boolean(a.as_f64() > b.as_f64()))
        }
        (BinaryOperator::LessThan, ValueData::Number(a), ValueData::Number(b)) => {
            Some(ValueData::Boolean(a.as_f64() < b.as_f64()))
        }
        (
            BinaryOperator::GreaterThanOrEqual,
            ValueData::Number(a),
            ValueData::Number(b),
        ) => Some(ValueData::Boolean(a.as_f64() >= b.as_f64())),
        (
            BinaryOperator::LessThanOrEqual,
            ValueData::Number(a),
            ValueData::Number(b),
        ) => Some(ValueData::Boolean(a.as_f64() <= b.as_f64())),
        (BinaryOperator::Equal, a, b) => Some(ValueData::Boolean(a == b)),
        (BinaryOperator::And, ValueData::Boolean(a), ValueData::Boolean(b)) => {
            Some(ValueData::Boolean(*a && *b))
        }
        (BinaryOperator::Or, ValueData::Boolean(a), ValueData::Boolean(b)) => {
            Some(ValueData::Boolean(*a || *b))
        }
        _ => None,
    }
}

fn hir_literal_from_data(ty: Type, value: ValueData, span: SourceSpan) -> HirExpr {
    let literal = match value {
        ValueData::Number(value) => LiteralValue::Number(value),
        ValueData::Boolean(value) => LiteralValue::Boolean(value),
        ValueData::String(value) => LiteralValue::String(value),
        ValueData::Color(value) => LiteralValue::Color(value),
        _ => unreachable!(),
    };
    HirExpr::Literal {
        ty,
        value: literal,
        span,
    }
}

fn eliminate_dead_control_flow(statements: &[HirStmt], changes: &mut usize) -> Vec<HirStmt> {
    let mut output = Vec::new();
    for statement in statements {
        match statement {
            HirStmt::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                let consequence = eliminate_dead_control_flow(consequence, changes);
                let alternative = alternative
                    .as_ref()
                    .map(|body| eliminate_dead_control_flow(body, changes));
                if let Some(ValueData::Boolean(value)) = literal_data(condition) {
                    *changes += 1;
                    if value {
                        output.extend(consequence);
                    } else if let Some(alternative) = alternative {
                        output.extend(alternative);
                    }
                } else {
                    let mut rebuilt = statement.clone();
                    if let HirStmt::If {
                        consequence: target_consequence,
                        alternative: target_alternative,
                        ..
                    } = &mut rebuilt
                    {
                        *target_consequence = consequence;
                        *target_alternative = alternative;
                    }
                    output.push(rebuilt);
                }
            }
            HirStmt::DomainScope { domain, body, span } => {
                output.push(HirStmt::DomainScope {
                    domain: domain.clone(),
                    body: eliminate_dead_control_flow(body, changes),
                    span: *span,
                });
            }
            HirStmt::ForEach {
                iterator_symbol_id,
                iterator_name,
                iterator_source_name,
                iterable,
                body,
                body_span,
                span,
            } => output.push(HirStmt::ForEach {
                iterator_symbol_id: *iterator_symbol_id,
                iterator_name: iterator_name.clone(),
                iterator_source_name: iterator_source_name.clone(),
                iterable: iterable.clone(),
                body: eliminate_dead_control_flow(body, changes),
                body_span: *body_span,
                span: *span,
            }),
            _ => output.push(statement.clone()),
        }
    }
    output
}

fn eliminate_dead_bindings(statements: &[HirStmt], changes: &mut usize) -> Vec<HirStmt> {
    let mut live = HashSet::new();
    let mut output = Vec::new();
    for statement in statements.iter().rev() {
        match statement {
            HirStmt::Let {
                symbol_id, value, ..
            } if !live.contains(symbol_id) && removable_expression(value) => {
                *changes += 1;
            }
            HirStmt::Let {
                symbol_id, value, ..
            } => {
                live.remove(symbol_id);
                collect_symbol_refs_expr(value, &mut live);
                output.push(statement.clone());
            }
            HirStmt::If {
                condition,
                consequence,
                alternative,
                consequence_span,
                alternative_span,
                span,
            } => {
                collect_symbol_refs_expr(condition, &mut live);
                let consequence = eliminate_dead_bindings(consequence, changes);
                let alternative = alternative
                    .as_ref()
                    .map(|body| eliminate_dead_bindings(body, changes));
                let mut nested_refs = HashSet::new();
                collect_symbol_refs_stmts(&consequence, &mut nested_refs);
                if let Some(alternative) = &alternative {
                    collect_symbol_refs_stmts(alternative, &mut nested_refs);
                }
                live.extend(nested_refs);
                output.push(HirStmt::If {
                    condition: condition.clone(),
                    consequence,
                    alternative,
                    consequence_span: *consequence_span,
                    alternative_span: *alternative_span,
                    span: *span,
                });
            }
            HirStmt::ForEach {
                iterator_symbol_id,
                iterator_name,
                iterator_source_name,
                iterable,
                body,
                body_span,
                span,
            } => {
                collect_symbol_refs_expr(iterable, &mut live);
                let body = eliminate_dead_bindings(body, changes);
                let mut body_refs = HashSet::new();
                collect_symbol_refs_stmts(&body, &mut body_refs);
                body_refs.remove(iterator_symbol_id);
                live.extend(body_refs);
                output.push(HirStmt::ForEach {
                    iterator_symbol_id: *iterator_symbol_id,
                    iterator_name: iterator_name.clone(),
                    iterator_source_name: iterator_source_name.clone(),
                    iterable: iterable.clone(),
                    body,
                    body_span: *body_span,
                    span: *span,
                });
            }
            HirStmt::DomainScope { domain, body, span } => {
                let body = eliminate_dead_bindings(body, changes);
                let mut body_refs = HashSet::new();
                collect_symbol_refs_stmts(&body, &mut body_refs);
                live.extend(body_refs);
                output.push(HirStmt::DomainScope {
                    domain: domain.clone(),
                    body,
                    span: *span,
                });
            }
            other => {
                collect_symbol_refs_stmt(other, &mut live);
                output.push(other.clone());
            }
        }
    }
    output.reverse();
    output
}

fn removable_expression(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Literal { .. } | HirExpr::SymbolRef { .. } => true,
        HirExpr::Promote { value, .. } => removable_expression(value),
        HirExpr::List { items, .. } => items.iter().all(removable_expression),
        HirExpr::Unary { operand, .. } => removable_expression(operand),
        HirExpr::Binary {
            operator,
            left,
            right,
            ..
        } => {
            if !removable_expression(left) || !removable_expression(right) {
                return false;
            }
            if matches!(operator, BinaryOperator::Divide) {
                return literal_data(right).is_some_and(
                    |value| matches!(value, ValueData::Number(number) if !number.is_zero()),
                );
            }
            true
        }
        HirExpr::DomainOperation { .. } => false,
    }
}

fn collect_symbol_refs_stmts(statements: &[HirStmt], refs: &mut HashSet<u64>) {
    for statement in statements {
        collect_symbol_refs_stmt(statement, refs);
    }
}

fn collect_symbol_refs_stmt(statement: &HirStmt, refs: &mut HashSet<u64>) {
    match statement {
        HirStmt::DomainScope { body, .. } => collect_symbol_refs_stmts(body, refs),
        HirStmt::Let { value, .. } => collect_symbol_refs_expr(value, refs),
        HirStmt::If {
            condition,
            consequence,
            alternative,
            ..
        } => {
            collect_symbol_refs_expr(condition, refs);
            collect_symbol_refs_stmts(consequence, refs);
            if let Some(alternative) = alternative {
                collect_symbol_refs_stmts(alternative, refs);
            }
        }
        HirStmt::ForEach {
            iterable, body, ..
        } => {
            collect_symbol_refs_expr(iterable, refs);
            collect_symbol_refs_stmts(body, refs);
        }
        HirStmt::DomainOperation { arguments, .. } => {
            for value in arguments.values() {
                collect_symbol_refs_expr(value, refs);
            }
        }
        HirStmt::SetProperty { value, .. } => collect_symbol_refs_expr(value, refs),
        _ => {}
    }
}

fn collect_symbol_refs_expr(expr: &HirExpr, refs: &mut HashSet<u64>) {
    match expr {
        HirExpr::SymbolRef { symbol_id, .. } => {
            refs.insert(*symbol_id);
        }
        HirExpr::Promote { value, .. } | HirExpr::Unary { operand: value, .. } => {
            collect_symbol_refs_expr(value, refs)
        }
        HirExpr::List { items, .. } => {
            for item in items {
                collect_symbol_refs_expr(item, refs);
            }
        }
        HirExpr::Binary { left, right, .. } => {
            collect_symbol_refs_expr(left, refs);
            collect_symbol_refs_expr(right, refs);
        }
        HirExpr::DomainOperation { arguments, .. } => {
            for value in arguments.values() {
                collect_symbol_refs_expr(value, refs);
            }
        }
        HirExpr::Literal { .. } => {}
    }
}

#[derive(Clone, Debug)]
pub struct EffectUse {
    pub effect: String,
    pub domain: String,
    pub operation: String,
    pub span: SourceSpan,
}

impl EffectUse {
    fn to_json(&self) -> JsonValue {
        json!({
            "effect": self.effect,
            "domain": self.domain,
            "operation": self.operation,
            "span": self.span
        })
    }
}

#[derive(Clone, Debug)]
pub struct EffectAnalysis {
    pub uses: Vec<EffectUse>,
}

impl EffectAnalysis {
    pub fn effects(&self) -> Vec<String> {
        let mut effects = self
            .uses
            .iter()
            .map(|usage| usage.effect.clone())
            .collect::<Vec<_>>();
        effects.sort();
        effects.dedup();
        effects
    }

    pub fn pure(&self) -> bool {
        self.uses.is_empty()
    }

    pub fn to_json(&self) -> JsonValue {
        json!({
            "effects": self.effects(),
            "pure": self.pure(),
            "uses": self.uses.iter().map(EffectUse::to_json).collect::<Vec<_>>()
        })
    }
}

fn analyze_effects(hir: &HirResult) -> EffectAnalysis {
    let mut uses = Vec::new();
    collect_effects_statements(&hir.statements, &mut uses);
    EffectAnalysis { uses }
}

fn collect_effects_statements(statements: &[HirStmt], uses: &mut Vec<EffectUse>) {
    for statement in statements {
        match statement {
            HirStmt::DomainScope { body, .. } => collect_effects_statements(body, uses),
            HirStmt::Let { value, .. } => collect_effects_expr(value, uses),
            HirStmt::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                collect_effects_expr(condition, uses);
                collect_effects_statements(consequence, uses);
                if let Some(alternative) = alternative {
                    collect_effects_statements(alternative, uses);
                }
            }
            HirStmt::ForEach {
                iterable, body, ..
            } => {
                collect_effects_expr(iterable, uses);
                collect_effects_statements(body, uses);
            }
            HirStmt::DomainOperation {
                domain,
                operation,
                arguments,
                effects,
                span,
                ..
            } => {
                for effect in effects {
                    uses.push(EffectUse {
                        effect: effect.clone(),
                        domain: domain.clone(),
                        operation: operation.clone(),
                        span: *span,
                    });
                }
                for argument in arguments.values() {
                    collect_effects_expr(argument, uses);
                }
            }
            HirStmt::SetProperty { value, .. } => collect_effects_expr(value, uses),
            _ => {}
        }
    }
}

fn collect_effects_expr(expr: &HirExpr, uses: &mut Vec<EffectUse>) {
    match expr {
        HirExpr::DomainOperation {
            domain,
            operation,
            arguments,
            effects,
            span,
            ..
        } => {
            for effect in effects {
                uses.push(EffectUse {
                    effect: effect.clone(),
                    domain: domain.clone(),
                    operation: operation.clone(),
                    span: *span,
                });
            }
            for argument in arguments.values() {
                collect_effects_expr(argument, uses);
            }
        }
        HirExpr::Promote { value, .. } | HirExpr::Unary { operand: value, .. } => {
            collect_effects_expr(value, uses)
        }
        HirExpr::List { items, .. } => {
            for item in items {
                collect_effects_expr(item, uses);
            }
        }
        HirExpr::Binary { left, right, .. } => {
            collect_effects_expr(left, uses);
            collect_effects_expr(right, uses);
        }
        _ => {}
    }
}

#[derive(Clone, Debug)]
pub struct CapabilityPolicy {
    allowed: BTreeSet<String>,
}

impl CapabilityPolicy {
    pub fn new<I, S>(allowed: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed: allowed.into_iter().map(Into::into).collect(),
        }
    }

    pub fn allow_none() -> Self {
        Self::new(Vec::<String>::new())
    }

    pub fn validate(&self, analysis: &EffectAnalysis) -> Result<()> {
        let missing = analysis
            .effects()
            .into_iter()
            .filter(|effect| !self.allowed.contains(effect))
            .collect::<Vec<_>>();
        if missing.is_empty() {
            return Ok(());
        }
        let first_use = missing
            .iter()
            .find_map(|effect| analysis.uses.iter().find(|usage| &usage.effect == effect));
        Err(SemauriError::semantic(
            "S334",
            format!(
                "Program requires capabilities not allowed by policy: {}",
                missing.join(", ")
            ),
            first_use.map(|usage| usage.span),
            Some(
                "Inspect required effects and explicitly allow only the capabilities this execution environment should grant."
                    .to_string(),
            ),
        ))
    }
}

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
pub enum Artifact {
    Web(WebDocument),
    Schema(SchemaDocument),
    Filesystem(OperationPlan),
}

impl Artifact {
    fn to_json(&self) -> JsonValue {
        match self {
            Artifact::Web(value) => value.to_json(),
            Artifact::Schema(value) => value.to_json(),
            Artifact::Filesystem(value) => value.to_json(),
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
                let condition = self.evaluate(condition)?;
                let BoundValue::Compile {
                    ty: Type::Boolean,
                    data: ValueData::Boolean(value),
                } = condition
                else {
                    return Err(SemauriError::semantic(
                        "S316",
                        "If condition must evaluate to boolean",
                        Some(condition_span(condition_ref_placeholder())),
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
                return Err(SemauriError::semantic(
                    "S326",
                    format!("Unsupported artifact '{kind}' in domain '{domain}'"),
                    Some(span),
                    None,
                ))
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

// These two helpers keep the impossible fallback branch in lower_statement small.
fn condition_ref_placeholder() -> SourceSpan {
    SourceSpan::point(1, 1)
}

fn condition_span(span: SourceSpan) -> SourceSpan {
    span
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

#[derive(Clone, Debug)]
pub struct Compiler {
    domains: DomainRegistry,
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
            let content = render_backend(&backend_name, &unit.artifact)?;
            outputs.push(DomainOutput {
                domain: unit.domain.clone(),
                backend: backend_name,
                content,
            });
        }
        Ok(outputs)
    }
}

fn render_backend(name: &str, artifact: &Artifact) -> Result<String> {
    match name {
        "html" => render_html(artifact),
        "json-schema" => render_json_schema(artifact),
        "posix-sh" => render_posix_shell(artifact),
        other => Err(SemauriError::backend(
            "S402",
            format!("Unknown backend '{other}'"),
            None,
        )),
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
                if result.outputs.len() != 1
                    || result.outputs[0].domain != "filesystem"
                    || result.outputs[0].backend != "posix-sh"
                {
                    return Err(SemauriError::backend(
                        "S407",
                        "run currently supports one filesystem program rendered by posix-sh",
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
    "Usage: semauri <command> [options] [FILE]\n\nCommands:\n  domains                  Print registered semantic domains as JSON\n  tokens FILE              Print lexer tokens as JSON\n  ast FILE                 Print the parsed syntax AST as JSON\n  hir FILE                 Print typed HIR before optimization\n  optimize FILE            Print optimized HIR and pass statistics\n  symbols FILE             Print semantic symbols as JSON\n  effects FILE             Print statically required effects/capabilities\n  plan FILE                Print the lowered runtime operation plan as JSON\n  explain FILE             Explain semantic decisions\n  check FILE [--allow ...] Validate source and optionally enforce capabilities\n  build FILE [-o PATH]     Compile one or more build-time domain outputs\n  run FILE --allow EFFECT  Execute supported effects after explicit authorization\n  version                  Print version\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_preserves_spans_and_domain_metadata() {
        let compiler = Compiler::new();
        let tokens = compiler
            .tokenize("Create a web for a pet store.")
            .unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Create);
        assert_eq!(tokens[2].kind, TokenKind::DomainArtifact);
        assert_eq!(tokens[0].span, SourceSpan::new(1, 1, 1, 7));
    }

    #[test]
    fn readme_program_compiles_to_red_button() {
        let source = r#"
Let price be 18.
Let tax be 4.
Let total be price plus tax.
Create a web called Pet Shop.
Add a button called Buy.
If total is greater than 20:
  Set the color of the button called Buy to red.
Otherwise:
  Set the color of the button called Buy to green.
End.
"#;
        let result = Compiler::new().compile(source, None).unwrap();
        let output = result.output.unwrap();
        assert!(output.contains("<title>Pet Shop</title>"));
        assert!(output.contains("<button style=\"color: red\">Buy</button>"));
        assert!(!output.contains("color: green"));
    }

    #[test]
    fn filesystem_plan_and_effects_are_explicit() {
        let source = r#"
Write "hello" to "notes.txt".
Copy "notes.txt" to "backup.txt".
"#;
        let compiler = Compiler::new();
        let result = compiler.compile(source, None).unwrap();
        assert_eq!(result.backend.as_deref(), Some("posix-sh"));
        assert!(result.output.as_deref().unwrap().contains("cp notes.txt backup.txt"));
        assert_eq!(
            result.effects.effects(),
            vec!["filesystem_read".to_string(), "filesystem_write".to_string()]
        );
    }

    #[test]
    fn ml_operations_form_runtime_ssa_plan() {
        let source = r#"
Within ml:
  Let dataset be Open dataset "./images".
  Let model be Load model "resnet18".
  Let device be Select device "cuda".
  Let run be Train model using dataset on device for 10 epochs.
End.
"#;
        let plan = Compiler::new().runtime_plan(source).unwrap();
        assert_eq!(plan.operations.len(), 4);
        assert_eq!(plan.operations[3].name, "train");
        assert_eq!(
            plan.operations[3].result.as_ref().unwrap().ty.to_string(),
            "ml.training_run"
        );
    }
}
