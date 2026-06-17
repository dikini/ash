//! Law coverage and bounded mutation reporting for `ash test`.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::test_runner::synthesized::{
    LawScope, LawTestEvidence, RunnerIntrospectionSnapshot, RunnerLawMetadata,
};
use crate::test_runner::types::{Outcome, TestResult};

/// Stable schema version for law coverage reports.
pub const LAW_COVERAGE_SCHEMA_VERSION: &str = "ash-law-coverage-v1.0";
/// Stable schema version for mutation reports.
pub const MUTATION_SCHEMA_VERSION: &str = "ash-mutation-v1.0";

/// Suite-level law/test coverage report.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LawCoverageReport {
    /// Report schema version.
    pub schema_version: String,
    /// Aggregated coverage totals.
    pub totals: LawCoverageTotals,
    /// Per-law coverage rows.
    pub laws: Vec<LawCoverageRow>,
    /// Convenience subset of uncovered law rows.
    pub uncovered_laws: Vec<LawCoverageRow>,
}

/// Aggregated law/test coverage totals.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct LawCoverageTotals {
    /// Discovered law declarations requiring test evidence.
    pub laws: usize,
    /// Laws with satisfied test evidence.
    pub covered_laws: usize,
    /// Laws without satisfied test evidence.
    pub uncovered_laws: usize,
    /// Authored `by test "..."` evidence links.
    pub authored_test_links: usize,
    /// Property evidence links.
    pub property_links: usize,
    /// Small-world evidence links.
    pub small_world_links: usize,
}

/// Per-law coverage row.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LawCoverageRow {
    /// Stable law id.
    pub id: String,
    /// Law declaration name.
    pub name: String,
    /// Scope display (`module` or `interface`).
    pub scope: String,
    /// Owning interface, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Source-level proposition summary.
    pub proposition: String,
    /// Evidence kind backing the law.
    pub evidence_kind: String,
    /// Coverage status (`covered`, `uncovered`, or `deferred`).
    pub evidence_status: String,
    /// Optional evidence target, such as an authored test name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_target: Option<String>,
}

/// Suite-level bounded mutation report.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MutationReport {
    /// Report schema version.
    pub schema_version: String,
    /// Mutant generation limit supplied by the CLI.
    pub limit: usize,
    /// Aggregated mutation totals.
    pub totals: MutationTotals,
    /// Per-mutant rows.
    pub mutants: Vec<MutationRow>,
}

/// Aggregated mutation totals.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct MutationTotals {
    /// Number of mutants generated within the limit.
    pub generated: usize,
    /// Mutants killed by satisfied law evidence.
    pub killed: usize,
    /// Mutants that survived because no executed evidence killed them.
    pub survived: usize,
    /// Mutants deferred because their evidence mode is unsupported by this slice.
    pub deferred: usize,
    /// Mutants that hit an execution or evidence error.
    pub errored: usize,
}

/// Per-mutant report row.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MutationRow {
    /// Stable mutant id.
    pub id: String,
    /// Source law id.
    pub law_id: String,
    /// Source law name.
    pub law: String,
    /// Mutating operator id.
    pub operator: String,
    /// Mutation status (`killed`, `survived`, `deferred`, or `errored`).
    pub status: String,
    /// Human-readable mutation target summary.
    pub target: String,
    /// Source proposition before mutation.
    pub original: String,
    /// Mutated proposition summary.
    pub replacement: String,
    /// Replay command hint through the Ash final surface.
    pub replay_command: String,
}

/// Build law/test coverage from runner introspection snapshots and authored test results.
pub fn coverage_report(
    snapshots: &[(std::path::PathBuf, RunnerIntrospectionSnapshot)],
    authored_tests: &BTreeMap<String, TestResult>,
) -> LawCoverageReport {
    let mut rows = Vec::new();
    for (_path, snapshot) in snapshots {
        for law in &snapshot.laws {
            rows.push(coverage_row(law, authored_tests));
        }
    }

    rows.sort_by(|a, b| a.id.cmp(&b.id));
    let uncovered_laws = rows
        .iter()
        .filter(|row| row.evidence_status != "covered")
        .cloned()
        .collect::<Vec<_>>();
    let totals = LawCoverageTotals {
        laws: rows.len(),
        covered_laws: rows
            .iter()
            .filter(|row| row.evidence_status == "covered")
            .count(),
        uncovered_laws: uncovered_laws.len(),
        authored_test_links: rows
            .iter()
            .filter(|row| row.evidence_kind == "authored_test")
            .count(),
        property_links: rows
            .iter()
            .filter(|row| row.evidence_kind == "property")
            .count(),
        small_world_links: rows
            .iter()
            .filter(|row| row.evidence_kind == "small_world")
            .count(),
    };

    LawCoverageReport {
        schema_version: LAW_COVERAGE_SCHEMA_VERSION.to_string(),
        totals,
        laws: rows,
        uncovered_laws,
    }
}

