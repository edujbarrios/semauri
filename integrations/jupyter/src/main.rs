// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! Native Jupyter wire protocol for isolated, effect-scoped Semauri cells.
//! No Python runtime, ipykernel adapter, or shell interpolation is involved.
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DELIMITER: &[u8] = b"<IDS|MSG>";
const KEYWORDS: &str = "let be then do print assert range from to of number string boolean map filter count reduce every if when otherwise elif unless end within define function taking invoke import square sqrt round truncate factorial even odd true false cube cbrt reciprocal digits binary octal hex hexadecimal prime capitalize swapcase uncapitalize titlecase sum average variance stdev power difference between absolute squared root rms squares mae mse rmse mbe bias error exp ln log2 log10 sin cos asin acos atan sinh cosh tanh asinh acosh atanh log1p expm1 minimum maximum median sorted distinct flatten partition split join joined different floor ceiling length reverse take drop first last top bottom json parse len abs mean min max strip upper lower avg var stddev total size unique cardinality head final med prod rev sort flat ascending descending running cumulative adjacent successive differences empty blank zero positive negative nonzero nonnegative nonpositive least most above below one divisible multiple includes excludes does contain include begins finishes begin finish bound report cap reattempt memoize constrain reset restrict validate normalize against schema";
const DIRECTIVES: &[&str] = &["check", "explain", "effects", "plan", "tokens", "ast", "hir", "optimize", "symbols", "graph", "fmt", "audit"];
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize, Clone)]
struct Connection {
    transport: String,
    ip: String,
    shell_port: u16,
    iopub_port: u16,
    stdin_port: u16,
    control_port: u16,
    hb_port: u16,
    key: String,
    signature_scheme: String,
}
impl Connection {
    fn address(&self, port: u16) -> String {
        format!("{}://{}:{port}", self.transport, self.ip)
    }
    fn validate(&self) -> Result<(), String> {
        if self.signature_scheme != "hmac-sha256" { return Err("Only hmac-sha256 Jupyter signatures are supported".into()); }
        if self.transport != "tcp" { return Err("Only TCP Jupyter kernel connections are supported".into()); }
        Ok(())
    }
}

