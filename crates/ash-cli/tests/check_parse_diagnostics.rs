use assert_cmd::Command;
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

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("observe ... with"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
}

#[test]
fn stale_with_role_gets_targeted_diagnostic() {
    let output = ash_check_output("workflow main() {\n  act Email.send with role: admin;\n}\n");

    assert!(output.contains("unsupported stale syntax"), "{output}");
    assert!(output.contains("with role:"), "{output}");
    assert!(!output.contains("ContextError"), "{output}");
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
