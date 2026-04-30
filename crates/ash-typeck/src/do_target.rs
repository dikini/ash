//! Generalized do-notation target resolution substrate.
//!
//! This module intentionally resolves only the computation target and hidden
//! sequencing dictionary. Statement typing and elaboration are deferred to the
//! typed do-block implementation task.

#![allow(clippy::result_large_err)]

use crate::error::ConstructorError;
use crate::{Kind, QualifiedName, TypeEnv};
use ash_parser::surface::DoTarget;

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
/// The accepted MVP targets are compiler-known `Act` and `Proc` unary type
/// constructors. This is deliberately shaped like future `Monad<K>` evidence,
/// but does not attempt interface/impl lookup yet.
pub(crate) fn resolve_do_target(
    env: &TypeEnv,
    target: &DoTarget,
) -> Result<DoDictionary, ConstructorError> {
    let target_name = target.name.as_ref();

    if !target.args.is_empty() {
        return Err(ConstructorError::UnsupportedExpression {
            kind: format!(
                "do target {target_name} with explicit type arguments is not supported in the MVP; future Monad<K> target holes such as Result<_, E> are deferred"
            ),
            span: target.span,
        });
    }

    let (qualified, type_info) =
        env.resolve_type(target_name)
            .map_err(|_| ConstructorError::UnsupportedExpression {
                kind: format!(
                    "unknown do target '{target_name}'; use a registered computation constructor such as Act or Proc"
                ),
                span: target.span,
            })?;

    if qualified.name == "Result" {
        return Err(ConstructorError::UnsupportedExpression {
            kind: "do target Result is deferred in the MVP; Result<_, E> hole targets require future Monad<K> dictionary resolution".to_string(),
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
                "do target {} has kind {kind}, expected {expected}; use a computation constructor such as Act or Proc",
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
        assert!(message.contains("Act or Proc"), "{message}");
    }

    #[test]
    fn do_target_missing_reports_unknown_target() {
        let message = error_text(resolve("Missing").expect_err("Missing target is unknown"));

        assert!(message.contains("unknown do target 'Missing'"), "{message}");
    }

    #[test]
    fn do_target_result_is_deferred_without_dictionary() {
        let message = error_text(resolve("Result").expect_err("Result is not an MVP dictionary"));

        assert!(message.contains("Result"), "{message}");
        assert!(message.contains("deferred"), "{message}");
        assert!(message.contains("Result<_, E>"), "{message}");
        assert!(message.contains("Monad<K>"), "{message}");
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
    fn do_target_with_explicit_args_is_deferred() {
        let env = TypeEnv::with_builtin_types();
        let result_target = DoTarget {
            name: Name::from("Result"),
            args: vec![Type::Name(Name::from("Int"))],
            span: Span::default(),
        };

        let message = error_text(
            resolve_do_target(&env, &result_target)
                .expect_err("explicit target args are not in the MVP"),
        );

        assert!(message.contains("explicit type arguments"), "{message}");
        assert!(message.contains("Result<_, E>"), "{message}");
        assert!(message.contains("deferred"), "{message}");
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
