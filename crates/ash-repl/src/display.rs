use ash_engine::Engine;
use winnow::prelude::*;

use crate::{ReplError, ast};

/// Infer the canonical Ash type name for an expression.
///
/// # Errors
///
/// Returns an error when the expression cannot be parsed or does not yield a
/// reportable canonical type.
pub fn infer_type_display(expr: &str) -> Result<String, ReplError> {
    Engine::default()
        .infer_expression_type(expr)
        .map_err(Into::into)
}

/// Format the surface AST for an expression.
///
/// # Errors
///
/// Returns an error when the input is not a complete expression.
pub fn ast_display(input: &str) -> Result<String, ReplError> {
    parse_expr_complete(input).map(|expr| ast::display_expr(&expr))
}

fn parse_expr_complete(input: &str) -> Result<ash_parser::surface::Expr, ReplError> {
    let mut parser_input = ash_parser::new_input(input);
    let expr = ash_parser::expr
        .parse_next(&mut parser_input)
        .map_err(|err| ReplError::ParseError(format!("{err}")))?;
    let remaining = parser_input.input.to_string();
    ensure_no_trailing_input(&remaining)?;
    Ok(expr)
}

fn ensure_no_trailing_input(remaining: &str) -> Result<(), ReplError> {
    let trailing = skip_trivia(remaining);
    if trailing.is_empty() {
        Ok(())
    } else {
        let snippet = trailing.lines().next().unwrap_or(trailing);
        Err(ReplError::ParseError(format!(
            "unexpected trailing input: {snippet}"
        )))
    }
}

fn skip_trivia(mut input: &str) -> &str {
    loop {
        let trimmed = input.trim_start_matches(char::is_whitespace);
        if let Some(rest) = trimmed.strip_prefix("--") {
            input = rest.find('\n').map_or("", |index| &rest[index + 1..]);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("/*") {
            input = skip_block_comment(rest);
            continue;
        }
        return trimmed;
    }
}

fn skip_block_comment(input: &str) -> &str {
    let mut depth = 1usize;
    let mut index = 0usize;

    while index < input.len() {
        let remaining = &input[index..];
        if remaining.starts_with("/*") {
            depth += 1;
            index += 2;
            continue;
        }
        if remaining.starts_with("*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return &input[index..];
            }
            continue;
        }

        if let Some(ch) = remaining.chars().next() {
            index += ch.len_utf8();
        } else {
            break;
        }
    }

    ""
}
