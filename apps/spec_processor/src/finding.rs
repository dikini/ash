//! Shared finding types and constants used across the spec-processor pipeline.

use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

/// Pre-compiled regex for `TASK-NNN` references (3+ digits).
///
/// Shared across modules that need to extract task identifiers from text.
pub(crate) static TASK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"TASK-\d{3,}").unwrap());

/// Severity tier for a finding.
///
/// Using an enum instead of a bare `u8` prevents invalid tier values and makes
/// the intent explicit at every call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(into = "u8")]
pub enum Tier {
    /// Informational — no action required.
    Info = 0,
    /// Warning — should be reviewed.
    Warning = 1,
    /// Error — blocks the pipeline.
    Error = 2,
}

impl From<Tier> for u8 {
    fn from(tier: Tier) -> Self {
        tier as Self
    }
}

impl Tier {
    /// Convert from the raw integer representation.
    ///
    /// Returns `None` for values outside the valid range (0–2).
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Self::Info),
            1 => Some(Self::Warning),
            2 => Some(Self::Error),
            _ => None,
        }
    }
}

/// A single finding emitted during repository analysis.
#[derive(Debug, Clone, Serialize)]
pub struct SpecFinding {
    /// Severity tier.
    pub tier: Tier,
    /// Machine-readable category, e.g. `"MissingTaskFile"`, `"BrokenLink"`.
    pub category: String,
    /// Human-readable description of the finding.
    pub description: String,
    /// Optional file path the finding relates to (relative to repo root).
    pub file: Option<String>,
    /// Optional task identifier the finding relates to.
    pub task_id: Option<String>,
}

impl SpecFinding {
    /// Create an informational finding (tier 0).
    #[must_use]
    pub fn info(category: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tier: Tier::Info,
            category: category.into(),
            description: description.into(),
            file: None,
            task_id: None,
        }
    }

    /// Create a warning finding (tier 1).
    #[must_use]
    pub fn warning(category: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tier: Tier::Warning,
            category: category.into(),
            description: description.into(),
            file: None,
            task_id: None,
        }
    }

    /// Create an error finding (tier 2).
    #[must_use]
    pub fn error(category: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            tier: Tier::Error,
            category: category.into(),
            description: description.into(),
            file: None,
            task_id: None,
        }
    }

    /// Attach a file path to this finding.
    #[must_use]
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Attach a task identifier to this finding.
    #[must_use]
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }
}
