#![cfg(unix)]

//! TASK-2041 contract for one declared source through four local Engine clients.
//!
//! `ash run`, `ash test`, and the REPL each complete locally before this test
//! starts a daemon. The daemon is a separate long-running-process host whose
//! descriptor route owns its own local Engine.

use ash_cli::test_runner::{
    Outcome, TestSource,
    executor::{SuiteConfig, SynthesizedSources, run_suite},
    synthesized::{RunnerContractMetadata, RunnerIntrospectionSnapshot},
};
use ash_core::Value as AshValue;
use ash_engine::CanonicalTerminalEnvelopeV1;
use ash_repl::Repl;
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
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::{TempDir, tempdir};

const SHARED_ROUTE_ID: &str = "TASK-2035-SHARED-ROUTE-001";
const SHARED_SOURCE_ID: &str = "task-2035-shared-int-42-v1";
const SHARED_SOURCE_DIGEST: &str =
    "sha256:ed4088d136e54744d258b170222ad3b2a064feda91b78b0a248f2ccfb9b7684c";
const SHARED_SOURCE: &str = "fn main() -> Int { 42 }\n";

struct DaemonChild {
    child: Child,
}

impl Drop for DaemonChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct DaemonFixture {
    _daemon: DaemonChild,
    socket: PathBuf,
    _root: TempDir,
    _socket_parent: TempDir,
    _state: TempDir,
    _cache: TempDir,
    _log: TempDir,
}

fn ash_bin() -> PathBuf {
    cargo_bin("ash")
}

fn expected_terminal() -> CanonicalTerminalEnvelopeV1 {
    CanonicalTerminalEnvelopeV1::returned(AshValue::Int(42))
}

fn run_terminal() -> CanonicalTerminalEnvelopeV1 {
    let source_dir = tempdir().expect("run source tempdir");
    let source_path = source_dir.path().join("shared.ash");
    fs::write(&source_path, SHARED_SOURCE).expect("write exact shared source");

    let output = Command::new(ash_bin())
        .arg("--color")
        .arg("never")
        .arg("run")
        .arg(&source_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run exact shared source");
    assert_eq!(
        output.status.code(),
        Some(0),
        "ash run status must succeed; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    normalize_cli_terminal(
        serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "ash run must emit terminal JSON: {error}; stdout:\n{}",
                String::from_utf8_lossy(&output.stdout),
            )
        }),
    )
}

fn test_runner_terminal() -> CanonicalTerminalEnvelopeV1 {
    test_runner_terminal_for_snapshot(shared_snapshot())
}

fn test_runner_terminal_for_snapshot(
    snapshot: RunnerIntrospectionSnapshot,
) -> CanonicalTerminalEnvelopeV1 {
    let path = PathBuf::from("task-2041-shared-route.ash");
    let suite = run_suite(&SuiteConfig {
        root: path.clone(),
        include_synthesized: true,
        only_synthesized: true,
        synthesized_sources: SynthesizedSources {
            contracts: true,
            obligations: false,
            laws: false,
        },
        synthesized_snapshots: vec![(path, snapshot)],
        ..SuiteConfig::default()
    });
    assert_eq!(
        suite.tests.len(),
        1,
        "ash test must produce exactly one declared shared contract result: {:#?}",
        suite.tests,
    );
    let only_result = suite
        .tests
        .first()
        .expect("exactly one declared shared contract result exists");
    assert_eq!(
        only_result.source,
        TestSource::Contract,
        "ash test must classify the declared shared route as a contract: {only_result:?}",
    );
    let result = suite
        .tests
        .iter()
        .find(|result| {
            result
                .repro_artifact
                .as_ref()
                .is_some_and(|artifact| artifact.case_id == SHARED_ROUTE_ID)
        })
        .expect("ash test returns the declared shared route result");
    assert_eq!(
        result.outcome,
        Outcome::Pass,
        "ash test must use its local Engine route: {result:?}",
    );
    let repro = result
        .repro_artifact
        .as_ref()
        .expect("declared test route preserves repro metadata");
    assert_eq!(repro.case_id, SHARED_ROUTE_ID);
    assert_eq!(
        repro.oracle_snapshot["source"],
        json!(SHARED_SOURCE),
        "ash test must not replace the declared source bytes",
    );
    assert_eq!(
        repro.oracle_snapshot["source_digest"],
        json!(SHARED_SOURCE_DIGEST)
    );
    normalize_test_terminal(&repro.oracle_snapshot["engine_terminal_envelope"])
}

