#![cfg(unix)]

//! TASK-2042 RED contract for daemon descriptor execution.
//!
//! The daemon and `ash run` use separate local Engines. These tests compare
//! their V1 terminal observations for one declared descriptor; they never
//! attempt to transport an opaque Engine request across the Unix socket.

use assert_cmd::cargo::cargo_bin;
use proptest::prelude::*;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::{
    fs::PermissionsExt,
    net::{UnixListener, UnixStream},
};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::{TempDir, tempdir};

const SHARED_SOURCE_ID: &str = "task-2035-shared-int-42-v1";
const SHARED_SOURCE_DIGEST: &str =
    "sha256:ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c";
const SHARED_SOURCE: &str = "fn main() -> Int { 42 }\n";
const STALE_SHARED_SOURCE_DIGEST: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

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
    socket: PathBuf,
    _socket_parent: TempDir,
    state: TempDir,
    cache: TempDir,
    log: TempDir,
}

struct DaemonFixture {
    _daemon: DaemonChild,
    dirs: DaemonDirs,
    _root: TempDir,
}

#[derive(Clone, Copy, Debug)]
enum NamedDescriptorMutation {
    MissingDigest,
    ForgedIdentity,
    StaleDigest,
    NonzeroDeadline,
    DeadlineWithCancellation,
}

fn ash_bin() -> PathBuf {
    cargo_bin("ash")
}

fn daemon_dirs() -> DaemonDirs {
    let socket_parent = tempdir().expect("socket parent tempdir");
    let socket = socket_parent.path().join("ashd.sock");
    let state = tempdir().expect("state tempdir");
    let cache = tempdir().expect("cache tempdir");
    let log = tempdir().expect("log tempdir");
    set_dir_mode(socket_parent.path(), 0o700);
    set_dir_mode(state.path(), 0o700);
    set_dir_mode(cache.path(), 0o700);
    set_dir_mode(log.path(), 0o700);

    DaemonDirs {
        socket,
        _socket_parent: socket_parent,
        state,
        cache,
        log,
    }
}

fn write_bootable_daemon_entry(root: &Path) {
    set_dir_mode(root, 0o700);
    fs::write(
        root.join("main.ash"),
        "use result::Result\nuse runtime::RuntimeError\n\nfn main() -> Result<(), RuntimeError> { Ok { value: {} } }\n",
    )
    .expect("write daemon bootstrap source");
}

fn daemon_fixture() -> Option<DaemonFixture> {
    let root = tempdir().expect("daemon root tempdir");
    write_bootable_daemon_entry(root.path());
    let dirs = daemon_dirs();
    if !unix_socket_bind_is_permitted(&dirs) {
        return None;
    }

    let child = Command::new(ash_bin())
        .arg("daemon")
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
        .env_remove("ASH_RUNTIME_SUPPORT_IDENTITY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn daemon");
    let daemon = DaemonChild {
        child: wait_for_socket(child, &dirs.socket),
    };

    Some(DaemonFixture {
        _daemon: daemon,
        dirs,
        _root: root,
    })
}

fn set_dir_mode(path: &Path, mode: u32) {
    let mut permissions = fs::metadata(path)
        .unwrap_or_else(|error| panic!("metadata for {}: {error}", path.display()))
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)
        .unwrap_or_else(|error| panic!("chmod {mode:o} {}: {error}", path.display()));
}

