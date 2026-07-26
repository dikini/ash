//! File-backed, Rust-first differential corpus support.
//!
//! The production engine remains the authority for the direct-runtime target.
//! A fixture that supplies a checked Core/CPS encoding is evaluated through
//! the checked prototype boundary and compared with the direct runtime.
//! Fixtures without that encoding remain explicitly unsupported.

use crate::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};
use ash_core::Value;
use ash_core::core_ash::{
    CoreAtom, CoreExpr, CoreMultiplicity, CorePrimOp, CoreRow, CoreType, CoreValue,
};
use ash_core::cps::{
    Atom as CpsAtom, ContMultiplicity, EffectItem, EffectItemKind, EffectOp,
    EffectRow as CpsEffectRow, Env as CpsEnv, HandlerChain, HandlerFrame, PrimOp as CpsPrimOp,
    Term as CpsTerm, Value as CpsValue,
};
use ash_interp::cps::{CpsError, CpsRunError, CpsTerminalOutcome, eval_checked_terminal};
use ash_interp::{Context as InterpContext, EvalError, eval::eval_expr};
use serde::Deserialize;
use serde_json::{Value as JsonValue, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

const CORE_CPS_RELATION: &str = "direct-runtime-to-checked-core-cps";
const CORE_CPS_OWNER: &str = "TASK-2004";
const TIME_SLEEP_PROVIDER_SOURCE: &str = "fn main() -> Null { time::sleep(0) }";
const STANDARD_APPLICATION_PROFILE: &str = "application_default";
const TIME_SLEEP_NULL_PROVIDER_DISCHARGE: &str = "time_sleep_null";
const TIME_SLEEP_PROVIDER_HANDLER: &str = "__phase202_time_sleep_provider";
const TIME_SLEEP_PROVIDER_ARGUMENT: &str = "__phase202_time_sleep_millis";
const TIME_SLEEP_PROVIDER_CONTINUATION: &str = "__phase202_time_sleep_resume";
/// One closed source witness for a Boolean-`Not` parity case.
///
/// These are corpus-admission facts, not a source lowering policy.  Each
/// tuple binds the fixture identity, exact source text, and checked primitive
/// operand together so the two complementary witnesses cannot be relabelled
/// or swapped under the same `SEM-CPS-PRIM-001` metadata.
#[derive(Clone, Copy)]
struct SourceEntryBoolNotWitness {
    case_id: &'static str,
    source: &'static str,
    operand: bool,
}

const SOURCE_ENTRY_BOOL_NOT_WITNESSES: &[SourceEntryBoolNotWitness] = &[
    SourceEntryBoolNotWitness {
        case_id: "phase202-source-bool-not-bridge-return-false",
        source: "fn main() -> Bool { !true }",
        operand: true,
    },
    SourceEntryBoolNotWitness {
        case_id: "phase202-source-bool-not-bridge-return-true",
        source: "fn main() -> Bool { !false }",
        operand: false,
    },
];

/// One closed lexical Boolean-`Not` source-entry witness.
///
/// This binds the corpus case identity to its exact source, the one lexical
/// binder, and its literal operand. It is deliberately separate from the
/// literal witnesses: accepting a `LetVal` spine must not widen either
/// witness family into a general lexical/unary source-entry rule.
#[derive(Clone, Copy)]
struct SourceEntryLexicalBoolNotWitness {
    case_id: &'static str,
    source: &'static str,
    binder: &'static str,
    operand: bool,
}

const SOURCE_ENTRY_LEXICAL_BOOL_NOT_WITNESSES: &[SourceEntryLexicalBoolNotWitness] = &[
    SourceEntryLexicalBoolNotWitness {
        case_id: "phase202-source-lexical-bool-not-bridge-return-false",
        source: "fn main() -> Bool { do { let flag = true; return !flag; } }",
        binder: "flag",
        operand: true,
    },
    SourceEntryLexicalBoolNotWitness {
        case_id: "phase202-source-lexical-bool-not-bridge-return-true",
        source: "fn main() -> Bool { do { let flag = false; return !flag; } }",
        binder: "flag",
        operand: false,
    },
];

const TRUSTED_DIRECT_REFERENCE_CASES: &[(&str, &str)] = &[
    (
        "phase202-checked-core-cps-failure-attribution",
        "fn main() -> Int { 7 }",
    ),
    (
        "phase202-direct-runtime-failure-attribution",
        "fn main() -> Int { missing_direct_value }",
    ),
    (
        "phase202-primitive-domain-trap",
        "fn main() -> Int { 1 / 0 }",
    ),
    ("phase202-return-unit", "fn main() { 7 }"),
    ("phase202-return-unit-mismatch", "fn main() { 7 }"),
    (
        "phase202-source-if-false-bridge-return-9",
        "fn main() -> Int { if false then 7 else 9 }",
    ),
    (
        "phase202-source-if-true-bridge-return-7",
        "fn main() -> Int { if true then 7 else 9 }",
    ),
    (
        "phase202-source-int-add-bridge-return-7",
        "fn main() -> Int { 2 + 5 }",
    ),
    (
        "phase202-source-bool-not-bridge-return-false",
        "fn main() -> Bool { !true }",
    ),
    (
        "phase202-source-bool-not-bridge-return-true",
        "fn main() -> Bool { !false }",
    ),
    (
        "phase202-source-lexical-int-add-bridge-return-7",
        "fn main() -> Int { do { let x = 2; let y = 5; return x + y; } }",
    ),
    (
        "phase202-source-lexical-bool-not-bridge-return-false",
        "fn main() -> Bool { do { let flag = true; return !flag; } }",
    ),
    (
        "phase202-source-lexical-bool-not-bridge-return-true",
        "fn main() -> Bool { do { let flag = false; return !flag; } }",
    ),
    (
        "phase202-source-return-continuation",
        "fn main() -> Int { do { return 42; } }",
    ),
    ("phase202-v3-int-add-return-7", "fn main() -> Int { 2 + 5 }"),
    (
        "phase202-v4-if-false-return-int-9",
        "fn main() -> Int { if false then 7 else 9 }",
    ),
    (
        "phase202-v4-if-true-return-int-7",
        "fn main() -> Int { if true then 7 else 9 }",
    ),
];

enum TrustedDirectOracleOutcome {
    Return(Value),
    PrimitiveDomainTrap,
}

/// The Rust execution target currently supported by the corpus harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustExecutionTarget {
    /// Execute the source through Ash's production direct runtime.
    DirectRuntime,
    /// Execute a narrow, checked λAsh-CPS prototype corpus input.
    CheckedCoreCpsPrototype,
}

/// Result of comparing one target against a corpus expectation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaseComparisonStatus {
    /// The observed result matched an exact or finite allowed expectation.
    Passed,
    /// The observed result did not match the declared expectation.
    Failed {
        /// Why the normalized result did not match the expectation.
        reason: String,
    },
    /// The selected target cannot yet be evaluated by this harness.
    Unsupported {
        /// Why the requested comparison cannot be evaluated.
        reason: String,
    },
}

/// Status of a relation between independently evaluated targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationStatus {
    /// The relation was evaluated and held.
    Passed,
    /// The targets were both evaluated but their normalized observations differ.
    Failed {
        /// Canonical rule governing the observable that drifted.
        canonical_rule_id: String,
        /// Why the paired observations did not agree.
        reason: String,
    },
    /// The relation has an explicit owner but no implementation yet.
    Unsupported {
        /// Task responsible for implementing the relation.
        owner: String,
        /// Name of the unavailable relation.
        relation: String,
    },
}

/// Observable dimension that TASK-2005 must compare or explicitly account for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservableDimension {
    /// Terminal returned values.
    Values,
    /// Structured terminal traps.
    StructuredTraps,
    /// Effect-handler frame selection order.
    FrameOrdering,
    /// Missing effect-discharge behavior.
    MissingDischarge,
    /// Row admission and requirement behavior.
    Rows,
    /// Continuation invocation and multiplicity behavior.
    ContinuationUse,
    /// Dynamic contract behavior at execution boundaries.
    DynamicContracts,
    /// Finite allowed outcomes at external boundaries.
    AllowedExternalOutcomes,
    /// Availability of checked Core/CPS execution for the comparison.
    CheckedCoreCpsExecution,
}

/// Parity evidence or explicitly owned non-parity for one observable dimension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParityDisposition {
    /// Both targets were executed and compared for this dimension.
    Compared {
        /// Canonical rule governing the compared observable.
        canonical_rule_id: String,
        /// Task or component responsible for this evidence.
        owner: String,
    },
    /// A deliberately bounded semantic difference is recorded.
    BoundedDivergence {
        /// Canonical rule governing the divergence.
        canonical_rule_id: String,
        /// Task responsible for resolving or retaining the divergence.
        owner: String,
    },
    /// The requested comparison is not available, with explicit ownership.
    Unsupported {
        /// Canonical rule governing the unavailable comparison.
        canonical_rule_id: String,
        /// Task responsible for making the comparison available.
        owner: String,
    },
}

impl ParityDisposition {
    /// Canonical rule governing this comparison or non-parity state.
    #[must_use]
    pub fn canonical_rule_id(&self) -> &str {
        match self {
            Self::Compared {
                canonical_rule_id, ..
            }
            | Self::BoundedDivergence {
                canonical_rule_id, ..
            }
            | Self::Unsupported {
                canonical_rule_id, ..
            } => canonical_rule_id,
        }
    }

    /// Task or component that owns this comparison or non-parity state.
    #[must_use]
    pub fn owner(&self) -> &str {
        match self {
            Self::Compared { owner, .. }
            | Self::BoundedDivergence { owner, .. }
            | Self::Unsupported { owner, .. } => owner,
        }
    }
}

/// Dimension-by-dimension parity accounting for one corpus fixture.
#[derive(Debug, Clone)]
pub struct ParityReport {
    source_fixture: String,
    dispositions: Vec<(ObservableDimension, ParityDisposition)>,
}

impl ParityReport {
    /// Corpus fixture from which this parity accounting was derived.
    #[must_use]
    pub fn source_fixture(&self) -> &str {
        &self.source_fixture
    }

    /// Parity disposition recorded for `dimension`, if present.
    #[must_use]
    pub fn disposition_for(&self, dimension: ObservableDimension) -> Option<&ParityDisposition> {
        self.dispositions
            .iter()
            .find(|(candidate, _)| *candidate == dimension)
            .map(|(_, disposition)| disposition)
    }
}

/// Errors while loading a file-backed differential corpus.
#[derive(Debug, Error)]
pub enum DifferentialHarnessError {
    /// A corpus file could not be read.
    #[error("could not read {path}: {source}")]
    Io {
        /// File that could not be read.
        path: String,
        /// Underlying read error.
        source: std::io::Error,
    },
    /// A corpus JSON document was malformed.
    #[error("could not parse JSON in {path}: {source}")]
    Json {
        /// JSON file that could not be parsed.
        path: String,
        /// Underlying JSON parse error.
        source: serde_json::Error,
    },
    /// A corpus document did not satisfy the small harness contract.
    #[error("invalid differential corpus case: {0}")]
    InvalidCase(String),
}

/// A loaded, reusable collection of differential corpus cases.
#[derive(Debug)]
pub struct DifferentialHarness {
    cases: HashMap<String, LoadedCase>,
    trusted_direct_oracle: bool,
}

/// An auditable report for one corpus case execution.
#[derive(Debug, Clone)]
pub struct DifferentialCaseReport {
    case_id: String,
    canonical_rule_ids: Vec<String>,
    expectation_kind: String,
    direct_runtime_status: CaseComparisonStatus,
    checked_core_cps_relation: RelationStatus,
    parity_report: ParityReport,
    actual_result: Option<JsonValue>,
}

/// Borrowed canonical rule IDs with string-oriented membership lookup.
#[derive(Debug, Clone, Copy)]
pub struct CanonicalRuleIds<'a>(&'a [String]);

impl CanonicalRuleIds<'_> {
    /// Returns whether this report cites `rule_id`.
    #[must_use]
    pub fn contains(&self, rule_id: &str) -> bool {
        self.0.iter().any(|candidate| candidate == rule_id)
    }
}

