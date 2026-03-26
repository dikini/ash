//! Surface AST formatting for REPL display.

use ash_parser::surface::{
    ActionRef, CapabilityDecl, CheckTarget, Constraint, ConstraintBlock, ConstraintField,
    ConstraintValue, Contract, EnsuresClause, Expr, Guard, MatchArm, ObligationRef, Parameter,
    PolicyExpr, PolicyInstance, Predicate, ReceiveArm, Requirement, RoleRef, StreamPattern, Type,
    Workflow, WorkflowDef, YieldArm,
};
use std::fmt::Write;

pub fn display_expr(expr: &Expr) -> String {
    render_expr(expr)
}

pub fn display_workflow_def(def: &WorkflowDef) -> String {
    let mut out = String::from("WorkflowDef {\n");
    push_field(&mut out, 2, "name", &format!("{:?}", def.name));
    push_field(
        &mut out,
        2,
        "params",
        &render_list(def.params.iter().map(render_parameter)),
    );
    push_field(
        &mut out,
        2,
        "plays_roles",
        &render_list(def.plays_roles.iter().map(render_role_ref)),
    );
    push_field(
        &mut out,
        2,
        "capabilities",
        &render_list(def.capabilities.iter().map(render_capability_decl)),
    );
    push_field(&mut out, 2, "body", &render_workflow(&def.body));
    push_optional_field(
        &mut out,
        2,
        "contract",
        def.contract.as_ref(),
        render_contract,
    );
    out.push('}');
    out
}

#[allow(clippy::too_many_lines)]
fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(literal) => format!("Literal({literal:?})"),
        Expr::Variable(name) => format!("Variable({name:?})"),
        Expr::FieldAccess { base, field, .. } => {
            let mut out = String::from("FieldAccess {\n");
            push_field(&mut out, 2, "base", &render_expr(base));
            push_field(&mut out, 2, "field", &format!("{field:?}"));
            out.push('}');
            out
        }
        Expr::IndexAccess { base, index, .. } => {
            let mut out = String::from("IndexAccess {\n");
            push_field(&mut out, 2, "base", &render_expr(base));
            push_field(&mut out, 2, "index", &render_expr(index));
            out.push('}');
            out
        }
        Expr::Unary { op, operand, .. } => {
            let mut out = String::from("Unary {\n");
            push_field(&mut out, 2, "op", &format!("{op:?}"));
            push_field(&mut out, 2, "operand", &render_expr(operand));
            out.push('}');
            out
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let mut out = String::from("Binary {\n");
            push_field(&mut out, 2, "op", &format!("{op:?}"));
            push_field(&mut out, 2, "left", &render_expr(left));
            push_field(&mut out, 2, "right", &render_expr(right));
            out.push('}');
            out
        }
        Expr::Call { func, args, .. } => {
            let mut out = String::from("Call {\n");
            push_field(&mut out, 2, "func", &format!("{func:?}"));
            push_field(
                &mut out,
                2,
                "args",
                &render_list(args.iter().map(render_expr)),
            );
            out.push('}');
            out
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            let mut out = String::from("Match {\n");
            push_field(&mut out, 2, "scrutinee", &render_expr(scrutinee));
            push_field(
                &mut out,
                2,
                "arms",
                &render_list(arms.iter().map(render_match_arm)),
            );
            out.push('}');
            out
        }
        Expr::Policy(policy) => {
            let mut out = String::from("Policy {\n");
            push_field(&mut out, 2, "expr", &render_policy_expr(policy));
            out.push('}');
            out
        }
        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            let mut out = String::from("IfLet {\n");
            push_field(&mut out, 2, "pattern", &format!("{pattern:?}"));
            push_field(&mut out, 2, "expr", &render_expr(expr));
            push_field(&mut out, 2, "then_branch", &render_expr(then_branch));
            push_field(&mut out, 2, "else_branch", &render_expr(else_branch));
            out.push('}');
            out
        }
        Expr::CheckObligation { obligation, .. } => {
            let mut out = String::from("CheckObligation {\n");
            push_field(&mut out, 2, "obligation", &format!("{obligation:?}"));
            out.push('}');
            out
        }
        Expr::Constructor { name, fields, .. } => {
            let mut out = String::from("Constructor {\n");
            push_field(&mut out, 2, "name", &format!("{name:?}"));
            push_field(
                &mut out,
                2,
                "fields",
                &render_list(fields.iter().map(|(field, expr)| {
                    let mut field_out = String::from("Field {\n");
                    push_field(&mut field_out, 2, "name", &format!("{field:?}"));
                    push_field(&mut field_out, 2, "value", &render_expr(expr));
                    field_out.push('}');
                    field_out
                })),
            );
            out.push('}');
            out
        }
    }
}

