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
    BinaryOp, BlockStmt, ConstructorPayload, Expr, Literal, Name, Pattern, UnaryOp,
};
use crate::token::Span;

/// Parse an expression (entry point).
///
/// This handles the full expression grammar with proper precedence.
pub fn expr(input: &mut ParseInput) -> ModalResult<Expr> {
    // Try closure syntax first: |params| => body  (TASK-557)
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
    pipe_expr(input)
}

/// Parse a pipe expression: left |> right
///
/// Desugars at the parser surface so the core IR never sees `Pipe`.
///   `lhs |> func(args)`  =>  `func(lhs, args)`
///   `lhs |> module::func(args)`  =>  `module::func(lhs, args)`
///   `lhs |> f`  =>  `f(lhs)`
fn pipe_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = ternary_expr(input)?;

    loop {
        if opt(literal_str("|>")).parse_next(input)?.is_some() {
            skip_whitespace_and_comments(input);
            let right = ternary_expr(input)?;
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

/// Parse a closure expression: `|params| => body`. SPEC-031 §4.3, §8.3
///
/// Desugars immediately to `Expr::FnDef { params, return_type: None, body }`.
/// `|x| => x + 1`  =>  `fn(x) { x + 1 }`
/// `|x, y| => x + y`  =>  `fn(x, y) { x + y }`
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

    // Mandatory `=>`
    if !input.input.starts_with("=>") {
        *input = saved;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    let _ = literal_str("=>").parse_next(input)?;
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

/// Parse a simple type name (e.g., `Int`, `String`, `Bool`).
///
/// This is used for type annotations in anonymous fn params and return types.
/// We deliberately keep this simple: a single identifier (no generics, no paths).
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
    Ok(name.into())
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
fn ternary_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let _start_pos = input.state.pos;
    let condition = or_expr(input)?;

    // Check for ternary operator
    if opt(preceded(literal_str("?"), or_expr))
        .parse_next(input)?
        .is_some()
    {
        let _then_branch = or_expr(input)?;
        let _ = preceded(literal_str(":"), or_expr).parse_next(input)?;
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
    let match_expr = ternary_expr(input)?;

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
pub(crate) fn or_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = and_expr(input)?;

    loop {
        if opt(literal_str("||")).parse_next(input)?.is_some() {
            let right = and_expr(input)?;
            let span = span_from(&start_pos, &input.state.pos);
            left = Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        } else {
            break;
        }
    }

    Ok(left)
}

/// Parse logical AND expressions: left && right
fn and_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = in_expr(input)?;

    loop {
        if opt(literal_str("&&")).parse_next(input)?.is_some() {
            let right = in_expr(input)?;
            let span = span_from(&start_pos, &input.state.pos);
            left = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        } else {
            break;
        }
    }

    Ok(left)
}

/// Parse IN expressions: left in right
fn in_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let left = comparison_expr(input)?;

    if opt(keyword("in")).parse_next(input)?.is_some() {
        let right = comparison_expr(input)?;
        let span = span_from(&start_pos, &input.state.pos);
        Ok(Expr::Binary {
            op: BinaryOp::In,
            left: Box::new(left),
            right: Box::new(right),
            span,
        })
    } else {
        Ok(left)
    }
}

/// Parse comparison expressions: ==, !=, <, >, <=, >=
fn comparison_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let left = additive_expr(input)?;

    // Try to match comparison operators
    let op = alt((
        literal_str("==").map(|_| BinaryOp::Eq),
        literal_str("!=").map(|_| BinaryOp::Neq),
        literal_str("<=").map(|_| BinaryOp::Leq),
        literal_str(">=").map(|_| BinaryOp::Geq),
        literal_str("<").map(|_| BinaryOp::Lt),
        literal_str(">").map(|_| BinaryOp::Gt),
    ))
    .parse_next(input);

    match op {
        Ok(op) => {
            let right = additive_expr(input)?;
            let span = span_from(&start_pos, &input.state.pos);
            Ok(Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            })
        }
        Err(_) => Ok(left),
    }
}

/// Parse additive expressions: +, -
fn additive_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = multiplicative_expr(input)?;

    loop {
        let op = alt((
            literal_str("+").map(|_| BinaryOp::Add),
            literal_str("-").map(|_| BinaryOp::Sub),
        ))
        .parse_next(input);

        match op {
            Ok(op) => {
                let right = multiplicative_expr(input)?;
                let span = span_from(&start_pos, &input.state.pos);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            }
            Err(_) => break,
        }
    }

    Ok(left)
}

