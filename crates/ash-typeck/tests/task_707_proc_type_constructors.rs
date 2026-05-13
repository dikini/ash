use ash_core::ast::TypeExpr;
use ash_parser::surface::{BuiltinFnDef, Param, Type as SurfaceType, Visibility};
use ash_parser::token::Span;
use ash_typeck::QualifiedName;
use ash_typeck::builtin_fn_signature_type;
use ash_typeck::type_env::{TypeEnv, type_expr_to_type};
use ash_typeck::types::Type;
use proptest::prelude::*;
use proptest::test_runner::Config;
use std::collections::HashMap;

fn builtin_with_param_and_return(
    name: &str,
    param_ty: SurfaceType,
    return_ty: SurfaceType,
) -> BuiltinFnDef {
    BuiltinFnDef {
        visibility: Visibility::Public,
        name: name.into(),
        type_params: vec![],
        params: vec![Param {
            name: "value".into(),
            ty: param_ty,
        }],
        return_type: return_ty,
        proposition_tail: None,
        span: Span::default(),
    }
}

fn constructor(name: &str, args: Vec<SurfaceType>) -> SurfaceType {
    SurfaceType::Constructor {
        name: name.into(),
        args,
    }
}

fn assert_constructor_with_single_arg(ty: &Type, expected_name: &str, expected_arg: &Type) {
    match ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.name, expected_name);
            assert_eq!(args, std::slice::from_ref(expected_arg));
        }
        other => panic!("expected {expected_name}<...> constructor, got {other:?}"),
    }
}

#[test]
fn task_707_proc_int_builtin_signature_preserves_constructor() {
    let env = TypeEnv::with_builtin_types();
    let builtin = builtin_with_param_and_return(
        "spawnable",
        SurfaceType::Name("Int".into()),
        constructor("Proc", vec![SurfaceType::Name("Int".into())]),
    );

    let ty = builtin_fn_signature_type(&env, &builtin).expect("Proc<Int> should resolve");

    match ty {
        Type::Fn(params, ret) => {
            assert_eq!(params, vec![Type::Int]);
            assert_constructor_with_single_arg(&ret, "Proc", &Type::Int);
        }
        other => panic!("expected function type, got {other:?}"),
    }
}

#[test]
fn task_707_p_string_builtin_signature_preserves_constructor() {
    let env = TypeEnv::with_builtin_types();
    let builtin = builtin_with_param_and_return(
        "handle",
        SurfaceType::Name("String".into()),
        constructor("P", vec![SurfaceType::Name("String".into())]),
    );

    let ty = builtin_fn_signature_type(&env, &builtin).expect("P<String> should resolve");

    match ty {
        Type::Fn(params, ret) => {
            assert_eq!(params, vec![Type::String]);
            assert_constructor_with_single_arg(&ret, "P", &Type::String);
        }
        other => panic!("expected function type, got {other:?}"),
    }
}

fn assert_arity_message(msg: &str, constructor_name: &str, expected: char, found: char) {
    assert!(
        msg.contains(constructor_name),
        "diagnostic should name {constructor_name}: {msg}"
    );
    assert!(
        msg.contains("arity") || (msg.contains("expected") && msg.contains("found")),
        "diagnostic should mention constructor arity/expected/found: {msg}"
    );
    assert!(
        msg.contains(expected),
        "diagnostic should mention expected arity {expected}: {msg}"
    );
    assert!(
        msg.contains(found),
        "diagnostic should mention found arity {found}: {msg}"
    );
}

#[test]
fn task_707_proc_without_type_argument_reports_constructor_arity() {
    let env = TypeEnv::with_builtin_types();
    let builtin = builtin_with_param_and_return(
        "bad_proc",
        SurfaceType::Name("Int".into()),
        constructor("Proc", vec![]),
    );

    let err = builtin_fn_signature_type(&env, &builtin)
        .expect_err("Proc without type argument should be rejected");
    assert_arity_message(&err.to_string(), "Proc", '1', '0');
}

#[test]
fn task_707_bare_proc_name_reports_constructor_arity() {
    let env = TypeEnv::with_builtin_types();
    let builtin = builtin_with_param_and_return(
        "bare_proc",
        SurfaceType::Name("Int".into()),
        SurfaceType::Name("Proc".into()),
    );

    let err = builtin_fn_signature_type(&env, &builtin)
        .expect_err("bare Proc should be rejected because Proc expects one type argument");
    assert_arity_message(&err.to_string(), "Proc", '1', '0');
}

