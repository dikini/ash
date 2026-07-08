//! Workflow statement parser for the Ash language.
//!
//! This module provides parsers for Ash workflow statements and definitions.

use winnow::combinator::{alt, delimited, separated};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::{ParseInput, Position};
use crate::parse_expr::expr;
use crate::parse_pattern::pattern;
use crate::parse_receive::parse_receive;
use crate::parse_send::parse_send;
use crate::parse_set::parse_set;
use crate::parse_utils::{
    parse_kind_annotation, skip_whitespace_and_comments, starts_with_kind_syntax,
};
use crate::surface::{
    ActionRef, CapabilityDecl, CheckTarget, ConstraintBlock, ConstraintField, ConstraintValue,
    Contract, EnsuresClause, Expr, Guard, InterfaceBound, Name, ObligationRef, Parameter, Pattern,
    Requirement, Spanned, Type, TypeParam, Workflow, WorkflowDef, WorkflowHeaderEvent,
};
use crate::token::Span;

/// Parse a workflow definition: `workflow <name>[(<params>)] [<contract>] { <body> }`
pub fn workflow_def(input: &mut ParseInput) -> ModalResult<WorkflowDef> {
    let start_pos = input.state.pos;

    let _ = keyword("workflow").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;

    // Parse optional generic parameters: <T: Explain>
    skip_whitespace_and_comments(input);
    let type_params = if input.input.starts_with("<") {
        parse_type_params(input)?
    } else {
        vec![]
    };

    // Parse optional parameters: (x: Int, y: String)
    skip_whitespace_and_comments(input);
    let params = if input.input.starts_with("(") {
        parse_params(input)?
    } else {
        vec![]
    };

    // Parse optional declared return type: -> Type
    skip_whitespace_and_comments(input);
    let declared_return_type = parse_optional_declared_return_type(input)?;

    let header_events = parse_workflow_header_events(input)?;
    let mut requires = Vec::new();
    let mut ensures = Vec::new();
    for event in &header_events {
        match event {
            WorkflowHeaderEvent::Requires { expr, .. } => {
                requires.push(Requirement::Arithmetic { expr: expr.clone() });
            }
            WorkflowHeaderEvent::Ensures { expr, span } => {
                ensures.push(EnsuresClause {
                    expr: expr.clone(),
                    span: *span,
                });
            }
        }
    }
    let contract = if requires.is_empty() && ensures.is_empty() {
        None
    } else {
        Some(Contract { requires, ensures })
    };

    skip_whitespace_and_comments(input);
    let body = delimited(literal_str("{"), workflow, literal_str("}")).parse_next(input)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(WorkflowDef {
        name: name.into(),
        type_params,
        params,
        declared_return_type,
        plays_roles: Vec::new(),
        capabilities: Vec::new(),
        header_events,
        body,
        contract,
        span,
    })
}

/// Parse source-ordered entry header clauses.
fn parse_workflow_header_events(input: &mut ParseInput) -> ModalResult<Vec<WorkflowHeaderEvent>> {
    let mut events = Vec::new();

    loop {
        skip_whitespace_and_comments(input);
        if starts_with_keyword(input, "requires") {
            let start_pos = input.state.pos;
            let _ = keyword("requires").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str(":").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let expr = expr(input)?;
            events.push(WorkflowHeaderEvent::Requires {
                expr,
                span: span_from(&start_pos, &input.state.pos),
            });
        } else if starts_with_keyword(input, "ensures") {
            let start_pos = input.state.pos;
            let _ = keyword("ensures").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str(":").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let expr = expr(input)?;
            events.push(WorkflowHeaderEvent::Ensures {
                expr,
                span: span_from(&start_pos, &input.state.pos),
            });
        } else {
            break;
        }
    }

    Ok(events)
}

/// Parse optional role capability metadata.
pub fn parse_capabilities_clause(input: &mut ParseInput) -> ModalResult<Vec<CapabilityDecl>> {
    if !starts_with_keyword(input, "capabilities") {
        return Ok(Vec::new());
    }

    let _ = keyword("capabilities").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);

    delimited(literal_str("["), parse_capability_list, literal_str("]")).parse_next(input)
}

