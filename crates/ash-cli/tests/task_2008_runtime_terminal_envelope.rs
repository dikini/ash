//! TASK-2008: `ash run` terminal-envelope runtime contracts.
//!
//! These tests intentionally exercise the binary rather than the isolated
//! serializer prototype. They remain red until `ash run --format json` maps
//! every terminal boundary to `CanonicalTerminalObservable`.

use assert_cmd::Command;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    process::{Child, Command as ProcessCommand, Stdio},
    sync::{OnceLock, mpsc},
    thread,
    time::{Duration, Instant},
};
use tempfile::tempdir;

#[cfg(all(unix, target_os = "linux"))]
unsafe extern "C" {
    fn signal(signal: std::os::raw::c_int, handler: usize) -> usize;
}

#[cfg(all(unix, target_os = "linux"))]
const TOKIO_SIGINT_PROBE_ENV: &str = "ASH_TASK_2008_TOKIO_SIGINT_PROBE";
#[cfg(all(unix, target_os = "linux"))]
const TOKIO_SIGINT_PROBE_READY: &str = "task-2008-tokio-sigint-ready";
#[cfg(all(unix, target_os = "linux"))]
const TOKIO_SIGINT_PROBE_TEST: &str = "tokio_sigint_delivery_probe";
#[cfg(all(unix, target_os = "linux"))]
static TOKIO_SIGINT_DELIVERY_AVAILABLE: OnceLock<bool> = OnceLock::new();

fn ash() -> Command {
    Command::cargo_bin("ash").expect("ash binary should be available to integration tests")
}

fn write_fixture(name: &str, source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().expect("temporary fixture root");
    let path = temp.path().join(name);
    fs::write(&path, source).expect("write Ash fixture");
    (temp, path)
}

fn terminal_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap_or_else(|error| {
        panic!(
            "ash run --format json must emit one canonical terminal envelope on stdout; \
             stdout was {:?}; parse error: {error}",
            String::from_utf8_lossy(bytes)
        )
    })
}

fn assert_no_implementation_telemetry(envelope: &Value) {
    for forbidden in [
        "_variant",
        "trace",
        "session",
        "runtime_artifact",
        "instance_id",
        "kernel_id",
        "artifact_id",
    ] {
        assert!(
            envelope.get(forbidden).is_none(),
            "canonical terminal envelope must not expose `{forbidden}`: {envelope}"
        );
    }
}

