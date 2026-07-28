//! Integration tests for `ash trace` observable output.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

const MISSING_TYPED_LOWERING_ERROR: &str = "application execution failed: checked Core/CPS admission rejected: no validated production typed lowering is available";
const PROVIDER_TRACE_REJECTION: &str =
    "trace is not supported for the admitted checked-CPS time::sleep route";
const HANDLER_TRACE_REJECTION: &str =
    "trace is not supported for the admitted checked-CPS handler route";

const ADMITTED_TIME_SLEEP: &str = "fn main() -> Null { time::sleep(0) }\n";
const ADMITTED_TRAP_SLEEP: &str = r"
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

#[test]
fn trace_stdout_emits_a_document_for_an_admitted_pure_return() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("main.ash");
    fs::write(&entry_path, "fn main() -> Int { 42 }\n").expect("write entry");

    Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("trace")
        .arg(&entry_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"final_value\": \"42\""))
        .stdout(predicate::str::contains("\"trace_id\""))
        .stderr(predicate::str::is_empty());
}

#[test]
fn trace_stdout_rejects_generic_source_without_emitting_a_document() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("main.ash");
    fs::write(&entry_path, "fn main() { 0 }\n").expect("write entry");

    Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("trace")
        .arg(&entry_path)
        .assert()
        .code(5)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(MISSING_TYPED_LOWERING_ERROR));
}

#[test]
fn trace_output_file_is_not_created_when_generic_source_is_not_admitted() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("main.ash");
    let output_path = temp.path().join("trace.json");
    fs::write(&entry_path, "fn main() { 1 }\n").expect("write entry");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("trace")
        .arg(&entry_path)
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .code(5)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(MISSING_TYPED_LOWERING_ERROR));

    assert!(
        !output_path.exists(),
        "closed admission must not emit a partial trace file"
    );
}

fn assert_non_traceable_admitted_route_is_rejected_before_trace_output(
    fixture_name: &str,
    source: &str,
    expected_error: &str,
) {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join(fixture_name);
    let output_path = temp.path().join("trace.json");
    fs::write(&entry_path, source).expect("write admitted non-pure source");

    let mut command = Command::cargo_bin("ash").expect("ash binary exists");
    command
        .arg("trace")
        .arg(&entry_path)
        .arg("--output")
        .arg(&output_path);

    command
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(expected_error));
    assert!(
        !output_path.exists(),
        "an Engine trace-policy rejection must happen before a trace document is written"
    );
}

#[test]
fn trace_rejects_an_admitted_provider_route_before_output() {
    assert_non_traceable_admitted_route_is_rejected_before_trace_output(
        "admitted-time-sleep.ash",
        ADMITTED_TIME_SLEEP,
        PROVIDER_TRACE_REJECTION,
    );
}

#[test]
fn trace_rejects_an_admitted_handler_route_before_output() {
    assert_non_traceable_admitted_route_is_rejected_before_trace_output(
        "admitted-trap-sleep.ash",
        ADMITTED_TRAP_SLEEP,
        HANDLER_TRACE_REJECTION,
    );
}
