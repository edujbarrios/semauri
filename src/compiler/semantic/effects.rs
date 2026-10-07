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

