//! Core-expression checking helpers.

use super::*;
use crate::check_expr::pattern_bridge::core_span_to_surface_span;

/// Type check a host/lowered core expression.
///
/// This intentionally supports only the pure core expression subset needed by
/// typeck-owned lowered code and host-IR validation. Unsupported core forms
/// remain explicit errors rather than being guessed.
pub fn check_core_expr(env: &TypeEnv, expr: &CoreExpr) -> CheckResult {
    match expr {
        CoreExpr::Literal(value) => check_core_literal(env, value),
        CoreExpr::Variable { name, .. } => match env.lookup_variable(name.as_ref()) {
            Some(ty) => CheckResult::success(ty),
            None => match env.get_variant(name.as_ref()) {
                Some((type_info, variant_idx, variant_info))
                    if matches!(variant_info.payload_shape, VariantPayloadShape::Unit) =>
                {
                    CheckResult::success(build_constructor_type(type_info, variant_idx))
                }
                _ => CheckResult::error(ConstructorError::UnboundVariable {
                    name: name.to_string(),
                    span: Span::default(),
                }),
            },
        },
        CoreExpr::Constructor { name, fields } => check_core_constructor(env, name, fields),
        CoreExpr::Let {
            pattern,
            expr,
            body,
            span,
        } => {
            let expr_result = check_core_expr(env, expr);
            if !expr_result.is_ok() {
                return expr_result;
            }

            let expr_ty = expr_result.substitution.apply(&expr_result.ty);
            let surface_pattern = match core_pattern_to_surface(env, pattern) {
                Ok(pattern) => pattern,
                Err(error) => return CheckResult::error(error),
            };
            let bindings = match check_irrefutable_let_pattern(
                env,
                "core let",
                &surface_pattern,
                &expr_ty,
                core_span_to_surface_span(*span),
            ) {
                Ok(bindings) => bindings,
                Err(error) => return CheckResult::error(error),
            };

            let mut body_env = env.clone();
            bind_irrefutable_pattern_bindings(&mut body_env, bindings);
            let body_result = check_core_expr(&body_env, body);
            let combined_sub = expr_result.substitution.compose(&body_result.substitution);
            if !body_result.is_ok() {
                return CheckResult {
                    ty: Type::Var(TypeVar::fresh()),
                    substitution: combined_sub,
                    errors: body_result.errors,
                };
            }
            CheckResult {
                ty: combined_sub.apply(&body_result.ty),
                substitution: combined_sub,
                errors: Vec::new(),
            }
        }
        other => CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: format!("core expression not supported by type checker: {other:?}"),
            span: Span::default(),
        }),
    }
}

fn check_core_literal(env: &TypeEnv, value: &ash_core::Value) -> CheckResult {
    match value {
        ash_core::Value::Variant { name, fields } => {
            let fields = fields
                .iter()
                .map(|(field, value)| (field.clone(), CoreExpr::Literal(value.clone())))
                .collect::<Vec<_>>();
            check_core_constructor(env, name, &fields)
        }
        _ => CheckResult::success(core_value_type(env, value)),
    }
}

fn core_value_type(env: &TypeEnv, value: &ash_core::Value) -> Type {
    match value {
        ash_core::Value::Int(_) => Type::Int,
        ash_core::Value::Float(_) => Type::Float,
        ash_core::Value::String(_) => Type::String,
        ash_core::Value::Bool(_) => Type::Bool,
        ash_core::Value::Null => Type::Null,
        value if value.is_list() => Type::List(Box::new(Type::Var(TypeVar::fresh()))),
        ash_core::Value::Record(fields) => Type::Record(
            fields
                .keys()
                .map(|name| (name.clone().into_boxed_str(), Type::Var(TypeVar::fresh())))
                .collect(),
        ),
        ash_core::Value::Variant { name, .. } => env
            .get_variant(name)
            .map(|(type_info, variant_idx, _)| build_constructor_type(type_info, variant_idx))
            .unwrap_or_else(|| Type::Var(TypeVar::fresh())),
        ash_core::Value::Cap(_) => Type::Cap {
            name: "Capability".into(),
            effect: ash_core::Effect::Operational,
        },
        ash_core::Value::Time(_)
        | ash_core::Value::Ref(_)
        | ash_core::Value::Instance(_)
        | ash_core::Value::InstanceAddr(_)
        | ash_core::Value::ControlLink(_)
        | ash_core::Value::Stream(_)
        | ash_core::Value::ProcessHandle(_)
        | ash_core::Value::ProcAwaitCapture(_)
        | ash_core::Value::ProcYieldCapture
        | ash_core::Value::ProcParCapture { .. }
        | ash_core::Value::ProcScatterCapture { .. }
        | ash_core::Value::ProcJoinCapture { .. }
        | ash_core::Value::ProcGatherCapture { .. }
        | ash_core::Value::Closure { .. }
        | ash_core::Value::ActEnvToken => Type::Var(TypeVar::fresh()),
    }
}

