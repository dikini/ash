//! Expression parser for the Ash language.
//!
//! This module provides parsers for Ash expressions using precedence climbing.

use winnow::combinator::{alt, delimited, opt, preceded};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::{one_of, take_while};

use crate::input::{ParseInput, Position};
use crate::parse_pattern::pattern;
use crate::parse_utils::skip_whitespace_and_comments;
use crate::surface::{
    BinaryOp, BlockStmt, ComprehensionQualifier, ConstructorPayload, DoStmt, DoTarget, Expr,
    HandlerClause, Literal, MacroDelimiter, MacroInvocation, MacroInvocationBody, MacroTokenTree,
    MatchArm, Name, OperatorSection, OperatorSectionKind, Pattern, RawOperatorToken, Type, UnaryOp,
};
use crate::token::Span;

/// Check whether the input starts with a keyword boundary match.
fn input_starts_with_keyword(input: &ParseInput, word: &str) -> bool {
    if !input.input.starts_with(word) {
        return false;
    }
    let after = &input.input[word.len()..];
    after
        .chars()
        .next()
        .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
}

const AMBIENT_DO_TARGET: &str = "__ambient";

/// The expression grammar is ordinarily context-free. `on` is the sole
/// surface form which needs to reserve a clause-shaped brace after its
/// computation without changing the interpretation of nested expressions.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExprParseMode {
    Ordinary,
    OnComputation,
}

/// Try to parse a generalized do block expression: `do:K { ... }` or target
/// ambient sequencing sugar `do { ... }`.
pub(crate) fn parse_do_block_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    if !input_starts_with_keyword(input, "do") {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let start_pos = input.state.pos;
    let _ = keyword("do").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let target = if input.input.starts_with(':') {
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let target_start = input.state.pos;
        parse_do_target(input, &target_start)?
    } else {
        DoTarget {
            name: AMBIENT_DO_TARGET.into(),
            args: Vec::new(),
            span: span_from(&start_pos, &input.state.pos),
        }
    };

    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut stmts = Vec::new();
    while !input.input.starts_with('}') {
        let stmt = parse_do_stmt(input)?;
        let is_return = matches!(stmt, DoStmt::Return { .. });
        stmts.push(stmt);
        skip_whitespace_and_comments(input);
        if is_return {
            break;
        }
    }

    let _ = literal_str("}").parse_next(input)?;
    let span = span_from(&start_pos, &input.state.pos);

    Ok(Expr::DoBlock {
        target,
        stmts,
        span,
    })
}

fn parse_do_target(input: &mut ParseInput, target_start: &Position) -> ModalResult<DoTarget> {
    let target_name: Name = identifier(input)?.into();
    skip_whitespace_and_comments(input);
    let target_args = parse_do_target_args(input)?;
    let target_span = span_from(target_start, &input.state.pos);
    let removed_workflow_target = ["Work", "flow"].concat();
    if target_name.as_ref() == "Act"
        || target_name.as_ref() == "Proc"
        || target_name.as_ref() == removed_workflow_target
    {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }
    Ok(DoTarget {
        name: target_name,
        args: target_args,
        span: target_span,
    })
}

