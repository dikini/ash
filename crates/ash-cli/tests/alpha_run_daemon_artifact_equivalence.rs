use assert_cmd::Command;
use serde_json::Value;
use std::fs;
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
    DaemonDirs {
        socket,
        _socket_parent: socket_parent,
        state: tempdir().expect("state tempdir"),
        cache: tempdir().expect("cache tempdir"),
        log: tempdir().expect("log tempdir"),
    }
}

fn ash_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("ash")
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

fn write_workflow(root: &Path, name: &str) -> std::path::PathBuf {
    let path = root.join(format!("{name}.ash"));
    fs::write(
        &path,
        format!(
            r#"
            use result::Result
            use runtime::RuntimeError

            workflow {name}() -> Result<(), RuntimeError> {{ done; }}
            "#
        ),
    )
    .expect("write workflow");
    path
}

fn run_kernel_report(workflow_path: &Path) -> Value {
    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg("--dry-run")
        .arg(workflow_path)
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

fn definition<'a>(list: &'a Value, workflow: &str) -> &'a Value {
    list["definitions"]
        .as_array()
        .expect("definitions")
        .iter()
        .find(|definition| definition["workflow"] == workflow)
        .expect("indexed workflow definition")
}

fn language_artifact_summary(value: &Value) -> &Value {
    let summary = &value["artifact_summary"];
    assert_eq!(
        summary["tcir"]["carrier_scope"], "alpha_checked_workflow_boundary",
        "TASK-936 alpha summaries compare the checked workflow-boundary carrier, not full body TCIR: {summary}"
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
    summary
}

#[test]
fn run_and_daemon_share_language_artifact_summary_but_not_host_mode() {
    let root = tempdir().expect("root tempdir");
    let workflow_path = write_workflow(root.path(), "main");
    let dirs = daemon_dirs();
    let _daemon = spawn_daemon(root.path(), &dirs);

    let run = run_kernel_report(&workflow_path);
    let list = daemon_json(&dirs.socket, &["list"]);
    let daemon_definition = definition(&list, "main");

    assert_eq!(run["host_mode"], "OneShot");
    assert_eq!(list["host_mode"], "Daemon");
    assert_ne!(run["host_mode"], list["host_mode"]);
    assert_eq!(run["workflow"], daemon_definition["workflow"]);

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
    let workflow_path = write_workflow(root.path(), "main");
    let dirs = daemon_dirs();
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

    fs::write(&workflow_path, "workflow main {").expect("write malformed workflow");
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
    assert_eq!(preserved_instance_summary, &admitted_summary);
}