fn check_core_constructor(env: &TypeEnv, name: &str, fields: &[(String, CoreExpr)]) -> CheckResult {
    let (type_info, variant_idx, variant_info) = match env.get_variant(name) {
        Some(result) => result,
        None => {
            return CheckResult::error(ConstructorError::UnknownConstructor(
                name.to_string(),
                Span::default(),
            ));
        }
    };

    let mut errors = Vec::new();
    let mut substitution = Substitution::new();
    match variant_info.payload_shape {
        VariantPayloadShape::Tuple => check_core_tuple_constructor_fields(
            env,
            name,
            variant_info,
            fields,
            &mut substitution,
            &mut errors,
        ),
        VariantPayloadShape::Unit | VariantPayloadShape::Record => {
            check_core_named_constructor_fields(
                env,
                name,
                variant_info,
                fields,
                &mut substitution,
                &mut errors,
            )
        }
    }

    CheckResult {
        ty: substitution.apply(&build_constructor_type(type_info, variant_idx)),
        substitution,
        errors,
    }
}

fn check_core_tuple_constructor_fields(
    env: &TypeEnv,
    constructor_name: &str,
    variant_info: &VariantInfo,
    fields: &[(String, CoreExpr)],
    substitution: &mut Substitution,
    errors: &mut Vec<ConstructorError>,
) {
    if fields.len() != variant_info.fields.len() {
        errors.push(ConstructorError::TupleArityMismatch {
            constructor: constructor_name.to_string(),
            expected: variant_info.fields.len(),
            actual: fields.len(),
            span: Span::default(),
        });
    }

    for (index, ((expected_name, expected_ty), (field_name, field_expr))) in
        variant_info.fields.iter().zip(fields.iter()).enumerate()
    {
        if expected_name != &tuple_field_name(index) || field_name != expected_name {
            errors.push(ConstructorError::TupleArityMismatch {
                constructor: constructor_name.to_string(),
                expected: variant_info.fields.len(),
                actual: fields.len(),
                span: Span::default(),
            });
            continue;
        }

        check_core_constructor_field_type(
            env,
            constructor_name,
            CoreConstructorField::Tuple(index),
            expected_ty,
            field_expr,
            substitution,
            errors,
        );
    }
}

fn check_core_named_constructor_fields(
    env: &TypeEnv,
    constructor_name: &str,
    variant_info: &VariantInfo,
    fields: &[(String, CoreExpr)],
    substitution: &mut Substitution,
    errors: &mut Vec<ConstructorError>,
) {
    let mut seen_fields = HashSet::new();
    for (field_name, _) in fields {
        if !seen_fields.insert(field_name.as_str()) {
            errors.push(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "duplicate field `{field_name}` in core constructor `{constructor_name}`"
                ),
                span: Span::default(),
            });
        }
    }

    let expected_fields = variant_info
        .fields
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<HashSet<_>>();
    let provided_fields = fields
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<HashSet<_>>();

    for expected in &expected_fields {
        if !provided_fields.contains(expected) {
            errors.push(ConstructorError::MissingField {
                constructor: constructor_name.to_string(),
                field: (*expected).to_string(),
                span: Span::default(),
            });
        }
    }

    for provided in &provided_fields {
        if !expected_fields.contains(provided) {
            errors.push(ConstructorError::UnknownField {
                constructor: constructor_name.to_string(),
                field: (*provided).to_string(),
                span: Span::default(),
            });
        }
    }

    let expected_types = variant_info
        .fields
        .iter()
        .map(|(name, ty)| (name.as_str(), ty))
        .collect::<HashMap<_, _>>();
    for (field_name, field_expr) in fields {
        if let Some(expected_ty) = expected_types.get(field_name.as_str()) {
            check_core_constructor_field_type(
                env,
                constructor_name,
                CoreConstructorField::Record(field_name),
                expected_ty,
                field_expr,
                substitution,
                errors,
            );
        }
    }
}

enum CoreConstructorField<'a> {
    Record(&'a str),
    Tuple(usize),
}