/// Parse a comma-separated list of capability declarations.
fn parse_capability_list(input: &mut ParseInput) -> ModalResult<Vec<CapabilityDecl>> {
    let mut capabilities = Vec::new();

    skip_whitespace_and_comments(input);

    // Check for empty list
    if input.input.starts_with("]") {
        return Ok(capabilities);
    }

    loop {
        skip_whitespace_and_comments(input);
        let cap = parse_capability_decl(input)?;
        capabilities.push(cap);

        skip_whitespace_and_comments(input);

        // Check for comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            continue;
        }

        break;
    }

    Ok(capabilities)
}

/// Parse a single capability declaration: `name [@ { constraints }]`
fn parse_capability_decl(input: &mut ParseInput) -> ModalResult<CapabilityDecl> {
    let start_pos = input.state.pos;

    let name = identifier(input)?;
    skip_whitespace_and_comments(input);

    // Parse optional constraint refinement: @ { ... }
    let constraints = if input.input.starts_with("@") {
        let _ = input.input.next_slice(1);
        input.state.advance('@');
        skip_whitespace_and_comments(input);
        Some(parse_constraint_block(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state.pos);

    Ok(CapabilityDecl {
        capability: name.into(),
        constraints,
        span,
    })
}

/// Parse a constraint block: `{ field: value, ... }`
fn parse_constraint_block(input: &mut ParseInput) -> ModalResult<ConstraintBlock> {
    let start_pos = input.state.pos;

    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut fields = Vec::new();

    // Check for empty block
    if input.input.starts_with("}") {
        let _ = literal_str("}").parse_next(input)?;
        let span = span_from(&start_pos, &input.state.pos);
        return Ok(ConstraintBlock { fields, span });
    }

    loop {
        skip_whitespace_and_comments(input);
        let field = parse_constraint_field(input)?;
        fields.push(field);

        skip_whitespace_and_comments(input);

        // Check for comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            continue;
        }

        break;
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;

    let span = span_from(&start_pos, &input.state.pos);
    Ok(ConstraintBlock { fields, span })
}

/// Parse a single constraint field: `name: value`
fn parse_constraint_field(input: &mut ParseInput) -> ModalResult<ConstraintField> {
    let start_pos = input.state.pos;

    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let value = parse_constraint_value(input)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(ConstraintField {
        name: name.into(),
        value,
        span,
    })
}

/// Parse a constraint value (bool, int, string, array, or object).
fn parse_constraint_value(input: &mut ParseInput) -> ModalResult<ConstraintValue> {
    skip_whitespace_and_comments(input);

    // Try boolean
    if keyword("true").parse_next(input).is_ok() {
        return Ok(ConstraintValue::Bool(true));
    }
    if keyword("false").parse_next(input).is_ok() {
        return Ok(ConstraintValue::Bool(false));
    }

    // Try string
    if input.input.starts_with('"') {
        return parse_constraint_string(input);
    }

    // Try array
    if input.input.starts_with('[') {
        return parse_constraint_array(input);
    }

    // Try object
    if input.input.starts_with('{') {
        return parse_constraint_object(input);
    }

    // Try integer (or fall back to identifier as string)
    parse_constraint_int_or_string(input)
}

/// Parse a string literal for constraint value.
fn parse_constraint_string(input: &mut ParseInput) -> ModalResult<ConstraintValue> {
    let _ = literal_str("\"").parse_next(input)?;
    let mut result = String::new();

    while !input.input.is_empty() && !input.input.starts_with('"') {
        if input.input.starts_with("\\") {
            // Handle escape sequences
            let _ = input.input.next_slice(1);
            input.state.advance('\\');
            if let Some(c) = input.input.chars().next() {
                let _ = input.input.next_slice(1);
                input.state.advance(c);
                match c {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => result.push(c),
                }
            }
        } else {
            let c = input.input.chars().next().unwrap();
            let _ = input.input.next_slice(1);
            input.state.advance(c);
            result.push(c);
        }
    }

    let _ = literal_str("\"").parse_next(input)?;
    Ok(ConstraintValue::String(result))
}

/// Parse an array of constraint values: `[value, ...]`
fn parse_constraint_array(input: &mut ParseInput) -> ModalResult<ConstraintValue> {
    let _ = literal_str("[").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut values = Vec::new();

    // Check for empty array
    if input.input.starts_with("]") {
        let _ = literal_str("]").parse_next(input)?;
        return Ok(ConstraintValue::Array(values));
    }

    loop {
        skip_whitespace_and_comments(input);
        let value = parse_constraint_value(input)?;
        values.push(value);

        skip_whitespace_and_comments(input);

        // Check for comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            continue;
        }

        break;
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("]").parse_next(input)?;
    Ok(ConstraintValue::Array(values))
}

/// Parse an object with key-value pairs: `{ key: value, ... }`
fn parse_constraint_object(input: &mut ParseInput) -> ModalResult<ConstraintValue> {
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut pairs = Vec::new();

    // Check for empty object
    if input.input.starts_with("}") {
        let _ = literal_str("}").parse_next(input)?;
        return Ok(ConstraintValue::Object(pairs));
    }

    loop {
        skip_whitespace_and_comments(input);

        // Parse key (identifier or string)
        let key = if input.input.starts_with('"') {
            // Parse string key
            let _ = literal_str("\"").parse_next(input)?;
            let mut key_str = String::new();
            while !input.input.is_empty() && !input.input.starts_with('"') {
                let c = input.input.chars().next().unwrap();
                let _ = input.input.next_slice(1);
                input.state.advance(c);
                key_str.push(c);
            }
            let _ = literal_str("\"").parse_next(input)?;
            key_str
        } else {
            identifier(input)?.to_string()
        };

        skip_whitespace_and_comments(input);
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let value = parse_constraint_value(input)?;
        pairs.push((key, value));

        skip_whitespace_and_comments(input);

        // Check for comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            continue;
        }

        break;
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;
    Ok(ConstraintValue::Object(pairs))
}

/// Parse an integer or identifier (as string).
fn parse_constraint_int_or_string(input: &mut ParseInput) -> ModalResult<ConstraintValue> {
    let start = input.state.pos;

    // Try to parse a number (optional minus followed by digits)
    let mut is_negative = false;
    if input.input.starts_with('-') {
        is_negative = true;
        let _ = input.input.next_slice(1);
        input.state.advance('-');
    }

    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    if is_negative || !digits.is_empty() {
        // We have a number
        if let Ok(num) = digits.parse::<i64>() {
            let result = if is_negative { -num } else { num };
            return Ok(ConstraintValue::Int(result));
        }
    }

    // Not a number - backtrack and try identifier
    input.state.pos = start;
    let name = identifier(input)?;
    Ok(ConstraintValue::String(name.to_string()))
}

/// Check if input starts with a keyword (ensures word boundary).
fn starts_with_keyword(input: &ParseInput, word: &str) -> bool {
    if !input.input.starts_with(word) {
        return false;
    }
    let after = &input.input[word.len()..];
    after
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
}

/// Parse generic type parameters with canonical interface bounds: `<T: Explain>`
fn parse_type_params(input: &mut ParseInput) -> ModalResult<Vec<TypeParam>> {
    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();

    loop {
        skip_whitespace_and_comments(input);
        let start = input.state.pos;
        let name = identifier(input)?;
        skip_whitespace_and_comments(input);

        let mut kind = None;
        let mut bounds = Vec::new();
        if input.input.starts_with(":") {
            let _ = literal_str(":").parse_next(input)?;
            skip_whitespace_and_comments(input);
            if starts_with_kind_syntax(input) {
                kind = Some(parse_kind_annotation(input)?);
            } else {
                bounds.push(parse_interface_bound(input)?);
            }
        }

        params.push(TypeParam {
            name: name.into(),
            kind,
            bounds,
            span: span_from(&start, &input.state.pos),
        });

        skip_whitespace_and_comments(input);
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            continue;
        }

        if input.input.starts_with(">") {
            let _ = literal_str(">").parse_next(input)?;
            break;
        }

        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(params)
}

fn parse_interface_bound(input: &mut ParseInput) -> ModalResult<InterfaceBound> {
    let start = input.state.pos;
    let interface = identifier(input)?;
    Ok(InterfaceBound {
        interface: interface.into(),
        span: span_from(&start, &input.state.pos),
    })
}

/// Parse workflow parameters: `(x: Int, y: String)`
fn parse_params(input: &mut ParseInput) -> ModalResult<Vec<Parameter>> {
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();

    // Check for empty params
    if input.input.starts_with(")") {
        let _ = literal_str(")").parse_next(input)?;
        return Ok(params);
    }

    loop {
        skip_whitespace_and_comments(input);
        let param_start = input.state.pos;

        // Parse parameter name and validate it's not a keyword
        let name = identifier(input)?;
        if is_keyword(name) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
        skip_whitespace_and_comments(input);

        // Parse colon
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);

        // Parse type
        let ty = parse_type(input)?;

        let param_span = span_from(&param_start, &input.state.pos);
        params.push(Parameter {
            name: name.into(),
            ty,
            span: param_span,
        });

        skip_whitespace_and_comments(input);

        // Check for comma or end of params
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            continue;
        } else if input.input.starts_with(")") {
            let _ = literal_str(")").parse_next(input)?;
            break;
        } else {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
    }

    Ok(params)
}

/// Parse a type: simple identifier or generic constructor like List<Int>
fn parse_type(input: &mut ParseInput) -> ModalResult<Type> {
    skip_whitespace_and_comments(input);

    if input.input.starts_with("()") {
        let _ = literal_str("()").parse_next(input)?;
        return Ok(Type::Name("()".into()));
    }

    if starts_with_keyword(input, "cap") {
        let _ = keyword("cap").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let name = identifier(input)?;
        return Ok(Type::Capability(name.into()));
    }

    let type_name = identifier(input)?;
    if type_name == "_" {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    let name = Name::from(type_name);

    skip_whitespace_and_comments(input);

    // Check for generic type arguments: <T, U>
    if input.input.starts_with("<") {
        let _ = literal_str("<").parse_next(input)?;
        skip_whitespace_and_comments(input);

        // Parse type arguments separated by commas
        let args = separated(1.., parse_type, parse_type_arg_separator).parse_next(input)?;

        skip_whitespace_and_comments(input);
        let _ = literal_str(">").parse_next(input)?;

        return Ok(Type::Constructor { name, args });
    }

    Ok(Type::Name(name))
}

/// Parse an optional declared workflow return type: `-> Type`.
fn parse_optional_declared_return_type(input: &mut ParseInput) -> ModalResult<Option<Type>> {
    if !input.input.starts_with("->") {
        return Ok(None);
    }

    let _ = literal_str("->").parse_next(input)?;
    skip_whitespace_and_comments(input);
    parse_type(input).map(Some)
}

/// Parse type argument separator (comma with optional whitespace)
fn parse_type_arg_separator(input: &mut ParseInput) -> ModalResult<()> {
    skip_whitespace_and_comments(input);
    literal_str(",").parse_next(input)?;
    skip_whitespace_and_comments(input);
    Ok(())
}

#[allow(dead_code)]
/// Parse optional contract clauses
fn parse_opt_contract(input: &mut ParseInput) -> ModalResult<Option<Contract>> {
    let mut requires = Vec::new();
    let mut ensures = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.starts_with("requires") {
            let req = parse_requires_clause(input)?;
            requires.push(req);
        } else if input.input.starts_with("ensures") {
            let ens = parse_ensures_clause(input)?;
            ensures.push(ens);
        } else {
            break;
        }
    }

    if requires.is_empty() && ensures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Contract { requires, ensures }))
    }
}

