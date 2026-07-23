//! Surface type lowering helpers for type-checking frontends.

use super::*;
use crate::error::TypeEnvError;

pub(super) fn synthetic_program_module_identity() -> ash_core::semantic_summary::ModuleIdentity {
    ash_core::semantic_summary::ModuleIdentity::new(
        None,
        ash_core::module_graph::ModuleId(0),
        vec!["<program>".to_string()],
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
            reason: "type_check_program default module context".to_string(),
        },
    )
}

/// Clone an environment with the declared kinds of a surface callable's type
/// parameters registered before its parameter and result types are lowered.
pub(super) fn register_surface_type_parameter_kinds(
    env: &TypeEnv,
    params: &[ash_parser::surface::TypeParam],
) -> Result<TypeEnv, TypeCheckError> {
    let mut scoped = env.clone();
    for param in params {
        let kind = param
            .kind
            .as_ref()
            .map(|annotation| annotation.kind.clone())
            .unwrap_or(Kind::Type);
        scoped
            .register_type_parameter_kind(param.name.to_string(), kind)
            .map_err(TypeCheckError::from)?;
    }
    Ok(scoped)
}

/// Create fresh type variables for a callable's surface binders and install
/// their declared interface assumptions in the same lowering scope.
pub(super) fn bind_surface_type_parameters(
    env: &TypeEnv,
    params: &[ash_parser::surface::TypeParam],
) -> Result<(TypeEnv, std::collections::HashMap<String, Type>), TypeCheckError> {
    let mut scoped = register_surface_type_parameter_kinds(env, params)?;
    let bindings = params
        .iter()
        .map(|param| (param.name.to_string(), Type::Var(TypeVar::fresh())))
        .collect::<std::collections::HashMap<_, _>>();

    for param in params {
        let Type::Var(var) = bindings[&param.name.to_string()] else {
            unreachable!("callable type parameter bindings are fresh variables");
        };
        for bound in &param.bounds {
            if !scoped.has_interface(bound.interface.as_ref()) {
                return Err(TypeEnvError::MissingInterface(
                    bound.interface.to_string(),
                    bound.span,
                )
                .into());
            }
            scoped.bind_type_var_interface_bound(var, bound.interface.as_ref());
        }
    }

    Ok((scoped, bindings))
}

pub(super) fn resolve_public_surface_associated_interface(
    env: &TypeEnv,
    base_ty: &Type,
    name: &str,
) -> Result<String, TypeCheckError> {
    let Type::Var(var) = base_ty else {
        return Err(TypeCheckError::TypeError(format!(
            "unresolved associated type '{name}'"
        )));
    };

    let Some(bounds) = env.type_var_interface_bounds.get(var) else {
        return Err(TypeCheckError::TypeError(format!(
            "unresolved associated type '{name}'"
        )));
    };

    let mut candidates = Vec::new();
    for bound_iface in bounds {
        match env.interfaces.get(bound_iface) {
            Some(iface_info) if iface_info.associated_types.contains(&name.to_string()) => {
                candidates.push(bound_iface.clone());
            }
            _ => {}
        }
    }

    if candidates.len() == 1 {
        Ok(candidates.into_iter().next().unwrap())
    } else if candidates.len() > 1 {
        Err(TypeCheckError::TypeError(format!(
            "ambiguous associated type '{name}'"
        )))
    } else {
        Err(TypeCheckError::TypeError(format!(
            "unresolved associated type '{name}'"
        )))
    }
}

