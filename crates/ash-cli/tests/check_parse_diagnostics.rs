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

fn removed_observe_with_source() -> String {
    let observe = ["ob", "serve"].concat();
    let with = ["wi", "th"].concat();
    format!("fn main() {{\n  {observe} Sensor.read {with} timeout: 10;\n}}\n")
}

fn removed_act_with_source() -> String {
    let act = ["a", "ct"].concat();
    let with = ["wi", "th"].concat();
    format!("fn main() {{\n  {act} Email.send {with} retry: true;\n}}\n")
}

#[test]
fn stale_if_without_then_gets_targeted_diagnostic() {
    let output = ash_check_output("fn main() {\n  if ready {\n    {};\n  }\n}\n");

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("if condition { ... }"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_for_in_loop_gets_targeted_diagnostic() {
    let output = ash_check_output("fn main() {\n  for item in items {\n    {};\n  }\n}\n");

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("for item in items { ... }"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_decide_else_gets_targeted_diagnostic() {
    let output = ash_check_output("fn main() {\n  decide approve else deny;\n}\n");

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("decide ... else"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_observe_with_gets_targeted_diagnostic() {
    let output = ash_check_output(&removed_observe_with_source());

    assert!(output.contains("DeprecatedSyntaxMigration"), "{output}");
    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("removed-observe-with"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_act_with_gets_targeted_diagnostic() {
    let output = ash_check_output(&removed_act_with_source());

    assert!(output.contains("DeprecatedSyntaxMigration"), "{output}");
    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("removed-act-with"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_with_role_gets_targeted_diagnostic() {
    let act = ["a", "ct"].concat();
    let with = ["wi", "th"].concat();
    let output = ash_check_output(&format!(
        "fn main() {{\n  {act} Email.send {with} role: admin;\n}}\n"
    ));

    assert!(output.contains("DeprecatedSyntaxMigration"), "{output}");
    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("with role:"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_observe_with_json_diagnostic_carries_migration_metadata() {
    let json = ash_check_json(&removed_observe_with_source());

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
            .contains("removed-observe-with"),
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
            .contains("Sensor.read"),
        "json={json}"
    );
    assert!(
        diagnostic["help"]
            .as_str()
            .unwrap_or_default()
            .contains("removed observe form"),
        "json={json}"
    );
}

#[test]
fn stale_act_with_json_diagnostic_carries_migration_metadata() {
    let json = ash_check_json(&removed_act_with_source());

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
            .contains("removed-act-with"),
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
            .contains("Email.send"),
        "json={json}"
    );
    assert!(
        diagnostic["help"]
            .as_str()
            .unwrap_or_default()
            .contains("removed act form"),
        "json={json}"
    );
}

#[test]
fn removed_fat_callable_arrow_in_list_type_gets_targeted_diagnostic() {
    let output = ash_check_output("fn f(x: [Int => Bool]) -> Bool { true }\nfn main() { true }\n");

    assert!(output.contains("removed-callable-arrow"), "{output}");
    assert!(output.contains("=>"), "{output}");
    assert!(output.contains("pure callable arrow `->`"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn removed_fat_callable_arrow_json_diagnostic_carries_migration_metadata() {
    let json = ash_check_json("fn f(x: [Int => Bool]) -> Bool { true }\nfn main() { true }\n");

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
            .contains("removed-callable-arrow"),
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
fn removed_dash_star_callable_arrow_in_type_gets_targeted_diagnostic() {
    let output = ash_check_output("fn f(x: [Int -*> Bool]) -> Bool { true }\nfn main() { true }\n");

    assert!(output.contains("removed-callable-arrow"), "{output}");
    assert!(output.contains("-*>"), "{output}");
    assert!(output.contains("pure callable arrow `->`"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn removed_equals_star_callable_arrow_in_type_gets_targeted_diagnostic() {
    let output = ash_check_output("fn f(x: [Int =*> Bool]) -> Bool { true }\nfn main() { true }\n");

    assert!(output.contains("removed-callable-arrow"), "{output}");
    assert!(output.contains("=*>"), "{output}");
    assert!(output.contains("pure callable arrow `->`"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn pure_callable_arrow_does_not_get_reserved_arrow_diagnostic() {
    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["check", "-"])
        .write_stdin("fn f(x: [Int -> Bool]) -> Bool { true }\nfn main() { true }\n")
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
