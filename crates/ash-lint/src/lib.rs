//! Ash Lint Library — AST-based lint diagnostics for Ash workflow files.
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
    /// OODA loop violations.
    Ooda,
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
        let mut rules = HashMap::new();
        rules.insert(LintCode("L001".into()), RuleLevel::Warn);
        rules.insert(LintCode("L002".into()), RuleLevel::Warn);
        rules.insert(LintCode("L004".into()), RuleLevel::Warn);
        Self {
            require_provenance: false,
            enable_policy_lints: true,
            rules,
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
    if let Some(wf) = &module.workflow {
        lint_workflow(wf, config, &mut diagnostics);
    }
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

// ---------------------------------------------------------------------------
// Internal: workflow-level linting
// ---------------------------------------------------------------------------

mod rules;
use rules::lint_workflow;

#[cfg(test)]
#[allow(
    clippy::needless_collect,
    clippy::match_same_arms,
    clippy::let_and_return
)]
mod tests {
    use super::*;
    use ash_parser::surface::{Expr, Workflow};

    fn test_config() -> LintConfig {
        LintConfig::default()
    }

    fn parse_wf(source: &str) -> ModuleFile {
        ash_parser::parse_surface_file(source).expect("parse should succeed")
    }

    // -- L001 tests --

    #[test]
    fn test_l001_workflow_with_observe() {
        let source = "workflow main { observe sensor done }";
        let diags = lint_source(source, &test_config());
        let l001: Vec<_> = diags.iter().filter(|d| d.code.0 == "L001").collect();
        assert!(l001.is_empty(), "should not flag workflow with observe");
    }

    #[test]
    fn test_l001_workflow_with_act() {
        let source = "workflow main { act io:write(\"hello\") done }";
        let diags = lint_source(source, &test_config());
        let l001: Vec<_> = diags.iter().filter(|d| d.code.0 == "L001").collect();
        assert!(l001.is_empty(), "should not flag workflow with act");
    }

    #[test]
    fn test_l001_workflow_with_only_orient() {
        let source = "workflow main { orient 1 + 2 done }";
        let diags = lint_source(source, &test_config());
        let l001: Vec<_> = diags.iter().filter(|d| d.code.0 == "L001").collect();
        assert_eq!(l001.len(), 1, "should flag workflow with only orient");
    }

    // -- L002 tests --

    #[test]
    fn test_l002_act_with_orient() {
        let source = "workflow main { orient 1 act io:write(\"hello\") done }";
        let diags = lint_source(source, &test_config());
        let l002: Vec<_> = diags.iter().filter(|d| d.code.0 == "L002").collect();
        assert!(l002.is_empty(), "orient before act is fine");
    }

    #[test]
    fn test_l002_act_without_orient() {
        let source = "workflow main { act io:write(\"hello\") done }";
        let diags = lint_source(source, &test_config());
        let l002: Vec<_> = diags.iter().filter(|d| d.code.0 == "L002").collect();
        assert_eq!(l002.len(), 1, "should flag act without orient");
        assert!(l002[0].span.line > 0, "span should be non-default");
    }

    // -- L004 tests --

    #[test]
    fn test_l004_decide_followed_by_check() {
        let source = r"workflow main {
            observe sensor
            decide { true } under my_policy then {
                check my_policy
            }
            done
        }";
        let diags = lint_source(source, &test_config());
        let l004: Vec<_> = diags.iter().filter(|d| d.code.0 == "L004").collect();
        assert!(l004.is_empty(), "decide followed by check is safe");
    }

    #[test]
    fn test_l004_decide_without_check() {
        // Construct a Decide node with a Policy expr in its body and no Check.
        // This directly tests the L004 rule logic via AST construction.
        use ash_parser::surface::{OperationalTarget, PolicyExpr};

        let policy_expr = Expr::Policy(PolicyExpr::Var {
            name: "my_policy".into(),
            span: Span::default(),
        });

        let decide = Workflow::Decide {
            expr: policy_expr,
            policy: Some("my_policy".into()),
            then_branch: Box::new(Workflow::Act {
                action: ash_parser::surface::ActionRef {
                    target: OperationalTarget::Explicit {
                        provider: "io".into(),
                        action: "write".into(),
                    },
                    args: vec![Expr::Literal(ash_parser::surface::Literal::String(
                        "ok".into(),
                    ))],
                },
                guard: None,
                result_name: None,
                continuation: Some(Box::new(Workflow::Done {
                    span: Span::default(),
                })),
                span: Span::default(),
            }),
            else_branch: None,
            span: Span::default(),
        };

        let wf_def = ash_parser::surface::WorkflowDef {
            name: "main".into(),
            type_params: vec![],
            params: vec![],
            declared_return_type: None,
            plays_roles: vec![],
            capabilities: vec![],
            owned_resources: vec![],
            used_bindings: vec![],
            header_events: vec![],
            body: decide,
            contract: None,
            span: Span::default(),
        };

        let mut diags = Vec::new();
        lint_workflow(&wf_def, &test_config(), &mut diags);
        let l004: Vec<_> = diags.iter().filter(|d| d.code.0 == "L004").collect();
        assert_eq!(l004.len(), 1, "should flag decide without check");
    }

    // -- Config tests --

    #[test]
    fn test_config_default_rules() {
        let cfg = LintConfig::default();
        assert_eq!(cfg.level_for(&LintCode("L001".into())), RuleLevel::Warn);
        assert_eq!(cfg.level_for(&LintCode("L002".into())), RuleLevel::Warn);
        assert_eq!(cfg.level_for(&LintCode("L004".into())), RuleLevel::Warn);
        assert_eq!(cfg.level_for(&LintCode("L999".into())), RuleLevel::Allow);
    }

    #[test]
    fn test_allow_suppresses_l001() {
        let mut cfg = LintConfig::default();
        cfg.rules.insert(LintCode("L001".into()), RuleLevel::Allow);
        let source = "workflow main { orient 1 done }";
        let diags = lint_source(source, &cfg);
        let l001: Vec<_> = diags.iter().filter(|d| d.code.0 == "L001").collect();
        assert!(l001.is_empty(), "allow should suppress L001");
    }

    // -- Idempotence test --

    #[test]
    fn test_lint_module_idempotent() {
        let source = "workflow main { act io:write(\"hello\") done }";
        let module = parse_wf(source);
        let r1 = lint_module(&module, &test_config());
        let r2 = lint_module(&module, &test_config());
        assert_eq!(r1, r2, "lint_module should be idempotent");
    }

    // -- Non-default span test --

    #[test]
    fn test_diagnostics_have_non_default_spans() {
        let source = "workflow main { act io:write(\"hello\") done }";
        let diags = lint_source(source, &test_config());
        for d in &diags {
            assert!(
                d.span.line > 0,
                "diagnostic {:?} should have non-default span",
                d.code.0
            );
        }
    }

    // -- lint_definition no-op test --

    #[test]
    fn test_lint_definition_noop() {
        let source = "capability sensor: epistemic();";
        let module = parse_wf(source);
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
