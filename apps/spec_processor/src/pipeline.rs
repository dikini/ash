//! Pipeline orchestrator — wires every spec-processor check into a single call.
//!
//! The pipeline scans the repository, runs all coherence and conformance checks,
//! and aggregates the findings into a [`Report`] suitable for CI gating.

use std::fmt;
use std::fs;
use std::path::Path;

use crate::capability_boundary;
use crate::changelog;
use crate::collect;
use crate::example_check;
use crate::meta_validation;
use crate::plan_index;
use crate::report::Report;
use crate::spec_links;

/// Error returned when the pipeline cannot complete.
///
/// Wraps the underlying error from whichever step failed, along with a
/// human-readable step name for diagnostics.
#[derive(Debug)]
pub struct PipelineError {
    step: String,
    source: PipelineErrorSource,
}

/// The underlying error category.
#[derive(Debug)]
enum PipelineErrorSource {
    /// An I/O error (file not found, permission denied, etc.).
    Io(std::io::Error),
    /// An engine error (parse failure, type error, engine build failure, etc.).
    Engine(ash_engine::EngineError),
}

impl PipelineError {
    /// Create a new pipeline error wrapping an I/O failure.
    fn from_io(step: impl Into<String>, source: std::io::Error) -> Self {
        Self {
            step: step.into(),
            source: PipelineErrorSource::Io(source),
        }
    }

    /// Create a new pipeline error wrapping an engine failure.
    fn from_engine(step: impl Into<String>, source: ash_engine::EngineError) -> Self {
        Self {
            step: step.into(),
            source: PipelineErrorSource::Engine(source),
        }
    }

    /// Returns the name of the pipeline step that failed.
    #[must_use]
    pub fn step(&self) -> &str {
        &self.step
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pipeline error at '{}': ", self.step)?;
        match &self.source {
            PipelineErrorSource::Io(e) => write!(f, "{e}"),
            PipelineErrorSource::Engine(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            PipelineErrorSource::Io(e) => Some(e),
            PipelineErrorSource::Engine(e) => Some(e),
        }
    }
}

/// Run all spec processor checks against the given repository root.
///
/// # What it does
///
/// The pipeline executes every check module in sequence:
///
/// 1. **File tree scan** — walks the repo and classifies files by naming convention.
/// 2. **PLAN-INDEX coherence** — verifies that every `TASK-NNN` referenced in the
///    plan index has a corresponding task file, and flags orphaned task files.
/// 3. **Spec link integrity** — checks that Markdown links inside `SPEC-*.md`
///    files resolve to existing files.
/// 4. **Changelog completeness** — confirms that every task marked as completed
///    in the plan index appears in `CHANGELOG.md`.
/// 5. **Example conformance** — parses and type-checks every `.ash` example file.
/// 6. **Capability boundary audit** — verifies that declared stdlib capability
///    stubs exist and contain substantive declarations.
/// 7. **Meta-validation** — self-checks the processor's own source tree for
///    empty files, broken doc refs, and test coverage gaps.
///
/// # Return value
///
/// Returns `Ok(Report)` on success. The report contains all findings from every
/// check, along with tier counts and a `blocked` flag that is `true` when at
/// least one error-tier finding is present.
///
/// Returns `Err(PipelineError)` if a critical step fails (e.g. the repository
/// root cannot be scanned, or the engine fails to build).
///
/// # Errors
///
/// Returns an error if the file tree scan, plan-index read, plan-index
/// coherence check, or engine construction encounters a failure.
///
/// # CI gating
///
/// To use this as a CI gate, assert that the pipeline succeeds **and** that the
/// report is not blocked:
///
/// ```rust,ignore
/// let report = spec_processor::run_pipeline(repo_root)?;
/// assert!(!report.blocked, "spec processor gate failed:\n{}", report.format_human());
/// ```
pub fn run_pipeline(repo_root: &Path) -> Result<Report, PipelineError> {
    // 1. Scan file tree.
    let tree = collect::scan_tree(repo_root).map_err(|e| PipelineError::from_io("scan_tree", e))?;

    let mut all_findings = Vec::new();

    // 2. PLAN-INDEX coherence check.
    let plan_index_path = repo_root.join("docs/plan/PLAN-INDEX.md");
    let tasks_dir = repo_root.join("docs/plan/tasks");

    if plan_index_path.exists() && tasks_dir.exists() {
        // Read plan index once — used for both coherence and changelog checks
        // to avoid a redundant I/O round-trip (plan_index::check also reads
        // the file internally, but that API takes a path, not content).
        let plan_content = fs::read_to_string(&plan_index_path)
            .map_err(|e| PipelineError::from_io("read_plan_index", e))?;

        match plan_index::check(&plan_index_path, &tasks_dir) {
            Ok(findings) => all_findings.extend(findings),
            Err(e) => return Err(PipelineError::from_io("plan_index::check", e)),
        }

        // 4. Changelog completeness — reuses plan_content read above.
        let changelog_path = tree.changelog_files.first().map(String::as_str);
        let changelog_content = match changelog_path {
            Some(path) => fs::read_to_string(repo_root.join(path))
                .map_err(|e| PipelineError::from_io("read_changelog", e))?,
            None => String::new(),
        };
        all_findings.extend(changelog::check_changelog(
            &plan_content,
            &changelog_content,
            changelog_path,
        ));
    }

    // 3. Spec link integrity.
    match spec_links::check_spec_links(&tree.spec_files, repo_root) {
        Ok(findings) => all_findings.extend(findings),
        Err(e) => return Err(PipelineError::from_io("spec_links::check_spec_links", e)),
    }

    // 5. Example conformance.
    match example_check::check_examples(&tree.example_files, repo_root) {
        Ok(findings) => all_findings.extend(findings),
        Err(e) => {
            return Err(PipelineError::from_engine(
                "example_check::check_examples",
                e,
            ));
        }
    }

    // 6. Capability boundary audit.
    all_findings.extend(capability_boundary::check_capabilities(repo_root));

    // 7. Meta-validation.
    let processor_src_dir = repo_root.join("apps/spec_processor/src");
    if processor_src_dir.exists() {
        all_findings.extend(meta_validation::check_meta(&processor_src_dir, repo_root));
    }

    Ok(Report::from_findings(all_findings))
}