fn unix_socket_bind_is_permitted(dirs: &DaemonDirs) -> bool {
    let probe = dirs
        .socket
        .parent()
        .expect("socket parent")
        .join("task-2042-preflight.sock");
    match UnixListener::bind(&probe) {
        Ok(listener) => {
            drop(listener);
            fs::remove_file(&probe).expect("remove Unix socket preflight");
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping TASK-2042 daemon test: Unix socket bind unavailable ({error})");
            false
        }
        Err(error) => panic!("bind Unix socket preflight {}: {error}", probe.display()),
    }
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

fn shared_descriptor(run_control: Value, host_configuration: Value) -> Value {
    json!({
        "version": 1,
        "source_identity": SHARED_SOURCE_ID,
        "source_digest": SHARED_SOURCE_DIGEST,
        "source": SHARED_SOURCE,
        "entry": "main",
        "inputs": [],
        "bindings": {},
        "run_control": run_control,
        "host_configuration": host_configuration,
    })
}

fn normal_control() -> Value {
    json!({
        "deadline_millis": null,
        "cancellation": "not_cancelled",
    })
}

fn daemon_descriptor_response(socket: &Path, descriptor: Value) -> Value {
    let request = json!({
        "command": "execute_admitted_descriptor",
        "descriptor": descriptor,
    });
    let mut stream = UnixStream::connect(socket).expect("connect daemon socket");
    serde_json::to_writer(&mut stream, &request).expect("write daemon descriptor request");
    stream
        .write_all(b"\n")
        .expect("terminate daemon descriptor request");

    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .expect("read daemon descriptor response");
    serde_json::from_str(&line).expect("daemon descriptor response is JSON")
}

fn assert_terminal(response: &Value, expected: Value) {
    assert_eq!(
        response["ok"], true,
        "daemon descriptor response: {response}"
    );
    assert_eq!(
        response["terminal"], expected,
        "daemon descriptor response: {response}"
    );
}

fn run_terminal_for_shared_source() -> Value {
    run_terminal_for_shared_source_with_args(&[])
}

fn run_terminal_for_shared_source_with_args(extra_args: &[&str]) -> Value {
    let source_dir = tempdir().expect("run source tempdir");
    let source_path = source_dir.path().join("shared.ash");
    fs::write(&source_path, SHARED_SOURCE).expect("write exact shared source");

    let mut command = Command::new(ash_bin());
    command
        .arg("--color")
        .arg("never")
        .arg("run")
        .arg(&source_path)
        .args(extra_args)
        .arg("--format")
        .arg("json");
    let output = command.output().expect("run shared source");
    assert_eq!(
        output.status.code(),
        Some(0),
        "ash run must preserve an ordinary successful process status; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    parse_run_terminal(&output)
}

fn parse_run_terminal(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "ash run must emit a canonical terminal JSON document: {error}; stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
    })
}

fn external_terminal(boundary: &str, outcome: &str) -> Value {
    json!({
        "schema_version": 1,
        "kind": "external",
        "boundary": boundary,
        "outcome": outcome,
    })
}

fn invalid_checked_artifact_terminal() -> Value {
    json!({
        "schema_version": 1,
        "kind": "pre_entry_failure",
        "class": "entry_verification",
        "message": "checked Core/CPS artifact is invalid",
    })
}

fn mutated_descriptor(mutation: NamedDescriptorMutation) -> Value {
    let mut descriptor = shared_descriptor(normal_control(), Value::Null);
    let fields = descriptor
        .as_object_mut()
        .expect("shared descriptor is an object");
    match mutation {
        NamedDescriptorMutation::MissingDigest => {
            fields.remove("source_digest");
        }
        NamedDescriptorMutation::ForgedIdentity => {
            fields.insert(
                "source_identity".to_string(),
                Value::String("task-2035-shared-int-42-forged-v1".to_string()),
            );
        }
        NamedDescriptorMutation::StaleDigest => {
            fields.insert(
                "source_digest".to_string(),
                Value::String(STALE_SHARED_SOURCE_DIGEST.to_string()),
            );
        }
        NamedDescriptorMutation::NonzeroDeadline => {
            fields.insert(
                "run_control".to_string(),
                json!({ "deadline_millis": 1, "cancellation": "not_cancelled" }),
            );
        }
        NamedDescriptorMutation::DeadlineWithCancellation => {
            fields.insert(
                "run_control".to_string(),
                json!({ "deadline_millis": 0, "cancellation": "cancelled" }),
            );
        }
    }
    descriptor
}

#[test]
fn shared_descriptor_reaches_the_daemon_local_engine_and_returns_the_run_terminal() {
    let Some(fixture) = daemon_fixture() else {
        return;
    };
    let response = daemon_descriptor_response(
        &fixture.dirs.socket,
        shared_descriptor(normal_control(), Value::Null),
    );
    assert_eq!(
        response["ok"], true,
        "daemon descriptor response: {response}"
    );

    let run_terminal = run_terminal_for_shared_source();
    assert_eq!(run_terminal["schema_version"], 1);
    assert_eq!(run_terminal["kind"], "return");
    assert_eq!(run_terminal["value"], 42);
    assert_eq!(
        response["terminal"], run_terminal,
        "daemon descriptor response: {response}"
    );
}

#[test]
fn shared_run_terminal_projection_is_authorized_by_the_engine_not_client_flags() {
    let terminal = run_terminal_for_shared_source_with_args(&["--timeout", "1"]);

    assert_eq!(terminal["schema_version"], 1);
    assert_eq!(terminal["kind"], "return");
    assert_eq!(terminal["value"], 42);
}