#[allow(dead_code)]
/// Parse a requires clause: `requires: <expr>`
fn parse_requires_clause(input: &mut ParseInput) -> ModalResult<Requirement> {
    let _ = keyword("requires").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse the requirement expression
    let expr = expr(input)?;

    Ok(Requirement::Arithmetic { expr })
}

#[allow(dead_code)]
/// Parse an ensures clause: `ensures: <expr>`
fn parse_ensures_clause(input: &mut ParseInput) -> ModalResult<EnsuresClause> {
    let start_pos = input.state.pos;

    let _ = keyword("ensures").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let expr = expr(input)?;
    let span = span_from(&start_pos, &input.state.pos);

    Ok(EnsuresClause { expr, span })
}

/// Parse a workflow body - sequence of statements separated by semicolons
pub fn workflow(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    // Parse statements
    let stmts = parse_stmt_list(input)?;

    if stmts.is_empty() {
        return Ok(Workflow::Done {
            span: span_from(&start_pos, &input.state.pos),
        });
    }

    // Lower statements to canonical nested form
    // Following SPEC-002 Section 4.4: right-associative folding into LET ... IN cont or SEQ forms
    // Even a single statement should be wrapped in appropriate continuation
    Ok(lower_stmts_to_nested(&stmts, start_pos, input))
}