/// Parse multiplicative expressions: *, /, %
fn multiplicative_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let mut left = unary_expr(input)?;

    loop {
        let op = alt((
            literal_str("*").map(|_| BinaryOp::Mul),
            literal_str("/").map(|_| BinaryOp::Div),
            literal_str("%").map(|_| BinaryOp::Mod),
        ))
        .parse_next(input);

        match op {
            Ok(op) => {
                let right = unary_expr(input)?;
                let span = span_from(&start_pos, &input.state.pos);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            }
            Err(_) => break,
        }
    }

    Ok(left)
}

/// Parse unary expressions: !, -
fn unary_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;

    // Try negation first
    if opt(literal_str("!")).parse_next(input)?.is_some() {
        let operand = unary_expr(input)?;
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
        let operand = primary_expr(input)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(operand),
            span,
        });
    }

    // Try keyword "not"
    if opt(keyword("not")).parse_next(input)?.is_some() {
        let operand = unary_expr(input)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span,
        });
    }

    primary_expr(input)
}

/// Parse primary expressions: literals, variables, field access, index access, calls
#[allow(clippy::collapsible_if)]
fn primary_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;

    // Try parenthesized expression first
    if let Ok(e) = delimited(literal_str("("), expr, literal_str(")")).parse_next(input) {
        return Ok(e);
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

    // Try identifier/variable (and potential field access/call)
    let (name, name_span) = identifier_with_span(input)?;
    let name_str: Name = name.into();

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
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Call {
            func: second_name,
            module: Some(name_str),
            args: vec![],
            span,
        });
    }

    if parse_inline_record_constructor_start(input) {
        skip_whitespace_and_comments(input);
        let fields = if literal_str("}").parse_next(input).is_ok() {
            vec![]
        } else {
            parse_constructor_fields(input)?
        };
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Constructor {
            name: name_str,
            payload: ConstructorPayload::Record(fields.clone()),
            fields,
            span,
        });
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
        return Ok(Expr::Constructor {
            name: name.into(),
            fields: vec![],
            payload: ConstructorPayload::Tuple(items),
            span,
        });
    }

    // Check for field access or method call
    let mut expr = Expr::Variable {
        name: name_str.clone(),
        span: name_span,
    };

    loop {
        // Field access: .field
        if opt(literal_str(".")).parse_next(input)?.is_some() {
            if let Ok(field) = identifier(input) {
                let span = span_from(&start_pos, &input.state.pos);
                expr = Expr::FieldAccess {
                    base: Box::new(expr),
                    field: field.into(),
                    span,
                };
                continue;
            }
        }

        // Index access: [index]
        if opt(literal_str("[")).parse_next(input)?.is_some() {
            let index = self::expr(input)?;
            let _ = literal_str("]").parse_next(input)?;
            let span = span_from(&start_pos, &input.state.pos);
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
            } else {
                let args = parse_args(input)?;
                let _ = literal_str(")").parse_next(input)?;
                args
            };
            let span = span_from(&start_pos, &input.state.pos);
            expr = Expr::Call {
                func: match &expr {
                    Expr::Variable { name: n, .. } => n.clone(),
                    _ => "call".into(),
                },
                module: None,
                args,
                span,
            };
            continue;
        }

        break;
    }

    Ok(expr)
}

