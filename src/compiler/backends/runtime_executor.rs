fn execute_runtime_plan(plan: &RuntimePlan, config: &crate::ProgramConfig) -> Result<String> {
    let mut values = BTreeMap::<String, ValueData>::new();
    let mut output = String::new();
    let mut ai = AiRuntimeState::default();
    for operation in &plan.operations {
        let argument = |name: &str| -> Result<ValueData> {
            match operation.arguments.get(name) {
                Some(RuntimeArgument::Value(value)) => Ok(value.clone()),
                Some(RuntimeArgument::Ref(reference)) => values.get(&reference.id).cloned().ok_or_else(|| {
                    runtime_error(operation, format!("runtime value '{}' is unavailable", reference.id))
                }),
                None => Err(runtime_error(operation, format!("missing argument '{name}'"))),
            }
        };
        let result = match (operation.domain.as_str(), operation.name.as_str()) {
            ("config", kind @ ("read_string" | "read_number" | "read_boolean")) => {
                let name = expect_string(argument("name")?, operation, "name")?;
                let value = config.resolve_binding(&name).map_err(|error| runtime_error(operation, error.to_string()))?;
                Some(json_config_value(value, kind, operation)?)
            }
            ("ai", "request_budget") => {
                ai.request_limit = Some(expect_nonnegative_integer(argument("count")?, operation, "count")?);
                None
            }
            ("ai", "retry_policy") => {
                ai.retries = expect_nonnegative_integer(argument("count")?, operation, "count")?;
                None
            }
            ("ai", "response_cache") => {
                ai.cache_enabled = true;
                None
            }
            ("ai", "language_model" | "vision_model") => Some(argument("identifier")?),
            ("ai", "generate_text") => {
                let model = expect_string(argument("model")?, operation, "model")?;
                let prompt = expect_string(argument("prompt")?, operation, "prompt")?;
                Some(ai.infer(&model, &prompt, operation)?)
            }
            ("ai", "inspect_image") => {
                let model = expect_string(argument("model")?, operation, "model")?;
                let image = expect_string(argument("image")?, operation, "image")?;
                let prompt = expect_string(argument("prompt")?, operation, "prompt")?;
                Some(ai.infer(&model, &format!("{prompt} [image: {image}]"), operation)?)
            }
            ("ai", "embed_text") => {
                let model = expect_string(argument("model")?, operation, "model")?;
                let text = expect_string(argument("text")?, operation, "text")?;
                ensure_local_model(&model, operation)?;
                let checksum = text.bytes().fold(0_u64, |sum, byte| sum.wrapping_mul(31).wrapping_add(u64::from(byte)));
                Some(ValueData::String(format!("local-embedding:{checksum:016x}")))
            }
            ("ai", "typed_tool") => Some(ValueData::String(expect_string(argument("name")?, operation, "name")?)),
            ("ai", "bounded_agent") => {
                let model = expect_string(argument("model")?, operation, "model")?;
                ensure_local_model(&model, operation)?;
                Some(ValueData::String(model))
            }
            ("ai", "run_agent") => {
                let model = expect_string(argument("agent")?, operation, "agent")?;
                let prompt = expect_string(argument("prompt")?, operation, "prompt")?;
                Some(ai.infer(&model, &prompt, operation)?)
            }
            ("console", "print") => {
                output.push_str(&display_console_value(&argument("value")?));
                output.push('\n');
                None
            }
            ("console", "assert") => match argument("condition")? {
                ValueData::Boolean(true) => None,
                ValueData::Boolean(false) => {
                    return Err(SemauriError::new(
                        ErrorKind::Runtime,
                        "S502",
                        "Assertion failed",
                        Some(operation.source_span),
                        Some("Inspect the condition or its input values.".to_string()),
                    ))
                }
                _ => return Err(runtime_error(operation, "assertion condition must be boolean")),
            },
            _ => return Err(runtime_error(operation, format!("unsupported runtime operation '{}.{}'", operation.domain, operation.name))),
        };
        if let (Some(reference), Some(value)) = (&operation.result, result) {
            values.insert(reference.id.clone(), value);
        }
    }
    Ok(output)
}

fn json_config_value(value: JsonValue, kind: &str, operation: &RuntimeOperation) -> Result<ValueData> {
    match (kind, value) {
        ("read_string", JsonValue::String(value)) => Ok(ValueData::String(value)),
        ("read_number", JsonValue::Number(value)) => value.as_i64().map(|v| ValueData::Number(NumberValue::Int(v)))
            .or_else(|| value.as_f64().map(|v| ValueData::Number(NumberValue::Float(v))))
            .ok_or_else(|| runtime_error(operation, "configured number is outside the supported range")),
        ("read_boolean", JsonValue::Bool(value)) => Ok(ValueData::Boolean(value)),
        (expected, _) => Err(runtime_error(operation, format!("configuration value has the wrong type for {expected}"))),
    }
}

fn expect_string(value: ValueData, operation: &RuntimeOperation, name: &str) -> Result<String> {
    match value {
        ValueData::String(value) => Ok(value),
        _ => Err(runtime_error(operation, format!("argument '{name}' must be a string"))),
    }
}

fn ensure_local_model(model: &str, operation: &RuntimeOperation) -> Result<()> {
    if model.starts_with("local/") { Ok(()) } else {
        Err(runtime_error(operation, format!("model '{model}' needs a provider adapter; this release executes local/* models only")))
    }
}

#[derive(Default)]
struct AiRuntimeState {
    request_limit: Option<u64>,
    requests: u64,
    retries: u64,
    cache_enabled: bool,
    cache: BTreeMap<String, ValueData>,
}

impl AiRuntimeState {
    fn infer(&mut self, model: &str, prompt: &str, operation: &RuntimeOperation) -> Result<ValueData> {
        ensure_local_model(model, operation)?;
        let key = format!("{model}\0{prompt}");
        if self.cache_enabled {
            if let Some(value) = self.cache.get(&key) {
                return Ok(value.clone());
            }
        }
        if self.request_limit.is_some_and(|limit| self.requests >= limit) {
            return Err(runtime_error(operation, "AI request budget exhausted"));
        }
        self.requests += 1;
        let _configured_retries = self.retries;
        let value = ValueData::String(format!("[{model}] {prompt}"));
        if self.cache_enabled {
            self.cache.insert(key, value.clone());
        }
        Ok(value)
    }
}

fn expect_nonnegative_integer(
    value: ValueData,
    operation: &RuntimeOperation,
    name: &str,
) -> Result<u64> {
    match value {
        ValueData::Number(NumberValue::Int(value)) if value >= 0 => Ok(value as u64),
        _ => Err(runtime_error(
            operation,
            format!("argument '{name}' must be a non-negative integer"),
        )),
    }
}

fn runtime_error(operation: &RuntimeOperation, message: impl Into<String>) -> SemauriError {
    SemauriError::new(ErrorKind::Runtime, "S503", message.into(), Some(operation.source_span), None)
}

fn display_console_value(value: &ValueData) -> String {
    match value {
        ValueData::String(value) | ValueData::Color(value) => value.clone(),
        ValueData::Number(value) => value.to_string(),
        ValueData::Boolean(value) => value.to_string(),
        ValueData::List(values) => format!(
            "[{}]",
            values
                .iter()
                .map(display_console_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ValueData::Unit => "unit".to_string(),
    }
}
