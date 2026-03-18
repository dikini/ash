//! Workflow statement parser for the Ash language.
//!
//! This module provides parsers for Ash workflow statements and definitions.

use winnow::combinator::{alt, delimited};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::{ParseInput, Position};
use crate::parse_expr::expr;
use crate::parse_send::parse_send;
use crate::parse_set::parse_set;
use crate::surface::{
    ActionRef, CheckTarget, Expr, Guard, Name, ObligationRef, Pattern, PolicyInstance, Workflow,
    WorkflowDef,
};
use crate::token::Span;

/// Parse a workflow definition: `workflow <name> { <body> }`
pub fn workflow_def(input: &mut ParseInput) -> ModalResult<WorkflowDef> {
    let start_pos = input.state;

    let _ = keyword("workflow").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    let body = delimited(literal_str("{"), workflow, literal_str("}")).parse_next(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(WorkflowDef {
        name: name.into(),
        body,
        span,
    })
}

/// Parse a workflow body - sequence of statements separated by semicolons
pub fn workflow(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    // Parse statements
    let stmts = parse_stmt_list(input)?;

    if stmts.is_empty() {
        return Ok(Workflow::Done {
            span: span_from(&start_pos, &input.state),
        });
    }

    if stmts.len() == 1 {
        return Ok(stmts.into_iter().next().unwrap());
    }

    // Combine statements into sequential composition
    let mut result = stmts[0].clone();
    for stmt in &stmts[1..] {
        let span = span_from(&start_pos, &input.state);
        result = Workflow::Seq {
            first: Box::new(result),
            second: Box::new(stmt.clone()),
            span,
        };
    }

    Ok(result)
}

/// Parse a list of statements separated by semicolons
fn parse_stmt_list(input: &mut ParseInput) -> ModalResult<Vec<Workflow>> {
    let mut stmts = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        // Check for closing brace or EOF
        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        // Try to parse a statement
        match parse_stmt(input) {
            Ok(stmt) => {
                stmts.push(stmt);

                // Check for semicolon
                skip_whitespace_and_comments(input);
                if input.input.starts_with(";") {
                    let _ = input.input.next_slice(1);
                    input.state.advance(';');
                } else if !input.input.is_empty() && !input.input.starts_with("}") {
                    // Statement without semicolon - that's okay if followed by }
                    continue;
                }
            }
            Err(_) => break,
        }

        skip_whitespace_and_comments(input);
    }

    Ok(stmts)
}

/// Parse a single workflow statement
fn parse_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    alt((
        observe_stmt,
        orient_stmt,
        propose_stmt,
        decide_stmt,
        check_stmt,
        act_stmt,
        let_stmt,
        if_stmt,
        for_stmt,
        par_stmt,
        with_stmt,
        maybe_stmt,
        must_stmt,
        set_stmt,
        send_stmt,
        ret_stmt,
        done_stmt,
    ))
    .parse_next(input)
}

