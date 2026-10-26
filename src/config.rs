// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct ConfigError {
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(formatter, "config line {line}: {}", self.message),
            None => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProgramConfig {
    parameters: BTreeMap<String, Value>,
    environment: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedConfig {
    values: BTreeMap<String, Value>,
    environment_sources: BTreeMap<String, String>,
}

impl ProgramConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let source = fs::read_to_string(path.as_ref()).map_err(|error| ConfigError {
            line: None,
            message: format!("cannot read '{}': {error}", path.as_ref().display()),
        })?;
        Self::parse(&source)
    }

    pub fn parse(source: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        for (index, raw_line) in source.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let statement = line.strip_suffix('.').ok_or_else(|| ConfigError {
                line: Some(line_number),
                message: "statements must end with a period".to_string(),
            })?;
            let (kind, rest) = statement.split_once(' ').ok_or_else(|| ConfigError {
                line: Some(line_number),
                message: "expected 'parameter NAME = VALUE' or 'environment NAME = \"VARIABLE\"'"
                    .to_string(),
            })?;
            let (name, raw_value) = rest.split_once('=').ok_or_else(|| ConfigError {
                line: Some(line_number),
                message: "expected '=' after the binding name".to_string(),
            })?;
            let name = name.trim();
            validate_name(name, line_number)?;
            if config.parameters.contains_key(name) || config.environment.contains_key(name) {
                return Err(ConfigError {
                    line: Some(line_number),
                    message: format!("duplicate binding '{name}'"),
                });
            }
            let value = raw_value.trim();
            match kind {
                "parameter" => {
                    let parsed = serde_json::from_str(value).map_err(|_| ConfigError {
                        line: Some(line_number),
                        message: "parameter values must be JSON strings, numbers, booleans, lists, objects, or null"
                            .to_string(),
                    })?;
                    config.parameters.insert(name.to_string(), parsed);
                }
                "environment" => {
                    let variable: String =
                        serde_json::from_str(value).map_err(|_| ConfigError {
                            line: Some(line_number),
                            message: "environment values must be quoted variable names".to_string(),
                        })?;
                    if variable.is_empty() {
                        return Err(ConfigError {
                            line: Some(line_number),
                            message: "environment variable name cannot be empty".to_string(),
                        });
                    }
                    config.environment.insert(name.to_string(), variable);
                }
                _ => {
                    return Err(ConfigError {
                        line: Some(line_number),
                        message: format!("unknown binding kind '{kind}'"),
                    })
                }
            }
        }
        Ok(config)
    }

    pub fn set_parameter(&mut self, name: &str, value: Value) -> Result<(), ConfigError> {
        if !self.parameters.contains_key(name) {
            return Err(ConfigError {
                line: None,
                message: format!("cannot override undeclared parameter '{name}'"),
            });
        }
        self.parameters.insert(name.to_string(), value);
        Ok(())
    }

    pub fn resolve(&self) -> Result<ResolvedConfig, ConfigError> {
        self.resolve_with(|name| std::env::var(name).ok())
    }

    pub fn resolve_with<F>(&self, mut read_environment: F) -> Result<ResolvedConfig, ConfigError>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut values = self.parameters.clone();
        for (binding, variable) in &self.environment {
            let value = read_environment(variable).ok_or_else(|| ConfigError {
                line: None,
                message: format!("required environment variable '{variable}' is not set"),
            })?;
            values.insert(binding.clone(), Value::String(value));
        }
        Ok(ResolvedConfig {
            values,
            environment_sources: self.environment.clone(),
        })
    }

    pub fn summary_json<F>(&self, mut is_set: F) -> Value
    where
        F: FnMut(&str) -> bool,
    {
        json!({
            "parameters": self.parameters,
            "environment": self.environment.iter().map(|(binding, variable)| json!({
                "binding": binding,
                "variable": variable,
                "set": is_set(variable),
            })).collect::<Vec<_>>(),
        })
    }
}

impl ResolvedConfig {
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn is_environment_binding(&self, name: &str) -> bool {
        self.environment_sources.contains_key(name)
    }
}

fn validate_name(name: &str, line: usize) -> Result<(), ConfigError> {
    let mut chars = name.chars();
    let valid = chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_');
    if valid {
        Ok(())
    } else {
        Err(ConfigError {
            line: Some(line),
            message: format!("invalid binding name '{name}'"),
        })
    }
}
