use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn ash_check_output(source: &str) -> String {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("invalid.ash");
    fs::write(&path, source).expect("write invalid ash fixture");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("check")
        .arg(&path)
        .output()
        .expect("run ash check");

    assert!(!output.status.success(), "invalid syntax should fail");

    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn ash_check_json(source: &str) -> Value {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("invalid.ash");
    fs::write(&path, source).expect("write invalid ash fixture");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["check", "--format", "json"])
        .arg(&path)
        .output()
        .expect("run ash check json");

    assert!(!output.status.success(), "invalid syntax should fail");

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!(
            "stdout should be JSON: {error}\nstdout={}\nstderr={}",
            stdout,
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn stale_if_without_then_gets_targeted_diagnostic() {
    let output = ash_check_output("workflow main() {\n  if ready {\n    done;\n  }\n}\n");

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("if condition { ... }"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_for_in_loop_gets_targeted_diagnostic() {
    let output = ash_check_output("workflow main() {\n  for item in items {\n    done;\n  }\n}\n");

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("for item in items { ... }"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_decide_else_gets_targeted_diagnostic() {
    let output = ash_check_output("workflow main() {\n  decide approve else deny;\n}\n");

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("decide ... else"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_observe_with_gets_targeted_diagnostic() {
    let output =
        ash_check_output("workflow main() {\n  observe Sensor.read with timeout: 10;\n}\n");

    assert!(output.contains("DeprecatedSyntaxMigration"), "{output}");
    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("observe ... with"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_act_with_gets_targeted_diagnostic() {
    let output = ash_check_output("workflow main() {\n  act Email.send with retry: true;\n}\n");

    assert!(output.contains("DeprecatedSyntaxMigration"), "{output}");
    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("act ... with"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_with_role_gets_targeted_diagnostic() {
    let output = ash_check_output("workflow main() {\n  act Email.send with role: admin;\n}\n");

    assert!(output.contains("DeprecatedSyntaxMigration"), "{output}");
    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("with role:"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_observe_with_json_diagnostic_carries_migration_metadata() {
    let json = ash_check_json("workflow main() {\n  observe Sensor.read with timeout: 10;\n}\n");

    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    assert_eq!(diagnostics.len(), 1, "json={json}");
    let diagnostic = &diagnostics[0];

    assert_eq!(
        diagnostic["code"].as_str(),
        Some("DeprecatedSyntaxMigration"),
        "json={json}"
    );
    assert!(
        diagnostic["message"]
            .as_str()
            .unwrap_or_default()
            .contains("observe ... with"),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["line"].as_u64(),
        Some(2),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["column"].as_u64(),
        Some(3),
        "json={json}"
    );
    assert!(
        diagnostic["context"]
            .as_str()
            .unwrap_or_default()
            .contains("observe Sensor.read with timeout"),
        "json={json}"
    );
    assert!(
        diagnostic["help"]
            .as_str()
            .unwrap_or_default()
            .contains("current observe statements"),
        "json={json}"
    );
}

#[test]
fn stale_act_with_json_diagnostic_carries_migration_metadata() {
    let json = ash_check_json("workflow main() {\n  act Email.send with retry: true;\n}\n");

    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    assert_eq!(diagnostics.len(), 1, "json={json}");
    let diagnostic = &diagnostics[0];

    assert_eq!(
        diagnostic["code"].as_str(),
        Some("DeprecatedSyntaxMigration"),
        "json={json}"
    );
    assert!(
        diagnostic["message"]
            .as_str()
            .unwrap_or_default()
            .contains("act ... with"),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["line"].as_u64(),
        Some(2),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["column"].as_u64(),
        Some(3),
        "json={json}"
    );
    assert!(
        diagnostic["context"]
            .as_str()
            .unwrap_or_default()
            .contains("act Email.send with retry"),
        "json={json}"
    );
    assert!(
        diagnostic["help"]
            .as_str()
            .unwrap_or_default()
            .contains("current act statements"),
        "json={json}"
    );
}

#[test]
fn reserved_proc_callable_arrow_in_list_type_gets_targeted_diagnostic() {
    let output =
        ash_check_output("fn f(x: [Int => Bool]) -> Bool { true }\nworkflow main { ret true }\n");

    assert!(
        output.contains("Proc callable syntax is reserved"),
        "{output}"
    );
    assert!(output.contains("=>"), "{output}");
    assert!(output.contains("pure callable arrow `->`"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn reserved_proc_callable_arrow_json_diagnostic_carries_migration_metadata() {
    let json =
        ash_check_json("fn f(x: [Int => Bool]) -> Bool { true }\nworkflow main { ret true }\n");

    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    assert_eq!(diagnostics.len(), 1, "json={json}");
    let diagnostic = &diagnostics[0];

    assert_eq!(
        diagnostic["code"].as_str(),
        Some("DeprecatedSyntaxMigration"),
        "json={json}"
    );
    assert!(
        diagnostic["message"]
            .as_str()
            .unwrap_or_default()
            .contains("Proc callable syntax is reserved"),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["line"].as_u64(),
        Some(1),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["column"].as_u64(),
        Some(14),
        "json={json}"
    );
    assert!(
        diagnostic["context"]
            .as_str()
            .unwrap_or_default()
            .contains("fn f(x: [Int => Bool])"),
        "json={json}"
    );
    assert!(
        diagnostic["help"]
            .as_str()
            .unwrap_or_default()
            .contains("pure callable arrow `->`"),
        "json={json}"
    );
}

#[test]
fn reserved_act_callable_arrow_in_type_gets_targeted_diagnostic() {
    let output =
        ash_check_output("fn f(x: [Int -*> Bool]) -> Bool { true }\nworkflow main { ret true }\n");

    assert!(
        output.contains("Act callable syntax is reserved"),
        "{output}"
    );
    assert!(output.contains("-*>"), "{output}");
    assert!(output.contains("pure callable arrow `->`"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn reserved_workflow_callable_arrow_in_type_gets_targeted_diagnostic() {
    let output =
        ash_check_output("fn f(x: [Int =*> Bool]) -> Bool { true }\nworkflow main { ret true }\n");

    assert!(
        output.contains("Workflow callable syntax is reserved"),
        "{output}"
    );
    assert!(output.contains("=*>"), "{output}");
    assert!(output.contains("pure callable arrow `->`"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn pure_callable_arrow_does_not_get_reserved_arrow_diagnostic() {
    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["check", "-"])
        .write_stdin("fn f(x: [Int -> Bool]) -> Bool { true }\nworkflow main { ret true }\n")
        .output()
        .expect("run ash check");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !combined.contains("callable syntax is reserved"),
        "{combined}"
    );
}
