//! Module declaration parser for the Ash language.
//!
//! This module provides parsers for module declarations, supporting both
//! file-based modules (`mod foo;`) and inline modules (`mod foo { ... }`).

use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::combinators::keyword;
use crate::input::ParseInput;
use crate::module::{ModuleDecl, ModuleSource};
use crate::parse_expr::{expr, parse_fn_expr_body_pub, parse_if_let_expr};
use crate::parse_utils::{
    parse_kind_annotation, skip_whitespace_and_comments, starts_with_kind_syntax,
};
use crate::parse_visibility;
use crate::parse_workflow::{parse_capabilities_clause, workflow_def};
use crate::surface::{
    AssociatedFamilyDecreases, AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind,
    BlockStmt, BuiltinFnDef, CapabilityDef, CapabilityImplementationDef,
    CapabilityImplementationDependency, CapabilityImplementationDependencyKind,
    CapabilityImplementationOperation, CapabilityInterfaceDef, CapabilityOperationMode,
    CapabilityOperationSig, CapabilityRef, Constraint, Contract, DataKindDef, Definition,
    DomainConstructor, DomainField, DomainSlot, EffectType, Expr, FnDef, ImplDef, ImplMethodDef,
    InterfaceDef, InterfaceEvidenceConstraint, InterfaceMethodSig, InterfaceTypeParam, LawDef,
    MatchArm, Name, Param, Pattern, Predicate, ProofBody, ProofDef, PropositionClause,
    PropositionClauseKind, PropositionPredicateDecl, PropositionPredicateParam, PropositionTail,
    ProxyDef, ResourceField, ResourceTypeDef, RoleDef, SealedDomainDef, Type, TypeBody, TypeDef,
    TypeField, TypeFnDecreases, TypeFnDef, TypeFnEquation, TypeFnParam, TypeParam, TypePattern,
    VariantDef, VariantPayload, Visibility, WhereBound, Workflow, YieldArm,
};
use crate::token::Span;

#[derive(Clone, Copy)]
enum TypeHolePolicy {
    Disallow,
    Allow,
}

fn starts_with_standalone_type_hole(input: &ParseInput<'_>) -> bool {
    let mut chars = input.input.as_ref().chars();
    matches!(chars.next(), Some('_'))
        && !chars
            .next()
            .is_some_and(crate::parse_utils::is_identifier_continue)
}

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

