//! Function and builtin-function parsers for module definitions.

use super::*;

pub fn parse_fn_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;

    // Parse optional visibility modifier
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);

    // Parse "fn" keyword
    let _ = keyword("fn").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse function name
    let name = callable_name(input)?;
    skip_whitespace_and_comments(input);

    // Optionally parse type parameters <T, U>
    let type_params = parse_optional_type_parameter_names(input)?;
    skip_whitespace_and_comments(input);

    // Parse parameter list (name: Type, ...)
    let _ = literal_str("(").parse_next(input)?;
    let params = parse_parameter_list(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Optionally parse -> return type
    let return_type = if input.input.starts_with("->") {
        let _ = literal_str("->").parse_next(input)?;
        skip_whitespace_and_comments(input);
        Some(parse_surface_type(input)?)
    } else {
        None
    };
    skip_whitespace_and_comments(input);

    let proposition_tail = if starts_with_keyword(input, "where") {
        Some(parse_proposition_tail(input)?)
    } else {
        None
    };
    skip_whitespace_and_comments(input);

    // Optionally parse contract clauses: requires: ..., ensures: ...
    let contract = parse_fn_contract(input)?;
    skip_whitespace_and_comments(input);

    // Parse block body { ... }
    let body = parse_fn_body(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);

    Ok(Definition::Function(FnDef {
        visibility,
        name: name.into(),
        type_params,
        params,
        return_type,
        proposition_tail,
        contract,
        body,
        span,
    }))
}

/// Parse a builtin function definition.
///
/// Syntax: `[pub] builtin fn <name>[<type_params>](<params>) -> <return_type>;`
///
/// Builtin functions are semicolon-terminated with no body. A return type is
/// required. If braces follow the signature an error is emitted.
pub fn parse_builtin_fn_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;

    // Parse optional visibility modifier
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);

    // Parse "builtin" keyword
    let _ = keyword("builtin").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse "fn" keyword
    let _ = keyword("fn").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse function name
    let name = callable_name(input)?;
    skip_whitespace_and_comments(input);

    // Optionally parse type parameters <T, U>
    let type_params = parse_optional_type_parameter_names(input)?;
    skip_whitespace_and_comments(input);

    // Parse parameter list (name: Type, ...)
    let _ = literal_str("(").parse_next(input)?;
    let params = parse_parameter_list(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Reject braces -- builtin fn must not have a body
    if input.input.starts_with("{") {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    // Parse REQUIRED return type
    if !input.input.starts_with("->") {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }
    let _ = literal_str("->").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let return_type = parse_surface_type(input)?;
    skip_whitespace_and_comments(input);

    let proposition_tail = if starts_with_keyword(input, "where") {
        Some(parse_proposition_tail(input)?)
    } else {
        None
    };
    skip_whitespace_and_comments(input);

    // Reject braces after return type -- builtin fn must not have a body
    if input.input.starts_with("{") {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    // Expect semicolon terminator
    let _ = literal_str(";").parse_next(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);

    Ok(Definition::BuiltinFn(BuiltinFnDef {
        visibility,
        name: name.into(),
        type_params,
        params,
        return_type,
        proposition_tail,
        span,
    }))
}

/// Parse optional contract clauses on a function definition.
fn parse_fn_contract(input: &mut ParseInput) -> ModalResult<Option<Contract>> {
    let mut requires = Vec::new();
    let mut ensures = Vec::new();

    while starts_with_keyword(input, "requires") {
        let _ = keyword("requires").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);

        let clause_exprs = parse_fn_contract_clause_exprs(input)?;
        requires.extend(
            clause_exprs
                .into_iter()
                .map(|expr| crate::surface::Requirement::Arithmetic { expr }),
        );
        skip_whitespace_and_comments(input);
    }

    while starts_with_keyword(input, "ensures") {
        let clause_start = input.state.pos;
        let _ = keyword("ensures").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);

        let clause_exprs = parse_fn_contract_clause_exprs(input)?;
        ensures.extend(
            clause_exprs
                .into_iter()
                .map(|expr| crate::surface::EnsuresClause {
                    expr,
                    span: crate::input::span_from(&clause_start, &input.state.pos),
                }),
        );
        skip_whitespace_and_comments(input);
    }

    if requires.is_empty() && ensures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Contract { requires, ensures }))
    }
}

