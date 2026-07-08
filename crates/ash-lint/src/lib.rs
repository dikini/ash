//! Ash Lint Library — AST-based lint diagnostics for Ash source files.
//!
//! Provides a public API for linting Ash source code, intended for use by
//! both the `ash-lint` CLI binary and future `ash-lsp-core` integration.

// Pedantic doc warnings for variant fields are too noisy on large enums.
#![allow(
    clippy::doc_markdown,
    missing_docs,
    clippy::option_if_let_else,
    clippy::match_same_arms,
    clippy::manual_let_else
)]

use ash_parser::surface::{Definition, ModuleFile};
use ash_parser::token::Span;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A single lint diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintDiagnostic {
    pub span: Span,
    pub code: LintCode,
    pub message: String,
    pub severity: LintSeverity,
    pub category: LintCategory,
    pub fixes: Vec<LintFix>,
    /// Reserved for future LSP-style related locations.
    pub related_information: Vec<LintRelatedInformation>,
}

/// Stable lint rule identifier (e.g. `L001`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintCode(pub String);

/// A suggested automatic fix for a lint diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintFix {
    pub span: Span,
    pub replacement: String,
    pub description: String,
}

/// Reserved for future LSP-style related diagnostic locations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LintRelatedInformation {
    pub span: Span,
    pub message: String,
}

/// Diagnostic severity aligned with LSP levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LintSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Category a lint rule belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LintCategory {
    /// Missing provenance annotations (syntax TBD).
    Provenance,
    /// Policy-related lints.
    Policy,
    /// General style nits (placeholder).
    Style,
}

/// Per-rule enforcement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RuleLevel {
    Allow,
    Warn,
    Deny,
}

/// Configuration for linting.
#[derive(Debug, Clone)]
pub struct LintConfig {
    pub require_provenance: bool,
    pub enable_policy_lints: bool,
    pub rules: HashMap<LintCode, RuleLevel>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            require_provenance: false,
            enable_policy_lints: true,
            rules: HashMap::new(),
        }
    }
}

impl LintConfig {
    /// Returns the effective level for a rule, defaulting to `Allow` if not configured.
    #[must_use]
    pub fn level_for(&self, code: &LintCode) -> RuleLevel {
        self.rules.get(code).copied().unwrap_or(RuleLevel::Allow)
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Lint a raw source string. Parse errors are silently ignored (handled by the parser).
#[must_use]
pub fn lint_source(source: &str, config: &LintConfig) -> Vec<LintDiagnostic> {
    let module = match ash_parser::parse_surface_file(source) {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };
    lint_module(&module, config)
}

/// Lint a parsed module.
#[must_use]
pub fn lint_module(module: &ModuleFile, config: &LintConfig) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    for def in &module.definitions {
        lint_definition(def, config, &mut diagnostics);
    }
    diagnostics
}

/// Lint a single definition, appending diagnostics.
pub const fn lint_definition(
    def: &Definition,
    _config: &LintConfig,
    _diagnostics: &mut Vec<LintDiagnostic>,
) {
    // No definition-level lints in the MVP.
    let _ = def;
}

#[cfg(test)]
#[allow(
    clippy::needless_collect,
    clippy::match_same_arms,
    clippy::let_and_return
)]
mod tests {
    use super::*;

    fn test_config() -> LintConfig {
        LintConfig::default()
    }

    fn parse_module(source: &str) -> ModuleFile {
        ash_parser::parse_surface_file(source).expect("parse should succeed")
    }

    // -- Config tests --

    #[test]
    fn test_config_default_rules() {
        let cfg = LintConfig::default();
        assert_eq!(cfg.level_for(&LintCode("L999".into())), RuleLevel::Allow);
        assert!(
            cfg.rules.is_empty(),
            "removed lint rules should not be active"
        );
    }

    #[test]
    fn test_allow_registers_rule_override() {
        let mut cfg = LintConfig::default();
        cfg.rules.insert(LintCode("L999".into()), RuleLevel::Allow);
        assert_eq!(cfg.level_for(&LintCode("L999".into())), RuleLevel::Allow);
    }

    // -- Idempotence test --

    #[test]
    fn test_lint_module_idempotent() {
        let module = parse_module("fn main() -> Int { 1 }");
        let r1 = lint_module(&module, &test_config());
        let r2 = lint_module(&module, &test_config());
        assert_eq!(r1, r2, "lint_module should be idempotent");
    }

    #[test]
    fn test_target_source_has_no_removed_rule_diagnostics() {
        let diags = lint_source("fn main() -> Int { 1 }", &test_config());
        assert!(diags.is_empty(), "target source should lint cleanly");
    }

    // -- lint_definition no-op test --

    #[test]
    fn test_lint_definition_noop() {
        let source = "interface Sensor { read() -> Int }";
        let module = parse_module(source);
        let mut diags = Vec::new();
        for def in &module.definitions {
            lint_definition(def, &test_config(), &mut diags);
        }
        assert!(
            diags.is_empty(),
            "definition-level lints should be no-op in MVP"
        );
    }

    // -- LintSource on parse error returns empty --

    #[test]
    fn test_lint_source_parse_error() {
        let source = "this is not valid ash {{{{";
        let diags = lint_source(source, &test_config());
        assert!(
            diags.is_empty(),
            "parse errors should produce no lint diagnostics"
        );
    }
}
