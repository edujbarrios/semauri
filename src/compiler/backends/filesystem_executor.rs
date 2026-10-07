fn execute_filesystem_plan(plan: &OperationPlan, cwd: Option<&Path>) -> std::io::Result<()> {
    let root = cwd.unwrap_or_else(|| Path::new("."));
    for operation in &plan.operations {
        let arg = |name: &str| -> String {
            match operation.arguments.get(name) {
                Some(ValueData::String(value) | ValueData::Color(value)) => value.clone(),
                Some(ValueData::Number(value)) => value.to_string(),
                Some(ValueData::Boolean(value)) => value.to_string(),
                _ => String::new(),
            }
        };
        match operation.name.as_str() {
            "write" => fs::write(root.join(arg("path")), arg("content"))?,
            "append" => {
                use std::io::Write;
                let path = root.join(arg("path"));
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                file.write_all(arg("content").as_bytes())?;
            }
            "copy" => {
                fs::copy(root.join(arg("source")), root.join(arg("destination")))?;
            }
            "move" => fs::rename(root.join(arg("source")), root.join(arg("destination")))?,
            "make_directory" => fs::create_dir_all(root.join(arg("path")))?,
            "touch" => {
                let path = root.join(arg("path"));
                let _ = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
            }
            "delete" => {
                let path = root.join(arg("path"));
                if path.exists() {
                    if path.is_dir() {
                        fs::remove_dir_all(path)?;
                    } else {
                        fs::remove_file(path)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