impl DifferentialCaseReport {
    /// Corpus case identifier.
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Canonical specification IDs the corpus case exercises.
    #[must_use]
    pub fn canonical_rule_ids(&self) -> CanonicalRuleIds<'_> {
        CanonicalRuleIds(&self.canonical_rule_ids)
    }

    /// Whether the fixture specifies one result or a finite allowed set.
    #[must_use]
    pub fn expectation_kind(&self) -> &str {
        &self.expectation_kind
    }

    /// Direct runtime comparison result.
    #[must_use]
    pub fn direct_runtime_status(&self) -> CaseComparisonStatus {
        self.direct_runtime_status.clone()
    }

    /// Checked Core/CPS relation status.
    #[must_use]
    pub fn checked_core_cps_relation(&self) -> RelationStatus {
        self.checked_core_cps_relation.clone()
    }

    /// Dimension-by-dimension parity accounting for this fixture.
    #[must_use]
    pub const fn parity_report(&self) -> &ParityReport {
        &self.parity_report
    }

    /// The normalized result that was compared to the expectation, if run.
    #[must_use]
    pub const fn actual_result(&self) -> Option<&JsonValue> {
        self.actual_result.as_ref()
    }
}

impl DifferentialHarness {
    /// Load every immediate case directory under `corpus_root`.
    ///
    /// # Errors
    ///
    /// Returns an error if a corpus file is unreadable, malformed, or violates
    /// the harness's manifest and expectation contract.
    pub fn load(corpus_root: impl AsRef<Path>) -> Result<Self, DifferentialHarnessError> {
        let corpus_root = corpus_root.as_ref();
        let trusted_direct_oracle = is_trusted_builtin_corpus_root(corpus_root);
        let entries = fs::read_dir(corpus_root).map_err(|source| DifferentialHarnessError::Io {
            path: corpus_root.display().to_string(),
            source,
        })?;
        let mut cases = HashMap::new();

        for entry in entries {
            let entry = entry.map_err(|source| DifferentialHarnessError::Io {
                path: corpus_root.display().to_string(),
                source,
            })?;
            let case_dir = entry.path();
            let metadata =
                fs::symlink_metadata(&case_dir).map_err(|source| DifferentialHarnessError::Io {
                    path: case_dir.display().to_string(),
                    source,
                })?;
            // The legacy adapter retains its historical directory behavior.
            // Canonical Core V1, however, has an explicitly literal-local
            // carrier boundary and must not enter through a symlinked case
            // directory either.
            if metadata.file_type().is_symlink() && case_dir.join("canonical-core.json").is_file() {
                return Err(DifferentialHarnessError::InvalidCase(format!(
                    "{} canonical Core V1 case directory must not be a symlink",
                    case_dir.display()
                )));
            }
            if !case_dir.is_dir() {
                continue;
            }
            let case = LoadedCase::load(&case_dir)?;
            if cases.insert(case.manifest.case_id.clone(), case).is_some() {
                return Err(DifferentialHarnessError::InvalidCase(format!(
                    "duplicate case ID in {}",
                    case_dir.display()
                )));
            }
        }
        Ok(Self {
            cases,
            trusted_direct_oracle,
        })
    }

    /// Run one case against the selected Rust target.
    #[must_use]
    pub fn run_case(&self, case_id: &str, target: RustExecutionTarget) -> DifferentialCaseReport {
        let Some(case) = self.cases.get(case_id) else {
            return DifferentialCaseReport {
                case_id: case_id.to_string(),
                canonical_rule_ids: Vec::new(),
                expectation_kind: "unknown".to_string(),
                direct_runtime_status: CaseComparisonStatus::Unsupported {
                    reason: format!("unknown differential corpus case `{case_id}`"),
                },
                checked_core_cps_relation: core_cps_unsupported(),
                parity_report: unsupported_parity_report(case_id),
                actual_result: None,
            };
        };

        // Canonical-Core V1 is deliberately a checked-CPS-only corpus
        // adapter.  In particular, selecting the direct target must not turn
        // its manifest-local Core text into a new Engine entry point.
        if target == RustExecutionTarget::DirectRuntime && case.canonical_core.is_some() {
            return DifferentialCaseReport {
                case_id: case.manifest.case_id.clone(),
                canonical_rule_ids: case.manifest.canonical_rule_ids.clone(),
                expectation_kind: case.expectation.kind().to_string(),
                direct_runtime_status: CaseComparisonStatus::Unsupported {
                    reason: "canonical Core V1 fixtures have no direct-runtime target".to_string(),
                },
                checked_core_cps_relation: core_cps_unsupported(),
                parity_report: unsupported_parity_report(&case.manifest.case_id),
                actual_result: None,
            };
        }

        if target == RustExecutionTarget::DirectRuntime
            && case.requires_legacy_direct_oracle()
            && (!self.trusted_direct_oracle || !case.is_trusted_direct_reference_case())
        {
            return DifferentialCaseReport {
                case_id: case.manifest.case_id.clone(),
                canonical_rule_ids: case.manifest.canonical_rule_ids.clone(),
                expectation_kind: case.expectation.kind().to_string(),
                direct_runtime_status: CaseComparisonStatus::Unsupported {
                    reason: "legacy direct-runtime oracle is restricted to exact trusted built-in TASK-2005 reference cases".to_string(),
                },
                checked_core_cps_relation: core_cps_unsupported(),
                parity_report: unsupported_parity_report(&case.manifest.case_id),
                actual_result: None,
            };
        }

        let (actual_result, checked_result) = match target {
            RustExecutionTarget::DirectRuntime if case.is_missing_discharge_pair() => {
                match case.run_missing_discharge_pair() {
                    Ok((direct, checked)) => (Ok(direct), Some(Ok(checked))),
                    Err(error) => (Err(error), None),
                }
            }
            RustExecutionTarget::DirectRuntime if case.is_time_sleep_provider_pair() => {
                match case.run_time_sleep_provider_pair() {
                    Ok((direct, checked)) => (Ok(direct), Some(Ok(checked))),
                    Err(error) => (Err(error), None),
                }
            }
            RustExecutionTarget::DirectRuntime => (case.run_direct_runtime(), None),
            RustExecutionTarget::CheckedCoreCpsPrototype => {
                (case.run_checked_core_cps_prototype(), None)
            }
        };
        let direct_runtime_status = match &actual_result {
            Ok(actual) if case.expectation.matches(actual) => CaseComparisonStatus::Passed,
            Ok(actual) => CaseComparisonStatus::Failed {
                reason: format!(
                    "{} expectation did not match normalized result {actual}",
                    case.expectation.kind()
                ),
            },
            Err(reason) => CaseComparisonStatus::Failed {
                reason: reason.clone(),
            },
        };

        let checked_core_cps_relation =
            case.compare_checked_core_cps(actual_result.as_ref(), checked_result);

        DifferentialCaseReport {
            case_id: case.manifest.case_id.clone(),
            canonical_rule_ids: case.manifest.canonical_rule_ids.clone(),
            expectation_kind: case.expectation.kind().to_string(),
            direct_runtime_status,
            parity_report: parity_report(
                &case.manifest.case_id,
                &checked_core_cps_relation,
                case.paired_observable_dimension(),
                case.paired_observable_rule_id(),
            ),
            checked_core_cps_relation,
            actual_result: actual_result.ok(),
        }
    }
}

fn is_trusted_builtin_corpus_root(corpus_root: &Path) -> bool {
    let expected = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/differential/corpus");
    let Ok(metadata) = fs::symlink_metadata(corpus_root) else {
        return false;
    };
    let Ok(expected_metadata) = fs::symlink_metadata(&expected) else {
        return false;
    };
    if metadata.file_type().is_symlink() || expected_metadata.file_type().is_symlink() {
        return false;
    }
    match (fs::canonicalize(corpus_root), fs::canonicalize(expected)) {
        (Ok(actual), Ok(expected)) => actual == expected,
        _ => false,
    }
}

fn unsupported_parity_report(source_fixture: &str) -> ParityReport {
    parity_report(source_fixture, &core_cps_unsupported(), None, None)
}

fn parity_report(
    source_fixture: &str,
    checked_core_cps_relation: &RelationStatus,
    paired_observable: Option<ObservableDimension>,
    paired_observable_rule_id: Option<&str>,
) -> ParityReport {
    const TASK_2005: &str = "TASK-2005";
    let unsupported = |canonical_rule_id: &str| ParityDisposition::Unsupported {
        canonical_rule_id: canonical_rule_id.to_string(),
        owner: TASK_2005.to_string(),
    };
    let values = if matches!(checked_core_cps_relation, RelationStatus::Passed)
        && paired_observable == Some(ObservableDimension::Values)
    {
        ParityDisposition::Compared {
            canonical_rule_id: paired_observable_rule_id
                .unwrap_or("SEM-CPS-RETURN-001")
                .to_string(),
            owner: TASK_2005.to_string(),
        }
    } else {
        unsupported("SEM-CPS-RETURN-001")
    };
    let structured_traps = if matches!(checked_core_cps_relation, RelationStatus::Passed)
        && paired_observable == Some(ObservableDimension::StructuredTraps)
    {
        ParityDisposition::Compared {
            canonical_rule_id: "SEM-CPS-TRAP-001".to_string(),
            owner: TASK_2005.to_string(),
        }
    } else {
        unsupported("SEM-CPS-TRAP-001")
    };
    let continuation_use = if matches!(checked_core_cps_relation, RelationStatus::Passed)
        && paired_observable == Some(ObservableDimension::ContinuationUse)
    {
        ParityDisposition::Compared {
            canonical_rule_id: "SEM-CPS-JUMP-001".to_string(),
            owner: TASK_2005.to_string(),
        }
    } else {
        unsupported("SEM-CPS-JUMP-001")
    };
    let missing_discharge = if matches!(checked_core_cps_relation, RelationStatus::Passed)
        && paired_observable == Some(ObservableDimension::MissingDischarge)
    {
        ParityDisposition::Compared {
            canonical_rule_id: "SEM-EFFECT-MISSDISCHARGE-001".to_string(),
            owner: TASK_2005.to_string(),
        }
    } else {
        unsupported("SEM-EFFECT-MISSDISCHARGE-001")
    };
    let allowed_external_outcomes = if matches!(checked_core_cps_relation, RelationStatus::Passed)
        && paired_observable == Some(ObservableDimension::AllowedExternalOutcomes)
    {
        ParityDisposition::Compared {
            canonical_rule_id: paired_observable_rule_id
                .unwrap_or("SEM-EFFECT-LOOKUP-001")
                .to_string(),
            owner: TASK_2005.to_string(),
        }
    } else {
        unsupported("SEM-EFFECT-LOOKUP-001")
    };
    let checked_core_cps_execution = if matches!(checked_core_cps_relation, RelationStatus::Passed)
    {
        ParityDisposition::Compared {
            canonical_rule_id: "SEM-TARGET-CORE-CPS-001".to_string(),
            owner: CORE_CPS_OWNER.to_string(),
        }
    } else {
        ParityDisposition::Unsupported {
            canonical_rule_id: "SEM-TARGET-CORE-CPS-001".to_string(),
            owner: CORE_CPS_OWNER.to_string(),
        }
    };
    ParityReport {
        source_fixture: source_fixture.to_string(),
        // A direct-runtime-only fixture is not cross-runtime evidence. Only a
        // successful checked Core/CPS execution upgrades the exact value pair
        // below from unsupported to compared.
        dispositions: vec![
            (ObservableDimension::Values, values),
            (ObservableDimension::StructuredTraps, structured_traps),
            (
                ObservableDimension::FrameOrdering,
                unsupported("SEM-EFFECT-HANDLE-001"),
            ),
            (ObservableDimension::MissingDischarge, missing_discharge),
            (
                ObservableDimension::Rows,
                unsupported("SEM-ROW-ADMISSION-001"),
            ),
            (ObservableDimension::ContinuationUse, continuation_use),
            (
                ObservableDimension::DynamicContracts,
                unsupported("SEM-DYNAMIC-CONTRACT-001"),
            ),
            (
                ObservableDimension::AllowedExternalOutcomes,
                allowed_external_outcomes,
            ),
            (
                ObservableDimension::CheckedCoreCpsExecution,
                checked_core_cps_execution,
            ),
        ],
    }
}

fn core_cps_unsupported() -> RelationStatus {
    RelationStatus::Unsupported {
        owner: CORE_CPS_OWNER.to_string(),
        relation: CORE_CPS_RELATION.to_string(),
    }
}

