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

