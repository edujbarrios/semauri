// External libraries are explicit, locally installed source dependencies.
// They are compiled into the same program and never introduce implicit
// networking, plugin execution, or new runtime capabilities.
const MAX_LIBRARY_CATALOG_BYTES: u64 = 64 * 1024;
const MAX_EXTERNAL_LIBRARIES: usize = 32;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryCatalog {
    schema: u32,
    libraries: Vec<LibraryDependency>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryDependency {
    name: String,
    version: String,
    path: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryDescriptor {
    schema: u32,
    name: String,
    version: String,
    entry: String,
}

#[derive(Clone, Debug)]
struct ExternalLibrary {
    version: String,
    root: PathBuf,
    entry: PathBuf,
}

fn library_error(file: &Path, detail: impl AsRef<str>) -> SemauriError {
    SemauriError::semantic(
        "S417",
        format!("{}: {}", file.display(), detail.as_ref()),
        None,
        Some("Declare local external libraries in .semauri-libraries.json and check each library's .semauri-library.json (schema 1, name, version, entry).".into()),
    )
}

fn valid_library_name(name: &str) -> bool {
    name.len() <= 64 && name.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase())
        && name.bytes().all(|byte|
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-')
}

fn valid_library_version(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() == 3 && parts.iter().all(|part| {
        !part.is_empty()
            && (part.len() == 1 || !part.starts_with('0'))
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && part.parse::<u64>().is_ok()
    })
}

fn read_library_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata = fs::symlink_metadata(path).map_err(|error|
        library_error(path, format!("Cannot inspect library metadata: {error}")))?;
    if !metadata.file_type().is_file() {
        return Err(library_error(path, "Library metadata must be a regular file (not a symlink)"));
    }
    if metadata.len() > MAX_LIBRARY_CATALOG_BYTES {
        return Err(library_error(path, "Library metadata exceeds 64 KiB"));
    }
    let source = fs::read_to_string(path).map_err(|error|
        library_error(path, format!("Cannot read UTF-8 library metadata: {error}")))?;
    serde_json::from_str(&source)
        .map_err(|error| library_error(path, format!("Invalid library metadata JSON: {error}")))
}

// A project's catalog explicitly authorizes additional local filesystem
// trees. Relative catalog paths may point to vendored directories or to
// sibling project directories, but imports inside a library cannot escape
// that library's canonical root, including via symlinks.
fn load_external_libraries(root: &Path) -> Result<HashMap<String, ExternalLibrary>> {
    let catalog_path = root.join(".semauri-libraries.json");
    match fs::symlink_metadata(&catalog_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(HashMap::new())
        }
        Err(error) => return Err(library_error(&catalog_path,
            format!("Cannot inspect library catalog: {error}"))),
        Ok(_) => {}
    }
    let catalog: LibraryCatalog = read_library_json(&catalog_path)?;
    if catalog.schema != 1 {
        return Err(library_error(&catalog_path, "Unsupported library catalog schema (expected 1)"));
    }
    if catalog.libraries.len() > MAX_EXTERNAL_LIBRARIES {
        return Err(library_error(&catalog_path, "Library catalog exceeds 32 dependencies"));
    }
    let mut resolved = HashMap::new();
    for pin in catalog.libraries {
        if !valid_library_name(&pin.name) || !valid_library_version(&pin.version) {
            return Err(library_error(&catalog_path,
                format!("Invalid library name or MAJOR.MINOR.PATCH version: {}", pin.name)));
        }
        if pin.path.is_empty() || pin.path.len() > 1024
            || Path::new(&pin.path).is_absolute()
        {
            return Err(library_error(&catalog_path,
                format!("Library '{}' path must be a nonempty relative directory path", pin.name)));
        }
        if resolved.contains_key(&pin.name) {
            return Err(library_error(&catalog_path,
                format!("Duplicate external library '{}'", pin.name)));
        }
        let directory = fs::canonicalize(root.join(&pin.path)).map_err(|error|
            library_error(&catalog_path,
                format!("Cannot locate external library '{}': {error}", pin.name)))?;
        if !directory.is_dir() {
            return Err(library_error(&catalog_path,
                format!("Library '{}' path must point to a directory", pin.name)));
        }
        let descriptor_path = directory.join(".semauri-library.json");
        let descriptor: LibraryDescriptor = read_library_json(&descriptor_path)?;
        if descriptor.schema != 1 || descriptor.name != pin.name
            || descriptor.version != pin.version
            || !valid_library_name(&descriptor.name)
            || !valid_library_version(&descriptor.version)
        {
            return Err(library_error(&descriptor_path,
                format!("Library '{}' identity or version does not match project catalog", pin.name)));
        }
        if descriptor.entry.is_empty() || Path::new(&descriptor.entry).is_absolute()
            || Path::new(&descriptor.entry).extension().is_none_or(|ext| ext != "suri")
        {
            return Err(library_error(&descriptor_path,
                "Library entry must be a relative .suri source file"));
        }
        let entry = fs::canonicalize(directory.join(&descriptor.entry)).map_err(|error|
            library_error(&descriptor_path, format!("Cannot resolve library entry: {error}")))?;
        if !entry.is_file() || !entry.starts_with(&directory) {
            return Err(library_error(&descriptor_path,
                "Library entry escapes the declared library directory (possibly via symlink)"));
        }
        resolved.insert(pin.name, ExternalLibrary {
            version: pin.version,
            root: directory,
            entry,
        });
    }
    Ok(resolved)
}

