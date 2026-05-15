//! Generalized do-notation target resolution substrate.
//!
//! This module intentionally resolves only the computation target and hidden
//! sequencing dictionary. Statement typing and elaboration are deferred to the
//! typed do-block implementation task.

#![allow(clippy::result_large_err)]

use crate::error::ConstructorError;
use crate::{Kind, PartialConstructorElaborationError, QualifiedName, TypeEnv};
use ash_parser::surface::{DoTarget, Type as SurfaceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DoTowerLevel {
    Effectful,
    Proc,
    Workflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DoDictionaryOp {
    HiddenActReturn,
    HiddenActBind,
    Ordinary(QualifiedName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DoDictionary {
    pub(crate) target: QualifiedName,
    pub(crate) value_constructor: QualifiedName,
    pub(crate) return_op: DoDictionaryOp,
    pub(crate) bind_op: DoDictionaryOp,
    pub(crate) tower_level: DoTowerLevel,
}

/// Resolve a surface `do:K` target to the MVP hidden dictionary.
///
/// The accepted MVP targets are compiler-known `Act`, `Proc`, and `Workflow` unary type
/// constructors. This is deliberately shaped like future `Monad<K>` evidence,
/// but does not attempt interface/impl lookup yet.
pub(crate) fn resolve_do_target(
    env: &TypeEnv,
    target: &DoTarget,
) -> Result<DoDictionary, ConstructorError> {
    let target_name = target.name.as_ref();

    if !target.args.is_empty() {
        let surface_target = SurfaceType::Constructor {
            name: target.name.to_string().into(),
            args: target.args.clone(),
        };
        env.elaborate_do_target_constructor_expr(&surface_target)
            .map_err(|err| do_target_shape_error(err, target.span))?;

        return Err(ConstructorError::UnsupportedExpression {
            kind: format!(
                "missing Monad evidence for do target {}; target shape elaborated successfully, but SPEC-067 Monad<K> dictionary resolution is not implemented",
                render_surface_type(&surface_target)
            ),
            span: target.span,
        });
    }

    let (qualified, type_info) =
        env.resolve_type(target_name)
            .map_err(|_| ConstructorError::UnsupportedExpression {
                kind: format!(
                    "unknown do target '{target_name}'; use a registered computation constructor such as Act, Proc, or Workflow"
                ),
                span: target.span,
            })?;

    if qualified.name == "Result" {
        let surface_target = SurfaceType::Name(target.name.to_string().into());
        if let Err(err) = env.elaborate_do_target_constructor_expr(&surface_target) {
            return Err(do_target_shape_error(err, target.span));
        }
        return Err(ConstructorError::UnsupportedExpression {
            kind: "missing Monad evidence for do target Result; target shape elaborated successfully, but SPEC-067 Monad<K> dictionary resolution is not implemented".to_string(),
            span: target.span,
        });
    }

    let arity = type_info
        .map(crate::type_env::TypeInfo::type_arg_count)
        .or_else(|| {
            env.lookup_type(&qualified.name)
                .map(|type_def| type_def.params.len())
        })
        .unwrap_or(0);
    let kind = Kind::n_ary(arity);
    let expected = Kind::n_ary(1);

    if kind != expected {
        return Err(ConstructorError::UnsupportedExpression {
            kind: format!(
                "do target {} has kind {kind}, expected {expected}; use a computation constructor such as Act, Proc, or Workflow",
                qualified.display()
            ),
            span: target.span,
        });
    }

    match qualified.name.as_str() {
        "Act" => Ok(DoDictionary {
            target: qualified.clone(),
            value_constructor: qualified,
            return_op: DoDictionaryOp::HiddenActReturn,
            bind_op: DoDictionaryOp::HiddenActBind,
            tower_level: DoTowerLevel::Effectful,
        }),
        "Proc" => Ok(DoDictionary {
            target: qualified.clone(),
            value_constructor: qualified,
            return_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["proc".to_string()],
                "unit",
            )),
            bind_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["proc".to_string()],
                "bind",
            )),
            tower_level: DoTowerLevel::Proc,
        }),
        "Workflow" => Ok(DoDictionary {
            target: qualified.clone(),
            value_constructor: qualified,
            return_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["workflow".to_string()],
                "unit",
            )),
            bind_op: DoDictionaryOp::Ordinary(QualifiedName::qualified(
                vec!["workflow".to_string()],
                "bind",
            )),
            tower_level: DoTowerLevel::Workflow,
        }),
        "Result" => unreachable!("Result is rejected before MVP dictionary selection"),
        other => Err(ConstructorError::UnsupportedExpression {
            kind: format!(
                "do target {other} has no MVP dictionary; future Monad<K> interface resolution is deferred"
            ),
            span: target.span,
        }),
    }
}

