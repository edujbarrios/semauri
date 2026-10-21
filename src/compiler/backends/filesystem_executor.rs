// Validate a complete plan before executing its first filesystem effect.
// Never silently convert missing/invalid arguments into empty paths or values.
fn filesystem_argument(operation: &OperationIr, name: &str) -> std::io::Result<String> {
    match operation.arguments.get(name) {
        Some(ValueData::String(value) | ValueData::Color(value)) => Ok(value.clone()),
        Some(ValueData::Number(value)) => Ok(value.to_string()),
        Some(ValueData::Boolean(value)) => Ok(value.to_string()),
        Some(_) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported argument '{name}' for filesystem operation '{}'", operation.name),
        )),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Missing argument '{name}' for filesystem operation '{}'", operation.name),
        )),
    }
}

fn validate_filesystem_operation(operation: &OperationIr) -> std::io::Result<()> {
    let required: &[&str] = match operation.name.as_str() {
        "write" | "append" => &["path", "content"],
        "copy" | "move" => &["source", "destination"],
        "make_directory" | "touch" | "delete" => &["path"],
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unsupported filesystem operation '{other}'"),
            ));
        }
    };
    for name in required {
        filesystem_argument(operation, name)?;
    }
    Ok(())
}

fn execute_filesystem_plan(plan: &OperationPlan, cwd: Option<&Path>) -> std::io::Result<()> {
    let root = cwd.unwrap_or_else(|| Path::new("."));

    // An invalid later operation must not leave earlier partial writes behind.
    for operation in &plan.operations {
        validate_filesystem_operation(operation)?;
    }

    for operation in &plan.operations {
        let arg = |name: &str| filesystem_argument(operation, name);
        match operation.name.as_str() {
            "write" => fs::write(root.join(arg("path")?), arg("content")?)?,
            "append" => {
                use std::io::Write;
                let path = root.join(arg("path")?);
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                file.write_all(arg("content")?.as_bytes())?;
            }
            "copy" => {
                fs::copy(root.join(arg("source")?), root.join(arg("destination")?))?;
            }
            "move" => fs::rename(root.join(arg("source")?), root.join(arg("destination")?))?,
            "make_directory" => fs::create_dir_all(root.join(arg("path")?))?,
            "touch" => {
                let path = root.join(arg("path")?);
                let _ = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
            }
            "delete" => {
                let path = root.join(arg("path")?);
                match fs::symlink_metadata(&path) {
                    Ok(metadata) if metadata.file_type().is_dir() => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Refusing to recursively delete directory '{}'; Delete supports files only",
                                path.display()
                            ),
                        ));
                    }
                    Ok(_) => fs::remove_file(&path)?,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
            _ => unreachable!("validated in the preflight pass"),
        }
    }
    Ok(())
}
