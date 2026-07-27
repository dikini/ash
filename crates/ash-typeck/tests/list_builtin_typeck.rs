//! TASK-639: Typecheck list ops at the ash-typeck level.
//!
//! These tests verify that `builtin_fn_signature_type` correctly produces
//! polymorphic types for list builtin declarations, and that `check_expr`
//! resolves calls like `len([1,2,3])` as Int and `head([1,2,3])` as the
//! element type.

use ash_parser::surface::{BuiltinFnDef, Expr, Literal, Param, Type as SurfaceType, Visibility};
use ash_parser::token::Span;
use ash_typeck::builtin_fn_signature_type;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;

fn span() -> Span {
    Span::default()
}

/// Build a `builtin fn len<a>(list: List<a>) -> Int` declaration.
fn builtin_len() -> BuiltinFnDef {
    BuiltinFnDef {
        visibility: Visibility::Public,
        name: "len".into(),
        type_params: vec!["a".into()],
        params: vec![Param {
            name: "list".into(),
            name_span: span(),
            ty: SurfaceType::Constructor {
                name: "List".into(),
                args: vec![SurfaceType::Name("a".into())],
            },
        }],
        return_type: SurfaceType::Name("Int".into()),
        proposition_tail: None,
        span: span(),
    }
}

/// Build a `builtin fn head<a>(list: List<a>) -> a` declaration.
fn builtin_head() -> BuiltinFnDef {
    BuiltinFnDef {
        visibility: Visibility::Public,
        name: "head".into(),
        type_params: vec!["a".into()],
        params: vec![Param {
            name: "list".into(),
            name_span: span(),
            ty: SurfaceType::Constructor {
                name: "List".into(),
                args: vec![SurfaceType::Name("a".into())],
            },
        }],
        return_type: SurfaceType::Name("a".into()),
        proposition_tail: None,
        span: span(),
    }
}

/// Build a `builtin fn tail<a>(list: List<a>) -> List<a>` declaration.
fn builtin_tail() -> BuiltinFnDef {
    BuiltinFnDef {
        visibility: Visibility::Public,
        name: "tail".into(),
        type_params: vec!["a".into()],
        params: vec![Param {
            name: "list".into(),
            name_span: span(),
            ty: SurfaceType::Constructor {
                name: "List".into(),
                args: vec![SurfaceType::Name("a".into())],
            },
        }],
        return_type: SurfaceType::Constructor {
            name: "List".into(),
            args: vec![SurfaceType::Name("a".into())],
        },
        proposition_tail: None,
        span: span(),
    }
}

/// Helper: typecheck an expression and return the fully-resolved type.
fn infer(env: &TypeEnv, expr: &Expr) -> Type {
    let result = check_expr(env, expr);
    assert!(result.is_ok(), "typecheck failed: {:?}", result.errors);
    result.substitution.apply(&result.ty)
}

/// Build an env with `len` bound from the builtin signature.
fn env_with_len() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    let len_ty =
        builtin_fn_signature_type(&env, &builtin_len()).expect("len signature should resolve");
    env.bind_variable("len", len_ty);
    env
}

/// Build an env with `head` bound from the builtin signature.
fn env_with_head() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    let head_ty =
        builtin_fn_signature_type(&env, &builtin_head()).expect("head signature should resolve");
    env.bind_variable("head", head_ty);
    env
}

/// Build an env with both `len` and `head` bound.
fn env_with_len_and_head() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    let len_ty =
        builtin_fn_signature_type(&env, &builtin_len()).expect("len signature should resolve");
    env.bind_variable("len", len_ty);
    let head_ty =
        builtin_fn_signature_type(&env, &builtin_head()).expect("head signature should resolve");
    env.bind_variable("head", head_ty);
    env
}

// ---------------------------------------------------------------------------
// Test 1: builtin_fn_signature_type produces correct type for len
// ---------------------------------------------------------------------------