fn do_target_shape_error(
    err: PartialConstructorElaborationError,
    fallback_span: ash_parser::token::Span,
) -> ConstructorError {
    let (kind, span) = match err {
        PartialConstructorElaborationError::BareHigherArityConstructor {
            constructor,
            arity,
            hint,
            span,
        } => (
            format!(
                "wrong target shape for do target {constructor}: bare constructor has arity {arity}; write {hint} with an explicit `_` hole"
            ),
            span,
        ),
        PartialConstructorElaborationError::MultipleHoles {
            constructor,
            count,
            span,
        } => (
            format!(
                "multiple type holes in do target {constructor}: found {count}; the MVP accepts exactly one value-position hole"
            ),
            span,
        ),
        PartialConstructorElaborationError::UnsupportedHolePosition { reason, span } => {
            (format!("unsupported do target shape: {reason}"), span)
        }
        PartialConstructorElaborationError::NoInversionBoundary { context, span } => (
            format!(
                "unsupported non-inverting do target shape: cannot elaborate type hole by inverting {context}"
            ),
            span,
        ),
        PartialConstructorElaborationError::MissingHole { constructor, span } => (
            format!(
                "wrong target shape for do target {constructor}: expected exactly one explicit `_` hole"
            ),
            span,
        ),
        PartialConstructorElaborationError::WrongArity {
            constructor,
            expected_arity,
            found_arity,
            span,
        } => (
            format!(
                "wrong target shape for do target {constructor}: expected {expected_arity} type arguments, found {found_arity}"
            ),
            span,
        ),
        PartialConstructorElaborationError::UnknownConstructor { constructor, span } => {
            (format!("unknown do target '{constructor}'"), span)
        }
        PartialConstructorElaborationError::ArgumentLoweringFailed {
            constructor,
            reason,
            span,
        } => (
            format!("unsupported do target shape for {constructor}: {reason}"),
            span,
        ),
    };

    ConstructorError::UnsupportedExpression {
        kind,
        span: if span == ash_parser::token::Span::default() {
            fallback_span
        } else {
            span
        },
    }
}