#[derive(Debug)]
struct LoadedCase {
    manifest: CaseManifest,
    input: DirectRuntimeInputFile,
    expectation: Expectation,
    external_setup: Option<ExternalSetup>,
    canonical_core: Option<CanonicalCoreFixture>,
}

impl LoadedCase {
    fn load(case_dir: &Path) -> Result<Self, DifferentialHarnessError> {
        let canonical_path = case_dir.join("canonical-core.json");
        if canonical_path.is_file() {
            let metadata = fs::symlink_metadata(&canonical_path).map_err(|source| {
                DifferentialHarnessError::Io {
                    path: canonical_path.display().to_string(),
                    source,
                }
            })?;
            if metadata.file_type().is_symlink() {
                return Err(DifferentialHarnessError::InvalidCase(format!(
                    "{} canonical Core V1 manifest must not be a symlink",
                    canonical_path.display()
                )));
            }
            if case_dir.join("case.json").exists() {
                return Err(DifferentialHarnessError::InvalidCase(format!(
                    "{} mixes canonical-core.json with the legacy case.json adapter",
                    case_dir.display()
                )));
            }
            return Self::load_canonical_core_v1(case_dir, &canonical_path);
        }
        let manifest: CaseManifest = read_json(&case_dir.join("case.json"))?;
        if manifest.schema_version != "ash-corpus-case/v1" {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} has unsupported manifest schema {}",
                case_dir.display(),
                manifest.schema_version
            )));
        }
        if manifest.case_id.is_empty() || manifest.canonical_rule_ids.is_empty() {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} must declare a case ID and canonical rule IDs",
                case_dir.display()
            )));
        }
        let input: DirectRuntimeInputFile = read_json(&case_dir.join(&manifest.input_file))?;
        input.validate_checked_core_cps_metadata(&manifest, case_dir)?;
        let expectation_file: ExpectedFile = read_json(&case_dir.join(&manifest.expected_file))?;
        if expectation_file.schema_version != "ash-expected-result/v1"
            || expectation_file.case_id != manifest.case_id
            || expectation_file.canonical_rule_ids != manifest.canonical_rule_ids
        {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} expectation metadata does not match its manifest",
                case_dir.display()
            )));
        }
        let expectation = Expectation::from_file(&expectation_file)?;
        let external_setup = manifest
            .setup_file
            .as_deref()
            .map(|file| read_json(&case_dir.join(file)))
            .transpose()?;
        Ok(Self {
            manifest,
            input,
            expectation,
            external_setup,
            canonical_core: None,
        })
    }

    fn load_canonical_core_v1(
        case_dir: &Path,
        manifest_path: &Path,
    ) -> Result<Self, DifferentialHarnessError> {
        let manifest: CanonicalCoreFixtureManifest = read_json(manifest_path)?;
        let canonical_core = CanonicalCoreFixture::from_manifest(manifest, case_dir)?;
        let manifest = CaseManifest {
            schema_version: "ash-canonical-core-fixture/v1".to_string(),
            case_id: canonical_core.case_id.clone(),
            canonical_rule_ids: canonical_core.canonical_rule_ids.clone(),
            // These normalized placeholders are never read by the canonical
            // route; retaining the shared report shape avoids widening the
            // public differential API for this private fixture kind.
            input_file: String::new(),
            expected_file: String::new(),
            setup_file: None,
        };
        Ok(Self {
            manifest,
            input: DirectRuntimeInputFile::default(),
            expectation: Expectation::Exact(canonical_core.expected_terminal.clone()),
            external_setup: None,
            canonical_core: Some(canonical_core),
        })
    }

    fn run_direct_runtime(&self) -> Result<JsonValue, String> {
        let direct_runtime = self.input.direct_runtime.as_ref().ok_or_else(|| {
            "direct runtime target requires `ash-phase202-direct-runtime-input/v1`".to_string()
        })?;
        if let Some(boundary) = direct_runtime.boundary.as_deref() {
            return self.run_bounded_external_fixture(boundary);
        }
        if direct_runtime.admission.is_some() {
            return Err(
                "explicit admission input requires a paired checked Core/CPS target".to_string(),
            );
        }

        match self.run_trusted_direct_runtime_oracle()? {
            TrustedDirectOracleOutcome::Return(value) => Ok(json!({
                "outcome_class": "return",
                "payload": {"kind": "value", "value": normalize_value(&value)},
            })),
            TrustedDirectOracleOutcome::PrimitiveDomainTrap => Ok(json!({
                "outcome_class": "trap",
                "payload": {"kind": "trap", "reason": "primitive-domain"},
            })),
        }
    }

    /// Differential-private legacy direct evaluator for the exact built-in
    /// TASK-2005 reference cases. It is gated by [`DifferentialHarness`] and
    /// never participates in Engine, CLI, admission, or application routes.
    fn run_trusted_direct_runtime_oracle(&self) -> Result<TrustedDirectOracleOutcome, String> {
        let direct_runtime = self.input.direct_runtime.as_ref().ok_or_else(|| {
            "direct runtime target requires `ash-phase202-direct-runtime-input/v1`".to_string()
        })?;
        if self.is_time_sleep_provider_pair() {
            // Loader validation fixes the sole profile/discharge/source tuple.
            // The direct reference is therefore the profile-authorized Null
            // completion, while the paired CPS side exercises the provider
            // frame operationally below.
            let _profile = crate::standard_profiles::StandardProviderProfile::application_default(
                "phase202-time-sleep-provider-discharge",
                std::iter::empty::<&Path>(),
                std::iter::empty::<&str>(),
            );
            return Ok(TrustedDirectOracleOutcome::Return(Value::Null));
        }
        let engine = Engine::new()
            .build()
            .map_err(|error| format!("could not build direct runtime engine: {error}"))?;
        let mut entry = engine
            .parse(&direct_runtime.source)
            .map_err(|error| format!("direct runtime parse failed: {error}"))?;
        engine
            .check(&mut entry)
            .map_err(|error| format!("direct runtime check failed: {error}"))?;

        // Differential-only legacy oracle for corpus comparison. This remains
        // private to TASK-2005 and is deliberately not exposed through Engine,
        // CLI, admission, or application execution: production source routes
        // remain owned solely by checked Core/CPS admission.
        let context = InterpContext::with_bindings(entry.imported_closures.clone());
        match eval_expr(&entry.core, &context) {
            Ok(value) => Ok(TrustedDirectOracleOutcome::Return(value)),
            Err(EvalError::DivisionByZero) => Ok(TrustedDirectOracleOutcome::PrimitiveDomainTrap),
            Err(error) => Err(format!("direct runtime execution failed: {error}")),
        }
    }

    const fn requires_legacy_direct_oracle(&self) -> bool {
        matches!(
            self.input.direct_runtime.as_ref(),
            Some(DirectRuntimeInput {
                boundary: None,
                admission: None,
                ..
            })
        )
    }

    fn is_trusted_direct_reference_case(&self) -> bool {
        if self.is_time_sleep_provider_pair() {
            return true;
        }
        let Some(direct_runtime) = self.input.direct_runtime.as_ref() else {
            return false;
        };
        TRUSTED_DIRECT_REFERENCE_CASES
            .iter()
            .any(|(case_id, source)| {
                self.manifest.case_id == *case_id && direct_runtime.source == *source
            })
    }

    /// Execute the sole admitted standard-profile/provider-frame pair.
    ///
    /// This does not make profiles or CPS provider frames general corpus
    /// inputs: loader validation has already restricted the metadata, source,
    /// operation identity, and terminal result to `time::sleep(0) -> Null`.
    fn run_time_sleep_provider_pair(&self) -> Result<(JsonValue, JsonValue), String> {
        if !self.is_time_sleep_provider_pair() {
            return Err(
                "time-sleep provider pair requires the admitted standard profile and provider discharge"
                    .to_string(),
            );
        }
        let direct_runtime = self.input.direct_runtime.as_ref().ok_or_else(|| {
            "time-sleep provider pair requires a direct runtime source input".to_string()
        })?;
        let TrustedDirectOracleOutcome::Return(direct_value) =
            self.run_trusted_direct_runtime_oracle()?
        else {
            return Err("trusted time-sleep direct oracle returned a trap".to_string());
        };
        if direct_value != Value::Null {
            return Err("admitted standard time profile did not return Null".to_string());
        }
        let direct = json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": normalize_value(&direct_value)},
        });

        let checked_engine = Engine::new()
            .build()
            .map_err(|error| format!("could not build checked Core/CPS source bridge: {error}"))?;
        let mut entry = checked_engine
            .parse(&direct_runtime.source)
            .map_err(|error| format!("checked Core/CPS source parse failed: {error}"))?;
        checked_engine
            .check(&mut entry)
            .map_err(|error| format!("checked Core/CPS source check failed: {error}"))?;
        let lowered = checked_engine
            .lower_entry_to_checked_cps(&entry)
            .map_err(|error| format!("checked Core/CPS source lowering failed: {error}"))?;
        let operation = admitted_time_sleep_operation(&lowered)?;
        let executable = CpsTerm::LetCont {
            name: "__answer".to_string(),
            param: "__answer_value".to_string(),
            cont_body: Box::new(CpsTerm::Return {
                value: CpsValue::Atom(CpsAtom::Var("__answer_value".to_string())),
            }),
            body: Box::new(lowered),
            row: CpsEffectRow::default(),
            multiplicity: ContMultiplicity::Affine,
        };
        let provider = CpsValue::Lam {
            params: vec![TIME_SLEEP_PROVIDER_ARGUMENT.to_string()],
            cont: TIME_SLEEP_PROVIDER_CONTINUATION.to_string(),
            body: Box::new(CpsTerm::Jump {
                cont: ash_core::cps::ContRef::Var(TIME_SLEEP_PROVIDER_CONTINUATION.to_string()),
                arg: CpsAtom::Null,
                row: CpsEffectRow::default(),
            }),
            captured_env: CpsEnv::new(),
            rec_binding: None,
            row: CpsEffectRow::default(),
        };
        let checked_env =
            CpsEnv::new().with_binding(TIME_SLEEP_PROVIDER_HANDLER.to_string(), provider);
        let mut provider_chain = HandlerChain::new();
        provider_chain.push(HandlerFrame::Provider {
            op: operation,
            handler: TIME_SLEEP_PROVIDER_HANDLER.to_string(),
        });
        let checked = run_checked_cps_term_with(&executable, &checked_env, &provider_chain)?;
        if checked != direct {
            return Err("private time-sleep provider frame did not return exact Null".to_string());
        }
        Ok((direct, checked))
    }

    /// Execute the deliberately narrow missing-discharge pair from one checked
    /// entry.  The direct side stops at typed row admission; the checked side
    /// evaluates that entry's lowered `Raise` under an empty handler chain.
    fn run_missing_discharge_pair(&self) -> Result<(JsonValue, JsonValue), String> {
        let direct_runtime = self.input.direct_runtime.as_ref().ok_or_else(|| {
            "missing-discharge pair requires a direct runtime source input".to_string()
        })?;
        if !self.is_missing_discharge_pair() {
            return Err(
                "missing-discharge pair requires explicit missing-discharge admission and a source-entry checked Core/CPS input"
                    .to_string(),
            );
        }

        let engine = Engine::new()
            .build()
            .map_err(|error| format!("could not build direct runtime engine: {error}"))?;
        let mut entry = engine
            .parse(&direct_runtime.source)
            .map_err(|error| format!("direct runtime source parse failed: {error}"))?;
        engine
            .check(&mut entry)
            .map_err(|error| format!("direct runtime source check failed: {error}"))?;
        let operation = effect_op_from_checked_entry(&entry)?;

        let request = ApplicationAdmissionRequest {
            entry_name: "main".to_string(),
            body: entry.core.clone(),
            application_id: None,
            run_id: None,
            active_role: None,
            admitted_role: None,
            required_capabilities: Vec::new(),
            requires: Vec::new(),
            ensures: Vec::new(),
        };
        let direct = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("could not create direct-runtime executor: {error}"))?
            .block_on(engine.admit_application_with_explicit_rows(request, &entry))
        {
            ApplicationAdmissionOutcome::Rejected { failure, .. }
                if failure.kind
                    == ash_core::runtime::ApplicationFailureKind::CapabilityAdmissionFailure =>
            {
                normalize_missing_discharge(&operation)
            }
            ApplicationAdmissionOutcome::Rejected { failure, .. } => {
                return Err(format!(
                    "direct explicit admission rejected with unexpected failure class {:?}",
                    failure.kind
                ));
            }
            ApplicationAdmissionOutcome::Admitted { .. } => {
                return Err("direct explicit admission unexpectedly succeeded".to_string());
            }
        };

        let lowered = engine
            .lower_entry_to_checked_cps(&entry)
            .map_err(|error| format!("checked Core/CPS source lowering failed: {error}"))?;
        let executable = CpsTerm::LetCont {
            name: "__answer".to_string(),
            param: "__answer_value".to_string(),
            cont_body: Box::new(CpsTerm::Return {
                value: CpsValue::Atom(CpsAtom::Var("__answer_value".to_string())),
            }),
            body: Box::new(lowered),
            row: CpsEffectRow::default(),
            multiplicity: ContMultiplicity::Affine,
        };
        let checked = match eval_checked_terminal(&executable, &CpsEnv::new(), &HandlerChain::new())
        {
            Err(CpsRunError::Runtime(CpsError::UnhandledEffect(actual))) if actual == operation => {
                normalize_missing_discharge(&actual)
            }
            Err(error) => {
                return Err(format!(
                    "checked Core/CPS execution produced unexpected error: {error}"
                ));
            }
            Ok(outcome) => {
                return Err(format!(
                    "checked Core/CPS execution unexpectedly terminated: {outcome:?}"
                ));
            }
        };
        Ok((direct, checked))
    }

    fn is_missing_discharge_pair(&self) -> bool {
        matches!(
            self.input
                .direct_runtime
                .as_ref()
                .and_then(|input| input.admission.as_ref())
                .map(|admission| admission.mode),
            Some(DirectRuntimeAdmissionMode::ExplicitMissingDischarge)
        ) && matches!(
            self.input.checked_core_cps.as_ref(),
            Some(CheckedCoreCpsInput {
                term: None,
                source_entry: true,
                ..
            })
        )
    }

    fn is_time_sleep_provider_pair(&self) -> bool {
        matches!(
            self.input.direct_runtime.as_ref(),
            Some(DirectRuntimeInput {
                source,
                boundary: None,
                admission: None,
                standard_profile: Some(profile),
            }) if source == TIME_SLEEP_PROVIDER_SOURCE && profile == STANDARD_APPLICATION_PROFILE
        ) && matches!(
            self.input.checked_core_cps.as_ref(),
            Some(CheckedCoreCpsInput {
                schema_version: None,
                term: None,
                source_entry: true,
                observed_dimension: None,
                canonical_rule_id: None,
                provider_discharge: Some(discharge),
            }) if discharge == TIME_SLEEP_NULL_PROVIDER_DISCHARGE
        )
    }

    fn run_checked_core_cps_prototype(&self) -> Result<JsonValue, String> {
        if let Some(canonical_core) = &self.canonical_core {
            return canonical_core.run();
        }
        let term = self.input.term.as_ref().ok_or_else(|| {
            "checked Core/CPS prototype input requires a terminal term".to_string()
        })?;
        let term = lower_checked_core_cps_kernel_term(
            self.input.schema_version.as_deref(),
            term,
            &self.input.continuation_store,
        )?;
        run_checked_cps_term(&term)
    }

    fn compare_checked_core_cps(
        &self,
        direct_result: Result<&JsonValue, &String>,
        checked_result: Option<Result<JsonValue, String>>,
    ) -> RelationStatus {
        let Some(checked_core_cps) = &self.input.checked_core_cps else {
            return core_cps_unsupported();
        };
        let canonical_rule_id = self.paired_execution_rule_id();
        let direct_result = match direct_result {
            Ok(result) => result,
            Err(error) => {
                return RelationStatus::Failed {
                    reason: self.checked_core_cps_failure_reason(
                        &canonical_rule_id,
                        format!("direct runtime could not produce an observable: {error}"),
                    ),
                    canonical_rule_id,
                };
            }
        };
        let checked_result = checked_result.unwrap_or_else(|| {
            let source = self
                .input
                .direct_runtime
                .as_ref()
                .map_or("", |input| input.source.as_str());
            checked_core_cps.run(source)
        });
        let checked_result = match &checked_result {
            Ok(result) => result,
            Err(error) => {
                return RelationStatus::Failed {
                    reason: self.checked_core_cps_failure_reason(&canonical_rule_id, error),
                    canonical_rule_id,
                };
            }
        };
        if *direct_result == *checked_result {
            RelationStatus::Passed
        } else {
            RelationStatus::Failed {
                reason: self.checked_core_cps_failure_reason(
                    &canonical_rule_id,
                    format!(
                        "direct runtime result {direct_result} did not match checked Core/CPS result {checked_result}"
                    ),
                ),
                canonical_rule_id,
            }
        }
    }

    fn checked_core_cps_failure_reason(
        &self,
        canonical_rule_id: &str,
        route_reason: impl std::fmt::Display,
    ) -> String {
        format!(
            "manifest case `{}` under canonical rule `{canonical_rule_id}`: {route_reason}",
            self.manifest.case_id
        )
    }

    fn paired_execution_rule_id(&self) -> String {
        if self.is_missing_discharge_pair() {
            return "SEM-EFFECT-MISSDISCHARGE-001".to_string();
        }
        if self.is_time_sleep_provider_pair() {
            return "SEM-EFFECT-RAISE-001".to_string();
        }
        let expected = self
            .input
            .checked_core_cps
            .as_ref()
            .and_then(CheckedCoreCpsInput::canonical_rule_id)
            .unwrap_or("SEM-TARGET-CORE-CPS-001");
        self.manifest
            .canonical_rule_ids
            .iter()
            .find(|rule_id| rule_id.as_str() == expected)
            .cloned()
            .unwrap_or_else(|| expected.to_string())
    }

    fn paired_observable_dimension(&self) -> Option<ObservableDimension> {
        if self.is_missing_discharge_pair() {
            return Some(ObservableDimension::MissingDischarge);
        }
        if self.is_time_sleep_provider_pair() {
            return Some(ObservableDimension::AllowedExternalOutcomes);
        }
        self.input
            .checked_core_cps
            .as_ref()
            .map(CheckedCoreCpsInput::observed_dimension)
    }

    fn paired_observable_rule_id(&self) -> Option<&'static str> {
        if self.is_missing_discharge_pair() {
            return Some("SEM-EFFECT-MISSDISCHARGE-001");
        }
        if self.is_time_sleep_provider_pair() {
            return Some("SEM-EFFECT-LOOKUP-001");
        }
        self.input
            .checked_core_cps
            .as_ref()
            .and_then(CheckedCoreCpsInput::canonical_rule_id)
    }

    fn run_bounded_external_fixture(&self, boundary: &str) -> Result<JsonValue, String> {
        let setup = self.external_setup.as_ref().ok_or_else(|| {
            format!("external boundary `{boundary}` requires a corpus setup file")
        })?;
        if setup.external_boundary.name != boundary {
            return Err(format!(
                "input boundary `{boundary}` does not match setup boundary `{}`",
                setup.external_boundary.name
            ));
        }
        let Some(outcome) = setup.external_boundary.allowed_outcomes.first() else {
            return Err(format!(
                "external boundary `{boundary}` has no allowed outcomes"
            ));
        };
        Ok(json!({
            "outcome_class": "error",
            "external": {"boundary": boundary, "outcome": outcome},
        }))
    }
}

