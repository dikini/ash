//! Surface AST formatting for REPL display.

use ash_parser::surface::{
    Constraint, Expr, MatchArm, Pattern, PolicyExpr, Predicate, Type, VariantPatternPayload,
};
use std::fmt::Write;

pub fn display_expr(expr: &Expr) -> String {
    render_expr(expr)
}

#[allow(clippy::too_many_lines)]
fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::OperatorSection { section } => {
            let mut out = String::from("OperatorSection {\n");
            push_field(&mut out, 2, "kind", &format!("{:?}", section.kind));
            push_field(
                &mut out,
                2,
                "operator",
                &format!("{:?}", section.operator.spelling),
            );
            if let Some(left) = &section.left {
                push_field(&mut out, 2, "left", &render_expr(left));
            }
            if let Some(right) = &section.right {
                push_field(&mut out, 2, "right", &render_expr(right));
            }
            out.push('}');
            out
        }
        Expr::Literal(literal) => format!("Literal({literal:?})"),
        Expr::Variable { name, .. } => format!("Variable({name:?})"),
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
        Expr::MacroInvocation { invocation } => {
            let mut out = String::from("MacroInvocation {\n");
            push_field(&mut out, 2, "name", &format!("{:?}", invocation.name));
            push_field(
                &mut out,
                2,
                "delimiter",
                &format!("{:?}", invocation.delimiter),
            );
            push_field(
                &mut out,
                2,
                "raw_body",
                &format!("{:?}", invocation.raw_body),
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
        Expr::Record { fields, .. } => {
            let mut out = String::from("Record {\n");
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
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            let mut out = String::from("If {\n");
            push_field(&mut out, 2, "condition", &render_expr(condition));
            push_field(&mut out, 2, "then_branch", &render_expr(then_branch));
            if let Some(e) = else_branch {
                push_field(&mut out, 2, "else_branch", &render_expr(e));
            }
            out.push('}');
            out
        }
        Expr::Panic { message, .. } => {
            let mut out = String::from("Panic {\n");
            push_field(&mut out, 2, "message", &format!("{message:?}"));
            out.push('}');
            out
        }
        Expr::Fail { payload, .. } => {
            let mut out = String::from("Fail {\n");
            push_field(&mut out, 2, "payload", &render_expr(payload));
            out.push('}');
            out
        }
        Expr::WithError { body, arms, .. } => {
            let mut out = String::from("WithError {\n");
            push_field(&mut out, 2, "body", &render_expr(body));
            push_field(
                &mut out,
                2,
                "arms",
                &render_list(arms.iter().map(render_match_arm)),
            );
            out.push('}');
            out
        }
        Expr::On {
            computation,
            clauses,
            ..
        } => {
            let mut out = String::from("On {\n");
            push_field(&mut out, 2, "computation", &render_expr(computation));
            push_field(
                &mut out,
                2,
                "clauses",
                &render_list(clauses.iter().map(|clause| match clause {
                    ash_parser::surface::HandlerClause::Operation {
                        impl_type,
                        operation,
                        pattern,
                        resume,
                        body,
                        ..
                    } => {
                        let mut clause_out = String::from("Operation {\n");
                        push_field(&mut clause_out, 2, "impl_type", &format!("{impl_type:?}"));
                        push_field(&mut clause_out, 2, "operation", &format!("{operation:?}"));
                        push_field(&mut clause_out, 2, "pattern", &render_pattern(pattern));
                        push_field(&mut clause_out, 2, "resume", &format!("{resume:?}"));
                        push_field(&mut clause_out, 2, "body", &render_expr(body));
                        clause_out.push('}');
                        clause_out
                    }
                    ash_parser::surface::HandlerClause::Done { binding, body, .. } => {
                        let mut clause_out = String::from("Done {\n");
                        push_field(&mut clause_out, 2, "binding", &format!("{binding:?}"));
                        push_field(&mut clause_out, 2, "body", &render_expr(body));
                        clause_out.push('}');
                        clause_out
                    }
                })),
            );
            out.push('}');
            out
        }
        Expr::HandleWith {
            expression,
            handler,
            ..
        } => {
            let mut out = String::from("HandleWith {\n");
            push_field(&mut out, 2, "expression", &render_expr(expression));
            push_field(&mut out, 2, "handler", &format!("{handler:?}"));
            out.push('}');
            out
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut out = String::from("Block {\n");
            push_field(
                &mut out,
                2,
                "statements",
                &render_list(statements.iter().map(|stmt| match stmt {
                    ash_parser::surface::BlockStmt::Let { pattern, expr, .. } => {
                        let mut s = String::from("Let {\n");
                        push_field(&mut s, 2, "pattern", &format!("{pattern:?}"));
                        push_field(&mut s, 2, "expr", &render_expr(expr));
                        s.push('}');
                        s
                    }
                    ash_parser::surface::BlockStmt::Expr { expr, .. } => {
                        format!("Expr({})", render_expr(expr))
                    }
                })),
            );
            if let Some(e) = tail_expr {
                push_field(&mut out, 2, "tail_expr", &render_expr(e));
            }
            out.push('}');
            out
        }
        Expr::FnDef {
            params,
            return_type,
            body,
            ..
        } => {
            let mut out = String::from("FnDef {\n");
            push_field(&mut out, 2, "params", &format!("{params:?}"));
            push_optional_debug_field(&mut out, 2, "return_type", return_type.as_ref());
            push_field(&mut out, 2, "body", &render_expr(body));
            out.push('}');
            out
        }
        Expr::FnApply { func, args, .. } => {
            let mut out = String::from("FnApply {\n");
            push_field(&mut out, 2, "func", &render_expr(func));
            push_field(
                &mut out,
                2,
                "args",
                &render_list(args.iter().map(render_expr)),
            );
            out.push('}');
            out
        }
        Expr::DoBlock { target, stmts, .. } => {
            let mut out = String::from("DoBlock {\n");
            push_field(&mut out, 2, "target", &format!("{:?}", target.name));
            push_field(
                &mut out,
                2,
                "stmts",
                &render_list(stmts.iter().map(|stmt| match stmt {
                    ash_parser::surface::DoStmt::Let { name, value, .. } => {
                        format!("Let({name:?}, {})", render_expr(value))
                    }
                    ash_parser::surface::DoStmt::Bind { name, value, .. } => {
                        format!("Bind({name:?}, {})", render_expr(value))
                    }
                    ash_parser::surface::DoStmt::Expr { value, .. } => {
                        format!("Expr({})", render_expr(value))
                    }
                    ash_parser::surface::DoStmt::Return { value, .. } => {
                        format!("Return({})", render_expr(value))
                    }
                })),
            );
            out.push('}');
            out
        }
        Expr::List { items, .. } => {
            let mut out = String::from("List {\n");
            push_field(
                &mut out,
                2,
                "items",
                &render_list(items.iter().map(render_expr)),
            );
            out.push('}');
            out
        }
        Expr::Comprehension {
            result,
            qualifiers,
            target,
            ..
        } => {
            let mut out = String::from("Comprehension {\n");
            push_field(&mut out, 2, "result", &render_expr(result));
            push_field(
                &mut out,
                2,
                "qualifiers",
                &render_list(qualifiers.iter().map(|qualifier| match qualifier {
                    ash_parser::surface::ComprehensionQualifier::Let { name, value, .. } => {
                        format!("Let({name:?}, {})", render_expr(value))
                    }
                    ash_parser::surface::ComprehensionQualifier::Bind { name, value, .. } => {
                        format!("Bind({name:?}, {})", render_expr(value))
                    }
                    ash_parser::surface::ComprehensionQualifier::DiscardBind { value, .. } => {
                        format!("DiscardBind({})", render_expr(value))
                    }
                })),
            );
            if let Some(target) = target {
                push_field(&mut out, 2, "target", &format!("{:?}", target.name));
            }
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

fn render_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Variable { name, .. } => format!("Variable({name:?})"),
        Pattern::Wildcard => String::from("Wildcard"),
        Pattern::Tuple(items) => {
            format!("Tuple({})", render_list(items.iter().map(render_pattern)))
        }
        Pattern::Record(fields) => format!(
            "Record({})",
            render_list(fields.iter().map(|(name, pattern)| {
                format!("Field({name:?}, {})", render_pattern(pattern))
            }))
        ),
        Pattern::List { elements, rest } => {
            let mut out = String::from("ListPattern {\n");
            push_field(
                &mut out,
                2,
                "elements",
                &render_list(elements.iter().map(render_pattern)),
            );
            match rest {
                Some(rest) => push_field(&mut out, 2, "rest", &format!("Some({rest:?})")),
                None => push_field(&mut out, 2, "rest", "None"),
            }
            out.push('}');
            out
        }
        Pattern::Literal(literal) => format!("Literal({literal:?})"),
        Pattern::Variant {
            name,
            fields,
            payload,
        } => {
            let mut out = String::from("VariantPattern {\n");
            push_field(&mut out, 2, "name", &format!("{name:?}"));
            match fields {
                Some(fields) => push_field(
                    &mut out,
                    2,
                    "fields",
                    &render_list(fields.iter().map(|(field, pattern)| {
                        format!("Field({field:?}, {})", render_pattern(pattern))
                    })),
                ),
                None => push_field(&mut out, 2, "fields", "None"),
            }
            push_field(
                &mut out,
                2,
                "payload",
                &render_variant_pattern_payload(payload),
            );
            out.push('}');
            out
        }
    }
}

fn render_variant_pattern_payload(payload: &VariantPatternPayload) -> String {
    match payload {
        VariantPatternPayload::Unit => String::from("Unit"),
        VariantPatternPayload::Record(fields) => format!(
            "Record({})",
            render_list(fields.iter().map(|(field, pattern)| {
                format!("Field({field:?}, {})", render_pattern(pattern))
            }))
        ),
        VariantPatternPayload::Tuple(items) => {
            format!("Tuple({})", render_list(items.iter().map(render_pattern)))
        }
    }
}

fn render_policy_expr(expr: &PolicyExpr) -> String {
    match expr {
        PolicyExpr::Var { name, .. } => format!("Var({name:?})"),
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