#[test]
fn task_707_p_with_two_type_arguments_reports_constructor_arity() {
    let env = TypeEnv::with_builtin_types();
    let builtin = builtin_with_param_and_return(
        "bad_p",
        SurfaceType::Name("Int".into()),
        constructor(
            "P",
            vec![
                SurfaceType::Name("Int".into()),
                SurfaceType::Name("String".into()),
            ],
        ),
    );

    let err = builtin_fn_signature_type(&env, &builtin)
        .expect_err("P with two type arguments should be rejected");
    let msg = err.to_string();

    assert!(msg.contains('P'), "diagnostic should name P: {msg}");
    assert!(
        msg.contains("arity") || (msg.contains("expected") && msg.contains("found")),
        "diagnostic should mention constructor arity/expected/found: {msg}"
    );
    assert!(
        msg.contains('1'),
        "diagnostic should mention expected arity 1: {msg}"
    );
    assert!(
        msg.contains('2'),
        "diagnostic should mention found arity 2: {msg}"
    );
}

#[test]
fn task_707_builtin_proc_and_p_still_enforce_direct_constructor_arity() {
    let env = TypeEnv::with_builtin_types();

    let proc_err = env
        .check_type_constructor_arity(&QualifiedName::root("Proc"), 0)
        .expect_err("builtin Proc should still require one type argument");
    assert_arity_message(&proc_err.to_string(), "Proc", '1', '0');

    let p_err = env
        .check_type_constructor_arity(&QualifiedName::root("P"), 2)
        .expect_err("builtin P should still require one type argument");
    assert_arity_message(&p_err.to_string(), "P", '1', '2');
}

#[test]
fn task_707_qualified_proc_and_p_are_not_treated_as_builtin_process_constructors() {
    let env = TypeEnv::with_builtin_types();

    env.check_type_constructor_arity(&QualifiedName::qualified(vec!["user".into()], "Proc"), 0)
        .expect("qualified user::Proc should not be forced to builtin Proc<T> arity");
    env.check_type_constructor_arity(&QualifiedName::qualified(vec!["imported".into()], "P"), 2)
        .expect("qualified imported::P should not be forced to builtin P<T> arity");
}

#[test]
fn task_707_unknown_process_like_constructor_remains_unbound() {
    let env = TypeEnv::with_builtin_types();
    let builtin = builtin_with_param_and_return(
        "unknown_process",
        SurfaceType::Name("Int".into()),
        constructor("Process", vec![SurfaceType::Name("Int".into())]),
    );

    let err = builtin_fn_signature_type(&env, &builtin)
        .expect_err("unknown Process<Int> constructor should be rejected");
    let msg = err.to_string();

    assert!(
        msg.contains("Process"),
        "diagnostic should name Process: {msg}"
    );
    assert!(
        msg.contains("Unbound") || msg.contains("unbound") || msg.contains("not found"),
        "diagnostic should remain unknown/unbound, not process-special cased: {msg}"
    );
}

#[test]
fn task_707_placeholder_generic_constructor_arity_is_deferred() {
    let mut env = TypeEnv::with_builtin_types();
    env.declare_type_name("FutureBox");
    let ty = TypeExpr::Constructor {
        name: "FutureBox".to_string(),
        args: vec![TypeExpr::Named("Int".to_string())],
    };

    let converted = type_expr_to_type(&ty, &HashMap::new(), &env)
        .expect("placeholder generic constructor arity should be deferred until type info exists");

    assert_constructor_with_single_arg(&converted, "FutureBox", &Type::Int);
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, ..Config::default() })]

    #[test]
    fn task_707_proc_and_p_preserve_single_primitive_arg(
        constructor_name in prop_oneof![Just("Proc"), Just("P")],
        primitive in prop_oneof![Just("Int"), Just("String"), Just("Bool")],
    ) {
        let env = TypeEnv::with_builtin_types();
        let expected_arg = match primitive {
            "Int" => Type::Int,
            "String" => Type::String,
            "Bool" => Type::Bool,
            other => unreachable!("unexpected primitive generated: {other}"),
        };
        let builtin = builtin_with_param_and_return(
            "prop_process_ctor",
            SurfaceType::Name(primitive.into()),
            constructor(constructor_name, vec![SurfaceType::Name(primitive.into())]),
        );

        let ty = builtin_fn_signature_type(&env, &builtin)
            .expect("Proc/P with exactly one primitive argument should resolve");

        match ty {
            Type::Fn(params, ret) => {
                prop_assert_eq!(params, vec![expected_arg.clone()]);
                match *ret {
                    Type::Constructor { name, args, .. } => {
                        prop_assert_eq!(name.name, constructor_name);
                        prop_assert_eq!(args, vec![expected_arg]);
                    }
                    other => prop_assert!(false, "expected {constructor_name}<...>, got {other:?}"),
                }
            }
            other => prop_assert!(false, "expected function type, got {other:?}"),
        }
    }
}
