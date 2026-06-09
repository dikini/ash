//! Structured synthesized-case execution.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use ash_core::Expr as CoreExpr;
use serde::Serialize;
use serde_json::Value;

use super::eval::{evaluate_contract_postcondition, evaluate_simple_bool_expression};
use super::{
    ObligationCloseoutBehavior, ObligationLifecycleModelKind, ObligationLifecycleRejection,
    ObligationLifecycleTransition, ObligationLifecycleTransitionPlan,
    ObligationLifecycleTransitionTrace, ObligationTerminalExpectation, PolicyTerminalOracle,
    PolicyTerminalOracleRow, PolicyTerminalOutcome,
};
use crate::test_runner::types::{Outcome, ReproArtifact, TestKind, TestResult, TestSource};

/// Executable synthesized case model.
#[derive(Debug, Clone, Serialize)]
pub struct SynthesizedCase {
    /// Stable case id.
    pub id: String,
    /// Source classification.
    pub source: TestSource,
    /// Target kind label.
    pub target_kind: String,
    /// Target name.
    pub target_name: String,
    /// Source file path.
    pub file_path: PathBuf,
    /// Tags attached to the result.
    pub tags: Vec<String>,
    /// Deterministic seed.
    pub seed: u64,
    /// Materialized inputs.
    pub inputs: SynthesizedInputs,
    /// Executable oracle.
    pub oracle: SynthesizedOracle,
    /// Reproducible artifact emitted with the result.
    pub repro: ReproArtifact,
}

/// Materialized synthesized input bindings.
#[derive(Debug, Clone, Serialize)]
pub struct SynthesizedInputs {
    /// Input bindings.
    pub bindings: BTreeMap<String, Value>,
    /// Input source label.
    pub generated_from: String,
    /// Case index, starting at 1.
    pub case_index: usize,
    /// World index, starting at 1, when applicable.
    pub world_index: Option<usize>,
}

/// Executable synthesized oracle.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SynthesizedOracle {
    /// Contract `requires` expression expected to evaluate to a boolean.
    ContractRequires { expression: String, expected: bool },
    /// Contract `ensures` expression evaluated after target execution.
    ContractEnsures {
        /// Ensures expression display text.
        expression: String,
        /// Checked/lowered postcondition oracle expression.
        oracle: CoreExpr,
        /// Actual target output.
        target_output: Value,
    },
    /// Policy terminal outcome equality over explicit metadata-provided cases.
    PolicyTerminalEquals {
        /// Expected terminal outcome.
        expected: PolicyTerminalOutcome,
        /// Lowered policy reference used by the target metadata.
        policy_ref: String,
        /// Terminal oracle evaluated against finite policy inputs.
        terminal_oracle: PolicyTerminalOracle,
    },
    /// Obligation lifecycle expectation over explicit finite lifecycle metadata.
    ObligationLifecycle {
        /// Expected lifecycle terminal.
        expectation: ObligationTerminalExpectation,
        /// Typed lifecycle transition plan to execute.
        transition_plan: ObligationLifecycleTransitionPlan,
        /// Typed lifecycle transition trace to execute.
        transition_trace: ObligationLifecycleTransitionTrace,
    },
}

