use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command as StdCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

struct DaemonChild {
    child: Child,
}

impl Drop for DaemonChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn ash_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("ash")
}

fn write_child_failure_workflow(root: &Path) {
    fs::write(
        root.join("main.ash"),
        r#"
use proc::bind;
use proc::join;
use proc::par;
use proc::then;
use proc::unit;

workflow main {
    ret bind(
        par(unit(1), then(unit(0), (fn(_) { fail "child proc boom" }))),
        (fn(handles) { join(handles.0, handles.1) })
    )
}
"#,
    )
    .expect("write child failure workflow");
}

fn spawn_daemon(root: &Path, dirs: &DaemonDirs) -> DaemonChild {
    let child = StdCommand::new(ash_bin())
        .arg("daemon")
        .arg("serve")
        .arg("--root")
        .arg(root)
        .arg("--socket")
        .arg(&dirs.socket)
        .arg("--state-dir")
        .arg(&dirs.state)
        .arg("--cache-dir")
        .arg(&dirs.cache)
        .arg("--log-dir")
        .arg(&dirs.log)
        .arg("--format")
        .arg("json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn daemon");

    wait_for_socket(&dirs.socket);
    DaemonChild { child }
}

fn wait_for_socket(socket: &Path) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if socket.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("daemon socket did not become ready: {}", socket.display());
}

struct DaemonDirs {
    socket: std::path::PathBuf,
    base: std::path::PathBuf,
    state: std::path::PathBuf,
    cache: std::path::PathBuf,
    log: std::path::PathBuf,
}

impl Drop for DaemonDirs {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn daemon_dirs() -> DaemonDirs {
    let base = std::path::PathBuf::from("target")
        .join("a940")
        .join(std::process::id().to_string());
    let socket_parent = base.join("s");
    let state = base.join("st");
    let cache = base.join("c");
    let log = base.join("l");
    for dir in [&socket_parent, &state, &cache, &log] {
        fs::create_dir_all(dir).expect("create daemon test dir");
    }
    let socket = socket_parent.join("d.sock");
    DaemonDirs {
        socket,
        base,
        state,
        cache,
        log,
    }
}

fn daemon_json(socket: &Path, args: &[&str]) -> Value {
    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
        .arg("daemon")
        .args(args)
        .arg("--socket")
        .arg(socket)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("daemon json response")
}

#[cfg(unix)]
fn daemon_protocol_json(socket: &Path, request: Value) -> Value {
    let mut stream = UnixStream::connect(socket).expect("connect daemon socket");
    serde_json::to_writer(&mut stream, &request).expect("write daemon request");
    stream.write_all(b"\n").expect("terminate daemon request");

    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .expect("read daemon response");
    serde_json::from_str(&line).expect("daemon protocol json response")
}

#[cfg(unix)]
#[test]
fn daemon_child_proc_failure_is_instance_failure_not_host_failure() {
    let root = tempdir().expect("root tempdir");
    write_child_failure_workflow(root.path());
    let dirs = daemon_dirs();
    let _daemon = spawn_daemon(root.path(), &dirs);

    let start = daemon_protocol_json(
        &dirs.socket,
        serde_json::json!({
            "command": "start",
            "workflow": "main",
            "execute": true
        }),
    );

    assert_eq!(start["ok"], true, "start response: {start}");
    assert_eq!(start["host_mode"], "Daemon");
    assert_eq!(start["status"], "failed", "start response: {start}");
    assert_eq!(
        start["class"], "workflow_child_failure",
        "start response: {start}"
    );
    assert_eq!(start["report"]["status"], "failed");
    assert_eq!(start["report"]["failure"]["tower"], "Proc");
    assert_eq!(start["report"]["failure"]["kind"], "child_proc_failure");
    assert_eq!(
        start["report"]["failure"]["host_failure"].as_bool(),
        Some(false),
        "child Proc failure must not be classified as daemon host failure: {start}"
    );
    let failure_message = start["report"]["failure"]["message"]
        .as_str()
        .expect("failure message");
    assert!(
        failure_message.contains("child proc boom"),
        "failure report must preserve child cause: {start}"
    );
    assert!(
        !failure_message.to_ascii_lowercase().contains("daemon host"),
        "failure report must not blame the daemon host: {start}"
    );
    let instance_id = start["instance_id"].as_str().expect("instance id");

    let status = daemon_json(&dirs.socket, &["status", "--instance", instance_id]);
    assert_eq!(status["ok"], true, "status response: {status}");
    assert_eq!(status["host_mode"], "Daemon");
    assert_eq!(status["status"], "failed", "status response: {status}");
    assert_eq!(status["class"], "workflow_child_failure");
    assert_eq!(status["report"]["failure"]["tower"], "Proc");
    assert_eq!(status["report"]["failure"]["kind"], "child_proc_failure");
    assert_eq!(status["report"]["failure"]["host_failure"], false);

    let list = daemon_json(&dirs.socket, &["list"]);
    assert_eq!(list["ok"], true, "list response: {list}");
    assert_eq!(list["host_mode"], "Daemon");
    let instances = list["instances"].as_array().expect("instances");
    assert_eq!(instances.len(), 1, "instances: {list}");
    assert_eq!(instances[0]["instance_id"], instance_id);
    assert_eq!(instances[0]["status"], "failed");
    assert_eq!(instances[0]["class"], "workflow_child_failure");
}
