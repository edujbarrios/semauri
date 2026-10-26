// JSON decoding shares one strict implementation across evaluation and folding.
// Numbers are never coerced from strings, and unsupported JSON shapes fail.
// The bounded input and list limits avoid unbounded pure-expression expansion.
const MAX_JSON_TEXT_BYTES: usize = 1024 * 1024;
const MAX_JSON_LIST_ITEMS: usize = 10_000;

fn json_decode_type(operator: &UnaryOperator) -> Option<Type> {
    match operator {
        UnaryOperator::DecodeJsonString => Some(Type::String),
        UnaryOperator::DecodeJsonNumber => Some(Type::Number),
        UnaryOperator::DecodeJsonBoolean => Some(Type::Boolean),
        UnaryOperator::DecodeJsonStrings => Some(Type::List(Box::new(Type::String))),
        UnaryOperator::DecodeJsonNumbers => Some(Type::List(Box::new(Type::Number))),
        UnaryOperator::DecodeJsonBooleans => Some(Type::List(Box::new(Type::Boolean))),
        _ => None,
    }
}

fn json_number_to_data(number: &serde_json::Number) -> Option<ValueData> {
    if let Some(value) = number.as_i64() {
        return Some(ValueData::Number(NumberValue::Int(value)));
    }
    // A larger integer must not lose significant digits through f64 rounding.
    let repr = number.to_string();
    if !repr.contains('.') && !repr.contains('e') && !repr.contains('E') {
        return None;
    }
    let value = number.as_f64()?;
    if value.is_finite() {
        Some(ValueData::Number(NumberValue::Float(value)))
    } else {
        None
    }
}

fn json_scalar_to_data(value: &JsonValue, expected: &Type) -> Option<ValueData> {
    match (expected, value) {
        (Type::String, JsonValue::String(text)) => Some(ValueData::String(text.clone())),
        (Type::Boolean, JsonValue::Bool(boolean)) => Some(ValueData::Boolean(*boolean)),
        (Type::Number, JsonValue::Number(number)) => json_number_to_data(number),
        _ => None,
    }
}

fn decode_json_text(source: &str, operator: &UnaryOperator) -> Option<ValueData> {
    if source.len() > MAX_JSON_TEXT_BYTES {
        return None;
    }
    let expected = json_decode_type(operator)?;
    let parsed: JsonValue = serde_json::from_str(source).ok()?;
    match expected {
        Type::List(element) => {
            let items = parsed.as_array()?;
            if items.len() > MAX_JSON_LIST_ITEMS {
                return None;
            }
            items.iter().map(|item| json_scalar_to_data(item, &element))
                .collect::<Option<Vec<_>>>()
                .map(ValueData::List)
        }
        scalar => json_scalar_to_data(&parsed, &scalar),
    }
}