/// Build a bounded mutation report from coverage rows.
pub fn mutation_report(
    root: &Path,
    coverage: &LawCoverageReport,
    limit: usize,
    mutation_id: Option<&str>,
) -> MutationReport {
    let mut mutants = Vec::new();
    for row in &coverage.laws {
        let mutant = mutation_row(root, row);
        if let Some(expected_id) = mutation_id
            && mutant.id != expected_id
        {
            continue;
        }
        mutants.push(mutant);
        if mutants.len() >= limit {
            break;
        }
    }

    let totals = MutationTotals {
        generated: mutants.len(),
        killed: mutants
            .iter()
            .filter(|mutant| mutant.status == "killed")
            .count(),
        survived: mutants
            .iter()
            .filter(|mutant| mutant.status == "survived")
            .count(),
        deferred: mutants
            .iter()
            .filter(|mutant| mutant.status == "deferred")
            .count(),
        errored: mutants
            .iter()
            .filter(|mutant| mutant.status == "errored")
            .count(),
    };

    MutationReport {
        schema_version: MUTATION_SCHEMA_VERSION.to_string(),
        limit,
        totals,
        mutants,
    }
}

fn coverage_row(
    law: &RunnerLawMetadata,
    authored_tests: &BTreeMap<String, TestResult>,
) -> LawCoverageRow {
    let (evidence_kind, evidence_status, evidence_target) = match &law.test_evidence {
        Some(LawTestEvidence::Authored { test_name }) => {
            let status = match authored_tests.get(test_name) {
                Some(result) if result.outcome == Outcome::Pass => "covered",
                Some(result) if result.outcome.is_failure() => "uncovered",
                Some(_) => "deferred",
                None => "uncovered",
            };
            (
                "authored_test".to_string(),
                status.to_string(),
                Some(test_name.clone()),
            )
        }
        Some(LawTestEvidence::Property { .. }) => (
            "property".to_string(),
            "deferred".to_string(),
            Some("by test property".to_string()),
        ),
        Some(LawTestEvidence::SmallWorld) => (
            "small_world".to_string(),
            "deferred".to_string(),
            Some("by test small_world".to_string()),
        ),
        None => ("none".to_string(), "uncovered".to_string(), None),
    };

    LawCoverageRow {
        id: law.id.clone(),
        name: law.name.clone(),
        scope: match law.scope {
            LawScope::Module => "module".to_string(),
            LawScope::Interface => "interface".to_string(),
        },
        owner: law.owner.clone(),
        proposition: law.proposition.clone(),
        evidence_kind,
        evidence_status,
        evidence_target,
    }
}

fn mutation_row(root: &Path, row: &LawCoverageRow) -> MutationRow {
    let (operator, replacement) = mutation_for_proposition(&row.proposition);
    let status = match row.evidence_status.as_str() {
        "covered" if mutated_proposition_is_killed(&replacement) => "killed",
        "covered" => "survived",
        "deferred" => "deferred",
        _ => "survived",
    };
    MutationRow {
        id: format!("mutant:{}:{operator}", row.id),
        law_id: row.id.clone(),
        law: row.name.clone(),
        operator: operator.to_string(),
        status: status.to_string(),
        target: "law proposition".to_string(),
        original: row.proposition.clone(),
        replacement,
        replay_command: format!(
            "ASH_UNDER_TEST=${{ASH_UNDER_TEST:?set Ash candidate binary}}; \"$ASH_UNDER_TEST\" test {} --mutation --mutation-id 'mutant:{}:{operator}' --mutation-limit 1 --format json",
            root.display(),
            row.id
        ),
    }
}