struct Request {
    identities: Vec<Vec<u8>>,
    header: Value,
    content: Value,
}
fn parse_request(frames: Vec<Vec<u8>>, key: &str) -> Result<Request, String> {
    let split = frames.iter().position(|part| part == DELIMITER).ok_or("Missing Jupyter message delimiter")?;
    if frames.len() < split + 6 { return Err("Truncated Jupyter message".into()); }
    let body = &frames[split + 1..];
    if !verify(key, &body[0], &body[1..5]) { return Err("Invalid Jupyter HMAC signature".into()); }
    Ok(Request {
        identities: frames[..split].to_vec(),
        header: serde_json::from_slice(&body[1]).map_err(|e| format!("Malformed message header: {e}"))?,
        content: serde_json::from_slice(&body[4]).map_err(|e| format!("Malformed message content: {e}"))?,
    })
}
fn signature(key: &str, payload: &[Vec<u8>]) -> Vec<u8> {
    if key.is_empty() { return Vec::new(); }
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC supports any key length");
    for part in payload { mac.update(part); }
    mac.finalize().into_bytes().iter().flat_map(|byte| format!("{byte:02x}").into_bytes()).collect()
}
fn verify(key: &str, got: &[u8], payload: &[Vec<u8>]) -> bool {
    if key.is_empty() { return got.is_empty(); }
    let expected = signature(key, payload);
    if expected.len() != got.len() { return false; }
    // Compare signatures without a byte-dependent early return.
    let mut difference = 0u8;
    for (a, b) in expected.iter().zip(got) { difference |= a ^ b; }
    difference == 0
}
fn message_id() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{:x}-{now:x}-{:x}", std::process::id(), NEXT_ID.fetch_add(1, Ordering::Relaxed))
}
fn header(request: &Request, msg_type: &str) -> Value {
    json!({
        "msg_id": message_id(),
        "msg_type": msg_type,
        "username": request.header.get("username").and_then(Value::as_str).unwrap_or("semauri"),
        "session": request.header.get("session").and_then(Value::as_str).unwrap_or(""),
        "date": Utc::now().to_rfc3339(),
        "version": "5.3"
    })
}
fn send(socket: &zmq::Socket, key: &str, request: &Request, msg_type: &str, content: Value, iopub: bool) -> Result<(), String> {
    let head = serde_json::to_vec(&header(request, msg_type)).map_err(|e| e.to_string())?;
    let parent = serde_json::to_vec(&request.header).map_err(|e| e.to_string())?;
    let metadata = b"{}".to_vec();
    let body = serde_json::to_vec(&content).map_err(|e| e.to_string())?;
    let payload = vec![head, parent, metadata, body];
    let mut frames = if iopub {
        vec![format!("kernel.{}.{}", request.header["session"].as_str().unwrap_or(""), msg_type).into_bytes()]
    } else { request.identities.clone() };
    frames.push(DELIMITER.to_vec());
    frames.push(signature(key, &payload));
    frames.extend(payload);
    socket.send_multipart(frames, 0).map_err(|e| e.to_string())
}
fn publish(iopub: &zmq::Socket, key: &str, request: &Request, typ: &str, content: Value) -> Result<(), String> {
    send(iopub, key, request, typ, content, true)
}
fn parse_cell(code: &str) -> Result<(String, bool, String), String> {
    let mut lines = code.split_inclusive('\n').peekable();
    while let Some(line) = lines.peek() {
        if line.trim().is_empty() { lines.next(); } else { break; }
    }
    let Some(first) = lines.next() else { return Ok(("run".into(), false, code.into())); };
    if !first.trim().starts_with("%%") { return Ok(("run".into(), false, code.into())); }
    let words: Vec<_> = first.split_whitespace().collect();
    let command = words[0].strip_prefix("%%").unwrap_or("");
    if !DIRECTIVES.contains(&command) { return Err(format!("Unknown Semauri cell directive: {}", words[0])); }
    let json_mode = words.len() == 2 && words[1] == "--json" && matches!(command, "check" | "explain");
    if words.len() != 1 && !json_mode { return Err("Unsupported Semauri cell directive arguments".into()); }
    let source = lines.collect::<String>();
    if source.trim().is_empty() { return Err("Semauri cell directive requires source code".into()); }
    Ok((command.into(), json_mode, source))
}
fn command_args(command: &str, json_mode: bool, path: &Path, effects: &str) -> Result<Vec<String>, String> {
    if command != "run" && !DIRECTIVES.contains(&command) { return Err("Unsupported Semauri command".into()); }
    if json_mode && !matches!(command, "check" | "explain") { return Err("JSON mode is not supported for this command".into()); }
    let mut args = vec![command.to_string()];
    if command == "run" {
        for effect in effects.split(',').map(str::trim).filter(|entry| !entry.is_empty()) {
            if !effect.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
                return Err("SEMAURI_ALLOW_EFFECTS contains an invalid effect name".into());
            }
            args.push("--allow".into());
            args.push(effect.into());
        }
    }
    if json_mode { args.push("--json".into()); }
    args.push(path.to_string_lossy().into_owned());
    Ok(args)
}
struct TempCell { source: PathBuf, stdout: PathBuf, stderr: PathBuf }
impl TempCell {
    fn new(text: &str) -> Result<Self, String> {
        // Keep temporary source in the notebook cwd to preserve project-local imports.
        let prefix = format!(".semauri-cell-{}", message_id());
        let source = PathBuf::from(format!("{prefix}.suri"));
        let stdout = PathBuf::from(format!("{prefix}.stdout"));
        let stderr = PathBuf::from(format!("{prefix}.stderr"));
        use std::io::Write;
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&source).map_err(|e| e.to_string())?;
        file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        Ok(Self { source, stdout, stderr })
    }
}
impl Drop for TempCell {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.source);
        let _ = fs::remove_file(&self.stdout);
        let _ = fs::remove_file(&self.stderr);
    }
}
fn execute_cell(code: &str) -> Result<(String, String), String> {
    let (command, json_mode, source) = parse_cell(code)?;
    let temp = TempCell::new(&source)?;
    let effects = env::var("SEMAURI_ALLOW_EFFECTS").unwrap_or_else(|_| "console_write".into());
    let args = command_args(&command, json_mode, &temp.source, &effects)?;
    let timeout = env::var("SEMAURI_CELL_TIMEOUT").unwrap_or_else(|_| "60".into()).parse::<f64>()
        .map_err(|_| "SEMAURI_CELL_TIMEOUT must be a finite positive number".to_string())?;
    if !timeout.is_finite() || timeout <= 0.0 { return Err("SEMAURI_CELL_TIMEOUT must be a finite positive number".into()); }
    let stdout = File::create(&temp.stdout).map_err(|e| e.to_string())?;
    let stderr = File::create(&temp.stderr).map_err(|e| e.to_string())?;
    let binary = env::var("SEMAURI_BINARY").unwrap_or_else(|_| "semauri".into());
    let mut child = Command::new(binary).args(&args)
        .stdin(Stdio::null()).stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr))
        .spawn().map_err(|e| format!("Cannot launch Semauri: {e}"))?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? { break status; }
        if started.elapsed().as_secs_f64() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("Semauri cell exceeded its timeout: {timeout} seconds"));
        }
        thread::sleep(Duration::from_millis(10));
    };
    let output = fs::read_to_string(&temp.stdout).map_err(|e| e.to_string())?;
    let error = fs::read_to_string(&temp.stderr).map_err(|e| e.to_string())?;
    if status.success() {
        Ok((output, error))
    } else {
        let diagnostic = if !error.trim().is_empty() { error.trim() } else if !output.trim().is_empty() { output.trim() } else { "Semauri process failed" };
        Err(diagnostic.to_string())
    }
}
fn diagnostic(message: &str, name: &str, count: u64) -> Value {
    json!({ "status":"error", "execution_count":count, "ename":name,
        "evalue":message, "traceback":message.lines().collect::<Vec<_>>() })
}
fn execute(request: &Request, iopub: &zmq::Socket, key: &str, count: &mut u64) -> Result<Value, String> {
    let code = request.content["code"].as_str().unwrap_or("");
    let silent = request.content["silent"].as_bool().unwrap_or(false);
    if !silent && request.content["store_history"].as_bool().unwrap_or(true) { *count += 1; }
    if !silent { publish(iopub, key, request, "execute_input", json!({"code": code,"execution_count": *count}))?; }
    if code.trim().is_empty() {
        return Ok(json!({"status":"ok","execution_count":*count,"payload":[],"user_expressions":{}}));
    }
    match execute_cell(code) {
        Ok((out, err)) => {
            if !silent {
                if !out.is_empty() { publish(iopub, key, request, "stream", json!({"name":"stdout","text":out}))?; }
                if !err.is_empty() { publish(iopub, key, request, "stream", json!({"name":"stderr","text":err}))?; }
            }
            Ok(json!({"status":"ok","execution_count":*count,"payload":[],"user_expressions":{}}))
        }
        Err(detail) => {
            let report = diagnostic(&detail, "SemauriError", *count);
            if !silent { publish(iopub, key, request, "error", json!({"ename":report["ename"],"evalue":report["evalue"],"traceback":report["traceback"]}))?; }
            Ok(report)
        }
    }
}
fn is_word(ch: char) -> bool { ch.is_ascii_alphanumeric() || ch == '_' }
fn preceding_word(code: &str, cursor: usize) -> (String, usize) {
    let before: String = code.chars().take(cursor).collect();
    let start_byte = before.trim_end_matches(is_word).len();
    let start = before[..start_byte].chars().count();
    (before[start_byte..].to_string(), start)
}
fn completion(code: &str, cursor: usize) -> Value {
    let (word, start) = preceding_word(code, cursor);
    if word.is_empty() { return json!({"matches":[],"cursor_start":cursor,"cursor_end":cursor,"metadata":{},"status":"ok"}); }
    let prefix = word.to_ascii_lowercase();
    let mut candidates: BTreeSet<String> = KEYWORDS.split_whitespace().map(str::to_string).collect();
    for line in code.lines() {
        let line = line.trim_start();
        if line.starts_with('#') || line.starts_with("//") || line.starts_with("%%") { continue; }
        let words: Vec<_> = line.split(|ch: char| !is_word(ch)).filter(|w| !w.is_empty()).collect();
        for pair in words.windows(3) {
            if pair[0].eq_ignore_ascii_case("let") && pair[2].eq_ignore_ascii_case("be") { candidates.insert(pair[1].into()); }
        }
        for triple in words.windows(3) {
            if triple[0].eq_ignore_ascii_case("define") && triple[1].eq_ignore_ascii_case("function") { candidates.insert(triple[2].into()); }
        }
    }
    let matches: Vec<_> = candidates.into_iter().filter(|name| name.to_ascii_lowercase().starts_with(&prefix)).collect();
    json!({"matches":matches,"cursor_start":start,"cursor_end":cursor,"metadata":{},"status":"ok"})
}
fn inspect(code: &str, cursor: usize) -> Value {
    let (word, _) = preceding_word(code, cursor);
    let help = match word.to_ascii_lowercase().as_str() {
        "let" => Some("let name be EXPRESSION. Create an immutable binding."),
        "print" => Some("print EXPRESSION. Requires the console_write permission."),
        "assert" => Some("assert CONDITION. Stop if the condition is false."),
        "import" => Some("import \"relative/path.suri\". Load a local module."),
        "define" => Some("define function NAME taking x as number to EXPRESSION."),
        "map" => Some("map every item in ITEMS to EXPRESSION. Transform a list."),
        "filter" => Some("filter every item in ITEMS where CONDITION. Filter a list."),
        "avg" => Some("avg of NUMBERS. Checked numeric average."),
        "unique" => Some("unique of LIST. Return distinct list items."),
        "%%check" => Some("%%check followed by Semauri source. Check without executing."),
        _ => None,
    };
    match help {
        Some(detail) => json!({"status":"ok","found":true,"data":{"text/plain":detail},"metadata":{}}),
        None => json!({"status":"ok","found":false,"data":{},"metadata":{}}),
    }
}
fn dispatch(request: &Request, iopub: &zmq::Socket, key: &str, counter: &mut u64, connection: &Connection) -> Result<(String, Value, bool), String> {
    let kind = request.header["msg_type"].as_str().unwrap_or("");
    let content = match kind {
        "kernel_info_request" => json!({
            "protocol_version":"5.3","implementation":"semauri","implementation_version":"0.11.0",
            "language_info":{"name":"semauri","version":"0.73.0","mimetype":"text/x-semauri",
                "file_extension":".suri","pygments_lexer":"text","codemirror_mode":"text"},
            "banner":"Semauri Jupyter kernel — standalone, capability-checked .suri cells",
            "status":"ok","help_links":[]
        }),
        "execute_request" => execute(request, iopub, key, counter)?,
        "complete_request" => completion(request.content["code"].as_str().unwrap_or(""),
            request.content["cursor_pos"].as_u64().unwrap_or(0) as usize),
        "inspect_request" => inspect(request.content["code"].as_str().unwrap_or(""),
            request.content["cursor_pos"].as_u64().unwrap_or(0) as usize),
        "is_complete_request" => json!({"status":"complete"}),
        "history_request" => json!({"status":"ok","history":[]}),
        "comm_info_request" => json!({"status":"ok","comms":{}}),
        "connect_request" => json!({"shell_port":connection.shell_port,"iopub_port":connection.iopub_port,
            "stdin_port":connection.stdin_port,"hb_port":connection.hb_port}),
        "shutdown_request" => json!({"status":"ok","restart":request.content["restart"].as_bool().unwrap_or(false)}),
        _ => json!({"status":"error","ename":"NotImplemented","evalue":format!("Unsupported message: {kind}"),"traceback":[]}),
    };
    let reply = kind.strip_suffix("_request").map(|n| format!("{n}_reply")).unwrap_or_else(|| "error".into());
    Ok((reply, content, kind == "shutdown_request"))
}
fn handle(request: Request, shell: &zmq::Socket, iopub: &zmq::Socket, key: &str, counter: &mut u64, connection: &Connection) -> Result<bool, String> {
    publish(iopub, key, &request, "status", json!({"execution_state":"busy"}))?;
    let (typ, content, shutdown) = dispatch(&request, iopub, key, counter, connection)?;
    send(shell, key, &request, &typ, content, false)?;
    publish(iopub, key, &request, "status", json!({"execution_state":"idle"}))?;
    Ok(shutdown)
}
fn run_kernel(path: &Path) -> Result<(), String> {
    let connection: Connection = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Malformed Jupyter connection file: {e}"))?;
    connection.validate()?;
    let context = zmq::Context::new();
    let shell = context.socket(zmq::ROUTER).map_err(|e| e.to_string())?;
    let control = context.socket(zmq::ROUTER).map_err(|e| e.to_string())?;
    let stdin = context.socket(zmq::ROUTER).map_err(|e| e.to_string())?;
    let iopub = context.socket(zmq::PUB).map_err(|e| e.to_string())?;
    let heartbeat = context.socket(zmq::REP).map_err(|e| e.to_string())?;
    for (socket, port) in [
        (&shell,connection.shell_port),(&control,connection.control_port),
        (&stdin,connection.stdin_port),(&iopub,connection.iopub_port),(&heartbeat,connection.hb_port)
    ] {
        socket.bind(&connection.address(port)).map_err(|e| e.to_string())?;
    }
    let mut counter = 0;
    loop {
        let mut poll = [
            shell.as_poll_item(zmq::POLLIN), control.as_poll_item(zmq::POLLIN),
            heartbeat.as_poll_item(zmq::POLLIN)
        ];
        zmq::poll(&mut poll, -1).map_err(|e| e.to_string())?;
        if poll[2].is_readable() {
            let frames = heartbeat.recv_multipart(0).map_err(|e| e.to_string())?;
            heartbeat.send_multipart(frames, 0).map_err(|e| e.to_string())?;
        }
        for (socket, ready) in [(&shell, poll[0].is_readable()), (&control, poll[1].is_readable())] {
            if ready {
                let frames = socket.recv_multipart(0).map_err(|e| e.to_string())?;
                match parse_request(frames, &connection.key) {
                    Ok(request) => {
                        if handle(request, socket, &iopub, &connection.key, &mut counter, &connection)? { return Ok(()); }
                    }
                    Err(message) => eprintln!("Ignored invalid Jupyter message: {message}"),
                }
            }
        }
    }
}
fn install(args: &[String]) -> Result<(), String> {
    let mut location: Option<PathBuf> = None;
    let mut mode = "user";
    for item in args {
        if item == "--user" { mode = "user"; }
        else if item == "--sys-prefix" { mode = "sys-prefix"; }
        else if let Some(prefix) = item.strip_prefix("--prefix=") { location = Some(PathBuf::from(prefix)); mode = "prefix"; }
        else { return Err(format!("Usage: semauri-jupyter install [--user|--sys-prefix|--prefix=PATH], got {item}")); }
    }
    let data = if let Some(prefix) = location {
        prefix.join("share").join("jupyter")
    } else if mode == "sys-prefix" {
        let prefix = env::var_os("VIRTUAL_ENV").or_else(|| env::var_os("CONDA_PREFIX"))
            .ok_or("--sys-prefix requires VIRTUAL_ENV or CONDA_PREFIX")?;
        PathBuf::from(prefix).join("share").join("jupyter")
    } else if let Some(override_dir) = env::var_os("JUPYTER_DATA_DIR") {
        PathBuf::from(override_dir)
    } else if cfg!(windows) {
        PathBuf::from(env::var_os("APPDATA").ok_or("APPDATA is not set")?).join("jupyter")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(env::var_os("HOME").ok_or("HOME is not set")?).join("Library/Jupyter")
    } else if let Some(xdg) = env::var_os("XDG_DATA_HOME") {
        PathBuf::from(xdg).join("jupyter")
    } else {
        PathBuf::from(env::var_os("HOME").ok_or("HOME is not set")?).join(".local/share/jupyter")
    };
    let target = data.join("kernels/semauri");
    fs::create_dir_all(&target).map_err(|e| format!("{}: {e}", target.display()))?;
    let binary = env::current_exe().map_err(|e| e.to_string())?;
    let kernel_spec = json!({
        "argv":[binary.to_string_lossy(),"-f","{connection_file}"],
        "display_name":"Semauri","language":"semauri","metadata":{"debugger":false}
    });
    fs::write(target.join("kernel.json"), format!("{}\n", serde_json::to_string_pretty(&kernel_spec).map_err(|e| e.to_string())?))
        .map_err(|e| e.to_string())?;
    println!("Installed Semauri kernel in {}", target.display());
    Ok(())
}
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let result = match args.as_slice() {
        [cmd, rest @ ..] if cmd == "install" => install(rest),
        [flag, path] if flag == "-f" || flag == "--connection-file" => run_kernel(Path::new(path)),
        [path] => run_kernel(Path::new(path)),
        _ => Err("Usage: semauri-jupyter install [--user|--sys-prefix|--prefix=PATH] OR semauri-jupyter -f CONNECTION_FILE".into()),
    };
    if let Err(error) = result { eprintln!("{error}"); std::process::exit(2); }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directives_are_read_only_and_validate_arguments() {
        assert_eq!(parse_cell("\n%%check --json\nassert true.").unwrap(),
            ("check".into(), true, "assert true.".into()));
        assert_eq!(parse_cell("print 1.").unwrap().0, "run");
        assert!(parse_cell("%%fmt --write\nprint 1.").is_err());
        assert!(parse_cell("%%unknown\nprint 1.").is_err());
        assert!(parse_cell("%%audit\n").is_err());
    }
    #[test]
    fn effects_are_allowlisted_as_names_and_never_injected_as_options() {
        let path = Path::new("cell.suri");
        assert_eq!(command_args("run", false, path, "console_write").unwrap(),
            ["run","--allow","console_write","cell.suri"]);
        assert!(command_args("run", false, path, "--help").is_err());
        assert_eq!(command_args("check", false, path, "filesystem_write").unwrap(),
            ["check","cell.suri"]);
    }
    #[test]
    fn signatures_reject_modified_messages() {
        let payload = vec![b"header".to_vec(),b"parent".to_vec(),b"{}".to_vec(),b"content".to_vec()];
        let sig = signature("secret", &payload);
        assert!(verify("secret", &sig, &payload));
        assert!(!verify("wrong", &sig, &payload));
        assert!(!verify("secret", &sig, &[b"changed".to_vec()]));
    }
    #[test]
    fn completions_include_locals_without_execution() {
        let source = "let score be 3.\nprint sc";
        let cursor = source.chars().count();
        let response = completion(source, cursor);
        assert!(response["matches"].as_array().unwrap().iter().any(|name| name == "score"));
        assert_eq!(response["cursor_start"], cursor - 2);
    }
}
