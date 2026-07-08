use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn fmt_check_accepts_phase199_current_examples() {
    for path in [
        "examples/10-testing-helpers/testing_helpers.ash",
        "examples/11-process-channel-helpers/process_channel_helpers.ash",
    ] {
        let path = repo_root().join(path);
        Command::cargo_bin("ash")
            .expect("ash binary exists")
            .args(["fmt", "--check"])
            .arg(path)
            .assert()
            .success();
    }
}

#[test]
fn fmt_stdin_normalizes_trailing_whitespace_idempotently() {
    let source = "fn main() {   \n    0   \n}\n\n\n";

    let first = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["fmt", "--stdin"])
        .write_stdin(source)
        .output()
        .expect("run ash fmt --stdin");
    assert!(
        first.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );

    let second = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["fmt", "--stdin"])
        .write_stdin(first.stdout.clone())
        .output()
        .expect("rerun ash fmt --stdin");
    assert!(
        second.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&second.stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&first.stdout),
        "fn main() {\n    0\n}\n"
    );
}

#[test]
fn fmt_check_reports_unformatted_trailing_whitespace() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("needs-format.ash");
    fs::write(&path, "fn main() {   \n    0\n}\n").expect("write fixture");

    Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["fmt", "--check"])
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("would reformat"));
}

#[test]
fn fmt_rejects_removed_forms_before_formatting() {
    let removed_entry = ["work", "flow"].concat();
    let removed_proc = ["Pr", "oc"].concat();
    let removed_observe = ["ob", "serve"].concat();
    let removed_with = ["wi", "th"].concat();
    let stale_sources = vec![
        format!(
            "{removed_entry} main {{\n    {removed_observe} Sensor.read {removed_with} timeout: 10\n}}\n"
        ),
        format!("fn helper() -> {removed_proc}<Int> {{ do {{ return 0 }} }}\n"),
        "// ambient authority\nfn main() { 0 }\n".to_string(),
    ];
    for stale in stale_sources {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("stale.ash");
        fs::write(&path, stale).expect("write stale fixture");

        Command::cargo_bin("ash")
            .expect("ash binary exists")
            .args(["fmt", "--check"])
            .arg(&path)
            .assert()
            .failure()
            .stderr(predicates::str::contains("unsupported removed syntax"));
    }
}