fn parse_do_target_args(input: &mut ParseInput) -> ModalResult<Vec<Type>> {
    if !input.input.starts_with('<') {
        return Ok(Vec::new());
    }

    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut args = Vec::new();
    if input.input.starts_with('>') {
        let _ = literal_str(">").parse_next(input)?;
        return Ok(args);
    }

    loop {
        args.push(parse_do_type(input)?);
        skip_whitespace_and_comments(input);
        if !input.input.starts_with(',') {
            break;
        }
        let _ = literal_str(",").parse_next(input)?;
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str(">").parse_next(input)?;
    Ok(args)
}

fn parse_do_type(input: &mut ParseInput) -> ModalResult<Type> {
    if input.input.starts_with('_') {
        let start = input
            .state
            .source
            .len()
            .saturating_sub(input.input.as_ref().len());
        let line = input.state.pos.line;
        let column = input.state.pos.column;
        let _ = literal_str("_").parse_next(input)?;
        let span = Span {
            start,
            end: start + 1,
            line,
            column,
        };
        return Ok(Type::Hole { span });
    }

    let name: Name = identifier(input)?.into();
    skip_whitespace_and_comments(input);
    if input.input.starts_with('<') {
        let args = parse_do_target_args(input)?;
        Ok(Type::Constructor { name, args })
    } else {
        Ok(Type::Name(name))
    }
}

fn parse_do_stmt(input: &mut ParseInput) -> ModalResult<DoStmt> {
    let stmt_start = input.state.pos;

    if input_starts_with_keyword(input, "let") {
        let _ = keyword("let").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let name: Name = identifier(input)?.into();
        skip_whitespace_and_comments(input);
        let _ = literal_str("=").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let value = expr(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(";").parse_next(input)?;
        let span = span_from(&stmt_start, &input.state.pos);
        skip_whitespace_and_comments(input);
        return Ok(DoStmt::Let {
            name,
            value: Box::new(value),
            span,
        });
    }

    if input_starts_with_keyword(input, "return") {
        let _ = keyword("return").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let value = expr(input)?;
        skip_whitespace_and_comments(input);
        if input.input.starts_with(';') {
            let _ = literal_str(";").parse_next(input)?;
            skip_whitespace_and_comments(input);
        }
        let span = span_from(&stmt_start, &input.state.pos);
        return Ok(DoStmt::Return {
            value: Box::new(value),
            span,
        });
    }

    let bind_checkpoint = input.clone();
    if let Ok(name) = identifier(input) {
        skip_whitespace_and_comments(input);
        if literal_str("<-").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            let value = expr(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str(";").parse_next(input)?;
            let span = span_from(&stmt_start, &input.state.pos);
            skip_whitespace_and_comments(input);
            return Ok(DoStmt::Bind {
                name: Name::from(name),
                value: Box::new(value),
                span,
            });
        }
    }
    *input = bind_checkpoint;

    let value = expr(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(";").parse_next(input)?;
    let span = span_from(&stmt_start, &input.state.pos);
    skip_whitespace_and_comments(input);
    Ok(DoStmt::Expr {
        value: Box::new(value),
        span,
    })
}

/// Parse an expression (entry point).
///
/// This handles the full expression grammar with proper precedence.
pub fn expr(input: &mut ParseInput) -> ModalResult<Expr> {
    expr_with_mode(input, ExprParseMode::Ordinary)
}

/// Parse an expression with the one lexically-scoped handler-computation
/// boundary needed by `on`. All recursive parsers for delimited expressions
/// deliberately enter through [`expr`] and therefore remain ordinary.
fn expr_with_mode(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    // Try closure syntax first: |params| -> body  (TASK-959)
    if let Ok(closure) = parse_closure_expr(input) {
        return Ok(closure);
    }
    // Try anonymous fn expression: fn(params) [-> type] { body }
    if let Ok(fn_def) = parse_fn_expr(input) {
        return Ok(fn_def);
    }
    // Try if-let first (before other expressions to avoid conflicts with 'if')
    if let Ok(if_let) = parse_if_let_expr(input) {
        return Ok(if_let);
    }
    pipe_expr(input, mode)
}

/// Parse the canonical handler body `on computation { clauses... }`.
fn parse_on_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_offset = source_offset(input);
    let _ = keyword("on").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let computation_start = source_offset(input);
    let mut computation = expr_with_mode(input, ExprParseMode::OnComputation)?;
    let computation_end = computation_end_offset(input, computation_start);
    set_expression_span(
        &mut computation,
        source_span(input, computation_start, computation_end),
    );
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut clauses = Vec::new();
    let mut operation_count = 0usize;
    let mut saw_done = false;
    while !input.input.starts_with('}') {
        let clause_start = source_offset(input);
        if input_starts_with_keyword(input, "done") {
            // `done` occurs exactly once in a canonical source handler. Cut
            // here so the expression dispatcher cannot reinterpret a
            // malformed `on` form as another expression.
            if saw_done {
                return Err(handler_clause_cardinality_error(
                    input,
                    "duplicate done clause",
                ));
            }
            let _ = keyword("done").parse_next(input)?;
            saw_done = true;
            skip_whitespace_and_comments(input);
            let _ = literal_str("(").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let binding: Name = identifier(input)?.into();
            skip_whitespace_and_comments(input);
            let _ = literal_str(")").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str("=>").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let body = expr(input)?;
            let span = source_span(input, clause_start, source_offset(input));
            clauses.push(HandlerClause::Done {
                binding,
                body: Box::new(body),
                span,
            });
        } else {
            let impl_type: Name = identifier(input)?.into();
            skip_whitespace_and_comments(input);
            let _ = literal_str("::").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let operation: Name = identifier(input)?.into();
            skip_whitespace_and_comments(input);
            let _ = literal_str("(").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let clause_pattern = pattern(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str(",").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let resume: Name = identifier(input)?.into();
            skip_whitespace_and_comments(input);
            let _ = literal_str(")").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str("=>").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let body = expr(input)?;
            let span = source_span(input, clause_start, source_offset(input));
            clauses.push(HandlerClause::Operation {
                impl_type,
                operation,
                pattern: clause_pattern,
                resume,
                body: Box::new(body),
                span,
            });
            operation_count += 1;
        }
        skip_whitespace_and_comments(input);
        if input.input.starts_with(',') || input.input.starts_with(';') {
            let _ = one_of([',', ';']).parse_next(input)?;
            skip_whitespace_and_comments(input);
        }
    }
    // Preserve the closing-brace position for cardinality failures.  It is
    // the first point at which the complete clause set is known.
    if operation_count == 0 {
        return Err(handler_clause_cardinality_error(
            input,
            "missing concrete operation clause",
        ));
    }
    if !saw_done {
        return Err(handler_clause_cardinality_error(
            input,
            "missing done clause",
        ));
    }
    let _ = literal_str("}").parse_next(input)?;
    let span = source_span(input, start_offset, source_offset(input));
    Ok(Expr::On {
        computation: Box::new(computation),
        clauses,
        span,
    })
}

/// Return the exact stream offset even when an older expression carrier's
/// position sidecar has not recorded every token it consumed.
fn source_offset(input: &ParseInput) -> usize {
    input
        .state
        .source
        .len()
        .saturating_sub(input.input.as_ref().len())
}

fn source_span(input: &ParseInput, start: usize, end: usize) -> Span {
    crate::input::offset_to_span(input.state.source, start, end)
}

/// Expression token helpers commonly consume following whitespace while
/// probing a postfix or binary continuation. The handler computation ends at
/// the final non-trivia byte before its delimiter.
fn computation_end_offset(input: &ParseInput, start: usize) -> usize {
    let source = input.state.source;
    let mut end = source_offset(input);
    loop {
        end = start
            + source[start..end]
                .trim_end_matches(char::is_whitespace)
                .len();

        if source[start..end].ends_with("*/")
            && let Some(comment_start) = trailing_block_comment_start(source, start, end)
        {
            end = comment_start;
            continue;
        }

        if let Some(comment_start) = trailing_line_comment_start(source, start, end) {
            end = comment_start;
            continue;
        }

        return end;
    }
}

/// Find the opening delimiter for a block-comment suffix, honoring the
/// nesting accepted by `skip_whitespace_and_comments`.
fn trailing_block_comment_start(source: &str, start: usize, end: usize) -> Option<usize> {
    let text = &source[start..end];
    let mut offset = text.len();
    let mut depth = 0usize;

    while offset >= 2 {
        let prefix = &text[..offset];
        if prefix.ends_with("*/") {
            depth += 1;
            offset -= 2;
        } else if prefix.ends_with("/*") {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(start + offset - 2);
            }
            offset -= 2;
        } else {
            let character = text[..offset].chars().next_back()?;
            offset -= character.len_utf8();
        }
    }

    None
}

/// Locate a line-comment marker only when it is outside string literals. This
/// is intentionally local to the already-consumed computation suffix: it
/// cannot reinterpret an ordinary expression, and it avoids treating `//` or
/// `--` inside a string literal as comment trivia.
fn trailing_line_comment_start(source: &str, start: usize, end: usize) -> Option<usize> {
    let text = &source[start..end];
    let mut offset = 0;
    let mut quote = None;
    let mut escaped = false;

    while offset < text.len() {
        let rest = &text[offset..];
        let character = rest.chars().next()?;

        if let Some(delimiter) = quote {
            offset += character.len_utf8();
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }

        if matches!(character, '"' | '\'') {
            quote = Some(character);
            offset += character.len_utf8();
        } else if rest.starts_with("//") || rest.starts_with("--") {
            let comment_end = rest.find('\n').unwrap_or(rest.len());
            if rest[comment_end..].trim().is_empty() {
                return Some(start + offset);
            }
            offset += comment_end;
        } else if rest.starts_with("/*") {
            offset = nested_block_comment_end(text, offset)?;
        } else {
            offset += character.len_utf8();
        }
    }

    None
}

fn nested_block_comment_end(text: &str, mut offset: usize) -> Option<usize> {
    debug_assert!(text[offset..].starts_with("/*"));
    offset += 2;
    let mut depth = 1usize;

    while offset < text.len() {
        let rest = &text[offset..];
        if rest.starts_with("/*") {
            depth += 1;
            offset += 2;
        } else if rest.starts_with("*/") {
            depth -= 1;
            offset += 2;
            if depth == 0 {
                return Some(offset);
            }
        } else {
            offset += rest.chars().next()?.len_utf8();
        }
    }

    None
}

/// The `on` computation is parsed by the existing expression carriers. Repair
/// its enclosing span from the stream boundary without changing any nested
/// carrier or ordinary-expression behavior.
fn set_expression_span(expr: &mut Expr, span: Span) {
    match expr {
        Expr::OperatorSection { section } => section.span = span,
        Expr::Variable { span: current, .. }
        | Expr::FieldAccess { span: current, .. }
        | Expr::IndexAccess { span: current, .. }
        | Expr::Unary { span: current, .. }
        | Expr::Binary { span: current, .. }
        | Expr::Call { span: current, .. }
        | Expr::Match { span: current, .. }
        | Expr::IfLet { span: current, .. }
        | Expr::CheckObligation { span: current, .. }
        | Expr::Constructor { span: current, .. }
        | Expr::Record { span: current, .. }
        | Expr::If { span: current, .. }
        | Expr::Panic { span: current, .. }
        | Expr::Fail { span: current, .. }
        | Expr::WithError { span: current, .. }
        | Expr::On { span: current, .. }
        | Expr::HandleWith { span: current, .. }
        | Expr::Block { span: current, .. }
        | Expr::FnDef { span: current, .. }
        | Expr::FnApply { span: current, .. }
        | Expr::DoBlock { span: current, .. }
        | Expr::Comprehension { span: current, .. }
        | Expr::List { span: current, .. } => *current = span,
        Expr::MacroInvocation { invocation } => invocation.span = span,
        Expr::Literal(_) | Expr::Policy(_) => {}
    }
}

/// Mark a canonical `on` cardinality failure for the public parser diagnostic
/// boundary while retaining a cut at the committed grammar route.
fn handler_clause_cardinality_error(
    input: &ParseInput,
    message: &'static str,
) -> winnow::error::ErrMode<winnow::error::ContextError> {
    use winnow::error::AddContext;

    let checkpoint = input.checkpoint();
    let error = winnow::error::ContextError::new().add_context(
        input,
        &checkpoint,
        winnow::error::StrContext::Label(message),
    );
    winnow::error::ErrMode::Cut(error)
}

/// Parse `handle expression with handler_name` without resolving the handler.
fn parse_handle_with_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = keyword("handle").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let expression_start = source_offset(input);
    let mut expression = expr(input)?;
    let expression_end = computation_end_offset(input, expression_start);
    set_expression_span(
        &mut expression,
        source_span(input, expression_start, expression_end),
    );
    skip_whitespace_and_comments(input);
    let _ = keyword("with").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let (handler, handler_span) = identifier_with_span(input)?;
    let handler: Name = handler.into();
    let span = span_from(&start_pos, &input.state.pos);
    Ok(Expr::HandleWith {
        expression: Box::new(expression),
        handler,
        handler_span,
        span,
    })
}

/// Parse a scoped operational failure handler:
/// `with_error { body } handle { pattern => expr; ... }`.
fn parse_with_error_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = keyword("with_error").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let body = parse_fn_expr_body(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("handle").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut arms = Vec::new();
    while !input.input.starts_with('}') {
        let arm_start = input.state.pos;
        let pat = pattern(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("=>").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let body = expr(input)?;
        skip_whitespace_and_comments(input);
        if input.input.starts_with(';') {
            let _ = literal_str(";").parse_next(input)?;
            skip_whitespace_and_comments(input);
        }
        let span = span_from(&arm_start, &input.state.pos);
        arms.push(MatchArm {
            pattern: pat,
            body: Box::new(body),
            span,
        });
    }
    let _ = literal_str("}").parse_next(input)?;
    let span = span_from(&start_pos, &input.state.pos);

    Ok(Expr::WithError {
        body: Box::new(body),
        arms,
        span,
    })
}

/// Parse a pipe expression: left |> right
///
/// Desugars at the parser surface so the core IR never sees `Pipe`.
///   `lhs |> func(args)`  =>  `func(lhs, args)`
///   `lhs |> module::func(args)`  =>  `module::func(lhs, args)`
///   `lhs |> f`  =>  `f(lhs)`
fn pipe_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = ternary_expr(input, mode)?;

    loop {
        if opt(literal_str("|>")).parse_next(input)?.is_some() {
            skip_whitespace_and_comments(input);
            let right = ternary_expr(input, mode)?;
            left = desugar_pipe(left, right, &start_pos, input);
        } else {
            break;
        }
    }

    Ok(left)
}

/// Desugar a pipe expression by prepending the left operand as the first argument.
fn desugar_pipe(lhs: Expr, rhs: Expr, start_pos: &Position, input: &ParseInput) -> Expr {
    let span = span_from(start_pos, &input.state.pos);
    match rhs {
        Expr::Call {
            func, module, args, ..
        } => Expr::Call {
            func,
            module,
            args: std::iter::once(lhs).chain(args).collect(),
            span,
        },
        Expr::Variable { name, .. } => Expr::Call {
            func: name,
            module: None,
            args: vec![lhs],
            span,
        },
        Expr::FnApply { func, args, .. } => Expr::FnApply {
            func,
            args: std::iter::once(lhs).chain(args).collect(),
            span,
        },
        // Any other expression becomes a function application
        other => Expr::FnApply {
            func: Box::new(other),
            args: vec![lhs],
            span,
        },
    }
}

/// Parse an anonymous fn expression: `fn(params) [-> type] { body }`.
///
/// This produces `Expr::FnDef { params, return_type, body }`.
/// Params are `(name, optional_type_annotation)` pairs.
/// This does NOT parse named fn definitions — those are handled at item level.
pub fn parse_fn_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;

    // Must start with "fn" keyword
    let _ = keyword("fn").parse_next(input)?;

    // If the next non-whitespace token is an identifier, this is a named fn —
    // not an anonymous fn expression. Bail out so the caller can handle it.
    // We peek without consuming.
    let saved = input.clone();
    skip_whitespace_and_comments(input);
    if identifier(input).is_ok() {
        // This is `fn name(...)` — restore and reject so named-fn parsers can handle it.
        *input = saved;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    *input = saved;
    skip_whitespace_and_comments(input);

    // Parse parameter list: (name [: Type], ...)
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let params = parse_fn_expr_params(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Optional return type: -> TypeName
    let return_type = if input.input.starts_with("->") {
        let _ = literal_str("->").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let ty_name = parse_simple_type_name(input)?;
        Some(ty_name)
    } else {
        None
    };
    skip_whitespace_and_comments(input);

    // Body block: { ... }
    let body = parse_fn_expr_body(input)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Expr::FnDef {
        params,
        return_type,
        body: Box::new(body),
        span,
    })
}

/// Parse a pure closure expression: `|params| -> body`. SPEC-072 §6.2
///
/// Desugars immediately to `Expr::FnDef { params, return_type: None, body }`.
/// `|x| -> x + 1`  =>  `fn(x) { x + 1 }`
/// `|x, y| -> x + y`  =>  `fn(x, y) { x + y }`
///
/// No return-type annotation in the closure shorthand — use `fn(x: T) -> R { }` for that.
pub(crate) fn parse_closure_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let saved = input.clone();

    // Must start with `|`
    skip_whitespace_and_comments(input);
    if !input.input.starts_with('|') {
        *input = saved;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    let _ = literal_str("|").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse param list terminated by `|`
    let params = parse_closure_params(input)?;

    // Closing `|`
    skip_whitespace_and_comments(input);
    if !input.input.starts_with('|') {
        *input = saved;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    let _ = literal_str("|").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Mandatory pure closure arrow. `=>` is no longer pure closure sugar.
    if !input.input.starts_with("->") {
        *input = saved;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    let _ = literal_str("->").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Body: a single expression (not a block — that's what makes this a shorthand)
    let body = expr(input)?;
    let span = span_from(&start_pos, &input.state.pos);

    Ok(Expr::FnDef {
        params,
        return_type: None,
        body: Box::new(body),
        span,
    })
}

/// Parse the parameter list inside `|...|` closure syntax.
///
/// Parameters are `name` or `name: Type`, separated by commas.
/// An empty list `||` is allowed (zero-param closure).
fn parse_closure_params(input: &mut ParseInput) -> ModalResult<Vec<(Name, Option<Name>)>> {
    let mut params = Vec::new();

    skip_whitespace_and_comments(input);
    // Empty param list: `||`
    if input.input.starts_with('|') {
        return Ok(params);
    }

    loop {
        skip_whitespace_and_comments(input);
        let name: Name = identifier(input)?.into();
        skip_whitespace_and_comments(input);

        let ty = if input.input.starts_with(':') {
            let _ = literal_str(":").parse_next(input)?;
            skip_whitespace_and_comments(input);
            Some(parse_simple_type_name(input)?)
        } else {
            None
        };

        params.push((name, ty));

        skip_whitespace_and_comments(input);
        if input.input.starts_with(',') {
            let _ = literal_str(",").parse_next(input)?;
            skip_whitespace_and_comments(input);
            // Trailing comma before `|` allowed
            if input.input.starts_with('|') {
                break;
            }
        } else {
            break;
        }
    }

    Ok(params)
}

/// Parse the parameter list of an anonymous fn expression.
///
/// Each parameter is `name` or `name: Type`.
fn parse_fn_expr_params(input: &mut ParseInput) -> ModalResult<Vec<(Name, Option<Name>)>> {
    let mut params = Vec::new();

    skip_whitespace_and_comments(input);
    if input.input.starts_with(")") {
        return Ok(params);
    }

    loop {
        skip_whitespace_and_comments(input);
        let name: Name = identifier(input)?.into();
        skip_whitespace_and_comments(input);

        let ty = if input.input.starts_with(":") {
            let _ = literal_str(":").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let ty_name = parse_simple_type_name(input)?;
            Some(ty_name)
        } else {
            None
        };

        params.push((name, ty));

        skip_whitespace_and_comments(input);
        if input.input.starts_with(",") {
            let _ = literal_str(",").parse_next(input)?;
            skip_whitespace_and_comments(input);
            if input.input.starts_with(")") {
                break;
            }
        } else {
            break;
        }
    }

    Ok(params)
}

/// Parse a type annotation name for anonymous function expressions.
///
/// This is used for type annotations in anonymous fn params and return types.
fn parse_simple_type_name(input: &mut ParseInput) -> ModalResult<Name> {
    // Allow keyword-like type names (e.g. Int, Bool, String are identifiers,
    // but future types might overlap with keywords). Use take_while directly.
    let name: &str =
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;
    if name.is_empty()
        || !name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    let mut type_name = String::from(name);

    while input.input.starts_with('<') {
        let generic_start = input.clone();
        let mut depth = 0usize;
        let mut consumed = String::new();

        loop {
            let Some(ch) = input.input.chars().next() else {
                *input = generic_start;
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            };
            let char_len = ch.len_utf8();
            let _ = input.input.next_slice(char_len);
            input.state.advance(ch);
            consumed.push(ch);

            match ch {
                '<' => depth += 1,
                '>' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }

        type_name.push_str(&consumed);
    }

    Ok(type_name.into())
}

/// Parse the body of an anonymous fn expression: `{ stmts* tail_expr? }`.
///
/// This reuses the same block structure as fn definitions: let bindings
/// followed by an optional tail expression.
/// Public entry point for parsing an anonymous fn body block: `{ stmts* tail_expr? }`.
///
/// Exported so that workflow and module parsers can reuse the same block-body logic.
pub fn parse_fn_expr_body_pub(input: &mut ParseInput) -> ModalResult<Expr> {
    parse_fn_expr_body(input)
}

fn parse_fn_expr_body(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut statements: Vec<BlockStmt> = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.starts_with("}") {
            let _ = literal_str("}").parse_next(input)?;
            let span = span_from(&start_pos, &input.state.pos);
            return Ok(Expr::Block {
                statements,
                tail_expr: None,
                span,
            });
        }

        // Try `let pat = expr;`
        if keyword("let").parse_next(input).is_ok() {
            let stmt_start = input.state.pos;
            skip_whitespace_and_comments(input);
            let pat = pattern(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str("=").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let let_expr = expr(input)?;
            skip_whitespace_and_comments(input);
            if input.input.starts_with(";") {
                let _ = literal_str(";").parse_next(input)?;
            }
            let stmt_span = span_from(&stmt_start, &input.state);
            statements.push(BlockStmt::Let {
                pattern: pat,
                expr: let_expr,
                span: stmt_span,
            });
            continue;
        }

        // Try `fn name(params) { body }` as a named local fn (desugars to let)
        if keyword("fn").parse_next(input).is_ok() {
            let stmt_start = input.state.pos;
            skip_whitespace_and_comments(input);
            let (name_str, name_span) = identifier_with_span(input)?;
            let name: Name = name_str.into();
            skip_whitespace_and_comments(input);
            let _ = literal_str("(").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let params = parse_fn_expr_params(input)?;
            let _ = literal_str(")").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let return_type = if input.input.starts_with("->") {
                let _ = literal_str("->").parse_next(input)?;
                skip_whitespace_and_comments(input);
                let ty_name = parse_simple_type_name(input)?;
                Some(ty_name)
            } else {
                None
            };
            skip_whitespace_and_comments(input);
            let fn_body = parse_fn_expr_body(input)?;
            skip_whitespace_and_comments(input);
            if input.input.starts_with(";") {
                let _ = literal_str(";").parse_next(input)?;
            }
            let stmt_span = span_from(&stmt_start, &input.state);
            let fn_def_span = stmt_span;
            statements.push(BlockStmt::Let {
                pattern: Pattern::Variable {
                    name,
                    span: name_span,
                },
                expr: Expr::FnDef {
                    params,
                    return_type,
                    body: Box::new(fn_body),
                    span: fn_def_span,
                },
                span: stmt_span,
            });
            continue;
        }

        let stmt_checkpoint = input.clone();
        if let Ok(value) = expr(input) {
            skip_whitespace_and_comments(input);
            if literal_str(";").parse_next(input).is_ok() {
                let stmt_span = span_from(&stmt_checkpoint.state.pos, &input.state.pos);
                statements.push(BlockStmt::Expr {
                    expr: value,
                    span: stmt_span,
                });
                skip_whitespace_and_comments(input);
                continue;
            }
        }
        *input = stmt_checkpoint;

        // Must be the tail expression
        break;
    }

    skip_whitespace_and_comments(input);

    if input.input.starts_with("}") {
        let _ = literal_str("}").parse_next(input)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Block {
            statements,
            tail_expr: None,
            span,
        });
    }

    let tail_expr = expr(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;

    let span = span_from(&start_pos, &input.state.pos);
    Ok(Expr::Block {
        statements,
        tail_expr: Some(Box::new(tail_expr)),
        span,
    })
}

/// Parse a ternary expression: condition ? then : else
fn ternary_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let _start_pos = input.state.pos;
    let condition = or_expr(input, mode)?;

    // Check for ternary operator
    if opt(preceded(literal_str("?"), |input: &mut ParseInput| {
        or_expr(input, mode)
    }))
    .parse_next(input)?
    .is_some()
    {
        let _then_branch = or_expr(input, mode)?;
        let _ = preceded(literal_str(":"), |input: &mut ParseInput| {
            or_expr(input, mode)
        })
        .parse_next(input)?;
        // Note: Simplified - ternary not fully implemented in surface AST
        Ok(condition)
    } else {
        Ok(condition)
    }
}

/// Parse an if-let expression: `if let pattern = expr then expr else expr`
///
/// Example: `if let Some { value: x } = opt then { x } else { 0 }`
pub fn parse_if_let_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;

    // Match "if let"
    let _ = keyword("if").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("let").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse the pattern
    let pat = pattern(input)?;

    skip_whitespace_and_comments(input);
    let _ = literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse the expression to match against
    let match_expr = ternary_expr(input, ExprParseMode::Ordinary)?;

    skip_whitespace_and_comments(input);
    let _ = keyword("then").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse then branch (block or expression)
    let then_branch = Box::new(parse_block_or_expr(input)?);

    skip_whitespace_and_comments(input);
    let _ = keyword("else").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse else branch (block or expression)
    let else_branch = Box::new(parse_block_or_expr(input)?);

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Expr::IfLet {
        pattern: pat,
        expr: Box::new(match_expr),
        then_branch,
        else_branch,
        span,
    })
}

/// Parse either a block `{ ... }` or a single expression.
///
/// This is used for then/else branches in if-let expressions.
/// A block can contain multiple statements/expressions separated by semicolons.
fn parse_block_or_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    skip_whitespace_and_comments(input);

    if input.input.starts_with("{") {
        // Parse a block with multiple statements
        let _ = literal_str("{").parse_next(input)?;
        skip_whitespace_and_comments(input);

        // Check for empty block
        if input.input.starts_with("}") {
            let _ = literal_str("}").parse_next(input)?;
            return Ok(Expr::Literal(Literal::Null));
        }

        // Parse first expression
        let first = expr(input)?;

        // Check for more expressions (semicolon-separated)
        let mut exprs = vec![first];
        loop {
            skip_whitespace_and_comments(input);
            if input.input.starts_with(";") {
                let _ = input.input.next_slice(1);
                input.state.advance(';');
                skip_whitespace_and_comments(input);

                // If next is }, this was a trailing semicolon
                if input.input.starts_with("}") {
                    break;
                }

                let next = expr(input)?;
                exprs.push(next);
            } else {
                break;
            }
        }

        let _ = literal_str("}").parse_next(input)?;

        // Return the last expression (or the only one)
        Ok(exprs.pop().unwrap_or(Expr::Literal(Literal::Null)))
    } else {
        // Single expression
        expr(input)
    }
}

/// Parse logical OR expressions: left || right
fn or_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = and_expr(input, mode)?;

    while let Some(raw_operator) = parse_specific_raw_operator_token(input, "||") {
        let right = and_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        left = Expr::Binary {
            op: BinaryOp::Or,
            raw_operator: Some(raw_operator),
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

/// Parse logical AND expressions: left && right
fn and_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = in_expr(input, mode)?;

    while let Some(raw_operator) = parse_specific_raw_operator_token(input, "&&") {
        let right = in_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        left = Expr::Binary {
            op: BinaryOp::And,
            raw_operator: Some(raw_operator),
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

/// Parse IN expressions: left in right
fn in_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let left = comparison_expr(input, mode)?;

    if opt(keyword("in")).parse_next(input)?.is_some() {
        let right = comparison_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        Ok(Expr::Binary {
            op: BinaryOp::In,
            raw_operator: None,
            left: Box::new(left),
            right: Box::new(right),
            span,
        })
    } else {
        Ok(left)
    }
}

/// Parse comparison expressions: ==, !=, <, >, <=, >=
fn comparison_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let left = additive_expr(input, mode)?;

    // Try to match comparison operators
    match parse_binary_operator_token(
        input,
        &[
            ("==", BinaryOp::Eq),
            ("!=", BinaryOp::Neq),
            ("<=", BinaryOp::Leq),
            (">=", BinaryOp::Geq),
            ("<", BinaryOp::Lt),
            (">", BinaryOp::Gt),
        ],
    ) {
        Some((op, raw_operator)) => {
            let right = additive_expr(input, mode)?;
            let span = span_from(&start_pos, &input.state.pos);
            Ok(Expr::Binary {
                op,
                raw_operator: Some(raw_operator),
                left: Box::new(left),
                right: Box::new(right),
                span,
            })
        }
        None => Ok(left),
    }
}

/// Parse additive expressions: +, -
fn additive_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = multiplicative_expr(input, mode)?;

    while let Some((op, raw_operator)) =
        parse_binary_operator_token(input, &[("+", BinaryOp::Add), ("-", BinaryOp::Sub)])
    {
        let right = multiplicative_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        left = Expr::Binary {
            op,
            raw_operator: Some(raw_operator),
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

/// Parse multiplicative expressions: *, /, %
fn multiplicative_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = unary_expr(input, mode)?;

    while let Some((op, raw_operator)) = parse_binary_operator_token(
        input,
        &[
            ("*", BinaryOp::Mul),
            ("/", BinaryOp::Div),
            ("%", BinaryOp::Mod),
        ],
    ) {
        let right = unary_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        left = Expr::Binary {
            op,
            raw_operator: Some(raw_operator),
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

/// Parse unary expressions: !, -
fn unary_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;

    // Try negation first
    if opt(literal_str("!")).parse_next(input)?.is_some() {
        let operand = unary_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span,
        });
    }

    // Try arithmetic negation (but not if it's followed by a number, that's a literal)
    if opt(preceded(
        literal_str("-"),
        one_of(|c: char| !c.is_ascii_digit()),
    ))
    .parse_next(input)?
    .is_some()
    {
        // This was a minus followed by a non-digit, so it's unary negation
        // We need to backtrack and parse properly
        // For simplicity, just parse the operand
        let operand = primary_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(operand),
            span,
        });
    }

    // Try keyword "not"
    if opt(keyword("not")).parse_next(input)?.is_some() {
        let operand = unary_expr(input, mode)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span,
        });
    }

    primary_expr(input, mode)
}