#[test]
fn declared_rejected_admission_control_returns_the_canonical_terminal() {
    let Some(fixture) = daemon_fixture() else {
        return;
    };
    let response = daemon_descriptor_response(
        &fixture.dirs.socket,
        shared_descriptor(normal_control(), json!({ "admission_profile": "reject" })),
    );

    assert_terminal(&response, external_terminal("admission", "rejected"));
}

#[test]
fn declared_host_rejection_precedes_admitted_request_minting() {
    let daemon_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands/daemon.rs"))
            .expect("read daemon command source");
    let execution_start = daemon_source
        .find("fn execute_descriptor_with_local_engine(")
        .expect("daemon retains descriptor execution boundary");
    let execution_end = daemon_source[execution_start..]
        .find("\n}\n\nfn default_config_id")
        .map(|offset| execution_start + offset)
        .expect("descriptor execution boundary ends before daemon configuration helpers");
    let execution = &daemon_source[execution_start..execution_end];
    let rejection = execution
        .find("ValidatedSubmittedProgramRecord::HostRejected")
        .expect("descriptor execution checks the validated host rejection record");
    let engine_build = execution
        .find("Engine::new()")
        .expect("descriptor execution builds an Engine only for executable records");
    let request_mint = execution
        .find(".new_admitted_program_request(")
        .expect("descriptor execution mints an Engine request only after admission");

    assert!(
        rejection < request_mint,
        "a rejected host profile must stop before daemon-local request minting"
    );
    assert!(
        rejection < engine_build,
        "a rejected host profile must stop before daemon-local Engine construction"
    );
}

#[test]
fn declared_timeout_and_cancellation_controls_return_canonical_terminals() {
    let Some(fixture) = daemon_fixture() else {
        return;
    };
    let timeout = daemon_descriptor_response(
        &fixture.dirs.socket,
        shared_descriptor(
            json!({ "deadline_millis": 0, "cancellation": "not_cancelled" }),
            Value::Null,
        ),
    );
    assert_terminal(&timeout, external_terminal("execution", "timeout"));

    let cancelled = daemon_descriptor_response(
        &fixture.dirs.socket,
        shared_descriptor(
            json!({ "deadline_millis": null, "cancellation": "cancelled" }),
            Value::Null,
        ),
    );
    assert_terminal(&cancelled, external_terminal("execution", "cancelled"));
}

#[test]
fn undeclared_run_control_records_reject_before_daemon_execution() {
    let Some(fixture) = daemon_fixture() else {
        return;
    };

    for mutation in [
        NamedDescriptorMutation::NonzeroDeadline,
        NamedDescriptorMutation::DeadlineWithCancellation,
    ] {
        let response =
            daemon_descriptor_response(&fixture.dirs.socket, mutated_descriptor(mutation));
        assert_terminal(&response, invalid_checked_artifact_terminal());
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn named_descriptor_mutations_reject_before_daemon_execution(
        mutation in prop_oneof![
            Just(NamedDescriptorMutation::MissingDigest),
            Just(NamedDescriptorMutation::ForgedIdentity),
            Just(NamedDescriptorMutation::StaleDigest),
            Just(NamedDescriptorMutation::NonzeroDeadline),
            Just(NamedDescriptorMutation::DeadlineWithCancellation),
        ],
    ) {
        let Some(fixture) = daemon_fixture() else {
            return Ok(());
        };
        let response = daemon_descriptor_response(&fixture.dirs.socket, mutated_descriptor(mutation));
        let expected = invalid_checked_artifact_terminal();

        prop_assert_eq!(
            response.get("ok"),
            Some(&Value::Bool(true)),
            "daemon descriptor response: {}",
            response,
        );
        prop_assert_eq!(
            response.get("terminal"),
            Some(&expected),
            "daemon descriptor response: {}",
            response,
        );
    }

    #[test]
    fn declared_corpus_descriptor_matches_the_run_terminal(
        descriptor in Just(shared_descriptor(normal_control(), Value::Null)),
    ) {
        let Some(fixture) = daemon_fixture() else {
            return Ok(());
        };
        let response = daemon_descriptor_response(&fixture.dirs.socket, descriptor);
        let run_terminal = run_terminal_for_shared_source();

        prop_assert_eq!(
            response.get("ok"),
            Some(&Value::Bool(true)),
            "daemon descriptor response: {}",
            response,
        );
        prop_assert_eq!(
            response.get("terminal"),
            Some(&run_terminal),
            "daemon and direct ash run must expose the same selected descriptor terminal: {}",
            response,
        );
    }
}
