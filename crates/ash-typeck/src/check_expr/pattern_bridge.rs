//! Pattern-environment bridge helpers for expression checking.

use super::*;
use crate::type_env::{PatternCanonicalConstructor, PatternCanonicalType};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};

pub(crate) fn pattern_type_env_from_type_env(env: &TypeEnv) -> crate::check_pattern::TypeEnv {
    let mut pattern_env = PatternTypeEnv::new();
    let mut type_defs = env
        .ast_type_defs()
        .map(|(_, type_def)| type_def.clone())
        .collect::<Vec<_>>();
    type_defs.sort_by(|left, right| left.name.cmp(&right.name));
    for type_def in order_pattern_type_defs(type_defs) {
        pattern_env.add_type_def(type_def.name.clone(), type_def.clone());
    }
    pattern_env
}

fn order_pattern_type_defs(mut type_defs: Vec<TypeDef>) -> Vec<TypeDef> {
    let all_names = type_defs
        .iter()
        .map(|type_def| type_def.name.to_string())
        .collect::<HashSet<_>>();
    let mut ordered = Vec::with_capacity(type_defs.len());
    let mut ordered_names = HashSet::new();

    while !type_defs.is_empty() {
        let Some(index) = type_defs.iter().position(|type_def| {
            let mut dependencies = HashSet::new();
            collect_type_def_dependencies(type_def, &mut dependencies);
            dependencies
                .iter()
                .filter(|name| all_names.contains(*name))
                .all(|name| ordered_names.contains(name))
        }) else {
            ordered.extend(type_defs);
            break;
        };

        let type_def = type_defs.remove(index);
        ordered_names.insert(type_def.name.to_string());
        ordered.push(type_def);
    }

    ordered
}

fn collect_type_def_dependencies(type_def: &TypeDef, dependencies: &mut HashSet<String>) {
    match &type_def.body {
        TypeBody::Struct(fields) => {
            for (_, field_type) in fields {
                collect_type_expr_dependencies(field_type, dependencies);
            }
        }
        TypeBody::Enum(variants) => {
            for variant in variants {
                for (_, field_type) in &variant.fields {
                    collect_type_expr_dependencies(field_type, dependencies);
                }
                match &variant.payload {
                    ash_core::ast::VariantPayload::Unit => {}
                    ash_core::ast::VariantPayload::Record(fields) => {
                        for (_, field_type) in fields {
                            collect_type_expr_dependencies(field_type, dependencies);
                        }
                    }
                    ash_core::ast::VariantPayload::Tuple(items) => {
                        for item in items {
                            collect_type_expr_dependencies(item, dependencies);
                        }
                    }
                }
            }
        }
        TypeBody::Alias(target) => collect_type_expr_dependencies(target, dependencies),
    }
}

fn collect_type_expr_dependencies(type_expr: &TypeExpr, dependencies: &mut HashSet<String>) {
    match type_expr {
        TypeExpr::Named(name) => {
            dependencies.insert(name.to_string());
        }
        TypeExpr::Constructor { name, args } => {
            dependencies.insert(name.to_string());
            for arg in args {
                collect_type_expr_dependencies(arg, dependencies);
            }
        }
        TypeExpr::Tuple(items) => {
            for item in items {
                collect_type_expr_dependencies(item, dependencies);
            }
        }
        TypeExpr::Record(fields) => {
            for (_, field_type) in fields {
                collect_type_expr_dependencies(field_type, dependencies);
            }
        }
        TypeExpr::Associated { base, .. } => collect_type_expr_dependencies(base, dependencies),
    }
}

pub(crate) fn check_irrefutable_let_pattern(
    env: &TypeEnv,
    construct_kind: &str,
    pattern: &Pattern,
    scrutinee_type: &Type,
    span: Span,
) -> Result<Bindings, ConstructorError> {
    let pattern_env = pattern_type_env_from_type_env(env);
    let canonicalization = pattern_canonicalization_for_scrutinee(env, scrutinee_type);
    let irrefutability = check_irrefutable_pattern_with_canonicalization(
        &pattern_env,
        pattern,
        scrutinee_type,
        &canonicalization,
    );

    match irrefutability.outcome {
        IrrefutabilityOutcome::Irrefutable => Ok(irrefutability.bindings),
        outcome => Err(ConstructorError::UnsupportedExpression {
            kind: format_irrefutable_let_error(construct_kind, pattern, scrutinee_type, &outcome),
            span,
        }),
    }
}

