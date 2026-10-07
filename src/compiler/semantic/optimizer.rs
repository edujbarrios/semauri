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
        (BinaryOperator::NotEqual, a, b) => Some(ValueData::Boolean(a != b)),
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

