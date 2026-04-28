//! TASK-760: CLI-level `ash check` baseline harness for the standard library corpus.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedFailure {
    path: &'static str,
    reason: &'static str,
}

const EXPECTED_STD_FILES: usize = 39;
const EXPECTED_STD_PASSING: usize = 33;
const EXPECTED_STD_FAILING: usize = 6;

const EXPECTED_PASS: &[&str] = &[
    "std/src/act.ash",
    "std/src/http.ash",
    "std/src/io/buf.ash",
    "std/src/io/dir.ash",
    "std/src/io/fs.ash",
    "std/src/io/meta.ash",
    "std/src/io/mod.ash",
    "std/src/io/path.ash",
    "std/src/io/stdio.ash",
    "std/src/json.ash",
    "std/src/lib.ash",
    "std/src/list.ash",
    "std/src/llm/dispatch.ash",
    "std/src/llm/mod.ash",
    "std/src/llm/openai.ash",
    "std/src/llm/prompt.ash",
    "std/src/llm/types.ash",
    "std/src/map.ash",
    "std/src/markdown.ash",
    "std/src/option.ash",
    "std/src/predicate.ash",
    "std/src/prelude.ash",
    "std/src/proc.ash",
    "std/src/process.ash",
    "std/src/record.ash",
    "std/src/regex.ash",
    "std/src/result.ash",
    "std/src/runtime/args.ash",
    "std/src/runtime/error.ash",
    "std/src/runtime/mod.ash",
    "std/src/string.ash",
    "std/src/test.ash",
    "std/src/time.ash",
];

const EXPECTED_FAIL: &[ExpectedFailure] = &[
    ExpectedFailure {
        path: "std/src/llm/conversation.ash",
        reason: "workflow export visibility/importability cannot resolve dispatch::complete",
    },
    ExpectedFailure {
        path: "std/src/llm/loading.ash",
        reason: "imports path/fs from the wrong nested std module surface",
    },
    ExpectedFailure {
        path: "std/src/llm/router.ash",
        reason: "workflow export visibility/importability cannot resolve dispatch::complete",
    },
    ExpectedFailure {
        path: "std/src/llm/supervised.ash",
        reason: "workflow export visibility/importability cannot resolve dispatch::complete",
    },
    ExpectedFailure {
        path: "std/src/llm/tool_agent.ash",
        reason: "workflow export visibility/importability cannot resolve dispatch::complete_with_tools",
    },
    ExpectedFailure {
        path: "std/src/runtime/supervisor.ash",
        reason: "relative super:: imports are treated as literal module names",
    },
];

const REFERENCE_ONLY: &[&str] = &[];

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
    visit(root, &root.join("std/src"), &mut files);
    files.sort();
    files
}

fn sorted_classified_files() -> Vec<&'static str> {
    let mut files = Vec::new();
    files.extend_from_slice(EXPECTED_PASS);
    files.extend(EXPECTED_FAIL.iter().map(|failure| failure.path));
    files.extend_from_slice(REFERENCE_ONLY);
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
fn stdlib_corpus_cli_check_baseline_is_classified_and_honest() {
    let repo = repo_root();
    let discovered = collect_ash_files(&repo);
    let classified = sorted_classified_files();

    assert_eq!(discovered.len(), EXPECTED_STD_FILES);
    assert_eq!(EXPECTED_PASS.len(), EXPECTED_STD_PASSING);
    assert_eq!(EXPECTED_FAIL.len(), EXPECTED_STD_FAILING);
    assert_eq!(REFERENCE_ONLY.len(), 0);
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
            "expected-pass std file failed `ash check`: {relative}\nstdout={stdout}\nstderr={stderr}"
        );
        passed += 1;
    }

    for failure in EXPECTED_FAIL {
        let (success, stdout, stderr) = run_ash_check(&repo, failure.path);
        assert!(
            !success,
            "expected-fail std file unexpectedly passed `ash check`: {}\nreason={}\nstdout={}\nstderr={}",
            failure.path, failure.reason, stdout, stderr
        );
        failed += 1;
    }

    println!(
        "TASK-760 std/src corpus baseline: files={EXPECTED_STD_FILES}, pass={passed}, fail={failed}, reference_only={}",
        REFERENCE_ONLY.len()
    );
    assert_eq!(passed, EXPECTED_STD_PASSING);
    assert_eq!(failed, EXPECTED_STD_FAILING);
}