#[allow(clippy::too_many_lines)]
fn render_workflow(workflow: &Workflow) -> String {
    match workflow {
        Workflow::Observe {
            capability,
            binding,
            continuation,
            ..
        } => {
            let mut out = String::from("Observe {\n");
            push_field(&mut out, 2, "capability", &format!("{capability:?}"));
            push_optional_debug_field(&mut out, 2, "binding", binding.as_ref());
            push_optional_field(
                &mut out,
                2,
                "continuation",
                continuation.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::Orient {
            expr,
            binding,
            continuation,
            ..
        } => {
            let mut out = String::from("Orient {\n");
            push_field(&mut out, 2, "expr", &render_expr(expr));
            push_optional_debug_field(&mut out, 2, "binding", binding.as_ref());
            push_optional_field(
                &mut out,
                2,
                "continuation",
                continuation.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::Propose {
            action,
            binding,
            continuation,
            ..
        } => {
            let mut out = String::from("Propose {\n");
            push_field(&mut out, 2, "action", &render_action_ref(action));
            push_optional_debug_field(&mut out, 2, "binding", binding.as_ref());
            push_optional_field(
                &mut out,
                2,
                "continuation",
                continuation.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::Decide {
            expr,
            policy,
            then_branch,
            else_branch,
            ..
        } => {
            let mut out = String::from("Decide {\n");
            push_field(&mut out, 2, "expr", &render_expr(expr));
            push_optional_debug_field(&mut out, 2, "policy", policy.as_ref());
            push_field(&mut out, 2, "then_branch", &render_workflow(then_branch));
            push_optional_field(
                &mut out,
                2,
                "else_branch",
                else_branch.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::Check {
            target,
            continuation,
            ..
        } => {
            let mut out = String::from("Check {\n");
            push_field(&mut out, 2, "target", &render_check_target(target));
            push_optional_field(
                &mut out,
                2,
                "continuation",
                continuation.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::Oblige { obligation, .. } => {
            let mut out = String::from("Oblige {\n");
            push_field(&mut out, 2, "obligation", &format!("{obligation:?}"));
            out.push('}');
            out
        }
        Workflow::Act { action, guard, .. } => {
            let mut out = String::from("Act {\n");
            push_field(&mut out, 2, "action", &render_action_ref(action));
            push_optional_field(&mut out, 2, "guard", guard.as_ref(), render_guard);
            out.push('}');
            out
        }
        Workflow::Let {
            pattern,
            expr,
            continuation,
            ..
        } => {
            let mut out = String::from("Let {\n");
            push_field(&mut out, 2, "pattern", &format!("{pattern:?}"));
            push_field(&mut out, 2, "expr", &render_expr(expr));
            push_optional_field(
                &mut out,
                2,
                "continuation",
                continuation.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            let mut out = String::from("If {\n");
            push_field(&mut out, 2, "condition", &render_expr(condition));
            push_field(&mut out, 2, "then_branch", &render_workflow(then_branch));
            push_optional_field(
                &mut out,
                2,
                "else_branch",
                else_branch.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::For {
            pattern,
            collection,
            body,
            ..
        } => {
            let mut out = String::from("For {\n");
            push_field(&mut out, 2, "pattern", &format!("{pattern:?}"));
            push_field(&mut out, 2, "collection", &render_expr(collection));
            push_field(&mut out, 2, "body", &render_workflow(body));
            out.push('}');
            out
        }
        Workflow::Par { branches, .. } => {
            let mut out = String::from("Par {\n");
            push_field(
                &mut out,
                2,
                "branches",
                &render_list(branches.iter().map(render_workflow)),
            );
            out.push('}');
            out
        }
        Workflow::With {
            capability, body, ..
        } => {
            let mut out = String::from("With {\n");
            push_field(&mut out, 2, "capability", &format!("{capability:?}"));
            push_field(&mut out, 2, "body", &render_workflow(body));
            out.push('}');
            out
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => {
            let mut out = String::from("Maybe {\n");
            push_field(&mut out, 2, "primary", &render_workflow(primary));
            push_field(&mut out, 2, "fallback", &render_workflow(fallback));
            out.push('}');
            out
        }
        Workflow::Must { body, .. } => {
            let mut out = String::from("Must {\n");
            push_field(&mut out, 2, "body", &render_workflow(body));
            out.push('}');
            out
        }
        Workflow::Seq { first, second, .. } => {
            let mut out = String::from("Seq {\n");
            push_field(&mut out, 2, "first", &render_workflow(first));
            push_field(&mut out, 2, "second", &render_workflow(second));
            out.push('}');
            out
        }
        Workflow::Done { .. } => String::from("Done"),
        Workflow::Ret { expr, .. } => {
            let mut out = String::from("Ret {\n");
            push_field(&mut out, 2, "expr", &render_expr(expr));
            out.push('}');
            out
        }
        Workflow::Set {
            capability,
            channel,
            value,
            continuation,
            ..
        } => {
            let mut out = String::from("Set {\n");
            push_field(&mut out, 2, "capability", &format!("{capability:?}"));
            push_field(&mut out, 2, "channel", &format!("{channel:?}"));
            push_field(&mut out, 2, "value", &render_expr(value));
            push_optional_field(
                &mut out,
                2,
                "continuation",
                continuation.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::Send {
            capability,
            channel,
            value,
            continuation,
            ..
        } => {
            let mut out = String::from("Send {\n");
            push_field(&mut out, 2, "capability", &format!("{capability:?}"));
            push_field(&mut out, 2, "channel", &format!("{channel:?}"));
            push_field(&mut out, 2, "value", &render_expr(value));
            push_optional_field(
                &mut out,
                2,
                "continuation",
                continuation.as_deref(),
                render_workflow,
            );
            out.push('}');
            out
        }
        Workflow::Receive {
            mode,
            arms,
            is_control,
            ..
        } => {
            let mut out = String::from("Receive {\n");
            push_field(&mut out, 2, "mode", &format!("{mode:?}"));
            push_field(
                &mut out,
                2,
                "arms",
                &render_list(arms.iter().map(render_receive_arm)),
            );
            push_field(&mut out, 2, "is_control", &is_control.to_string());
            out.push('}');
            out
        }
        Workflow::Yield {
            role,
            expr,
            resume_var,
            resume_type,
            arms,
            ..
        } => {
            let mut out = String::from("Yield {\n");
            push_field(&mut out, 2, "role", &format!("{role:?}"));
            push_field(&mut out, 2, "expr", &render_expr(expr));
            push_field(&mut out, 2, "resume_var", &format!("{resume_var:?}"));
            push_field(&mut out, 2, "resume_type", &format!("{resume_type:?}"));
            push_field(
                &mut out,
                2,
                "arms",
                &render_list(arms.iter().map(render_yield_arm)),
            );
            out.push('}');
            out
        }
        Workflow::Resume { expr, ty, .. } => {
            let mut out = String::from("Resume {\n");
            push_field(&mut out, 2, "expr", &render_expr(expr));
            push_field(&mut out, 2, "ty", &format!("{ty:?}"));
            out.push('}');
            out
        }
    }
}

fn render_match_arm(arm: &MatchArm) -> String {
    let mut out = String::from("MatchArm {\n");
    push_field(&mut out, 2, "pattern", &format!("{:?}", arm.pattern));
    push_field(&mut out, 2, "body", &render_expr(&arm.body));
    out.push('}');
    out
}

fn render_policy_expr(expr: &PolicyExpr) -> String {
    match expr {
        PolicyExpr::Var(name) => format!("Var({name:?})"),
        PolicyExpr::And(exprs) => {
            format!("And({})", render_list(exprs.iter().map(render_policy_expr)))
        }
        PolicyExpr::Or(exprs) => {
            format!("Or({})", render_list(exprs.iter().map(render_policy_expr)))
        }
        PolicyExpr::Not(expr) => format!("Not({})", render_policy_expr(expr)),
        PolicyExpr::Implies(left, right) => {
            let mut out = String::from("Implies {\n");
            push_field(&mut out, 2, "left", &render_policy_expr(left));
            push_field(&mut out, 2, "right", &render_policy_expr(right));
            out.push('}');
            out
        }
        PolicyExpr::Sequential(exprs) => {
            format!(
                "Sequential({})",
                render_list(exprs.iter().map(render_policy_expr))
            )
        }
        PolicyExpr::Concurrent(exprs) => {
            format!(
                "Concurrent({})",
                render_list(exprs.iter().map(render_policy_expr))
            )
        }
        PolicyExpr::ForAll {
            var, items, body, ..
        } => {
            let mut out = String::from("ForAll {\n");
            push_field(&mut out, 2, "var", &format!("{var:?}"));
            push_field(&mut out, 2, "items", &render_expr(items));
            push_field(&mut out, 2, "body", &render_policy_expr(body));
            out.push('}');
            out
        }
        PolicyExpr::Exists {
            var, items, body, ..
        } => {
            let mut out = String::from("Exists {\n");
            push_field(&mut out, 2, "var", &format!("{var:?}"));
            push_field(&mut out, 2, "items", &render_expr(items));
            push_field(&mut out, 2, "body", &render_policy_expr(body));
            out.push('}');
            out
        }
        PolicyExpr::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            let mut out = String::from("MethodCall {\n");
            push_field(&mut out, 2, "receiver", &render_policy_expr(receiver));
            push_field(&mut out, 2, "method", &format!("{method:?}"));
            push_field(
                &mut out,
                2,
                "args",
                &render_list(args.iter().map(render_expr)),
            );
            out.push('}');
            out
        }
        PolicyExpr::Call { func, args, .. } => {
            let mut out = String::from("Call {\n");
            push_field(&mut out, 2, "func", &format!("{func:?}"));
            push_field(
                &mut out,
                2,
                "args",
                &render_list(args.iter().map(render_expr)),
            );
            out.push('}');
            out
        }
    }
}

fn render_action_ref(action: &ActionRef) -> String {
    let mut out = String::from("ActionRef {\n");
    push_field(&mut out, 2, "name", &format!("{:?}", action.name));
    push_field(
        &mut out,
        2,
        "args",
        &render_list(action.args.iter().map(render_expr)),
    );
    out.push('}');
    out
}

fn render_check_target(target: &CheckTarget) -> String {
    match target {
        CheckTarget::Obligation(obligation) => {
            let mut out = String::from("Obligation {\n");
            push_field(&mut out, 2, "value", &render_obligation_ref(obligation));
            out.push('}');
            out
        }
        CheckTarget::Policy(policy) => {
            let mut out = String::from("Policy {\n");
            push_field(&mut out, 2, "value", &render_policy_instance(policy));
            out.push('}');
            out
        }
    }
}

fn render_obligation_ref(obligation: &ObligationRef) -> String {
    let mut out = String::from("ObligationRef {\n");
    push_field(&mut out, 2, "role", &format!("{:?}", obligation.role));
    push_field(
        &mut out,
        2,
        "condition",
        &render_expr(&obligation.condition),
    );
    out.push('}');
    out
}

fn render_policy_instance(policy: &PolicyInstance) -> String {
    let mut out = String::from("PolicyInstance {\n");
    push_field(&mut out, 2, "name", &format!("{:?}", policy.name));
    push_field(
        &mut out,
        2,
        "fields",
        &render_list(policy.fields.iter().map(|(name, expr)| {
            let mut field_out = String::from("Field {\n");
            push_field(&mut field_out, 2, "name", &format!("{name:?}"));
            push_field(&mut field_out, 2, "value", &render_expr(expr));
            field_out.push('}');
            field_out
        })),
    );
    out.push('}');
    out
}

fn render_guard(guard: &Guard) -> String {
    match guard {
        Guard::Always => String::from("Always"),
        Guard::Never => String::from("Never"),
        Guard::Pred(predicate) => {
            let mut out = String::from("Pred {\n");
            push_field(&mut out, 2, "predicate", &render_predicate(predicate));
            out.push('}');
            out
        }
        Guard::And(left, right) => {
            let mut out = String::from("And {\n");
            push_field(&mut out, 2, "left", &render_guard(left));
            push_field(&mut out, 2, "right", &render_guard(right));
            out.push('}');
            out
        }
        Guard::Or(left, right) => {
            let mut out = String::from("Or {\n");
            push_field(&mut out, 2, "left", &render_guard(left));
            push_field(&mut out, 2, "right", &render_guard(right));
            out.push('}');
            out
        }
        Guard::Not(inner) => {
            let mut out = String::from("Not {\n");
            push_field(&mut out, 2, "guard", &render_guard(inner));
            out.push('}');
            out
        }
    }
}

fn render_predicate(predicate: &Predicate) -> String {
    let mut out = String::from("Predicate {\n");
    push_field(&mut out, 2, "name", &format!("{:?}", predicate.name));
    push_field(
        &mut out,
        2,
        "args",
        &render_list(predicate.args.iter().map(render_expr)),
    );
    out.push('}');
    out
}

fn render_parameter(parameter: &Parameter) -> String {
    let mut out = String::from("Parameter {\n");
    push_field(&mut out, 2, "name", &format!("{:?}", parameter.name));
    push_field(&mut out, 2, "ty", &format!("{:?}", parameter.ty));
    out.push('}');
    out
}

fn render_role_ref(role: &RoleRef) -> String {
    let mut out = String::from("RoleRef {\n");
    push_field(&mut out, 2, "name", &format!("{:?}", role.name));
    out.push('}');
    out
}

fn render_capability_decl(capability: &CapabilityDecl) -> String {
    let mut out = String::from("CapabilityDecl {\n");
    push_field(
        &mut out,
        2,
        "capability",
        &format!("{:?}", capability.capability),
    );
    push_optional_field(
        &mut out,
        2,
        "constraints",
        capability.constraints.as_ref(),
        render_constraint_block,
    );
    out.push('}');
    out
}

fn render_contract(contract: &Contract) -> String {
    let mut out = String::from("Contract {\n");
    push_field(
        &mut out,
        2,
        "requires",
        &render_list(contract.requires.iter().map(render_requirement)),
    );
    push_field(
        &mut out,
        2,
        "ensures",
        &render_list(contract.ensures.iter().map(render_ensures_clause)),
    );
    out.push('}');
    out
}

fn render_requirement(requirement: &Requirement) -> String {
    match requirement {
        Requirement::HasCapability { cap, min_effect } => {
            let mut out = String::from("HasCapability {\n");
            push_field(&mut out, 2, "cap", &format!("{cap:?}"));
            push_field(&mut out, 2, "min_effect", &format!("{min_effect:?}"));
            out.push('}');
            out
        }
        Requirement::HasRole(name) => format!("HasRole({name:?})"),
        Requirement::Arithmetic { expr } => {
            let mut out = String::from("Arithmetic {\n");
            push_field(&mut out, 2, "expr", &render_expr(expr));
            out.push('}');
            out
        }
    }
}

fn render_ensures_clause(clause: &EnsuresClause) -> String {
    let mut out = String::from("EnsuresClause {\n");
    push_field(&mut out, 2, "expr", &render_expr(&clause.expr));
    out.push('}');
    out
}

fn render_constraint_block(block: &ConstraintBlock) -> String {
    let mut out = String::from("ConstraintBlock {\n");
    push_field(
        &mut out,
        2,
        "fields",
        &render_list(block.fields.iter().map(render_constraint_field)),
    );
    out.push('}');
    out
}

fn render_constraint_field(field: &ConstraintField) -> String {
    let mut out = String::from("ConstraintField {\n");
    push_field(&mut out, 2, "name", &format!("{:?}", field.name));
    push_field(&mut out, 2, "value", &render_constraint_value(&field.value));
    out.push('}');
    out
}

fn render_constraint_value(value: &ConstraintValue) -> String {
    match value {
        ConstraintValue::Bool(value) => format!("Bool({value})"),
        ConstraintValue::Int(value) => format!("Int({value})"),
        ConstraintValue::String(value) => format!("String({value:?})"),
        ConstraintValue::Array(values) => {
            format!(
                "Array({})",
                render_list(values.iter().map(render_constraint_value))
            )
        }
        ConstraintValue::Object(fields) => {
            let mut out = String::from("Object {\n");
            push_field(
                &mut out,
                2,
                "fields",
                &render_list(fields.iter().map(|(name, value)| {
                    let mut field_out = String::from("Field {\n");
                    push_field(&mut field_out, 2, "name", &format!("{name:?}"));
                    push_field(&mut field_out, 2, "value", &render_constraint_value(value));
                    field_out.push('}');
                    field_out
                })),
            );
            out.push('}');
            out
        }
    }
}

fn render_receive_arm(arm: &ReceiveArm) -> String {
    let mut out = String::from("ReceiveArm {\n");
    push_field(&mut out, 2, "pattern", &render_stream_pattern(&arm.pattern));
    push_optional_field(&mut out, 2, "guard", arm.guard.as_ref(), render_expr);
    push_field(&mut out, 2, "body", &render_workflow(&arm.body));
    out.push('}');
    out
}

fn render_stream_pattern(pattern: &StreamPattern) -> String {
    match pattern {
        StreamPattern::Wildcard => String::from("Wildcard"),
        StreamPattern::Literal(value) => format!("Literal({value:?})"),
        StreamPattern::Binding {
            capability,
            channel,
            pattern,
        } => {
            let mut out = String::from("Binding {\n");
            push_field(&mut out, 2, "capability", &format!("{capability:?}"));
            push_field(&mut out, 2, "channel", &format!("{channel:?}"));
            push_field(&mut out, 2, "pattern", &format!("{pattern:?}"));
            out.push('}');
            out
        }
    }
}

fn render_yield_arm(arm: &YieldArm) -> String {
    let mut out = String::from("YieldArm {\n");
    push_field(&mut out, 2, "pattern", &format!("{:?}", arm.pattern));
    push_field(&mut out, 2, "body", &render_workflow(&arm.body));
    out.push('}');
    out
}

fn render_list<I>(items: I) -> String
where
    I: IntoIterator<Item = String>,
{
    let rendered: Vec<String> = items.into_iter().collect();
    if rendered.is_empty() {
        return String::from("[]");
    }

    let mut out = String::from("[\n");
    for item in rendered {
        push_list_item(&mut out, 2, &item);
    }
    out.push(']');
    out
}

fn push_field(out: &mut String, indent: usize, name: &str, value: &str) {
    if !value.contains('\n') {
        let _ = writeln!(out, "{}{}: {},", " ".repeat(indent), name, value);
        return;
    }

    let mut lines = value.lines();
    if let Some(first) = lines.next() {
        let _ = writeln!(out, "{}{}: {}", " ".repeat(indent), name, first);
    }

    let rest: Vec<&str> = lines.collect();
    for (index, line) in rest.iter().enumerate() {
        let suffix = if index + 1 == rest.len() { "," } else { "" };
        let _ = writeln!(out, "{}{}{}", " ".repeat(indent), line, suffix);
    }
}

fn push_list_item(out: &mut String, indent: usize, value: &str) {
    if !value.contains('\n') {
        let _ = writeln!(out, "{}{},", " ".repeat(indent), value);
        return;
    }

    let mut lines = value.lines();
    if let Some(first) = lines.next() {
        let _ = writeln!(out, "{}{}", " ".repeat(indent), first);
    }

    let rest: Vec<&str> = lines.collect();
    for (index, line) in rest.iter().enumerate() {
        let suffix = if index + 1 == rest.len() { "," } else { "" };
        let _ = writeln!(out, "{}{}{}", " ".repeat(indent), line, suffix);
    }
}

fn push_optional_field<T>(
    out: &mut String,
    indent: usize,
    name: &str,
    value: Option<&T>,
    render: fn(&T) -> String,
) {
    match value {
        Some(value) => push_field(out, indent, name, &render(value)),
        None => push_field(out, indent, name, "None"),
    }
}

fn push_optional_debug_field<T: std::fmt::Debug>(
    out: &mut String,
    indent: usize,
    name: &str,
    value: Option<&T>,
) {
    match value {
        Some(value) => push_field(out, indent, name, &format!("{value:?}")),
        None => push_field(out, indent, name, "None"),
    }
}

#[allow(dead_code)]
fn render_type(ty: &Type) -> String {
    format!("{ty:?}")
}

#[allow(dead_code)]
fn render_constraint(constraint: &Constraint) -> String {
    let mut out = String::from("Constraint {\n");
    push_field(
        &mut out,
        2,
        "predicate",
        &render_predicate(&constraint.predicate),
    );
    out.push('}');
    out
}