/// Check if a workflow is a binding statement (creates a lexical scope)
fn is_binding_stmt(stmt: &Workflow) -> bool {
    match stmt {
        Workflow::Let { .. } => true,
        Workflow::Observe { binding, .. } => binding.is_some(),
        Workflow::Orient { binding, .. } => binding.is_some(),
        Workflow::Propose { binding, .. } => binding.is_some(),
        Workflow::Act { result_name, .. } => result_name.is_some(),
        _ => false,
    }
}

/// Check if a workflow is a terminal statement.
/// Terminal statements are `Ret`, `Done`, `Act` without `result_name`, or
/// conditionals whose branches are themselves terminal.
fn is_terminal_stmt(stmt: &Workflow) -> bool {
    match stmt {
        Workflow::Ret { .. } | Workflow::Done { .. } => true,
        Workflow::Let { continuation, .. }
        | Workflow::Observe { continuation, .. }
        | Workflow::Orient { continuation, .. }
        | Workflow::Propose { continuation, .. } => {
            continuation.as_deref().is_some_and(is_terminal_stmt)
        }
        Workflow::Act {
            result_name,
            continuation,
            ..
        } => result_name.is_none() || continuation.as_deref().is_some_and(is_terminal_stmt),
        Workflow::Decide {
            then_branch,
            else_branch,
            ..
        }
        | Workflow::If {
            then_branch,
            else_branch,
            ..
        } => is_terminal_stmt(then_branch) && else_branch.as_deref().is_none_or(is_terminal_stmt),
        Workflow::Check { continuation, .. } => {
            continuation.as_deref().is_none_or(is_terminal_stmt)
        }
        Workflow::Receive { arms, .. } => arms.iter().all(|arm| is_terminal_stmt(&arm.body)),
        _ => false,
    }
}