#[test]
fn len_signature_produces_fn_type() {
    let env = TypeEnv::with_builtin_types();
    let ty = builtin_fn_signature_type(&env, &builtin_len()).expect("len signature should resolve");

    match &ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 1, "len should take 1 parameter");
            // Return type should be Int
            assert_eq!(**ret, Type::Int, "len should return Int");
            // Parameter should be a List-like constructor with a type var arg
            match &params[0] {
                Type::Constructor { name, args, .. } => {
                    assert_eq!(name.name, "List", "Expected List<...> parameter");
                    assert_eq!(args.len(), 1, "List should have 1 type argument");
                    assert!(
                        matches!(args[0], Type::Var(_)),
                        "Expected type var arg, got {:?}",
                        args[0]
                    );
                }
                Type::List(inner) => {
                    assert!(
                        matches!(inner.as_ref(), Type::Var(_)),
                        "Expected List<type_var>, got List<{:?}",
                        inner
                    );
                }
                other => panic!("Expected List<_> or Constructor parameter, got {other:?}"),
            }
        }
        other => panic!("Expected Fn type for len, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 2: len([1,2,3]) typechecks as Int
// ---------------------------------------------------------------------------

#[test]
fn len_of_int_list_typechecks_as_int() {
    let env = env_with_len();

    let expr = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::Int(1),
            Literal::Int(2),
            Literal::Int(3),
        ]))],
        span: span(),
    };

    let ty = infer(&env, &expr);
    assert_eq!(ty, Type::Int, "len([1,2,3]) should typecheck as Int");
}

// ---------------------------------------------------------------------------
// Test 3: head([1,2,3]) typechecks as Int (element type)
// ---------------------------------------------------------------------------

#[test]
fn head_of_int_list_typechecks_as_int() {
    let env = env_with_head();

    let expr = Expr::Call {
        func: "head".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::Int(1),
            Literal::Int(2),
            Literal::Int(3),
        ]))],
        span: span(),
    };

    let ty = infer(&env, &expr);
    assert_eq!(
        ty,
        Type::Int,
        "head([1,2,3]) should typecheck as Int (element type)"
    );
}

// ---------------------------------------------------------------------------
// Test 4: head(["a","b"]) typechecks as String
// ---------------------------------------------------------------------------

#[test]
fn head_of_string_list_typechecks_as_string() {
    let env = env_with_head();

    let expr = Expr::Call {
        func: "head".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::String("a".into()),
            Literal::String("b".into()),
        ]))],
        span: span(),
    };

    let ty = infer(&env, &expr);
    assert_eq!(
        ty,
        Type::String,
        "head([\"a\",\"b\"]) should typecheck as String (element type)"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Polymorphic len calls with different types are independent
// ---------------------------------------------------------------------------

#[test]
fn len_polymorphic_calls_independent() {
    let env = env_with_len();

    // First call: len([1, 2, 3]) -> Int
    let expr_int = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::Int(1),
            Literal::Int(2),
            Literal::Int(3),
        ]))],
        span: span(),
    };
    let ty1 = infer(&env, &expr_int);
    assert_eq!(ty1, Type::Int, "len([1,2,3]) should return Int");

    // Second call: len(["a", "b"]) -> Int (different element type, same return)
    let expr_str = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::String("a".into()),
            Literal::String("b".into()),
        ]))],
        span: span(),
    };
    let ty2 = infer(&env, &expr_str);
    assert_eq!(ty2, Type::Int, "len([\"a\",\"b\"]) should return Int");
}

// ---------------------------------------------------------------------------
// Test 6: Both len and head in same environment work
// ---------------------------------------------------------------------------

#[test]
fn len_and_head_in_same_env() {
    let env = env_with_len_and_head();

    // len([1, 2, 3]) + head([4, 5, 6]) should typecheck as Int + Int
    let expr = Expr::Binary {
        op: ash_parser::surface::BinaryOp::Add,
        raw_operator: None,
        left: Box::new(Expr::Call {
            func: "len".into(),
            module: None,
            args: vec![Expr::Literal(Literal::List(vec![
                Literal::Int(1),
                Literal::Int(2),
                Literal::Int(3),
            ]))],
            span: span(),
        }),
        right: Box::new(Expr::Call {
            func: "head".into(),
            module: None,
            args: vec![Expr::Literal(Literal::List(vec![
                Literal::Int(4),
                Literal::Int(5),
                Literal::Int(6),
            ]))],
            span: span(),
        }),
        span: span(),
    };

    let ty = infer(&env, &expr);
    assert_eq!(
        ty,
        Type::Int,
        "len([1,2,3]) + head([4,5,6]) should typecheck as Int"
    );
}