// Local-only, compile-time imports. All imported declarations and effects enter
// the same checked program; imports do not grant runtime filesystem access.
const MAX_MODULE_FILES: usize = 64;
const MAX_MODULE_DEPTH: usize = 32;
const MAX_MODULE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug)]
struct NamespaceColumnEdit {
    generated_start: usize,
    generated_end: usize,
    original_start: usize,
    original_end: usize,
}

#[derive(Clone, Debug)]
struct LoadedSource {
    source: String,
    origins: Vec<(PathBuf, usize)>,
    files: HashMap<PathBuf, String>,
    column_edits: Vec<Vec<NamespaceColumnEdit>>,
}

impl LoadedSource {
    fn source_column(&self, line: usize, column: usize) -> usize {
        let Some(edits) = self.column_edits.get(line.saturating_sub(1)) else {
            return column;
        };
        let mut shift = 0_isize;
        for edit in edits {
            if column < edit.generated_start { break; }
            if column <= edit.generated_end {
                return edit.original_start + (column - edit.generated_start)
                    .min(edit.original_end - edit.original_start);
            }
            shift += (edit.generated_end - edit.generated_start) as isize
                - (edit.original_end - edit.original_start) as isize;
        }
        (column as isize - shift).max(1) as usize
    }
}

struct ModuleLoader {
    libraries: HashMap<String, ExternalLibrary>,
    visited: HashSet<(PathBuf, String)>,
    next_namespace: usize,
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


// Qualified imports remain ordinary Semauri source: the loader rewrites only
// declaration/call identifier tokens, never strings, comments or other code.
// The reserved generated prefix keeps scoped names separate from user symbols.
const PRIVATE_NAMESPACE_PREFIX: &str = "semauri_ns_";

fn valid_import_alias(alias: &str) -> bool {
    !alias.is_empty() && alias.len() <= 64
        && keyword_kind(alias).is_none() && !is_color(alias)
        && DomainRegistry::builtins().classify(alias).is_none()
        && alias.bytes().next().is_some_and(|ch| ch.is_ascii_lowercase())
        && alias.bytes().all(|ch| ch.is_ascii_lowercase()
            || ch.is_ascii_digit() || ch == b'_')
}

// Parse the complete, single-line import, including an optional "as ALIAS".
// Find the end of the JSON string without confusing escaped quotes with its
// closing quote. All other trailing tokens are rejected rather than ignored.
fn parse_module_import(raw: &str, file: &Path, line: usize)
    -> Result<(String, Option<String>)>
{
    let statement = raw.strip_suffix('.').ok_or_else(||
        module_error(file, line, "Expected import \"relative/file.suri\" [as alias]."))?.trim();
    if !statement.starts_with('"') {
        return Err(module_error(file, line, "Import path must be a quoted string"));
    }
    let mut escaped = false;
    let mut closing = None;
    for (index, byte) in statement.bytes().enumerate().skip(1) {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            closing = Some(index + 1);
            break;
        }
    }
    let closing = closing.ok_or_else(||
        module_error(file, line, "Unclosed import string"))?;
    let path: String = serde_json::from_str(&statement[..closing]).map_err(|_|
        module_error(file, line, "Import path must be a JSON quoted string"))?;
    let remainder = statement[closing..].trim();
    let alias = if remainder.is_empty() {
        None
    } else {
        let parts = remainder.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 2 || !parts[0].eq_ignore_ascii_case("as")
            || !valid_import_alias(&parts[1].to_ascii_lowercase())
        {
            return Err(module_error(file, line,
                "Expected 'as alias' with a lowercase ASCII identifier (letters, digits, underscore)"));
        }
        Some(parts[1].to_ascii_lowercase())
    };
    Ok((path, alias))
}