/// Lower a list of statements to canonical nested form
/// Following SPEC-002 Section 4.4: right-associative folding
///
/// Binding statements (let, observe/orient/propose with binding) capture the
/// lowered remainder as continuation (LET pat = expr IN cont)
///
/// Non-binding statements lower via SEQ (SEQ stmt cont)
fn lower_stmts_to_nested(stmts: &[Workflow], start_pos: Position, input: &ParseInput) -> Workflow {
    let end_span = span_from(&start_pos, &input.state.pos);

    // Fold right-associatively: [s1, s2, s3, done] becomes
    // LET/SEQ s1 (LET/SEQ s2 (LET/SEQ s3 Done))
    stmts
        .iter()
        .rfold(Workflow::Done { span: end_span }, |cont, stmt| {
            let stmt_span = stmt.span();

            if is_binding_stmt(stmt) {
                // Binding statement: extract components and set continuation
                match stmt.clone() {
                    Workflow::Let {
                        pattern,
                        expr,
                        span,
                        ..
                    } => Workflow::Let {
                        pattern,
                        expr,
                        continuation: Some(Box::new(cont)),
                        span,
                    },
                    Workflow::Observe {
                        capability,
                        binding,
                        span,
                        ..
                    } => Workflow::Observe {
                        capability,
                        binding,
                        continuation: Some(Box::new(cont)),
                        span,
                    },
                    Workflow::Orient {
                        expr,
                        binding,
                        span,
                        ..
                    } => Workflow::Orient {
                        expr,
                        binding,
                        continuation: Some(Box::new(cont)),
                        span,
                    },
                    Workflow::Propose {
                        action,
                        binding,
                        span,
                        ..
                    } => Workflow::Propose {
                        action,
                        binding,
                        continuation: Some(Box::new(cont)),
                        span,
                    },
                    Workflow::Act {
                        action,
                        guard,
                        result_name,
                        continuation: existing_cont,
                        span,
                    } => {
                        // Compose existing `then` continuation with the outer tail.
                        // If act already has a `then <body>`, that body runs first,
                        // then the remaining statements follow.
                        let composed = match existing_cont {
                            Some(inner) => Workflow::Seq {
                                first: Box::new(*inner),
                                second: Box::new(cont),
                                span,
                            },
                            None => cont,
                        };
                        Workflow::Act {
                            action,
                            guard,
                            result_name,
                            continuation: Some(Box::new(composed)),
                            span,
                        }
                    }
                    _ => {
                        unreachable!("is_binding_stmt should only return true for binding variants")
                    }
                }
            } else {
                // Check if this is a terminal statement with Done continuation
                // If so, return the statement directly without wrapping in Seq
                if is_terminal_stmt(stmt) && matches!(cont, Workflow::Done { .. }) {
                    stmt.clone()
                } else {
                    // Non-binding statement: use SEQ
                    Workflow::Seq {
                        first: Box::new(stmt.clone()),
                        second: Box::new(cont),
                        span: stmt_span,
                    }
                }
            }
        })
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

        let stmt = parse_stmt(input)?;
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

    Ok(stmts)
}

