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

use ash_parser::surface::{Definition, Expr, ModuleFile, Workflow, WorkflowDef};
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

fn lint_workflow(wf: &WorkflowDef, config: &LintConfig, diagnostics: &mut Vec<LintDiagnostic>) {
    // L001: Workflow lacks Observe or Act
    if config.level_for(&LintCode("L001".into())) != RuleLevel::Allow
        && !has_observe_or_act(&wf.body)
    {
        diagnostics.push(LintDiagnostic {
            span: wf.span,
            code: LintCode("L001".into()),
            message: "workflow has no observe or act step".into(),
            severity: severity_for(config, "L001"),
            category: LintCategory::Ooda,
            fixes: vec![],
            related_information: vec![],
        });
    }

    // L002: Act without preceding Orient
    if config.level_for(&LintCode("L002".into())) != RuleLevel::Allow {
        check_l002(&wf.body, false, config, diagnostics);
    }

    // L004: Policy conflict not checked
    if config.enable_policy_lints
        && config.level_for(&LintCode("L004".into())) != RuleLevel::Allow
        && !safe_l004(&wf.body, false)
    {
        diagnostics.push(LintDiagnostic {
            span: wf.span,
            code: LintCode("L004".into()),
            message: "decide/policy not followed by check on all control-flow paths".into(),
            severity: severity_for(config, "L004"),
            category: LintCategory::Policy,
            fixes: vec![],
            related_information: vec![],
        });
    }
}

fn severity_for(config: &LintConfig, code: &str) -> LintSeverity {
    match config.level_for(&LintCode(code.into())) {
        RuleLevel::Deny => LintSeverity::Error,
        RuleLevel::Warn => LintSeverity::Warning,
        RuleLevel::Allow => LintSeverity::Hint,
    }
}

// ---------------------------------------------------------------------------
// L001: has Observe or Act
// ---------------------------------------------------------------------------

fn has_observe_or_act(wf: &Workflow) -> bool {
    match wf {
        Workflow::Observe { .. } | Workflow::Act { .. } => true,
        Workflow::Orient { continuation, .. }
        | Workflow::Propose { continuation, .. }
        | Workflow::Let { continuation, .. }
        | Workflow::Set { continuation, .. }
        | Workflow::Send { continuation, .. } => {
            continuation.as_ref().is_some_and(|c| has_observe_or_act(c))
        }
        Workflow::Decide {
            then_branch,
            else_branch,
            ..
        } => {
            has_observe_or_act(then_branch)
                || else_branch.as_ref().is_some_and(|e| has_observe_or_act(e))
        }
        Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            has_observe_or_act(then_branch)
                || else_branch.as_ref().is_some_and(|e| has_observe_or_act(e))
        }
        Workflow::Seq { first, second, .. } => {
            has_observe_or_act(first) || has_observe_or_act(second)
        }
        Workflow::For { body, .. } | Workflow::With { body, .. } | Workflow::Must { body, .. } => {
            has_observe_or_act(body)
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => has_observe_or_act(primary) || has_observe_or_act(fallback),
        Workflow::Check { continuation, .. } => {
            continuation.as_ref().is_some_and(|c| has_observe_or_act(c))
        }
        Workflow::Yield { arms, .. } => arms.iter().any(|a| has_observe_or_act(&a.body)),
        Workflow::Done { .. }
        | Workflow::Ret { .. }
        | Workflow::Oblige { .. }
        | Workflow::Resume { .. }
        | Workflow::Receive { .. } => false,
    }
}

// ---------------------------------------------------------------------------
// L002: Act without preceding Orient
// ---------------------------------------------------------------------------