/// Parse an observe statement: `observe <capability> [as <pattern>]`
fn observe_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("observe").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let capability = identifier(input)?;

    // Check for optional binding
    let binding = if keyword("as").parse_next(input).is_ok() {
        Some(pattern(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Observe {
        capability: capability.into(),
        binding,
        continuation: None,
        span,
    })
}

/// Parse an orient statement: `orient <expr> [as <pattern>]`
fn orient_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("orient").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let e = expr(input)?;

    // Check for optional binding
    let binding = if keyword("as").parse_next(input).is_ok() {
        Some(pattern(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Orient {
        expr: e,
        binding,
        continuation: None,
        span,
    })
}

/// Parse a propose statement: `propose <action> [as <pattern>]`
fn propose_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("propose").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let action = action_ref(input)?;

    // Check for optional binding
    let binding = if keyword("as").parse_next(input).is_ok() {
        Some(pattern(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Propose {
        action,
        binding,
        continuation: None,
        span,
    })
}

/// Parse a decide statement: `decide <expr> [with <policy>] then <workflow> [else <workflow>]`
fn decide_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("decide").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let e = expr(input)?;

    // Optional policy name
    let policy = if keyword("with").parse_next(input).is_ok() {
        Some(identifier(input)?.into())
    } else {
        None
    };

    let _ = keyword("then").parse_next(input)?;
    let then_branch = Box::new(parse_single_stmt_or_block(input)?);

    let else_branch = if keyword("else").parse_next(input).is_ok() {
        Some(Box::new(parse_single_stmt_or_block(input)?))
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Decide {
        expr: e,
        policy,
        then_branch,
        else_branch,
        span,
    })
}

/// Parse a check statement: `check <obligation>` or `check <policy_instance>`
fn check_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("check").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let target = check_target(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Check {
        target,
        continuation: None,
        span,
    })
}

/// Parse a check target - either an obligation reference or a policy instance.
fn check_target(input: &mut ParseInput) -> ModalResult<CheckTarget> {
    // Try to parse as policy instance first (identifier { ... })
    if let Ok(target) = policy_instance(input) {
        return Ok(CheckTarget::Policy(target));
    }

    // Otherwise parse as obligation reference (role.condition)
    obligation_ref(input).map(CheckTarget::Obligation)
}

/// Parse a policy instance: `PolicyName { field: expr, ... }`
fn policy_instance(input: &mut ParseInput) -> ModalResult<PolicyInstance> {
    let start_pos = input.state;
    let checkpoint = *input;

    let name = identifier(input)?;

    // Check for opening brace, restore checkpoint if not found
    if literal_str("{").parse_next(input).is_err() {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let fields = parse_policy_field_inits(input)?;
    let _ = literal_str("}").parse_next(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(PolicyInstance {
        name: name.into(),
        fields,
        span,
    })
}

/// Parse field initializations for a policy instance.
fn parse_policy_field_inits(input: &mut ParseInput) -> ModalResult<Vec<(Name, Expr)>> {
    let mut fields = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        let field_name = identifier(input)?;
        let _ = literal_str(":").parse_next(input)?;
        let value = expr(input)?;
        fields.push((field_name.into(), value));

        skip_whitespace_and_comments(input);

        // Optional comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        }
    }

    Ok(fields)
}

/// Parse an act statement: `act <action> [where <guard>]`
fn act_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("act").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let action = action_ref(input)?;

    // Optional guard
    let guard = if keyword("where").parse_next(input).is_ok() {
        Some(parse_guard(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Act {
        action,
        guard,
        span,
    })
}

/// Parse a let statement: `let <pattern> = <expr>`
fn let_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("let").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let pat = pattern(input)?;
    let _ = literal_str("=").parse_next(input)?;
    let e = expr(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Let {
        pattern: pat,
        expr: e,
        continuation: None,
        span,
    })
}

/// Parse an if statement: `if <expr> then <workflow> [else <workflow>]`
fn if_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("if").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let condition = expr(input)?;
    let _ = keyword("then").parse_next(input)?;
    let then_branch = Box::new(parse_single_stmt_or_block(input)?);

    let else_branch = if keyword("else").parse_next(input).is_ok() {
        Some(Box::new(parse_single_stmt_or_block(input)?))
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::If {
        condition,
        then_branch,
        else_branch,
        span,
    })
}

/// Parse a for statement: `for <pattern> in <expr> do <workflow>`
fn for_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("for").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let pat = pattern(input)?;
    let _ = keyword("in").parse_next(input)?;
    let collection = expr(input)?;
    let _ = keyword("do").parse_next(input)?;
    let body = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::For {
        pattern: pat,
        collection,
        body,
        span,
    })
}

/// Parse a parallel block: `par { <workflows> }`
fn par_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("par").parse_next(input)?;
    let branches =
        delimited(literal_str("{"), parse_par_branches, literal_str("}")).parse_next(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Par { branches, span })
}

/// Parse parallel branches
fn parse_par_branches(input: &mut ParseInput) -> ModalResult<Vec<Workflow>> {
    let mut branches = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        let branch = workflow(input)?;
        branches.push(branch);

        skip_whitespace_and_comments(input);

        // Optional comma or semicolon between branches
        if input.input.starts_with(",") || input.input.starts_with(";") {
            let _ = input.input.next_slice(1);
            input
                .state
                .advance(input.input.chars().next().unwrap_or(' '));
        }
    }

    Ok(branches)
}

/// Parse a with statement: `with <capability> do <workflow>`
fn with_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("with").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let capability = identifier(input)?;
    let _ = keyword("do").parse_next(input)?;
    let body = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::With {
        capability: capability.into(),
        body,
        span,
    })
}

/// Parse a maybe statement: `maybe <workflow> else <workflow>`
fn maybe_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("maybe").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let primary = Box::new(parse_single_stmt_or_block(input)?);
    let _ = keyword("else").parse_next(input)?;
    let fallback = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Maybe {
        primary,
        fallback,
        span,
    })
}

/// Parse a must statement: `must <workflow>`
fn must_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("must").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let body = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Must { body, span })
}

/// Parse a done statement: `done`
fn done_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("done").parse_next(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Done { span })
}

/// Parse a ret statement: `ret <expr>;`
fn ret_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state;

    let _ = keyword("ret").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let e = expr(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(Workflow::Ret { expr: e, span })
}