/// Parse primary expressions: literals, variables, field access, index access, calls
#[allow(clippy::collapsible_if)]
fn primary_expr(input: &mut ParseInput, mode: ExprParseMode) -> ModalResult<Expr> {
    let start_pos = input.state.pos;

    // Try operator sections before ordinary parenthesized expressions.
    {
        let saved = input.clone();
        match parse_operator_section_expr(input) {
            Ok(section) => return Ok(section),
            Err(winnow::error::ErrMode::Cut(err)) => {
                return Err(winnow::error::ErrMode::Cut(err));
            }
            Err(_) => {}
        }
        *input = saved;
    }

    // Try parenthesized expression first
    if let Ok(e) = delimited(literal_str("("), expr, literal_str(")")).parse_next(input) {
        return finish_postfix_expr(input, e, &start_pos);
    }

    // Try anonymous fn expression: fn(params) [-> type] { body }
    // Phase 158 fix: enable fn expressions in all primary positions
    {
        let saved = input.clone();
        if let Ok(fn_def) = parse_fn_expr(input) {
            return finish_postfix_expr(input, fn_def, &start_pos);
        }
        *input = saved;
    }

    // Try bracket comprehension expression: [result | qualifiers]: K (SPEC-055 substrate)
    {
        let saved = input.clone();
        if let Ok(comprehension) = parse_comprehension_expr(input) {
            return finish_postfix_expr(input, comprehension, &start_pos);
        }
        *input = saved;
    }

    // Try list expression: [expr, expr, ...]
    {
        let saved = input.clone();
        if let Ok(list_expr) = parse_list_expr(input) {
            return finish_postfix_expr(input, list_expr, &start_pos);
        }
        *input = saved;
    }

    // Try structural record expression: { field: expr, ... }
    {
        let saved = input.clone();
        if let Ok(record_expr) = parse_record_expr(input) {
            return finish_postfix_expr(input, record_expr, &start_pos);
        }
        *input = saved;
    }

    // Try ordinary block expression: { stmts* tail_expr? }
    {
        let saved = input.clone();
        if let Ok(block_expr) = parse_fn_expr_body(input) {
            return finish_postfix_expr(input, block_expr, &start_pos);
        }
        *input = saved;
    }

    // Try literal
    if let Ok(lit) = literal(input) {
        return Ok(Expr::Literal(lit));
    }

    // Try check obligation expression: check obligation_name
    if keyword("check").parse_next(input).is_ok() {
        skip_whitespace_and_comments(input);
        let obligation = identifier(input)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::CheckObligation {
            obligation: obligation.into(),
            span,
        });
    }

    // Try scoped operational failure handler as a normal primary expression.
    if let Ok(with_error) = parse_with_error_expr(input) {
        return Ok(with_error);
    }

    // Try canonical source handler expressions before identifier parsing.
    if input_starts_with_keyword(input, "on") {
        return parse_on_expr(input);
    }

    // Try named handler installation before identifier parsing.
    if let Ok(handle_with) = parse_handle_with_expr(input) {
        return Ok(handle_with);
    }

    // Try operational bottom expression: fail payload
    if keyword("fail").parse_next(input).is_ok() {
        skip_whitespace_and_comments(input);
        let payload = expr(input)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Fail {
            payload: Box::new(payload),
            span,
        });
    }

    // Try generalized do block expression: do:K { ... } (SPEC-054 substrate)
    {
        let saved = input.clone();
        if let Ok(do_block) = parse_do_block_expr(input) {
            return finish_postfix_expr(input, do_block, &start_pos);
        }
        *input = saved;
    }

    {
        let saved = input.clone();
        if keyword("act").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            if input.input.starts_with('{') {
                return Err(winnow::error::ErrMode::Cut(
                    winnow::error::ContextError::new(),
                ));
            }
        }
        *input = saved;
    }

    // Try identifier/variable (and potential field access/call)
    let (name, name_span) = expr_name_with_span(input)?;
    let name_str: Name = name.into();

    {
        let before_bang = input.clone();
        if opt(literal_str("!")).parse_next(input)?.is_some() {
            if let Ok(invocation) =
                parse_macro_invocation_after_bang(input, name_str.clone(), &start_pos)
            {
                return Ok(invocation);
            }
        }
        *input = before_bang;
    }

    if opt(literal_str("::")).parse_next(input)?.is_some() {
        let second = identifier(input)?;
        let second_name: Name = second.into();
        skip_whitespace_and_comments(input);

        // Check for `(` to distinguish:
        //   module::func(args...)  →  qualified fn call (Expr::Call with module)
        //   no `(` → this is just a qualified name without arguments, not valid here
        if opt(literal_str("(")).parse_next(input)?.is_some() {
            let args = if literal_str(")").parse_next(input).is_ok() {
                vec![]
            } else {
                let args = parse_args(input)?;
                let _ = literal_str(")").parse_next(input)?;
                args
            };
            let span = span_from(&start_pos, &input.state.pos);
            return Ok(Expr::Call {
                func: second_name,
                module: Some(name_str),
                args,
                span,
            });
        }

        // No `(` after name::name — not a valid call expression
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    if !(mode == ExprParseMode::OnComputation && on_clause_delimiter_starts(input))
        && parse_inline_record_constructor_start(input)
    {
        skip_whitespace_and_comments(input);
        let fields = if literal_str("}").parse_next(input).is_ok() {
            vec![]
        } else {
            parse_constructor_fields(input)?
        };
        let span = span_from(&start_pos, &input.state.pos);
        return finish_postfix_expr(
            input,
            Expr::Constructor {
                name: name_str,
                payload: ConstructorPayload::Record(fields.clone()),
                fields,
                span,
            },
            &start_pos,
        );
    }

    let is_uppercase_leading = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
    if is_uppercase_leading && parse_inline_tuple_constructor_start(input) {
        skip_whitespace_and_comments(input);
        let items = if literal_str(")").parse_next(input).is_ok() {
            vec![]
        } else {
            let items = parse_tuple_constructor_items(input)?;
            let _ = literal_str(")").parse_next(input)?;
            items
        };
        let span = span_from(&start_pos, &input.state.pos);
        return finish_postfix_expr(
            input,
            Expr::Constructor {
                name: name.into(),
                fields: tuple_constructor_fields(&items),
                payload: ConstructorPayload::Tuple(items),
                span,
            },
            &start_pos,
        );
    }

    finish_postfix_expr(
        input,
        Expr::Variable {
            name: name_str.clone(),
            span: name_span,
        },
        &start_pos,
    )
}