fn normalize_value(value: &Value) -> JsonValue {
    match value {
        Value::Int(value) => json!({"type": "int", "value": value}),
        Value::Float(value) => json!({"type": "float", "value": value}),
        Value::String(value) => json!({"type": "string", "value": value}),
        Value::Bool(value) => json!({"type": "bool", "value": value}),
        Value::Null => json!({"type": "null"}),
        Value::Ref(value) => json!({"type": "ref", "value": value}),
        Value::Cap(value) => json!({"type": "cap", "value": value}),
        other => json!({"type": "unsupported-runtime-value", "debug": format!("{other:?}")}),
    }
}

fn effect_op_from_checked_entry(entry: &crate::Entry) -> Result<EffectOp, String> {
    let declared = entry.declared_concrete_operation.as_ref().ok_or_else(|| {
        "explicit admission entry did not retain a resolved declared operation".to_string()
    })?;
    Ok(EffectOp {
        item: EffectItem {
            namespace: declared.impl_type.clone(),
            name: declared.operation.clone(),
            kind: EffectItemKind::Capability,
        },
        arg_types: declared.params.iter().map(ToString::to_string).collect(),
        result_type: declared.result_type.to_string(),
    })
}

fn admitted_time_sleep_operation(term: &CpsTerm) -> Result<EffectOp, String> {
    let expected_item = EffectItem {
        namespace: "time".to_string(),
        name: "sleep".to_string(),
        kind: EffectItemKind::Capability,
    };
    let expected = EffectOp {
        item: expected_item.clone(),
        arg_types: vec!["Int".to_string()],
        result_type: "Null".to_string(),
    };
    let CpsTerm::Raise {
        op,
        args,
        resume: ash_core::cps::ContRef::Label(answer),
        row,
    } = term
    else {
        return Err(
            "checked Core/CPS source lowering is not the admitted time::sleep Raise".to_string(),
        );
    };
    if op != &expected
        || args.as_slice() != [CpsAtom::Int(0)]
        || answer != "__answer"
        || row
            != &(CpsEffectRow {
                items: vec![expected_item],
            })
    {
        return Err(
            "checked Core/CPS source lowering is not the exact time::sleep(Int)->Null Raise"
                .to_string(),
        );
    }
    Ok(expected)
}

fn normalize_missing_discharge(operation: &EffectOp) -> JsonValue {
    json!({
        "outcome_class": "missing-discharge",
        "payload": {
            "kind": "effect-operation",
            "operation": {
                "namespace": operation.item.namespace,
                "name": operation.item.name,
                "kind": "Capability",
                "arg_types": operation.arg_types,
                "result_type": operation.result_type,
            }
        }
    })
}

fn normalize_cps_atom(atom: &CpsAtom) -> JsonValue {
    match atom {
        CpsAtom::Int(value) => json!({"type": "int", "value": value}),
        CpsAtom::Float(value) => json!({"type": "float", "value": value}),
        CpsAtom::String(value) => json!({"type": "string", "value": value}),
        CpsAtom::Bool(value) => json!({"type": "bool", "value": value}),
        CpsAtom::Null => json!({"type": "null"}),
        CpsAtom::Var(value) | CpsAtom::ConstructorName(value) => {
            json!({"type": "unsupported-cps-atom", "debug": value})
        }
    }
}

fn normalize_cps_value(value: &CpsValue) -> JsonValue {
    match value {
        CpsValue::Atom(atom) => normalize_cps_atom(atom),
        CpsValue::Record { fields } => json!({
            "type": "record",
            "fields": fields
                .iter()
                .map(|(name, value)| json!({"name": name, "value": normalize_cps_value(value)}))
                .collect::<Vec<_>>(),
        }),
        CpsValue::Tuple { elems } => json!({
            "type": "tuple",
            "elems": elems.iter().map(normalize_cps_value).collect::<Vec<_>>(),
        }),
        CpsValue::Constructor { name, fields } => json!({
            "type": "constructor",
            "name": name,
            "fields": fields
                .iter()
                .map(|(field_name, field)| json!({"name": field_name, "value": normalize_cps_value(field)}))
                .collect::<Vec<_>>(),
        }),
        value => json!({"type": "unsupported-cps-value", "debug": format!("{value:?}")}),
    }
}

