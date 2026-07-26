use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_ash_test(args: &[&str]) -> (std::process::ExitStatus, serde_json::Value, String) {
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("ash test fixture should launch");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "fixture did not emit valid JSON: {error}\nstatus={}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (
        output.status,
        json,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

const MISSING_TYPED_LOWERING_ERROR: &str = "application execution failed: checked Core/CPS admission rejected: no validated production typed lowering is available";

#[test]
fn coverage_json_reports_uncovered_authored_evidence_when_execution_is_closed() {
    let (status, output, stderr) = run_ash_test(&[
        "test",
        "fixtures/phase147-coverage",
        "--coverage",
        "--format",
        "json",
    ]);
    assert!(
        !status.success(),
        "Path B must fail closed instead of running authored evidence: {stderr}"
    );
    assert_eq!(output["success"], false);
    assert_eq!(output["tests"][0]["outcome"], "error");
    assert_eq!(output["tests"][0]["message"], MISSING_TYPED_LOWERING_ERROR);

    let coverage = &output["coverage"];
    assert_eq!(coverage["schema_version"], "ash-law-coverage-v1.0");
    assert_eq!(coverage["totals"]["laws"], 2);
    assert_eq!(coverage["totals"]["covered_laws"], 0);
    assert_eq!(coverage["totals"]["uncovered_laws"], 2);
    assert_eq!(coverage["laws"][0]["evidence_status"], "uncovered");
    assert_eq!(coverage["laws"][0]["evidence_kind"], "authored_test");
    assert_eq!(coverage["uncovered_laws"][0]["name"], "covered_identity");
    assert_eq!(coverage["uncovered_laws"][1]["name"], "uncovered_identity");
}

#[test]
fn mutation_json_reports_survivors_when_authored_execution_is_closed() {
    let (status, output, stderr) = run_ash_test(&[
        "test",
        "fixtures/phase147-coverage",
        "--mutation",
        "--mutation-limit",
        "20",
        "--format",
        "json",
    ]);
    assert!(
        !status.success(),
        "Path B must fail closed instead of running authored evidence: {stderr}"
    );
    assert_eq!(output["success"], false);
    assert_eq!(output["tests"][0]["outcome"], "error");
    assert_eq!(output["tests"][0]["message"], MISSING_TYPED_LOWERING_ERROR);

    let mutation = &output["mutation"];
    assert_eq!(mutation["schema_version"], "ash-mutation-v1.0");
    assert_eq!(mutation["limit"], 20);
    assert_eq!(mutation["totals"]["generated"], 2);
    assert_eq!(mutation["totals"]["killed"], 0);
    assert_eq!(mutation["totals"]["survived"], 2);
    let mutants = mutation["mutants"].as_array().unwrap();
    assert!(mutants.iter().any(|mutant| {
        mutant["law"] == "covered_identity"
            && mutant["status"] == "survived"
            && mutant["replay_command"]
                .as_str()
                .unwrap()
                .contains("--mutation-id")
    }));
    assert!(
        mutants.iter().any(|mutant| {
            mutant["law"] == "uncovered_identity" && mutant["status"] == "survived"
        })
    );
}
