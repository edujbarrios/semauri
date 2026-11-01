// Local-only, compile-time imports. All imported declarations and effects enter
// the same checked program; imports do not grant runtime filesystem access.
const MAX_MODULE_FILES: usize = 64;
const MAX_MODULE_DEPTH: usize = 32;
const MAX_MODULE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug)]
struct LoadedSource {
    source: String,
    origins: Vec<(PathBuf, usize)>,
    files: HashMap<PathBuf, String>,
}

struct ModuleLoader {
    root: PathBuf,
    visited: HashSet<PathBuf>,
    visiting: Vec<PathBuf>,
    bytes: usize,
    loaded: LoadedSource,
}

fn module_error(file: &Path, line: usize, detail: impl AsRef<str>) -> SemauriError {
    SemauriError::semantic("S416",
        format!("{}:{line}: {}", file.display(), detail.as_ref()), None,
        Some("Place import \"relative/file.suri\". before the program's statements, inside the project root.".into()))
}

fn import_line(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let prefix = trimmed.get(..6)?;
    if !prefix.eq_ignore_ascii_case("import") { return None; }
    match trimmed.as_bytes().get(6) {
        Some(byte) if byte.is_ascii_whitespace() => Some(trimmed[6..].trim()),
        None => Some(""),
        _ => None,
    }
}

// The nearest real .semauri-root file defines the project boundary.
// Without one, a standalone script is confined to its own directory.
fn module_project_root(entry: &Path) -> PathBuf {
    let original = entry.parent().expect("canonical entry has a parent");
    let mut directory = original;
    loop {
        if let Ok(metadata) = fs::symlink_metadata(directory.join(".semauri-root")) {
            if metadata.file_type().is_file() {
                return directory.to_path_buf();
            }
        }
        match directory.parent() {
            Some(parent) => directory = parent,
            None => return original.to_path_buf(),
        }
    }
}

fn load_module_source(entry: &Path) -> Result<LoadedSource> {
    let entry = fs::canonicalize(entry).map_err(|error|
        usage_error(format!("Cannot open {}: {error}", entry.display())))?;
    if entry.extension().is_none_or(|ext| ext != "suri") {
        return Err(module_error(&entry, 1, "Source files must end in .suri"));
    }
    let root = module_project_root(&entry);
    let mut loader = ModuleLoader {
        root,
        visited: HashSet::new(),
        visiting: Vec::new(),
        bytes: 0,
        loaded: LoadedSource {
            source: String::new(),
            origins: Vec::new(),
            files: HashMap::new(),
        },
    };
    loader.visit(&entry, 0)?;
    Ok(loader.loaded)
}

impl ModuleLoader {
    fn visit(&mut self, file: &Path, depth: usize) -> Result<()> {
        if self.visiting.iter().any(|item| item == file) {
            let mut chain = self.visiting.clone();
            chain.push(file.to_path_buf());
            let names = chain.iter().map(|path| path.display().to_string())
                .collect::<Vec<_>>().join(" -> ");
            return Err(module_error(file, 1, format!("Circular import: {names}")));
        }
        if self.visited.contains(file) { return Ok(()); }
        if depth > MAX_MODULE_DEPTH || self.visited.len() >= MAX_MODULE_FILES {
            return Err(module_error(file, 1, "Import exceeds 32 levels or 64 files"));
        }
        let size = fs::metadata(file).map_err(|error|
            module_error(file, 1, format!("Cannot inspect module: {error}")))?.len();
        if size > MAX_MODULE_BYTES as u64
            || self.bytes.saturating_add(size as usize) > MAX_MODULE_BYTES {
            return Err(module_error(file, 1, "Source graph exceeds 4 MiB"));
        }
        let content = fs::read_to_string(file).map_err(|error|
            module_error(file, 1, format!("Cannot read UTF-8 module: {error}")))?;
        self.bytes += content.len();
        self.loaded.files.insert(file.to_path_buf(), content.clone());
        self.visited.insert(file.to_path_buf());
        self.visiting.push(file.to_path_buf());

        let mut imports = Vec::new();
        let mut body = Vec::new();
        let mut has_executable = false;
        let mut in_comment_block = false;
        for (index, line) in content.lines().enumerate() {
            let number = index + 1;
            let trimmed = line.trim();
            if in_comment_block {
                if trimmed.contains("*/") { in_comment_block = false; }
                body.push((line, number));
                continue;
            }
            if trimmed.starts_with("/*") {
                if !trimmed.contains("*/") { in_comment_block = true; }
                body.push((line, number));
                continue;
            }
            if let Some(raw) = import_line(line) {
                if has_executable {
                    return Err(module_error(file, number, "Import must precede executable statements"));
                }
                let quoted = raw.strip_suffix('.')
                    .ok_or_else(|| module_error(file, number,
                        "Expected import \"relative/file.suri\"."))?.trim();
                let relative: String = serde_json::from_str(quoted)
                    .map_err(|_| module_error(file, number,
                        "Import path must be a quoted string followed by a period"))?;
                if relative.is_empty() || Path::new(&relative).is_absolute() {
                    return Err(module_error(file, number, "Import path must be nonempty and relative"));
                }
                if Path::new(&relative).extension().is_none_or(|ext| ext != "suri") {
                    return Err(module_error(file, number, "Only .suri files can be imported"));
                }
                imports.push((relative, number));
            } else {
                if !trimmed.is_empty() && !trimmed.starts_with('#')
                    && !trimmed.starts_with("//") {
                    has_executable = true;
                }
                body.push((line, number));
            }
        }
        for (relative, number) in imports {
            let base = file.parent().expect("canonical source has a parent");
            let destination = base.join(relative);
            let resolved = fs::canonicalize(&destination).map_err(|error|
                module_error(file, number,
                    format!("Cannot import {}: {error}", destination.display())))?;
            if !resolved.starts_with(&self.root) {
                return Err(module_error(file, number,
                    "Import escapes project root (including through symlinks)"));
            }
            if !resolved.is_file() {
                return Err(module_error(file, number, "Import target must be a file"));
            }
            self.visit(&resolved, depth + 1)?;
        }
        for (line, original_line) in body {
            self.loaded.source.push_str(line);
            self.loaded.source.push('\n');
            self.loaded.origins.push((file.to_path_buf(), original_line));
        }
        self.visiting.pop();
        Ok(())
    }
}