/// Parse a callable name.
///
/// Ash normally rejects keywords as identifiers, but a small set of contextual
/// keywords are allowed as function names to support standard-library helpers
/// like `then` and `guard` from SPEC-047.
fn callable_name<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    let checkpoint = input.clone();
    if let Ok(name) = identifier(input) {
        return Ok(name);
    }
    *input = checkpoint;

    for keyword_name in ["then", "guard"] {
        let checkpoint = input.clone();
        if keyword(keyword_name).parse_next(input).is_ok() {
            return Ok(keyword_name);
        }
        *input = checkpoint;
    }

    Err(winnow::error::ErrMode::Backtrack(
        winnow::error::ContextError::new(),
    ))
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

        if starts_with_visible_resource_type(input) {
            definitions.push(parse_resource_type_definition(input)?);
            continue;
        }

        if starts_with_type_fn_definition(input) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }

        if starts_with_visible_keyword(input, "prop") {
            definitions.push(parse_proposition_predicate_decl(input)?);
            continue;
        }

        if starts_with_data_kind(input) {
            definitions.push(parse_data_kind_definition(input)?);
            continue;
        }

        if starts_with_type_definition(input) {
            definitions.push(parse_type_definition(input)?);
            continue;
        }
        // `starts_with_unsupported_inline_definition` check below.
        // We do NOT add `starts_with_sealed_domain` here; it must
        // fall through to the unsupported-inline guard.

        if starts_with_visible_capability_interface(input) {
            definitions.push(parse_capability_interface_definition(input)?);
            continue;
        }

        if starts_with_visible_capability_impl(input) {
            definitions.push(parse_capability_implementation_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "capability") {
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

        if starts_with_keyword(input, "law") {
            definitions.push(parse_law_definition_as_definition(input)?);
            continue;
        }

        if starts_with_keyword(input, "proof") {
            definitions.push(parse_proof_definition_as_definition(input)?);
            continue;
        }

        if starts_with_unsupported_inline_definition(input) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }

        if starts_with_unsupported_promotion_surface(input) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }

        if starts_with_unsupported_proposition_surface(input) {
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

fn parse_resource_type_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("resource").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut fields = Vec::new();
    if !input.input.starts_with("}") {
        loop {
            fields.push(parse_resource_field(input)?);
            skip_whitespace_and_comments(input);
            if consume_comma_separator(input) {
                continue;
            }
            break;
        }
    }

    let _ = literal_str("}").parse_next(input)?;

    Ok(Definition::ResourceType(ResourceTypeDef {
        visibility,
        name: name.into(),
        fields,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_resource_field(input: &mut ParseInput) -> ModalResult<ResourceField> {
    let start_pos = input.state.pos;
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let ty = parse_surface_type(input)?;

    Ok(ResourceField {
        name: name.into(),
        ty,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    })
}

fn parse_type_fn_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);

    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("fn").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let (name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);

    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut params = Vec::new();
    if input.input.starts_with(")") {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }
    loop {
        params.push(parse_type_fn_param(input)?);
        skip_whitespace_and_comments(input);
        if consume_comma_separator(input) {
            continue;
        }
        break;
    }
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("->").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let return_type = parse_surface_type(input)?;
    let header_span = crate::input::span_from(&start_pos, &input.state.pos);
    skip_whitespace_and_comments(input);

    let decreases = if starts_with_keyword(input, "decreases") {
        let decreases_start = input.state.pos;
        let _ = keyword("decreases").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let (param, _) = identifier_with_span(input)?;
        let span = crate::input::span_from(&decreases_start, &input.state.pos);
        skip_whitespace_and_comments(input);
        Some(TypeFnDecreases {
            param: param.into(),
            span,
        })
    } else {
        None
    };

    let proposition_tail = if starts_with_keyword(input, "where") {
        Some(parse_proposition_tail(input)?)
    } else {
        None
    };
    skip_whitespace_and_comments(input);

    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut equations = Vec::new();
    while !input.input.starts_with("}") {
        equations.push(parse_type_fn_equation(input, name)?);
        skip_whitespace_and_comments(input);
    }
    let _ = literal_str("}").parse_next(input)?;

    Ok(Definition::TypeFn(TypeFnDef {
        visibility,
        name: name.into(),
        params,
        return_type,
        decreases,
        proposition_tail,
        equations,
        header_span,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_proposition_tail(input: &mut ParseInput) -> ModalResult<PropositionTail> {
    let tail_start = input.state.pos;
    let where_start = input.state.pos;
    let _ = keyword("where").parse_next(input)?;
    let where_span = crate::input::span_from(&where_start, &input.state.pos);
    skip_whitespace_and_comments(input);

    let mut clauses = Vec::new();
    clauses.push(parse_proposition_clause(input)?);
    skip_whitespace_and_comments(input);
    while consume_comma_separator(input) {
        clauses.push(parse_proposition_clause(input)?);
        skip_whitespace_and_comments(input);
    }

    Ok(PropositionTail {
        clauses,
        where_span,
        span: crate::input::span_from(&tail_start, &input.state.pos),
    })
}

fn parse_proposition_clause(input: &mut ParseInput) -> ModalResult<PropositionClause> {
    let clause_start = input.state.pos;
    let lhs = parse_surface_type(input)?;
    skip_whitespace_and_comments(input);

    if input.input.starts_with("==") {
        let op_start = input.state.pos;
        let _ = literal_str("==").parse_next(input)?;
        let op_span = crate::input::span_from(&op_start, &input.state.pos);
        skip_whitespace_and_comments(input);
        let rhs = parse_surface_type(input)?;
        return Ok(PropositionClause {
            kind: PropositionClauseKind::Equality { lhs, rhs, op_span },
            span: crate::input::span_from(&clause_start, &input.state.pos),
        });
    }

    if input.input.starts_with("!=") {
        let op_start = input.state.pos;
        let _ = literal_str("!=").parse_next(input)?;
        let op_span = crate::input::span_from(&op_start, &input.state.pos);
        skip_whitespace_and_comments(input);
        let rhs = parse_surface_type(input)?;
        return Ok(PropositionClause {
            kind: PropositionClauseKind::Disequality { lhs, rhs, op_span },
            span: crate::input::span_from(&clause_start, &input.state.pos),
        });
    }

    if input.input.starts_with(":") {
        let colon_start = input.state.pos;
        let _ = literal_str(":").parse_next(input)?;
        let colon_span = crate::input::span_from(&colon_start, &input.state.pos);
        skip_whitespace_and_comments(input);
        let interface = parse_surface_type(input)?;
        return Ok(PropositionClause {
            kind: PropositionClauseKind::InterfaceBound {
                subject: lhs,
                interface,
                colon_span,
            },
            span: crate::input::span_from(&clause_start, &input.state.pos),
        });
    }

    let Some((name, args)) = type_as_named_predicate(lhs) else {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    };
    let name_span = Span::new(
        clause_start.offset,
        clause_start.offset.saturating_add(name.len()),
        clause_start.line,
        clause_start.column,
    );
    Ok(PropositionClause {
        kind: PropositionClauseKind::NamedPredicate {
            name,
            name_span,
            args,
        },
        span: crate::input::span_from(&clause_start, &input.state.pos),
    })
}

fn type_as_named_predicate(ty: Type) -> Option<(Name, Vec<Type>)> {
    match ty {
        Type::Name(name) => Some((name, Vec::new())),
        Type::Constructor { name, args } => Some((name, args)),
        _ => None,
    }
}

fn parse_proposition_predicate_decl(input: &mut ParseInput) -> ModalResult<Definition> {
    let start = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("prop").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let (name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();
    if input.input.starts_with("<") {
        let _ = literal_str("<").parse_next(input)?;
        skip_whitespace_and_comments(input);
        if input.input.starts_with(">") {
            return Err(winnow::error::ErrMode::Cut(
                winnow::error::ContextError::new(),
            ));
        }
        loop {
            params.push(parse_proposition_predicate_param(input)?);
            skip_whitespace_and_comments(input);
            if consume_comma_separator(input) {
                continue;
            }
            break;
        }
        let _ = literal_str(">").parse_next(input)?;
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str(";").parse_next(input)?;
    Ok(Definition::PropositionPredicate(PropositionPredicateDecl {
        visibility,
        name: name.into(),
        params,
        span: crate::input::span_from(&start, &input.state.pos),
    }))
}

fn parse_proposition_predicate_param(
    input: &mut ParseInput,
) -> ModalResult<PropositionPredicateParam> {
    let start = input.state.pos;
    let (name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let (domain, kind) = if starts_with_kind_syntax(input) {
        let kind = parse_kind_annotation(input)?;
        (Type::Name(kind.kind.to_string().into()), Some(kind))
    } else {
        (parse_surface_type(input)?, None)
    };
    Ok(PropositionPredicateParam {
        name: name.into(),
        domain,
        kind,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_data_kind_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);

    let _ = keyword("data").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("kind").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let (name, _) = identifier_with_span(input)?;
    for ch in name.chars() {
        input.state.advance(ch);
    }
    skip_whitespace_and_comments(input);
    let _ = keyword("from").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let (source_adt, _) = identifier_with_span(input)?;
    for ch in source_adt.chars() {
        input.state.advance(ch);
    }
    skip_whitespace_and_comments(input);
    let _ = literal_str(";").parse_next(input)?;

    Ok(Definition::DataKind(DataKindDef {
        visibility,
        name: name.into(),
        source_adt: source_adt.into(),
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_type_fn_param(input: &mut ParseInput) -> ModalResult<TypeFnParam> {
    let start = input.state.pos;
    let (name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let (ty, kind) = if starts_with_kind_syntax(input) {
        let kind = parse_kind_annotation(input)?;
        (Type::Name(kind.kind.to_string().into()), Some(kind))
    } else {
        (parse_surface_type(input)?, None)
    };
    Ok(TypeFnParam {
        name: name.into(),
        ty,
        kind,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_type_fn_equation(
    input: &mut ParseInput,
    expected_head: &str,
) -> ModalResult<TypeFnEquation> {
    let start = input.state.pos;
    let _ = keyword("case").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let (head, head_span) = identifier_with_span(input)?;
    if head != expected_head {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }
    skip_whitespace_and_comments(input);
    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut patterns = Vec::new();
    if input.input.starts_with(">") {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }
    loop {
        patterns.push(parse_type_pattern(input)?);
        skip_whitespace_and_comments(input);
        if consume_comma_separator(input) {
            continue;
        }
        break;
    }
    let _ = literal_str(">").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let result_start = input.state.pos;
    let result = parse_surface_type(input)?;
    let mut result_span = crate::input::span_from(&result_start, &input.state.pos);
    if result_span.end <= result_span.start {
        result_span.end = result_span.start.saturating_add(1);
    }
    skip_whitespace_and_comments(input);
    let _ = literal_str(";").parse_next(input)?;

    Ok(TypeFnEquation {
        head: head.into(),
        head_span,
        patterns,
        result,
        result_span,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_type_pattern(input: &mut ParseInput) -> ModalResult<TypePattern> {
    skip_whitespace_and_comments(input);
    let start = input.state.pos;
    if input.input.starts_with("_") {
        let _ = literal_str("_").parse_next(input)?;
        return Ok(TypePattern::Wildcard {
            span: nonempty_span(&start, &input.state.pos),
        });
    }

    let (name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);
    if input.input.starts_with("<") {
        let _ = literal_str("<").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let mut args = Vec::new();
        if input.input.starts_with(">") {
            return Err(winnow::error::ErrMode::Cut(
                winnow::error::ContextError::new(),
            ));
        }
        loop {
            args.push(parse_type_pattern(input)?);
            skip_whitespace_and_comments(input);
            if consume_comma_separator(input) {
                continue;
            }
            break;
        }
        let _ = literal_str(">").parse_next(input)?;
        return Ok(TypePattern::Constructor {
            name: name.into(),
            args,
            span: nonempty_span(&start, &input.state.pos),
        });
    }

    let is_var = name.chars().next().is_some_and(|c| c.is_ascii_lowercase());
    if is_var {
        Ok(TypePattern::Var {
            name: name.into(),
            span: nonempty_span(&start, &input.state.pos),
        })
    } else {
        Ok(TypePattern::Constructor {
            name: name.into(),
            args: Vec::new(),
            span: nonempty_span(&start, &input.state.pos),
        })
    }
}

fn nonempty_span(start: &crate::input::Position, end: &crate::input::Position) -> Span {
    let mut span = crate::input::span_from(start, end);
    if span.end <= span.start {
        span.end = span.start.saturating_add(1);
    }
    span
}

fn parse_type_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let start_input = input.input.to_string();
    let mut lookahead = crate::input::new_input(&start_input);
    let parsed = crate::parse_type_def::parse_type_def(&mut lookahead)?;
    let consumed = start_input.len().saturating_sub(lookahead.input.len());

    for _ in start_input[..consumed].chars() {
        let Some(c) = input.input.next_token() else {
            break;
        };
        input.state.advance(c);
    }
    if input.input.starts_with(';') {
        let _ = input.input.next_slice(1);
        input.state.advance(';');
    }

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(Definition::Type(convert_type_def(parsed, span)))
}

fn convert_type_def(parsed: crate::parse_type_def::TypeDef, span: Span) -> TypeDef {
    TypeDef {
        visibility: convert_type_visibility(parsed.visibility),
        name: parsed.name.into_boxed_str(),
        params: parsed
            .params
            .into_iter()
            .map(String::into_boxed_str)
            .collect(),
        body: convert_type_body(parsed.body, span),
        builtin: parsed.builtin,
        span,
        source: None,
    }
}

fn convert_type_visibility(visibility: crate::parse_type_def::Visibility) -> Visibility {
    match visibility {
        crate::parse_type_def::Visibility::Public => Visibility::Public,
        crate::parse_type_def::Visibility::Crate => Visibility::Crate,
        crate::parse_type_def::Visibility::Private => Visibility::Inherited,
    }
}

fn convert_type_body(body: crate::parse_type_def::TypeBody, span: Span) -> TypeBody {
    match body {
        crate::parse_type_def::TypeBody::Struct(fields) => TypeBody::Struct(
            fields
                .into_iter()
                .map(|(name, ty)| convert_type_field(name, ty, span))
                .collect(),
        ),
        crate::parse_type_def::TypeBody::Enum(variants) => TypeBody::Enum(
            variants
                .into_iter()
                .map(|variant| convert_variant_def(variant, span))
                .collect(),
        ),
        crate::parse_type_def::TypeBody::Alias(ty) => TypeBody::Alias(convert_type_expr(ty)),
    }
}

fn convert_variant_def(variant: crate::parse_type_def::VariantDef, span: Span) -> VariantDef {
    let fields: Vec<TypeField> = variant
        .fields
        .into_iter()
        .map(|(name, ty)| convert_type_field(name, ty, span))
        .collect();
    let payload = match variant.payload {
        crate::parse_type_def::VariantPayload::Unit => VariantPayload::Unit,
        crate::parse_type_def::VariantPayload::Record(record_fields) => VariantPayload::Record(
            record_fields
                .into_iter()
                .map(|(name, ty)| convert_type_field(name, ty, span))
                .collect(),
        ),
        crate::parse_type_def::VariantPayload::Tuple(items) => {
            VariantPayload::Tuple(items.into_iter().map(convert_type_expr).collect())
        }
    };

    VariantDef {
        name: variant.name.into_boxed_str(),
        fields,
        payload,
        span,
    }
}

fn convert_type_field(name: String, ty: crate::parse_type_def::TypeExpr, span: Span) -> TypeField {
    TypeField {
        name: name.into_boxed_str(),
        ty: convert_type_expr(ty),
        span,
    }
}

fn convert_type_expr(ty: crate::parse_type_def::TypeExpr) -> Type {
    match ty {
        crate::parse_type_def::TypeExpr::Named(name) => Type::Name(name.into_boxed_str()),
        crate::parse_type_def::TypeExpr::Constructor { name, args } if name == "Fn" => {
            let mut args: Vec<Type> = args.into_iter().map(convert_type_expr).collect();
            if let Some(ret) = args.pop() {
                Type::Fn(args, Box::new(ret))
            } else {
                Type::Constructor {
                    name: name.into_boxed_str(),
                    args,
                }
            }
        }
        crate::parse_type_def::TypeExpr::Constructor { name, args } => Type::Constructor {
            name: name.into_boxed_str(),
            args: args.into_iter().map(convert_type_expr).collect(),
        },
        crate::parse_type_def::TypeExpr::AssociatedFamilyProjection {
            interface,
            args,
            member,
            span,
        } => Type::AssociatedFamilyProjection {
            interface: interface.into_boxed_str(),
            args: args.into_iter().map(convert_type_expr).collect(),
            member: member.into_boxed_str(),
            span,
        },
        crate::parse_type_def::TypeExpr::Associated { base, name } => Type::Associated {
            base: Box::new(convert_type_expr(*base)),
            name: name.into_boxed_str(),
        },
        crate::parse_type_def::TypeExpr::Tuple(items) => {
            Type::Tuple(items.into_iter().map(convert_type_expr).collect())
        }
        crate::parse_type_def::TypeExpr::Record(fields) => Type::Record(
            fields
                .into_iter()
                .map(|(name, ty)| (name.into_boxed_str(), convert_type_expr(ty)))
                .collect(),
        ),
    }
}

/// Parse a sealed type-level domain declaration.
///
/// Syntax: `[visibility] sealed type domain Name { Constructor* }`
fn parse_sealed_domain_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);

    let _ = keyword("sealed").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("domain").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let (name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);

    // Reject generic domain parameters: `domain Name<T>` is not allowed.
    if input.input.starts_with("<") {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut constructors = Vec::new();
    while !input.input.starts_with("}") {
        constructors.push(parse_domain_constructor(input)?);
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str("}").parse_next(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(Definition::SealedDomain(SealedDomainDef {
        visibility,
        name: name.into(),
        constructors,
        span,
    }))
}

/// Parse a single domain constructor.
///
/// Syntax: `Name` or `Name<field: Slot, ...>;`
fn parse_domain_constructor(input: &mut ParseInput) -> ModalResult<DomainConstructor> {
    let start_pos = input.state.pos;

    // Reject per-constructor visibility modifiers.
    if starts_with_keyword(input, "pub") {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    let (ctor_name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);

    let fields = if input.input.starts_with("<") {
        let _ = literal_str("<").parse_next(input)?;
        skip_whitespace_and_comments(input);

        let mut fields = Vec::new();
        if !input.input.starts_with(">") {
            loop {
                fields.push(parse_domain_field(input)?);
                skip_whitespace_and_comments(input);
                if consume_comma_separator(input) {
                    continue;
                }
                break;
            }
        }

        let _ = literal_str(">").parse_next(input)?;
        skip_whitespace_and_comments(input);
        fields
    } else {
        Vec::new()
    };

    let _ = literal_str(";").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(DomainConstructor {
        name: ctor_name.into(),
        fields,
        span,
    })
}

/// Parse a single domain field.
///
/// Syntax: `name: Type` or `name: DomainRef`
fn parse_domain_field(input: &mut ParseInput) -> ModalResult<DomainField> {
    let start_pos = input.state.pos;
    let (field_name, _) = identifier_with_span(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let slot = parse_domain_slot(input)?;

    let span = crate::input::span_from(&start_pos, &input.state.pos);
    Ok(DomainField {
        name: field_name.into(),
        slot,
        span,
    })
}

/// Parse a domain slot annotation.
///
/// Accepts the literal keyword `Type` (unconstrained) or an identifier
/// referring to a sealed domain name.
fn parse_domain_slot(input: &mut ParseInput) -> ModalResult<DomainSlot> {
    let (slot_name, _) = identifier_with_span(input)?;
    if slot_name == "Type" {
        Ok(DomainSlot::Type)
    } else {
        Ok(DomainSlot::DomainRef(slot_name.into()))
    }
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

    if !input.input.starts_with("(") {
        let _ = identifier(input)?;
        skip_whitespace(input);
    }

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

    skip_legacy_capability_alternatives(input)?;

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

fn skip_legacy_capability_alternatives(input: &mut ParseInput) -> ModalResult<()> {
    loop {
        skip_whitespace_and_comments(input);
        if !input.input.starts_with("|") {
            break;
        }

        let _ = literal_str("|").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = parse_effect_type(input)?;
        skip_whitespace_and_comments(input);
        if !input.input.starts_with("(") {
            let _ = identifier(input)?;
            skip_whitespace_and_comments(input);
        }
        let _ = literal_str("(").parse_next(input)?;
        let _ = parse_parameter_list(input)?;
        let _ = literal_str(")").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = parse_optional_return_type(input)?;
    }

    Ok(())
}

fn parse_capability_interface_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;

    let visibility = parse_visibility(input)?;
    skip_whitespace(input);

    let _ = keyword("capability").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("interface").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut operations = Vec::new();
    while !input.input.starts_with(";") {
        operations.push(parse_capability_operation_signature(input)?);
        skip_whitespace_and_comments(input);

        if literal_str("|").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            continue;
        }

        break;
    }

    let _ = literal_str(";").parse_next(input)?;
    reject_duplicate_capability_operations(&operations)?;

    Ok(Definition::CapabilityInterface(CapabilityInterfaceDef {
        visibility,
        name: name.into(),
        operations,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_capability_implementation_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;

    let visibility = parse_visibility(input)?;
    skip_whitespace(input);

    let _ = keyword("capability").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("impl").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("for").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let interface = identifier(input)?;
    skip_whitespace_and_comments(input);

    let mut dependencies = Vec::new();
    while starts_with_keyword(input, "requires") {
        dependencies.push(parse_capability_implementation_dependency(input)?);
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut operations = Vec::new();
    while !input.input.starts_with("}") {
        operations.push(parse_capability_implementation_operation(input)?);
        skip_whitespace_and_comments(input);
    }

    let _ = literal_str("}").parse_next(input)?;
    reject_duplicate_capability_implementation_operations(&operations)?;

    Ok(Definition::CapabilityImplementation(
        CapabilityImplementationDef {
            visibility,
            name: name.into(),
            interface: interface.into(),
            dependencies,
            operations,
            span: crate::input::span_from(&start_pos, &input.state.pos),
        },
    ))
}

fn parse_capability_implementation_dependency(
    input: &mut ParseInput,
) -> ModalResult<CapabilityImplementationDependency> {
    let start = input.state.pos;
    let _ = keyword("requires").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let kind = if keyword("resource").parse_next(input).is_ok() {
        CapabilityImplementationDependencyKind::Resource
    } else if keyword("capability").parse_next(input).is_ok() {
        CapabilityImplementationDependencyKind::Capability
    } else if keyword("config").parse_next(input).is_ok() {
        CapabilityImplementationDependencyKind::Config
    } else {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    };

    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let ty = parse_surface_type(input)?;

    Ok(CapabilityImplementationDependency {
        kind,
        name: name.into(),
        ty,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_capability_implementation_operation(
    input: &mut ParseInput,
) -> ModalResult<CapabilityImplementationOperation> {
    let start = input.state.pos;
    let signature = parse_capability_operation_signature(input)?;
    skip_whitespace_and_comments(input);
    let body = parse_fn_expr_body_pub(input)?;

    Ok(CapabilityImplementationOperation {
        mode: signature.mode,
        name: signature.name,
        params: signature.params,
        return_type: signature.return_type,
        body,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn reject_duplicate_capability_implementation_operations(
    operations: &[CapabilityImplementationOperation],
) -> ModalResult<()> {
    for (idx, operation) in operations.iter().enumerate() {
        if operations[..idx]
            .iter()
            .any(|previous| previous.name == operation.name)
        {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
    }

    Ok(())
}

fn parse_capability_operation_signature(
    input: &mut ParseInput,
) -> ModalResult<CapabilityOperationSig> {
    let start = input.state.pos;
    let mode = parse_capability_operation_mode(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("(").parse_next(input)?;
    let params = parse_parameter_list(input)?;
    let _ = literal_str(")").parse_next(input)?;
    reject_duplicate_params(&params)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("returns").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let return_type = parse_surface_type(input)?;

    Ok(CapabilityOperationSig {
        mode,
        name: name.into(),
        params,
        return_type,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_capability_operation_mode(input: &mut ParseInput) -> ModalResult<CapabilityOperationMode> {
    if keyword("observe").parse_next(input).is_ok() {
        Ok(CapabilityOperationMode::Observe)
    } else if keyword("execute").parse_next(input).is_ok() {
        Ok(CapabilityOperationMode::Execute)
    } else {
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

fn reject_duplicate_capability_operations(
    operations: &[CapabilityOperationSig],
) -> ModalResult<()> {
    for (idx, operation) in operations.iter().enumerate() {
        if operations[..idx]
            .iter()
            .any(|previous| previous.name == operation.name)
        {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
    }

    Ok(())
}

fn reject_duplicate_params(params: &[Param]) -> ModalResult<()> {
    for (idx, param) in params.iter().enumerate() {
        if params[..idx]
            .iter()
            .any(|previous| previous.name == param.name)
        {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
    }

    Ok(())
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
    let type_params = parse_optional_interface_type_params(input)?;
    skip_whitespace_and_comments(input);
    let evidence_constraints = if starts_with_keyword(input, "where") {
        parse_interface_evidence_constraints(input)?
    } else {
        Vec::new()
    };
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut associated_types = Vec::new();
    let mut methods = Vec::new();
    let mut laws = Vec::new();
    while !input.input.starts_with("}") {
        if starts_with_keyword(input, "sealed") || starts_with_keyword(input, "type") {
            associated_types.push(parse_associated_type_decl(input)?);
        } else if starts_with_keyword(input, "law") {
            laws.push(parse_law_definition(input)?);
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
        evidence_constraints,
        associated_types,
        methods,
        laws,
        span: crate::input::span_from(&start_pos, &input.state.pos),
    }))
}

fn parse_interface_evidence_constraints(
    input: &mut ParseInput,
) -> ModalResult<Vec<InterfaceEvidenceConstraint>> {
    let _ = keyword("where").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut constraints = Vec::new();
    loop {
        let start = input.state.pos;
        let subject = parse_surface_type(input)?;
        skip_whitespace_and_comments(input);
        let colon_start = input.state.pos;
        let _ = literal_str(":").parse_next(input)?;
        let colon_span = crate::input::span_from(&colon_start, &input.state.pos);
        skip_whitespace_and_comments(input);
        let interface = parse_surface_type(input)?;
        constraints.push(InterfaceEvidenceConstraint {
            subject,
            interface,
            colon_span,
            span: crate::input::span_from(&start, &input.state.pos),
        });
        skip_whitespace_and_comments(input);
        if consume_comma_separator(input) {
            continue;
        }
        break;
    }

    Ok(constraints)
}

fn parse_associated_type_decl(input: &mut ParseInput) -> ModalResult<AssociatedTypeDecl> {
    if starts_with_keyword(input, "sealed") {
        return parse_sealed_associated_family_decl(input);
    }

    let start = input.state.pos;
    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(";").parse_next(input)?;
    Ok(AssociatedTypeDecl {
        name: name.into(),
        kind: AssociatedTypeKind::Ordinary,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_sealed_associated_family_decl(input: &mut ParseInput) -> ModalResult<AssociatedTypeDecl> {
    let start = input.state.pos;
    let _ = keyword("sealed").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("type").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("family").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let result_domain = parse_surface_type(input)?;
    skip_whitespace_and_comments(input);
    let decreases = if starts_with_keyword(input, "decreases") {
        let decreases_start = input.state.pos;
        let _ = keyword("decreases").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let param = identifier(input)?;
        Some(AssociatedFamilyDecreases {
            param: param.into(),
            span: crate::input::span_from(&decreases_start, &input.state.pos),
        })
    } else {
        None
    };
    let span = crate::input::span_from(&start, &input.state.pos);

    Ok(AssociatedTypeDecl {
        name: name.into(),
        kind: AssociatedTypeKind::SealedFamily {
            result_domain,
            decreases,
            span,
        },
        span,
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

fn parse_law_definition(input: &mut ParseInput) -> ModalResult<LawDef> {
    let start = input.state.pos;
    let _ = keyword("law").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let params = parse_parameter_list(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let constraints = if starts_with_keyword(input, "where") {
        parse_constraint_list(input)?
    } else {
        Vec::new()
    };
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let proposition = expr(input)?;
    Ok(LawDef {
        name: name.into(),
        params,
        constraints,
        proposition,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_law_definition_as_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let law = parse_law_definition(input)?;
    Ok(Definition::Law(law))
}

fn parse_proof_definition_as_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let proof = parse_proof_definition(input)?;
    Ok(Definition::Proof(proof))
}

fn parse_impl_definition(input: &mut ParseInput) -> ModalResult<Definition> {
    let start_pos = input.state.pos;
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);
    let _ = keyword("impl").parse_next(input)?;
    skip_whitespace(input);
    let type_params = parse_optional_interface_type_params(input)?;
    skip_whitespace_and_comments(input);
    let interface = identifier(input)?;
    skip_whitespace_and_comments(input);
    let type_args = parse_optional_impl_head_type_arguments(input)?;
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
    let mut proofs = Vec::new();
    while !input.input.starts_with("}") {
        if starts_with_keyword(input, "type") {
            associated_type_bindings.push(parse_associated_type_binding(input)?);
        } else if starts_with_keyword(input, "proof") {
            proofs.push(parse_proof_definition(input)?);
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
        proofs,
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

fn parse_proof_definition(input: &mut ParseInput) -> ModalResult<ProofDef> {
    let start = input.state.pos;
    let _ = keyword("proof").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let params = parse_parameter_list(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let constraints = if starts_with_keyword(input, "where") {
        parse_constraint_list(input)?
    } else {
        Vec::new()
    };
    skip_whitespace_and_comments(input);
    let _ = literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let body = if starts_with_keyword(input, "by_definition") {
        let _ = keyword("by_definition").parse_next(input)?;
        ProofBody::ByDefinition
    } else if starts_with_keyword(input, "by") {
        let _ = keyword("by").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = keyword("test").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let test_name = parse_string_literal_content(input)?;
        ProofBody::ByTest { test_name }
    } else {
        let e = expr(input)?;
        ProofBody::Expr(e)
    };

    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;

    Ok(ProofDef {
        name: name.into(),
        params,
        constraints,
        body,
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_string_literal_content(input: &mut ParseInput) -> ModalResult<String> {
    let _ = literal_str("\"").parse_next(input)?;
    let content: &str = take_while(0.., |c: char| c != '"').parse_next(input)?;
    let _ = literal_str("\"").parse_next(input)?;
    Ok(content.to_string())
}

fn parse_optional_interface_type_params(
    input: &mut ParseInput,
) -> ModalResult<Vec<InterfaceTypeParam>> {
    if !input.input.starts_with("<") {
        return Ok(Vec::new());
    }

    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut params = Vec::new();

    loop {
        let start = input.state.pos;
        let name = identifier(input)?;
        skip_whitespace_and_comments(input);
        let (domain, kind) = if literal_str(":").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            if starts_with_kind_syntax(input) {
                (None, Some(parse_kind_annotation(input)?))
            } else {
                (Some(parse_surface_type(input)?), None)
            }
        } else {
            (None, None)
        };
        params.push(InterfaceTypeParam {
            name: name.into(),
            domain,
            kind,
            span: crate::input::span_from(&start, &input.state.pos),
        });
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

fn parse_optional_type_parameter_names(input: &mut ParseInput) -> ModalResult<Vec<TypeParam>> {
    if !input.input.starts_with("<") {
        return Ok(Vec::new());
    }

    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut params = Vec::new();

    loop {
        let start = input.state.pos;
        let name = identifier(input)?;
        skip_whitespace_and_comments(input);
        let kind = if literal_str(":").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            if starts_with_kind_syntax(input) {
                Some(parse_kind_annotation(input)?)
            } else {
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
        } else {
            None
        };
        params.push(TypeParam {
            name: name.into(),
            kind,
            bounds: Vec::new(),
            span: crate::input::span_from(&start, &input.state.pos),
        });
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
    parse_required_type_arguments_with_holes(input, TypeHolePolicy::Disallow)
}

fn parse_required_type_arguments_with_holes(
    input: &mut ParseInput,
    hole_policy: TypeHolePolicy,
) -> ModalResult<Vec<Type>> {
    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let mut args = Vec::new();

    loop {
        args.push(parse_surface_type_with_holes(input, hole_policy)?);
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

fn parse_optional_impl_head_type_arguments(input: &mut ParseInput) -> ModalResult<Vec<Type>> {
    if !input.input.starts_with("<") {
        return Ok(Vec::new());
    }

    parse_required_type_arguments_with_holes(input, TypeHolePolicy::Allow)
}

fn parse_effect_type(input: &mut ParseInput) -> ModalResult<EffectType> {
    if keyword("observe").parse_next(input).is_ok() {
        Ok(EffectType::Observe)
    } else if keyword("execute").parse_next(input).is_ok() {
        Ok(EffectType::Operational)
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
    parse_surface_type_with_holes(input, TypeHolePolicy::Disallow)
}

fn parse_surface_type_with_holes(
    input: &mut ParseInput,
    hole_policy: TypeHolePolicy,
) -> ModalResult<Type> {
    skip_whitespace_and_comments(input);

    if input.input.starts_with("<") {
        return match hole_policy {
            TypeHolePolicy::Disallow => parse_associated_family_projection_type(input),
            TypeHolePolicy::Allow => {
                parse_associated_family_projection_type_with_holes(input, hole_policy)
            }
        };
    }

    // Parse explicit Fn(T1, T2) -> T3 type syntax
    if starts_with_keyword(input, "Fn") {
        let _ = keyword("Fn").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("(").parse_next(input)?;
        let mut params = Vec::new();
        skip_whitespace_and_comments(input);
        if !input.input.starts_with(")") {
            params.push(parse_surface_type_with_holes(input, hole_policy)?);
            loop {
                if !consume_comma_separator(input) {
                    break;
                }
                params.push(parse_surface_type_with_holes(input, hole_policy)?);
            }
        }
        let _ = literal_str(")").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("->").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let ret = parse_surface_type_with_holes(input, hole_policy)?;
        return Ok(Type::Fn(params, Box::new(ret)));
    }

    if input.input.starts_with("(") {
        let checkpoint = input.checkpoint();
        if let Ok(fn_type) = parse_parenthesized_callable_type_with_holes(input, hole_policy) {
            return Ok(fn_type);
        }
        input.reset(&checkpoint);
    }

    let lhs = match hole_policy {
        TypeHolePolicy::Disallow => parse_surface_type_atom(input)?,
        TypeHolePolicy::Allow => parse_surface_type_atom_with_holes(input, hole_policy)?,
    };
    skip_whitespace_and_comments(input);
    if literal_str("->").parse_next(input).is_ok() {
        skip_whitespace_and_comments(input);
        let rhs = parse_surface_type_with_holes(input, hole_policy)?;
        Ok(Type::Fn(vec![lhs], Box::new(rhs)))
    } else {
        Ok(lhs)
    }
}

fn parse_parenthesized_callable_type_with_holes(
    input: &mut ParseInput,
    hole_policy: TypeHolePolicy,
) -> ModalResult<Type> {
    let _ = literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let mut params = Vec::new();
    if !input.input.starts_with(")") {
        params.push(parse_surface_type_with_holes(input, hole_policy)?);
        loop {
            if !consume_comma_separator(input) {
                break;
            }
            params.push(parse_surface_type_with_holes(input, hole_policy)?);
        }
    }

    let _ = literal_str(")").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("->").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let ret = parse_surface_type_with_holes(input, hole_policy)?;

    Ok(Type::Fn(params, Box::new(ret)))
}

fn parse_associated_family_projection_type(input: &mut ParseInput) -> ModalResult<Type> {
    parse_associated_family_projection_type_with_holes(input, TypeHolePolicy::Disallow)
}

fn parse_associated_family_projection_type_with_holes(
    input: &mut ParseInput,
    hole_policy: TypeHolePolicy,
) -> ModalResult<Type> {
    let start = input.state.pos;
    let _ = literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let interface = identifier(input)?;
    skip_whitespace_and_comments(input);
    let args = match hole_policy {
        TypeHolePolicy::Disallow => parse_required_type_arguments(input)?,
        TypeHolePolicy::Allow => parse_required_type_arguments_with_holes(input, hole_policy)?,
    };
    skip_whitespace_and_comments(input);
    let _ = literal_str(">").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str("::").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let member = identifier(input)?;

    Ok(Type::AssociatedFamilyProjection {
        interface: interface.into(),
        args,
        member: member.into(),
        span: crate::input::span_from(&start, &input.state.pos),
    })
}

fn parse_surface_type_atom(input: &mut ParseInput) -> ModalResult<Type> {
    parse_surface_type_atom_with_holes(input, TypeHolePolicy::Disallow)
}

fn parse_surface_type_atom_with_holes(
    input: &mut ParseInput,
    hole_policy: TypeHolePolicy,
) -> ModalResult<Type> {
    skip_whitespace_and_comments(input);

    if starts_with_standalone_type_hole(input) {
        if matches!(hole_policy, TypeHolePolicy::Disallow) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
        let start = input.state.pos;
        let _ = literal_str("_").parse_next(input)?;
        return Ok(Type::Hole {
            span: crate::input::span_from(&start, &input.state.pos),
        });
    }

    if starts_with_keyword(input, "capability") {
        let _ = keyword("capability").parse_next(input)?;
        skip_whitespace_and_comments(input);
        let name = identifier(input)?;
        return Ok(Type::Capability(name.into()));
    }

    if input.input.starts_with("[") {
        let _ = literal_str("[").parse_next(input)?;
        let inner = parse_surface_type_with_holes(input, hole_policy)?;
        skip_whitespace_and_comments(input);
        let _ = literal_str("]").parse_next(input)?;
        return Ok(Type::List(Box::new(inner)));
    }

    if input.input.starts_with("(") {
        let _ = literal_str("(").parse_next(input)?;
        skip_whitespace_and_comments(input);

        let mut items = Vec::new();
        if !input.input.starts_with(")") {
            items.push(parse_surface_type_with_holes(input, hole_policy)?);
            loop {
                if !consume_comma_separator(input) {
                    break;
                }
                items.push(parse_surface_type_with_holes(input, hole_policy)?);
            }
        }

        let _ = literal_str(")").parse_next(input)?;
        return Ok(Type::Tuple(items));
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
            let field_type = parse_surface_type_with_holes(input, hole_policy)?;
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
        let args = match hole_policy {
            TypeHolePolicy::Disallow => parse_required_type_arguments(input)?,
            TypeHolePolicy::Allow => parse_required_type_arguments_with_holes(input, hole_policy)?,
        };
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

fn starts_with_type_fn_definition(input: &ParseInput) -> bool {
    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(_) => {
            skip_whitespace_and_comments(&mut lookahead);
            if !starts_with_keyword(&lookahead, "type") {
                return false;
            }
            let rest = skip_ws_in(&lookahead.input["type".len()..]);
            starts_with_keyword_from(rest, "fn")
        }
        Err(_) => false,
    }
}

fn starts_with_type_definition(input: &ParseInput) -> bool {
    if starts_with_type_fn_definition(input) {
        return false;
    }
    if starts_with_keyword(input, "type") {
        return true;
    }

    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(_) => {
            skip_whitespace_and_comments(&mut lookahead);
            if starts_with_keyword(&lookahead, "builtin") {
                let rest = skip_ws_in(&lookahead.input["builtin".len()..]);
                starts_with_keyword_from(rest, "type")
            } else {
                starts_with_keyword(&lookahead, "type")
            }
        }
        Err(_) => false,
    }
}

fn starts_with_data_kind(input: &ParseInput) -> bool {
    if starts_with_keyword(input, "data") {
        let rest = skip_ws_in(&input.input["data".len()..]);
        return starts_with_keyword_from(rest, "kind");
    }

    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(Visibility::Inherited) | Err(_) => false,
        Ok(_) => {
            skip_whitespace_and_comments(&mut lookahead);
            if !starts_with_keyword(&lookahead, "data") {
                return false;
            }
            let rest = skip_ws_in(&lookahead.input["data".len()..]);
            starts_with_keyword_from(rest, "kind")
        }
    }
}

fn starts_with_visible_resource_type(input: &ParseInput) -> bool {
    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(_) => {
            skip_whitespace(&mut lookahead);
            if !starts_with_keyword(&lookahead, "resource") {
                return false;
            }
            let rest = skip_ws_in(&lookahead.input["resource".len()..]);
            starts_with_keyword_from(rest, "type")
        }
        Err(_) => false,
    }
}

fn starts_with_visible_capability_interface(input: &ParseInput) -> bool {
    starts_with_visible_capability_subkeyword(input, "interface")
}

fn starts_with_visible_capability_impl(input: &ParseInput) -> bool {
    starts_with_visible_capability_subkeyword(input, "impl")
}

fn starts_with_visible_capability_subkeyword(input: &ParseInput, subkeyword: &str) -> bool {
    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(_) => {
            skip_whitespace(&mut lookahead);
            if !starts_with_keyword(&lookahead, "capability") {
                return false;
            }
            let rest = skip_ws_in(&lookahead.input["capability".len()..]);
            starts_with_keyword_from(rest, subkeyword)
        }
        Err(_) => false,
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

fn starts_with_sealed_domain(input: &ParseInput) -> bool {
    // Check for `sealed type domain` directly
    if starts_with_keyword(input, "sealed") {
        let rest = skip_ws_in(&input.input["sealed".len()..]);
        if starts_with_keyword_from(rest, "type") {
            let rest2 = skip_ws_in(&rest["type".len()..]);
            return starts_with_keyword_from(rest2, "domain");
        }
    }

    // Check for `[visibility] sealed type domain`
    let mut lookahead = crate::input::new_input(&input.input);
    match parse_visibility(&mut lookahead) {
        Ok(Visibility::Inherited) | Err(_) => false,
        Ok(_) => {
            skip_whitespace(&mut lookahead);
            if !starts_with_keyword(&lookahead, "sealed") {
                return false;
            }
            let rest = skip_ws_in(&lookahead.input["sealed".len()..]);
            if !starts_with_keyword_from(rest, "type") {
                return false;
            }
            let rest2 = skip_ws_in(&rest["type".len()..]);
            starts_with_keyword_from(rest2, "domain")
        }
    }
}

fn starts_with_unsupported_inline_definition(input: &ParseInput) -> bool {
    [
        "pub",
        "workflow",
        "policy",
        "datatype",
        "memory",
        "mod",
        "interface",
        "impl",
        "proof",
        "sealed",
    ]
    .into_iter()
    .any(|keyword| starts_with_keyword(input, keyword))
}

fn starts_with_unsupported_promotion_surface(input: &ParseInput) -> bool {
    let mut lookahead = input.clone();
    skip_whitespace_and_comments(&mut lookahead);
    lookahead.input.starts_with("@promote")
}

fn starts_with_unsupported_proposition_surface(input: &ParseInput) -> bool {
    let mut lookahead = input.clone();
    skip_whitespace_and_comments(&mut lookahead);

    if starts_with_keyword(&lookahead, "where") {
        return true;
    }

    let source = lookahead.input.as_ref();
    if !source
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '<' || ch == '(')
    {
        return false;
    }

    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, ch) in source.char_indices() {
        if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
            if ch == ';' {
                return looks_like_named_predicate_clause(&source[..index]);
            }

            if matches!(ch, '{' | '}') {
                return false;
            }

            if source[index..].starts_with("==") || source[index..].starts_with("!=") {
                return true;
            }

            if ch == ':' && !source[index..].starts_with("::") && !source[..index].ends_with(':') {
                return true;
            }
        }

        match ch {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
    }

    looks_like_named_predicate_clause(source)
}

fn looks_like_named_predicate_clause(source: &str) -> bool {
    let source = source.trim();
    let Some(first) = source.chars().next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    let name_end = source
        .char_indices()
        .find_map(|(index, ch)| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                None
            } else {
                Some(index)
            }
        })
        .unwrap_or(source.len());

    let rest = source[name_end..].trim_start();
    if rest.is_empty() {
        return true;
    }

    if !rest.starts_with('<') {
        return false;
    }

    let mut depth = 0usize;
    for (index, ch) in rest.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return rest[index + ch.len_utf8()..].trim().is_empty();
                }
            }
            _ => {}
        }
    }

    false
}

fn starts_with_recoverable_definition(input: &ParseInput) -> bool {
    starts_with_visible_keyword(input, "workflow")
        || starts_with_visible_keyword(input, "mod")
        || starts_with_keyword(input, "role")
        || starts_with_visible_resource_type(input)
        || starts_with_type_fn_definition(input)
        || starts_with_data_kind(input)
        || starts_with_type_definition(input)
        || starts_with_sealed_domain(input)
        || starts_with_visible_capability_interface(input)
        || starts_with_visible_capability_impl(input)
        || starts_with_visible_keyword(input, "capability")
        || starts_with_keyword(input, "proxy")
        || starts_with_visible_keyword(input, "interface")
        || starts_with_visible_keyword(input, "impl")
        || starts_with_builtin_fn(input)
        || starts_with_visible_keyword(input, "fn")
        || starts_with_unsupported_inline_definition(input)
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
            if starts_with_recoverable_definition(input) {
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
mod fn_defs;
pub use fn_defs::{parse_builtin_fn_definition, parse_fn_body, parse_fn_definition};

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

        if starts_with_visible_resource_type(input) {
            definitions.push(parse_resource_type_definition(input)?);
            continue;
        }

        if starts_with_type_fn_definition(input) {
            definitions.push(parse_type_fn_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "prop") {
            definitions.push(parse_proposition_predicate_decl(input)?);
            continue;
        }

        if starts_with_data_kind(input) {
            definitions.push(parse_data_kind_definition(input)?);
            continue;
        }

        if starts_with_type_definition(input) {
            definitions.push(parse_type_definition(input)?);
            continue;
        }

        if starts_with_sealed_domain(input) {
            definitions.push(parse_sealed_domain_definition(input)?);
            continue;
        }

        if starts_with_visible_capability_interface(input) {
            definitions.push(parse_capability_interface_definition(input)?);
            continue;
        }

        if starts_with_visible_capability_impl(input) {
            definitions.push(parse_capability_implementation_definition(input)?);
            continue;
        }

        if starts_with_visible_keyword(input, "capability") {
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

        if starts_with_keyword(input, "law") {
            definitions.push(parse_law_definition_as_definition(input)?);
            continue;
        }

        if starts_with_keyword(input, "proof") {
            definitions.push(parse_proof_definition_as_definition(input)?);
            continue;
        }

        if starts_with_unsupported_proposition_surface(input) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }

        if starts_with_unsupported_promotion_surface(input) {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
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
mod tests;