/// Wait until Linux reports that the child catches SIGINT before delivering
/// the sole cancellation signal.  The test process inherits an ignored
/// SIGINT disposition from `cargo test`, so a fixed delay cannot establish
/// that the child has installed its own Tokio listener.
///
/// This is deliberately Linux-only: `/proc/<pid>/status` is the portable
/// kernel observation surface available to this repository's Unix process
/// tests.  The cancellation integration controls below are correspondingly
/// Linux-gated rather than guessing listener readiness on another Unix.
#[cfg(target_os = "linux")]
fn wait_for_sigint_listener(child: &mut Child) {
    const SIGINT_CATCHER_BIT: u64 = 1 << 1;
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut observed_listener = false;

    loop {
        let status_path = format!("/proc/{}/status", child.id());
        let catches_sigint = fs::read_to_string(&status_path)
            .ok()
            .and_then(|status| {
                status.lines().find_map(|line| {
                    line.strip_prefix("SigCgt:")
                        .and_then(|mask| u64::from_str_radix(mask.trim(), 16).ok())
                })
            })
            .is_some_and(|mask| mask & SIGINT_CATCHER_BIT != 0);
        if catches_sigint {
            if observed_listener {
                return;
            }
            observed_listener = true;
        } else {
            observed_listener = false;
        }

        if let Some(status) = child.try_wait().expect("poll child status") {
            panic!(
                "ash exited before registering its SIGINT listener: {status}; expected the admitted time::sleep route to await the Engine driver"
            );
        }
        assert!(
            Instant::now() < deadline,
            "ash did not register a SIGINT listener within five seconds; refusing to send a signal before the cancellation bridge is ready"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

/// Reset the child SIGINT disposition to the foreground-process default.
///
/// The pre-exec hook performs only the checked libc signal-disposition reset,
/// then the caller executes the exact selected binary. It models normal
/// foreground invocation without changing the process arguments or adding a
/// shell process to the observation boundary.
#[cfg(all(unix, target_os = "linux"))]
fn restore_default_sigint_before_exec(command: &mut ProcessCommand) {
    use std::os::unix::process::CommandExt;

    const SIGINT: std::os::raw::c_int = 2;
    const SIG_DFL: usize = 0;
    const SIG_ERR: usize = usize::MAX;

    // SAFETY: the closure runs only in the child immediately before `exec`.
    // It calls the C signal-disposition setter with fixed SIGINT/SIG_DFL
    // values and returns the kernel error unchanged if registration fails.
    unsafe {
        command.pre_exec(|| {
            let previous = signal(SIGINT, SIG_DFL);
            if previous == SIG_ERR {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn ash_with_default_sigint() -> ProcessCommand {
    let mut command = ProcessCommand::new(env!("CARGO_BIN_EXE_ash"));
    restore_default_sigint_before_exec(&mut command);
    command
}

#[cfg(all(unix, target_os = "linux"))]
fn abort_tokio_sigint_probe(
    child: &mut Child,
    stdout_reader: thread::JoinHandle<String>,
    stderr_reader: thread::JoinHandle<String>,
    reason: &str,
) -> ! {
    let status = match child.try_wait().expect("poll isolated Tokio SIGINT probe") {
        Some(status) => status,
        None => {
            child
                .kill()
                .expect("stop failed isolated Tokio SIGINT probe");
            child
                .wait()
                .expect("reap failed isolated Tokio SIGINT probe")
        }
    };
    let stdout = stdout_reader
        .join()
        .expect("join isolated Tokio SIGINT stdout reader");
    let stderr = stderr_reader
        .join()
        .expect("join isolated Tokio SIGINT stderr reader");
    panic!(
        "isolated Tokio SIGINT probe failed before proving a deliverable listener: {reason}; \
         status={status}; stdout={stdout:?}; stderr={stderr:?}"
    );
}

/// Runs a self-contained Tokio SIGINT receiver in a reset-disposition child.
/// This checks a harness capability, not Ash behavior: cancellation evidence
/// is skipped only when this isolated child proves that programmatic SIGINT
/// cannot reach Tokio's listener in the current test environment.
#[cfg(all(unix, target_os = "linux"))]
fn tokio_sigint_delivery_available() -> bool {
    *TOKIO_SIGINT_DELIVERY_AVAILABLE.get_or_init(|| {
        let test_binary = std::env::current_exe().expect("locate integration-test binary");
        let mut command = ProcessCommand::new(test_binary);
        command
            .args([
                "--ignored",
                "--exact",
                "--nocapture",
                TOKIO_SIGINT_PROBE_TEST,
            ])
            .env(TOKIO_SIGINT_PROBE_ENV, "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        restore_default_sigint_before_exec(&mut command);
        let mut child = command.spawn().expect("start isolated Tokio SIGINT probe");
        let stdout = child
            .stdout
            .take()
            .expect("capture Tokio SIGINT probe stdout");
        let stderr = child
            .stderr
            .take()
            .expect("capture Tokio SIGINT probe stderr");
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let stdout_reader = thread::spawn(move || {
            let mut transcript = String::new();
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => {
                        transcript.push_str(&line);
                        transcript.push('\n');
                        if line == TOKIO_SIGINT_PROBE_READY {
                            let _ = ready_sender.send(Ok(()));
                            return transcript;
                        }
                    }
                    Err(error) => {
                        let _ = ready_sender.send(Err(format!(
                            "could not read isolated Tokio SIGINT probe stdout: {error}"
                        )));
                        return transcript;
                    }
                }
            }
            let _ = ready_sender.send(Err(
                "isolated Tokio SIGINT probe exited before readiness".to_string()
            ));
            transcript
        });
        let stderr_reader = thread::spawn(move || {
            let mut transcript = String::new();
            let _ = BufReader::new(stderr).read_to_string(&mut transcript);
            transcript
        });

        match ready_receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => {}
            Ok(Err(reason)) => {
                abort_tokio_sigint_probe(&mut child, stdout_reader, stderr_reader, &reason);
            }
            Err(error) => {
                abort_tokio_sigint_probe(
                    &mut child,
                    stdout_reader,
                    stderr_reader,
                    &format!("readiness channel timed out: {error}"),
                );
            }
        }

        let signal_status = ProcessCommand::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status();
        if !signal_status.is_ok_and(|status| status.success()) {
            abort_tokio_sigint_probe(
                &mut child,
                stdout_reader,
                stderr_reader,
                "the one programmatic SIGINT could not be delivered",
            );
        }

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(status) = child.try_wait().expect("poll Tokio SIGINT probe") {
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return status.success();
            }
            if Instant::now() >= deadline {
                child
                    .kill()
                    .expect("stop ready Tokio SIGINT probe after delivery timeout");
                child
                    .wait()
                    .expect("reap ready Tokio SIGINT probe after delivery timeout");
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return false;
            }
            thread::sleep(Duration::from_millis(10));
        }
    })
}

/// The self-spawned probe reports readiness only after Tokio has created an
/// explicit SIGINT stream, then succeeds only when that stream receives the
/// parent-delivered signal.
#[cfg(all(unix, target_os = "linux"))]
#[test]
#[ignore]
fn tokio_sigint_delivery_probe() {
    if std::env::var_os(TOKIO_SIGINT_PROBE_ENV).is_none() {
        return;
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build isolated Tokio signal runtime");
    runtime.block_on(async {
        let mut signals = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("register isolated Tokio SIGINT listener");
        println!("{TOKIO_SIGINT_PROBE_READY}");
        std::io::stdout()
            .flush()
            .expect("flush isolated Tokio SIGINT readiness");
        assert!(
            signals.recv().await.is_some(),
            "the isolated Tokio SIGINT stream must yield its delivered signal"
        );
    });
}

// This is TASK-2014's entire currently admitted host-operation source slice.
// It deliberately has no entry bootstrap wrapper, handler, residual row, or
// legacy-evaluator-compatible alternate form: a successful CLI execution must
// therefore reach the Engine's sealed checked-CPS `time::sleep` route.
const ADMITTED_TIME_SLEEP_RETURN: &str = "fn main() -> Null { time::sleep(0) }\n";

// TASK-2013/TASK-2014's first admitted abortive-handler slice. Its clause
// deliberately does not invoke `resume`; the fixed division must become an
// Engine-owned, post-admission language trap rather than an admission error.
const ADMITTED_TRAP_SLEEP: &str = r#"
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
"#;

// This remains syntactically and type-correctly a `trap_sleep` program, so a
// lexical candidate selector must not turn the changed operation argument into
// a type error. It differs from the sealed witness only at the validated Core
// lowering fact: `sleep(1)` has no production token and must reject admission.
const UNADMITTED_TRAP_SLEEP_ARGUMENT: &str = r#"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler trap_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => 1 / 0,
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(1) with trap_sleep }
"#;

// This source remains type-valid and is deliberately selected lexically by
// `handler trap_sleep`. Its second concrete operation clause makes the
// checked handler structurally ineligible before the private Core inspection
// bridge can run: the bounded production token permits exactly one clause.
const STRUCTURALLY_UNADMITTED_TRAP_SLEEP: &str = r#"
interface Clock<T> {
    sleep(Int) -> Int
    wake(Int) -> Int
}
type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    wake(milliseconds) = milliseconds
}
handler trap_sleep(comp: () -> { TestClock::sleep, TestClock::wake } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => 1 / 0,
        TestClock::wake(ms, resume) => ms,
        done(value) => value,
    }
}
fn main() -> Int {
    handle { TestClock::sleep(0); TestClock::wake(0) } with trap_sleep
}
"#;

#[test]
fn run_json_projects_an_admitted_checked_cps_time_sleep_return() {
    let (_temp, source) = write_fixture(
        "checked-cps-time-sleep-return.ash",
        ADMITTED_TIME_SLEEP_RETURN,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .success()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({"schema_version": 1, "kind": "return", "value": null}),
        "the only admitted time::sleep source slice must project the Engine checked-CPS return, not a direct-evaluator value"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_text_projects_an_admitted_checked_cps_time_sleep_return_as_null() {
    let (_temp, source) = write_fixture(
        "checked-cps-time-sleep-return-text.ash",
        ADMITTED_TIME_SLEEP_RETURN,
    );

    let output = ash()
        .arg("run")
        .arg(source)
        .assert()
        .success()
        .get_output()
        .clone();

    assert_eq!(
        output.stdout, b"null\n",
        "the admitted checked-CPS Null return must retain ordinary text Value output rather than disappear behind the JSON-only terminal projection"
    );
    assert!(
        output.stderr.is_empty(),
        "the normal text route must not emit terminal telemetry on stderr"
    );
}

#[test]
fn run_text_writes_an_admitted_checked_cps_time_sleep_return_to_output_file() {
    let (temp, source) = write_fixture(
        "checked-cps-time-sleep-return-output.ash",
        ADMITTED_TIME_SLEEP_RETURN,
    );
    let output_path = temp.path().join("time-sleep-result.txt");

    let output = ash()
        .args(["run", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .success()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "--output exclusively owns the ordinary text result"
    );
    assert_eq!(
        fs::read_to_string(output_path).expect("read checked-CPS text result"),
        "null",
        "the output file must retain the ordinary Value::Null text representation"
    );
}

#[test]
fn run_json_rejects_trace_for_the_admitted_checked_cps_time_sleep_route() {
    let (_temp, source) = write_fixture(
        "checked-cps-time-sleep-trace.ash",
        ADMITTED_TIME_SLEEP_RETURN,
    );

    let output = ash()
        .args(["run", "--trace", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "configuration",
            "message": "trace is not supported for the admitted checked-CPS time::sleep route",
        }),
        "the unchecked trace route must fail before production admission rather than silently omitting an expected trace"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_text_rejects_trace_for_the_admitted_checked_cps_time_sleep_route() {
    let (_temp, source) = write_fixture(
        "checked-cps-time-sleep-trace-text.ash",
        ADMITTED_TIME_SLEEP_RETURN,
    );

    let output = ash()
        .args(["run", "--trace"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the unsupported text trace route must not silently return an untraced value"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("trace is not supported for the admitted checked-CPS time::sleep route"),
        "text mode must report the same explicit pre-entry configuration rejection as JSON mode"
    );
}

#[test]
fn run_json_rejects_trace_for_an_admitted_checked_cps_handler_route() {
    let (_temp, source) = write_fixture("admitted-trap-sleep-trace.ash", ADMITTED_TRAP_SLEEP);

    let output = ash()
        .args(["run", "--trace", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "configuration",
            "message": "trace is not supported for the admitted checked-CPS handler route",
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_text_keeps_helper_time_sleep_out_of_the_production_main_route() {
    let (_temp, source) = write_fixture(
        "helper-time-sleep-is-not-main.ash",
        r#"
        fn helper() -> Null { time::sleep(0) }
        fn main() -> Int { 42 }
        "#,
    );

    let output = ash()
        .arg("run")
        .arg(source)
        .assert()
        .success()
        .get_output()
        .clone();

    assert_eq!(
        output.stdout, b"42\n",
        "a helper operation spelling must not select the closed production time::sleep route or reject an otherwise pure main"
    );
    assert!(
        output.stderr.is_empty(),
        "the ordinary pure-main route must not emit production-route diagnostics"
    );
}

#[test]
fn run_json_projects_an_admitted_checked_cps_time_sleep_timeout() {
    let (_temp, source) = write_fixture(
        "checked-cps-time-sleep-timeout.ash",
        "fn main() -> Null { time::sleep(1500) }\n",
    );

    let output = ash()
        .args(["run", "--format", "json", "--timeout", "1"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "execution",
            "outcome": "timeout",
        }),
        "the CLI must project the Engine run-wide checked-CPS timeout rather than wrapping a legacy evaluator in an outer timeout"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[cfg(all(unix, target_os = "linux"))]
#[test]
fn run_json_projects_cancellation_of_an_admitted_checked_cps_time_sleep() {
    if !tokio_sigint_delivery_available() {
        eprintln!(
            "skipping Ash SIGINT cancellation assertion: isolated Tokio SIGINT probe proved programmatic delivery unavailable in this harness"
        );
        return;
    }

    let (_temp, source) = write_fixture(
        "checked-cps-time-sleep-cancelled.ash",
        "fn main() -> Null { time::sleep(10000) }\n",
    );

    let mut child = ash_with_default_sigint()
        .args(["run", "--format", "json"])
        .arg(source)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("ash binary starts");

    // Do not send the one SIGINT while the test-harness-inherited ignored
    // disposition is still active. Listener registration proves this reaches
    // the CLI cancellation boundary rather than merely testing startup time.
    wait_for_sigint_listener(&mut child);

    let signal_status = ProcessCommand::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("system kill command sends SIGINT to ash");
    assert!(signal_status.success(), "SIGINT delivery must succeed");

    let output = child
        .wait_with_output()
        .expect("cancelled ash process exits");
    assert_eq!(
        output.status.code(),
        Some(130),
        "the existing CLI cancellation exit policy remains authoritative"
    );

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "execution",
            "outcome": "cancelled",
        }),
        "the CLI must project Engine cooperative cancellation through the V1 envelope"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[cfg(all(unix, target_os = "linux"))]
#[test]
fn run_json_writes_admitted_time_sleep_cancellation_to_requested_output_file() {
    if !tokio_sigint_delivery_available() {
        eprintln!(
            "skipping Ash SIGINT cancellation output assertion: isolated Tokio SIGINT probe proved programmatic delivery unavailable in this harness"
        );
        return;
    }

    let (temp, source) = write_fixture(
        "checked-cps-time-sleep-cancelled-output.ash",
        "fn main() -> Null { time::sleep(10000) }\n",
    );
    let output_path = temp.path().join("terminal.json");

    let mut child = ash_with_default_sigint()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("ash binary starts");

    // See the stdout-owner control above: only signal after the child has
    // installed its SIGINT listener.
    wait_for_sigint_listener(&mut child);

    let signal_status = ProcessCommand::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("system kill command sends SIGINT to ash");
    assert!(signal_status.success(), "SIGINT delivery must succeed");

    let output = child
        .wait_with_output()
        .expect("cancelled ash process exits");
    assert_eq!(
        output.status.code(),
        Some(130),
        "the existing CLI cancellation exit policy remains authoritative"
    );
    assert!(
        output.stdout.is_empty(),
        "the requested output file exclusively owns the cancellation terminal envelope"
    );

    let envelope =
        terminal_json(&fs::read(output_path).expect("read cancellation terminal envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "execution",
            "outcome": "cancelled",
        }),
        "the output file must contain the same Engine cooperative cancellation envelope"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_a_successful_entry_as_a_canonical_return_envelope() {
    let (_temp, source) = write_fixture(
        "return.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .success()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({"schema_version": 1, "kind": "return", "value": {}})
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_the_canonical_return_envelope_to_the_requested_output_file() {
    let (temp, source) = write_fixture(
        "return-output.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    );
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .success()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the requested JSON output file owns the terminal envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read terminal envelope"));
    assert_eq!(
        envelope,
        json!({"schema_version": 1, "kind": "return", "value": {}})
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_a_declared_runtime_error_as_a_canonical_trap_envelope() {
    let (_temp, source) = write_fixture(
        "trap.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> {
            Err { error: RuntimeError(42, "boom") }
        }
        "#,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .code(42)
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "trap",
            "reason": "RuntimeError(42, \"boom\")"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_an_invalid_declared_exit_code_from_a_completed_entry_as_a_trap() {
    let (_temp, source) = write_fixture(
        "invalid-exit-code.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> {
            Err { error: RuntimeError(999, "boom") }
        }
        "#,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "trap",
            "reason": "RuntimeError(999, \"boom\")"
        })
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("invalid runtime exit code"),
        "a completed entry must project its retained terminal value, not the host exit-code diagnostic"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_an_invalid_declared_exit_code_trap_exclusively_to_output_file() {
    let (temp, source) = write_fixture(
        "invalid-exit-code-output.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> {
            Err { error: RuntimeError(999, "boom") }
        }
        "#,
    );
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the requested JSON output file exclusively owns the completed-entry trap envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read terminal envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "trap",
            "reason": "RuntimeError(999, \"boom\")"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_checked_but_unlowered_division_as_an_admission_rejection() {
    let (_temp, source) = write_fixture(
        "division-by-zero.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> {
            do {
                1 / 0;
                return Ok { value: {} }
            }
        }
        "#,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "a source division has no sealed checked-CPS lowering yet, so it must reject before any legacy execution trap"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_an_admitted_abortive_handler_division_as_a_post_admission_trap() {
    let (_temp, source) = write_fixture("admitted-trap-sleep.ash", ADMITTED_TRAP_SLEEP);

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .code(5)
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(envelope["schema_version"], json!(1));
    assert_eq!(envelope["kind"], json!("trap"));
    let reason = envelope["reason"]
        .as_str()
        .expect("the canonical handler trap envelope carries a string language reason");
    assert!(
        reason.to_lowercase().contains("division by zero"),
        "the fixed handler-body primitive fault must remain recognizable as a language reason: {envelope}"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_an_admitted_abortive_handler_division_trap_only_to_output() {
    let (temp, source) = write_fixture("admitted-trap-sleep-output.ash", ADMITTED_TRAP_SLEEP);
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .code(5)
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "--output exclusively owns the admitted checked-CPS handler-trap envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read handler-trap envelope"));
    assert_eq!(envelope["schema_version"], json!(1));
    assert_eq!(envelope["kind"], json!("trap"));
    let reason = envelope["reason"]
        .as_str()
        .expect("the canonical handler trap envelope carries a string language reason");
    assert!(
        reason.to_lowercase().contains("division by zero"),
        "the output envelope must preserve the handler-body language trap reason: {envelope}"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_rejects_a_lexical_trap_sleep_candidate_without_the_exact_validated_lowering() {
    let (_temp, source) = write_fixture(
        "unadmitted-trap-sleep-argument.ash",
        UNADMITTED_TRAP_SLEEP_ARGUMENT,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .code(1)
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "a lexical trap_sleep candidate without the exact checked lowering must reject at admission"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_an_unadmitted_lexical_trap_sleep_rejection_only_to_output() {
    let (temp, source) = write_fixture(
        "unadmitted-trap-sleep-argument-output.ash",
        UNADMITTED_TRAP_SLEEP_ARGUMENT,
    );
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .code(1)
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "--output exclusively owns the unadmitted lexical-handler envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read admission envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "the output file must preserve the canonical missing-admission envelope"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_rejects_a_type_valid_lexical_trap_sleep_with_two_checked_clauses_at_admission() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine builds for the structural handler-admission control");
    let mut entry = engine
        .parse(STRUCTURALLY_UNADMITTED_TRAP_SLEEP)
        .expect("two-clause trap_sleep source parses");
    engine
        .check(&mut entry)
        .expect("two-clause trap_sleep source remains type-valid before admission");

    let (_temp, source) = write_fixture(
        "structurally-unadmitted-trap-sleep.ash",
        STRUCTURALLY_UNADMITTED_TRAP_SLEEP,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .code(1)
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "a type-valid lexical trap_sleep candidate with two checked clauses must reject admission before inspection/lowering"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_checked_but_unlowered_division_admission_rejection_to_output_file() {
    let (temp, source) = write_fixture(
        "division-by-zero-output.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> {
            do {
                1 / 0;
                return Ok { value: {} }
            }
        }
        "#,
    );
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the requested JSON output file owns the unlowered-source admission envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read terminal envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "the output file must retain the strict admission boundary rather than a bootstrap trap"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_missing_entry_as_a_canonical_pre_entry_failure_envelope() {
    let (_temp, source) = write_fixture("missing-main.ash", "fn other() -> Int { 0 }\n");

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "entry_verification",
            "message": "entry file has no 'main' entry"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_dry_run_json_projects_a_declaration_only_source_as_an_entry_verification_failure() {
    let (_temp, source) = write_fixture(
        "declaration-only-dry-run.ash",
        "pub type ReviewMarker = Int;\n",
    );

    let output = ash()
        .args(["run", "--dry-run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "entry_verification",
            "message": "entry file has no 'main' entry"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_dry_run_json_writes_a_declaration_only_entry_verification_failure_to_output_file() {
    let (temp, source) = write_fixture(
        "declaration-only-dry-run-output.ash",
        "pub type ReviewMarker = Int;\n",
    );
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--dry-run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the requested JSON output file owns the declaration-only pre-entry envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read terminal envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "entry_verification",
            "message": "entry file has no 'main' entry"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_a_malformed_entry_as_a_canonical_pre_entry_failure_envelope() {
    let (_temp, source) = write_fixture("malformed.ash", "fn main( {\n");

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "parse",
            "message": "entry source could not be parsed"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_an_unreadable_source_as_a_canonical_pre_entry_failure_envelope() {
    let temp = tempdir().expect("temporary fixture root");
    let missing_source = temp.path().join("missing.ash");

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(&missing_source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "input",
            "message": "entry source could not be read"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_malformed_capability_impl_as_a_configuration_pre_entry_failure() {
    let temp = tempdir().expect("temporary fixture root");
    let missing_source = temp.path().join("not-read.ash");

    let output = ash()
        .args(["run", "--format", "json", "--capability-impl", "malformed"])
        .arg(&missing_source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "configuration",
            "message": "run configuration is invalid"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_malformed_capability_impl_configuration_failure_to_requested_output_file() {
    let temp = tempdir().expect("temporary fixture root");
    let missing_source = temp.path().join("not-read.ash");
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args([
            "run",
            "--format",
            "json",
            "--output",
            output_path.to_str().expect("output path is UTF-8"),
            "--capability-impl",
            "malformed",
        ])
        .arg(&missing_source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the requested JSON output file owns the configuration pre-entry envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read terminal envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "configuration",
            "message": "run configuration is invalid"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_unknown_capability_impl_selection_as_a_configuration_pre_entry_failure() {
    let (_temp, source) = write_fixture(
        "unknown-capability-impl.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    );

    let output = ash()
        .args([
            "run",
            "--format",
            "json",
            "--capability-impl",
            "binding=missing_impl",
        ])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "configuration",
            "message": "run configuration is invalid"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_unknown_capability_impl_selection_configuration_failure_to_requested_output_file()
 {
    let (temp, source) = write_fixture(
        "unknown-capability-impl-output.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    );
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args([
            "run",
            "--format",
            "json",
            "--output",
            output_path.to_str().expect("output path is UTF-8"),
            "--capability-impl",
            "binding=missing_impl",
        ])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the requested JSON output file owns the source-aware configuration pre-entry envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read terminal envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "configuration",
            "message": "run configuration is invalid"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_an_unreadable_source_pre_entry_failure_to_the_requested_output_file() {
    let temp = tempdir().expect("temporary fixture root");
    let missing_source = temp.path().join("missing.ash");
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(&missing_source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "the requested JSON output file owns the pre-entry envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read terminal envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "input",
            "message": "entry source could not be read"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_an_unbound_entry_name_as_a_canonical_pre_entry_failure_envelope() {
    let (_temp, source) = write_fixture(
        "type-error.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { missing_name }
        "#,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "typecheck",
            "message": "entry failed type checking"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_a_wrong_entry_contract_as_a_canonical_pre_entry_failure_envelope() {
    let (_temp, source) = write_fixture("wrong-entry-contract.ash", "fn main() -> Int { 0 }\n");

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "entry_verification",
            "message": "entry contract verification failed"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_rejected_host_admission_as_a_bounded_external_outcome() {
    let (_temp, source) = write_fixture(
        "admission.ash",
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    );

    let output = ash()
        .args(["run", "--format", "json", "--admission-profile", "reject"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected"
        })
    );
    assert_no_implementation_telemetry(&envelope);
}

// This is a checked runtime-entry shape whose nested arithmetic remains
// outside the sealed checked Core/CPS production lowering. It must reject at
// admission rather than reach a legacy evaluator or be relabelled as a
// parse/type failure.
const CHECKED_BUT_UNLOWERED_RUNTIME_ENTRY: &str = r#"
use result::Result
use runtime::RuntimeError

fn main() -> Result<(), RuntimeError> {
    Err { error: RuntimeError((1 + 2) + 3, "boom") }
}
"#;

// Keep this deliberately different from the nested-arithmetic control above:
// it has a local callable in the constructor field, so a syntactic
// constructor/arithmetic guard cannot accidentally stand in for the typed
// closed-admission decision.
const CHECKED_BUT_UNLOWERED_LOCAL_CALL_RUNTIME_ENTRY: &str = r#"
use result::Result
use runtime::RuntimeError

fn local_status() -> Int { 7 }

fn main() -> Result<(), RuntimeError> {
    Err { error: RuntimeError(local_status(), "boom") }
}
"#;

#[test]
fn run_json_projects_checked_but_unlowered_source_as_an_admission_rejection() {
    let (_temp, source) = write_fixture(
        "checked-but-unlowered-production-entry.ash",
        CHECKED_BUT_UNLOWERED_RUNTIME_ENTRY,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "checked-but-unlowered source must reject at the strict production admission boundary"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_checked_local_call_without_lowering_as_an_admission_rejection() {
    let (_temp, source) = write_fixture(
        "checked-but-unlowered-local-call-production-entry.ash",
        CHECKED_BUT_UNLOWERED_LOCAL_CALL_RUNTIME_ENTRY,
    );

    let output = ash()
        .args(["run", "--format", "json"])
        .arg(source)
        .assert()
        .code(1)
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "every checked but unlowered entry must reject at admission instead of reaching the legacy bootstrap evaluator"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_writes_checked_but_unlowered_admission_rejection_to_output_file() {
    let (temp, source) = write_fixture(
        "checked-but-unlowered-production-entry-output.ash",
        CHECKED_BUT_UNLOWERED_RUNTIME_ENTRY,
    );
    let output_path = temp.path().join("terminal.json");

    let output = ash()
        .args(["run", "--format", "json", "--output"])
        .arg(&output_path)
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        output.stdout.is_empty(),
        "--output exclusively owns the checked-but-unlowered admission envelope"
    );
    let envelope = terminal_json(&fs::read(output_path).expect("read admission envelope"));
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected",
        }),
        "the requested output file must contain the canonical admission rejection envelope"
    );
    assert_no_implementation_telemetry(&envelope);
}

#[test]
fn run_json_projects_an_unlowered_sleep_entry_as_an_admission_rejection_before_timeout() {
    let (_temp, source) = write_fixture(
        "timeout.ash",
        r#"
        use time::{sleep}
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> {
            do {
                sleep(1000);
                return Ok { value: {} }
            }
        }
        "#,
    );

    let output = ash()
        .args(["run", "--format", "json", "--timeout", "0"])
        .arg(source)
        .assert()
        .failure()
        .get_output()
        .clone();

    let envelope = terminal_json(&output.stdout);
    assert_eq!(
        envelope,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected"
        }),
        "an unsupported Result-returning sleep entry must reject before a timeout can relabel it as execution"
    );
    assert_no_implementation_telemetry(&envelope);
}