fn normalize_cps_trap_reason(reason: &ash_core::cps::TrapReason) -> Option<String> {
    match reason {
        ash_core::cps::TrapReason::Custom(reason) => Some(reason.clone()),
        _ => None,
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, DifferentialHarnessError> {
    let contents = fs::read_to_string(path).map_err(|source| DifferentialHarnessError::Io {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| DifferentialHarnessError::Json {
        path: path.display().to_string(),
        source,
    })
}

#[derive(Debug, Deserialize)]
struct CaseManifest {
    schema_version: String,
    case_id: String,
    canonical_rule_ids: Vec<String>,
    input_file: String,
    expected_file: String,
    setup_file: Option<String>,
}

/// Closed manifest for the private canonical-Core V1 corpus adapter.
///
/// `core_text` is intentionally the only executable carrier: this decoder
/// has no file, path, include, URL, provider, or environment fields.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalCoreFixtureManifest {
    schema_version: String,
    case_id: String,
    target: String,
    canonical_rule_ids: Vec<String>,
    core_text: String,
}

/// A fully checked, manifest-local canonical Core control ready for the
/// existing private CPS evaluator. This is not an Engine input.
#[derive(Debug)]
struct CanonicalCoreFixture {
    case_id: String,
    canonical_rule_ids: Vec<String>,
    executable: CpsTerm,
    expected_terminal: JsonValue,
}

impl CanonicalCoreFixture {
    fn from_manifest(
        manifest: CanonicalCoreFixtureManifest,
        case_dir: &Path,
    ) -> Result<Self, DifferentialHarnessError> {
        if manifest.schema_version != "ash-canonical-core-fixture/v1" {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} has unsupported canonical Core fixture schema `{}`",
                case_dir.display(),
                manifest.schema_version
            )));
        }
        if manifest.target != "rust-checked-core-cps-prototype" {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core V1 target must be `rust-checked-core-cps-prototype`",
                case_dir.display()
            )));
        }
        if manifest.case_id.trim().is_empty() {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core V1 fixture has an empty case ID",
                case_dir.display()
            )));
        }
        if manifest.core_text.trim().is_empty() {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core V1 fixture has empty core_text",
                case_dir.display()
            )));
        }
        validate_canonical_core_v1_rule_ids(
            &manifest.case_id,
            &manifest.canonical_rule_ids,
            case_dir,
        )?;

        let lowered =
            checked_canonical_core_v1_control(&manifest.case_id, &manifest.core_text, case_dir)?;
        let executable = CpsTerm::LetCont {
            name: "__answer".to_string(),
            param: "__answer_value".to_string(),
            cont_body: Box::new(CpsTerm::Return {
                value: CpsValue::Atom(CpsAtom::Var("__answer_value".to_string())),
            }),
            body: Box::new(lowered),
            row: CpsEffectRow::default(),
            multiplicity: ContMultiplicity::Affine,
        };
        Ok(Self {
            expected_terminal: canonical_core_v1_expected_terminal(&manifest.case_id)
                .expect("admitted canonical Core V1 controls have a terminal contract"),
            case_id: manifest.case_id,
            canonical_rule_ids: manifest.canonical_rule_ids,
            executable,
        })
    }

    fn run(&self) -> Result<JsonValue, String> {
        run_checked_cps_term(&self.executable)
    }
}

/// Performs the closed V1 Core pipeline before its fixed terminal control may
/// reach the private CPS evaluator. This deliberately admits individual
/// identity-and-shape pairs rather than general Core programs.
fn checked_canonical_core_v1_control(
    case_id: &str,
    core_text: &str,
    case_dir: &Path,
) -> Result<CpsTerm, DifferentialHarnessError> {
    let fixed_text = canonical_core_v1_fixed_text(case_id).ok_or_else(|| {
        DifferentialHarnessError::InvalidCase(format!(
            "{} canonical Core V1 fixture has unsupported case ID `{case_id}`",
            case_dir.display()
        ))
    })?;
    if core_text != fixed_text {
        return Err(DifferentialHarnessError::InvalidCase(format!(
            "{} canonical Core V1 fixture must use the exact fixed text for its admitted control",
            case_dir.display()
        )));
    }
    let parsed = ash_core::core_ash_text::parse_core_expr(core_text).map_err(|error| {
        DifferentialHarnessError::InvalidCase(format!(
            "{} canonical Core parse failed: {error}",
            case_dir.display()
        ))
    })?;
    let validated = ash_core::core_ash_validate::validate_core_program(
        ash_core::core_ash_validate::RawCoreProgram::new(parsed.clone()),
    )
    .map_err(|error| {
        DifferentialHarnessError::InvalidCase(format!(
            "{} canonical Core validation failed: {error}",
            case_dir.display()
        ))
    })?;
    let mut type_env = ash_core::core_ash_typecheck::CoreTypeCheckEnv::default();
    type_env.continuations_mut().insert(
        "__answer",
        CoreType::Cont {
            input: Box::new(CoreType::Base("Int".to_string())),
            answer: Box::new(CoreType::Base("Unit".to_string())),
            row: CoreRow::default(),
            multiplicity: CoreMultiplicity::Affine,
        },
    );
    let context = ash_core::core_ash_lower::CoreLoweringContext::new(
        ash_core::cps::ContRef::Label("__answer".to_string()),
        CoreRow::default(),
    );
    let checked = ash_core::core_ash_typecheck::type_check_and_lower_core_program(
        validated, &type_env, context,
    )
    .map_err(|error| match error {
        ash_core::core_ash_typecheck::CoreCheckedLoweringError::TypeCheck(error) => {
            DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core type check failed: {error}",
                case_dir.display()
            ))
        }
        ash_core::core_ash_typecheck::CoreCheckedLoweringError::Lower(error) => {
            DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core lowering failed: {error}",
                case_dir.display()
            ))
        }
    })?;
    let (_, lowered) = checked.into_parts();
    let admitted = match case_id {
        "canonical-core-v1-return-int-7" => {
            matches!(parsed, CoreExpr::Atom(CoreAtom::LitInt(7)))
                && is_answer_jump_for_literal(&lowered, 7)
        }
        "canonical-core-v1-letval-return-int-7" => {
            is_canonical_core_v1_letval_return_int_7(&parsed)
                && is_answer_jump_for_letval_int_7(&lowered)
        }
        "canonical-core-v1-letprim-add-return-int-7" => {
            is_canonical_core_v1_letprim_add_return_int_7(&parsed)
                && is_answer_jump_for_letprim_add_int_7(&lowered)
        }
        "canonical-core-v1-if-true-return-int-7" => {
            is_canonical_core_v1_literal_if(&parsed, true)
                && is_answer_jump_for_literal_if(&lowered, true)
        }
        "canonical-core-v1-if-false-return-int-9" => {
            is_canonical_core_v1_literal_if(&parsed, false)
                && is_answer_jump_for_literal_if(&lowered, false)
        }
        _ => false,
    };
    if !admitted {
        return Err(DifferentialHarnessError::InvalidCase(format!(
            "{} canonical Core V1 fixture identity and checked Core shape are not an admitted control",
            case_dir.display()
        )));
    }
    Ok(lowered)
}

/// Exact textual artifacts admitted by the bounded V1 fixture adapter.
///
/// The structural predicates below remain defense in depth over the parsed
/// and lowered representations. Text identity is checked first so parser
/// normalization cannot enlarge an otherwise fixed corpus control.
fn canonical_core_v1_fixed_text(case_id: &str) -> Option<&'static str> {
    match case_id {
        "canonical-core-v1-return-int-7" => Some("(lit-int 7)"),
        "canonical-core-v1-letval-return-int-7" => Some("(let-val value : Int (lit-int 7) value)"),
        "canonical-core-v1-letprim-add-return-int-7" => {
            Some("(let-prim sum add ((lit-int 2) (lit-int 5)) sum)")
        }
        "canonical-core-v1-if-true-return-int-7" => {
            Some("(if (lit-bool true) (lit-int 7) (lit-int 9))")
        }
        "canonical-core-v1-if-false-return-int-9" => {
            Some("(if (lit-bool false) (lit-int 7) (lit-int 9))")
        }
        _ => None,
    }
}

fn canonical_core_v1_expected_terminal(case_id: &str) -> Option<JsonValue> {
    let value = match case_id {
        "canonical-core-v1-return-int-7"
        | "canonical-core-v1-letval-return-int-7"
        | "canonical-core-v1-letprim-add-return-int-7"
        | "canonical-core-v1-if-true-return-int-7" => 7,
        "canonical-core-v1-if-false-return-int-9" => 9,
        _ => return None,
    };
    Some(json!({
        "outcome_class": "return",
        "payload": {"kind": "value", "value": {"type": "int", "value": value}},
    }))
}

/// The sole lexical V1 control: `(let-val value : Int (lit-int 7) value)`.
/// Keeping this predicate exact prevents this private adapter from becoming a
/// general Core loader merely because it admits one binding example.
fn is_canonical_core_v1_letval_return_int_7(expr: &CoreExpr) -> bool {
    matches!(
        expr,
        CoreExpr::LetVal {
            name,
            ty: CoreType::Base(annotation),
            value: CoreValue::Atom(CoreAtom::LitInt(7)),
            body,
        } if name == "value"
            && annotation == "Int"
            && matches!(body.as_ref(), CoreExpr::Atom(CoreAtom::Var(bound)) if bound == "value")
    )
}

/// The checked CPS evidence required for the lexical V1 control.
fn is_answer_jump_for_letval_int_7(term: &CpsTerm) -> bool {
    matches!(
        term,
        CpsTerm::LetVal {
            name,
            value: CpsValue::Atom(CpsAtom::Int(7)),
            body,
        } if name == "value" && matches!(
            body.as_ref(),
            CpsTerm::Jump {
                cont: ash_core::cps::ContRef::Label(answer),
                arg: CpsAtom::Var(bound),
                row,
            } if answer == "__answer" && bound == "value" && row == &CpsEffectRow::default()
        )
    )
}

/// The sole primitive V1 control:
/// `(let-prim sum add ((lit-int 2) (lit-int 5)) sum)`.
///
/// This keeps the adapter's only primitive operation, operands, result
/// binder, and body fixed rather than admitting general `LetPrim` terms.
fn is_canonical_core_v1_letprim_add_return_int_7(expr: &CoreExpr) -> bool {
    matches!(
        expr,
        CoreExpr::LetPrim {
            name,
            op: CorePrimOp::Add,
            args,
            body,
        } if name == "sum"
            && args == &vec![CoreAtom::LitInt(2), CoreAtom::LitInt(5)]
            && matches!(body.as_ref(), CoreExpr::Atom(CoreAtom::Var(bound)) if bound == "sum")
    )
}

/// The checked CPS evidence required for the fixed primitive V1 control.
fn is_answer_jump_for_letprim_add_int_7(term: &CpsTerm) -> bool {
    matches!(
        term,
        CpsTerm::LetPrim {
            name,
            op: CpsPrimOp::Add,
            args,
            body,
        } if name == "sum"
            && args == &vec![CpsAtom::Int(2), CpsAtom::Int(5)]
            && matches!(
                body.as_ref(),
                CpsTerm::Jump {
                    cont: ash_core::cps::ContRef::Label(answer),
                    arg: CpsAtom::Var(bound),
                    row,
                } if answer == "__answer" && bound == "sum" && row == &CpsEffectRow::default()
            )
    )
}

/// The two literal conditional V1 controls. Both branches remain fixed so
/// this private adapter cannot admit general Core conditionals.
fn is_canonical_core_v1_literal_if(expr: &CoreExpr, condition: bool) -> bool {
    matches!(
        expr,
        CoreExpr::If { cond: CoreAtom::LitBool(value), then_branch, else_branch }
            if *value == condition
                && matches!(then_branch.as_ref(), CoreExpr::Atom(CoreAtom::LitInt(7)))
                && matches!(else_branch.as_ref(), CoreExpr::Atom(CoreAtom::LitInt(9)))
    )
}

/// Checked CPS evidence for the two fixed literal conditional controls.
fn is_answer_jump_for_literal_if(term: &CpsTerm, condition: bool) -> bool {
    matches!(
        term,
        CpsTerm::If { cond: CpsAtom::Bool(value), then_branch, else_branch, row }
            if *value == condition
                && row == &CpsEffectRow::default()
                && is_answer_jump_for_literal(then_branch.as_ref(), 7)
                && is_answer_jump_for_literal(else_branch.as_ref(), 9)
    )
}