pub(super) fn workflow_surface_type_to_type(
    env: &TypeEnv,
    ty: &ash_parser::surface::Type,
    type_params: &std::collections::HashMap<String, Type>,
) -> Result<Type, TypeCheckError> {
    match ty {
        ash_parser::surface::Type::Hole { span } => Err(TypeCheckError::TypeError(format!(
            "type holes are only accepted in audited SPEC-066 do-target positions; this semantic lowering path does not accept source holes at {span:?}"
        ))),
        ash_parser::surface::Type::Name(name) => {
            if let Some(ty) = type_params.get(name.as_ref()) {
                if let Some(kind) = env.type_parameter_kind(name.as_ref())
                    && !kind.is_type()
                {
                    return Err(TypeCheckError::TypeError(format!(
                        "constructor variable '{}' has kind {}; expected a fully applied proper type",
                        name, kind
                    )));
                }
                return Ok(ty.clone());
            }

            match name.as_ref() {
                "Int" => Ok(Type::Int),
                "String" => Ok(Type::String),
                "Bool" => Ok(Type::Bool),
                "Prop" => Ok(Type::Constructor {
                    name: QualifiedName::root("Prop"),
                    args: vec![],
                    kind: Kind::Prop,
                }),
                "Null" => Ok(Type::Null),
                "Time" => Ok(Type::Time),
                "Ref" => Ok(Type::Ref),
                _ => {
                    let (qualified, _) = env
                        .resolve_type(name.as_ref())
                        .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
                    env.check_type_constructor_arity(&qualified, 0)
                        .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
                    if let Some(target) = env.transparent_alias_target(&qualified, &[]) {
                        Ok(target)
                    } else {
                        Ok(Type::Constructor {
                            name: qualified,
                            args: vec![],
                            kind: Kind::Type,
                        })
                    }
                }
            }
        }
        ash_parser::surface::Type::List(item) => {
            workflow_surface_type_to_type(env, item, type_params)
                .map(|item| Type::List(Box::new(item)))
        }
        ash_parser::surface::Type::Tuple(items) => {
            let items = items
                .iter()
                .enumerate()
                .map(|(index, ty)| {
                    workflow_surface_type_to_type(env, ty, type_params)
                        .map(|ty| (ash_core::adt::tuple_field_name(index).into_boxed_str(), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Record(items))
        }
        ash_parser::surface::Type::Record(fields) => {
            let fields = fields
                .iter()
                .map(|(name, ty)| {
                    workflow_surface_type_to_type(env, ty, type_params)
                        .map(|ty| (Box::from(name.as_ref()), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Record(fields))
        }
        ash_parser::surface::Type::Capability(name) => Ok(Type::Cap {
            name: Box::from(name.as_ref()),
            effect: ash_core::Effect::Operational,
        }),
        ash_parser::surface::Type::Constructor { name, args } => {
            if let Some(kind) = env.type_parameter_kind(name.as_ref()) {
                if kind.is_type() {
                    return Err(TypeCheckError::TypeError(format!(
                        "proper type variable '{}' of kind * cannot be applied as a constructor",
                        name
                    )));
                }
                let expected_arity = kind.arity();
                if args.len() != expected_arity {
                    return Err(TypeCheckError::TypeError(format!(
                        "wrong arity for constructor variable '{}': expected {}, found {}",
                        name,
                        expected_arity,
                        args.len()
                    )));
                }
                let args = args
                    .iter()
                    .map(|arg| workflow_surface_type_to_type(env, arg, type_params))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(Type::ConstructorVariableApp {
                    constructor: name.to_string(),
                    args,
                    kind: Kind::Type,
                });
            }
            let (qualified, _) = env
                .resolve_type(name.as_ref())
                .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
            env.check_type_constructor_arity(&qualified, args.len())
                .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
            let args = args
                .iter()
                .map(|arg| workflow_surface_type_to_type(env, arg, type_params))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(target) = env.transparent_alias_target(&qualified, &args) {
                Ok(target)
            } else {
                Ok(Type::Constructor {
                    name: qualified,
                    args,
                    kind: Kind::Type,
                })
            }
        }
        ash_parser::surface::Type::Fn(params, _row, ret) => {
            // Pure function type: Fn(T, U) -> V => Type::Fn(params, ret)
            let param_types: Result<Vec<_>, _> = params
                .iter()
                .map(|p| workflow_surface_type_to_type(env, p, type_params))
                .collect();
            let ret_type = workflow_surface_type_to_type(env, ret, type_params)?;
            Ok(Type::Fn(param_types?, Box::new(ret_type)))
        }
        ash_parser::surface::Type::Associated { base, name } => {
            let base_ty = workflow_surface_type_to_type(env, base, type_params)?;
            let interface = resolve_public_surface_associated_interface(env, &base_ty, name)?;
            Ok(Type::Associated {
                interface,
                base: Box::new(base_ty),
                name: name.to_string(),
            })
        }
        ash_parser::surface::Type::AssociatedFamilyProjection {
            interface,
            args,
            member,
            span,
        } => {
            let declaration = env
                .lookup_associated_family_declaration(interface.as_ref(), member.as_ref())
                .ok_or_else(|| {
                    TypeCheckError::TypeError(format!(
                        "unknown sealed associated-family projection '<{}<...>>::{}'",
                        interface, member
                    ))
                })?;
            if declaration.interface_params.len() != args.len() {
                return Err(TypeCheckError::TypeError(format!(
                    "associated-family projection '{}::{}' at {:?} expects {} interface arguments, found {}",
                    interface,
                    member,
                    span,
                    declaration.interface_params.len(),
                    args.len()
                )));
            }
            let args = args
                .iter()
                .map(|arg| workflow_surface_type_to_type(env, arg, type_params))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Associated {
                interface: interface.to_string(),
                base: Box::new(Type::Constructor {
                    name: QualifiedName::root(interface.as_ref()),
                    args,
                    kind: Kind::Type,
                }),
                name: member.to_string(),
            })
        }
    }
}

pub(crate) fn bind_pattern_variables(
    env: &mut TypeEnv,
    pattern: &ash_parser::surface::Pattern,
    ty: &Type,
) {
    match pattern {
        ash_parser::surface::Pattern::Variable { name, .. } => {
            env.bind_variable(name.as_ref(), ty.clone());
        }
        ash_parser::surface::Pattern::Tuple(items) => {
            for (index, item) in items.iter().enumerate() {
                let item_ty = if let Type::Record(fields) = ty {
                    fields
                        .iter()
                        .find(|(field, _)| field.as_ref() == ash_core::adt::tuple_field_name(index))
                        .map(|(_, field_ty)| field_ty.clone())
                        .unwrap_or_else(|| Type::Var(TypeVar::fresh()))
                } else {
                    Type::Var(TypeVar::fresh())
                };
                bind_pattern_variables(env, item, &item_ty);
            }
        }
        ash_parser::surface::Pattern::Record(fields) => {
            for (field_name, pattern) in fields {
                let field_ty = if let Type::Record(record_fields) = ty {
                    record_fields
                        .iter()
                        .find(|(name, _)| name.as_ref() == field_name.as_ref())
                        .map(|(_, field_ty)| field_ty.clone())
                        .unwrap_or_else(|| Type::Var(TypeVar::fresh()))
                } else {
                    Type::Var(TypeVar::fresh())
                };
                bind_pattern_variables(env, pattern, &field_ty);
            }
        }
        ash_parser::surface::Pattern::List { elements, rest } => {
            let item_ty = match ty {
                Type::List(item_ty) => item_ty.as_ref().clone(),
                _ => Type::Var(TypeVar::fresh()),
            };

            for element in elements {
                bind_pattern_variables(env, element, &item_ty);
            }

            if let Some(rest) = rest {
                env.bind_variable(rest.as_ref(), Type::List(Box::new(item_ty)));
            }
        }
        ash_parser::surface::Pattern::Variant {
            name,
            fields,
            payload,
        } => {
            let variant_fields = variant_field_types(env, ty, name.as_ref());

            if let Some(fields) = fields {
                for (field_name, pattern) in fields {
                    let field_ty = variant_fields
                        .as_ref()
                        .and_then(|variant_fields| {
                            variant_fields
                                .iter()
                                .find(|(name, _)| name == field_name.as_ref())
                                .map(|(_, field_ty)| field_ty.clone())
                        })
                        .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
                    bind_pattern_variables(env, pattern, &field_ty);
                }
            }

            match payload {
                ash_parser::surface::VariantPatternPayload::Unit => {}
                ash_parser::surface::VariantPatternPayload::Tuple(items) => {
                    for (index, item) in items.iter().enumerate() {
                        let field_ty = variant_fields
                            .as_ref()
                            .and_then(|variant_fields| {
                                variant_fields
                                    .iter()
                                    .find(|(name, _)| {
                                        name == &ash_core::adt::tuple_field_name(index)
                                    })
                                    .map(|(_, field_ty)| field_ty.clone())
                            })
                            .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
                        bind_pattern_variables(env, item, &field_ty);
                    }
                }
                ash_parser::surface::VariantPatternPayload::Record(fields) => {
                    for (field_name, pattern) in fields {
                        let field_ty = variant_fields
                            .as_ref()
                            .and_then(|variant_fields| {
                                variant_fields
                                    .iter()
                                    .find(|(name, _)| name == field_name.as_ref())
                                    .map(|(_, field_ty)| field_ty.clone())
                            })
                            .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
                        bind_pattern_variables(env, pattern, &field_ty);
                    }
                }
            }
        }
        ash_parser::surface::Pattern::Wildcard | ash_parser::surface::Pattern::Literal(_) => {}
    }
}
