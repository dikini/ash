//! Module declaration parser for the Ash language.
//!
//! This module provides parsers for module declarations, supporting both
//! file-based modules (`mod foo;`) and inline modules (`mod foo { ... }`).

use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::stream::Stream;

use crate::combinators::keyword;
use crate::input::ParseInput;
use crate::module::{ModuleDecl, ModuleSource};
use crate::parse_expr::expr;
use crate::parse_utils::skip_whitespace_and_comments;
use crate::parse_visibility;
use crate::parse_workflow::{parse_capabilities_clause, workflow_def};
use crate::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, BlockStmt, BuiltinFnDef, CapabilityDef,
    CapabilityRef, Constraint, Contract, Definition, EffectType, Expr, FnDef, ImplDef,
    ImplMethodDef, InterfaceDef, InterfaceMethodSig, MatchArm, Name, Param, Pattern, Predicate,
    ProxyDef, RoleDef, Type, Visibility, WhereBound, Workflow, YieldArm,
};
use crate::token::Span;

/// Parse a module declaration.
///
/// Supports both file-based modules (`mod foo;`) and inline modules (`mod foo { ... }`).
/// Visibility modifiers are optional.
///
/// # Examples
///
/// ```
/// use ash_parser::parse_module::parse_module_decl;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// // Parse file-based module
/// let mut input = new_input("mod foo;");
/// let result = parse_module_decl.parse_next(&mut input).unwrap();
/// assert!(result.is_file_based());
/// ```
pub fn parse_module_decl(input: &mut ParseInput) -> ModalResult<ModuleDecl> {
    // Parse optional visibility modifier
    skip_whitespace(input);
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);

    // Parse "mod" keyword
    let _ = keyword("mod").parse_next(input)?;
    skip_whitespace(input);

    // Parse module name
    let name = identifier(input)?;
    skip_whitespace(input);

    // Determine if this is file-based (`;`) or inline (`{ ... }`)
    let source = if literal_str(";").parse_next(input).is_ok() {
        ModuleSource::File
    } else {
        // Inline module: parse definitions inside `{ ... }`
        let definitions =
            delimited(literal_str("{"), parse_definitions, literal_str("}")).parse_next(input)?;
        ModuleSource::Inline(definitions)
    };

    let span = crate::input::span_from(&start_pos, &input.state.pos);

    Ok(ModuleDecl {
        name: name.into(),
        visibility,
        source,
        span,
    })
}

/// Parse an identifier.
fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    crate::parse_utils::identifier(input)
}

/// Parse an identifier and return it with its source span.
fn identifier_with_span<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, Span)> {
    crate::parse_utils::identifier_with_span(input)
}

/// Parse a string literal token.
fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace(input);
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