/// Parse a single workflow statement
fn parse_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    alt((
        observe_stmt,
        orient_stmt,
        propose_stmt,
        receive_stmt,
        decide_stmt,
        oblige_stmt,
        check_stmt,
        act_stmt,
        fn_let_stmt,
        let_stmt,
        if_stmt,
        for_stmt,
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

/// Parse a named local fn statement: `fn name(params) [-> type] { body }`.
///
/// Desugars to `Workflow::Let { pattern: Pattern::Variable { name: "name", span: ash_parser::token::Span::default() }, expr: Expr::FnDef { ... } }`.
fn fn_let_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("fn").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Must have a name (this distinguishes it from anonymous fn expressions)
    let (name_str, name_span) = crate::parse_utils::identifier_with_span(input)?;
    let name: Name = name_str.into();
    skip_whitespace_and_comments(input);

    // Parameter list
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let params = parse_fn_expr_params_wf(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Optional return type
    let return_type = if input.input.starts_with("->") {
        let _ = literal_str("->").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let ty_name = parse_simple_type_name_wf(input)?;
        Some(ty_name)
    } else {
        None
    };
    skip_whitespace_and_comments(input);

    // Body block
    let body = crate::parse_expr::parse_fn_expr_body_pub(input)?;

    let span = span_from(&start_pos, &input.state.pos);
    let fn_def_span = span;

    Ok(Workflow::Let {
        pattern: Pattern::Variable {
            name,
            span: name_span,
        },
        expr: Expr::FnDef {
            params,
            return_type,
            body: Box::new(body),
            span: fn_def_span,
        },
        continuation: None,
        span,
    })
}

/// Parse fn-expr params in workflow context (same logic, local copy to avoid cross-crate issues).
fn parse_fn_expr_params_wf(input: &mut ParseInput) -> ModalResult<Vec<(Name, Option<Name>)>> {
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
            let ty_name = parse_simple_type_name_wf(input)?;
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

/// Parse a simple type name in workflow context.
fn parse_simple_type_name_wf(input: &mut ParseInput) -> ModalResult<Name> {
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

/// Parse an observe statement: `observe <capability> [as <pattern>]`
fn observe_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("observe").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let capability = identifier(input)?;
    skip_whitespace_and_comments(input);

    let capability: Name = if capability == "Args" && starts_with_observe_index(input) {
        let index = parse_observe_index(input)?;
        format!("{capability}:{index}").into_boxed_str()
    } else {
        capability.into()
    };

    skip_whitespace_and_comments(input);

    // Check for optional binding
    let binding = if starts_with_keyword(input, "as") {
        let _ = keyword("as").parse_next(input)?;
        skip_whitespace_and_comments(input);
        Some(pattern(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Observe {
        capability,
        binding,
        continuation: None,
        span,
    })
}

fn starts_with_observe_index(input: &ParseInput) -> bool {
    input
        .input
        .chars()
        .next()
        .is_some_and(|c| c == '-' || c.is_ascii_digit())
}

fn parse_observe_index(input: &mut ParseInput) -> ModalResult<Box<str>> {
    let mut index = String::new();

    if input.input.starts_with('-') {
        let _ = input.input.next_slice(1);
        input.state.advance('-');
        index.push('-');
    }

    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    index.push_str(digits);

    Ok(index.into_boxed_str())
}

/// Parse an orient statement: `orient <expr> [as <pattern>]`
fn orient_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("orient").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let e = expr(input)?;

    // Check for optional binding
    let binding = if keyword("as").parse_next(input).is_ok() {
        Some(pattern(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Orient {
        expr: e,
        binding,
        continuation: None,
        span,
    })
}

/// Parse a propose statement: `propose <action> [as <pattern>]`
fn propose_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("propose").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let action = action_ref(input)?;

    // Check for optional binding
    let binding = if keyword("as").parse_next(input).is_ok() {
        Some(pattern(input)?)
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Propose {
        action,
        binding,
        continuation: None,
        span,
    })
}

/// Parse a receive statement using the dedicated receive parser.
fn receive_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    parse_receive(input)
}

/// Parse a decide statement: `decide { <expr> } under <policy> then <workflow>`
fn decide_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("decide").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let e = expr(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;
    let _ = keyword("under").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let policy = Some(identifier(input)?.into());

    let _ = keyword("then").parse_next(input)?;
    let then_branch = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Decide {
        expr: e,
        policy,
        then_branch,
        else_branch: None,
        span,
    })
}

/// Parse an oblige statement: `oblige <obligation>`
fn oblige_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("oblige").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let obligation = identifier(input)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Oblige {
        obligation: obligation.into(),
        span,
    })
}

/// Parse a check statement: `check <obligation>`
fn check_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("check").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let target = obligation_ref(input).map(CheckTarget::Obligation)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Check {
        target,
        continuation: None,
        span,
    })
}