/// Execute a structured synthesized case and emit a runner result.
pub fn execute_synthesized_case(case: &SynthesizedCase) -> TestResult {
    let started = Instant::now();
    let (outcome, message) = match &case.oracle {
        SynthesizedOracle::ContractRequires {
            expression,
            expected,
        } => match evaluate_simple_bool_expression(expression, &case.inputs.bindings) {
            Ok(actual) if actual == *expected => (
                Outcome::Pass,
                Some(format!(
                    "executed synthesized oracle: {expression} == {expected}"
                )),
            ),
            Ok(actual) => (
                Outcome::Fail,
                Some(format!(
                    "synthesized oracle failed: {expression} evaluated to {actual}, expected {expected}"
                )),
            ),
            Err(reason) => (
                Outcome::Skip,
                Some(format!(
                    "deferred: unsupported synthesized oracle: {reason}"
                )),
            ),
        },
        SynthesizedOracle::ContractEnsures {
            expression,
            oracle,
            target_output,
        } => match evaluate_contract_postcondition(oracle, &case.inputs.bindings, target_output) {
            Ok(true) => (
                Outcome::Pass,
                Some(format!(
                    "executed synthesized contract postcondition oracle: {expression}"
                )),
            ),
            Ok(false) => (
                Outcome::Fail,
                Some(format!(
                    "synthesized contract postcondition failed: {expression} over target output {target_output}"
                )),
            ),
            Err(reason) => (
                Outcome::Skip,
                Some(format!(
                    "deferred: unsupported synthesized contract postcondition oracle: {reason}"
                )),
            ),
        },
        SynthesizedOracle::PolicyTerminalEquals {
            expected,
            policy_ref,
            terminal_oracle,
        } => {
            match evaluate_policy_terminal_oracle(terminal_oracle, &case.inputs.bindings) {
                Some(actual) if actual == *expected => (
                    Outcome::Pass,
                    Some(format!(
                        "executed synthesized policy terminal oracle {policy_ref}: {:?}",
                        expected,
                    )),
                ),
                Some(actual) => (
                    Outcome::Fail,
                    Some(format!(
                        "synthesized policy oracle {policy_ref} failed: terminal {:?}, expected {:?}",
                        actual, expected,
                    )),
                ),
                None => (
                    Outcome::Skip,
                    Some(
                        "deferred: unsupported synthesized policy oracle: no terminal matched finite input"
                            .to_string(),
                    ),
                ),
            }
        }
        SynthesizedOracle::ObligationLifecycle {
            expectation,
            transition_plan,
            transition_trace,
        } => evaluate_obligation_lifecycle_oracle(
            expectation,
            transition_plan,
            transition_trace,
        ),
    };

    let mut result = TestResult::new(&case.id, case.file_path.clone())
        .with_outcome(outcome)
        .with_source(case.source)
        .with_kind(TestKind::Unit)
        .with_duration(started.elapsed())
        .with_repro_artifact(case.repro.clone());
    if let Some(message) = message {
        result = result.with_message(message);
    }
    result.world_index = case.inputs.world_index;
    result.tags = case.tags.clone();
    result
}

fn evaluate_obligation_lifecycle_oracle(
    expectation: &ObligationTerminalExpectation,
    transition_plan: &ObligationLifecycleTransitionPlan,
    transition_trace: &ObligationLifecycleTransitionTrace,
) -> (Outcome, Option<String>) {
    let Some(expected_terminal) = expected_obligation_lifecycle_terminal(expectation) else {
        return (
            Outcome::Skip,
            Some("deferred: unsupported synthesized obligation lifecycle expectation".to_string()),
        );
    };
    match execute_obligation_lifecycle_trace(transition_plan, transition_trace) {
        Ok(actual_terminal) if actual_terminal == expected_terminal => (
            Outcome::Pass,
            Some(format!(
                "executed synthesized obligation lifecycle transition oracle: {:?}",
                expectation
            )),
        ),
        Ok(actual_terminal) => (
            Outcome::Fail,
            Some(format!(
                "synthesized obligation lifecycle oracle failed: executed terminal {:?}, expected {:?}",
                actual_terminal, expected_terminal,
            )),
        ),
        Err(reason) => (
            Outcome::Skip,
            Some(format!(
                "deferred: unsupported synthesized obligation lifecycle execution: {reason}"
            )),
        ),
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "terminal", rename_all = "snake_case")]
pub(super) enum ExecutedObligationLifecycleTerminal {
    NotIntroduced,
    Introduced,
    Discharged,
    Rejected {
        reason: ObligationLifecycleRejection,
    },
}

impl ExecutedObligationLifecycleTerminal {
    pub(super) fn control_state(&self) -> &'static str {
        match self {
            Self::NotIntroduced => "not_introduced",
            Self::Introduced => "introduced",
            Self::Discharged => "discharged",
            Self::Rejected { .. } => "rejected",
        }
    }
}

pub(super) fn expected_obligation_lifecycle_terminal(
    expectation: &ObligationTerminalExpectation,
) -> Option<ExecutedObligationLifecycleTerminal> {
    match expectation {
        ObligationTerminalExpectation::Introduced => {
            Some(ExecutedObligationLifecycleTerminal::Introduced)
        }
        ObligationTerminalExpectation::Discharged => {
            Some(ExecutedObligationLifecycleTerminal::Discharged)
        }
        ObligationTerminalExpectation::MissingDischargeRejected => {
            Some(ExecutedObligationLifecycleTerminal::Rejected {
                reason: ObligationLifecycleRejection::MissingDischarge,
            })
        }
        ObligationTerminalExpectation::DoubleDischargeRejected => {
            Some(ExecutedObligationLifecycleTerminal::Rejected {
                reason: ObligationLifecycleRejection::DoubleDischarge,
            })
        }
        ObligationTerminalExpectation::Unsupported => None,
    }
}

