use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
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

fn write_entry(root: &Path, name: &str, value: i32) {
    #[cfg(unix)]
    set_dir_mode(root, 0o700);
    fs::write(
        root.join(format!("{name}.ash")),
        format!(
            r#"use result::Result
use runtime::RuntimeError

fn {name}() -> Result<(), RuntimeError> {{ Ok {{ value: {{}} }} }}
// fixture value: {value}
"#
        ),
    )
    .expect("write entry");
}

fn write_source(root: &Path, name: &str, source: &str) {
    #[cfg(unix)]
    set_dir_mode(root, 0o700);
    fs::write(root.join(format!("{name}.ash")), source).expect("write source fixture");
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
        .env_remove("ASH_RUNTIME_SUPPORT_IDENTITY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn daemon");

    let child = wait_for_socket(child, &dirs.socket);
    DaemonChild { child }
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
                "skipping daemon control-plane test: Unix socket bind is not permitted in this environment ({error})"
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

#[cfg(unix)]
fn assert_daemon_serve_rejects(root: &Path, dirs: &DaemonDirs, expected_stderr: &str) {
    let mut child = StdCommand::new(ash_bin())
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
        .expect("spawn daemon rejection probe");

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if child
            .try_wait()
            .expect("poll daemon rejection probe")
            .is_some()
        {
            let output = child.wait_with_output().expect("daemon rejection output");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "daemon serve unexpectedly succeeded; stderr:\n{stderr}"
            );
            assert!(
                stderr.contains(expected_stderr),
                "expected stderr to contain {expected_stderr:?}; stderr:\n{stderr}"
            );
            return;
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("daemon rejection output after kill");
            panic!(
                "daemon serve started instead of rejecting unsafe local-control paths; stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn expected_runtime_kernel_digest(parts: &[&str]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.len().to_le_bytes());
        hasher.update(part.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
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
fn ashd_serve_indexes_definitions_without_running_applications() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 1);
    let dirs = daemon_dirs();
    #[cfg(unix)]
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);
    #[cfg(unix)]
    assert_eq!(
        fs::symlink_metadata(&dirs.socket)
            .expect("daemon socket metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600,
        "daemon socket must be same-user-only"
    );

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
        .map(|definition| {
            definition["application"]
                .as_str()
                .expect("application name")
        })
        .collect();
    assert!(names.contains(&"main"), "definitions: {list}");
    let main_definition = list["definitions"]
        .as_array()
        .expect("definitions")
        .iter()
        .find(|definition| definition["application"] == "main")
        .expect("main definition");
    let main_source = fs::read_to_string(root.path().join("main.ash")).expect("read main source");
    let expected_source_hash = expected_runtime_kernel_digest(&["source", &main_source]);
    // The checked-TCIR fingerprint is part of this cache identity. It is a
    // canonical serialization of the typechecked function artifact, so this
    // external daemon test fixes the expected deterministic hash instead of
    // attempting to duplicate the internal serializer.
    let expected_check_summary_hash =
        "sha256:9ba547697243e0d54d51b0d8782e594372d4a45ccf360e3f76203ad0176fffee";
    assert_eq!(
        main_definition["source_hash"], expected_source_hash,
        "daemon source identity must use stable SHA-256 content digest"
    );
    assert_eq!(
        main_definition["check_summary_hash"], expected_check_summary_hash,
        "daemon summary identity must bind the checked-function provenance"
    );

    let start = daemon_json(&dirs.socket, &["start", "main"]);
    let instance_id = start["instance_id"].as_str().expect("instance id");
    let cancel = daemon_json(&dirs.socket, &["cancel", instance_id]);
    assert_eq!(cancel["host_mode"], "Daemon");
    assert_eq!(cancel["class"], "cancelled");
    assert_eq!(cancel["status"], "cancelled");
    assert_eq!(cancel["service_lifecycle"]["lifecycle"], "terminated");
    assert_eq!(cancel["service_lifecycle"]["shutdown_mode"], "graceful");
    assert_eq!(cancel["service_lifecycle"]["terminal"], true);
    assert_eq!(cancel["service_lifecycle"]["retained"], true);
}

#[cfg(unix)]
#[test]
fn ashd_start_protocol_round_trips_args_config_and_admission_profile() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 7);
    let dirs = daemon_dirs();
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);

    let start = daemon_protocol_json(
        &dirs.socket,
        serde_json::json!({
            "command": "start",
            "application": "main",
            "args": ["alpha", "beta"],
            "config_id": "default",
            "admission_profile": "allow"
        }),
    );

    assert_eq!(start["ok"], true);
    assert_eq!(start["host_mode"], "Daemon");
    assert_eq!(start["application"], "main");
    assert_eq!(start["args"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(start["config_id"], "default");
    assert_eq!(start["admission"]["status"], "admitted");
    assert_eq!(start["admission"]["profile"], "allow");
    assert_eq!(start["admission"]["capability_grants"], 0);
    assert_eq!(start["admission"]["resource_grants"], 0);
    assert_eq!(
        start["artifact_summary"]["invocation_packet"]["admission_profile"]["name"],
        "allow"
    );
    assert_eq!(
        start["artifact_summary"]["invocation_packet"]["admission_profile"]["grants_authority"],
        false
    );
    assert_eq!(
        start["artifact_summary"]["invocation_packet"]["boundary_bindings"]["boundary_source"],
        "daemon:start.boundary"
    );
    assert_eq!(
        start["artifact_summary"]["invocation_packet"]["boundary_bindings"]["providers"],
        serde_json::json!(["Args:0", "Args:1"])
    );
    assert_eq!(
        start["artifact_summary"]["invocation_packet"]["boundary_bindings"]["grants_authority"],
        false
    );
    assert_eq!(
        start["application_report"]["terminal_outcome"]["status"],
        "admitted"
    );
    assert_eq!(
        start["application_report"]["terminal_outcome"]["is_terminal"],
        false
    );
    assert_eq!(
        start["application_report"]["source_identity"],
        start["artifact_summary"]["invocation_packet"]["source_identity"]
    );
    assert_eq!(
        start["application_report"]["trace_bundle"]["boundary_facts"][0],
        "boundary_source:daemon:start.boundary"
    );
    assert_eq!(
        start["application_report"]["trace_bundle"]["grants_authority"],
        false
    );
    assert_eq!(
        start["application_report"]["trace_bundle"]["mutates_authority"],
        false
    );
    assert_eq!(start["service_lifecycle"]["name"], "main");
    assert_eq!(start["service_lifecycle"]["lifecycle"], "running");
    assert_eq!(start["service_lifecycle"]["health"], "healthy");
    assert_eq!(start["service_lifecycle"]["reload_generation"], 0);
    assert_eq!(start["service_lifecycle"]["terminal"], false);
    assert_eq!(start["service_lifecycle"]["retained"], true);
    let instance_id = start["instance_id"].as_str().expect("instance id");

    let status = daemon_json(&dirs.socket, &["status", "--instance", instance_id]);
    assert_eq!(status["instance_id"], instance_id);
    assert_eq!(status["args"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(status["config_id"], "default");
    assert_eq!(status["admission"]["status"], "admitted");
    assert_eq!(status["admission"]["profile"], "allow");
    assert_eq!(
        status["artifact_summary"]["invocation_packet"]["admission_profile"]["name"],
        "allow"
    );
    assert_eq!(
        status["artifact_summary"]["invocation_packet"]["boundary_bindings"]["providers"],
        serde_json::json!(["Args:0", "Args:1"])
    );
    assert_eq!(
        status["application_report"]["terminal_outcome"]["status"],
        "admitted"
    );
    assert_eq!(status["service_lifecycle"], start["service_lifecycle"]);

    let list = daemon_json(&dirs.socket, &["list"]);
    let instances = list["instances"].as_array().expect("instances");
    assert_eq!(instances.len(), 1, "instances: {list}");
    assert_eq!(instances[0]["instance_id"], instance_id);
    assert_eq!(instances[0]["args"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(instances[0]["config_id"], "default");
    assert_eq!(instances[0]["admission"]["profile"], "allow");
    assert_eq!(
        instances[0]["artifact_summary"]["invocation_packet"]["admission_profile"]["name"],
        "allow"
    );
    assert_eq!(
        instances[0]["artifact_summary"]["invocation_packet"]["boundary_bindings"]["providers"],
        serde_json::json!(["Args:0", "Args:1"])
    );
    assert_eq!(
        instances[0]["application_report"]["terminal_outcome"]["status"],
        "admitted"
    );
    assert_eq!(
        instances[0]["service_lifecycle"],
        start["service_lifecycle"]
    );
}

#[cfg(unix)]
#[test]
fn ashd_start_rejects_non_default_config_id_without_recording_instance() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 8);
    let dirs = daemon_dirs();
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);

    let protocol_rejected = daemon_protocol_json(
        &dirs.socket,
        serde_json::json!({
            "command": "start",
            "application": "main",
            "args": ["would-not-record"],
            "config_id": "staging",
            "admission_profile": "allow"
        }),
    );
    assert_eq!(protocol_rejected["ok"], false);
    assert_eq!(protocol_rejected["error"]["class"], "request_failure");
    assert!(
        protocol_rejected["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("non-default daemon config_id"),
        "protocol response must explain unsupported non-default config IDs: {protocol_rejected}"
    );

    let after_protocol_reject = daemon_json(&dirs.socket, &["list"]);
    assert_eq!(
        after_protocol_reject["instances"]
            .as_array()
            .expect("instances")
            .len(),
        0,
        "non-default config rejection must not record an instance: {after_protocol_reject}"
    );

    let mut rejected = Command::cargo_bin("ash").expect("ash binary exists");
    rejected
        .arg("daemon")
        .arg("start")
        .arg("--arg")
        .arg("would-not-record")
        .arg("--config-id")
        .arg("staging")
        .arg("--admission-profile")
        .arg("allow")
        .arg("main")
        .arg("--socket")
        .arg(&dirs.socket)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicates::str::contains("non-default daemon config_id"));

    let after_cli_reject = daemon_json(&dirs.socket, &["list"]);
    assert_eq!(
        after_cli_reject["instances"]
            .as_array()
            .expect("instances")
            .len(),
        0,
        "CLI non-default config rejection must not record an instance: {after_cli_reject}"
    );
}

#[cfg(unix)]
#[test]
fn ashd_start_cli_rejects_admission_profile_without_recording_instance() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 9);
    let dirs = daemon_dirs();
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);

    let before = daemon_json(&dirs.socket, &["list"]);
    assert_eq!(before["instances"].as_array().expect("instances").len(), 0);

    let protocol_rejected = daemon_protocol_json(
        &dirs.socket,
        serde_json::json!({
            "command": "start",
            "application": "main",
            "args": ["would-not-run"],
            "config_id": "default",
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
        .arg("default")
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
    write_entry(root.path(), "main", 11);
    let dirs = daemon_dirs();
    #[cfg(unix)]
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
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
    write_entry(root.path(), "main", 1);
    let dirs = daemon_dirs();
    #[cfg(unix)]
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
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

    fs::write(
        root.path().join("main.ash"),
        "fn main() -> Result<(), RuntimeError> {",
    )
    .expect("write malformed entry");
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

    fs::write(
        root.path().join("main.ash"),
        r#"use result::Result
use runtime::RuntimeError

fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
"#,
    )
    .expect("rewrite entry");
    let reload = daemon_json(&dirs.socket, &["reload"]);
    assert_eq!(reload["host_mode"], "Daemon");
    assert_eq!(reload["status"], "reloaded");
    assert_eq!(reload["service_lifecycle"]["lifecycle"], "running");
    assert_eq!(reload["service_lifecycle"]["reload_generation"], 1);

    let second = daemon_json(&dirs.socket, &["start", "main"]);
    let second_artifact = second["artifact_id"].as_str().expect("artifact id");
    assert_ne!(first_artifact, second_artifact);

    let status = daemon_json(&dirs.socket, &["status", "--instance", &instance_id]);
    assert_eq!(status["host_mode"], "Daemon");
    assert_eq!(status["instance_id"], instance_id);
    assert_eq!(status["artifact_id"], first_artifact);
}

#[test]
fn ashd_reload_rejects_type_invalid_application_and_preserves_prior_index() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 1);
    let dirs = daemon_dirs();
    #[cfg(unix)]
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let _daemon = spawn_daemon(root.path(), &dirs);

    let original = daemon_json(&dirs.socket, &["start", "main"]);
    let original_artifact = original["artifact_id"].clone();

    fs::write(
        root.path().join("main.ash"),
        r#"use result::Result
use runtime::RuntimeError

fn main() -> Result<(), RuntimeError> { missing_name }
"#,
    )
    .expect("write type-invalid entry");
    let mut failed_reload = Command::cargo_bin("ash").expect("ash binary exists");
    failed_reload
        .arg("daemon")
        .arg("reload")
        .arg("--socket")
        .arg(&dirs.socket)
        .assert()
        .failure()
        .stderr(predicate::str::contains("parse/check/index failure"));

    let preserved = daemon_json(&dirs.socket, &["start", "main"]);
    assert_eq!(
        preserved["artifact_id"], original_artifact,
        "failed check reload must preserve the previously admitted artifact identity"
    );
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

#[cfg(unix)]
#[test]
fn ashd_serve_rejects_world_writable_local_control_dirs() {
    for unsafe_path in ["root", "socket_parent", "state", "cache", "log"] {
        let root = tempdir().expect("root tempdir");
        write_entry(root.path(), "main", 1);
        let dirs = daemon_dirs();

        match unsafe_path {
            "root" => set_dir_mode(root.path(), 0o777),
            "socket_parent" => set_dir_mode(dirs.socket.parent().expect("socket parent"), 0o777),
            "state" => set_dir_mode(dirs.state.path(), 0o777),
            "cache" => set_dir_mode(dirs.cache.path(), 0o777),
            "log" => set_dir_mode(dirs.log.path(), 0o777),
            other => panic!("unknown unsafe path fixture {other}"),
        }

        assert_daemon_serve_rejects(root.path(), &dirs, "group/world-writable");
        assert!(
            !dirs.socket.exists(),
            "unsafe {unsafe_path} must not leave daemon socket {}",
            dirs.socket.display()
        );
    }
}

#[cfg(unix)]
#[test]
fn ashd_rejects_selected_noncanonical_engine_routes_before_execution() {
    const TIME_SLEEP: &str = "fn main() -> Null { time::sleep(0) }";
    const TRAP_SLEEP: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler trap_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => 1 / 0,
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with trap_sleep }
";
    const DEEP_AFFINE_CLOCK: &str = r"
interface Clock<T> { sleep(Int) -> Int wake(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds wake(milliseconds) = milliseconds }
handler deep_affine_clock(comp: () -> { TestClock::sleep, TestClock::wake } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        TestClock::wake(ms, resume) => resume(ms),
        done(value) => value + 100,
    }
}
fn main() -> Int { handle { TestClock::sleep(0); TestClock::wake(1); TestClock::sleep(2); 7 } with deep_affine_clock }
";
    const FORWARD_SLEEP: &str = r"
interface Clock<T> { sleep(Int) -> Int wake(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds wake(milliseconds) = milliseconds }
handler forward_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => TestClock::wake(ms),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with forward_sleep }
";

    for (name, source) in [
        ("time_sleep", TIME_SLEEP),
        ("trap_sleep", TRAP_SLEEP),
        ("deep_affine_clock", DEEP_AFFINE_CLOCK),
        ("forward_sleep", FORWARD_SLEEP),
    ] {
        let root = tempdir().expect("root tempdir");
        write_source(root.path(), "main", source);
        let dirs = daemon_dirs();
        if !unix_socket_bind_is_permitted(&dirs) {
            return;
        }

        assert_daemon_serve_rejects(root.path(), &dirs, "type error");
        assert!(
            !dirs.socket.exists(),
            "the noncanonical {name} route must reject before the daemon worker binds a socket"
        );
    }
}