fn check_core_constructor_field_type(
    env: &TypeEnv,
    constructor_name: &str,
    field: CoreConstructorField<'_>,
    expected_ty: &Type,
    field_expr: &CoreExpr,
    substitution: &mut Substitution,
    errors: &mut Vec<ConstructorError>,
) {
    let field_result = check_core_expr(env, field_expr);
    let field_ty = field_result.substitution.apply(&field_result.ty);
    errors.extend(field_result.errors);

    let expected_ty_subst = substitution.apply(expected_ty);
    match unify(&expected_ty_subst, &field_ty) {
        Ok(sub) => *substitution = substitution.compose(&sub),
        Err(_) => match field {
            CoreConstructorField::Record(field_name) => {
                errors.push(ConstructorError::FieldTypeMismatch {
                    constructor: constructor_name.to_string(),
                    field: field_name.to_string(),
                    expected: expected_ty.to_string(),
                    actual: field_ty.to_string(),
                    span: Span::default(),
                })
            }
            CoreConstructorField::Tuple(position) => {
                errors.push(ConstructorError::TupleFieldTypeMismatch {
                    constructor: constructor_name.to_string(),
                    position,
                    expected: expected_ty.to_string(),
                    actual: field_ty.to_string(),
                    span: Span::default(),
                });
            }
        },
    }
}

fn core_pattern_to_surface(
    env: &TypeEnv,
    pattern: &CorePattern,
) -> Result<Pattern, ConstructorError> {
    match pattern {
        CorePattern::Variable { name, .. } => Ok(Pattern::Variable {
            name: name.clone().into(),
            span: Span::default(),
        }),
        CorePattern::Tuple(items) => items
            .iter()
            .map(|item| core_pattern_to_surface(env, item))
            .collect::<Result<Vec<_>, _>>()
            .map(Pattern::Tuple),
        CorePattern::Record(fields) => fields
            .iter()
            .map(|(name, pattern)| {
                core_pattern_to_surface(env, pattern).map(|pattern| (name.clone().into(), pattern))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Pattern::Record),
        CorePattern::List(items, rest) => Ok(Pattern::List {
            elements: items
                .iter()
                .map(|item| core_pattern_to_surface(env, item))
                .collect::<Result<Vec<_>, _>>()?,
            rest: rest.clone().map(Into::into),
        }),
        CorePattern::Wildcard => Ok(Pattern::Wildcard),
        CorePattern::Literal(value) => core_value_to_surface_literal(value)
            .map(Pattern::Literal)
            .ok_or_else(|| ConstructorError::UnsupportedExpression {
                kind: format!("core let literal pattern cannot be converted: {value:?}"),
                span: Span::default(),
            }),
        CorePattern::Variant { name, fields } => match fields {
            None => Ok(Pattern::Variant {
                name: name.clone().into(),
                fields: None,
                payload: ash_parser::surface::VariantPatternPayload::Unit,
            }),
            Some(fields) => core_variant_pattern_to_surface(env, name, fields),
        },
    }
}

fn core_variant_pattern_to_surface(
    env: &TypeEnv,
    name: &str,
    fields: &[(String, CorePattern)],
) -> Result<Pattern, ConstructorError> {
    let surface_fields = fields
        .iter()
        .map(|(field, pattern)| {
            core_pattern_to_surface(env, pattern).map(|pattern| (field.clone().into(), pattern))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some((_, _, variant)) = env.get_variant(name)
        && variant.payload_shape == VariantPayloadShape::Tuple
        && fields
            .iter()
            .enumerate()
            .all(|(index, (field, _))| field == &tuple_field_name(index))
    {
        let tuple_items = surface_fields
            .iter()
            .map(|(_, pattern)| pattern.clone())
            .collect::<Vec<_>>();
        return Ok(Pattern::Variant {
            name: name.into(),
            fields: Some(surface_fields),
            payload: ash_parser::surface::VariantPatternPayload::Tuple(tuple_items),
        });
    }

    Ok(Pattern::Variant {
        name: name.into(),
        fields: Some(surface_fields.clone()),
        payload: ash_parser::surface::VariantPatternPayload::Record(surface_fields),
    })
}

fn core_value_to_surface_literal(value: &ash_core::Value) -> Option<Literal> {
    match value {
        ash_core::Value::Int(value) => Some(Literal::Int(*value)),
        ash_core::Value::Float(value) => Some(Literal::Float(ordered_float::OrderedFloat(*value))),
        ash_core::Value::String(value) => Some(Literal::String(value.clone().into_boxed_str())),
        ash_core::Value::Bool(value) => Some(Literal::Bool(*value)),
        ash_core::Value::Null => Some(Literal::Null),
        value if value.is_list() => value
            .list_to_vec()?
            .iter()
            .map(core_value_to_surface_literal)
            .collect::<Option<Vec<_>>>()
            .map(Literal::List),
        _ => None,
    }
}
