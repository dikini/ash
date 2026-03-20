//! Integration tests for the CLI REPL entrypoint.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn repl_help_exposes_canonical_session_flags() {
    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("repl").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--history"))
        .stdout(predicate::str::contains("--no-history"))
        .stdout(predicate::str::contains("Start interactive REPL"))
        .stdout(predicate::str::contains(".ash_history").not());
}