fn repl_terminal() -> CanonicalTerminalEnvelopeV1 {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("REPL runtime builds");
    let value = runtime.block_on(async {
        let mut repl = Repl::new(true).expect("REPL initializes without history");
        repl.eval(SHARED_SOURCE)
            .await
            .expect("REPL evaluates the declared shared source")
    });
    CanonicalTerminalEnvelopeV1::returned(value)
}

fn shared_snapshot() -> RunnerIntrospectionSnapshot {
    RunnerIntrospectionSnapshot {
        schema_version: "ash-synthesized-v1.0".to_string(),
        module_identity: "task-2041-four-client-parity".to_string(),
        source_artifact_id: "task-2035-source-catalogue-v1".to_string(),
        check_summary_id: "task-2035-source-catalogue-checked-v1".to_string(),
        contracts: vec![RunnerContractMetadata {
            id: SHARED_ROUTE_ID.to_string(),
            callable_name: "main".to_string(),
            callable_kind: "pure_function".to_string(),
            param_names: Vec::new(),
            param_types: Vec::new(),
            return_type: Some("Int".to_string()),
            ..RunnerContractMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    }
}

#[test]
#[should_panic(expected = "exactly one")]
fn declared_corpus_rejects_an_out_of_scope_catalogue_contract() {
    let mut snapshot = shared_snapshot();
    snapshot.contracts.push(RunnerContractMetadata {
        id: "TASK-2035-SYNTH-WRAPPER-001".to_string(),
        callable_name: "contract_target_zero".to_string(),
        callable_kind: "pure_function".to_string(),
        param_names: Vec::new(),
        param_types: Vec::new(),
        return_type: Some("Bool".to_string()),
        ..RunnerContractMetadata::default()
    });

    test_runner_terminal_for_snapshot(snapshot);
}

fn normalize_cli_terminal(terminal: Value) -> CanonicalTerminalEnvelopeV1 {
    assert_eq!(terminal["schema_version"], 1, "CLI terminal: {terminal}");
    assert_eq!(terminal["kind"], "return", "CLI terminal: {terminal}");
    assert_eq!(terminal["value"], 42, "CLI terminal: {terminal}");
    expected_terminal()
}

fn normalize_test_terminal(terminal: &Value) -> CanonicalTerminalEnvelopeV1 {
    assert_eq!(terminal, &json!({ "returned": { "Int": 42 } }));
    expected_terminal()
}

fn write_bootable_daemon_entry(root: &Path) {
    fs::write(
        root.join("main.ash"),
        "use result::Result\nuse runtime::RuntimeError\n\nfn main() -> Result<(), RuntimeError> { Ok { value: {} } }\n",
    )
    .expect("write daemon bootstrap source");
}

fn daemon_fixture() -> DaemonFixture {
    let root = tempdir().expect("daemon root tempdir");
    set_dir_mode(root.path(), 0o700);
    write_bootable_daemon_entry(root.path());
    let socket_parent = tempdir().expect("socket parent tempdir");
    set_dir_mode(socket_parent.path(), 0o700);
    let socket = socket_parent.path().join("ashd.sock");
    preflight_daemon_socket_support(&socket);
    let state = tempdir().expect("state tempdir");
    let cache = tempdir().expect("cache tempdir");
    let log = tempdir().expect("log tempdir");
    set_dir_mode(state.path(), 0o700);
    set_dir_mode(cache.path(), 0o700);
    set_dir_mode(log.path(), 0o700);
    let child = Command::new(ash_bin())
        .arg("daemon")
        .arg("serve")
        .arg("--root")
        .arg(root.path())
        .arg("--socket")
        .arg(&socket)
        .arg("--state-dir")
        .arg(state.path())
        .arg("--cache-dir")
        .arg(cache.path())
        .arg("--log-dir")
        .arg(log.path())
        .arg("--format")
        .arg("json")
        .env_remove("ASH_RUNTIME_SUPPORT_IDENTITY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn daemon");
    DaemonFixture {
        _daemon: DaemonChild {
            child: wait_for_socket(child, &socket),
        },
        socket,
        _root: root,
        _socket_parent: socket_parent,
        _state: state,
        _cache: cache,
        _log: log,
    }
}

fn set_dir_mode(path: &Path, mode: u32) {
    let mut permissions = fs::metadata(path)
        .unwrap_or_else(|error| panic!("metadata for {}: {error}", path.display()))
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)
        .unwrap_or_else(|error| panic!("chmod {mode:o} {}: {error}", path.display()));
}

fn require_daemon_socket_support(listener: std::io::Result<UnixListener>) -> UnixListener {
    match listener {
        Ok(listener) => listener,
        Err(error) => {
            panic!("TASK-2041 four-client parity requires Unix socket support: {error}");
        }
    }
}

fn preflight_daemon_socket_support(socket: &Path) {
    let probe = socket.with_file_name("task-2041-preflight.sock");
    let listener = require_daemon_socket_support(UnixListener::bind(&probe));
    drop(listener);
    fs::remove_file(&probe).expect("remove Unix socket preflight");
}

#[test]
#[should_panic(expected = "requires Unix socket support")]
fn four_client_evidence_rejects_daemon_socket_permission_denial() {
    require_daemon_socket_support(Err(std::io::Error::from(
        std::io::ErrorKind::PermissionDenied,
    )));
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

fn daemon_terminal(socket: &Path) -> CanonicalTerminalEnvelopeV1 {
    let descriptor = json!({
        "version": 1,
        "source_identity": SHARED_SOURCE_ID,
        "source_digest": SHARED_SOURCE_DIGEST,
        "source": SHARED_SOURCE,
        "entry": "main",
        "inputs": [],
        "bindings": {},
        "run_control": {
            "deadline_millis": null,
            "cancellation": "not_cancelled",
        },
        "host_configuration": null,
    });
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
    let response: Value = serde_json::from_str(&line).expect("daemon response JSON");
    assert_eq!(response["ok"], true, "daemon response: {response}");
    normalize_cli_terminal(response["terminal"].clone())
}

fn assert_run_and_repl_are_local_clients() {
    let run_source = include_str!("../src/commands/run.rs");
    let repl_source = include_str!("../../ash-repl/src/lib.rs");

    assert!(
        run_source.contains("fn build_engine") && run_source.contains("Engine::new()"),
        "ash run must build its own local Engine",
    );
    assert!(
        !run_source.contains("UnixStream") && !run_source.contains("DaemonRequest"),
        "ash run must not use the daemon control transport",
    );
    assert!(
        repl_source.contains("let engine = Engine::default()")
            && repl_source.contains("execute_admitted_program"),
        "REPL must hold and execute through its own local Engine",
    );
    assert!(
        !repl_source.contains("UnixStream") && !repl_source.contains("DaemonRequest"),
        "REPL must not use the daemon control transport",
    );
}

fn assert_four_client_terminals() {
    assert_run_and_repl_are_local_clients();

    let run = run_terminal();
    let test = test_runner_terminal();
    let repl = repl_terminal();
    let expected = expected_terminal();
    assert_eq!(run, expected, "ash run terminal");
    assert_eq!(test, expected, "ash test terminal");
    assert_eq!(repl, expected, "REPL terminal");

    // Four-client parity is evidence only when every declared client ran. A
    // host that rejects the daemon socket fails this target during preflight.
    let fixture = daemon_fixture();
    assert_eq!(
        daemon_terminal(&fixture.socket),
        expected,
        "daemon terminal"
    );
}

#[test]
fn declared_shared_route_has_normalized_terminal_parity_through_four_local_clients() {
    assert_four_client_terminals();
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn declared_corpus_property_keeps_the_single_shared_route(
        declared_case in prop_oneof![Just(0_u8)],
    ) {
        prop_assert_eq!(declared_case, 0);
        assert_four_client_terminals();
    }
}