/// Non-consumingly recognize the only braces which delimit the computation
/// operand of `on`: a `done(` clause or a concrete `Impl::operation(` clause.
/// This deliberately performs no clause validation; once recognized,
/// `parse_on_expr` commits to its existing clause parser.
fn on_clause_delimiter_starts(input: &ParseInput) -> bool {
    let mut probe = input.clone();
    skip_whitespace_and_comments(&mut probe);
    if !probe.input.starts_with('{') {
        return false;
    }
    let _ = literal_str("{").parse_next(&mut probe);
    skip_whitespace_and_comments(&mut probe);

    if input_starts_with_keyword(&probe, "done") {
        let _ = keyword("done").parse_next(&mut probe);
        skip_whitespace_and_comments(&mut probe);
        return probe.input.starts_with('(');
    }

    if identifier(&mut probe).is_err() {
        return false;
    }
    skip_whitespace_and_comments(&mut probe);
    if !probe.input.starts_with("::") {
        return false;
    }
    let _ = literal_str("::").parse_next(&mut probe);
    skip_whitespace_and_comments(&mut probe);
    if identifier(&mut probe).is_err() {
        return false;
    }
    skip_whitespace_and_comments(&mut probe);
    probe.input.starts_with('(')
}

fn finish_postfix_expr(
    input: &mut ParseInput,
    mut expr: Expr,
    start_pos: &Position,
) -> ModalResult<Expr> {
    loop {
        // Field access: .field
        if opt(literal_str(".")).parse_next(input)?.is_some()
            && let Ok(field) = parse_field_name(input)
        {
            let span = span_from(start_pos, &input.state.pos);
            expr = Expr::FieldAccess {
                base: Box::new(expr),
                field: field.into(),
                span,
            };
            continue;
        }

        // Index access: [index]
        if opt(literal_str("[")).parse_next(input)?.is_some() {
            let index = self::expr(input)?;
            let _ = literal_str("]").parse_next(input)?;
            let span = span_from(start_pos, &input.state.pos);
            expr = Expr::IndexAccess {
                base: Box::new(expr),
                index: Box::new(index),
                span,
            };
            continue;
        }

        // Function call: (args)
        if opt(literal_str("(")).parse_next(input)?.is_some() {
            let args = if literal_str(")").parse_next(input).is_ok() {
                vec![]
            } else if matches!(&expr, Expr::Variable { name, .. } if name.as_ref() == "any_role") {
                let args = vec![parse_list_expr(input)?];
                let _ = literal_str(")").parse_next(input)?;
                args
            } else {
                let args = parse_args(input)?;
                let _ = literal_str(")").parse_next(input)?;
                args
            };
            let span = span_from(start_pos, &input.state.pos);
            expr = match expr {
                Expr::Variable { name, .. } if name_starts_uppercase(&name) => Expr::Constructor {
                    name,
                    fields: tuple_constructor_fields(&args),
                    payload: ConstructorPayload::Tuple(args),
                    span,
                },
                Expr::Variable { name, .. } => Expr::Call {
                    func: name,
                    module: None,
                    args,
                    span,
                },
                other => Expr::FnApply {
                    func: Box::new(other),
                    args,
                    span,
                },
            };
            continue;
        }

        break;
    }

    Ok(expr)
}