/// Parse definitions inside an inline module.
fn parse_definitions(input: &mut ParseInput) -> ModalResult<Vec<Definition>> {
    let mut definitions = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        // Check for closing brace or EOF
        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        if starts_with_keyword(input, "role") {
            definitions.push(parse_role_definition(input)?);
            continue;
        }

        if starts_with_keyword(input, "capability") {
            definitions.push(parse_capability_definition(input)?);
            continue;
        }

        if starts_with_keyword(input, "proxy") {
            definitions.push(parse_proxy_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "interface") {
            definitions.push(parse_interface_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "impl") {
            definitions.push(parse_impl_definition(input)?);
            continue;
        }

        if starts_with_builtin_fn(input) {
            definitions.push(parse_builtin_fn_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "fn") {
            definitions.push(parse_fn_definition(input)?);
            continue;
        }

        if starts_with_unsupported_inline_definition(input) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }

        skip_unknown_definition(input);

        if input.input.starts_with(";") {
            let _ = input.input.next_slice(1);
            input.state.advance(';');
        }
    }

    Ok(definitions)
}

fn parse_capability_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;

    // Parse optional visibility modifier before "capability" keyword
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);

    let _ = keyword("capability").parse_next(input)?;
    skip_whitespace(input);
    let name = identifier(input)?;
    skip_whitespace(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace(input);
    let effect = parse_effect_type(input)?;
    skip_whitespace(input);
    let _ = literal_str("(").parse_next(input)?;
    let params = parse_parameter_list(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let return_type = parse_optional_return_type(input)?;
    skip_whitespace_and_comments(input);

    let constraints = if starts_with_keyword(input, "where") {
        parse_constraint_list(input)?
    } else {
        Vec::new()
    };

    Ok(Definition::Capability(CapabilityDef {
        visibility,
        name: name.into(),
        effect,
        params,
        return_type,
        constraints,
        target_provider: None,
        target_action: None,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_role_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;

    let _ = keyword("role").parse_next(input)?;
    skip_whitespace(input);
    let name = identifier(input)?;
    skip_whitespace(input);
    let _ = literal_str("{").parse_next(input)?;

    skip_whitespace_and_comments(input);
    let capabilities = parse_capabilities_clause(input)?;

    skip_whitespace_and_comments(input);
    consume_optional_comma(input);
    skip_whitespace_and_comments(input);
    let obligations = if starts_with_keyword(input, "obligations") {
        let obligations = parse_obligations_clause(input)?;
        skip_whitespace_and_comments(input);
        consume_optional_comma(input);
        obligations
    } else {
        Vec::new()
    };

    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;

    Ok(Definition::Role(RoleDef {
        name: name.into(),
        capabilities,
        obligations,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_interface_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);
    let _ = keyword("interface").parse_next(input)?;
    skip_whitespace(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let type_params = parse_optional_type_parameter_names(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut associated_types = Vec::new();
    let mut methods = Vec::new();
    while !input.input.starts_with("}") {
        if starts_with_keyword(input, "type") {
            associated_types.push(parse_associated_type_decl(input)?);
        } else {
            methods.push(parse_interface_method_signature(input)?);
        }
        skip_whitespace_and_comments(input);
        consume_optional_comma(input);
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str("}").parse_next(input)?;

    Ok(Definition::Interface(InterfaceDef {
        visibility,
        name: name.into(),
        type_params,
        associated_types,
        methods,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_associated_type_decl(input: &mut ParseInput) -> ModalResult<AssociatedTypeDecl> {
    let start = input.state.pos;
    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(";").parse_next(input)?;
    Ok(AssociatedTypeDecl {
        name: name.into(),
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_interface_method_signature(input: &mut ParseInput) -> ModalResult<InterfaceMethodSig> {
    let start = input.state.pos;
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();
    if literal_str(")").parse_next(input).is_err() {
        params.push(parse_surface_type(input)?);
        skip_whitespace_and_comments(input);
        while literal_str(",").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            params.push(parse_surface_type(input)?);
            skip_whitespace_and_comments(input);
        }
        let _ = literal_str(")").parse_next(input)?;
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("->").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let return_type = parse_surface_type(input)?;

    Ok(InterfaceMethodSig {
        name: name.into(),
        params,
        return_type,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_impl_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);
    let _ = keyword("impl").parse_next(input)?;
    skip_whitespace(input);
    let type_params = parse_optional_type_parameter_names(input)?;
    skip_whitespace_and_comments(input);
    let interface = identifier(input)?;
    skip_whitespace_and_comments(input);
    let type_args = parse_optional_type_arguments(input)?;
    skip_whitespace_and_comments(input);
    let where_bounds = if starts_with_keyword(input, "where") {
        parse_where_bounds(input)?
    } else {
        Vec::new()
    };
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut associated_type_bindings = Vec::new();
    let mut methods = Vec::new();
    while !input.input.starts_with("}") {
        if starts_with_keyword(input, "type") {
            associated_type_bindings.push(parse_associated_type_binding(input)?);
        } else {
            methods.push(parse_impl_method_definition(input)?);
        }
        skip_whitespace_and_comments(input);
        consume_optional_comma(input);
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str("}").parse_next(input)?;

    Ok(Definition::Impl(ImplDef {
        visibility,
        interface: interface.into(),
        type_params,
        type_args,
        where_bounds,
        associated_type_bindings,
        methods,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_where_bounds(input: &mut ParseInput) -> ModalResult<Vec<WhereBound>> {
    let _ = keyword("where").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut bounds = Vec::new();
    loop {
        let start = input.state.pos;
        let param = identifier(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let bound = identifier(input)?;
        bounds.push(WhereBound {
            param: param.into(),
            bound: bound.into(),
            span: crate::input::span_from(&start, &input.state.pos),
        });
        skip_whitespace_and_comments(input);
        if consume_comma_separator(input) {
            continue;
        }
        break;
    }
    Ok(bounds)
}

fn parse_associated_type_binding(input: &mut ParseInput) -> ModalResult<AssociatedTypeBinding> {
    let start = input.state.pos;
    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let ty = parse_surface_type(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(";").parse_next(input)?;
    Ok(AssociatedTypeBinding {
        name: name.into(),
        ty,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_impl_method_definition(input: &mut ParseInput) -> ModalResult<ImplMethodDef> {
    let start = input.state.pos;
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();
    if literal_str(")").parse_next(input).is_err() {
        params.push(identifier(input)?.into());
        skip_whitespace_and_comments(input);
        while literal_str(",").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            params.push(identifier(input)?.into());
            skip_whitespace_and_comments(input);
        }
        let _ = literal_str(")").parse_next(input)?;
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let body = expr(input)?;

    Ok(ImplMethodDef {
        name: name.into(),
        params,
        body,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_optional_type_parameter_names(input: &mut ParseInput) -> ModalResult<Vec<Box<str>>> {
    if !input.input.starts_with("<") {
        return Ok(Vec::new());
    }

    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut params = Vec::new();

    loop {
        let name = identifier(input)?;
        params.push(name.into());
        skip_whitespace_and_comments(input);

        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            skip_whitespace_and_comments(input);
            continue;
        }

        let _ = literal_str(">").parse_next(input)?;
        break;
    }

    Ok(params)
}

fn parse_required_type_arguments(input: &mut ParseInput) -> ModalResult<Vec<Type>> {
    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut args = Vec::new();

    loop {
        args.push(parse_surface_type(input)?);
        skip_whitespace_and_comments(input);

        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
            skip_whitespace_and_comments(input);
            continue;
        }

        let _ = literal_str(">").parse_next(input)?;
        break;
    }

    Ok(args)
}

fn parse_optional_type_arguments(input: &mut ParseInput) -> ModalResult<Vec<Type>> {
    if !input.input.starts_with("<") {
        return Ok(Vec::new());
    }

    parse_required_type_arguments(input)
}

fn parse_effect_type(input: &mut ParseInput) -> ModalResult<EffectType> {
    if keyword("observe").parse_next(input).is_ok() {
        Ok(EffectType::Observe)
    } else if keyword("read").parse_next(input).is_ok() {
        Ok(EffectType::Read)
    } else if keyword("analyze").parse_next(input).is_ok() {
        Ok(EffectType::Analyze)
    } else if keyword("decide").parse_next(input).is_ok() {
        Ok(EffectType::Decide)
    } else if keyword("act").parse_next(input).is_ok() {
        Ok(EffectType::Act)
    } else if keyword("write").parse_next(input).is_ok() {
        Ok(EffectType::Write)
    } else if keyword("external").parse_next(input).is_ok() {
        Ok(EffectType::External)
    } else if keyword("epistemic").parse_next(input).is_ok() {
        Ok(EffectType::Epistemic)
    } else if keyword("deliberative").parse_next(input).is_ok() {
        Ok(EffectType::Deliberative)
    } else if keyword("evaluative").parse_next(input).is_ok() {
        Ok(EffectType::Evaluative)
    } else if keyword("operational").parse_next(input).is_ok() {
        Ok(EffectType::Operational)
    } else {
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

fn parse_parameter_list(input: &mut ParseInput) -> ModalResult<Vec<Param>> {
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();

    if input.input.starts_with(")") {
        return Ok(params);
    }

    loop {
        let name = identifier(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let ty = parse_surface_type(input)?;

        params.push(Param {
            name: name.into(),
            ty,
        });

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    Ok(params)
}

fn parse_constraint_list(input: &mut ParseInput) -> ModalResult<Vec<Constraint>> {
    let _ = keyword("where").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut constraints = Vec::new();

    loop {
        constraints.push(parse_constraint(input)?);

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    Ok(constraints)
}

fn parse_constraint(input: &mut ParseInput) -> ModalResult<Constraint> {
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let args = delimited(
        literal_str("("),
        parse_constraint_arguments,
        literal_str(")"),
    )
    .parse_next(input)?;

    Ok(Constraint {
        predicate: Predicate {
            name: name.into(),
            args,
        },
    })
}

fn parse_constraint_arguments(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let mut args = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with(")") {
            break;
        }

        args.push(expr(input)?);

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    Ok(args)
}

fn parse_optional_return_type(input: &mut ParseInput) -> ModalResult<Option<Type>> {
    if !starts_with_keyword(input, "returns") {
        return Ok(None);
    }

    let _ = keyword("returns").parse_next(input)?;
    skip_whitespace_and_comments(input);

    parse_surface_type(input).map(Some)
}

fn parse_surface_type(input: &mut ParseInput) -> ModalResult<Type> {
    skip_whitespace_and_comments(input);

    // Parse Fn(T1, T2) -> T3 type syntax
    if starts_with_keyword(input, "Fn") {
        let _ = keyword("Fn").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("(").parse_next(input)?;
        let mut params = Vec::new();
        skip_whitespace_and_comments(input);
        if !input.input.starts_with(")") {
            params.push(parse_surface_type(input)?);
            loop {
                if !consume_comma_separator(input) {
                    break;
                }
                params.push(parse_surface_type(input)?);
            }
        }
        let _ = literal_str(")").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("->").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let ret = parse_surface_type(input)?;
        return Ok(Type::Fn(params, Box::new(ret)));
    }

    if starts_with_keyword(input, "capability") {
        let _ = keyword("capability").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let name = identifier(input)?;
        return Ok(Type::Capability(name.into()));
    }

    if input.input.starts_with("[") {
        let _ = literal_str("[").parse_next(input)?;
        let inner = parse_surface_type(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("]").parse_next(input)?;
        return Ok(Type::List(Box::new(inner)));
    }

    if input.input.starts_with("{") {
        let _ = literal_str("{").parse_next(input)?;
        skip_whitespace_and_comments(input);

        let mut fields = Vec::new();

        if input.input.starts_with("}") {
            let _ = literal_str("}").parse_next(input)?;
            return Ok(Type::Record(fields));
        }

        loop {
            let field_name = identifier(input)?;
            skip_whitespace_and_comments(input);
            let _ = literal_str(":").parse_next(input)?;
            skip_whitespace_and_comments(input);
            let field_type = parse_surface_type(input)?;
            fields.push((field_name.into(), field_type));

            if consume_comma_separator(input) {
                continue;
            }

            break;
        }

        let _ = literal_str("}").parse_next(input)?;
        return Ok(Type::Record(fields));
    }

    let name = identifier(input)?;
    skip_whitespace_and_comments(input);

    let mut base = if input.input.starts_with("<") {
        let args = parse_required_type_arguments(input)?;
        Type::Constructor {
            name: name.into(),
            args,
        }
    } else {
        Type::Name(name.into())
    };

    while input.input.starts_with("::") {
        let _ = literal_str("::").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let assoc_name = identifier(input)?;
        base = Type::Associated {
            base: Box::new(base),
            name: assoc_name.into(),
        };
    }

    Ok(base)
}

fn parse_obligations_clause(input: &mut ParseInput) -> ModalResult<Vec<Box<str>>> {
    let _ = keyword("obligations").parse_next(input)?;
    skip_whitespace(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace(input);

    parse_name_list(input)
}

fn parse_name_list(input: &mut ParseInput) -> ModalResult<Vec<Box<str>>> {
    let _ = literal_str("[").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut names = Vec::new();

    if input.input.starts_with("]") {
        let _ = literal_str("]").parse_next(input)?;
        return Ok(names);
    }

    loop {
        let name = identifier(input)?;
        names.push(name.into());

        if consume_comma_separator(input) {
            continue;
        }

        break;
    }

    let _ = literal_str("]").parse_next(input)?;
    Ok(names)
}

fn starts_with_keyword(input: &ParseInput, word: &str) -> bool {
    if !input.input.starts_with(word) {
        return false;
    }

    let after = &input.input[word.len()..];
    after
        .chars()
        .next()
        .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
}

fn starts_with_visible_keyword(input: &ParseInput, word: &str) -> bool {
    if starts_with_keyword(input, word) {
        return true;
    }

    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(Visibility::Inherited) | Err(_) => false,
        Ok(_) => {
            skip_whitespace(&mut lookahead);
            starts_with_keyword(&lookahead, word)
        }
    }
}

/// Check if input starts with `[pub] builtin fn` pattern.
fn starts_with_builtin_fn(input: &ParseInput) -> bool {
    // Check for "builtin fn" directly
    if starts_with_keyword(input, "builtin") {
        let rest = skip_ws_in(&input.input["builtin".len()..]);
        if starts_with_keyword_from(rest, "fn") {
            return true;
        }
    }

    // Check for "[visibility] builtin fn"
    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(Visibility::Inherited) | Err(_) => false,
        Ok(_) => {
            skip_whitespace(&mut lookahead);
            // After visibility, check for "builtin fn"
            if starts_with_keyword(&lookahead, "builtin") {
                let rest = skip_ws_in(&lookahead.input["builtin".len()..]);
                starts_with_keyword_from(rest, "fn")
            } else {
                false
            }
        }
    }
}

fn starts_with_keyword_from(src: &str, word: &str) -> bool {
    if !src.starts_with(word) {
        return false;
    }
    let after = &src[word.len()..];
    after
        .chars()
        .next()
        .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
}

fn skip_ws_in(s: &str) -> &str {
    let mut len = 0;
    for c in s.chars() {
        if c.is_ascii_whitespace() {
            len += c.len_utf8();
        } else {
            break;
        }
    }
    &s[len..]
}

fn starts_with_unsupported_inline_definition(input: &ParseInput) -> bool {
    [
        "pub",
        "workflow",
        "policy",
        "type",
        "datatype",
        "memory",
        "mod",
        "interface",
        "impl",
    ]
    .into_iter()
    .any(|keyword| starts_with_keyword(input, keyword))
}

fn consume_optional_comma(input: &mut ParseInput) {
    if input.input.starts_with(",") {
        let _ = input.input.next_slice(1);
        input.state.advance(',');
    }
}

fn consume_comma_separator(input: &mut ParseInput) -> bool {
    skip_whitespace_and_comments(input);

    if !input.input.starts_with(",") {
        return false;
    }

    let _ = input.input.next_slice(1);
    input.state.advance(',');
    skip_whitespace_and_comments(input);
    true
}

fn skip_unknown_definition(input: &mut ParseInput) {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut consumed_any = false;

    while !input.input.is_empty() {
        if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 && consumed_any {
            skip_whitespace_and_comments(input);
            if starts_with_keyword(input, "role")
                || starts_with_keyword(input, "capability")
                || starts_with_unsupported_inline_definition(input)
            {
                break;
            }
        }

        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && (input.input.starts_with(";") || input.input.starts_with("}"))
        {
            break;
        }

        let Some(c) = input.input.next_token() else {
            break;
        };
        input.state.advance(c);
        consumed_any = true;

        match c {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
    }
}

/// Skip whitespace (simple version for use in this module).
fn skip_whitespace(input: &mut ParseInput) {
    while input
        .input
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_whitespace())
    {
        let Some(c) = input.input.next_token() else {
            break;
        };
        input.state.advance(c);
    }
}

/// Parse a proxy definition.
///
/// Syntax: `proxy <name> handles role(<role_name>) [observes cap, ...] [receives cap, ...] { <body> }`
pub fn proxy_def(input: &mut ParseInput) -> ModalResult<ProxyDef> {
    parse_proxy_definition_inner(input)
}

/// Internal implementation of proxy definition parsing.
fn parse_proxy_definition_inner(input: &mut ParseInput) -> ModalResult<ProxyDef> {
    let start_pos = input.state.pos;

    // Parse optional visibility modifier
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);

    let _ = keyword("proxy").parse_next(input)?;
    skip_whitespace(input);

    // Parse proxy name
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);

    // Parse "handles role(<role_name>)"
    let _ = keyword("handles").parse_next(input)?;
    skip_whitespace(input);
    let _ = keyword("role").parse_next(input)?;
    skip_whitespace(input);
    let _ = literal_str("(").parse_next(input)?;
    let role = identifier(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse optional observes clause
    let observes = if starts_with_keyword(input, "observes") {
        let caps = parse_observes_clause(input)?;
        skip_whitespace_and_comments(input);
        caps
    } else {
        Vec::new()
    };

    // Parse optional receives clause
    let receives = if starts_with_keyword(input, "receives") {
        let caps = parse_receives_clause(input)?;
        skip_whitespace_and_comments(input);
        caps
    } else {
        Vec::new()
    };

    // Parse body
    let body = delimited(literal_str("{"), parse_proxy_body, literal_str("}")).parse_next(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);

    Ok(ProxyDef {
        visibility,
        name: name.into(),
        role: role.into(),
        observes,
        receives,
        body,
        span,
    })
}

/// Parse proxy definition and wrap it in Definition enum.
fn parse_proxy_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    parse_proxy_definition_inner(input).map(Definition::Proxy)
}

/// Parse a clause like `observes cap1:channel1, cap2:channel2` or `receives cap1, cap2`.
fn parse_observes_clause(input: &mut ParseInput) -> ModalResult<Vec<CapabilityRef>> {
    let _ = keyword("observes").parse_next(input)?;
    skip_whitespace(input);
    parse_capability_refs(input)
}

/// Parse a clause like `receives cap1, cap2`.
fn parse_receives_clause(input: &mut ParseInput) -> ModalResult<Vec<CapabilityRef>> {
    let _ = keyword("receives").parse_next(input)?;
    skip_whitespace(input);
    parse_capability_refs(input)
}

/// Parse comma-separated capability references.
fn parse_capability_refs(input: &mut ParseInput) -> ModalResult<Vec<CapabilityRef>> {
    let mut refs = Vec::new();

    // Parse first capability reference
    let cap_ref = parse_capability_ref(input)?;
    refs.push(cap_ref);

    // Parse additional comma-separated references
    loop {
        skip_whitespace_and_comments(input);
        if !input.input.starts_with(",") {
            break;
        }
        let _ = input.input.next_slice(1);
        input.state.advance(',');
        skip_whitespace_and_comments(input);

        let cap_ref = parse_capability_ref(input)?;
        refs.push(cap_ref);
    }

    Ok(refs)
}

/// Parse a single capability reference, optionally with channel: `name` or `name:channel`.
fn parse_capability_ref(input: &mut ParseInput) -> ModalResult<CapabilityRef> {
    let name = identifier(input)?;

    skip_whitespace(input);
    let channel = if input.input.starts_with(":") {
        let _ = input.input.next_slice(1);
        input.state.advance(':');
        skip_whitespace(input);
        let ch = identifier(input)?;
        Some(ch.into())
    } else {
        None
    };

    Ok(CapabilityRef {
        name: name.into(),
        channel,
    })
}

/// Parse the body of a proxy definition.
fn parse_proxy_body(input: &mut ParseInput) -> ModalResult<Workflow> {
    skip_whitespace_and_comments(input);

    // For now, we parse a workflow body using the workflow parser
    // but with some restrictions. We use a simple approach:
    // parse statements until we hit the closing brace.
    crate::parse_workflow::workflow(input)
}

/// Parse a yield expression for role delegation.
///
/// Syntax: `yield role(<role_name>) <expression> resume <var> : <Type> { <arms> }`
pub fn parse_yield(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("yield").parse_next(input)?;
    skip_whitespace(input);

    // Parse role(<role_name>)
    let _ = keyword("role").parse_next(input)?;
    skip_whitespace(input);
    let _ = literal_str("(").parse_next(input)?;
    let role = identifier(input)?;
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace(input);

    // Parse the expression to send
    let expr = expr(input)?;
    skip_whitespace_and_comments(input);

    // Parse resume clause: resume <var> : <Type>
    let _ = keyword("resume").parse_next(input)?;
    skip_whitespace(input);
    let resume_var = identifier(input)?;
    skip_whitespace(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace(input);
    let resume_type = parse_surface_type(input)?;
    skip_whitespace_and_comments(input);

    // Parse match arms
    let arms = delimited(literal_str("{"), parse_yield_arms, literal_str("}")).parse_next(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Yield {
        role: role.into(),
        expr,
        resume_var: resume_var.into(),
        resume_type,
        arms,
        span,
    })
}

/// Parse yield match arms.
fn parse_yield_arms(input: &mut ParseInput) -> ModalResult<Vec<YieldArm>> {
    let mut arms = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        let arm_start = input.state.pos;

        // Parse pattern
        let pattern = crate::parse_pattern::pattern(input)?;
        skip_whitespace(input);

        // Parse =>
        let _ = literal_str("=>").parse_next(input)?;
        skip_whitespace(input);

        // Parse body (either a block or single statement)
        let body = crate::parse_workflow::parse_single_stmt_or_block(input)?;

        let arm_span = crate::input::span_from(&arm_start, &input.state.pos);
        arms.push(YieldArm {
            pattern,
            body,
            span: arm_span,
        });

        // Optional comma
        skip_whitespace_and_comments(input);
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        }
    }

    Ok(arms)
}

/// Parse a resume statement.
///
/// Syntax: `resume <expression> : <Type>`
pub fn parse_resume(input: &mut ParseInput) -> ModalResult<Workflow> {
    let start_pos = input.state.pos;

    let _ = keyword("resume").parse_next(input)?;
    skip_whitespace(input);

    // Parse the expression
    let expr = expr(input)?;
    skip_whitespace(input);

    // Parse : Type
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace(input);
    let ty = parse_surface_type(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);

    Ok(Workflow::Resume { expr, ty, span })
}

// ---------------------------------------------------------------------------
// Pure function definition parser
// ---------------------------------------------------------------------------

/// Parse a pure function definition.
///
/// Syntax: `[pub] fn <name>[<T, U>](<params>) [-> <return_type>] [requires: ...] [ensures: ...] { <body> }`
pub fn parse_fn_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;

    // Parse optional visibility modifier
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);

    // Parse "fn" keyword
    let _ = keyword("fn").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse function name
    let name = identifier(input)?;
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
    let name = identifier(input)?;
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

/// Parse a complete `.ash` source file into a `ModuleFile`.
pub fn module_file(input: &mut ParseInput) -> ModalResult<crate::surface::ModuleFile> {
    let start_pos = input.state.pos;
    let mut definitions = Vec::new();
    let mut module_decls = Vec::new();
    let mut workflow = None;

    loop {
        skip_whitespace_and_comments(input);
        if input.input.is_empty() {
            break;
        }

        if starts_with_visible_keyword(input, "workflow") {
            let w = workflow_def(input)?;
            workflow = Some(w);
            continue;
        }

        if starts_with_visible_keyword(input, "mod") {
            let decl = parse_module_decl(input)?;
            module_decls.push(decl);
            continue;
        }

        if starts_with_keyword(input, "role") {
            definitions.push(parse_role_definition(input)?);
            continue;
        }

        if starts_with_keyword(input, "capability") {
            definitions.push(parse_capability_definition(input)?);
            continue;
        }

        if starts_with_keyword(input, "proxy") {
            definitions.push(parse_proxy_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "interface") {
            definitions.push(parse_interface_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "impl") {
            definitions.push(parse_impl_definition(input)?);
            continue;
        }

        if starts_with_builtin_fn(input) {
            definitions.push(parse_builtin_fn_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "fn") {
            definitions.push(parse_fn_definition(input)?);
            continue;
        }

        // Unknown item: try to skip past it to avoid infinite loop
        skip_unknown_definition(input);
        if input.input.is_empty() {
            break;
        }
        if input.input.starts_with(";") {
            let _ = input.input.next_slice(1);
            input.state.advance(';');
        }
    }

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(crate::surface::ModuleFile {
        definitions,
        module_decls,
        workflow,
        span,
        comments: crate::parse_utils::CommentTable::default(),
        path: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;
    use crate::surface::{
        Constraint, Definition, EffectType, Expr, Literal, Predicate, Visibility,
    };

    /// Test helper to create a ParseInput for testing
    fn test_input(s: &str) -> ParseInput<'_> {
        new_input(s)
    }

    fn inline_module_with_unknown_item(body_after_unknown: &str) -> String {
        format!("mod governance {{ extension custom {{ enabled: true }} {body_after_unknown} }}")
    }

    fn assert_inline_module_rejects_after_unknown_item(
        body_after_unknown: &str,
        item_description: &str,
    ) {
        let source = inline_module_with_unknown_item(body_after_unknown);
        let mut input = test_input(&source);

        let result = parse_module_decl(&mut input);

        match result {
            Err(_) => {}
            Ok(decl) => panic!(
                "Expected parse to fail instead of silently skipping an unsupported {item_description} after unknown-item recovery, but parsed definitions: {:?}",
                decl.definitions()
            ),
        }
    }

    // ========================================================================
    // File-based Module Tests
    // ========================================================================

    #[test]
    fn test_parse_mod_foo_semicolon() {
        // Test: `mod foo;` → file-based module
        let mut input = test_input("mod foo;");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Inherited);
        assert!(decl.is_file_based());
        assert!(!decl.is_inline());
        assert!(matches!(decl.source, ModuleSource::File));
    }

    #[test]
    fn test_parse_pub_mod_foo_semicolon() {
        // Test: `pub mod foo;` → public file-based module
        let mut input = test_input("pub mod foo;");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Public);
        assert!(decl.is_file_based());
        assert!(!decl.is_inline());
    }

    #[test]
    fn test_parse_pub_crate_mod_foo_semicolon() {
        // Test: `pub(crate) mod foo;` → crate-visible file-based module
        let mut input = test_input("pub(crate) mod foo;");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Crate);
        assert!(decl.is_file_based());
    }

    // ========================================================================
    // Inline Module Tests
    // ========================================================================

    #[test]
    fn test_parse_inline_module_empty() {
        // Test: `mod foo {}` → empty inline module
        let mut input = test_input("mod foo {}");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Inherited);
        assert!(!decl.is_file_based());
        assert!(decl.is_inline());

        let defs = decl
            .definitions()
            .expect("inline module should have definitions");
        assert!(defs.is_empty());
    }

    #[test]
    fn test_parse_inline_module_with_capability() {
        let mut input =
            test_input("mod foo { capability approve: decide() where requires_mfa(); }");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert!(decl.is_inline());

        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert_eq!(capability.name.as_ref(), "approve");
        assert_eq!(capability.effect, EffectType::Decide);
        assert!(matches!(
            &capability.constraints[..],
            [Constraint {
                predicate: Predicate { name, args }
            }] if name.as_ref() == "requires_mfa" && args.is_empty()
        ));
    }

    #[test]
    fn test_parse_inline_module_with_capability_constraint_arguments() {
        let mut input =
            test_input("mod foo { capability approve: decide() where requires_region(\"EU\"); }");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert!(matches!(
            &capability.constraints[..],
            [Constraint {
                predicate: Predicate { name, args }
            }] if name.as_ref() == "requires_region"
                && matches!(&args[..], [Expr::Literal(Literal::String(region))] if region.as_ref() == "EU")
        ));
    }

    #[test]
    fn test_parse_inline_module_preserves_capability_signature_metadata() {
        let mut input = test_input(
            "mod foo { capability approve: decide(user: User, scopes: [Scope]) returns Bool where requires_mfa(); }",
        );
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert_eq!(capability.params.len(), 2);
        assert!(matches!(
            &capability.params[..],
            [
                Param { name: user_name, ty: Type::Name(user_type) },
                Param { name: scopes_name, ty: Type::List(inner) }
            ] if user_name.as_ref() == "user"
                && user_type.as_ref() == "User"
                && scopes_name.as_ref() == "scopes"
                && matches!(inner.as_ref(), Type::Name(scope_type) if scope_type.as_ref() == "Scope")
        ));
        assert!(matches!(
            capability.return_type.as_ref(),
            Some(Type::Name(name)) if name.as_ref() == "Bool"
        ));
    }

    #[test]
    fn test_parse_inline_module_with_capability_returns_and_constraint_arguments() {
        let mut input = test_input(
            "mod foo { capability approve: decide() returns Bool where requires_region(\"EU\"); }",
        );
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert!(matches!(
            &capability.constraints[..],
            [Constraint {
                predicate: Predicate { name, args }
            }] if name.as_ref() == "requires_region"
                && matches!(&args[..], [Expr::Literal(Literal::String(region))] if region.as_ref() == "EU")
        ));
    }

    #[test]
    fn test_parse_inline_module_rejects_invalid_constraint_predicate_identifier() {
        let mut input =
            test_input("mod foo { capability approve: decide() where 1requires_mfa(); }");

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail for a non-canonical predicate identifier"
        );
    }

    #[test]
    fn test_parse_inline_module_with_role_definition() {
        let mut input = test_input(
            "mod governance { role reviewer { capabilities: [approve, review], obligations: [check_tests, audit_log] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        assert_eq!(definitions.len(), 1);

        let Definition::Role(role) = &definitions[0] else {
            panic!("expected first definition to be a role: {definitions:?}");
        };

        assert_eq!(role.name.as_ref(), "reviewer");
        assert_eq!(role.capabilities.len(), 2);
        assert_eq!(role.capabilities[0].capability.as_ref(), "approve");
        assert_eq!(role.capabilities[1].capability.as_ref(), "review");
        assert_eq!(role.obligations.len(), 2);
        assert_eq!(role.obligations[0].as_ref(), "check_tests");
        assert_eq!(role.obligations[1].as_ref(), "audit_log");
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_inline_workflow_before_role() {
        let mut input = test_input(
            "mod governance { workflow main { done } role reviewer { capabilities: [approve] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail instead of silently skipping unsupported inline workflow items"
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_inline_workflow_before_capability_and_role() {
        let mut input = test_input(
            "mod governance { workflow main { done } capability approve: decide() where requires_mfa(); role reviewer { capabilities: [approve] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected parse to fail instead of silently skipping unsupported inline workflow items"
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_workflow_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "workflow main { done } role reviewer { capabilities: [approve] }",
            "workflow",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_policy_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "policy approval: when true then permit role reviewer { capabilities: [approve] }",
            "policy",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_datatype_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "datatype review_state = Pending | Approved; role reviewer { capabilities: [approve] }",
            "datatype",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_visibility_qualified_item_after_unknown_item() {
        assert_inline_module_rejects_after_unknown_item(
            "pub capability approve: decide() role reviewer { capabilities: [approve] }",
            "visibility-qualified item",
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_unsupported_canonical_datatype_definition() {
        let mut input = test_input(
            "mod governance { datatype review_state = Pending | Approved; role reviewer { capabilities: [approve] } }",
        );

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected inline modules to reject unsupported canonical datatype definitions explicitly"
        );
    }

    #[test]
    fn test_parse_inline_module_rejects_visibility_qualified_capabilities_until_supported() {
        let mut input = test_input("mod governance { pub capability approve: decide() }");

        let result = parse_module_decl(&mut input);

        assert!(
            result.is_err(),
            "Expected inline modules to reject visibility-qualified capability items explicitly until they are supported"
        );
    }

    #[test]
    fn test_parse_pub_inline_module() {
        // Test: `pub mod foo {}` → public inline module
        let mut input = test_input("pub mod foo {}");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert_eq!(decl.visibility, Visibility::Public);
        assert!(decl.is_inline());
    }

    // ========================================================================
    // Whitespace and Formatting Tests
    // ========================================================================

    #[test]
    fn test_parse_mod_with_whitespace() {
        // Test parsing with extra whitespace
        let mut input = test_input("  mod   foo   ;  ");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert!(decl.is_file_based());
    }

    #[test]
    fn test_parse_inline_mod_with_whitespace() {
        // Test parsing inline module with extra whitespace
        let mut input = test_input("  mod   foo   {   }  ");
        let result = parse_module_decl(&mut input);

        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let decl = result.unwrap();
        assert_eq!(decl.name.as_ref(), "foo");
        assert!(decl.is_inline());
    }

    #[test]
    fn test_parse_inline_module_definition_spans_track_comments_and_indentation() {
        let mut input = test_input(
            "mod foo {\n  -- comment before capability\n  capability approve: decide()\n}",
        );

        let decl = parse_module_decl(&mut input).expect("inline module should parse");
        let definitions = decl
            .definitions()
            .expect("inline module should expose parsed definitions");

        let Definition::Capability(capability) = &definitions[0] else {
            panic!("expected first definition to be a capability: {definitions:?}");
        };

        assert_eq!(capability.span.line, 3);
        assert_eq!(capability.span.column, 3);
    }

    // =========================================================================
    // TASK-674: act block expression parsing tests
    // =========================================================================

    #[test]
    fn test_parse_act_block_simple_return() {
        let mut input = test_input("{ act { ret 42; } }");
        let result = parse_fn_body(&mut input);
        assert!(result.is_ok(), "parse failed: {:?}", result);
        let expr = result.unwrap();
        assert!(
            matches!(expr, Expr::Block { ref tail_expr, .. } if tail_expr.is_some()),
            "expected a block with a tail expression, got: {:?}",
            expr
        );
    }

    #[test]
    fn test_parse_act_block_bind_and_return() {
        let mut input = test_input("{ act { x = 42; ret x; } }");
        let result = parse_fn_body(&mut input);
        assert!(result.is_ok(), "parse failed: {:?}", result);
    }

    #[test]
    fn test_parse_act_block_nested_calls() {
        let mut input = test_input("{ act { result = read_file(path); ret result; } }");
        let result = parse_fn_body(&mut input);
        assert!(result.is_ok(), "parse failed: {:?}", result);
    }

    #[test]
    fn test_parse_act_block_empty() {
        let mut input = test_input("{ act {} }");
        let result = parse_fn_body(&mut input);
        assert!(result.is_ok(), "parse failed: {:?}", result);
    }
}
