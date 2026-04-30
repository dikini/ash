//! TASK-760: CLI-level `ash check` baseline harness for the example corpus.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedFailure {
    path: &'static str,
    reason: &'static str,
}

const EXPECTED_EXAMPLE_FILES: usize = 42;
const EXPECTED_EXAMPLE_PASSING: usize = 31;
const EXPECTED_EXAMPLE_FAILING: usize = 0;
const EXPECTED_EXAMPLE_REFERENCE_ONLY: usize = 11;

const EXPECTED_PASS: &[&str] = &[
    "examples/01-basics/01-hello-world.ash",
    "examples/01-basics/02-variables.ash",
    "examples/01-basics/03-expressions.ash",
    "examples/01-basics/04-observe.ash",
    "examples/02-control-flow/01-conditionals.ash",
    "examples/02-control-flow/02-foreach.ash",
    "examples/02-control-flow/03-sequential.ash",
    "examples/02-control-flow/04-sequential.ash",
    "examples/03-io/directory_listing.ash",
    "examples/03-io/file_read_write.ash",
    "examples/03-io/path_operations.ash",
    "examples/05-phase98/01-fail-with-error.ash",
    "examples/05-phase98/02-proc-par-await-join.ash",
    "examples/05-phase98/03-proc-scatter-gather.ash",
    "examples/05-phase98/04-workflow-boundary-reporting.ash",
    "examples/06-capability-implementations/01-mock-internal-kv.ash",
    "examples/06-capability-implementations/02-caching-kv-adapter.ash",
    "examples/06-capability-implementations/03-recording-replay-sketch.ash",
    "examples/07-phase105/01-do-act.ash",
    "examples/07-phase105/02-act-sugar.ash",
    "examples/07-phase105/03-do-proc-from-act.ash",
    "examples/08-phase106/01-act-comprehension.ash",
    "examples/08-phase106/02-proc-comprehension-from-act.ash",
    "examples/08-phase106/03-deferred-pure-targets.ash",
    "examples/09-phase108/01-do-workflow-unit.ash",
    "examples/09-phase108/02-do-workflow-contract-statements.ash",
    "examples/09-phase108/05-workflow-comprehension.ash",
    "examples/09-phase108/06-legacy-workflow-migration-warning.ash",
    "examples/code_review.ash",
    "examples/entrypoint_args.ash",
    "examples/entrypoint_minimal.ash",
];

const EXPECTED_FAIL: &[ExpectedFailure] = &[];

const REFERENCE_ONLY: &[ExpectedFailure] = &[
    ExpectedFailure {
        path: "examples/03-policies/01-role-based.ash",
        reason: "historical policy sketch syntax is not accepted by the current parser",
    },
    ExpectedFailure {
        path: "examples/03-policies/02-time-based.ash",
        reason: "historical policy sketch syntax is not accepted by the current parser",
    },
    ExpectedFailure {
        path: "examples/04-real-world/code-review.ash",
        reason: "older real-world sketch uses historical workflow/policy syntax",
    },
    ExpectedFailure {
        path: "examples/04-real-world/customer-support.ash",
        reason: "older real-world sketch uses historical workflow/policy syntax",
    },
    ExpectedFailure {
        path: "examples/09-phase108/03-workflow-algebra-intrinsics.reference.ash",
        reason: "workflow algebra intrinsic source-file examples remain reference-only until full parse_file elaboration is available",
    },
    ExpectedFailure {
        path: "examples/09-phase108/04-workflow-explicit-lifts.reference.ash",
        reason: "explicit lower-tower Workflow lift examples remain reference-only until full source-file first-class workflow expression elaboration is available",
    },
    ExpectedFailure {
        path: "examples/multi_agent_research.ash",
        reason: "older workflow sketch uses historical syntax outside current parser support",
    },
    ExpectedFailure {
        path: "examples/simple_workflow.ash",
        reason: "older workflow sketch uses historical syntax outside current parser support",
    },
    ExpectedFailure {
        path: "examples/support_ticket.ash",
        reason: "older workflow sketch uses historical syntax outside current parser support",
    },
    ExpectedFailure {
        path: "examples/workflows/40_tdd_workflow.ash",
        reason: "older TDD workflow sketch uses historical syntax outside current parser support",
    },
    ExpectedFailure {
        path: "examples/workflows/40a_tdd_concrete_example.ash",
        reason: "older TDD workflow sketch uses historical syntax outside current parser support",
    },
];

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

    for reference in REFERENCE_ONLY {
        assert!(
            !reference.reason.trim().is_empty(),
            "{} must carry a reference-only reason",
            reference.path
        );

        let source = std::fs::read_to_string(repo.join(reference.path))
            .unwrap_or_else(|err| panic!("read reference-only example {}: {err}", reference.path));
        assert!(
            source
                .lines()
                .take(5)
                .any(|line| line.contains("REFERENCE-ONLY")),
            "{} must start with a visible REFERENCE-ONLY marker",
            reference.path
        );
    }

    for required_phase_example in [
        "examples/07-phase105/01-do-act.ash",
        "examples/07-phase105/02-act-sugar.ash",
        "examples/07-phase105/03-do-proc-from-act.ash",
        "examples/08-phase106/01-act-comprehension.ash",
        "examples/08-phase106/02-proc-comprehension-from-act.ash",
        "examples/08-phase106/03-deferred-pure-targets.ash",
        "examples/09-phase108/01-do-workflow-unit.ash",
        "examples/09-phase108/02-do-workflow-contract-statements.ash",
        "examples/09-phase108/05-workflow-comprehension.ash",
        "examples/09-phase108/06-legacy-workflow-migration-warning.ash",
    ] {
        assert!(
            EXPECTED_PASS.contains(&required_phase_example),
            "Phase 105/106/108 example must remain in expected-pass corpus: {required_phase_example}"
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

    for reference in REFERENCE_ONLY {
        let (success, stdout, stderr) = run_ash_check(&repo, reference.path);
        assert!(
            !success,
            "reference-only example unexpectedly passed `ash check`: {}\nreason={}\nstdout={}\nstderr={}",
            reference.path, reference.reason, stdout, stderr
        );
    }

    println!(
        "TASK-760 examples corpus baseline: files={EXPECTED_EXAMPLE_FILES}, pass={passed}, fail={failed}, reference_only={}",
        REFERENCE_ONLY.len()
    );
    assert_eq!(passed, EXPECTED_EXAMPLE_PASSING);
    assert_eq!(failed, EXPECTED_EXAMPLE_FAILING);
    assert_eq!(REFERENCE_ONLY.len(), EXPECTED_EXAMPLE_REFERENCE_ONLY);
}
