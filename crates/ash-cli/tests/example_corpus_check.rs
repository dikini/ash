//! TASK-760: CLI-level `ash check` baseline harness for the example corpus.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedFailure {
    path: &'static str,
    reason: &'static str,
}

const EXPECTED_EXAMPLE_FILES: usize = 2;
const EXPECTED_EXAMPLE_PASSING: usize = 2;
const EXPECTED_EXAMPLE_FAILING: usize = 0;
const EXPECTED_EXAMPLE_REFERENCE_ONLY: usize = 0;

const EXPECTED_PASS: &[&str] = &[
    "examples/10-testing-helpers/testing_helpers.ash",
    "examples/11-process-channel-helpers/process_channel_helpers.ash",
];

const EXPECTED_FAIL: &[ExpectedFailure] = &[];

const REFERENCE_ONLY: &[ExpectedFailure] = &[];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_ash_files(root: &Path) -> Vec<String> {
    fn visit(base: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("read corpus directory") {
            let entry = entry.expect("read corpus entry");
            let path = entry.path();
            if path.is_dir() {
                visit(base, &path, out);
            } else if path.extension().is_some_and(|ext| ext == "ash") {
                out.push(
                    path.strip_prefix(base)
                        .expect("corpus file should be under repository root")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }

    let mut files = Vec::new();
    visit(root, &root.join("examples"), &mut files);
    files.sort();
    files
}

fn sorted_classified_files() -> Vec<&'static str> {
    let mut files = Vec::new();
    files.extend_from_slice(EXPECTED_PASS);
    files.extend(EXPECTED_FAIL.iter().map(|failure| failure.path));
    files.extend(REFERENCE_ONLY.iter().map(|reference| reference.path));
    files.sort_unstable();
    files
}

fn run_ash_check(repo: &Path, relative: &str) -> (bool, String, String) {
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo)
        .args(["check", relative])
        .output()
        .expect("ash CLI should launch");

    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn example_corpus_cli_check_baseline_is_classified_and_honest() {
    let repo = repo_root();
    let discovered = collect_ash_files(&repo);
    let classified = sorted_classified_files();

    assert_eq!(discovered.len(), EXPECTED_EXAMPLE_FILES);
    assert_eq!(EXPECTED_PASS.len(), EXPECTED_EXAMPLE_PASSING);
    assert_eq!(EXPECTED_FAIL.len(), EXPECTED_EXAMPLE_FAILING);
    assert_eq!(REFERENCE_ONLY.len(), EXPECTED_EXAMPLE_REFERENCE_ONLY);
    assert_eq!(classified, discovered);

    for failure in EXPECTED_FAIL {
        assert!(
            !failure.reason.trim().is_empty(),
            "{} must carry an expected-fail reason",
            failure.path
        );
    }

    let mut passed = 0;
    let mut failed = 0;

    for relative in EXPECTED_PASS {
        let (success, stdout, stderr) = run_ash_check(&repo, relative);
        assert!(
            success,
            "expected-pass example failed `ash check`: {relative}\nstdout={stdout}\nstderr={stderr}"
        );
        passed += 1;
    }

    for failure in EXPECTED_FAIL {
        let (success, stdout, stderr) = run_ash_check(&repo, failure.path);
        assert!(
            !success,
            "expected-fail example unexpectedly passed `ash check`: {}\nreason={}\nstdout={}\nstderr={}",
            failure.path, failure.reason, stdout, stderr
        );
        failed += 1;
    }

    println!(
        "TASK-760 examples corpus baseline: files={EXPECTED_EXAMPLE_FILES}, pass={passed}, fail={failed}, reference_only={}",
        REFERENCE_ONLY.len()
    );
    assert_eq!(passed, EXPECTED_EXAMPLE_PASSING);
    assert_eq!(failed, EXPECTED_EXAMPLE_FAILING);
    assert_eq!(REFERENCE_ONLY.len(), EXPECTED_EXAMPLE_REFERENCE_ONLY);
}
