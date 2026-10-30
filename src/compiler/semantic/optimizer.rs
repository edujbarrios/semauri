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
        HirExpr::Repeat { ty, count, value, span } => HirExpr::Repeat {
            ty: ty.clone(),
            count: Box::new(fold_expr(count, constants, changes)),
            value: Box::new(fold_expr(value, constants, changes)),
            span: *span,
        },
        HirExpr::Conditional { ty, condition, consequence, alternative, span } => {
            // Never evaluate/fold the unselected branch. This also prevents
            // overflow or divide-by-zero in unreachable expressions.
            let condition = fold_expr(condition, constants, changes);
            if let HirExpr::Literal {
                value: LiteralValue::Boolean(selected), ..
            } = &condition {
                *changes += 1;
                return fold_expr(if *selected { consequence } else { alternative }, constants, changes);
            }
            HirExpr::Conditional {
                ty: ty.clone(),
                condition: Box::new(condition),
                consequence: Box::new(fold_expr(consequence, constants, changes)),
                alternative: Box::new(fold_expr(alternative, constants, changes)),
                span: *span,
            }
        }
        HirExpr::FunctionCall {
            ty, name, parameters, arguments, body, span,
        } => HirExpr::FunctionCall {
            ty: ty.clone(),
            name: name.clone(),
            parameters: parameters.clone(),
            arguments: arguments.iter().map(|arg| fold_expr(arg, constants, changes)).collect(),
            // Parameter bindings are scoped to each invocation.
            // Do not specialize the body using caller constants.
            body: body.clone(),
            span: *span,
        },
        HirExpr::Reduce {
            ty, iterator_symbol_id, accumulator_symbol_id, iterator_name, accumulator_name,
            iterable, initial, body, span,
        } => HirExpr::Reduce {
            ty: ty.clone(),
            iterator_symbol_id: *iterator_symbol_id,
            accumulator_symbol_id: *accumulator_symbol_id,
            iterator_name: iterator_name.clone(),
            accumulator_name: accumulator_name.clone(),
            iterable: Box::new(fold_expr(iterable, constants, changes)),
            initial: Box::new(fold_expr(initial, constants, changes)),
            // Scoped step bindings prevent safe caller-based specialization.
            body: body.clone(),
            span: *span,
        },
        HirExpr::ZipTransform {
            ty, left_symbol_id, left_name, left_list,
            right_symbol_id, right_name, right_list, body, span,
        } => HirExpr::ZipTransform {
            ty: ty.clone(), left_symbol_id: *left_symbol_id,
            left_name: left_name.clone(),
            left_list: Box::new(fold_expr(left_list, constants, changes)),
            right_symbol_id: *right_symbol_id, right_name: right_name.clone(),
            right_list: Box::new(fold_expr(right_list, constants, changes)),
            body: body.clone(), span: *span,
        },
        HirExpr::ListTransform {
            ty, kind, iterator_symbol_id, iterator_name, iterable, body, span,
        } => HirExpr::ListTransform {
            ty: ty.clone(),
            kind: *kind,
            iterator_symbol_id: *iterator_symbol_id,
            iterator_name: iterator_name.clone(),
            iterable: Box::new(fold_expr(iterable, constants, changes)),
            body: Box::new(fold_expr(body, constants, changes)),
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
            match (operator, literal_data(&operand)) {
                (UnaryOperator::Not, Some(ValueData::Boolean(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::Boolean,
                        value: LiteralValue::Boolean(!value),
                        span: *span,
                    };
                }
                (UnaryOperator::Absolute, Some(ValueData::Number(value))) => {
                    let absolute = match value {
                        NumberValue::Int(integer) => integer.checked_abs().map(NumberValue::Int),
                        NumberValue::Float(number) if number.is_finite() => Some(NumberValue::Float(number.abs())),
                        _ => None,
                    };
                    // Keep invalid values in HIR so evaluation emits S388.
                    if let Some(absolute) = absolute {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(absolute),
                            span: *span,
                        };
                    }
                }
                (UnaryOperator::Sign, Some(ValueData::Number(value))) => {
                    if let Some(sign) = numeric_sign(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(sign),
                            span: *span,
                        };
                    }
                    // Non-finite values retain a runtime S418 diagnostic.
                }
                (operator @ (UnaryOperator::Floor | UnaryOperator::Ceiling),
                    Some(ValueData::Number(value))) => {
                    if let Some(rounded) = numeric_round(value, *operator == UnaryOperator::Ceiling) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(rounded),
                            span: *span,
                        };
                    }
                    // Let evaluator emit S420 for non-finite values.
                }
                (UnaryOperator::Round, Some(ValueData::Number(value))) => {
                    if let Some(rounded) = numeric_nearest(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(rounded),
                            span: *span,
                        };
                    }
                    // Preserve invalid float values for runtime S428.
                }
                (UnaryOperator::Truncate, Some(ValueData::Number(value))) => {
                    if let Some(truncated) = numeric_truncate(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(truncated),
                            span: *span,
                        };
                    }
                    // Retain S430 validation for invalid inputs.
                }
                (UnaryOperator::Factorial, Some(ValueData::Number(value))) => {
                    if let Some(factorial) = numeric_factorial(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(factorial),
                            span: *span,
                        };
                    }
                    // Keep invalid factorial inputs to report S432 at evaluation.
                }
                (UnaryOperator::Prime, Some(ValueData::Number(value))) => {
                    if let Some(prime) = numeric_prime(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Boolean,
                            value: LiteralValue::Boolean(prime),
                            span: *span,
                        };
                    }
                    // Preserve a source-located S436 for invalid numeric values.
                }
                (UnaryOperator::Cube, Some(ValueData::Number(value))) => {
                    if let Some(cube) = numeric_cube(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(cube),
                            span: *span,
                        };
                    }
                    // Preserve failed arithmetic for a source-located S434 diagnostic.
                }
                (UnaryOperator::Square, Some(ValueData::Number(value))) => {
                    if let Some(square) = numeric_square(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(square),
                            span: *span,
                        };
                    }
                    // Preserve overflow to emit S422 at evaluation.
                }
                (UnaryOperator::Digits, Some(ValueData::Number(value))) => {
                    if let Some(result) = numeric_digits(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(result),
                            span: *span,
                        };
                    }
                    // Invalid fractional/out-of-range inputs retain S442.
                }
                (UnaryOperator::Reciprocal, Some(ValueData::Number(value))) => {
                    if let Some(result) = numeric_reciprocal(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(result),
                            span: *span,
                        };
                    }
                    // Keep invalid inputs for a source-located S440 at evaluation.
                }
                (UnaryOperator::CubeRoot, Some(ValueData::Number(value))) => {
                    if let Some(root) = numeric_cbrt(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(root),
                            span: *span,
                        };
                    }
                    // Keep non-finite inputs for a source-located S438 diagnostic.
                }
                (operator @ (UnaryOperator::Exponential | UnaryOperator::NaturalLog
                    | UnaryOperator::Sine | UnaryOperator::Cosine), Some(ValueData::Number(value))) => {
                    if let Some(result) = numeric_transcendental(value, operator.clone()) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(result),
                            span: *span,
                        };
                    }
                    // Keep invalid domains/overflow to emit source-located S466.
                }
                (UnaryOperator::SquareRoot, Some(ValueData::Number(value))) => {
                    if let Some(root) = numeric_sqrt(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(root),
                            span: *span,
                        };
                    }
                    // Negative or non-finite roots keep a located S424 diagnostic.
                }
                (operator @ (UnaryOperator::Even | UnaryOperator::Odd),
                    Some(ValueData::Number(value))) => {
                    if let Some(even) = numeric_is_even(value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Boolean,
                            value: LiteralValue::Boolean(if *operator == UnaryOperator::Even { even } else { !even }),
                            span: *span,
                        };
                    }
                    // Invalid or out-of-range decimal values remain for S426.
                }
                (UnaryOperator::Negate, Some(ValueData::Number(value))) => {
                    let negated = match value {
                        NumberValue::Int(integer) => integer.checked_neg().map(NumberValue::Int),
                        NumberValue::Float(number) => Some(NumberValue::Float(-number)),
                    };
                    // Do not fold an overflowing negation; evaluation reports S358.
                    if let Some(negated) = negated {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(negated),
                            span: *span,
                        };
                    }
                }
                (operator @ (UnaryOperator::Characters | UnaryOperator::Codepoints
                    | UnaryOperator::Codepoint | UnaryOperator::CharacterFrom), Some(value)) => {
                    if let Some(result) = eval_unicode_scalar(operator, &value) {
                        *changes += 1;
                        return match result {
                            ValueData::List(items) => {
                                let item_type = if *operator == UnaryOperator::Characters {
                                    Type::String
                                } else {
                                    Type::Number
                                };
                                HirExpr::List {
                                    ty: ty.clone(),
                                    items: items.into_iter().map(|item|
                                        hir_literal_from_data(item_type.clone(), item, *span)
                                    ).collect(),
                                    span: *span,
                                }
                            }
                            scalar => hir_literal_from_data(ty.clone(), scalar, *span),
                        };
                    }
                }
                (UnaryOperator::JsonKeys, Some(ValueData::String(source))) => {
                    if let Some(keys) = json_object_keys(&source) {
                        *changes += 1;
                        return HirExpr::List {
                            ty: Type::List(Box::new(Type::String)),
                            items: keys.into_iter().map(|key| HirExpr::Literal {
                                ty: Type::String,
                                value: LiteralValue::String(key),
                                span: *span,
                            }).collect(),
                            span: *span,
                        };
                    }
                }
                (operator @ (UnaryOperator::DecodeJsonString | UnaryOperator::DecodeJsonNumber
                    | UnaryOperator::DecodeJsonBoolean | UnaryOperator::DecodeJsonStrings
                    | UnaryOperator::DecodeJsonNumbers | UnaryOperator::DecodeJsonBooleans),
                    Some(ValueData::String(source))) => {
                    if let Some(decoded) = decode_json_text(&source, operator) {
                        *changes += 1;
                        let expected_type = json_decode_type(operator)
                            .expect("known JSON decode operator");
                        return match decoded {
                            ValueData::List(items) => HirExpr::List {
                                ty: expected_type,
                                items: items.into_iter().map(|item| {
                                    let item_type = match &item {
                                        ValueData::String(_) => Type::String,
                                        ValueData::Number(_) => Type::Number,
                                        ValueData::Boolean(_) => Type::Boolean,
                                        _ => unreachable!(),
                                    };
                                    hir_literal_from_data(item_type, item, *span)
                                }).collect(),
                                span: *span,
                            },
                            scalar => hir_literal_from_data(expected_type, scalar, *span),
                        };
                    }
                    // Keep invalid JSON in the HIR for a located evaluation error.
                }
                (UnaryOperator::ParseNumber, Some(ValueData::String(value))) => {
                    if let Some(parsed) = parse_number_text(&value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(parsed),
                            span: *span,
                        };
                    }
                }
                (UnaryOperator::ParseBoolean, Some(ValueData::String(value))) => {
                    if let Some(parsed) = parse_boolean_text(&value) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Boolean,
                            value: LiteralValue::Boolean(parsed),
                            span: *span,
                        };
                    }
                }
                (UnaryOperator::Json, Some(value)) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(serde_json::to_string(&value.to_json())
                            .expect("serde_json::Value always serializes")),
                        span: *span,
                    };
                }
                (UnaryOperator::Trim, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(value.trim().to_string()),
                        span: *span,
                    };
                }
                (UnaryOperator::Words | UnaryOperator::Lines, Some(ValueData::String(value))) => {
                    let segments: Vec<&str> = match operator {
                        UnaryOperator::Words => value.split_whitespace().collect(),
                        UnaryOperator::Lines => value.lines().collect(),
                        _ => unreachable!(),
                    };
                    *changes += 1;
                    return HirExpr::List {
                        ty: Type::List(Box::new(Type::String)),
                        items: segments.into_iter().map(|part| HirExpr::Literal {
                            ty: Type::String,
                            value: LiteralValue::String(part.to_string()),
                            span: *span,
                        }).collect(),
                        span: *span,
                    };
                }
                (UnaryOperator::Uppercase, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(value.to_uppercase()),
                        span: *span,
                    };
                }
                (UnaryOperator::Swapcase, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(text_swapcase(&value)),
                        span: *span,
                    };
                }
                (UnaryOperator::Titlecase, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(text_titlecase(&value)),
                        span: *span,
                    };
                }
                (UnaryOperator::Uncapitalize, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(text_uncapitalize(&value)),
                        span: *span,
                    };
                }
                (UnaryOperator::Capitalize, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(text_capitalize(&value)),
                        span: *span,
                    };
                }
                (UnaryOperator::Lowercase, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(value.to_lowercase()),
                        span: *span,
                    };
                }
                (UnaryOperator::Reverse, Some(ValueData::String(value))) => {
                    *changes += 1;
                    return HirExpr::Literal {
                        ty: Type::String,
                        value: LiteralValue::String(value.chars().rev().collect()),
                        span: *span,
                    };
                }
                (UnaryOperator::Length, Some(ValueData::String(value))) => {
                    if let Ok(length) = i64::try_from(value.chars().count()) {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::Number,
                            value: LiteralValue::Number(NumberValue::Int(length)),
                            span: *span,
                        };
                    }
                }
                _ => {}
            }
            if *operator == UnaryOperator::Length {
                if let HirExpr::List { items, .. } = &operand {
                    // Do not erase an effectful list element just to count it.
                    if items.iter().all(is_constant_expr) {
                        if let Ok(length) = i64::try_from(items.len()) {
                            *changes += 1;
                            return HirExpr::Literal {
                                ty: Type::Number,
                                value: LiteralValue::Number(NumberValue::Int(length)),
                                span: *span,
                            };
                        }
                    }
                }
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
            if matches!(operator, BinaryOperator::Divide | BinaryOperator::Modulo)
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
            // A list-valued substring search folds to HIR list expressions,
            // not to unsupported scalar literals.
            if *operator == BinaryOperator::TextPositions {
                if let (Some(ValueData::String(needle)), Some(ValueData::String(text))) =
                    (literal_data(&left), literal_data(&right))
                {
                    if let Some(positions) = text_positions(&text, &needle) {
                        *changes += 1;
                        return HirExpr::List {
                            ty: Type::List(Box::new(Type::Number)),
                            items: positions.into_iter().map(|index| HirExpr::Literal {
                                ty: Type::Number,
                                value: LiteralValue::Number(NumberValue::Int(index)),
                                span: *span,
                            }).collect(),
                            span: *span,
                        };
                    }
                }
            }
            // Fold string-list joins directly: literal_data intentionally does not
            // represent list expressions as scalar literals.
            if *operator == BinaryOperator::Join {
                if let (HirExpr::List { items, .. }, Some(ValueData::String(separator))) =
                    (&left, literal_data(&right))
                {
                    let parts = items.iter().map(|item| match literal_data(item) {
                        Some(ValueData::String(value)) => Some(value),
                        _ => None,
                    }).collect::<Option<Vec<_>>>();
                    if let Some(parts) = parts {
                        *changes += 1;
                        return HirExpr::Literal {
                            ty: Type::String,
                            value: LiteralValue::String(parts.join(&separator)),
                            span: *span,
                        };
                    }
                }
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
        (BinaryOperator::Add, ValueData::String(a), ValueData::String(b)) => {
            Some(ValueData::String(format!("{a}{b}")))
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
        (BinaryOperator::Modulo, ValueData::Number(a), ValueData::Number(b))
            if !b.is_zero() =>
        {
            Some(ValueData::Number(NumberValue::from_f64(
                a.as_f64() % b.as_f64(),
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
        (BinaryOperator::Join, ValueData::List(items), ValueData::String(separator)) => {
            let parts = items.iter().map(|item| match item {
                ValueData::String(value) => Some(value.as_str()),
                _ => None,
            }).collect::<Option<Vec<_>>>()?;
            Some(ValueData::String(parts.join(separator)))
        }
        // Split returns a list; leave it to normal evaluation to preserve the
        // existing optimizer's scalar-literal conversion invariant.
        (operator @ (BinaryOperator::JsonFieldString | BinaryOperator::JsonFieldNumber
            | BinaryOperator::JsonFieldBoolean | BinaryOperator::JsonFieldStrings
            | BinaryOperator::JsonFieldNumbers | BinaryOperator::JsonFieldBooleans),
            ValueData::String(key), ValueData::String(source)) => {
            match extract_json_field(source, key, operator)? {
                ValueData::List(_) => None, // Lists keep their typed HIR for runtime evaluation.
                scalar => Some(scalar),
            }
        }
        (BinaryOperator::TextOccurrences, ValueData::String(needle), ValueData::String(text))
            if !needle.is_empty() => {
            Some(ValueData::Number(NumberValue::Int(
                text.matches(needle).count() as i64,
            )))
        }
        (BinaryOperator::TextRemoveAll, ValueData::String(needle), ValueData::String(text))
            if !needle.is_empty() => {
            Some(ValueData::String(text.replace(needle, "")))
        }
        (BinaryOperator::CharacterAt, ValueData::Number(index), ValueData::String(text)) => {
            text_character_at(index, text).map(ValueData::String)
        }
        (BinaryOperator::TextLastPosition, ValueData::String(needle), ValueData::String(text)) => {
            text_last_position(text, needle)
                .map(|index| ValueData::Number(NumberValue::Int(index)))
        }
        (BinaryOperator::TextPosition, ValueData::String(needle), ValueData::String(text)) => {
            Some(ValueData::Number(NumberValue::Int(
                text.find(needle).map(|offset| text[..offset].chars().count() as i64).unwrap_or(-1),
            )))
        }
        (BinaryOperator::Take | BinaryOperator::Drop, ValueData::Number(NumberValue::Int(count)), ValueData::String(text))
            if *count >= 0 => {
            let count = usize::try_from(*count).ok()?;
            let selected: String = if *operator == BinaryOperator::Take {
                text.chars().take(count).collect()
            } else {
                text.chars().skip(count).collect()
            };
            Some(ValueData::String(selected))
        }
        (BinaryOperator::JsonPointer, ValueData::String(pointer), ValueData::String(source)) => {
            extract_json_pointer_text(source, pointer).map(ValueData::String)
        }
        (BinaryOperator::JsonHasKey, ValueData::String(key), ValueData::String(source)) => {
            json_object_has_key(source, key).map(ValueData::Boolean)
        }
        (BinaryOperator::StartsWith, ValueData::String(text), ValueData::String(prefix)) => {
            Some(ValueData::Boolean(text.starts_with(prefix)))
        }
        (BinaryOperator::EndsWith, ValueData::String(text), ValueData::String(suffix)) => {
            Some(ValueData::Boolean(text.ends_with(suffix)))
        }
        (BinaryOperator::Contains, ValueData::String(text), ValueData::String(needle)) => {
            Some(ValueData::Boolean(text.contains(needle)))
        }
        (BinaryOperator::Contains, ValueData::List(items), needle) => {
            Some(ValueData::Boolean(items.contains(needle)))
        },
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
            if matches!(operator, BinaryOperator::Range) {
                // Large or fractional ranges can fail at evaluation time.
                return false;
            }
            if matches!(operator, BinaryOperator::Divide | BinaryOperator::Modulo) {
                return literal_data(right).is_some_and(
                    |value| matches!(value, ValueData::Number(number) if !number.is_zero()),
                );
            }
            true
        }
        HirExpr::DomainOperation { .. } => false,
        HirExpr::Reduce { .. } => false,
        HirExpr::ZipTransform { .. } => false,
        HirExpr::ListTransform { .. } => false,
        HirExpr::FunctionCall { .. } | HirExpr::Conditional { .. } | HirExpr::Repeat { .. } => false,
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
        HirExpr::Repeat { count, value, .. } => {
            collect_symbol_refs_expr(count, refs);
            collect_symbol_refs_expr(value, refs);
        }
        HirExpr::Conditional { condition, consequence, alternative, .. } => {
            collect_symbol_refs_expr(condition, refs);
            collect_symbol_refs_expr(consequence, refs);
            collect_symbol_refs_expr(alternative, refs);
        }
        HirExpr::FunctionCall { arguments, body, parameters, .. } => {
            for argument in arguments { collect_symbol_refs_expr(argument, refs); }
            let mut body_refs = HashSet::new();
            collect_symbol_refs_expr(body, &mut body_refs);
            for (symbol_id, _) in parameters { body_refs.remove(symbol_id); }
            refs.extend(body_refs);
        }
        HirExpr::Reduce { iterable, initial, body, iterator_symbol_id, accumulator_symbol_id, .. } => {
            collect_symbol_refs_expr(iterable, refs);
            collect_symbol_refs_expr(initial, refs);
            let mut body_refs = HashSet::new();
            collect_symbol_refs_expr(body, &mut body_refs);
            body_refs.remove(iterator_symbol_id);
            body_refs.remove(accumulator_symbol_id);
            refs.extend(body_refs);
        }
        HirExpr::ZipTransform {
            left_list, right_list, body, left_symbol_id, right_symbol_id, ..
        } => {
            collect_symbol_refs_expr(left_list, refs);
            collect_symbol_refs_expr(right_list, refs);
            let mut body_refs = HashSet::new();
            collect_symbol_refs_expr(body, &mut body_refs);
            body_refs.remove(left_symbol_id);
            body_refs.remove(right_symbol_id);
            refs.extend(body_refs);
        }
        HirExpr::ListTransform { iterable, body, .. } => {
            collect_symbol_refs_expr(iterable, refs);
            collect_symbol_refs_expr(body, refs);
        }
        HirExpr::Literal { .. } => {}
    }
}