fn rewrite_scoped_source(
    source: &str, namespace: &str, aliases: &HashMap<String, String>, file: &Path,
) -> Result<(String, Vec<Vec<NamespaceColumnEdit>>)> {
    if namespace.is_empty() && aliases.is_empty()
        && !source.to_ascii_lowercase().contains(PRIVATE_NAMESPACE_PREFIX)
    {
        return Ok((source.to_owned(), vec![Vec::new(); source.lines().count()]));
    }
    let tokens = Lexer::new(source, &DomainRegistry::builtins()).tokens()
        .map_err(|error| module_error(file,
            error.span.map_or(1, |span| span.line()),
            format!("Cannot parse scoped import source: {}", error.message)))?;
    // Validate original tokens before skipping over function names while
    // rewriting definitions/calls: a user symbol must never impersonate an
    // internal namespace generated by this loader.
    for token in &tokens {
        if token.kind == TokenKind::Word
            && token.lexeme.to_ascii_lowercase().starts_with(PRIVATE_NAMESPACE_PREFIX)
        {
            return Err(module_error(file, token.span.line(),
                "The semauri_ns_ prefix is reserved for compiler-generated namespace symbols"));
        }
    }
    let lines = source.split('\n').collect::<Vec<_>>();
    let mut line_offsets = Vec::with_capacity(lines.len());
    let mut offset = 0;
    for line in &lines {
        line_offsets.push(offset);
        offset += line.len() + 1;
    }
    let position = |line: usize, column: usize| -> usize {
        let text = lines[line - 1];
        let local = text.char_indices().nth(column.saturating_sub(1))
            .map_or(text.len(), |(index, _)| index);
        line_offsets[line - 1] + local
    };
    let start = |token: &Token| position(token.span.line(), token.span.column());
    let end = |token: &Token| position(token.span.end_line(), token.span.end_column());
    let mut edits = Vec::<(usize, usize, String, usize, usize, usize)>::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind == TokenKind::Word
            && token.lexeme.to_ascii_lowercase().starts_with(PRIVATE_NAMESPACE_PREFIX)
        {
            return Err(module_error(file, token.span.line(),
                "The semauri_ns_ prefix is reserved for compiler-generated namespace symbols"));
        }
        // Preserve names of functions inside a namespaced import, including
        // nested local imports. Non-namespaced source remains unchanged.
        if !namespace.is_empty()
            && token.lexeme.eq_ignore_ascii_case("define")
            && tokens.get(index + 1).is_some_and(|next|
                next.lexeme.eq_ignore_ascii_case("function"))
            && tokens.get(index + 2).is_some_and(|next| next.kind == TokenKind::Word)
        {
            let name = &tokens[index + 2];
            edits.push((start(name), end(name),
                format!("{namespace}{}", name.lexeme.to_lowercase()),
                name.span.line(), name.span.column(), name.span.end_column()));
            index += 3;
            continue;
        }
        if token.kind == TokenKind::Word && token.lexeme.eq_ignore_ascii_case("invoke") {
            let next = tokens.get(index + 1);
            // invoke ALIAS.FUNCTION to VALUE.
            if let (Some(alias), Some(dot), Some(function), Some(to)) = (
                next, tokens.get(index + 2), tokens.get(index + 3), tokens.get(index + 4),
            ) {
                if alias.kind == TokenKind::Word && dot.kind == TokenKind::Dot
                    && function.kind == TokenKind::Word && to.kind == TokenKind::To
                {
                    let scope = aliases.get(&alias.lexeme.to_ascii_lowercase())
                        .ok_or_else(|| module_error(file, alias.span.line(),
                            format!("Unknown import namespace '{}'", alias.lexeme)))?;
                    if alias.span.line() != function.span.end_line() {
                        return Err(module_error(file, alias.span.line(),
                            "Keep qualified function names on a single line"));
                    }
                    edits.push((start(alias), end(function),
                        format!("{scope}{}", function.lexeme.to_lowercase()),
                        alias.span.line(), alias.span.column(), function.span.end_column()));
                    index += 5;
                    continue;
                }
            }
            if !namespace.is_empty() {
                if let (Some(name), Some(to)) = (next, tokens.get(index + 2)) {
                    if name.kind == TokenKind::Word && to.kind == TokenKind::To {
                        edits.push((start(name), end(name),
                            format!("{namespace}{}", name.lexeme.to_lowercase()),
                            name.span.line(), name.span.column(), name.span.end_column()));
                        index += 3;
                        continue;
                    }
                }
            }
        }
        index += 1;
    }
    if edits.is_empty() {
        return Ok((source.to_owned(), vec![Vec::new(); source.lines().count()]));
    }
    // Rebuild from original offsets; replacements may change character count,
    // but do not add/remove lines, preserving the original file/line map.
    let mut output = String::with_capacity(source.len() + edits.len() * 20);
    let mut columns = vec![Vec::new(); lines.len()];
    let mut shifts = vec![0_isize; lines.len()];
    let mut cursor = 0;
    for (begin, finish, replacement, line, original_start, original_end) in edits {
        let row = line - 1;
        let generated_start = (original_start as isize + shifts[row]) as usize;
        let generated_end = generated_start + replacement.chars().count();
        columns[row].push(NamespaceColumnEdit {
            generated_start, generated_end, original_start, original_end,
        });
        shifts[row] += replacement.chars().count() as isize
            - (original_end - original_start) as isize;
        output.push_str(&source[cursor..begin]);
        output.push_str(&replacement);
        cursor = finish;
    }
    output.push_str(&source[cursor..]);
    Ok((output, columns))
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
    let libraries = load_external_libraries(&root)?;
    let mut loader = ModuleLoader {
        libraries,
        visited: HashSet::new(),
        next_namespace: 0,
        visiting: Vec::new(),
        bytes: 0,
        loaded: LoadedSource {
            source: String::new(),
            origins: Vec::new(),
            files: HashMap::new(),
            column_edits: Vec::new(),
        },
    };
    loader.visit(&entry, 0, &root, "")?;
    Ok(loader.loaded)
}