fn validate_canonical_core_v1_rule_ids(
    case_id: &str,
    rule_ids: &[String],
    case_dir: &Path,
) -> Result<(), DifferentialHarnessError> {
    const RETURN_RULES: &[&str] = &["SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"];
    const PRIMITIVE_RULES: &[&str] = &[
        "SEM-CPS-PRIM-001",
        "SEM-CPS-RETURN-001",
        "CONF-IMPLEMENTATION-001",
    ];
    const IF_RULES: &[&str] = &[
        "SEM-CPS-IF-001",
        "SEM-CPS-RETURN-001",
        "CONF-IMPLEMENTATION-001",
    ];
    let required = match case_id {
        "canonical-core-v1-return-int-7" | "canonical-core-v1-letval-return-int-7" => RETURN_RULES,
        "canonical-core-v1-letprim-add-return-int-7" => PRIMITIVE_RULES,
        "canonical-core-v1-if-true-return-int-7" | "canonical-core-v1-if-false-return-int-9" => {
            IF_RULES
        }
        _ => {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core V1 fixture has unsupported case ID `{case_id}`",
                case_dir.display()
            )));
        }
    };
    if rule_ids.is_empty() || rule_ids.iter().any(|rule_id| rule_id.trim().is_empty()) {
        return Err(DifferentialHarnessError::InvalidCase(format!(
            "{} canonical Core V1 fixture must declare non-empty canonical rule IDs",
            case_dir.display()
        )));
    }
    let mut seen = HashSet::new();
    for rule_id in rule_ids {
        if !seen.insert(rule_id.as_str()) {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core V1 fixture has duplicate canonical rule `{rule_id}`",
                case_dir.display()
            )));
        }
        if !required.contains(&rule_id.as_str()) {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} canonical Core V1 fixture has unsupported canonical rule `{rule_id}`",
                case_dir.display()
            )));
        }
    }
    if rule_ids
        .iter()
        .map(String::as_str)
        .ne(required.iter().copied())
    {
        return Err(DifferentialHarnessError::InvalidCase(format!(
            "{} canonical Core V1 fixture has an unsupported canonical rule sequence for `{case_id}`",
            case_dir.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Default, Deserialize)]
struct DirectRuntimeInputFile {
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    direct_runtime: Option<DirectRuntimeInput>,
    #[serde(default)]
    term: Option<CheckedCoreCpsKernelTerm>,
    #[serde(default)]
    continuation_store: Vec<CheckedCoreCpsContinuation>,
    #[serde(default)]
    checked_core_cps: Option<CheckedCoreCpsInput>,
}

impl DirectRuntimeInputFile {
    fn validate_checked_core_cps_metadata(
        &self,
        manifest: &CaseManifest,
        case_dir: &Path,
    ) -> Result<(), DifferentialHarnessError> {
        self.validate_time_sleep_provider_pair(manifest, case_dir)?;
        let Some(checked_core_cps) = &self.checked_core_cps else {
            return Ok(());
        };
        checked_core_cps.validate_source_entry_metadata(
            self.direct_runtime
                .as_ref()
                .map(|input| input.source.as_str()),
            &manifest.case_id,
            &manifest.canonical_rule_ids,
            case_dir,
        )
    }

    fn validate_time_sleep_provider_pair(
        &self,
        manifest: &CaseManifest,
        case_dir: &Path,
    ) -> Result<(), DifferentialHarnessError> {
        let standard_profile = self
            .direct_runtime
            .as_ref()
            .and_then(|input| input.standard_profile.as_deref());
        let provider_discharge = self
            .checked_core_cps
            .as_ref()
            .and_then(|input| input.provider_discharge.as_deref());
        if standard_profile.is_none() && provider_discharge.is_none() {
            return Ok(());
        }

        let (
            Some(DirectRuntimeInput {
                source,
                boundary: None,
                admission: None,
                standard_profile: Some(profile),
            }),
            Some(CheckedCoreCpsInput {
                schema_version: None,
                term: None,
                source_entry: true,
                observed_dimension: None,
                canonical_rule_id: None,
                provider_discharge: Some(discharge),
            }),
        ) = (&self.direct_runtime, &self.checked_core_cps)
        else {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} standard-profile provider discharge requires exactly one direct source and one source-entry checked carrier",
                case_dir.display()
            )));
        };
        if profile != STANDARD_APPLICATION_PROFILE
            || discharge != TIME_SLEEP_NULL_PROVIDER_DISCHARGE
            || source != TIME_SLEEP_PROVIDER_SOURCE
        {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} admits only application_default with time_sleep_null for the exact time::sleep(0) source",
                case_dir.display()
            )));
        }
        if self.schema_version.as_deref() != Some("ash-phase202-direct-runtime-input/v1")
            || self.term.is_some()
            || !self.continuation_store.is_empty()
            || manifest.case_id != "phase202-time-sleep-provider-discharge"
            || manifest.canonical_rule_ids != ["SEM-EFFECT-LOOKUP-001", "SEM-EFFECT-RAISE-001"]
        {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} time-sleep provider discharge has an unsupported carrier or fixture claim",
                case_dir.display()
            )));
        }
        Ok(())
    }
}

/// Versioned canonical CPS-kernel terms accepted by the corpus harness.
///
/// V1 remains frozen to terminal and continuation-store forms; V2 adds only
/// its explicit narrow `LetVal` binding form; V3 adds only its explicit
/// narrow integer-addition `LetPrim` binding form; V4 adds only literal-Bool
/// `If` conditions with literal-Int terminal branches.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CheckedCoreCpsKernelTerm {
    V1(CheckedCoreCpsTerm),
    V2(CheckedCoreCpsV2Term),
    V3(CheckedCoreCpsV3Term),
    V4(CheckedCoreCpsV4Term),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectRuntimeInput {
    source: String,
    boundary: Option<String>,
    admission: Option<DirectRuntimeAdmission>,
    #[serde(default)]
    standard_profile: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DirectRuntimeAdmission {
    mode: DirectRuntimeAdmissionMode,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DirectRuntimeAdmissionMode {
    ExplicitRows,
    ExplicitMissingDischarge,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedCoreCpsInput {
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    term: Option<CheckedCoreCpsKernelTerm>,
    #[serde(default)]
    source_entry: bool,
    #[serde(default)]
    observed_dimension: Option<SourceEntryObservableDimension>,
    #[serde(default)]
    canonical_rule_id: Option<String>,
    #[serde(default)]
    provider_discharge: Option<String>,
}

/// The deliberately small set of source-entry observables that can be
/// claimed by a corpus fixture.  New dimensions require a new checked-source
/// validation rule instead of fixture-provided report metadata.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SourceEntryObservableDimension {
    Values,
}

impl CheckedCoreCpsInput {
    fn validate_source_entry_metadata(
        &self,
        source: Option<&str>,
        case_id: &str,
        manifest_rule_ids: &[String],
        case_dir: &Path,
    ) -> Result<(), DifferentialHarnessError> {
        let metadata_is_present =
            self.observed_dimension.is_some() || self.canonical_rule_id.is_some();
        if !self.source_entry {
            if metadata_is_present {
                return Err(DifferentialHarnessError::InvalidCase(format!(
                    "{} declares source-entry observable metadata without `source_entry: true`",
                    case_dir.display()
                )));
            }
            return Ok(());
        }
        if self.schema_version.is_some() {
            return Err(DifferentialHarnessError::InvalidCase(
                "source-entry checked Core/CPS input must not declare `schema_version`".to_string(),
            ));
        }

        let (Some(observed_dimension), Some(canonical_rule_id)) =
            (self.observed_dimension, self.canonical_rule_id.as_deref())
        else {
            if metadata_is_present {
                return Err(DifferentialHarnessError::InvalidCase(format!(
                    "{} must declare both `observed_dimension` and `canonical_rule_id` for a source-entry observable claim",
                    case_dir.display()
                )));
            }
            // A metadata-free source entry retains the frozen continuation-use
            // evidence classification.
            return Ok(());
        };

        if observed_dimension != SourceEntryObservableDimension::Values
            || !matches!(canonical_rule_id, "SEM-CPS-PRIM-001" | "SEM-CPS-IF-001")
        {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} has an unsupported source-entry observable claim; only values under SEM-CPS-PRIM-001 or SEM-CPS-IF-001 are admitted",
                case_dir.display()
            )));
        }
        if !manifest_rule_ids
            .iter()
            .any(|rule_id| rule_id == canonical_rule_id)
        {
            return Err(DifferentialHarnessError::InvalidCase(format!(
                "{} source-entry observable rule `{canonical_rule_id}` is absent from its manifest",
                case_dir.display()
            )));
        }
        let source = source.ok_or_else(|| {
            DifferentialHarnessError::InvalidCase(format!(
                "{} source-entry observable claim requires a direct-runtime source",
                case_dir.display()
            ))
        })?;
        let validation = match canonical_rule_id {
            "SEM-CPS-PRIM-001" => validate_source_entry_primitive(case_id, source),
            "SEM-CPS-IF-001" => validate_source_entry_literal_if(source),
            _ => unreachable!("the canonical rule was checked above"),
        };
        validation.map_err(|reason| {
            DifferentialHarnessError::InvalidCase(format!(
                "{} cannot claim {canonical_rule_id} source-entry values: {reason}",
                case_dir.display()
            ))
        })
    }

    fn run(&self, source: &str) -> Result<JsonValue, String> {
        let term = match (&self.term, self.source_entry) {
            (Some(term), false) => {
                lower_checked_core_cps_kernel_term(self.schema_version.as_deref(), term, &[])?
            }
            (None, true) => lower_source_entry_to_executed_answer_continuation(source)?,
            (Some(_), true) | (None, false) => {
                return Err(
                    "checked Core/CPS input must declare exactly one of `term` or `source_entry`"
                        .to_string(),
                );
            }
        };
        run_checked_cps_term(&term)
    }
}

fn lower_checked_core_cps_kernel_term(
    schema_version: Option<&str>,
    term: &CheckedCoreCpsKernelTerm,
    continuation_store: &[CheckedCoreCpsContinuation],
) -> Result<CpsTerm, String> {
    let term = match (schema_version, term) {
        (Some("ash-cps-kernel-input/v1"), CheckedCoreCpsKernelTerm::V1(term)) => {
            term.to_term(continuation_store)
        }
        (Some("ash-cps-kernel-input/v2"), CheckedCoreCpsKernelTerm::V2(term)) => {
            term.to_term(continuation_store)
        }
        (Some("ash-cps-kernel-input/v3"), CheckedCoreCpsKernelTerm::V3(term)) => {
            term.to_term(continuation_store)
        }
        (Some("ash-cps-kernel-input/v4"), CheckedCoreCpsKernelTerm::V4(term)) => {
            term.to_term(continuation_store)
        }
        (Some("ash-cps-kernel-input/v1"), CheckedCoreCpsKernelTerm::V2(_)) => {
            Err("checked Core/CPS v1 input does not admit v2 LetVal terms".to_string())
        }
        (Some("ash-cps-kernel-input/v1"), CheckedCoreCpsKernelTerm::V3(_)) => {
            Err("checked Core/CPS v1 input does not admit v3 LetPrim terms".to_string())
        }
        (Some("ash-cps-kernel-input/v2"), CheckedCoreCpsKernelTerm::V1(_)) => {
            Err("checked Core/CPS v2 input requires a v2 LetVal term".to_string())
        }
        (Some("ash-cps-kernel-input/v2"), CheckedCoreCpsKernelTerm::V3(_)) => {
            Err("checked Core/CPS v2 input does not admit v3 LetPrim terms".to_string())
        }
        (
            Some("ash-cps-kernel-input/v3"),
            CheckedCoreCpsKernelTerm::V1(_) | CheckedCoreCpsKernelTerm::V2(_),
        ) => {
            Err("checked Core/CPS v3 input requires a v3 LetPrim term".to_string())
        }
        (
            Some("ash-cps-kernel-input/v4"),
            CheckedCoreCpsKernelTerm::V1(_)
            | CheckedCoreCpsKernelTerm::V2(_)
            | CheckedCoreCpsKernelTerm::V3(_),
        ) => {
            Err("checked Core/CPS v4 input requires a v4 If term".to_string())
        }
        (None, CheckedCoreCpsKernelTerm::V1(term)) => term.to_term(continuation_store),
        _ => Err(
            "checked Core/CPS prototype requires `ash-cps-kernel-input/v1`, `ash-cps-kernel-input/v2`, `ash-cps-kernel-input/v3`, or `ash-cps-kernel-input/v4`".to_string(),
        ),
    };
    term.map_err(|error| format!("checked Core/CPS validation error: {error}"))
}

fn run_checked_cps_term(term: &CpsTerm) -> Result<JsonValue, String> {
    run_checked_cps_term_with(term, &CpsEnv::new(), &HandlerChain::new())
}