/// Walk workflow tracking whether an Orient was seen. Emit on first Act-without-Orient.
fn check_l002(
    wf: &Workflow,
    seen_orient: bool,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    match wf {
        Workflow::Orient { continuation, .. } => {
            if let Some(c) = continuation {
                check_l002(c, true, config, diagnostics);
            }
        }
        Workflow::Act {
            span, continuation, ..
        } => {
            if !seen_orient {
                diagnostics.push(LintDiagnostic {
                    span: *span,
                    code: LintCode("L002".into()),
                    message: "act without preceding orient step".into(),
                    severity: severity_for(config, "L002"),
                    category: LintCategory::Ooda,
                    fixes: vec![],
                    related_information: vec![],
                });
            }
            if let Some(c) = continuation {
                check_l002(c, seen_orient, config, diagnostics);
            }
        }
        Workflow::Observe { continuation, .. }
        | Workflow::Propose { continuation, .. }
        | Workflow::Let { continuation, .. }
        | Workflow::Set { continuation, .. }
        | Workflow::Send { continuation, .. }
        | Workflow::Check { continuation, .. } => {
            if let Some(c) = continuation {
                check_l002(c, seen_orient, config, diagnostics);
            }
        }
        Workflow::Decide {
            then_branch,
            else_branch,
            ..
        } => {
            check_l002(then_branch, seen_orient, config, diagnostics);
            if let Some(e) = else_branch {
                check_l002(e, seen_orient, config, diagnostics);
            }
        }
        Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            check_l002(then_branch, seen_orient, config, diagnostics);
            if let Some(e) = else_branch {
                check_l002(e, seen_orient, config, diagnostics);
            }
        }
        Workflow::Seq { first, second, .. } => {
            // First may set seen_orient for second's context.
            // We need to know if first contains an orient.
            check_l002(first, seen_orient, config, diagnostics);
            let orient_in_first = contains_orient(first);
            check_l002(second, seen_orient || orient_in_first, config, diagnostics);
        }
        Workflow::For { body, .. } | Workflow::With { body, .. } | Workflow::Must { body, .. } => {
            check_l002(body, seen_orient, config, diagnostics);
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => {
            check_l002(primary, seen_orient, config, diagnostics);
            check_l002(fallback, seen_orient, config, diagnostics);
        }
        Workflow::Yield { arms, .. } => {
            for arm in arms {
                check_l002(&arm.body, seen_orient, config, diagnostics);
            }
        }
        Workflow::Done { .. }
        | Workflow::Ret { .. }
        | Workflow::Oblige { .. }
        | Workflow::Resume { .. }
        | Workflow::Receive { .. } => {}
    }
}

/// Returns true if the workflow tree contains an Orient node.
fn contains_orient(wf: &Workflow) -> bool {
    match wf {
        Workflow::Orient { .. } => true,
        Workflow::Observe { continuation, .. }
        | Workflow::Propose { continuation, .. }
        | Workflow::Act { continuation, .. }
        | Workflow::Let { continuation, .. }
        | Workflow::Set { continuation, .. }
        | Workflow::Send { continuation, .. }
        | Workflow::Check { continuation, .. } => {
            continuation.as_ref().is_some_and(|c| contains_orient(c))
        }
        Workflow::Decide {
            then_branch,
            else_branch,
            ..
        } => {
            contains_orient(then_branch) || else_branch.as_ref().is_some_and(|e| contains_orient(e))
        }
        Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            contains_orient(then_branch) || else_branch.as_ref().is_some_and(|e| contains_orient(e))
        }
        Workflow::Seq { first, second, .. } => contains_orient(first) || contains_orient(second),
        Workflow::For { body, .. } | Workflow::With { body, .. } | Workflow::Must { body, .. } => {
            contains_orient(body)
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => contains_orient(primary) || contains_orient(fallback),
        Workflow::Yield { arms, .. } => arms.iter().any(|a| contains_orient(&a.body)),
        Workflow::Done { .. }
        | Workflow::Ret { .. }
        | Workflow::Oblige { .. }
        | Workflow::Resume { .. }
        | Workflow::Receive { .. } => false,
    }
}

// ---------------------------------------------------------------------------
// L004: Policy conflict not checked (CPS condition)
// ---------------------------------------------------------------------------