/// Check a refutable pattern using the same canonical constructor universe as
/// `let`, `match`, and `if let`. Scoped source visitors use this only to carry
/// already-validated arm bindings into nested checks.
pub(crate) fn check_pattern_bindings(
    env: &TypeEnv,
    pattern: &Pattern,
    scrutinee_type: &Type,
) -> Result<Bindings, crate::solver::TypeError> {
    let pattern_env = pattern_type_env_from_type_env(env);
    match pattern_canonicalization_for_scrutinee(env, scrutinee_type) {
        PatternCanonicalization::Matchable(canonical) => {
            crate::check_pattern::check_pattern_with_canonical_type(
                &pattern_env,
                pattern,
                &canonical,
            )
        }
        PatternCanonicalization::Blocked { .. } => {
            crate::check_pattern::check_pattern(&pattern_env, pattern, scrutinee_type)
        }
    }
}

/// Select the canonical constructor universe used by every supported surface
/// pattern boundary. Nominal newtypes contribute their closed singleton only
/// after the existing exact-identity and visibility checks succeed; all other
/// types retain the ordinary canonicalization path.
pub(crate) fn pattern_canonicalization_for_scrutinee(
    env: &TypeEnv,
    scrutinee_type: &Type,
) -> PatternCanonicalization {
    nominal_newtype_pattern_canonicalization(env, scrutinee_type)
        .unwrap_or_else(|| env.canonicalize_type_for_pattern(scrutinee_type))
}

/// Build the singleton constructor universe for a checked non-generic nominal
/// newtype. This is limited to checked surface pattern boundaries: it neither
/// changes ordinary ADT canonicalization nor exposes runtime pattern behavior.
///
/// The visible scrutinee name must resolve to the newtype's exact declaration
/// identity. That admits a public named import while preserving the provider's
/// `TypeDeclId`; a same-spelled or local wrapper cannot supply the constructor
/// contract for a different nominal type. A public re-export is eligible only
/// when it preserves that same provider-owned identity.
fn nominal_newtype_pattern_canonicalization(
    env: &TypeEnv,
    scrutinee_type: &Type,
) -> Option<PatternCanonicalization> {
    let Type::Constructor { name, args, .. } = scrutinee_type else {
        return None;
    };
    if !name.is_root() || !args.is_empty() {
        return None;
    }

    let newtype = env.nominal_newtype(name.name.as_str())?;
    if env.nominal_type_identity(name.name.as_str())? != newtype.identity() {
        return None;
    }
    let source_local = env
        .current_module_identity()
        .is_some_and(|module| newtype.identity().module == *module);
    if !source_local
        && !env.is_visible_imported_nominal_newtype(name.name.as_str(), &newtype.identity())
    {
        return None;
    }
    let representation = newtype.representation()?.clone();

    Some(PatternCanonicalization::Matchable(PatternCanonicalType {
        source_type: scrutinee_type.clone(),
        canonical_type: scrutinee_type.clone(),
        canonical_name: name.clone(),
        canonical_type_args: Vec::new(),
        constructors: vec![PatternCanonicalConstructor {
            name: newtype.constructor().to_string(),
            variant_index: 0,
            fields: vec![(tuple_field_name(0), representation)],
            payload_shape: VariantPayloadShape::Tuple,
        }],
    }))
}

pub(crate) fn bind_irrefutable_pattern_bindings(env: &mut TypeEnv, bindings: Bindings) {
    for (name, ty) in bindings {
        env.bind_variable(&name, ty);
    }
}

pub(crate) fn surface_pattern_span(pattern: &Pattern, fallback: Span) -> Span {
    match pattern {
        Pattern::Variable { span, .. } => *span,
        Pattern::Tuple(items) => items
            .iter()
            .map(|item| surface_pattern_span(item, fallback))
            .find(|span| *span != Span::default())
            .unwrap_or(fallback),
        Pattern::Record(fields) => fields
            .iter()
            .map(|(_, pattern)| surface_pattern_span(pattern, fallback))
            .find(|span| *span != Span::default())
            .unwrap_or(fallback),
        Pattern::List { elements, .. } => elements
            .iter()
            .map(|item| surface_pattern_span(item, fallback))
            .find(|span| *span != Span::default())
            .unwrap_or(fallback),
        Pattern::Variant {
            payload, fields, ..
        } => match payload {
            ash_parser::surface::VariantPatternPayload::Record(fields) => fields
                .iter()
                .map(|(_, pattern)| surface_pattern_span(pattern, fallback))
                .find(|span| *span != Span::default())
                .unwrap_or(fallback),
            ash_parser::surface::VariantPatternPayload::Tuple(items) => items
                .iter()
                .map(|pattern| surface_pattern_span(pattern, fallback))
                .find(|span| *span != Span::default())
                .unwrap_or(fallback),
            ash_parser::surface::VariantPatternPayload::Unit => fields
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(|(_, pattern)| surface_pattern_span(pattern, fallback))
                .find(|span| *span != Span::default())
                .unwrap_or(fallback),
        },
        Pattern::Wildcard | Pattern::Literal(_) => fallback,
    }
}