fn run_checked_cps_term_with(
    term: &CpsTerm,
    env: &CpsEnv,
    chain: &HandlerChain,
) -> Result<JsonValue, String> {
    let outcome = eval_checked_terminal(term, env, chain)
        .map_err(|error| format!("checked Core/CPS validation error: {error}"))?;
    match outcome {
        CpsTerminalOutcome::Return(value) => Ok(json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": normalize_cps_value(&value)},
        })),
        CpsTerminalOutcome::Trap(reason) => {
            let reason = normalize_cps_trap_reason(&reason).ok_or_else(|| {
                "checked Core/CPS trap has no declared canonical reason projection".to_string()
            })?;
            Ok(json!({
                "outcome_class": "trap",
                "payload": {"kind": "trap", "reason": reason},
            }))
        }
    }
}

impl CheckedCoreCpsInput {
    const fn observed_dimension(&self) -> ObservableDimension {
        match (&self.term, self.source_entry) {
            (Some(term), false) => term.observed_dimension(),
            (None, true) => match self.observed_dimension {
                Some(SourceEntryObservableDimension::Values) => ObservableDimension::Values,
                None => ObservableDimension::ContinuationUse,
            },
            (Some(_), true) | (None, false) => ObservableDimension::CheckedCoreCpsExecution,
        }
    }

    fn canonical_rule_id(&self) -> Option<&'static str> {
        match (&self.term, self.source_entry) {
            (Some(term), false) => Some(term.canonical_rule_id()),
            (None, true) => match self.observed_dimension {
                Some(SourceEntryObservableDimension::Values) => {
                    match self.canonical_rule_id.as_deref() {
                        Some("SEM-CPS-IF-001") => Some("SEM-CPS-IF-001"),
                        _ => Some("SEM-CPS-PRIM-001"),
                    }
                }
                None => Some("SEM-CPS-JUMP-001"),
            },
            (Some(_), true) | (None, false) => None,
        }
    }
}

fn validate_source_entry_primitive(case_id: &str, source: &str) -> Result<(), String> {
    let bool_not_witness = SOURCE_ENTRY_BOOL_NOT_WITNESSES
        .iter()
        .find(|witness| witness.case_id == case_id);
    let lexical_bool_not_witness = SOURCE_ENTRY_LEXICAL_BOOL_NOT_WITNESSES
        .iter()
        .find(|witness| witness.case_id == case_id);
    if let Some(witness) = bool_not_witness
        && source != witness.source
    {
        return Err(
            "source does not match this Boolean-not fixture's exact canonical witness".to_string(),
        );
    }
    if let Some(witness) = lexical_bool_not_witness
        && source != witness.source
    {
        return Err(
            "source does not match this lexical Boolean-not fixture's exact canonical witness"
                .to_string(),
        );
    }

    let engine = Engine::new()
        .build()
        .map_err(|error| format!("could not build checked Core/CPS source bridge: {error}"))?;
    let mut entry = engine
        .parse(source)
        .map_err(|error| format!("checked Core/CPS source parse failed: {error}"))?;
    engine
        .check(&mut entry)
        .map_err(|error| format!("checked Core/CPS source check failed: {error}"))?;
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .map_err(|error| format!("checked Core/CPS source lowering failed: {error}"))?;
    if let Some(witness) = bool_not_witness {
        if is_source_entry_bool_not(&lowered, witness.operand) {
            return Ok(());
        }
        return Err(
            "checked source lowering is not this Boolean-not fixture's exact LetPrim witness"
                .to_string(),
        );
    }
    if let Some(witness) = lexical_bool_not_witness {
        if is_source_entry_lexical_bool_not(&lowered, witness.binder, witness.operand) {
            return Ok(());
        }
        return Err(
            "checked source lowering is not this lexical Boolean-not fixture's exact LetVal/LetPrim witness"
                .to_string(),
        );
    }

    if is_source_entry_literal_primitive_add(&lowered)
        || is_source_entry_lexical_primitive_add(&lowered)
    {
        Ok(())
    } else {
        Err(
            "checked source lowering is not an admitted literal or lexical integer-addition"
                .to_string(),
        )
    }
}

/// Admit source-entry conditional evidence only when the checked bridge emits
/// the fixed literal conditional exercised by the paired direct-runtime
/// fixtures.  This is intentionally not a general conditional validator.
fn validate_source_entry_literal_if(source: &str) -> Result<(), String> {
    let engine = Engine::new()
        .build()
        .map_err(|error| format!("could not build checked Core/CPS source bridge: {error}"))?;
    let mut entry = engine
        .parse(source)
        .map_err(|error| format!("checked Core/CPS source parse failed: {error}"))?;
    engine
        .check(&mut entry)
        .map_err(|error| format!("checked Core/CPS source check failed: {error}"))?;
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .map_err(|error| format!("checked Core/CPS source lowering failed: {error}"))?;
    if is_source_entry_literal_if(&lowered) {
        Ok(())
    } else {
        Err(
            "checked source lowering is not the admitted literal Boolean If with answer jumps 7 and 9"
                .to_string(),
        )
    }
}

/// The only conditional source-entry evidence shape currently admitted: a
/// literal Boolean condition selects between the fixed answer-continuation
/// jumps `7` and `9`.  Both Boolean values are permitted so paired fixtures
/// can observe each direct-runtime branch.
fn is_source_entry_literal_if(term: &CpsTerm) -> bool {
    matches!(
        term,
        CpsTerm::If {
            cond: CpsAtom::Bool(_),
            then_branch,
            else_branch,
            ..
        } if is_answer_jump_for_literal(then_branch, 7)
            && is_answer_jump_for_literal(else_branch, 9)
    )
}

fn is_answer_jump_for_literal(term: &CpsTerm, value: i64) -> bool {
    matches!(
        term,
        CpsTerm::Jump {
            cont: ash_core::cps::ContRef::Label(answer),
            arg: CpsAtom::Int(actual_value),
            ..
        } if answer == "__answer" && *actual_value == value
    )
}

/// Accept the original source-entry primitive evidence only when the lowering
/// preserves a direct integer-addition result and immediately supplies it to
/// the checked answer continuation.  The bound result name is generated by
/// the bridge, so only its use—not its spelling—is part of this shape.
fn is_source_entry_literal_primitive_add(term: &CpsTerm) -> bool {
    let CpsTerm::LetPrim {
        name,
        op: CpsPrimOp::Add,
        args,
        body,
    } = term
    else {
        return false;
    };

    matches!(args.as_slice(), [CpsAtom::Int(_), CpsAtom::Int(_)])
        && is_answer_jump_for_result(body, name)
}

/// Verify one already-selected Boolean-negation corpus witness.  The caller
/// binds its expected operand to the fixture ID and exact source text before
/// lowering, so this remains a two-case evidence check rather than a general
/// unary-expression admission rule.
fn is_source_entry_bool_not(term: &CpsTerm, operand: bool) -> bool {
    let CpsTerm::LetPrim {
        name,
        op: CpsPrimOp::Not,
        args,
        body,
    } = term
    else {
        return false;
    };

    matches!(args.as_slice(), [CpsAtom::Bool(actual)] if *actual == operand)
        && is_answer_jump_for_result(body, name)
}

/// Verify the single selected lexical Boolean-negation witness. The caller
/// first binds the fixture identity and full source text, then supplies this
/// witness's exact binder and literal. This is evidence validation only, not
/// a reusable lexical lowering predicate.
fn is_source_entry_lexical_bool_not(term: &CpsTerm, binder: &str, operand: bool) -> bool {
    let CpsTerm::LetVal {
        name,
        value: CpsValue::Atom(CpsAtom::Bool(actual_operand)),
        body: let_body,
    } = term
    else {
        return false;
    };
    if name != binder || *actual_operand != operand {
        return false;
    }

    let CpsTerm::LetPrim {
        name: result_name,
        op: CpsPrimOp::Not,
        args,
        body,
    } = let_body.as_ref()
    else {
        return false;
    };

    matches!(args.as_slice(), [CpsAtom::Var(argument)] if argument == binder)
        && is_answer_jump_for_result(body, result_name)
}

/// Admit exactly the lexical source form generated for `let x = 2; let y =
/// 5; return x + y`.  This is deliberately not a general let/variable
/// validator: changing either binding, operand, primitive, or answer jump
/// requires a new source-entry evidence rule.
fn is_source_entry_lexical_primitive_add(term: &CpsTerm) -> bool {
    let CpsTerm::LetVal {
        name: x_name,
        value: CpsValue::Atom(CpsAtom::Int(2)),
        body: x_body,
    } = term
    else {
        return false;
    };
    if x_name != "x" {
        return false;
    }

    let CpsTerm::LetVal {
        name: y_name,
        value: CpsValue::Atom(CpsAtom::Int(5)),
        body: y_body,
    } = x_body.as_ref()
    else {
        return false;
    };
    if y_name != "y" {
        return false;
    }

    let CpsTerm::LetPrim {
        name: result_name,
        op: CpsPrimOp::Add,
        args,
        body,
    } = y_body.as_ref()
    else {
        return false;
    };

    matches!(
        args.as_slice(),
        [CpsAtom::Var(left), CpsAtom::Var(right)] if left == x_name && right == y_name
    ) && is_answer_jump_for_result(body, result_name)
}

fn is_answer_jump_for_result(term: &CpsTerm, result_name: &str) -> bool {
    matches!(
        term,
        CpsTerm::Jump {
            cont: ash_core::cps::ContRef::Label(answer),
            arg: CpsAtom::Var(result),
            ..
        } if answer == "__answer" && result == result_name
    )
}

impl CheckedCoreCpsKernelTerm {
    const fn observed_dimension(&self) -> ObservableDimension {
        match self {
            Self::V1(term) => term.observed_dimension(),
            Self::V2(_) | Self::V3(_) | Self::V4(_) => ObservableDimension::Values,
        }
    }

    const fn canonical_rule_id(&self) -> &'static str {
        match self {
            Self::V1(term) => term.canonical_rule_id(),
            Self::V2(_) => "SEM-CPS-LETVAL-001",
            Self::V3(_) => "SEM-CPS-PRIM-001",
            Self::V4(_) => "SEM-CPS-IF-001",
        }
    }
}

