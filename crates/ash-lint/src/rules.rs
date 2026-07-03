use ash_parser::surface::{Expr, Workflow, WorkflowDef};

use super::{LintCategory, LintCode, LintConfig, LintDiagnostic, LintSeverity, RuleLevel};

pub fn lint_workflow(wf: &WorkflowDef, config: &LintConfig, diagnostics: &mut Vec<LintDiagnostic>) {
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
        Workflow::Receive { arms, .. } => arms.iter().any(|a| has_observe_or_act(&a.body)),
        Workflow::Done { .. }
        | Workflow::Ret { .. }
        | Workflow::Oblige { .. }
        | Workflow::Resume { .. } => false,
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
        Workflow::Receive { arms, .. } => {
            for arm in arms {
                check_l002(&arm.body, seen_orient, config, diagnostics);
            }
        }
        Workflow::Done { .. }
        | Workflow::Ret { .. }
        | Workflow::Oblige { .. }
        | Workflow::Resume { .. } => {}
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
        Workflow::Receive { arms, .. } => arms.iter().any(|a| contains_orient(&a.body)),
        Workflow::Done { .. }
        | Workflow::Ret { .. }
        | Workflow::Oblige { .. }
        | Workflow::Resume { .. } => false,
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
        Expr::Record { fields, .. } => fields.iter().any(|(_, e)| contains_policy(e)),
        Expr::Variable { .. }
        | Expr::Literal(_)
        | Expr::OperatorSection { .. }
        | Expr::MacroInvocation { .. }
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. } => false,
        Expr::Fail { payload, .. } => contains_policy(payload),
        Expr::WithError { body, arms, .. } => {
            contains_policy(body) || arms.iter().any(|arm| contains_policy(&arm.body))
        }
        Expr::ActBlock { stmts, .. } => stmts.iter().any(|stmt| {
            let value = match stmt {
                ash_parser::surface::ActStmt::Bind { value, .. } => value,
                ash_parser::surface::ActStmt::Return { value, .. } => value,
            };
            contains_policy(value)
        }),
        Expr::DoBlock { stmts, .. } => stmts.iter().any(|stmt| match stmt {
            ash_parser::surface::DoStmt::Let { value, .. }
            | ash_parser::surface::DoStmt::Bind { value, .. }
            | ash_parser::surface::DoStmt::Expr { value, .. }
            | ash_parser::surface::DoStmt::Return { value, .. } => contains_policy(value),
            ash_parser::surface::DoStmt::WorkflowRequires { .. }
            | ash_parser::surface::DoStmt::WorkflowEnsures { .. } => false,
        }),
        Expr::Comprehension {
            result, qualifiers, ..
        } => {
            qualifiers.iter().any(|qualifier| {
                let value = match qualifier {
                    ash_parser::surface::ComprehensionQualifier::Let { value, .. }
                    | ash_parser::surface::ComprehensionQualifier::Bind { value, .. }
                    | ash_parser::surface::ComprehensionQualifier::DiscardBind { value, .. } => {
                        value
                    }
                };
                contains_policy(value)
            }) || contains_policy(result)
        }
        Expr::List { items, .. } => items.iter().any(contains_policy),
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

        // 8. Variants with continuation but no policy-relevant expression fields
        Workflow::Observe { continuation, .. }
        | Workflow::Propose { continuation, .. }
        | Workflow::Act { continuation, .. } => match continuation {
            Some(c) => safe_l004(c, pending),
            None => !pending,
        },
        Workflow::Let {
            expr, continuation, ..
        } => {
            let has_pol = workflow_expr_has_policy(expr);
            match continuation {
                Some(c) => safe_l004(c, pending || has_pol),
                None => !(pending || has_pol),
            }
        }
        Workflow::Set {
            value,
            continuation,
            ..
        }
        | Workflow::Send {
            value,
            continuation,
            ..
        } => {
            let has_pol = workflow_expr_has_policy(value);
            match continuation {
                Some(c) => safe_l004(c, pending || has_pol),
                None => !(pending || has_pol),
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
        Workflow::For {
            collection, body, ..
        } => {
            let has_pol = workflow_expr_has_policy(collection);
            safe_l004(body, pending || has_pol)
        }
        Workflow::With { body, .. } => safe_l004(body, pending),
        Workflow::Maybe {
            primary, fallback, ..
        } => safe_l004(primary, pending) && safe_l004(fallback, pending),
        Workflow::Must { body, .. } => safe_l004(body, pending),
        Workflow::Receive { arms, .. } => arms.iter().all(|a| {
            let guard_pol = a.guard.as_ref().is_some_and(workflow_expr_has_policy);
            safe_l004(&a.body, pending || guard_pol)
        }),
        Workflow::Yield { arms, .. } => arms.iter().all(|a| safe_l004(&a.body, pending)),
        Workflow::Resume { expr, .. } => !(pending || workflow_expr_has_policy(expr)),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