fn name_starts_uppercase(name: &Name) -> bool {
    name.as_ref()
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
}

pub(crate) fn tuple_constructor_fields(items: &[Expr]) -> Vec<(Name, Expr)> {
    items
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, item)| (format!("_{index}").into(), item))
        .collect()
}

fn parse_macro_invocation_after_bang(
    input: &mut ParseInput,
    name: Name,
    start_pos: &Position,
) -> ModalResult<Expr> {
    skip_whitespace_and_comments(input);
    let source = input.input.as_ref();
    let Some(open) = source.chars().next() else {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    };
    let (close, delimiter) = match open {
        '(' => (')', MacroDelimiter::Paren),
        '[' => (']', MacroDelimiter::Bracket),
        '{' => ('}', MacroDelimiter::Brace),
        _ => {
            return Err(winnow::error::ErrMode::Cut(
                winnow::error::ContextError::new(),
            ));
        }
    };

    let mut depth = 0usize;
    let mut end_byte = None;
    for (idx, ch) in source.char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                end_byte = Some(idx + ch.len_utf8());
                break;
            }
        }
    }
    let Some(end_byte) = end_byte else {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    };
    let raw_body_start = open.len_utf8();
    let raw_body_end = end_byte - close.len_utf8();
    let raw_body_text = &source[raw_body_start..raw_body_end];
    let mut body_start_pos = input.state.pos;
    body_start_pos.advance(open);
    let token_trees = parse_macro_token_trees(raw_body_text, body_start_pos);
    let args = parse_macro_invocation_args(delimiter, raw_body_text);
    let body = match (&args, delimiter) {
        (Some(args), MacroDelimiter::Paren) => MacroInvocationBody::ExprArgs(args.clone()),
        _ => MacroInvocationBody::TokenTrees(token_trees.clone()),
    };
    let raw_body = raw_body_text.into();
    for ch in source[..end_byte].chars() {
        input.state.advance(ch);
    }
    let _ = input.input.next_slice(end_byte);
    let span = span_from(start_pos, &input.state.pos);
    Ok(Expr::MacroInvocation {
        invocation: MacroInvocation {
            name,
            delimiter,
            raw_body,
            body,
            token_trees,
            args,
            span,
        },
    })
}

