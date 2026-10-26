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

// Preserve integer exactness before serde_json can convert an out-of-range
// negative integer into floating point notation. Skip quoted JSON strings,
// including escaped quotes, and let serde_json validate the actual syntax.
fn json_integer_literals_fit_i64(source: &str) -> bool {
    let bytes = source.as_bytes();
    let (mut index, mut in_string, mut escaped) = (0, false, false);
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }
        if byte == b'-' || byte.is_ascii_digit() {
            let start = index;
            while index < bytes.len()
                && !matches!(bytes[index], b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}' | b':')
            {
                index += 1;
            }
            let token = &source[start..index];
            if !token.contains('.') && !token.contains('e') && !token.contains('E')
                && token.parse::<i64>().is_err()
            {
                return false;
            }
            continue;
        }
        index += 1;
    }
    true
}

fn parse_json_object_text(source: &str) -> Option<serde_json::Map<String, JsonValue>> {
    if source.len() > MAX_JSON_TEXT_BYTES {
        return None;
    }
    let JsonValue::Object(object) = serde_json::from_str::<JsonValue>(source).ok()? else {
        return None;
    };
    if object.len() > MAX_JSON_LIST_ITEMS {
        return None;
    }
    Some(object)
}

fn json_object_keys(source: &str) -> Option<Vec<String>> {
    let object = parse_json_object_text(source)?;
    let mut keys = object.keys().cloned().collect::<Vec<_>>();
    keys.sort_unstable(); // Deterministic even with an insertion-ordered JSON backend.
    Some(keys)
}

fn json_object_has_key(source: &str, key: &str) -> Option<bool> {
    Some(parse_json_object_text(source)?.contains_key(key))
}

fn decode_json_text(source: &str, operator: &UnaryOperator) -> Option<ValueData> {
    if source.len() > MAX_JSON_TEXT_BYTES || !json_integer_literals_fit_i64(source) {
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