fn parse_fn_contract_clause_exprs(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let mut exprs = vec![parse_fn_expr(input)?];
    skip_whitespace_and_comments(input);

    while input.input.starts_with(",") {
        let _ = literal_str(",").parse_next(input)?;
        skip_whitespace_and_comments(input);
        exprs.push(parse_fn_expr(input)?);
        skip_whitespace_and_comments(input);
    }

    Ok(exprs)
}

/// Parse a fn body (block expression).
///
/// Syntax: `{ [let pat = expr;]* [tail_expr] }`
pub fn parse_fn_body(input: &mut ParseInput) -> ModalResult<Expr> {
    parse_fn_block_expr(input)
}

/// Parse fn-expr params for use inside a fn body block.
///
/// Each parameter is `name` or `name: Type` (type is optional).
fn parse_fn_expr_params_local(input: &mut ParseInput) -> ModalResult<Vec<(Name, Option<Name>)>> {
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
            let ty_name = parse_simple_type_name_local(input)?;
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

/// Parse a simple type name for local fn parameter/return annotations.
fn parse_simple_type_name_local(input: &mut ParseInput) -> ModalResult<Name> {
    use winnow::token::take_while;
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

/// Parse a block expression: `{ [let pat = expr;]* [tail_expr] }`
fn parse_fn_block_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut statements = Vec::new();

    // Parse let bindings and statements
    loop {
        skip_whitespace_and_comments(input);

        // Check for closing brace
        if input.input.starts_with("}") {
            let _ = literal_str("}").parse_next(input)?;
            let span = crate::input::span_from(&start_pos, &input.state.pos);
            return Ok(Expr::Block {
                statements,
                tail_expr: None,
                span,
            });
        }

        // Try to parse `let` binding
        if starts_with_keyword(input, "let") {
            let stmt_start = input.state.pos;
            let _ = keyword("let").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let pat = crate::parse_pattern::pattern(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str("=").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let let_expr = parse_fn_expr(input)?;
            skip_whitespace_and_comments(input);
            // Optional semicolon
            if input.input.starts_with(";") {
                let _ = input.input.next_slice(1);
                input.state.advance(';');
            }
            let stmt_span = crate::input::span_from(&stmt_start, &input.state.pos);
            statements.push(BlockStmt::Let {
                pattern: pat,
                expr: let_expr,
                span: stmt_span,
            });
            continue;
        }

        // Try to parse a named local fn: `fn name(params) [-> type] { body }`
        // This desugars to BlockStmt::Let { pattern: Variable("name"), expr: FnDef { ... } }
        if starts_with_keyword(input, "fn") {
            let stmt_start = input.state.pos;
            let _ = keyword("fn").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let (fn_name_str, fn_name_span) = identifier_with_span(input)?;
            let fn_name: Name = Box::<str>::from(fn_name_str);
            skip_whitespace_and_comments(input);
            let _ = literal_str("(").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let params = parse_fn_expr_params_local(input)?;
            let _ = literal_str(")").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let return_type = if input.input.starts_with("->") {
                let _ = literal_str("->").parse_next(input)?;
                skip_whitespace_and_comments(input);
                let ty_name = parse_simple_type_name_local(input)?;
                Some(ty_name)
            } else {
                None
            };
            skip_whitespace_and_comments(input);
            let fn_body = crate::parse_expr::parse_fn_expr_body_pub(input)?;
            skip_whitespace_and_comments(input);
            if input.input.starts_with(";") {
                let _ = input.input.next_slice(1);
                input.state.advance(';');
            }
            let stmt_span = crate::input::span_from(&stmt_start, &input.state.pos);
            statements.push(BlockStmt::Let {
                pattern: Pattern::Variable {
                    name: fn_name,
                    span: fn_name_span,
                },
                expr: Expr::FnDef {
                    params,
                    return_type,
                    body: Box::new(fn_body),
                    span: stmt_span,
                },
                span: stmt_span,
            });
            continue;
        }

        // Must be a tail expression
        break;
    }

    skip_whitespace_and_comments(input);

    // Parse tail expression
    if input.input.starts_with("}") {
        let _ = literal_str("}").parse_next(input)?;
        let span = crate::input::span_from(&start_pos, &input.state.pos);
        return Ok(Expr::Block {
            statements,
            tail_expr: None,
            span,
        });
    }

    let tail_expr = parse_fn_expr(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(Expr::Block {
        statements,
        tail_expr: Some(Box::new(tail_expr)),
        span,
    })
}

/// Parse an expression inside a fn body.
///
/// This is the entry point for fn-body expressions. It dispatches to
/// if/match/panic before falling back to the general expression parser
/// (which handles act blocks, closures, and all other expression forms).
fn parse_fn_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    skip_whitespace_and_comments(input);

    // `if let` is its own value-producing expression. Dispatch it before the
    // ordinary `if` parser so the `let` keyword is not parsed as a condition.
    if starts_with_keyword(input, "if") {
        let mut lookahead = input.clone();
        let _ = keyword("if").parse_next(&mut lookahead)?;
        skip_whitespace_and_comments(&mut lookahead);
        if starts_with_keyword(&lookahead, "let") {
            return parse_if_let_expr(input);
        }
    }

    // Try if expression
    if starts_with_keyword(input, "if") {
        return parse_fn_if_expr(input);
    }

    // Try match expression
    if starts_with_keyword(input, "match") {
        return parse_fn_match_expr(input);
    }

    // Try panic expression
    if starts_with_keyword(input, "panic") {
        return parse_panic_expr(input);
    }

    // Fall back to the general expression parser
    expr(input)
}

/// Parse a value-producing if expression: `if condition then then_branch [else else_branch]`
fn parse_fn_if_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = keyword("if").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let condition = parse_fn_expr(input)?;
    skip_whitespace_and_comments(input);

    let _ = keyword("then").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let then_branch = parse_fn_block_or_expr(input)?;
    skip_whitespace_and_comments(input);

    let else_branch = if starts_with_keyword(input, "else") {
        let _ = keyword("else").parse_next(input)?;
        skip_whitespace_and_comments(input);
        Some(Box::new(parse_fn_block_or_expr(input)?))
    } else {
        None
    };

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(Expr::If {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        else_branch,
        span,
    })
}

/// Parse an expression suitable for a match scrutinee.
/// This is a restricted expression parser that does NOT try to parse `{`
/// as a record constructor, which would conflict with the match body delimiter.
fn parse_fn_scrutinee(input: &mut ParseInput) -> ModalResult<Expr> {
    skip_whitespace_and_comments(input);

    // Parenthesized expression
    if input.input.starts_with("(") {
        let _ = literal_str("(").parse_next(input)?;
        let inner = parse_fn_scrutinee(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(")").parse_next(input)?;
        return Ok(inner);
    }

    // Literal
    if let Ok(lit) = crate::parse_expr::literal(input) {
        return Ok(Expr::Literal(lit));
    }

    // Variable / identifier (may have binary ops after)
    let (name, name_span) = crate::parse_expr::identifier_with_span(input)?;
    let mut result = Expr::Variable {
        name: name.into(),
        span: name_span,
    };

    // Handle binary operators (but NOT { which would be a constructor)
    skip_whitespace_and_comments(input);
    while let Some(op) = try_parse_bin_op(input) {
        skip_whitespace_and_comments(input);
        let right = parse_fn_scrutinee(input)?;
        let span = crate::token::Span::default();
        result = Expr::Binary {
            op,
            raw_operator: None,
            left: Box::new(result),
            right: Box::new(right),
            span,
        };
        skip_whitespace_and_comments(input);
    }

    Ok(result)
}

/// Try to parse a binary operator, returning None if not found.
fn try_parse_bin_op(input: &mut ParseInput) -> Option<crate::surface::BinaryOp> {
    use crate::surface::BinaryOp;
    skip_whitespace_and_comments(input);
    // Check two-char ops first
    if input.input.starts_with("==") {
        let _ = input.input.next_slice(2);
        input.state.advance('=');
        input.state.advance('=');
        return Some(BinaryOp::Eq);
    }
    if input.input.starts_with("!=") {
        let _ = input.input.next_slice(2);
        input.state.advance('!');
        input.state.advance('=');
        return Some(BinaryOp::Neq);
    }
    if input.input.starts_with("<=") {
        let _ = input.input.next_slice(2);
        input.state.advance('<');
        input.state.advance('=');
        return Some(BinaryOp::Leq);
    }
    if input.input.starts_with(">=") {
        let _ = input.input.next_slice(2);
        input.state.advance('>');
        input.state.advance('=');
        return Some(BinaryOp::Geq);
    }
    if input.input.starts_with("&&") {
        let _ = input.input.next_slice(2);
        input.state.advance('&');
        input.state.advance('&');
        return Some(BinaryOp::And);
    }
    if input.input.starts_with("||") {
        let _ = input.input.next_slice(2);
        input.state.advance('|');
        input.state.advance('|');
        return Some(BinaryOp::Or);
    }
    // Single-char ops
    if input.input.starts_with("+") {
        let _ = input.input.next_slice(1);
        input.state.advance('+');
        return Some(BinaryOp::Add);
    }
    if input.input.starts_with("-") && !input.input.starts_with("->") {
        let _ = input.input.next_slice(1);
        input.state.advance('-');
        return Some(BinaryOp::Sub);
    }
    if input.input.starts_with("*") {
        let _ = input.input.next_slice(1);
        input.state.advance('*');
        return Some(BinaryOp::Mul);
    }
    if input.input.starts_with("/") {
        let _ = input.input.next_slice(1);
        input.state.advance('/');
        return Some(BinaryOp::Div);
    }
    if input.input.starts_with("%") {
        let _ = input.input.next_slice(1);
        input.state.advance('%');
        return Some(BinaryOp::Mod);
    }
    if input.input.starts_with("<") {
        let _ = input.input.next_slice(1);
        input.state.advance('<');
        return Some(BinaryOp::Lt);
    }
    if input.input.starts_with(">") {
        let _ = input.input.next_slice(1);
        input.state.advance('>');
        return Some(BinaryOp::Gt);
    }
    None
}

/// Parse a match expression: `match scrutinee { pattern => expr [, ...] }`
fn parse_fn_match_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = keyword("match").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Use or_expr for scrutinee to avoid consuming { as a record constructor.
    // parse_fn_expr -> expr -> primary_expr would see "name {" and try to
    // parse a record constructor, which would eat the match body delimiter.
    // We use a simple atom parser that only handles variables, literals,
    // parenthesized expressions, and binary ops — not constructors.
    let scrutinee = parse_fn_scrutinee(input)?;
    skip_whitespace_and_comments(input);

    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut arms = Vec::new();
    while !input.input.starts_with("}") {
        let arm_start = input.state.pos;
        let pat = crate::parse_pattern::pattern(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("=>").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let body = parse_fn_block_or_expr(input)?;
        let arm_span = crate::input::span_from(&arm_start, &input.state.pos);
        arms.push(MatchArm {
            pattern: pat,
            body: Box::new(body),
            span: arm_span,
        });
        skip_whitespace_and_comments(input);
        // Optional trailing comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        }
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str("}").parse_next(input)?;
    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(Expr::Match {
        scrutinee: Box::new(scrutinee),
        arms,
        span,
    })
}

/// Parse a panic expression: `panic "message"`
fn parse_panic_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state.pos;
    let _ = keyword("panic").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse string literal for the message
    let message = parse_panic_string(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(Expr::Panic { message, span })
}

/// Parse a string literal, returning the content as a String.
fn parse_panic_string(input: &mut ParseInput) -> ModalResult<Box<str>> {
    skip_whitespace_and_comments(input);
    let _ = literal_str("\"").parse_next(input)?;

    // Collect characters until closing quote
    let mut content = String::new();
    loop {
        let Some(c) = input.input.next_token() else {
            break;
        };
        input.state.advance(c);
        if c == '"' {
            break;
        }
        content.push(c);
    }

    Ok(content.into_boxed_str())
}

/// Parse either a block `{ ... }` or a single expression for fn body branches.
fn parse_fn_block_or_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    skip_whitespace_and_comments(input);
    if input.input.starts_with("{") {
        parse_fn_block_expr(input)
    } else {
        parse_fn_expr(input)
    }
}
