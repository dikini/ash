#![allow(missing_docs)]

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn algebra_examples_cli_checks_interfaces_and_carrier_owned_helpers() {
    let project = tempdir().expect("project");
    let main = project.path().join("algebra_examples.ash");
    std::fs::write(
        &main,
        r#"use algebra::{Functor, Applicative, Monad, Monoid}
use algebra::comonad::{Comonad}
use string::{concat}

workflow main { ret concat("ok", "!") }
"#,
    )
    .expect("write example");

    let output = Command::cargo_bin("ash")
        .expect("ash binary")
        .arg("check")
        .arg(&main)
        .output()
        .expect("run ash check");

    assert!(
        output.status.success(),
        "ash check failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
