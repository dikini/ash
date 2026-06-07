#![allow(missing_docs)]

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn algebra_examples_cli_checks_option_result_monoid_and_tower_imports() {
    let project = tempdir().expect("project");
    let main = project.path().join("algebra_examples.ash");
    std::fs::write(
        &main,
        r#"use algebra::functor::{map_option, map_result, map_list}
use algebra::applicative::{pure_option, apply_option, pure_result, apply_result}
use algebra::monad::{unit_option, bind_option, unit_result, bind_result}
use algebra::monoid::{concat_string, concat_list}
use act::{unit, then}

workflow main { ret 0 }
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