/// Returns true iff `expr` contains any `Expr::Policy` node.
fn contains_policy(expr: &Expr) -> bool {
    match expr {
        Expr::Policy(_) => true,
        Expr::FieldAccess { base, .. } => contains_policy(base),
        Expr::IndexAccess { base, index, .. } => contains_policy(base) || contains_policy(index),
        Expr::Unary { operand, .. } => contains_policy(operand),
        Expr::Binary { left, right, .. } => contains_policy(left) || contains_policy(right),
        Expr::Call { args, .. } => args.iter().any(contains_policy),
        Expr::Match {
            scrutinee, arms, ..
        } => contains_policy(scrutinee) || arms.iter().any(|a| contains_policy(&a.body)),
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => contains_policy(expr) || contains_policy(then_branch) || contains_policy(else_branch),
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            let else_has = else_branch.as_ref().is_some_and(|e| contains_policy(e));
            contains_policy(condition) || contains_policy(then_branch) || else_has
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            statements.iter().any(|s| match s {
                ash_parser::surface::BlockStmt::Let { expr, .. } => contains_policy(expr),
            }) || tail_expr.as_ref().is_some_and(|e| contains_policy(e))
        }
        Expr::FnDef { body, .. } => contains_policy(body),
        Expr::FnApply { func, args, .. } => {
            contains_policy(func) || args.iter().any(contains_policy)
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            fields.iter().any(|(_, e)| contains_policy(e))
                || match payload {
                    ash_parser::surface::ConstructorPayload::Unit => false,
                    ash_parser::surface::ConstructorPayload::Record(fs) => {
                        fs.iter().any(|(_, e)| contains_policy(e))
                    }
                    ash_parser::surface::ConstructorPayload::Tuple(es) => {
                        es.iter().any(contains_policy)
                    }
                }
        }
        Expr::Variable { .. }
        | Expr::Literal(_)
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. } => false,
    }
}

/// Returns true iff the expression tree contains any Policy node.
/// Wrapper used in workflow contexts.
fn workflow_expr_has_policy(expr: &Expr) -> bool {
    contains_policy(expr)
}

/// CPS safety check for L004.
///
/// `pending = true` means there is an unmatched Decide/Policy above on this path.
/// Returns `true` if all paths are safe (every pending policy is checked before termination).
fn safe_l004(wf: &Workflow, pending: bool) -> bool {
    match wf {
        // 1. Terminal nodes: safe only if nothing is pending
        Workflow::Done { .. } => !pending,
        // 2. Ret: safe only if nothing pending and no policy in expression
        Workflow::Ret { expr, .. } => !pending && !workflow_expr_has_policy(expr),

        // 3-4. Check resets pending
        Workflow::Check { continuation, .. } => match continuation {
            Some(c) => safe_l004(c, false),
            None => !pending,
        },

        // 5. Decide: both branches inherit pending = true
        Workflow::Decide {
            then_branch,
            else_branch,
            expr,
            ..
        } => {
            let has_pol = workflow_expr_has_policy(expr);
            let p = pending || has_pol;
            match else_branch {
                Some(e) => safe_l004(then_branch, p) && safe_l004(e, p),
                None => safe_l004(then_branch, p),
            }
        }

        // 6. If: propagates pending state
        Workflow::If {
            then_branch,
            else_branch,
            condition,
            ..
        } => {
            let has_pol = workflow_expr_has_policy(condition);
            let p = pending || has_pol;
            match else_branch {
                Some(e) => safe_l004(then_branch, p) && safe_l004(e, p),
                None => safe_l004(then_branch, p),
            }
        }

        // 7. Seq: both branches inherit pending
        Workflow::Seq { first, second, .. } => {
            safe_l004(first, pending) && safe_l004(second, pending)
        }

        // 8. All other variants
        Workflow::Observe { continuation, .. }
        | Workflow::Propose { continuation, .. }
        | Workflow::Act { continuation, .. }
        | Workflow::Let { continuation, .. }
        | Workflow::Set { continuation, .. }
        | Workflow::Send { continuation, .. } => {
            // None of these variants directly contain Policy expressions in their
            // workflow-level fields (they may have Expr children, but those are not
            // Decide/Policy at the workflow level).
            match continuation {
                Some(c) => safe_l004(c, pending),
                None => !pending,
            }
        }
        Workflow::Orient {
            expr, continuation, ..
        } => {
            let has_pol = workflow_expr_has_policy(expr);
            match continuation {
                Some(c) => safe_l004(c, pending || has_pol),
                None => !(pending || has_pol),
            }
        }
        Workflow::Oblige { .. } => !pending,
        Workflow::For { body, .. } => safe_l004(body, pending),
        Workflow::With { body, .. } => safe_l004(body, pending),
        Workflow::Maybe {
            primary, fallback, ..
        } => safe_l004(primary, pending) && safe_l004(fallback, pending),
        Workflow::Must { body, .. } => safe_l004(body, pending),
        Workflow::Receive { .. } => !pending,
        Workflow::Yield { arms, .. } => arms.iter().all(|a| safe_l004(&a.body, pending)),
        Workflow::Resume { .. } => !pending,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
