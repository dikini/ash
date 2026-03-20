//! Integration tests for the standalone `ash-repl` binary surface.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn standalone_binary_starts_with_canonical_banner_and_clean_stderr() {
    let mut cmd = Command::cargo_bin("ash-repl").expect("ash-repl binary exists");
    cmd.write_stdin("");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Ash REPL - Type :help for help, :quit to exit",
        ))
        .stdout(predicate::str::contains("exit"))
        .stderr(predicate::str::is_empty());
}
