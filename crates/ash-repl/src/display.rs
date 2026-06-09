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

/// Format the surface AST for an expression or workflow definition.
///
/// Expressions are parsed directly as [`ash_parser::surface::Expr`]. If the
/// input is not a complete expression, workflow parsing is attempted and the
/// resulting [`ash_parser::surface::WorkflowDef`] is formatted instead.
///
/// # Errors
///
/// Returns an error when the input is neither a complete expression nor a
/// complete workflow definition.
pub fn ast_display(input: &str) -> Result<String, ReplError> {
    match parse_expr_complete(input) {
        Ok(expr) => Ok(ast::display_expr(&expr)),
        Err(expr_error) => match parse_workflow_complete(input) {
            Ok(def) => Ok(ast::display_workflow_def(&def)),
            Err(workflow_error) => {
                if trimmed_starts_with_workflow(input) {
                    Err(workflow_error)
                } else {
                    Err(expr_error)
                }
            }
        },
    }
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

fn parse_workflow_complete(input: &str) -> Result<ash_parser::surface::WorkflowDef, ReplError> {
    let mut parser_input = ash_parser::new_input(input);
    let workflow = ash_parser::workflow_def
        .parse_next(&mut parser_input)
        .map_err(|err| ReplError::ParseError(format!("{err}")))?;
    let remaining = parser_input.input.to_string();
    ensure_no_trailing_input(&remaining)?;
    Ok(workflow)
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

fn trimmed_starts_with_workflow(input: &str) -> bool {
    let trimmed = skip_trivia(input);
    trimmed.strip_prefix("workflow").is_some_and(|rest| {
        rest.is_empty() || !rest.chars().next().is_some_and(char::is_alphanumeric)
    })
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