// ---------------------------------------------------------------------------
// Test 7: tail signature resolves correctly
// ---------------------------------------------------------------------------

#[test]
fn tail_signature_produces_fn_type() {
    let env = TypeEnv::with_builtin_types();
    let ty =
        builtin_fn_signature_type(&env, &builtin_tail()).expect("tail signature should resolve");

    match &ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 1, "tail should take 1 parameter");
            // Return type should be a List-like Constructor with type var
            match ret.as_ref() {
                Type::Constructor { name, args, .. } => {
                    assert_eq!(name.name, "List", "Expected List<...> return for tail");
                    assert_eq!(args.len(), 1, "List should have 1 type argument");
                    assert!(
                        matches!(args[0], Type::Var(_)),
                        "Expected type var arg, got {:?}",
                        args[0]
                    );
                }
                Type::List(inner) => {
                    assert!(
                        matches!(inner.as_ref(), Type::Var(_)),
                        "Expected List<type_var> return, got List<{:?}",
                        inner
                    );
                }
                other => {
                    panic!("Expected List<_> or Constructor return type for tail, got {other:?}")
                }
            }
            // Parameter should also be List-like with type var
            match &params[0] {
                Type::Constructor { name, args, .. } => {
                    assert_eq!(name.name, "List", "Expected List<...> parameter for tail");
                    assert_eq!(args.len(), 1, "List should have 1 type argument");
                    assert!(
                        matches!(args[0], Type::Var(_)),
                        "Expected type var arg, got {:?}",
                        args[0]
                    );
                }
                Type::List(inner) => {
                    assert!(
                        matches!(inner.as_ref(), Type::Var(_)),
                        "Expected List<type_var> parameter, got List<{:?}",
                        inner
                    );
                }
                other => {
                    panic!("Expected List<_> or Constructor parameter for tail, got {other:?}")
                }
            }
        }
        other => panic!("Expected Fn type for tail, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 8: head signature produces correct polymorphic type
// ---------------------------------------------------------------------------

#[test]
fn head_signature_produces_fn_type_with_polymorphic_return() {
    let env = TypeEnv::with_builtin_types();
    let ty =
        builtin_fn_signature_type(&env, &builtin_head()).expect("head signature should resolve");

    match &ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 1, "head should take 1 parameter");
            // Return type should be a type variable (the element type `a`)
            assert!(
                matches!(ret.as_ref(), Type::Var(_)),
                "head return type should be a type variable, got {:?}",
                ret
            );
        }
        other => panic!("Expected Fn type for head, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 9: len + head with heterogeneous list operations
// ---------------------------------------------------------------------------

#[test]
fn len_of_head_result_works() {
    // This tests: len(tail([1, 2, 3]))
    // tail returns List<Int>, len takes List<a> -> Int
    let mut env = TypeEnv::with_builtin_types();
    let len_ty =
        builtin_fn_signature_type(&env, &builtin_len()).expect("len signature should resolve");
    env.bind_variable("len", len_ty);
    let tail_ty =
        builtin_fn_signature_type(&env, &builtin_tail()).expect("tail signature should resolve");
    env.bind_variable("tail", tail_ty);

    let expr = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Call {
            func: "tail".into(),
            module: None,
            args: vec![Expr::Literal(Literal::List(vec![
                Literal::Int(1),
                Literal::Int(2),
                Literal::Int(3),
            ]))],
            span: span(),
        }],
        span: span(),
    };

    let ty = infer(&env, &expr);
    assert_eq!(ty, Type::Int, "len(tail([1,2,3])) should typecheck as Int");
}