pub(super) fn core_span_to_surface_span(span: CoreSpan) -> Span {
    Span {
        start: span.start,
        end: span.end,
        line: 0,
        column: 0,
    }
}

pub(super) fn format_irrefutable_let_error(
    construct_kind: &str,
    pattern: &Pattern,
    scrutinee_type: &Type,
    outcome: &IrrefutabilityOutcome,
) -> String {
    match outcome {
        IrrefutabilityOutcome::Irrefutable => format!(
            "non-irrefutable pattern in {construct_kind}: pattern {} over type {} unexpectedly classified as irrefutable; use match or if let ... else",
            format_surface_pattern(pattern),
            scrutinee_type
        ),
        IrrefutabilityOutcome::Refutable { witness } => format!(
            "non-irrefutable pattern in {construct_kind}: pattern {} over type {} is refutable; missing {}; use match or if let ... else",
            format_surface_pattern(pattern),
            scrutinee_type,
            format_irrefutability_witness(witness)
        ),
        IrrefutabilityOutcome::Impossible { reason } => format!(
            "non-irrefutable pattern in {construct_kind}: pattern {} over type {} is impossible; reason {}; use match or if let ... else",
            format_surface_pattern(pattern),
            scrutinee_type,
            format_irrefutability_impossible_reason(reason)
        ),
        IrrefutabilityOutcome::Blocked { reason } => format!(
            "non-irrefutable pattern in {construct_kind}: pattern {} over type {} is blocked; reason {}; use match or if let ... else",
            format_surface_pattern(pattern),
            scrutinee_type,
            format_irrefutability_blocked_reason(reason)
        ),
    }
}

fn format_irrefutability_witness(witness: &IrrefutabilityWitness) -> String {
    match witness {
        IrrefutabilityWitness::Pattern(pattern) => {
            format!("witness {}", format_surface_pattern(pattern))
        }
        IrrefutabilityWitness::ShortList { minimum_len } => {
            format!("short list with fewer than {minimum_len} elements")
        }
        IrrefutabilityWitness::NonLiteralValue { literal } => {
            format!("non-literal value different from {literal:?}")
        }
        IrrefutabilityWitness::Description(description) => description.clone(),
    }
}

fn format_irrefutability_impossible_reason(reason: &IrrefutabilityImpossibleReason) -> String {
    match reason {
        IrrefutabilityImpossibleReason::DuplicateBinder { name } => {
            format!("duplicate binder `{name}`")
        }
        IrrefutabilityImpossibleReason::PatternTypeError(error) => error.to_string(),
        IrrefutabilityImpossibleReason::UnknownConstructor {
            name,
            scrutinee_type,
        } => format!("unknown constructor `{name}` for scrutinee type {scrutinee_type}"),
    }
}

fn format_irrefutability_blocked_reason(reason: &IrrefutabilityBlockedReason) -> String {
    match reason {
        IrrefutabilityBlockedReason::Canonicalization {
            source_type,
            reason,
        } => format!("canonicalization of {source_type} blocked: {reason:?}"),
        IrrefutabilityBlockedReason::ProductShapeUnavailable {
            scrutinee_type,
            pattern_shape,
        } => format!("product shape `{pattern_shape}` unavailable for {scrutinee_type}"),
        IrrefutabilityBlockedReason::ConstructorUniverseUnavailable { scrutinee_type } => {
            format!("constructor universe unavailable for {scrutinee_type}")
        }
    }
}

pub(super) fn format_surface_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Variable { name, .. } => name.to_string(),
        Pattern::Wildcard => "_".to_string(),
        Pattern::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(format_surface_pattern)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Pattern::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, pattern)| format!("{name}: {}", format_surface_pattern(pattern)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Pattern::List { elements, rest } => {
            let mut parts = elements
                .iter()
                .map(format_surface_pattern)
                .collect::<Vec<_>>();
            if let Some(rest) = rest {
                parts.push(format!("..{rest}"));
            }
            format!("[{}]", parts.join(", "))
        }
        Pattern::Literal(literal) => format!("{literal:?}"),
        Pattern::Variant { name, payload, .. } => match payload {
            ash_parser::surface::VariantPatternPayload::Unit => name.to_string(),
            ash_parser::surface::VariantPatternPayload::Record(fields) => format!(
                "{name} {{{}}}",
                fields
                    .iter()
                    .map(|(field, pattern)| {
                        format!("{field}: {}", format_surface_pattern(pattern))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ash_parser::surface::VariantPatternPayload::Tuple(items) => format!(
                "{name}({})",
                items
                    .iter()
                    .map(format_surface_pattern)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
    }
}
