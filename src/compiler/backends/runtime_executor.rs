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
            ("random", "password" | "secure_string") => {
                let count = match argument("count")? {
                    ValueData::Number(NumberValue::Int(count))
                        if (1..=MAX_SECURE_RANDOM_CHARACTERS as i64).contains(&count) =>
                            count as usize,
                    _ => return Err(runtime_error(
                        operation, "random string length must be an integer from 1 to 4096",
                    )),
                };
                let alphabet = if operation.name == "password" {
                    DEFAULT_PASSWORD_ALPHABET.to_string()
                } else {
                    expect_string(argument("alphabet")?, operation, "alphabet")?
                };
                let characters = checked_random_alphabet(&alphabet).ok_or_else(|| runtime_error(
                    operation,
                    "random alphabet must contain 1 to 4096 distinct Unicode characters",
                ))?;
                let mut rng = OsSecureRandom::new().map_err(|_| runtime_error(
                    operation, "operating-system secure random source is unavailable",
                ))?;
                let generated = sample_secure_string(&mut rng, count, &characters)
                    .map_err(|_| runtime_error(operation,
                        "operating-system secure random source failed"))?;
                Some(ValueData::String(generated))
            }
            ("random", "character") => {
                let alphabet = expect_string(argument("alphabet")?, operation, "alphabet")?;
                let characters = checked_random_alphabet(&alphabet).ok_or_else(|| runtime_error(
                    operation,
                    "random alphabet must contain 1 to 4096 distinct Unicode characters",
                ))?;
                let mut rng = OsSecureRandom::new().map_err(|_| runtime_error(
                    operation, "operating-system secure random source is unavailable",
                ))?;
                let generated = sample_secure_string(&mut rng, 1, &characters)
                    .map_err(|_| runtime_error(operation,
                        "operating-system secure random source failed"))?;
                Some(ValueData::String(generated))
            }
            ("random", "integer") => {
                let minimum = match argument("minimum")? {
                    ValueData::Number(NumberValue::Int(value)) => value,
                    _ => return Err(runtime_error(
                        operation, "random integer minimum must be a signed 64-bit integer",
                    )),
                };
                let maximum = match argument("maximum")? {
                    ValueData::Number(NumberValue::Int(value)) => value,
                    _ => return Err(runtime_error(
                        operation, "random integer maximum must be a signed 64-bit integer",
                    )),
                };
                if minimum > maximum {
                    return Err(runtime_error(
                        operation, "random integer minimum must not exceed maximum",
                    ));
                }
                let mut rng = OsSecureRandom::new().map_err(|_| runtime_error(
                    operation, "operating-system secure random source is unavailable",
                ))?;
                let generated = sample_secure_integer(&mut rng, minimum, maximum)
                    .map_err(|_| runtime_error(operation,
                        "operating-system secure random source failed"))?;
                Some(ValueData::Number(NumberValue::Int(generated)))
            }
            ("config", kind @ ("read_string" | "read_number" | "read_boolean")) => {
                let name = expect_string(argument("name")?, operation, "name")?;
                let value = config.resolve_binding(&name).map_err(|error| runtime_error(operation, error.to_string()))?;
                Some(json_config_value(value, kind, operation)?)
            }
            ("ai", "request_budget") => {
                ai.request_limit = Some(expect_nonnegative_integer(argument("count")?, operation, "count")?);
                None
            }
            ("ai", "prompt_limit") => {
                ai.prompt_limit = Some(expect_nonnegative_integer(argument("count")?, operation, "count")?);
                None
            }
            ("ai", "cache_capacity") => {
                ai.cache_limit = Some(expect_nonnegative_integer(argument("count")?, operation, "count")?);
                ai.trim_cache();
                None
            }
            ("ai", "clear_cache") => {
                ai.cache.clear();
                ai.cache_order.clear();
                None
            }
            ("ai", "output_limit") => {
                ai.output_limit = Some(expect_nonnegative_integer(argument("count")?, operation, "count")?);
                None
            }
            ("ai", "usage_report") => Some(ValueData::String(json!({
                "requests": ai.requests,
                "request_limit": ai.request_limit,
                "prompt_character_limit": ai.prompt_limit,
                "cache_hits": ai.cache_hits,
                "cache_entries": ai.cache.len(),
                "cache_capacity": ai.cache_limit,
                "cache_evictions": ai.cache_evictions,
                "output_character_limit": ai.output_limit,
                "agent_count": ai.agents.len(),
                "agent_turns": ai.agent_turns,
            }).to_string())),
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
                ai.check_prompt_length(&text, operation)?;
                ai.account_request(operation)?;
                let checksum = text.bytes().fold(0_u64, |sum, byte| sum.wrapping_mul(31).wrapping_add(u64::from(byte)));
                Some(ValueData::String(format!("local-embedding:{checksum:016x}")))
            }
            ("ai", "validate_json_response") => {
                let document = expect_string(argument("document")?, operation, "document")?;
                let schema = expect_string(argument("schema")?, operation, "schema")?;
                let matches = validate_ai_json_response(&document, &schema)
                    .map_err(|message| runtime_error(operation, message))?;
                Some(ValueData::Boolean(matches))
            }
            ("ai", "normalize_json_response") => {
                let document = expect_string(argument("document")?, operation, "document")?;
                let schema = expect_string(argument("schema")?, operation, "schema")?;
                let canonical = normalize_ai_json_response(&document, &schema)
                    .map_err(|message| runtime_error(operation, message))?;
                Some(ValueData::String(canonical))
            }
            ("ai", "typed_tool") => Some(ValueData::String(expect_string(argument("name")?, operation, "name")?)),
            ("ai", "bounded_agent") => {
                let model = expect_string(argument("model")?, operation, "model")?;
                let system = expect_string(argument("system")?, operation, "system")?;
                let max_turns = expect_nonnegative_integer(argument("max_turns")?, operation, "max_turns")?;
                ensure_local_model(&model, operation)?;
                let handle = format!("semauri-agent:{}", ai.agents.len() + 1);
                ai.agents.insert(handle.clone(), AgentRuntime {
                    model, system, max_turns, turns: 0,
                });
                Some(ValueData::String(handle))
            }
            ("ai", "run_agent") => {
                let handle = expect_string(argument("agent")?, operation, "agent")?;
                let prompt = expect_string(argument("prompt")?, operation, "prompt")?;
                let agent = ai.agents.get(&handle).cloned()
                    .ok_or_else(|| runtime_error(operation, "unknown AI agent handle"))?;
                if agent.turns >= agent.max_turns {
                    return Err(runtime_error(operation, "AI agent turn limit exhausted"));
                }
                let answer = ai.infer_with_context(&agent.model, &prompt, &agent.system, operation)?;
                if let Some(agent) = ai.agents.get_mut(&handle) {
                    agent.turns += 1;
                }
                ai.agent_turns += 1;
                Some(answer)
            }
            // Pure built-in boolean expressions evaluated after reading runtime
            // configuration or other dynamic values. These never grant effects.
            ("core", "not") => match argument("value")? {
                ValueData::Boolean(value) => Some(ValueData::Boolean(!value)),
                _ => return Err(runtime_error(operation, "runtime 'not' requires a boolean")),
            },
            ("core", "xor") => match (argument("left")?, argument("right")?) {
                (ValueData::Boolean(a), ValueData::Boolean(b)) =>
                    Some(ValueData::Boolean(a ^ b)),
                _ => return Err(runtime_error(operation, "runtime 'xor' requires booleans")),
            },
            ("core", "equal" | "not_equal") => {
                let same = argument("left")? == argument("right")?;
                Some(ValueData::Boolean(if operation.name == "equal" { same } else { !same }))
            }
            ("core", "negate") => match argument("value")? {
                ValueData::Number(NumberValue::Int(value)) => {
                    let negated = value.checked_neg().ok_or_else(|| SemauriError::new(
                        ErrorKind::Runtime, "S358",
                        "Negating this integer exceeds the signed 64-bit range",
                        Some(operation.source_span), None,
                    ))?;
                    Some(ValueData::Number(NumberValue::Int(negated)))
                }
                ValueData::Number(NumberValue::Float(value)) if value.is_finite() =>
                    Some(ValueData::Number(NumberValue::Float(-value))),
                _ => return Err(runtime_error(operation, "runtime 'minus' requires a finite number")),
            },
            ("core", "subtract") => match (argument("left")?, argument("right")?) {
                (ValueData::Number(a), ValueData::Number(b)) => {
                    let difference = numeric_subtract(&a, &b).ok_or_else(|| SemauriError::new(
                        ErrorKind::Runtime, "S470",
                        "Subtraction overflows the signed integer range or produces a non-finite result",
                        Some(operation.source_span), None,
                    ))?;
                    Some(ValueData::Number(difference))
                }
                _ => return Err(runtime_error(operation, "runtime subtraction requires numbers")),
            },
            ("core", kind @ ("greater_than" | "less_than"
                | "greater_than_or_equal" | "less_than_or_equal")) => {
                let (ValueData::Number(a), ValueData::Number(b)) =
                    (argument("left")?, argument("right")?)
                else {
                    return Err(runtime_error(operation, "runtime numeric comparison requires numbers"));
                };
                let ordering = numeric_compare(&a, &b);
                let yes = match kind {
                    "greater_than" => ordering == Some(std::cmp::Ordering::Greater),
                    "less_than" => ordering == Some(std::cmp::Ordering::Less),
                    "greater_than_or_equal" => matches!(ordering,
                        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
                    _ => matches!(ordering,
                        Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
                };
                Some(ValueData::Boolean(yes))
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

#[derive(Clone)]
struct AgentRuntime {
    model: String,
    // Preserved as a separate context for provider adapters and cache isolation.
    // The local/test backend is a deterministic simulator, not an instruction follower.
    system: String,
    max_turns: u64,
    turns: u64,
}

#[derive(Default)]
struct AiRuntimeState {
    request_limit: Option<u64>,
    prompt_limit: Option<u64>,
    output_limit: Option<u64>,
    cache_limit: Option<u64>,
    requests: u64,
    cache_hits: u64,
    cache_evictions: u64,
    retries: u64,
    cache_enabled: bool,
    cache: BTreeMap<String, ValueData>,
    // FIFO insertion order; a cache hit does not change eviction priority.
    cache_order: VecDeque<String>,
    agents: BTreeMap<String, AgentRuntime>,
    agent_turns: u64,
}

impl AiRuntimeState {
    fn check_prompt_length(&self, prompt: &str, operation: &RuntimeOperation) -> Result<()> {
        self.check_prompt_with_context(prompt, "", operation)
    }

    fn check_prompt_with_context(
        &self, prompt: &str, context: &str, operation: &RuntimeOperation,
    ) -> Result<()> {
        let length = (prompt.chars().count() as u64).saturating_add(context.chars().count() as u64);
        if let Some(limit) = self.prompt_limit {
            if length > limit {
                return Err(runtime_error(
                    operation,
                    format!("AI prompt exceeds character limit: got {length} characters, maximum {limit}"),
                ));
            }
        }
        Ok(())
    }

    fn check_output_length(&self, output: &ValueData, operation: &RuntimeOperation) -> Result<()> {
        if let (Some(limit), ValueData::String(text)) = (self.output_limit, output) {
            let length = text.chars().count() as u64;
            if length > limit {
                return Err(runtime_error(
                    operation,
                    format!("AI output exceeds character limit: got {length} characters, maximum {limit}"),
                ));
            }
        }
        Ok(())
    }

    fn account_request(&mut self, operation: &RuntimeOperation) -> Result<()> {
        if self.request_limit.is_some_and(|limit| self.requests >= limit) {
            return Err(runtime_error(operation, "AI request budget exhausted"));
        }
        self.requests += 1;
        Ok(())
    }

    fn trim_cache(&mut self) {
        if let Some(limit) = self.cache_limit {
            while (self.cache.len() as u64) > limit {
                let Some(oldest) = self.cache_order.pop_front() else { break };
                if self.cache.remove(&oldest).is_some() {
                    self.cache_evictions += 1;
                }
            }
        }
    }

    fn infer(&mut self, model: &str, prompt: &str, operation: &RuntimeOperation) -> Result<ValueData> {
        self.infer_with_context(model, prompt, "", operation)
    }

    fn infer_with_context(
        &mut self, model: &str, prompt: &str, context: &str, operation: &RuntimeOperation,
    ) -> Result<ValueData> {
        ensure_local_model(model, operation)?;
        self.check_prompt_with_context(prompt, context, operation)?;
        // System context scopes the cache without leaking it in the local simulator's output.
        let key = format!("{model}\0{context}\0{prompt}");
        if self.cache_enabled {
            if let Some(value) = self.cache.get(&key) {
                self.check_output_length(value, operation)?;
                self.cache_hits += 1;
                return Ok(value.clone());
            }
        }
        self.account_request(operation)?;
        let _configured_retries = self.retries;
        let value = ValueData::String(format!("[{model}] {prompt}"));
        // A rejected response is neither returned nor cached; the attempt consumed a request.
        self.check_output_length(&value, operation)?;
        if self.cache_enabled && self.cache_limit != Some(0) {
            self.cache.insert(key.clone(), value.clone());
            self.cache_order.push_back(key);
            self.trim_cache();
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
        ValueData::Record(fields) => ValueData::Record(fields.clone()).to_json().to_string(),
        ValueData::Dictionary(entries) =>
            ValueData::Dictionary(entries.clone()).to_json().to_string(),
        ValueData::Unit => "unit".to_string(),
    }
}