fn lower_source_entry_to_executed_answer_continuation(source: &str) -> Result<CpsTerm, String> {
    let engine = Engine::new()
        .build()
        .map_err(|error| format!("could not build checked Core/CPS source bridge: {error}"))?;
    let mut entry = engine
        .parse(source)
        .map_err(|error| format!("checked Core/CPS source parse failed: {error}"))?;
    engine
        .check(&mut entry)
        .map_err(|error| format!("checked Core/CPS source check failed: {error}"))?;
    let body = engine
        .lower_entry_to_checked_cps(&entry)
        .map_err(|error| format!("checked Core/CPS source lowering failed: {error}"))?;

    Ok(CpsTerm::LetCont {
        name: "__answer".to_string(),
        param: "__answer_value".to_string(),
        cont_body: Box::new(CpsTerm::Return {
            value: CpsValue::Atom(CpsAtom::Var("__answer_value".to_string())),
        }),
        body: Box::new(body),
        row: ash_core::cps::EffectRow::default(),
        multiplicity: ash_core::cps::ContMultiplicity::Affine,
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "form")]
enum CheckedCoreCpsTerm {
    Return {
        value: CheckedCoreCpsAtom,
    },
    Trap {
        reason: CheckedCoreCpsTrapReason,
    },
    Jump {
        continuation: String,
        argument: CheckedCoreCpsAtom,
        row: Vec<String>,
    },
}

/// V2-only narrow literal binding form for the active CPS corpus.
#[derive(Debug, Deserialize)]
#[serde(tag = "form", deny_unknown_fields)]
enum CheckedCoreCpsV2Term {
    LetVal {
        name: String,
        value: CheckedCoreCpsAtom,
        body: CheckedCoreCpsV2LetValBody,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "form", deny_unknown_fields)]
enum CheckedCoreCpsV2LetValBody {
    Return { value: CheckedCoreCpsAtom },
}

/// V3-only narrow primitive binding form for the active CPS corpus.
#[derive(Debug, Deserialize)]
#[serde(tag = "form", deny_unknown_fields)]
enum CheckedCoreCpsV3Term {
    LetPrim {
        name: String,
        primitive: String,
        arguments: Vec<CheckedCoreCpsAtom>,
        body: CheckedCoreCpsV3LetPrimBody,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "form", deny_unknown_fields)]
enum CheckedCoreCpsV3LetPrimBody {
    Return { value: CheckedCoreCpsAtom },
}

/// V4-only literal conditional form for the active CPS corpus.
#[derive(Debug, Deserialize)]
#[serde(tag = "form", deny_unknown_fields)]
enum CheckedCoreCpsV4Term {
    If {
        condition: CheckedCoreCpsV4Atom,
        then_branch: CheckedCoreCpsV4Return,
        else_branch: CheckedCoreCpsV4Return,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "form", deny_unknown_fields)]
enum CheckedCoreCpsV4Return {
    Return { value: CheckedCoreCpsV4Atom },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum CheckedCoreCpsV4Atom {
    Bool { value: bool },
    Int { value: i64 },
}

impl CheckedCoreCpsV2Term {
    fn to_term(
        &self,
        continuation_store: &[CheckedCoreCpsContinuation],
    ) -> Result<CpsTerm, String> {
        reject_unexpected_continuation_store(continuation_store)?;
        match self {
            Self::LetVal { name, value, body } => {
                if name.is_empty() {
                    return Err("LetVal bound name must be non-empty".to_string());
                }
                let CheckedCoreCpsAtom::Int { value } = value else {
                    return Err("LetVal value must be a literal Int".to_string());
                };
                let CheckedCoreCpsV2LetValBody::Return { value: returned } = body;
                let CheckedCoreCpsAtom::Var {
                    value: returned_name,
                } = returned
                else {
                    return Err("LetVal body must return bound variable".to_string());
                };
                if returned_name != name {
                    return Err(format!("LetVal body must return bound variable `{name}`"));
                }
                Ok(CpsTerm::LetVal {
                    name: name.clone(),
                    value: ash_core::cps::Value::Atom(CpsAtom::Int(*value)),
                    body: Box::new(CpsTerm::Return {
                        value: CpsValue::Atom(CpsAtom::Var(name.clone())),
                    }),
                })
            }
        }
    }
}

impl CheckedCoreCpsV3Term {
    fn to_term(
        &self,
        continuation_store: &[CheckedCoreCpsContinuation],
    ) -> Result<CpsTerm, String> {
        reject_unexpected_continuation_store(continuation_store)?;
        match self {
            Self::LetPrim {
                name,
                primitive,
                arguments,
                body,
            } => {
                if name.is_empty() {
                    return Err("LetPrim bound name must be non-empty".to_string());
                }
                if primitive != "int_add" {
                    return Err(format!(
                        "unsupported v3 primitive `{primitive}`; only `int_add` is admitted"
                    ));
                }
                let [
                    CheckedCoreCpsAtom::Int { value: left },
                    CheckedCoreCpsAtom::Int { value: right },
                ] = arguments.as_slice()
                else {
                    return Err("v3 int_add requires exactly two literal Int arguments".to_string());
                };
                let CheckedCoreCpsV3LetPrimBody::Return { value: returned } = body;
                let CheckedCoreCpsAtom::Var {
                    value: returned_name,
                } = returned
                else {
                    return Err("LetPrim body must return bound variable".to_string());
                };
                if returned_name != name {
                    return Err(format!("LetPrim body must return bound variable `{name}`"));
                }
                Ok(CpsTerm::LetPrim {
                    name: name.clone(),
                    op: CpsPrimOp::Add,
                    args: vec![CpsAtom::Int(*left), CpsAtom::Int(*right)],
                    body: Box::new(CpsTerm::Return {
                        value: CpsValue::Atom(CpsAtom::Var(name.clone())),
                    }),
                })
            }
        }
    }
}

impl CheckedCoreCpsV4Term {
    fn to_term(
        &self,
        continuation_store: &[CheckedCoreCpsContinuation],
    ) -> Result<CpsTerm, String> {
        reject_unexpected_continuation_store(continuation_store)?;
        match self {
            Self::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let CheckedCoreCpsV4Atom::Bool { value } = condition else {
                    return Err("v4 If condition must be a literal Bool".to_string());
                };
                let then_branch = lower_v4_literal_return(then_branch)?;
                let else_branch = lower_v4_literal_return(else_branch)?;
                Ok(CpsTerm::If {
                    cond: CpsAtom::Bool(*value),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                    row: ash_core::cps::EffectRow::default(),
                })
            }
        }
    }
}

fn lower_v4_literal_return(term: &CheckedCoreCpsV4Return) -> Result<CpsTerm, String> {
    let CheckedCoreCpsV4Return::Return { value } = term;
    let CheckedCoreCpsV4Atom::Int { value } = value else {
        return Err("v4 If branches must be literal Int Return terms".to_string());
    };
    Ok(CpsTerm::Return {
        value: CpsValue::Atom(CpsAtom::Int(*value)),
    })
}

impl CheckedCoreCpsTerm {
    fn to_term(
        &self,
        continuation_store: &[CheckedCoreCpsContinuation],
    ) -> Result<CpsTerm, String> {
        match self {
            Self::Return { value } => {
                reject_unexpected_continuation_store(continuation_store)?;
                Ok(CpsTerm::Return {
                    value: CpsValue::Atom(value.to_atom()),
                })
            }
            Self::Trap { reason } => {
                reject_unexpected_continuation_store(continuation_store)?;
                Ok(CpsTerm::Trap {
                    reason: reason.to_trap_reason(),
                })
            }
            Self::Jump {
                continuation,
                argument,
                row,
            } => {
                lower_jump_with_continuation_store(continuation, argument, row, continuation_store)
            }
        }
    }

    const fn observed_dimension(&self) -> ObservableDimension {
        match self {
            Self::Return { .. } => ObservableDimension::Values,
            Self::Trap { .. } => ObservableDimension::StructuredTraps,
            Self::Jump { .. } => ObservableDimension::ContinuationUse,
        }
    }

    const fn canonical_rule_id(&self) -> &'static str {
        match self {
            Self::Return { .. } => "SEM-CPS-RETURN-001",
            Self::Trap { .. } => "SEM-CPS-TRAP-001",
            Self::Jump { .. } => "SEM-CPS-JUMP-001",
        }
    }
}

/// A continuation admitted by the deliberately narrow CPS-kernel input schema.
///
/// The current schema accepts only affine continuations that immediately return
/// their sole parameter through an empty effect row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedCoreCpsContinuation {
    name: String,
    parameter: String,
    body: CheckedCoreCpsContinuationBody,
    multiplicity: String,
    row: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "form", deny_unknown_fields)]
enum CheckedCoreCpsContinuationBody {
    Return { value: CheckedCoreCpsAtom },
}

fn reject_unexpected_continuation_store(
    continuation_store: &[CheckedCoreCpsContinuation],
) -> Result<(), String> {
    if continuation_store.is_empty() {
        Ok(())
    } else {
        Err("continuation_store is only admitted for a CPS-kernel Jump term".to_string())
    }
}

fn lower_jump_with_continuation_store(
    continuation: &str,
    argument: &CheckedCoreCpsAtom,
    row: &[String],
    continuation_store: &[CheckedCoreCpsContinuation],
) -> Result<CpsTerm, String> {
    if !row.is_empty() {
        return Err("CPS-kernel Jump must declare an empty effect row".to_string());
    }

    let mut continuation_names = HashSet::new();
    for stored in continuation_store {
        validate_continuation(stored)?;
        if !continuation_names.insert(stored.name.as_str()) {
            return Err(format!(
                "explicit continuation store contains duplicate continuation `{}`",
                stored.name
            ));
        }
    }
    if !continuation_names.contains(continuation) {
        return Err(format!(
            "continuation `{continuation}` is absent from the explicit continuation store"
        ));
    }

    let mut term = CpsTerm::Jump {
        cont: ash_core::cps::ContRef::Label(continuation.to_string()),
        arg: argument.to_atom(),
        row: ash_core::cps::EffectRow::default(),
    };
    for stored in continuation_store.iter().rev() {
        term = CpsTerm::LetCont {
            name: stored.name.clone(),
            param: stored.parameter.clone(),
            cont_body: Box::new(CpsTerm::Return {
                value: CpsValue::Atom(CpsAtom::Var(stored.parameter.clone())),
            }),
            body: Box::new(term),
            row: ash_core::cps::EffectRow::default(),
            multiplicity: ash_core::cps::ContMultiplicity::Affine,
        };
    }
    Ok(term)
}

fn validate_continuation(continuation: &CheckedCoreCpsContinuation) -> Result<(), String> {
    if continuation.name.is_empty() || continuation.parameter.is_empty() {
        return Err("CPS-kernel continuation names and parameters must be non-empty".to_string());
    }
    if continuation.multiplicity != "affine" {
        return Err(format!(
            "CPS-kernel continuation `{}` must have affine multiplicity",
            continuation.name
        ));
    }
    if !continuation.row.is_empty() {
        return Err(format!(
            "CPS-kernel continuation `{}` must declare an empty effect row",
            continuation.name
        ));
    }
    let CheckedCoreCpsContinuationBody::Return { value } = &continuation.body;
    let CheckedCoreCpsAtom::Var { value } = value else {
        return Err(format!(
            "CPS-kernel continuation `{}` body must return its parameter",
            continuation.name
        ));
    };
    if value != &continuation.parameter {
        return Err(format!(
            "CPS-kernel continuation `{}` body must return parameter `{}`",
            continuation.name, continuation.parameter
        ));
    }
    Ok(())
}

/// The deliberately small typed trap-reason grammar admitted by the active
/// CPS-kernel fixture schema. It is not a generic serialized CPS term.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum CheckedCoreCpsTrapReason {
    Custom { value: String },
}

impl CheckedCoreCpsTrapReason {
    fn to_trap_reason(&self) -> ash_core::cps::TrapReason {
        match self {
            Self::Custom { value } => ash_core::cps::TrapReason::Custom(value.clone()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum CheckedCoreCpsAtom {
    Int { value: i64 },
    Var { value: String },
}

impl CheckedCoreCpsAtom {
    fn to_atom(&self) -> CpsAtom {
        match self {
            Self::Int { value } => CpsAtom::Int(*value),
            Self::Var { value } => CpsAtom::Var(value.clone()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExternalSetup {
    external_boundary: ExternalBoundary,
}

#[derive(Debug, Deserialize)]
struct ExternalBoundary {
    name: String,
    allowed_outcomes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedFile {
    schema_version: String,
    case_id: String,
    canonical_rule_ids: Vec<String>,
    expectation: JsonValue,
}

#[derive(Debug)]
enum Expectation {
    Exact(JsonValue),
    AllowedSet(Vec<JsonValue>),
}

impl Expectation {
    fn from_file(file: &ExpectedFile) -> Result<Self, DifferentialHarnessError> {
        let kind = file
            .expectation
            .get("kind")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| {
                DifferentialHarnessError::InvalidCase("expectation kind is missing".into())
            })?;
        match kind {
            "exact" => file
                .expectation
                .get("result")
                .cloned()
                .map(Self::Exact)
                .ok_or_else(|| {
                    DifferentialHarnessError::InvalidCase("exact expectation has no result".into())
                }),
            "allowed_set" => file
                .expectation
                .get("results")
                .and_then(JsonValue::as_array)
                .filter(|results| !results.is_empty())
                .cloned()
                .map(Self::AllowedSet)
                .ok_or_else(|| {
                    DifferentialHarnessError::InvalidCase(
                        "allowed_set expectation has no results".into(),
                    )
                }),
            other => Err(DifferentialHarnessError::InvalidCase(format!(
                "unsupported expectation kind `{other}`"
            ))),
        }
    }

    const fn kind(&self) -> &'static str {
        match self {
            Self::Exact(_) => "exact",
            Self::AllowedSet(_) => "allowed_set",
        }
    }

    fn matches(&self, actual: &JsonValue) -> bool {
        match self {
            Self::Exact(expected) => json_contains(expected, actual),
            Self::AllowedSet(expected) => expected.iter().any(|item| json_contains(item, actual)),
        }
    }
}

/// Returns whether every field asserted by an expected result appears in an
/// actual normalized result. Corpus expectations may omit irrelevant evidence.
fn json_contains(expected: &JsonValue, actual: &JsonValue) -> bool {
    match (expected, actual) {
        (JsonValue::Object(expected), JsonValue::Object(actual)) => {
            expected.iter().all(|(key, value)| {
                actual
                    .get(key)
                    .is_some_and(|actual_value| json_contains(value, actual_value))
            })
        }
        (JsonValue::Array(expected), JsonValue::Array(actual)) => {
            expected.len() == actual.len()
                && expected
                    .iter()
                    .zip(actual)
                    .all(|(left, right)| json_contains(left, right))
        }
        _ => expected == actual,
    }
}