/// Parse a set statement in a workflow: `set capability:channel = expr`
fn set_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_span = input.state;
    let set_expr = parse_set(input)?;
    let span = span_from(&start_span, &input.state);

    Ok(Workflow::Set {
        capability: set_expr.capability,
        channel: set_expr.channel,
        value: set_expr.value,
        continuation: None,
        span,
    })
}

/// Parse a send statement in a workflow: `send capability:channel expr`
fn send_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_span = input.state;
    let send_expr = parse_send(input)?;
    let span = span_from(&start_span, &input.state);

    Ok(Workflow::Send {
        capability: send_expr.capability,
        channel: send_expr.channel,
        value: send_expr.value,
        continuation: None,
        span,
    })
}

/// Parse a single statement or a block
fn parse_single_stmt_or_block(input: &mut ParseInput) -> ModalResult<Workflow> {
    skip_whitespace_and_comments(input);

    if input.input.starts_with("{") {
        delimited(literal_str("{"), workflow, literal_str("}")).parse_next(input)
    } else {
        parse_stmt(input)
    }
}

/// Parse an action reference: `name(args...)`
pub fn action_ref(input: &mut ParseInput) -> ModalResult<ActionRef> {
    let name = identifier(input)?;

    let args = if input.input.starts_with("(") {
        delimited(literal_str("("), parse_expr_list, literal_str(")")).parse_next(input)?
    } else {
        vec![]
    };

    Ok(ActionRef {
        name: name.into(),
        args,
    })
}

/// Parse an obligation reference
fn obligation_ref(input: &mut ParseInput) -> ModalResult<ObligationRef> {
    // Simplified: role.condition
    let role = identifier(input)?;
    let _ = literal_str(".").parse_next(input)?;
    let condition = expr(input)?;

    Ok(ObligationRef {
        role: role.into(),
        condition,
    })
}

/// Parse a pattern
pub fn pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    skip_whitespace_and_comments(input);
    alt((
        parse_wildcard_pattern,
        parse_tuple_pattern,
        parse_list_pattern,
        parse_variable_pattern,
    ))
    .parse_next(input)
}

/// Parse a wildcard pattern: `_`
fn parse_wildcard_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let _ = literal_str("_").parse_next(input)?;
    Ok(Pattern::Wildcard)
}

/// Parse a variable pattern: just an identifier
fn parse_variable_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let name = identifier(input)?;
    Ok(Pattern::Variable(name.into()))
}

/// Parse a tuple pattern: `(pat1, pat2, ...)`
fn parse_tuple_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let patterns =
        delimited(literal_str("("), parse_pattern_list, literal_str(")")).parse_next(input)?;
    Ok(Pattern::Tuple(patterns))
}

/// Parse a list pattern: `[pat1, pat2, ..rest]`
fn parse_list_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    delimited(literal_str("["), parse_list_pattern_inner, literal_str("]")).parse_next(input)
}

/// Parse the inner content of a list pattern
fn parse_list_pattern_inner(input: &mut ParseInput) -> ModalResult<Pattern> {
    let mut elements = Vec::new();
    let mut rest = None;

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with("]") {
            break;
        }

        // Check for rest pattern: ..name
        if input.input.starts_with("..") {
            let _ = input.input.next_slice(2);
            input.state.advance('.');
            input.state.advance('.');
            rest = Some(identifier(input)?.into());
            break;
        }

        let pat = pattern(input)?;
        elements.push(pat);

        skip_whitespace_and_comments(input);

        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        } else {
            break;
        }
    }

    Ok(Pattern::List { elements, rest })
}

/// Parse a comma-separated list of patterns
fn parse_pattern_list(input: &mut ParseInput) -> ModalResult<Vec<Pattern>> {
    let mut patterns = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with(")") {
            break;
        }

        let pat = pattern(input)?;
        patterns.push(pat);

        skip_whitespace_and_comments(input);

        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        } else {
            break;
        }
    }

    Ok(patterns)
}

/// Parse a comma-separated list of expressions
fn parse_expr_list(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let mut exprs = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with(")") {
            break;
        }

        let e = expr(input)?;
        exprs.push(e);

        skip_whitespace_and_comments(input);

        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        } else {
            break;
        }
    }

    Ok(exprs)
}