fn mutated_proposition_is_killed(replacement: &str) -> bool {
    !evaluate_mutated_proposition(replacement)
}

fn evaluate_mutated_proposition(replacement: &str) -> bool {
    if replacement == "true" {
        return true;
    }
    if replacement == "false" {
        return false;
    }
    let Some((left, right)) = replacement.split_once(" != ") else {
        return true;
    };
    left.trim() != right.trim()
}

fn mutation_for_proposition(proposition: &str) -> (&'static str, String) {
    if proposition.contains(" == ") {
        (
            "equality_inversion",
            proposition.replacen(" == ", " != ", 1),
        )
    } else if proposition.contains(" != ") {
        (
            "equality_inversion",
            proposition.replacen(" != ", " == ", 1),
        )
    } else if proposition == "true" {
        ("boolean_flip", "false".to_string())
    } else if proposition == "false" {
        ("boolean_flip", "true".to_string())
    } else {
        ("truth_boundary", format!("!({proposition})"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_runner::synthesized::{LawScope, LawTestEvidence, RunnerLawMetadata};
    use crate::test_runner::types::TestResult;
    use std::path::PathBuf;

    #[test]
    fn coverage_marks_missing_and_passing_authored_evidence() {
        let snapshot = RunnerIntrospectionSnapshot {
            laws: vec![
                law(
                    "covered",
                    Some(LawTestEvidence::Authored {
                        test_name: "t".into(),
                    }),
                ),
                law("uncovered", None),
            ],
            ..RunnerIntrospectionSnapshot::default()
        };
        let mut authored = BTreeMap::new();
        authored.insert("t".into(), TestResult::new("t", PathBuf::from("t.ash")));
        let report = coverage_report(&[(PathBuf::from("suite.ash"), snapshot)], &authored);
        assert_eq!(report.totals.laws, 2);
        assert_eq!(report.totals.covered_laws, 1);
        assert_eq!(report.totals.uncovered_laws, 1);
    }

    #[test]
    fn mutation_distinguishes_killed_and_survived() {
        let coverage = LawCoverageReport {
            schema_version: LAW_COVERAGE_SCHEMA_VERSION.into(),
            totals: LawCoverageTotals::default(),
            laws: vec![
                coverage_law("covered", "covered"),
                coverage_law("uncovered", "uncovered"),
            ],
            uncovered_laws: vec![],
        };
        let report = mutation_report(Path::new("fixtures/example"), &coverage, 20, None);
        assert_eq!(report.totals.generated, 2);
        assert_eq!(report.totals.killed, 1);
        assert_eq!(report.totals.survived, 1);
    }

    #[test]
    fn coverage_defers_property_and_small_world_without_execution_rows() {
        let snapshot = RunnerIntrospectionSnapshot {
            laws: vec![
                law("property", Some(LawTestEvidence::Property { strategies: vec![] })),
                law("small", Some(LawTestEvidence::SmallWorld)),
            ],
            ..RunnerIntrospectionSnapshot::default()
        };
        let report = coverage_report(&[(PathBuf::from("suite.ash"), snapshot)], &BTreeMap::new());
        assert_eq!(report.totals.covered_laws, 0);
        assert_eq!(report.totals.uncovered_laws, 2);
        assert!(
            report
                .laws
                .iter()
                .all(|law| law.evidence_status == "deferred")
        );
    }

    fn law(name: &str, evidence: Option<LawTestEvidence>) -> RunnerLawMetadata {
        RunnerLawMetadata {
            id: format!("law:module:{name}"),
            name: name.into(),
            scope: LawScope::Module,
            owner: None,
            params: vec!["x: Int".into()],
            proposition: "x == x".into(),
            delegated_test: None,
            test_evidence: evidence,
        }
    }

    fn coverage_law(name: &str, status: &str) -> LawCoverageRow {
        LawCoverageRow {
            id: format!("law:module:{name}"),
            name: name.into(),
            scope: "module".into(),
            owner: None,
            proposition: "x == x".into(),
            evidence_kind: "none".into(),
            evidence_status: status.into(),
            evidence_target: None,
        }
    }
}
