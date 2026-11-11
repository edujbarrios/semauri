// Checked, bounded validation of JSON output shapes for AI workflows.
// This is a strict subset, not an implementation of the JSON Schema specification.
// Reject duplicate JSON object keys, including in nested data and schemas.
// serde_json::Value alone silently keeps the last occurrence of a key.
struct UniqueAiJson(JsonValue);

impl<'de> serde::Deserialize<'de> for UniqueAiJson {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueAiJsonVisitor)
    }
}

struct UniqueAiJsonVisitor;

impl<'de> serde::de::Visitor<'de> for UniqueAiJsonVisitor {
    type Value = UniqueAiJson;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(UniqueAiJson(JsonValue::Null))
    }

    fn visit_bool<E: serde::de::Error>(self, value: bool) -> std::result::Result<Self::Value, E> {
        Ok(UniqueAiJson(JsonValue::Bool(value)))
    }

    fn visit_i64<E: serde::de::Error>(self, value: i64) -> std::result::Result<Self::Value, E> {
        Ok(UniqueAiJson(json!(value)))
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> std::result::Result<Self::Value, E> {
        Ok(UniqueAiJson(json!(value)))
    }

    fn visit_f64<E: serde::de::Error>(self, value: f64) -> std::result::Result<Self::Value, E> {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("non-finite JSON number"))?;
        Ok(UniqueAiJson(JsonValue::Number(number)))
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
        Ok(UniqueAiJson(JsonValue::String(value.to_owned())))
    }

    fn visit_string<E: serde::de::Error>(self, value: String) -> std::result::Result<Self::Value, E> {
        Ok(UniqueAiJson(JsonValue::String(value)))
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self, mut sequence: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueAiJson>()? {
            values.push(value.0);
        }
        Ok(UniqueAiJson(JsonValue::Array(values)))
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self, mut map: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut values = JsonMap::new();
        while let Some((key, value)) = map.next_entry::<String, UniqueAiJson>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom("duplicate JSON object key"));
            }
            values.insert(key, value.0);
        }
        Ok(UniqueAiJson(JsonValue::Object(values)))
    }
}

fn parse_unique_ai_json(value: &str) -> std::result::Result<JsonValue, serde_json::Error> {
    serde_json::from_str::<UniqueAiJson>(value).map(|value| value.0)
}

const AI_JSON_DOCUMENT_MAX_BYTES: usize = 1024 * 1024;
const AI_JSON_SCHEMA_MAX_BYTES: usize = 64 * 1024;
const AI_JSON_SCHEMA_MAX_DEPTH: usize = 16;
const AI_JSON_SCHEMA_MAX_NODES: usize = 4096;
const AI_JSON_SCHEMA_MAX_FIELDS: usize = 128;
const AI_JSON_SIZE_CONSTRAINT_MAX: u64 = 1024 * 1024;

fn validate_ai_json_response(
    document: &str,
    schema_text: &str,
) -> std::result::Result<bool, &'static str> {
    if schema_text.len() > AI_JSON_SCHEMA_MAX_BYTES {
        return Err("AI JSON schema exceeds the 64 KiB limit");
    }
    let schema: JsonValue = parse_unique_ai_json(schema_text)
        .map_err(|_| "AI JSON schema is not valid JSON")?;
    let mut nodes = 0usize;
    check_ai_json_schema(&schema, 1, &mut nodes)?;
    if document.len() > AI_JSON_DOCUMENT_MAX_BYTES {
        return Ok(false);
    }
    let Ok(value) = parse_unique_ai_json(document) else {
        return Ok(false);
    };
    Ok(matches_ai_json_schema(&value, &schema))
}

fn normalize_ai_json_response(
    document: &str,
    schema: &str,
) -> std::result::Result<String, &'static str> {
    if !validate_ai_json_response(document, schema)? {
        return Err("AI JSON response does not match its declared schema");
    }
    // The first validation already bounded and parsed the input. JSON objects
    // use the stable serde_json map ordering and no fields are synthesized.
    let value: JsonValue = parse_unique_ai_json(document)
        .map_err(|_| "AI JSON response is not valid JSON")?;
    Ok(value.to_string())
}

fn ai_json_length_constraint(
    schema: &JsonMap<String, JsonValue>,
    name: &str,
) -> std::result::Result<Option<u64>, &'static str> {
    match schema.get(name) {
        None => Ok(None),
        Some(value) => match value.as_u64() {
            Some(n) if n <= AI_JSON_SIZE_CONSTRAINT_MAX => Ok(Some(n)),
            _ => Err("AI JSON schema length constraints must be nonnegative bounded integers"),
        },
    }
}