/// Parse a guard expression
fn parse_guard(input: &mut ParseInput) -> ModalResult<Guard> {
    // Simplified guard parsing
    if keyword("always").parse_next(input).is_ok() {
        return Ok(Guard::Always);
    }

    if keyword("never").parse_next(input).is_ok() {
        return Ok(Guard::Never);
    }

    // Predicate guard: pred(args)
    let name = identifier(input)?;
    let args = if input.input.starts_with("(") {
        delimited(literal_str("("), parse_expr_list, literal_str(")")).parse_next(input)?
    } else {
        vec![]
    };

    Ok(Guard::Pred(crate::surface::Predicate {
        name: name.into(),
        args,
    }))
}

/// Parse a capability reference
pub fn capability_ref(input: &mut ParseInput) -> ModalResult<Name> {
    identifier(input).map(|s| s.into())
}

/// Parse an identifier.
fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    // Use take_while to match the entire identifier at once
    // First char: letter or underscore, rest: alphanumeric, underscore, or hyphen
    let result: &str = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    })
    .parse_next(input)?;

    // Check that first character is a letter or underscore (not a digit)
    if result.is_empty()
        || !result.chars().next().unwrap().is_ascii_alphabetic() && !result.starts_with('_')
    {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    // Check that it's not a keyword
    if is_keyword(result) {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(result)
}

/// Parse a keyword (ensures word boundary).
fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, winnow::error::ContextError> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);

        if input.input.starts_with(word) {
            let after = &input.input[word.len()..];
            if after.is_empty() || !after.chars().next().unwrap().is_ascii_alphanumeric() {
                // Update position state
                for c in word.chars() {
                    input.state.advance(c);
                }
                // Advance the inner stream
                let _ = input.input.next_slice(word.len());
                return Ok(word);
            }
        }
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

/// Whitespace wrapper.
#[allow(dead_code)]
fn ws<'a, F, O>(mut parser: F) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<O>
where
    F: FnMut(&mut ParseInput<'a>) -> ModalResult<O>,
{
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);
        let result = parser(input)?;
        skip_whitespace_and_comments(input);
        Ok(result)
    }
}

/// Parse a string literal token.
fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);
        if input.input.starts_with(s) {
            // Update position state
            for c in s.chars() {
                input.state.advance(c);
            }
            // Advance the inner stream
            let _ = input.input.next_slice(s.len());
            Ok(s)
        } else {
            Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ))
        }
    }
}

/// Skip whitespace and comments.
fn skip_whitespace_and_comments(input: &mut ParseInput) {
    loop {
        // Skip whitespace
        let _: ModalResult<&str> =
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input);

        // Check for line comment
        if input.input.starts_with("--") {
            let _: ModalResult<&str> = take_while(0.., |c: char| c != '\n').parse_next(input);
            continue;
        }

        // Check for block comment
        if input.input.starts_with("/*") {
            let _ = input.input.next_slice(2);
            let mut depth = 1;
            while depth > 0 && !input.input.is_empty() {
                if input.input.starts_with("/*") {
                    let _ = input.input.next_slice(2);
                    depth += 1;
                } else if input.input.starts_with("*/") {
                    let _ = input.input.next_slice(2);
                    depth -= 1;
                } else {
                    let _ = input.input.next_token();
                }
            }
            continue;
        }

        break;
    }
}

/// Check if a string is a keyword.
fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "workflow"
            | "capability"
            | "policy"
            | "role"
            | "observe"
            | "orient"
            | "propose"
            | "decide"
            | "act"
            | "oblige"
            | "check"
            | "let"
            | "if"
            | "then"
            | "else"
            | "for"
            | "do"
            | "par"
            | "with"
            | "maybe"
            | "must"
            | "set"
            | "send"
            | "attempt"
            | "retry"
            | "timeout"
            | "done"
            | "epistemic"
            | "deliberative"
            | "evaluative"
            | "operational"
            | "authority"
            | "obligations"
            | "supervises"
            | "when"
            | "returns"
            | "where"
            | "permit"
            | "deny"
            | "require_approval"
            | "escalate"
            | "in"
            | "not"
            | "and"
            | "or"
            | "true"
            | "false"
            | "null"
    )
}