impl ModuleLoader {
    fn visit(&mut self, file: &Path, depth: usize, boundary: &Path, namespace: &str) -> Result<()> {
        if self.visiting.iter().any(|item| item == file) {
            let mut chain = self.visiting.clone();
            chain.push(file.to_path_buf());
            let names = chain.iter().map(|path| path.display().to_string())
                .collect::<Vec<_>>().join(" -> ");
            return Err(module_error(file, 1, format!("Circular import: {names}")));
        }
        if self.visited.contains(&(file.to_path_buf(), namespace.to_owned())) { return Ok(()); }
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
        // Normalize a single leading UTF-8 BOM before scanning imports;
        // otherwise an import on line 1 looks like an executable statement.
        let content = content.strip_prefix('\u{feff}').unwrap_or(&content).to_owned();
        self.bytes += content.len();
        self.loaded.files.insert(file.to_path_buf(), content.clone());
        self.visited.insert((file.to_path_buf(), namespace.to_owned()));
        self.visiting.push(file.to_path_buf());

        let mut imports = Vec::new();
        let mut aliases = HashMap::new();
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
                let (relative, alias) = parse_module_import(raw, file, number)?;
                let alias_scope = if let Some(alias) = alias {
                    if aliases.contains_key(&alias) {
                        return Err(module_error(file, number,
                            format!("Duplicate import namespace '{alias}'")));
                    }
                    self.next_namespace += 1;
                    let scope = format!("{PRIVATE_NAMESPACE_PREFIX}{}__", self.next_namespace);
                    aliases.insert(alias, scope.clone());
                    Some(scope)
                } else {
                    None
                };
                if relative.is_empty() || Path::new(&relative).is_absolute() {
                    return Err(module_error(file, number, "Import path must be nonempty and relative"));
                }
                if let Some(name) = relative.strip_prefix("library:") {
                    if !valid_library_name(name) {
                        return Err(library_error(file,
                            format!("Invalid external library alias '{name}' at line {number}")));
                    }
                } else if Path::new(&relative).extension().is_none_or(|ext| ext != "suri") {
                    return Err(module_error(file, number,
                        "Only .suri files or library:NAME aliases can be imported"));
                }
                imports.push((relative, number, alias_scope));
            } else {
                if !trimmed.is_empty() && !trimmed.starts_with('#')
                    && !trimmed.starts_with("//") {
                    has_executable = true;
                }
                body.push((line, number));
            }
        }
        for (relative, number, alias_scope) in imports {
            let child_namespace = alias_scope.as_deref().unwrap_or(namespace);
            if let Some(name) = relative.strip_prefix("library:") {
                let external = self.libraries.get(name).ok_or_else(|| library_error(file,
                    format!("Undeclared external library '{name}' at line {number}")))?.clone();
                self.visit(&external.entry, depth + 1, &external.root, child_namespace)?;
                continue;
            }
            let base = file.parent().expect("canonical source has a parent");
            let destination = base.join(relative);
            let resolved = fs::canonicalize(&destination).map_err(|error|
                module_error(file, number,
                    format!("Cannot import {}: {error}", destination.display())))?;
            if !resolved.starts_with(boundary) {
                return Err(module_error(file, number,
                    "Import escapes its project/library root (including through symlinks)"));
            }
            if !resolved.is_file() {
                return Err(module_error(file, number, "Import target must be a file"));
            }
            self.visit(&resolved, depth + 1, boundary, child_namespace)?;
        }
        let (transformed, column_edits) =
            rewrite_scoped_source(&content, namespace, &aliases, file)?;
        let transformed_lines = transformed.lines().collect::<Vec<_>>();
        for (_, original_line) in body {
            self.loaded.source.push_str(transformed_lines.get(original_line - 1)
                .copied().unwrap_or(""));
            self.loaded.source.push('\n');
            self.loaded.origins.push((file.to_path_buf(), original_line));
            self.loaded.column_edits.push(
                column_edits.get(original_line - 1).cloned().unwrap_or_default());
        }
        self.visiting.pop();
        Ok(())
    }
}