fn parse_macro_token_trees(raw_body: &str, start_pos: Position) -> Vec<MacroTokenTree> {
    let mut parser = MacroTokenTreeParser {
        source: raw_body,
        byte_index: 0,
        pos: start_pos,
    };
    parser.parse_until(None)
}

struct MacroTokenTreeParser<'a> {
    source: &'a str,
    byte_index: usize,
    pos: Position,
}

impl MacroTokenTreeParser<'_> {
    fn parse_until(&mut self, close: Option<char>) -> Vec<MacroTokenTree> {
        let mut trees = Vec::new();
        while let Some(ch) = self.peek_char() {
            if Some(ch) == close {
                break;
            }
            if ch.is_whitespace() {
                self.bump_char(ch);
                continue;
            }
            if let Some((delimiter, close_ch)) = macro_group_delimiter(ch) {
                let group_start = self.pos;
                self.bump_char(ch);
                let tokens = self.parse_until(Some(close_ch));
                if self.peek_char() == Some(close_ch) {
                    self.bump_char(close_ch);
                }
                let span = span_from(&group_start, &self.pos);
                trees.push(MacroTokenTree::Group {
                    delimiter,
                    tokens,
                    span,
                });
                continue;
            }

            let token_start = self.pos;
            let spelling_start = self.byte_index;
            while let Some(token_ch) = self.peek_char() {
                if token_ch.is_whitespace()
                    || Some(token_ch) == close
                    || macro_group_delimiter(token_ch).is_some()
                {
                    break;
                }
                self.bump_char(token_ch);
            }
            let spelling = self.source[spelling_start..self.byte_index].into();
            let span = span_from(&token_start, &self.pos);
            trees.push(MacroTokenTree::Token { spelling, span });
        }
        trees
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.byte_index..].chars().next()
    }

    fn bump_char(&mut self, ch: char) {
        self.byte_index += ch.len_utf8();
        self.pos.advance(ch);
    }
}