fn parse_constructor_fields(input: &mut ParseInput) -> ModalResult<Vec<(Name, Expr)>> {
    let mut fields = vec![parse_constructor_field(input)?];

    loop {
        skip_whitespace_and_comments(input);
        if opt(literal_str(",")).parse_next(input)?.is_some() {
            skip_whitespace_and_comments(input);
            if literal_str("}").parse_next(input).is_ok() {
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

/// Parse a field name in a constructor expression.
/// Unlike `identifier`, this allows keywords as field names (e.g. `role: User`).
fn parse_field_name<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
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
fn parse_args(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
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
        Ok(f) => Ok(Literal::Float(f)),
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
                    .is_some_and(|c| c.is_ascii_alphanumeric())
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
mod tests {
    use super::*;

    fn test_input(s: &str) -> ParseInput<'_> {
        crate::input::new_input(s)
    }

    #[test]
    fn test_parse_int_literal() {
        let mut input = test_input("42");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_parse_float_literal() {
        let mut input = test_input("3.14");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Float(f)) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_parse_string_literal() {
        let mut input = test_input("\"hello world\"");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::String(s)) if s.as_ref() == "hello world"));
    }

    #[test]
    fn test_parse_bool_literal() {
        let mut input = test_input("true");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Bool(true))));

        let mut input = test_input("false");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_parse_null_literal() {
        let mut input = test_input("null");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Null)));
    }

    #[test]
    fn test_parse_variable() {
        let mut input = test_input("my_variable");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Variable { name, .. } if name.as_ref() == "my_variable"));
    }

    #[test]
    fn test_parse_binary_addition() {
        let mut input = test_input("1 + 2");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_binary_multiplication() {
        let mut input = test_input("3 * 4");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_precedence() {
        // Multiplication has higher precedence than addition
        let mut input = test_input("1 + 2 * 3");
        let result = expr(&mut input).unwrap();

        // Should be: 1 + (2 * 3), not (1 + 2) * 3
        match result {
            Expr::Binary {
                op: BinaryOp::Add,
                left,
                right,
                ..
            } => {
                assert!(matches!(left.as_ref(), Expr::Literal(Literal::Int(1))));
                assert!(matches!(
                    right.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        ..
                    }
                ));
            }
            _ => panic!("Expected Add expression"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let mut input = test_input("x > 5");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Gt,
                ..
            }
        ));

        let mut input = test_input("a == b");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Eq,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_logical_and() {
        let mut input = test_input("a && b");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::And,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_logical_or() {
        let mut input = test_input("a || b");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_field_access() {
        let mut input = test_input("obj.field");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::FieldAccess { .. }));
    }

    #[test]
    fn test_parse_function_call() {
        let mut input = test_input("foo()");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Call { func, .. } if func.as_ref() == "foo"));
    }

    #[test]
    fn test_parse_function_call_with_args() {
        let mut input = test_input("foo(1, 2, 3)");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call { func, args, .. } => {
                assert_eq!(func.as_ref(), "foo");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_parse_variable_named_supervises() {
        let mut input = test_input("supervises");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Variable { name, .. } if name.as_ref() == "supervises"));
    }

    #[test]
    fn test_parse_parenthesized() {
        let mut input = test_input("(1 + 2) * 3");
        let result = expr(&mut input).unwrap();

        // Should be: (1 + 2) * 3
        match result {
            Expr::Binary {
                op: BinaryOp::Mul,
                left,
                ..
            } => {
                assert!(matches!(
                    left.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ));
            }
            _ => panic!("Expected Mul expression"),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let mut input = test_input("a + b * c - d / e");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Binary { .. }));
    }

    #[test]
    fn test_parse_in_expression() {
        let mut input = test_input("x in list");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::In,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_pipe_operator_to_unqualified_call() {
        let mut input = test_input("x |> f");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "f");
                assert!(module.is_none());
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::Variable { ref name, .. } if name.as_ref() == "x"));
            }
            other => panic!("expected desugared pipe call, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_pipe_operator_chain_is_left_associative() {
        let mut input = test_input("x |> f |> g");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "g");
                assert!(module.is_none());
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Expr::Call {
                        func, module, args, ..
                    } => {
                        assert_eq!(func.as_ref(), "f");
                        assert!(module.is_none());
                        assert_eq!(args.len(), 1);
                        assert!(
                            matches!(args[0], Expr::Variable { ref name, .. } if name.as_ref() == "x")
                        );
                    }
                    other => {
                        panic!("expected nested call for left-associative pipe, got {other:?}")
                    }
                }
            }
            other => panic!("expected desugared pipe chain, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_pipe_operator_to_module_qualified_call() {
        let mut input = test_input("x |> io::read(y)");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "read");
                assert_eq!(module.as_ref().map(|m| m.as_ref()), Some("io"));
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expr::Variable { ref name, .. } if name.as_ref() == "x"));
                assert!(matches!(args[1], Expr::Variable { ref name, .. } if name.as_ref() == "y"));
            }
            other => panic!("expected module-qualified desugared pipe call, got {other:?}"),
        }
    }

    #[test]
    fn test_pipe_lower_precedence_than_addition() {
        // `a + b |> f` should parse as `(a + b) |> f`, i.e. desugars to `f(a + b)`
        let mut input = test_input("a + b |> f");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "f");
                assert!(module.is_none());
                assert_eq!(args.len(), 1);
                // The single argument should be a binary add: a + b
                assert!(
                    matches!(
                        &args[0],
                        Expr::Binary {
                            op: BinaryOp::Add,
                            ..
                        }
                    ),
                    "expected Binary::Add as the piped argument, got {:?}",
                    args[0]
                );
            }
            other => panic!("expected desugared pipe call, got {other:?}"),
        }
    }

    #[test]
    fn test_pipe_into_call_with_args() {
        // `x |> f(a, b)` should desugar to `f(x, a, b)` — x prepended as first arg
        let mut input = test_input("x |> f(a, b)");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "f");
                assert!(module.is_none());
                assert_eq!(args.len(), 3);
                // First arg: the piped value x
                assert!(
                    matches!(&args[0], Expr::Variable { name, .. } if name.as_ref() == "x"),
                    "expected first arg to be 'x', got {:?}",
                    args[0]
                );
                // Second and third args: a, b
                assert!(
                    matches!(&args[1], Expr::Variable { name, .. } if name.as_ref() == "a"),
                    "expected second arg to be 'a', got {:?}",
                    args[1]
                );
                assert!(
                    matches!(&args[2], Expr::Variable { name, .. } if name.as_ref() == "b"),
                    "expected third arg to be 'b', got {:?}",
                    args[2]
                );
            }
            other => panic!("expected desugared pipe call with prepended arg, got {other:?}"),
        }
    }

    // ============================================================
    // If-Let Expression Tests (TASK-126)
    // ============================================================

    #[test]
    fn test_parse_if_let_simple() {
        // Simple if-let with variant pattern
        let mut input = test_input("if let Some { value: x } = opt then { x } else { 0 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                expr,
                then_branch,
                else_branch,
                ..
            } => {
                // Pattern should be a Variant pattern
                assert!(
                    matches!(pattern, crate::surface::Pattern::Variant { name, .. } if name.as_ref() == "Some")
                );
                // Expression should be variable 'opt'
                assert!(
                    matches!(expr.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "opt")
                );
                // Then branch should be variable 'x'
                assert!(
                    matches!(then_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
                );
                // Else branch should be literal 0
                assert!(matches!(
                    else_branch.as_ref(),
                    Expr::Literal(Literal::Int(0))
                ));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_unit_variant() {
        // Unit variant pattern (just the name without fields)
        let mut input = test_input("if let None = opt then { \"none\" } else { \"some\" }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                then_branch,
                else_branch,
                ..
            } => {
                // Unit variants like `None` parse as variant patterns without braces.
                assert!(matches!(
                    pattern,
                    crate::surface::Pattern::Variant { name, fields, .. }
                        if name.as_ref() == "None" && fields.is_none()
                ));
                // Then branch should be string "none"
                assert!(
                    matches!(then_branch.as_ref(), Expr::Literal(Literal::String(s)) if s.as_ref() == "none")
                );
                // Else branch should be string "some"
                assert!(
                    matches!(else_branch.as_ref(), Expr::Literal(Literal::String(s)) if s.as_ref() == "some")
                );
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_variable_pattern() {
        // Simple variable pattern
        let mut input = test_input("if let x = value then { x } else { 0 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                then_branch,
                else_branch,
                ..
            } => {
                assert!(
                    matches!(pattern, crate::surface::Pattern::Variable { name, .. } if name.as_ref() == "x")
                );
                assert!(
                    matches!(then_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
                );
                assert!(matches!(
                    else_branch.as_ref(),
                    Expr::Literal(Literal::Int(0))
                ));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_wildcard_pattern() {
        // Wildcard pattern
        let mut input = test_input("if let _ = value then { 1 } else { 0 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet { pattern, .. } => {
                assert!(matches!(pattern, crate::surface::Pattern::Wildcard));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_tuple_pattern() {
        // Tuple pattern
        let mut input = test_input("if let (a, b) = pair then { a } else { b }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                then_branch,
                else_branch,
                ..
            } => {
                assert!(matches!(pattern, crate::surface::Pattern::Tuple(pats) if pats.len() == 2));
                assert!(
                    matches!(then_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "a")
                );
                assert!(
                    matches!(else_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "b")
                );
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_complex_expression() {
        // If-let with complex expression in match position
        let mut input = test_input("if let x = foo() + bar then { x } else { 0 }");
        let result = expr(&mut input).unwrap();

        assert!(matches!(result, Expr::IfLet { .. }));
    }

    #[test]
    fn test_parse_if_let_nested_expressions() {
        // Nested expressions in branches
        let mut input = test_input("if let Some { value: x } = opt then { x + 1 } else { x - 1 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                then_branch,
                else_branch,
                ..
            } => {
                // Both branches should be binary expressions
                assert!(matches!(
                    then_branch.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ));
                assert!(matches!(
                    else_branch.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Sub,
                        ..
                    }
                ));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_constructor_expression() {
        let mut input = test_input("Ok { value: 42 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::Constructor {
                name,
                fields,
                payload,
                ..
            } => {
                assert_eq!(name.as_ref(), "Ok");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0.as_ref(), "value");
                assert!(matches!(fields[0].1, Expr::Literal(Literal::Int(42))));
                assert!(matches!(payload, ConstructorPayload::Record(items) if items.len() == 1));
            }
            other => panic!("Expected Constructor expression, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_nested_constructor_expression() {
        let mut input = test_input(r#"Err { error: RuntimeError(42, "boom") }"#);
        let result = expr(&mut input).unwrap();

        match result {
            Expr::Constructor {
                name,
                fields,
                payload,
                ..
            } => {
                assert_eq!(name.as_ref(), "Err");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0.as_ref(), "error");
                assert!(matches!(payload, ConstructorPayload::Record(items) if items.len() == 1));
                match &fields[0].1 {
                    Expr::Constructor {
                        name,
                        fields,
                        payload,
                        ..
                    } => {
                        assert_eq!(name.as_ref(), "RuntimeError");
                        assert!(fields.is_empty());
                        assert!(
                            matches!(payload, ConstructorPayload::Tuple(items) if matches!(
                                items.as_slice(),
                                [
                                    Expr::Literal(Literal::Int(42)),
                                    Expr::Literal(Literal::String(message))
                                ] if message.as_ref() == "boom"
                            ))
                        );
                    }
                    other => panic!("Expected nested constructor, got {other:?}"),
                }
            }
            other => panic!("Expected Constructor expression, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_multi_field_constructor_expression() {
        // "role" is an Ash keyword but should be allowed as constructor field name
        let mut input = test_input("Msg { role: x, text: y }");
        let result = expr(&mut input);
        match result {
            Ok(Expr::Constructor { name, fields, .. }) => {
                assert_eq!(name.as_ref(), "Msg");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0.as_ref(), "role");
                assert_eq!(fields[1].0.as_ref(), "text");
            }
            Ok(other) => panic!("Expected Constructor, got {other:?}"),
            Err(e) => panic!("Parse failed: {e:?}"),
        }
    }

    // =========================================================================
    // Qualified fn call parsing tests (TASK-503)
    // =========================================================================

    #[test]
    fn test_qualified_fn_call_no_args() {
        let mut input = test_input("math::pi()");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "pi");
                assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("math"));
                assert!(args.is_empty());
            }
            other => panic!("Expected Call with module, got {other:?}"),
        }
    }

    #[test]
    fn test_qualified_fn_call_with_args() {
        let mut input = test_input("math::add(1, 2)");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "add");
                assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("math"));
                assert_eq!(args.len(), 2);
            }
            other => panic!("Expected Call with module, got {other:?}"),
        }
    }

    #[test]
    fn test_qualified_fn_call_single_arg() {
        let mut input = test_input("utils::transform(x)");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "transform");
                assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("utils"));
                assert_eq!(args.len(), 1);
            }
            other => panic!("Expected Call with module, got {other:?}"),
        }
    }

    #[test]
    fn test_unqualified_fn_call_still_works() {
        let mut input = test_input("foo(1, 2)");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call {
                func, module, args, ..
            } => {
                assert_eq!(func.as_ref(), "foo");
                assert!(module.is_none());
                assert_eq!(args.len(), 2);
            }
            other => panic!("Expected Call without module, got {other:?}"),
        }
    }
}