#[cfg(unix)]
#[test]
fn ashd_serve_rejects_root_not_owned_by_current_effective_user_when_available() {
    let current_user_dir = tempdir().expect("current user tempdir");
    let current_uid = fs::metadata(current_user_dir.path())
        .expect("current user dir metadata")
        .uid();
    let root_not_owned_by_current_user = Path::new("/");
    let candidate_uid = fs::metadata(root_not_owned_by_current_user)
        .expect("candidate root metadata")
        .uid();
    if candidate_uid == current_uid {
        eprintln!(
            "skipping non-current-user ownership check: / is owned by the test effective user"
        );
        return;
    }

    let dirs = daemon_dirs();

    assert_daemon_serve_rejects(
        root_not_owned_by_current_user,
        &dirs,
        "current effective user",
    );
    assert!(
        !dirs.socket.exists(),
        "non-current-user root must not leave daemon socket"
    );
}

#[cfg(unix)]
#[test]
fn ashd_serve_validates_socket_parent_before_removing_stale_socket() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 1);
    let dirs = daemon_dirs();
    if !unix_socket_bind_is_permitted(&dirs) {
        return;
    }
    let stale_listener = UnixListener::bind(&dirs.socket)
        .unwrap_or_else(|error| panic!("bind stale socket: {error}"));
    drop(stale_listener);
    assert!(
        fs::symlink_metadata(&dirs.socket)
            .expect("stale socket metadata")
            .file_type()
            .is_socket(),
        "fixture must create a stale Unix socket"
    );
    set_dir_mode(dirs.socket.parent().expect("socket parent"), 0o777);

    assert_daemon_serve_rejects(root.path(), &dirs, "group/world-writable");

    assert!(
        fs::symlink_metadata(&dirs.socket)
            .expect("stale socket must remain after unsafe parent rejection")
            .file_type()
            .is_socket(),
        "unsafe socket parent rejection must happen before stale socket removal"
    );
}

#[cfg(unix)]
#[test]
fn ashd_serve_rejects_symlinked_socket_parent_before_socket_bind() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 1);
    let real_socket_parent = tempdir().expect("real socket parent");
    set_dir_mode(real_socket_parent.path(), 0o700);
    let symlink_parent_root = tempdir().expect("symlink parent root");
    let symlink_parent = symlink_parent_root.path().join("socket-link");
    std::os::unix::fs::symlink(real_socket_parent.path(), &symlink_parent)
        .expect("create socket parent symlink");
    let mut dirs = daemon_dirs();
    dirs.socket = symlink_parent.join("ashd.sock");

    assert_daemon_serve_rejects(root.path(), &dirs, "symbolic links are not allowed");
    assert!(
        !dirs.socket.exists(),
        "symlinked socket parent must be rejected before daemon socket bind"
    );
}

#[test]
fn ashd_rejects_preexisting_non_socket_control_path_without_deleting_it() {
    let root = tempdir().expect("root tempdir");
    write_entry(root.path(), "main", 1);
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
