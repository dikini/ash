use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::{Child, Command as StdCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::{TempDir, tempdir};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

struct DaemonChild {
    child: Child,
}

impl Drop for DaemonChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct DaemonDirs {
    socket: std::path::PathBuf,
    _socket_parent: TempDir,
    state: TempDir,
    cache: TempDir,
    log: TempDir,
}

fn daemon_dirs() -> DaemonDirs {
    let socket_parent = tempdir().expect("socket tempdir");
    let socket = socket_parent.path().join("ashd.sock");
    let state = tempdir().expect("state tempdir");
    let cache = tempdir().expect("cache tempdir");
    let log = tempdir().expect("log tempdir");
    #[cfg(unix)]
    {
        set_dir_mode(socket_parent.path(), 0o700);
        set_dir_mode(state.path(), 0o700);
        set_dir_mode(cache.path(), 0o700);
        set_dir_mode(log.path(), 0o700);
    }
    DaemonDirs {
        socket,
        _socket_parent: socket_parent,
        state,
        cache,
        log,
    }
}

#[cfg(unix)]
fn unix_socket_bind_is_permitted(dirs: &DaemonDirs) -> bool {
    let probe = dirs
        .socket
        .parent()
        .expect("daemon socket has a parent")
        .join("ashd-preflight.sock");
    match UnixListener::bind(&probe) {
        Ok(listener) => {
            drop(listener);
            fs::remove_file(&probe).unwrap_or_else(|error| {
                panic!("remove Unix socket preflight {}: {error}", probe.display())
            });
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!(
                "skipping daemon artifact-equivalence test: Unix socket bind is not permitted in this environment ({error})"
            );
            false
        }
        Err(error) => panic!("bind Unix socket preflight {}: {error}", probe.display()),
    }
}

#[cfg(unix)]
fn set_dir_mode(path: &Path, mode: u32) {
    let mut permissions = fs::metadata(path)
        .unwrap_or_else(|error| panic!("metadata for {}: {error}", path.display()))
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)
        .unwrap_or_else(|error| panic!("chmod {mode:o} {}: {error}", path.display()));
}

fn ash_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("ash")
}

fn wait_for_socket(mut child: Child, socket: &Path) -> Child {
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if socket.exists() {
            return child;
        }
        if child
            .try_wait()
            .expect("poll daemon while waiting for socket")
            .is_some()
        {
            daemon_startup_failure(child, socket);
        }
        thread::sleep(Duration::from_millis(50));
    }

    daemon_startup_failure(child, socket)
}

fn daemon_startup_failure(mut child: Child, socket: &Path) -> ! {
    if child
        .try_wait()
        .expect("poll daemon startup failure")
        .is_none()
    {
        let _ = child.kill();
    }
    let output = child
        .wait_with_output()
        .expect("collect daemon startup output");
    panic!(
        "daemon socket did not become ready: {}\ndaemon exit status: {}\ndaemon stdout:\n{}\ndaemon stderr:\n{}",
        socket.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
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
        .arg(dirs.state.path())
        .arg("--cache-dir")
        .arg(dirs.cache.path())
        .arg("--log-dir")
        .arg(dirs.log.path())
        .arg("--format")
        .arg("json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn daemon");

    let child = wait_for_socket(child, &dirs.socket);
    DaemonChild { child }
}

fn write_entry(root: &Path, name: &str) -> std::path::PathBuf {
    #[cfg(unix)]
    set_dir_mode(root, 0o700);
    let path = root.join(format!("{name}.ash"));
    fs::write(
        &path,
        format!(
            r#"use result::Result
use runtime::RuntimeError

fn {name}() -> Result<(), RuntimeError> {{ Ok {{ value: {{}} }} }}
"#
        ),
    )
    .expect("write entry");
    path
}

fn run_kernel_report(entry_path: &Path) -> Value {
    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg("--dry-run")
        .arg(entry_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    serde_json::from_slice(&output).expect("run kernel report json")
}

fn daemon_json(socket: &Path, args: &[&str]) -> Value {
    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
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

fn definition<'a>(list: &'a Value, application: &str) -> &'a Value {
    list["definitions"]
        .as_array()
        .expect("definitions")
        .iter()
        .find(|definition| definition["application"] == application)
        .expect("indexed entry definition")
}

fn language_artifact_summary(value: &Value) -> Value {
    let mut summary = value["artifact_summary"].clone();
    assert_eq!(
        summary["tcir"]["carrier_scope"], "checked_function_artifact",
        "TASK-1972 runtime summaries must carry checked-function provenance: {summary}"
    );
    assert!(
        summary["tcir"].is_object(),
        "artifact summary must include verifier-normalized alpha TCIR boundary provenance: {summary}"
    );
    assert!(
        summary["amir"].is_object(),
        "artifact summary must include verifier-normalized AMIR provenance: {summary}"
    );
    assert!(
        summary["bytecode"].is_object(),
        "artifact summary must include verifier-normalized bytecode: {summary}"
    );
    assert_eq!(summary["bytecode"]["requires_source_reparse"], false);
    normalize_application_boundary_metadata(&mut summary);
    summary
}

fn normalize_application_boundary_metadata(summary: &mut Value) {
    let Some(invocation_packet) = summary.get_mut("invocation_packet") else {
        return;
    };

    if let Some(admission_profile) = invocation_packet.get_mut("admission_profile") {
        admission_profile["boundary_source"] = Value::String("<runtime-boundary>".to_string());
    }
    if let Some(boundary_bindings) = invocation_packet.get_mut("boundary_bindings") {
        boundary_bindings["boundary_source"] = Value::String("<runtime-boundary>".to_string());
        boundary_bindings["redacted_evidence_identity"] =
            Value::String("<redacted-boundary-evidence>".to_string());
    }
}

#[test]
fn run_and_daemon_share_language_artifact_summary_but_not_host_mode() {
    let root = tempdir().expect("root tempdir");
    let entry_path = write_entry(root.path(), "main");
    let dirs = daemon_dirs();
    #[cfg(unix)]
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);

    let run = run_kernel_report(&entry_path);
    let list = daemon_json(&dirs.socket, &["list"]);
    let daemon_definition = definition(&list, "main");

    assert_eq!(run["host_mode"], "OneShot");
    assert_eq!(list["host_mode"], "Daemon");
    assert_ne!(run["host_mode"], list["host_mode"]);
    assert_eq!(run["application"], daemon_definition["application"]);

    let run_summary = language_artifact_summary(&run);
    let daemon_summary = language_artifact_summary(daemon_definition);
    assert_eq!(run_summary["tcir"], daemon_summary["tcir"]);
    assert_eq!(run_summary["amir"], daemon_summary["amir"]);
    assert_eq!(run_summary["bytecode"], daemon_summary["bytecode"]);
    assert_eq!(run_summary, daemon_summary);
}