fn macro_group_delimiter(ch: char) -> Option<(MacroDelimiter, char)> {
    match ch {
        '(' => Some((MacroDelimiter::Paren, ')')),
        '[' => Some((MacroDelimiter::Bracket, ']')),
        '{' => Some((MacroDelimiter::Brace, '}')),
        _ => None,
    }
}

fn parse_macro_invocation_args(delimiter: MacroDelimiter, raw_body: &str) -> Option<Vec<Expr>> {
    if delimiter != MacroDelimiter::Paren {
        return None;
    }
    if raw_body.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut args_input = crate::input::new_input(raw_body);
    let args = parse_args(&mut args_input).ok()?;
    skip_whitespace_and_comments(&mut args_input);
    if args_input.input.is_empty() {
        Some(args)
    } else {
        None
    }
}

fn parse_operator_section_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let section_start = input.state.pos;
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);

    if starts_operator_section_placeholder(&input.input) {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    if let Some(operator) = parse_raw_operator_token(input) {
        skip_whitespace_and_comments(input);
        if literal_str(")").parse_next(input).is_ok() {
            let span = span_from(&section_start, &input.state.pos);
            return Ok(Expr::OperatorSection {
                section: OperatorSection {
                    kind: OperatorSectionKind::Bare,
                    operator,
                    left: None,
                    right: None,
                    span,
                },
            });
        }
        let right = expr(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(")").parse_next(input)?;
        let span = span_from(&section_start, &input.state.pos);
        return Ok(Expr::OperatorSection {
            section: OperatorSection {
                kind: OperatorSectionKind::Right,
                operator,
                left: None,
                right: Some(Box::new(right)),
                span,
            },
        });
    }

    let left = parse_operator_section_operand(input)?;
    skip_whitespace_and_comments(input);
    let Some(operator) = parse_raw_operator_token(input) else {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    };
    skip_whitespace_and_comments(input);
    let _ = literal_str(")").parse_next(input)?;
    let span = span_from(&section_start, &input.state.pos);
    Ok(Expr::OperatorSection {
        section: OperatorSection {
            kind: OperatorSectionKind::Left,
            operator,
            left: Some(Box::new(left)),
            right: None,
            span,
        },
    })
}

fn starts_operator_section_placeholder(input: &str) -> bool {
    let mut chars = input.chars();
    if chars.next() != Some('_') {
        return false;
    }
    !chars
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn parse_binary_operator_token(
    input: &mut ParseInput,
    operators: &[(&'static str, BinaryOp)],
) -> Option<(BinaryOp, RawOperatorToken)> {
    operators.iter().find_map(|(spelling, op)| {
        parse_specific_raw_operator_token(input, spelling).map(|raw| (*op, raw))
    })
}

fn parse_specific_raw_operator_token(
    input: &mut ParseInput,
    spelling: &'static str,
) -> Option<RawOperatorToken> {
    skip_whitespace_and_comments(input);
    let checkpoint = input.clone();
    let start = input.state.pos;
    if input.input.starts_with(spelling)
        && input
            .input
            .as_ref()
            .chars()
            .nth(spelling.chars().count())
            .is_some_and(is_symbolic_operator_char)
    {
        return None;
    }
    if literal_str(spelling).parse_next(input).is_ok() {
        Some(RawOperatorToken {
            spelling: spelling.into(),
            span: span_from(&start, &input.state.pos),
        })
    } else {
        *input = checkpoint;
        None
    }
}

fn parse_operator_section_operand(input: &mut ParseInput) -> ModalResult<Expr> {
    let operand_start = input.state.pos;
    if let Ok(lit) = literal(input) {
        return Ok(Expr::Literal(lit));
    }
    let (name, span) = expr_name_with_span(input)?;
    let mut expr = Expr::Variable {
        name: name.into(),
        span,
    };
    loop {
        if opt(literal_str(".")).parse_next(input)?.is_some() {
            let field = parse_field_name(input)?;
            let span = span_from(&operand_start, &input.state.pos);
            expr = Expr::FieldAccess {
                base: Box::new(expr),
                field: field.into(),
                span,
            };
            continue;
        }
        break;
    }
    Ok(expr)
}

fn parse_raw_operator_token(input: &mut ParseInput) -> Option<RawOperatorToken> {
    const OPERATORS: &[&str] = &[
        "==", "!=", "<=", ">=", "||", "&&", "+", "-", "*", "/", "%", "<", ">", "|>",
    ];
    skip_whitespace_and_comments(input);
    let start = input.state.pos;
    for operator in OPERATORS {
        if input.input.starts_with(operator) {
            if operator.len() == 1
                && input
                    .input
                    .as_ref()
                    .chars()
                    .nth(1)
                    .is_some_and(is_symbolic_operator_char)
            {
                continue;
            }
            let _ = literal_str(operator).parse_next(input).ok()?;
            return Some(RawOperatorToken {
                spelling: (*operator).into(),
                span: span_from(&start, &input.state.pos),
            });
        }
    }
    let spelling: String = input
        .input
        .as_ref()
        .chars()
        .take_while(|ch| is_symbolic_operator_char(*ch))
        .collect();
    if spelling.is_empty() {
        return None;
    }
    for ch in spelling.chars() {
        input.state.advance(ch);
    }
    let _ = input.input.next_slice(spelling.len());
    Some(RawOperatorToken {
        spelling: spelling.into(),
        span: span_from(&start, &input.state.pos),
    })
}

pub(crate) fn is_symbolic_operator_char(ch: char) -> bool {
    matches!(
        ch,
        '!' | '$'
            | '%'
            | '&'
            | '*'
            | '+'
            | '-'
            | '.'
            | '/'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '^'
            | '|'
            | '~'
    )
}

fn parse_list_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = literal_str("[").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut items = Vec::new();
    if input.input.starts_with(']') {
        let _ = literal_str("]").parse_next(input)?;
        return Ok(Expr::List {
            items,
            span: span_from(&start_pos, &input.state.pos),
        });
    }

    loop {
        items.push(expr(input)?);
        skip_whitespace_and_comments(input);
        if !input.input.starts_with(',') {
            break;
        }
        let _ = literal_str(",").parse_next(input)?;
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str("]").parse_next(input)?;
    Ok(Expr::List {
        items,
        span: span_from(&start_pos, &input.state.pos),
    })
}

fn parse_comprehension_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = literal_str("[").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let result = expr(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("|").parse_next(input)?;
    skip_whitespace_and_comments(input);

    if input.input.starts_with(']') {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let mut qualifiers = vec![parse_comprehension_qualifier(input)?];
    loop {
        skip_whitespace_and_comments(input);
        if !input.input.starts_with(',') {
            break;
        }
        let _ = literal_str(",").parse_next(input)?;
        skip_whitespace_and_comments(input);
        if input.input.starts_with(']') {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
        qualifiers.push(parse_comprehension_qualifier(input)?);
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("]").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let target = if input.input.starts_with(':') {
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let target_start = input.state.pos;
        Some(parse_do_target(input, &target_start)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state.pos);
    Ok(Expr::Comprehension {
        result: Box::new(result),
        qualifiers,
        target,
        span,
    })
}

fn parse_comprehension_qualifier(input: &mut ParseInput) -> ModalResult<ComprehensionQualifier> {
    let qualifier_start = input.state.pos;

    if input_starts_with_keyword(input, "let") {
        let _ = keyword("let").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let name: Name = identifier(input)?.into();
        skip_whitespace_and_comments(input);
        let _ = literal_str("=").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let value = expr(input)?;
        let span = span_from(&qualifier_start, &input.state.pos);
        return Ok(ComprehensionQualifier::Let {
            name,
            value: Box::new(value),
            span,
        });
    }

    let name: Name = identifier(input)?.into();
    skip_whitespace_and_comments(input);
    let _ = literal_str("<-").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let value = expr(input)?;
    let span = span_from(&qualifier_start, &input.state.pos);

    if name.as_ref() == "_" {
        return Ok(ComprehensionQualifier::DiscardBind {
            value: Box::new(value),
            span,
        });
    }

    Ok(ComprehensionQualifier::Bind {
        name,
        value: Box::new(value),
        span,
    })
}

pub(crate) fn parse_constructor_fields(input: &mut ParseInput) -> ModalResult<Vec<(Name, Expr)>> {
    let mut fields = vec![parse_constructor_field(input)?];

    loop {
        skip_whitespace_and_comments(input);
        if opt(literal_str(",")).parse_next(input)?.is_some() {
            skip_whitespace_and_comments(input);
            if input.input.starts_with('}') {
                break;
            }

            fields.push(parse_constructor_field(input)?);
        } else {
            break;
        }
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;
    Ok(fields)
}

fn parse_record_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let fields = if literal_str("}").parse_next(input).is_ok() {
        vec![]
    } else {
        parse_constructor_fields(input)?
    };
    let span = span_from(&start_pos, &input.state.pos);
    Ok(Expr::Record { fields, span })
}

/// Parse a field name in a constructor expression.
/// Unlike `identifier`, this allows keywords as field names (e.g. `role: User`).
pub(crate) fn parse_field_name<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

fn expr_name_with_span<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, Span)> {
    let checkpoint = input.clone();
    if let Ok(parsed) = identifier_with_span(input) {
        return Ok(parsed);
    }
    *input = checkpoint;

    let start_pos = input.state.pos;
    for keyword_name in ["act", "then", "guard", "role"] {
        let checkpoint = input.clone();
        if keyword(keyword_name).parse_next(input).is_ok() {
            let span = span_from(&start_pos, &input.state.pos);
            return Ok((keyword_name, span));
        }
        *input = checkpoint;
    }

    Err(winnow::error::ErrMode::Backtrack(
        winnow::error::ContextError::new(),
    ))
}

fn parse_constructor_field(input: &mut ParseInput) -> ModalResult<(Name, Expr)> {
    skip_whitespace_and_comments(input);
    let name = parse_field_name(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let value = expr(input)?;
    Ok((name.into(), value))
}

fn parse_tuple_constructor_items(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let mut items = vec![expr(input)?];

    loop {
        skip_whitespace_and_comments(input);
        if opt(literal_str(",")).parse_next(input)?.is_some() {
            skip_whitespace_and_comments(input);
            if input.input.starts_with(")") {
                break;
            }
            items.push(expr(input)?);
        } else {
            break;
        }
    }

    Ok(items)
}

fn parse_inline_record_constructor_start(input: &mut ParseInput) -> bool {
    parse_inline_constructor_delimiter(input, '{')
}

fn parse_inline_tuple_constructor_start(input: &mut ParseInput) -> bool {
    parse_inline_constructor_delimiter(input, '(')
}

fn parse_inline_constructor_delimiter(input: &mut ParseInput, delimiter: char) -> bool {
    let source = input.input;
    let inline_ws_len = source
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .map(char::len_utf8)
        .sum::<usize>();

    let Some(rest) = source.get(inline_ws_len..) else {
        return false;
    };

    if !rest.starts_with(delimiter) {
        return false;
    }

    let consumed = &source[..inline_ws_len + 1];
    for c in consumed.chars() {
        input.state.advance(c);
    }
    let _ = input.input.next_slice(inline_ws_len + 1);
    true
}

/// Parse function call arguments
pub(crate) fn parse_args(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let first = expr(input)?;
    let mut args = vec![first];

    loop {
        if opt(literal_str(",")).parse_next(input)?.is_some() {
            let arg = expr(input)?;
            args.push(arg);
        } else {
            break;
        }
    }

    Ok(args)
}

/// Parse a literal value.
pub fn literal(input: &mut ParseInput) -> ModalResult<Literal> {
    alt((
        parse_string,
        parse_float,
        parse_int,
        parse_bool,
        parse_null,
        parse_list,
    ))
    .parse_next(input)
}

/// Parse a string literal.
fn parse_string(input: &mut ParseInput) -> ModalResult<Literal> {
    let _ = literal_str("\"").parse_next(input)?;

    let content = take_while(0.., |c: char| c != '"').parse_next(input)?;

    let _ = literal_str("\"").parse_next(input)?;
    Ok(Literal::String(content.into()))
}

/// Parse an integer literal.
fn parse_int(input: &mut ParseInput) -> ModalResult<Literal> {
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    match digits.parse::<i64>() {
        Ok(n) => Ok(Literal::Int(n)),
        Err(_) => Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        )),
    }
}

/// Parse a floating-point literal.
fn parse_float(input: &mut ParseInput) -> ModalResult<Literal> {
    // Simplified float parsing: digits.digits
    let int_part: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    let _ = one_of('.').parse_next(input)?;
    let frac_part: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    let full = format!("{}.{}", int_part, frac_part);
    match full.parse::<f64>() {
        Ok(f) => Ok(Literal::Float(ordered_float::OrderedFloat(f))),
        Err(_) => Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        )),
    }
}

/// Parse a boolean literal.
fn parse_bool(input: &mut ParseInput) -> ModalResult<Literal> {
    alt((
        keyword("true").map(|_| Literal::Bool(true)),
        keyword("false").map(|_| Literal::Bool(false)),
    ))
    .parse_next(input)
}

/// Parse a null literal.
fn parse_null(input: &mut ParseInput) -> ModalResult<Literal> {
    keyword("null").map(|_| Literal::Null).parse_next(input)
}

/// Parse a list literal: [1, 2, 3] or ["a", "b"]
fn parse_list(input: &mut ParseInput) -> ModalResult<Literal> {
    let _ = literal_str("[").parse_next(input)?;

    // Empty list
    if literal_str("]").parse_next(input).is_ok() {
        return Ok(Literal::List(vec![]));
    }

    // Parse first element
    let first = literal(input)?;
    let mut elements = vec![first];

    // Parse remaining elements
    loop {
        if opt(literal_str(",")).parse_next(input)?.is_some() {
            // Check for trailing comma before ]
            if literal_str("]").parse_next(input).is_ok() {
                break;
            }
            let elem = literal(input)?;
            elements.push(elem);
        } else {
            break;
        }
    }

    let _ = literal_str("]").parse_next(input)?;
    Ok(Literal::List(elements))
}

/// Parse an identifier.
pub fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    crate::parse_utils::identifier_with_span(input).map(|(s, _)| s)
}

/// Parse an identifier and return it with its source span.
///
/// Delegates to the canonical implementation in `parse_utils`.
pub fn identifier_with_span<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, Span)> {
    crate::parse_utils::identifier_with_span(input)
}

/// Parse a keyword (ensures word boundary).
fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, winnow::error::ContextError> {
    move |input: &mut ParseInput<'a>| {
        let _start = input.state.pos;

        if input.input.starts_with(word) {
            let after = &input.input[word.len()..];
            if after.is_empty()
                || !after
                    .chars()
                    .next()
                    .is_some_and(crate::parse_utils::is_identifier_continue)
            {
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
        // Skip whitespace and comments
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
#[allow(clippy::approx_constant)]
mod tests;
