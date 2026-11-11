// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
//! End-to-end ZeroMQ protocol smoke test; runs the native kernel and compiler.
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

struct KernelProcess(Child, PathBuf);
impl Drop for KernelProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
        let _ = fs::remove_file(&self.1);
    }
}
fn available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}
fn signed(key: &str, parts: &[Vec<u8>]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).unwrap();
    for part in parts { mac.update(part); }
    mac.finalize().into_bytes().iter().flat_map(|b| format!("{b:02x}").into_bytes()).collect()
}
fn request(socket: &zmq::Socket, key: &str, typ: &str, msg_id: &str, content: Value) {
    let header = json!({"msg_type":typ,"msg_id":msg_id,"session":"native-test",
        "username":"test","version":"5.3"});
    let parts = [header, json!({}), json!({}), content]
        .iter().map(|v| serde_json::to_vec(v).unwrap()).collect::<Vec<_>>();
    let mut frames = vec![b"<IDS|MSG>".to_vec(), signed(key, &parts)];
    frames.extend(parts);
    socket.send_multipart(frames, 0).unwrap();
}
fn received(socket: &zmq::Socket) -> (Value, Value) {
    let frames = socket.recv_multipart(0).expect("Jupyter message timed out");
    let i = frames.iter().position(|f| f == b"<IDS|MSG>").unwrap();
    (serde_json::from_slice(&frames[i + 2]).unwrap(),
     serde_json::from_slice(&frames[i + 5]).unwrap())
}
#[test]
fn native_kernel_handles_heartbeat_execution_and_shutdown() {
    let cargo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let compiler = cargo_root.join("../../target/debug").join(if cfg!(windows) { "semauri.exe" } else { "semauri" });
    if !compiler.is_file() {
        eprintln!("Skipping protocol smoke test because the core Semauri executable is not built");
        return;
    }
    let ports = (0..5).map(|_| available_port()).collect::<Vec<_>>();
    let key = "test-secret";
    let config = json!({
        "transport":"tcp", "ip":"127.0.0.1", "key":key,
        "signature_scheme":"hmac-sha256",
        "shell_port":ports[0], "iopub_port":ports[1], "stdin_port":ports[2],
        "control_port":ports[3], "hb_port":ports[4]
    });
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let file = std::env::temp_dir().join(format!("semauri-jupyter-{}-{nonce}.json",std::process::id()));
    fs::write(&file, serde_json::to_vec(&config).unwrap()).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_semauri-jupyter"))
        .arg("-f").arg(&file).env("SEMAURI_BINARY",compiler)
        .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap();
    let mut kernel = KernelProcess(child, file);
    let context = zmq::Context::new();
    let shell = context.socket(zmq::DEALER).unwrap();
    let iopub = context.socket(zmq::SUB).unwrap();
    let hb = context.socket(zmq::REQ).unwrap();
    shell.set_rcvtimeo(15000).unwrap();
    iopub.set_rcvtimeo(15000).unwrap();
    hb.set_rcvtimeo(15000).unwrap();
    iopub.set_subscribe(b"").unwrap();
    shell.connect(&format!("tcp://127.0.0.1:{}",ports[0])).unwrap();
    iopub.connect(&format!("tcp://127.0.0.1:{}",ports[1])).unwrap();
    hb.connect(&format!("tcp://127.0.0.1:{}",ports[4])).unwrap();
    thread::sleep(Duration::from_millis(400));
    hb.send(b"semauri-ping" as &[u8], 0).unwrap();
    assert_eq!(hb.recv_bytes(0).unwrap(), b"semauri-ping");

    request(&shell,key,"kernel_info_request","info",json!({}));
    let (info_header, info) = received(&shell);
    assert_eq!(info_header["msg_type"],"kernel_info_reply");
    assert_eq!(info["language_info"]["name"],"semauri");

    request(&shell,key,"execute_request","execute",json!({
        "code":"print \"Jupyter\".","silent":false,"store_history":true,
        "user_expressions":{},"allow_stdin":false,"stop_on_error":true
    }));
    let (_, reply) = received(&shell);
    assert_eq!(reply["status"],"ok","{reply}");
    let mut streams = String::new();
    for _ in 0..8 {
        let (head, content) = received(&iopub);
        if head["msg_type"] == "stream" {
            streams.push_str(content["text"].as_str().unwrap_or(""));
        }
        if head["msg_type"] == "status" && content["execution_state"] == "idle" { break; }
    }
    assert_eq!(streams, "Jupyter\n");

    request(&shell,key,"shutdown_request","shutdown",json!({"restart":false}));
    let (head, reply) = received(&shell);
    assert_eq!(head["msg_type"],"shutdown_reply");
    assert_eq!(reply["status"],"ok");
    let _ = kernel.0.wait();
}