#[test]
fn failed_daemon_reload_preserves_admitted_artifact_summary() {
    let root = tempdir().expect("root tempdir");
    let entry_path = write_entry(root.path(), "main");
    let dirs = daemon_dirs();
    #[cfg(unix)]
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);

    let initial_list = daemon_json(&dirs.socket, &["list"]);
    let initial_definition_summary =
        language_artifact_summary(definition(&initial_list, "main")).clone();
    let start = daemon_json(&dirs.socket, &["start", "main"]);
    let instance_id = start["instance_id"]
        .as_str()
        .expect("instance id")
        .to_string();
    let admitted_summary = language_artifact_summary(&start).clone();
    assert_eq!(initial_definition_summary, admitted_summary);

    fs::write(&entry_path, "fn main() -> Result<(), RuntimeError> {")
        .expect("write malformed entry");
    Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("daemon")
        .arg("reload")
        .arg("--socket")
        .arg(&dirs.socket)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicates::str::contains("parse"));

    let preserved_list = daemon_json(&dirs.socket, &["list"]);
    let preserved_definition_summary =
        language_artifact_summary(definition(&preserved_list, "main")).clone();
    let status = daemon_json(&dirs.socket, &["status", "--instance", &instance_id]);
    let preserved_instance_summary = language_artifact_summary(&status);

    assert_eq!(preserved_definition_summary, initial_definition_summary);
    assert_eq!(preserved_instance_summary, admitted_summary);
}

#[test]
fn daemon_start_execute_fails_closed_when_live_source_drifts_from_admitted_artifact() {
    let root = tempdir().expect("root tempdir");
    let entry_path = write_entry(root.path(), "main");
    let dirs = daemon_dirs();
    #[cfg(unix)]
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);

    let list = daemon_json(&dirs.socket, &["list"]);
    let admitted_definition = definition(&list, "main");
    let admitted_source_hash = admitted_definition["source_hash"]
        .as_str()
        .expect("source hash")
        .to_string();

    fs::write(
        &entry_path,
        r#"use result::Result
use runtime::RuntimeError

fn main() -> Result<(), RuntimeError> {
    Ok { value: {} }
}
"#,
    )
    .expect("rewrite entry after daemon admission");

    let start = daemon_json(&dirs.socket, &["start-execute", "main"]);
    assert_eq!(start["status"], "failed", "start response: {start}");
    assert_eq!(start["source_hash"], admitted_source_hash);
    assert_eq!(
        start["report"]["failure"]["kind"], "application_execution_failure",
        "drift is a application-boundary execution failure, not a daemon host crash: {start}"
    );
    let failure_message = start["report"]["failure"]["message"]
        .as_str()
        .expect("failure message");
    assert!(
        failure_message.contains("admitted artifact drift")
            && failure_message.contains(&admitted_source_hash),
        "failure should identify the pinned admitted source/artifact boundary: {start}"
    );
}

#[cfg(unix)]
#[test]
fn daemon_start_execute_uses_hashed_source_bytes_after_drift_check() {
    let root = tempdir().expect("root tempdir");
    let entry_path = write_entry(root.path(), "main");
    let dirs = daemon_dirs();
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);
    let admitted_source_hash =
        definition(&daemon_json(&dirs.socket, &["list"]), "main")["source_hash"]
            .as_str()
            .expect("admitted source hash")
            .to_string();

    let socket = dirs.socket.clone();
    let response = thread::spawn(move || {
        daemon_protocol_json(
            &socket,
            serde_json::json!({
                "command": "start",
                "application": "main",
                "config_id": "default",
                "admission_profile": "allow",
                "execute": true
            }),
        )
    });

    thread::sleep(Duration::from_millis(15));
    fs::write(&entry_path, "fn main() -> Result<(), RuntimeError> {")
        .expect("mutate entry after drift check");

    let start = response.join().expect("daemon response thread");
    assert_eq!(start["ok"], true, "start-execute response: {start}");
    assert_eq!(start["status"], "failed", "start-execute response: {start}");
    assert_eq!(start["report"]["status"], "failed");
    assert_eq!(start["source_hash"], admitted_source_hash);

    let failure_message = start["report"]["failure"]["message"]
        .as_str()
        .expect("closed-admission failure message");
    assert!(
        failure_message.contains("checked Core/CPS admission rejected"),
        "generic daemon execution must fail at Path B admission: {start}"
    );
    assert!(
        !failure_message.contains("parse")
            && !failure_message.contains("malformed")
            && !failure_message.contains("admitted artifact drift"),
        "hashed admitted bytes must be used after the drift check, not the malformed live rewrite: {start}"
    );
}