/// Parse an act statement: `act <action> [where <guard>] [as <name>] [then <workflow>]`
fn act_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("act").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let action = action_ref(input)?;

    // Optional guard
    let guard = if keyword("where").parse_next(input).is_ok() {
        Some(parse_guard(input)?)
    } else {
        None
    };

    // Optional result binding: as <name>
    let result_name = if keyword("as").parse_next(input).is_ok() {
        skip_whitespace_and_comments(input);
        Some(identifier(input)?.into())
    } else {
        None
    };

    // Optional continuation: then <workflow>
    let continuation = if keyword("then").parse_next(input).is_ok() {
        Some(Box::new(parse_single_stmt_or_block(input)?))
    } else {
        None
    };

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Act {
        action,
        guard,
        result_name,
        continuation,
        span,
    })
}

/// Parse a let statement: `let <pattern> = <expr>`
///
/// Also handles sugar: `let <name> = <action_ref>` desugars to
/// `act <action_ref> as <name>` (SurfaceWorkflow::Act).
fn let_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("let").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let pat = pattern(input)?;
    let _ = literal_str("=").parse_next(input)?;

    // Try action_ref() first (backtracking) for cap-call sugar:
    //   let name = provider:action(args)  =>  act provider:action(args) as name
    //
    // Only accept action_ref if what follows is a statement boundary (newline,
    // EOF, semicolon, or a keyword). Otherwise it could be a normal expression
    // like `items[0]` where action_ref would incorrectly consume just `items`.
    skip_whitespace_and_comments(input);
    let checkpoint = input.clone();
    if let Ok(action) = action_ref(input) {
        if let crate::surface::Pattern::Variable { ref name, .. } = pat {
            // Do not treat known builtin function names as capability action targets.
            // Builtin functions like `record(...)` would otherwise be misinterpreted as
            // capability calls when they appear as the RHS of a `let` binding followed
            // by a statement boundary (newline or `}`).
            // Only `Symbolic` targets can shadow builtins; `Qualified` and `Explicit`
            // forms are not affected.
            let is_builtin_fn = matches!(
                &action.target,
                crate::surface::OperationalTarget::Symbolic { capability_name }
                    if crate::lower::BUILTIN_FUNCTIONS.contains(&capability_name.as_ref())
            );
            // Verify what follows is a statement boundary, not an expression continuation.
            // Use skip_horizontal_ws_and_comments to preserve newlines as delimiters.
            crate::parse_utils::skip_horizontal_ws_and_comments(input);
            let next_is_boundary = input.input.is_empty()
                || input.input.starts_with('\n')
                || input.input.starts_with(';')
                || input.input.starts_with('}')
                || starts_with_keyword(input, "then")
                || starts_with_keyword(input, "else");

            if next_is_boundary && !is_builtin_fn {
                let span = span_from(&start_pos, &input.state.pos);
                return Ok(Workflow::Act {
                    action,
                    guard: None,
                    result_name: Some(name.clone()),
                    continuation: None,
                    span,
                });
            }
            // Not a boundary or a known builtin — backtrack and parse as normal expr
            *input = checkpoint;
        } else {
            // Pattern is not a simple name; fall through to expr parsing
            *input = checkpoint;
        }
    } else {
        *input = checkpoint;
    }

    let e = expr(input)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Let {
        pattern: pat,
        expr: e,
        continuation: None,
        span,
    })
}

