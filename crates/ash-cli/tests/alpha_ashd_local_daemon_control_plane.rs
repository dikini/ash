use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command as StdCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::{TempDir, tempdir};

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

fn write_workflow(root: &Path, name: &str, value: i32) {
    fs::write(
        root.join(format!("{name}.ash")),
        format!("workflow {name} {{ ret {value}; }}\n"),
    )
    .expect("write workflow");
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
    DaemonDirs {
        socket,
        _socket_parent: socket_parent,
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

#[test]
fn ashd_serve_indexes_definitions_without_running_workflows() {
    let root = tempdir().expect("root tempdir");
    write_workflow(root.path(), "alpha", 1);
    write_workflow(root.path(), "beta", 2);
    let dirs = daemon_dirs();
    let _daemon = spawn_daemon(root.path(), &dirs);

    let list = daemon_json(&dirs.socket, &["list"]);

    assert_eq!(list["host_mode"], "Daemon");
    assert_eq!(
        list["provider_registry"]["grants_admission_authority"],
        false
    );
    assert_eq!(list["instances"].as_array().expect("instances").len(), 0);
    let names: Vec<&str> = list["definitions"]
        .as_array()
        .expect("definitions")
        .iter()
        .map(|definition| definition["workflow"].as_str().expect("workflow name"))
        .collect();
    assert!(names.contains(&"alpha"), "definitions: {list}");
    assert!(names.contains(&"beta"), "definitions: {list}");

    let start = daemon_json(&dirs.socket, &["start", "alpha"]);
    let instance_id = start["instance_id"].as_str().expect("instance id");
    let cancel = daemon_json(&dirs.socket, &["cancel", instance_id]);
    assert_eq!(cancel["host_mode"], "Daemon");
    assert_eq!(cancel["class"], "cancelled");
    assert_eq!(cancel["status"], "cancelled");
}

#[cfg(unix)]
#[test]
fn ashd_start_protocol_round_trips_args_config_and_admission_profile() {
    let root = tempdir().expect("root tempdir");
    write_workflow(root.path(), "main", 7);
    let dirs = daemon_dirs();
    let _daemon = spawn_daemon(root.path(), &dirs);

    let start = daemon_protocol_json(
        &dirs.socket,
        serde_json::json!({
            "command": "start",
            "workflow": "main",
            "args": ["alpha", "beta"],
            "config_id": "staging",
            "admission_profile": "allow"
        }),
    );

    assert_eq!(start["ok"], true);
    assert_eq!(start["host_mode"], "Daemon");
    assert_eq!(start["workflow"], "main");
    assert_eq!(start["args"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(start["config_id"], "staging");
    assert_eq!(start["admission"]["status"], "admitted");
    assert_eq!(start["admission"]["profile"], "allow");
    assert_eq!(start["admission"]["capability_grants"], 0);
    assert_eq!(start["admission"]["resource_grants"], 0);
    let instance_id = start["instance_id"].as_str().expect("instance id");

    let status = daemon_json(&dirs.socket, &["status", "--instance", instance_id]);
    assert_eq!(status["instance_id"], instance_id);
    assert_eq!(status["args"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(status["config_id"], "staging");
    assert_eq!(status["admission"]["status"], "admitted");
    assert_eq!(status["admission"]["profile"], "allow");

    let list = daemon_json(&dirs.socket, &["list"]);
    let instances = list["instances"].as_array().expect("instances");
    assert_eq!(instances.len(), 1, "instances: {list}");
    assert_eq!(instances[0]["instance_id"], instance_id);
    assert_eq!(instances[0]["args"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(instances[0]["config_id"], "staging");
    assert_eq!(instances[0]["admission"]["profile"], "allow");
}

#[cfg(unix)]
#[test]
fn ashd_start_cli_rejects_admission_profile_without_recording_instance() {
    let root = tempdir().expect("root tempdir");
    write_workflow(root.path(), "main", 9);
    let dirs = daemon_dirs();
    let _daemon = spawn_daemon(root.path(), &dirs);

    let before = daemon_json(&dirs.socket, &["list"]);
    assert_eq!(before["instances"].as_array().expect("instances").len(), 0);

    let protocol_rejected = daemon_protocol_json(
        &dirs.socket,
        serde_json::json!({
            "command": "start",
            "workflow": "main",
            "args": ["would-not-run"],
            "config_id": "rejected-config",
            "admission_profile": "reject"
        }),
    );
    assert_eq!(protocol_rejected["ok"], false);
    assert_eq!(protocol_rejected["error"]["class"], "admission_rejected");
    assert!(
        protocol_rejected["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("admission rejected"),
        "protocol response must classify rejected admission: {protocol_rejected}"
    );

    let after_protocol_reject = daemon_json(&dirs.socket, &["list"]);
    assert_eq!(
        after_protocol_reject["instances"]
            .as_array()
            .expect("instances")
            .len(),
        0,
        "protocol rejected admission must not record an instance: {after_protocol_reject}"
    );

    let mut rejected = Command::cargo_bin("ash").expect("ash binary exists");
    rejected
        .arg("daemon")
        .arg("start")
        .arg("--arg")
        .arg("would-not-run")
        .arg("--config-id")
        .arg("rejected-config")
        .arg("--admission-profile")
        .arg("reject")
        .arg("main")
        .arg("--socket")
        .arg(&dirs.socket)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicates::str::contains("admission").and(predicates::str::contains("rejected")));

    let after = daemon_json(&dirs.socket, &["list"]);
    assert_eq!(
        after["instances"].as_array().expect("instances").len(),
        0,
        "rejected admission must not record an instance: {after}"
    );
}

#[test]
fn ashd_start_cli_records_default_empty_admission_fields() {
    let root = tempdir().expect("root tempdir");
    write_workflow(root.path(), "main", 11);
    let dirs = daemon_dirs();
    let _daemon = spawn_daemon(root.path(), &dirs);

    let start = daemon_json(&dirs.socket, &["start", "main"]);

    assert_eq!(start["ok"], true);
    assert_eq!(start["args"], serde_json::json!([]));
    assert_eq!(start["config_id"], "default");
    assert_eq!(start["admission"]["status"], "admitted");
    assert_eq!(start["admission"]["profile"], "empty");
}

#[test]
fn ashd_reload_updates_definition_table_and_preserves_kernel_mode() {
    let root = tempdir().expect("root tempdir");
    write_workflow(root.path(), "main", 1);
    let dirs = daemon_dirs();
    let _daemon = spawn_daemon(root.path(), &dirs);

    let first = daemon_json(&dirs.socket, &["start", "main"]);
    let instance_id = first["instance_id"]
        .as_str()
        .expect("instance id")
        .to_string();
    let first_artifact = first["artifact_id"]
        .as_str()
        .expect("artifact id")
        .to_string();

    fs::write(root.path().join("main.ash"), "workflow main {").expect("write malformed workflow");
    let mut failed_reload = Command::cargo_bin("ash").expect("ash binary exists");
    failed_reload
        .arg("daemon")
        .arg("reload")
        .arg("--socket")
        .arg(&dirs.socket)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicates::str::contains("parse"));

    let preserved = daemon_json(&dirs.socket, &["start", "main"]);
    assert_eq!(
        preserved["artifact_id"].as_str().expect("artifact id"),
        first_artifact
    );

    fs::write(root.path().join("main.ash"), "workflow main { ret 2; }\n")
        .expect("rewrite workflow");
    let reload = daemon_json(&dirs.socket, &["reload"]);
    assert_eq!(reload["host_mode"], "Daemon");
    assert_eq!(reload["status"], "reloaded");

    let second = daemon_json(&dirs.socket, &["start", "main"]);
    let second_artifact = second["artifact_id"].as_str().expect("artifact id");
    assert_ne!(first_artifact, second_artifact);

    let status = daemon_json(&dirs.socket, &["status", "--instance", &instance_id]);
    assert_eq!(status["host_mode"], "Daemon");
    assert_eq!(status["instance_id"], instance_id);
    assert_eq!(status["artifact_id"], first_artifact);
}

#[test]
fn ashd_rejects_invalid_root() {
    let root_parent = tempdir().expect("root parent");
    let missing_root = root_parent.path().join("missing");
    let dirs = daemon_dirs();

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("daemon")
        .arg("serve")
        .arg("--root")
        .arg(&missing_root)
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
        .assert()
        .failure()
        .stderr(predicates::str::contains("root"));

    assert!(
        !dirs.socket.exists(),
        "invalid root must not leave daemon socket"
    );
}

#[test]
fn ashd_rejects_preexisting_non_socket_control_path_without_deleting_it() {
    let root = tempdir().expect("root tempdir");
    write_workflow(root.path(), "main", 1);
    let dirs = daemon_dirs();
    fs::write(&dirs.socket, "important local file").expect("preexisting socket path file");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("daemon")
        .arg("serve")
        .arg("--root")
        .arg(root.path())
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
        .assert()
        .failure()
        .stderr(predicates::str::contains("socket"));

    assert_eq!(
        fs::read_to_string(&dirs.socket).expect("preexisting file preserved"),
        "important local file"
    );
    #[cfg(unix)]
    assert!(
        !fs::symlink_metadata(&dirs.socket)
            .expect("metadata for preserved file")
            .file_type()
            .is_socket(),
        "preexisting regular file must not be replaced by a daemon socket"
    );
}