fn check_ai_json_schema(
    schema: &JsonValue,
    depth: usize,
    nodes: &mut usize,
) -> std::result::Result<(), &'static str> {
    *nodes += 1;
    if depth > AI_JSON_SCHEMA_MAX_DEPTH || *nodes > AI_JSON_SCHEMA_MAX_NODES {
        return Err("AI JSON schema exceeds nesting or node limits");
    }
    let object = schema.as_object().ok_or("AI JSON schema nodes must be objects")?;
    let kind = object.get("type")
        .and_then(JsonValue::as_str)
        .ok_or("AI JSON schema must declare a string type")?;
    if !matches!(kind, "object" | "array" | "string" | "number" | "integer" | "boolean" | "null") {
        return Err("AI JSON schema uses an unsupported type");
    }

    for name in object.keys() {
        let supported = match name.as_str() {
            "type" => true,
            "required" | "properties" | "additionalProperties" => kind == "object",
            "items" | "minItems" | "maxItems" => kind == "array",
            "minLength" | "maxLength" | "enum" => kind == "string",
            _ => false,
        };
        if !supported {
            return Err("AI JSON schema contains an unsupported keyword for its type");
        }
    }

    if kind == "object" {
        let properties = object.get("properties").map(|v| v.as_object()
            .ok_or("AI JSON schema properties must be an object")).transpose()?;
        if let Some(properties) = properties {
            if properties.len() > AI_JSON_SCHEMA_MAX_FIELDS {
                return Err("AI JSON schema has too many properties");
            }
            for nested in properties.values() {
                check_ai_json_schema(nested, depth + 1, nodes)?;
            }
        }
        if let Some(required) = object.get("required") {
            let fields = required.as_array()
                .ok_or("AI JSON schema required must be a string array")?;
            if fields.len() > AI_JSON_SCHEMA_MAX_FIELDS {
                return Err("AI JSON schema has too many required fields");
            }
            let mut names = BTreeSet::new();
            for field in fields {
                let key = field.as_str()
                    .ok_or("AI JSON schema required must contain only strings")?;
                if !names.insert(key) {
                    return Err("AI JSON schema required fields must be unique");
                }
            }
        }
        if object.get("additionalProperties").is_some_and(|v| !v.is_boolean()) {
            return Err("AI JSON schema additionalProperties must be boolean");
        }
    } else if kind == "array" {
        if let Some(items) = object.get("items") {
            check_ai_json_schema(items, depth + 1, nodes)?;
        }
        let min = ai_json_length_constraint(object, "minItems")?;
        let max = ai_json_length_constraint(object, "maxItems")?;
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err("AI JSON schema minimum cannot exceed maximum");
        }
    } else if kind == "string" {
        if let Some(choices) = object.get("enum") {
            let variants = choices.as_array()
                .ok_or("AI JSON schema string enum must be an array")?;
            if variants.is_empty() || variants.len() > AI_JSON_SCHEMA_MAX_FIELDS {
                return Err("AI JSON schema string enum must have 1 to 128 values");
            }
            let mut distinct = BTreeSet::new();
            for variant in variants {
                let text = variant.as_str()
                    .ok_or("AI JSON schema string enum must contain only strings")?;
                if !distinct.insert(text) {
                    return Err("AI JSON schema string enum contains duplicate values");
                }
            }
        }
        let min = ai_json_length_constraint(object, "minLength")?;
        let max = ai_json_length_constraint(object, "maxLength")?;
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err("AI JSON schema minimum cannot exceed maximum");
        }
    }
    Ok(())
}

fn matches_ai_json_schema(value: &JsonValue, schema: &JsonValue) -> bool {
    let object = schema.as_object().expect("validated JSON schema object");
    match object["type"].as_str().expect("validated JSON schema type") {
        "object" => {
            let Some(record) = value.as_object() else { return false };
            let properties = object.get("properties").and_then(JsonValue::as_object);
            if let Some(required) = object.get("required").and_then(JsonValue::as_array) {
                if required.iter().any(|key| !record.contains_key(
                    key.as_str().expect("validated required name")
                )) {
                    return false;
                }
            }
            if object.get("additionalProperties").and_then(JsonValue::as_bool) == Some(false)
                && record.keys().any(|key| !properties.is_some_and(|p| p.contains_key(key))) {
                return false;
            }
            if let Some(properties) = properties {
                if properties.iter().any(|(name, expected)|
                    record.get(name).is_some_and(|found| !matches_ai_json_schema(found, expected))) {
                    return false;
                }
            }
            true
        }
        "array" => {
            let Some(values) = value.as_array() else { return false };
            if !ai_json_size_matches(object, "minItems", "maxItems", values.len() as u64) {
                return false;
            }
            match object.get("items") {
                Some(items) => values.iter().all(|item| matches_ai_json_schema(item, items)),
                None => true,
            }
        }
        "string" => {
            let Some(text) = value.as_str() else { return false };
            if !ai_json_size_matches(object, "minLength", "maxLength", text.chars().count() as u64) {
                return false;
            }
            object.get("enum").is_none_or(|values| values.as_array()
                .expect("validated enum").iter().any(|v| v.as_str() == Some(text)))
        }
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some()
            || value.as_f64().is_some_and(|n| n.is_finite() && n.fract() == 0.0),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => unreachable!("schema kind was validated"),
    }
}

fn ai_json_size_matches(
    schema: &JsonMap<String, JsonValue>,
    min_name: &str,
    max_name: &str,
    length: u64,
) -> bool {
    if schema.get(min_name).is_some_and(|n| length < n.as_u64().unwrap()) {
        return false;
    }
    if schema.get(max_name).is_some_and(|n| length > n.as_u64().unwrap()) {
        return false;
    }
    true
}