/// Parse an if statement: `if <expr> then <workflow> [else <workflow>]`
fn if_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

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

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::If {
        condition,
        then_branch,
        else_branch,
        span,
    })
}

/// Parse a for statement: `for <pattern> in <expr> do <workflow>`
fn for_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("for").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let pat = pattern(input)?;
    let _ = keyword("in").parse_next(input)?;
    let collection = expr(input)?;
    let _ = keyword("do").parse_next(input)?;
    let body = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::For {
        pattern: pat,
        collection,
        body,
        span,
    })
}

/// Parse a with statement: `with <capability> do <workflow>`
fn with_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("with").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let capability = identifier(input)?;
    let _ = keyword("do").parse_next(input)?;
    let body = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::With {
        capability: capability.into(),
        body,
        span,
    })
}

/// Parse a maybe statement: `maybe <workflow> else <workflow>`
fn maybe_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("maybe").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let primary = Box::new(parse_single_stmt_or_block(input)?);
    let _ = keyword("else").parse_next(input)?;
    let fallback = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Maybe {
        primary,
        fallback,
        span,
    })
}

/// Parse a must statement: `must <workflow>`
fn must_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("must").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let body = Box::new(parse_single_stmt_or_block(input)?);

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Must { body, span })
}

/// Parse a done statement: `done`
fn done_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("done").parse_next(input)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Done { span })
}

/// Parse a ret statement: `ret <expr>;`
fn ret_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("ret").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let e = expr(input)?;

    let span = span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Ret { expr: e, span })
}

/// Parse a set statement in a workflow: `set capability:channel = expr`
fn set_stmt(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_span = input.state.pos;
    let set_expr = parse_set(input)?;
    let span = span_from(&start_span, &input.state.pos);

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
    let start_span = input.state.pos;
    let send_expr = parse_send(input)?;
    let span = span_from(&start_span, &input.state.pos);

    Ok(Workflow::Send {
        capability: send_expr.capability,
        channel: send_expr.channel,
        value: send_expr.value,
        continuation: None,
        span,
    })
}

/// Parse a single statement or a block
pub(crate) fn parse_single_stmt_or_block(input: &mut ParseInput) -> ModalResult<Workflow> {
    skip_whitespace_and_comments(input);

    if input.input.starts_with("{") {
        delimited(literal_str("{"), workflow, literal_str("}")).parse_next(input)
    } else {
        parse_stmt(input)
    }
}

/// Parse an action reference.
///
/// Supports three forms:
/// - `capability(args)` - symbolic capability call (resolved via resolver)
/// - `module::capability(args)` - module-qualified symbolic call
/// - `provider:action(args)` - explicit provider:action call
pub fn action_ref(input: &mut ParseInput) -> ModalResult<ActionRef> {
    let first_name = identifier(input)?;

    skip_whitespace_and_comments(input);

    // Check for :: (module-qualified) or : (explicit provider:action) form
    let target = if input.input.starts_with("::") {
        // Module-qualified form: module::capability(args)
        let _ = literal_str("::").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let capability_name = identifier(input)?;
        crate::surface::OperationalTarget::Qualified {
            module: first_name.into(),
            capability_name: capability_name.into(),
        }
    } else if input.input.starts_with(":") {
        // Explicit form: provider:action(args)
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let action_name = identifier(input)?;
        crate::surface::OperationalTarget::Explicit {
            provider: first_name.into(),
            action: action_name.into(),
        }
    } else {
        // Symbolic form: capability(args)
        crate::surface::OperationalTarget::Symbolic {
            capability_name: first_name.into(),
        }
    };

    let args = if input.input.starts_with("(") {
        delimited(literal_str("("), parse_expr_list, literal_str(")")).parse_next(input)?
    } else {
        vec![]
    };

    Ok(ActionRef { target, args })
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
    crate::parse_utils::identifier_with_span(input).map(|(s, _)| s)
}

/// Check if a string is a keyword.
fn is_keyword(s: &str) -> bool {
    crate::parse_utils::is_keyword(s)
}

/// Create a span from start position to current position.
fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, winnow::error::ContextError> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);

        if input.input.starts_with(word) {
            let after = &input.input[word.len()..];
            if after
                .chars()
                .next()
                .is_none_or(|next| !next.is_ascii_alphanumeric())
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
mod tests;