pub(super) fn execute_obligation_lifecycle_trace(
    plan: &ObligationLifecycleTransitionPlan,
    trace: &ObligationLifecycleTransitionTrace,
) -> Result<ExecutedObligationLifecycleTerminal, String> {
    if plan.model != ObligationLifecycleModelKind::IntroduceDischargeCheck {
        return Err("unsupported lifecycle model".to_string());
    }
    if plan.required_closeout != ObligationCloseoutBehavior::RejectIfOpen {
        return Err("unsupported closeout behavior".to_string());
    }
    if plan.introduction_sites.is_empty()
        || plan.discharge_sites.is_empty()
        || plan.check_sites.is_empty()
    {
        return Err("transition plan lacks introduction, discharge, or check sites".to_string());
    }
    if trace.transitions.is_empty() {
        return Err("transition trace is empty".to_string());
    }

    let mut terminal = ExecutedObligationLifecycleTerminal::NotIntroduced;
    for transition in &trace.transitions {
        match transition {
            ObligationLifecycleTransition::Introduce { site } => {
                if !plan.introduction_sites.contains(site) {
                    return Err(format!("unknown introduction site {site:?}"));
                }
                terminal = match terminal {
                    ExecutedObligationLifecycleTerminal::NotIntroduced => {
                        ExecutedObligationLifecycleTerminal::Introduced
                    }
                    ExecutedObligationLifecycleTerminal::Introduced
                    | ExecutedObligationLifecycleTerminal::Discharged => {
                        return Err("duplicate introduction is outside supported slice".to_string());
                    }
                    ExecutedObligationLifecycleTerminal::Rejected { .. } => {
                        return Err(
                            "transition after rejection is outside supported slice".to_string()
                        );
                    }
                };
            }
            ObligationLifecycleTransition::Discharge { site } => {
                if !plan.discharge_sites.contains(site) {
                    return Err(format!("unknown discharge site {site:?}"));
                }
                terminal = match terminal {
                    ExecutedObligationLifecycleTerminal::Introduced => {
                        ExecutedObligationLifecycleTerminal::Discharged
                    }
                    ExecutedObligationLifecycleTerminal::Discharged => {
                        ExecutedObligationLifecycleTerminal::Rejected {
                            reason: ObligationLifecycleRejection::DoubleDischarge,
                        }
                    }
                    ExecutedObligationLifecycleTerminal::NotIntroduced => {
                        return Err(
                            "discharge before introduction is outside supported slice".to_string()
                        );
                    }
                    ExecutedObligationLifecycleTerminal::Rejected { .. } => {
                        return Err(
                            "transition after rejection is outside supported slice".to_string()
                        );
                    }
                };
            }
            ObligationLifecycleTransition::Check { site } => {
                if !plan.check_sites.contains(site) {
                    return Err(format!("unknown check site {site:?}"));
                }
                terminal = match terminal {
                    ExecutedObligationLifecycleTerminal::Introduced => {
                        ExecutedObligationLifecycleTerminal::Rejected {
                            reason: ObligationLifecycleRejection::MissingDischarge,
                        }
                    }
                    ExecutedObligationLifecycleTerminal::Discharged => {
                        ExecutedObligationLifecycleTerminal::Discharged
                    }
                    ExecutedObligationLifecycleTerminal::NotIntroduced => {
                        return Err(
                            "check before introduction is outside supported slice".to_string()
                        );
                    }
                    ExecutedObligationLifecycleTerminal::Rejected { .. } => terminal,
                };
            }
            ObligationLifecycleTransition::Reject { reason } => match &terminal {
                ExecutedObligationLifecycleTerminal::Rejected {
                    reason: actual_reason,
                } if actual_reason == reason => {}
                ExecutedObligationLifecycleTerminal::Rejected {
                    reason: actual_reason,
                } => {
                    return Err(format!(
                        "explicit rejection reason {reason:?} disagrees with executed reason {actual_reason:?}"
                    ));
                }
                _ => {
                    return Err(
                        "explicit rejection is not justified by prior lifecycle transitions"
                            .to_string(),
                    );
                }
            },
        }
    }

    Ok(terminal)
}

pub(super) fn evaluate_policy_terminal_oracle(
    terminal_oracle: &PolicyTerminalOracle,
    bindings: &BTreeMap<String, Value>,
) -> Option<PolicyTerminalOutcome> {
    let PolicyTerminalOracle::ExactMatchTable {
        input_binding,
        rows,
    } = terminal_oracle
    else {
        return None;
    };
    let input = bindings.get(input_binding)?;
    rows.iter()
        .find(|row| policy_terminal_oracle_row_matches(input, row))
        .map(|row| row.terminal.clone())
}

fn policy_terminal_oracle_row_matches(input: &Value, row: &PolicyTerminalOracleRow) -> bool {
    row.when
        .iter()
        .all(|(field, expected)| input.get(field) == Some(expected))
}
