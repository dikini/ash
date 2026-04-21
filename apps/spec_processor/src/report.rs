//! Report aggregator — collects findings into a structured report with
//! human-readable and JSON output formats.

use std::fmt::Write;

use crate::finding::{SpecFinding, Tier};

/// Aggregated analysis report.
///
/// Tracks tier counts, determines blocked status (any error-tier finding), and
/// supports rendering as human-readable text or JSON.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Report {
    /// All findings collected during analysis.
    pub findings: Vec<SpecFinding>,
    /// `true` when at least one error-tier finding is present.
    pub blocked: bool,
    /// Count of info-tier findings.
    pub info_count: usize,
    /// Count of warning-tier findings.
    pub warning_count: usize,
    /// Count of error-tier findings.
    pub error_count: usize,
}

impl Report {
    /// Build a report from a vector of findings.
    ///
    /// Tier counts and blocked status are computed automatically.
    #[must_use]
    pub fn from_findings(findings: Vec<SpecFinding>) -> Self {
        let info_count = findings.iter().filter(|f| f.tier == Tier::Info).count();
        let warning_count = findings.iter().filter(|f| f.tier == Tier::Warning).count();
        let error_count = findings.iter().filter(|f| f.tier == Tier::Error).count();
        let blocked = error_count > 0;
        Self {
            findings,
            blocked,
            info_count,
            warning_count,
            error_count,
        }
    }

    /// Format the report as human-readable text.
    #[must_use]
    pub fn format_human(&self) -> String {
        let mut out = String::new();
        out.push_str("Spec Processor Report\n");
        out.push_str("======================\n");
        // Writing to String is infallible — unwrap() will never panic.
        writeln!(
            out,
            "Info: {} | Warnings: {} | Errors: {}",
            self.info_count, self.warning_count, self.error_count
        )
        .unwrap();
        writeln!(out, "Blocked: {}\n", self.blocked).unwrap();

        for f in &self.findings {
            let level = match f.tier {
                Tier::Info => "INFO",
                Tier::Warning => "WARN",
                Tier::Error => "ERROR",
            };
            write!(
                out,
                "[{}] {} ({}): {}",
                level,
                f.category,
                f.file.as_deref().unwrap_or("-"),
                f.description
            )
            .unwrap();
            if let Some(ref tid) = f.task_id {
                write!(out, " [{tid}]").unwrap();
            }
            out.push('\n');
        }

        out
    }

    /// Format the report as a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns an error if `serde_json` serialization fails (e.g. if a
    /// finding contains a value that cannot be represented in JSON).
    #[must_use = "JSON string should be used by the caller"]
    pub fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