fn render_surface_type(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Name(name) => name.to_string(),
        SurfaceType::Hole { .. } => "_".to_string(),
        SurfaceType::List(item) => format!("[{}]", render_surface_type(item)),
        SurfaceType::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", render_surface_type(ty)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Capability(name) => format!("capability {name}"),
        SurfaceType::Constructor { name, args } => format!(
            "{}<{}>",
            name,
            args.iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Associated { base, name } => {
            format!("{}::{name}", render_surface_type(base))
        }
        SurfaceType::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => format!(
            "<{}<{}>>::{}",
            interface,
            args.iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", "),
            member
        ),
        SurfaceType::Fn(params, ret) => format!(
            "Fn({}) -> {}",
            params
                .iter()
                .map(render_surface_type)
                .collect::<Vec<_>>()
                .join(", "),
            render_surface_type(ret)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{Name as CoreName, TypeBody, TypeDef, Visibility};
    use ash_parser::surface::{Name, Type};
    use ash_parser::token::Span;

    fn target(name: &str) -> DoTarget {
        DoTarget {
            name: Name::from(name),
            args: vec![],
            span: Span::default(),
        }
    }

    fn resolve(name: &str) -> Result<DoDictionary, ConstructorError> {
        resolve_do_target(&TypeEnv::with_builtin_types(), &target(name))
    }

    fn error_text(err: ConstructorError) -> String {
        match err {
            ConstructorError::UnsupportedExpression { kind, .. } => kind,
            other => other.to_string(),
        }
    }

    #[test]
    fn do_target_act_resolves_to_hidden_act_dictionary() {
        let dict = resolve("Act").expect("Act target should resolve");

        assert_eq!(dict.target, QualifiedName::root("Act"));
        assert_eq!(dict.value_constructor, QualifiedName::root("Act"));
        assert_eq!(dict.return_op, DoDictionaryOp::HiddenActReturn);
        assert_eq!(dict.bind_op, DoDictionaryOp::HiddenActBind);
        assert_eq!(dict.tower_level, DoTowerLevel::Effectful);
    }

    #[test]
    fn do_target_proc_resolves_to_hidden_proc_dictionary() {
        let dict = resolve("Proc").expect("Proc target should resolve");

        assert_eq!(dict.target, QualifiedName::root("Proc"));
        assert_eq!(dict.value_constructor, QualifiedName::root("Proc"));
        assert_eq!(
            dict.return_op,
            DoDictionaryOp::Ordinary(QualifiedName::qualified(vec!["proc".to_string()], "unit"))
        );
        assert_eq!(
            dict.bind_op,
            DoDictionaryOp::Ordinary(QualifiedName::qualified(vec!["proc".to_string()], "bind"))
        );
        assert_eq!(dict.tower_level, DoTowerLevel::Proc);
    }

    #[test]
    fn do_target_int_reports_wrong_kind_not_computation_constructor() {
        let message = error_text(resolve("Int").expect_err("Int is a proper type"));

        assert!(message.contains("do target Int has kind *"), "{message}");
        assert!(message.contains("expected * -> *"), "{message}");
        assert!(message.contains("Act, Proc, or Workflow"), "{message}");
    }

    #[test]
    fn do_target_missing_reports_unknown_target() {
        let message = error_text(resolve("Missing").expect_err("Missing target is unknown"));

        assert!(message.contains("unknown do target 'Missing'"), "{message}");
        assert!(message.contains("Act, Proc, or Workflow"), "{message}");
    }

    #[test]
    fn do_target_bare_result_reports_wrong_shape_with_hole_hint() {
        let message = error_text(resolve("Result").expect_err("Result is not an MVP dictionary"));

        assert!(message.contains("Result"), "{message}");
        assert!(message.contains("Result<_, E>"), "{message}");
        assert!(message.contains("wrong target shape"), "{message}");
        assert!(!message.contains("missing Monad evidence"), "{message}");
    }

    #[test]
    fn do_target_resolution_does_not_import_dictionary_ops_into_lexical_scope() {
        let env = TypeEnv::with_builtin_types();
        let _dict = resolve_do_target(&env, &target("Proc")).expect("Proc target should resolve");

        assert!(env.lookup_variable("bind").is_none());
        assert!(env.lookup_variable("unit").is_none());
        assert!(env.lookup_variable("proc::bind").is_some());
        assert!(env.lookup_variable("proc::unit").is_some());
    }

    #[test]
    fn do_target_with_partial_explicit_args_reaches_missing_monad_evidence() {
        let mut env = TypeEnv::new();
        for type_def in [
            TypeDef {
                name: CoreName::from("Result"),
                params: vec!["T".into(), "E".into()],
                body: TypeBody::Struct(vec![]),
                visibility: Visibility::Public,
                builtin: false,
            },
            TypeDef {
                name: CoreName::from("E"),
                params: vec![],
                body: TypeBody::Struct(vec![]),
                visibility: Visibility::Public,
                builtin: false,
            },
        ] {
            env.register_type(&type_def)
                .expect("register do-target fixture type");
        }
        let result_target = DoTarget {
            name: Name::from("Result"),
            args: vec![
                Type::Hole {
                    span: Span::default(),
                },
                Type::Name(Name::from("E")),
            ],
            span: Span::default(),
        };

        let message = error_text(
            resolve_do_target(&env, &result_target)
                .expect_err("Result<_, E> has shape but no Monad evidence"),
        );

        assert!(message.contains("Result<_, E>"), "{message}");
        assert!(message.contains("missing Monad evidence"), "{message}");
        assert!(!message.contains("wrong target shape"), "{message}");
    }

    #[test]
    fn do_target_with_wrong_explicit_arg_count_reports_shape_error() {
        let env = TypeEnv::with_builtin_types();
        let result_target = DoTarget {
            name: Name::from("Result"),
            args: vec![Type::Name(Name::from("Int"))],
            span: Span::default(),
        };

        let message = error_text(
            resolve_do_target(&env, &result_target)
                .expect_err("Result<Int> is the wrong do-target shape"),
        );

        assert!(message.contains("wrong target shape"), "{message}");
        assert!(message.contains("expected 2 type arguments"), "{message}");
        assert!(!message.contains("missing Monad evidence"), "{message}");
    }

    #[test]
    fn do_target_uses_ast_type_params_when_type_info_is_absent() {
        let mut env = TypeEnv::new();
        env.register_type_identity(&TypeDef {
            name: CoreName::from("Boxed"),
            params: vec!["A".into()],
            body: TypeBody::Struct(vec![]),
            visibility: Visibility::Public,
            builtin: false,
        })
        .expect("register generic type identity");

        env.remove_type_info_for_test("Boxed");

        let message = error_text(
            resolve_do_target(&env, &target("Boxed"))
                .expect_err("generic AST-only target has kind but no MVP dictionary"),
        );
        assert!(message.contains("no MVP dictionary"), "{message}");
        assert!(!message.contains("has kind *"), "{message}");
    }
}