/// Create a span from start position to current position.
fn span_from(start: &Position, end: &Position) -> Span {
    Span {
        start: start.offset,
        end: end.offset,
        line: start.line,
        column: start.column,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input(s: &str) -> ParseInput<'_> {
        crate::input::new_input(s)
    }

    #[test]
    fn test_workflow_def() {
        let mut input = test_input("workflow main { done }");
        let result = workflow_def(&mut input).unwrap();
        assert_eq!(result.name.as_ref(), "main");
    }

    #[test]
    fn test_observe_stmt() {
        let mut input = test_input("observe read_db");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Observe { .. }));
    }

    #[test]
    fn test_observe_with_binding() {
        let mut input = test_input("observe read_db as data");
        let result = parse_stmt(&mut input).unwrap();
        match result {
            Workflow::Observe { binding, .. } => {
                assert!(binding.is_some());
            }
            _ => panic!("Expected Observe"),
        }
    }

    #[test]
    fn test_let_stmt() {
        let mut input = test_input("let x = 42");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Let { .. }));
    }

    #[test]
    fn test_if_stmt() {
        let mut input = test_input("if true then done");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::If { .. }));
    }

    #[test]
    fn test_if_else_stmt() {
        let mut input = test_input("if x > 0 then done else done");
        let result = parse_stmt(&mut input).unwrap();
        match result {
            Workflow::If { else_branch, .. } => {
                assert!(else_branch.is_some());
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_act_stmt() {
        let mut input = test_input("act log_event(\"test\")");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Act { .. }));
    }

    #[test]
    fn test_done_stmt() {
        let mut input = test_input("done");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Done { .. }));
    }

    #[test]
    fn test_for_stmt() {
        let mut input = test_input("for item in items do done");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::For { .. }));
    }

    #[test]
    fn test_with_stmt() {
        let mut input = test_input("with db do done");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::With { .. }));
    }

    #[test]
    fn test_maybe_stmt() {
        let mut input = test_input("maybe done else done");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Maybe { .. }));
    }

    #[test]
    fn test_must_stmt() {
        let mut input = test_input("must done");
        let result = parse_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Must { .. }));
    }

    #[test]
    fn test_seq_workflow() {
        let mut input = test_input("let x = 1; let y = 2; done");
        let result = workflow(&mut input).unwrap();
        assert!(matches!(result, Workflow::Seq { .. }));
    }

    #[test]
    fn test_pattern_variable() {
        let mut input = test_input("my_var");
        let result = pattern(&mut input).unwrap();
        assert!(matches!(result, Pattern::Variable(name) if name.as_ref() == "my_var"));
    }

    #[test]
    fn test_pattern_wildcard() {
        let mut input = test_input("_");
        let result = pattern(&mut input).unwrap();
        assert!(matches!(result, Pattern::Wildcard));
    }

    #[test]
    fn test_pattern_tuple() {
        let mut input = test_input("(a, b, c)");
        let result = pattern(&mut input).unwrap();
        assert!(matches!(result, Pattern::Tuple(pats) if pats.len() == 3));
    }

    #[test]
    fn test_action_ref() {
        let mut input = test_input("send_email(\"to\", \"subject\")");
        let result = action_ref(&mut input).unwrap();
        assert_eq!(result.name.as_ref(), "send_email");
        assert_eq!(result.args.len(), 2);
    }

    #[test]
    fn test_check_stmt_with_obligation() {
        let mut input = test_input("check admin.is_active");
        let result = check_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Check { .. }));
        match result {
            Workflow::Check { target, .. } => {
                assert!(matches!(target, CheckTarget::Obligation(_)));
            }
            _ => panic!("Expected Check workflow"),
        }
    }

    #[test]
    fn test_check_stmt_with_policy_instance() {
        let mut input = test_input("check RateLimit { requests: 100, window_secs: 60 }");
        let result = check_stmt(&mut input).unwrap();
        assert!(matches!(result, Workflow::Check { .. }));
        match result {
            Workflow::Check { target, .. } => {
                assert!(matches!(target, CheckTarget::Policy(_)));
            }
            _ => panic!("Expected Check workflow with policy target"),
        }
    }

    #[test]
    fn test_policy_instance_parsing() {
        let mut input = test_input("RateLimit { requests: 100, window_secs: 60 }");
        let result = policy_instance(&mut input).unwrap();
        assert_eq!(result.name.as_ref(), "RateLimit");
        assert_eq!(result.fields.len(), 2);
        assert_eq!(result.fields[0].0.as_ref(), "requests");
        assert_eq!(result.fields[1].0.as_ref(), "window_secs");
    }

    #[test]
    fn test_policy_instance_single_field() {
        let mut input = test_input("MaxLatency { milliseconds: 500 }");
        let result = policy_instance(&mut input).unwrap();
        assert_eq!(result.name.as_ref(), "MaxLatency");
        assert_eq!(result.fields.len(), 1);
        assert_eq!(result.fields[0].0.as_ref(), "milliseconds");
    }

    #[test]
    fn test_policy_instance_with_string_value() {
        let mut input = test_input("DataResidency { allowed_regions: [\"us-east-1\"] }");
        let result = policy_instance(&mut input).unwrap();
        assert_eq!(result.name.as_ref(), "DataResidency");
        assert_eq!(result.fields.len(), 1);
    }
}
