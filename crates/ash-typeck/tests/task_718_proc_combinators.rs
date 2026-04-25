use ash_parser::surface::{
    BuiltinFnDef, Expr as SurfaceExpr, Param, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::builtin_fn_signature_type;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;
use proptest::prelude::*;
use proptest::test_runner::Config;

fn constructor(name: &str, args: Vec<SurfaceType>) -> SurfaceType {
    SurfaceType::Constructor {
        name: name.into(),
        args,
    }
}

fn builtin(name: &str, params: Vec<SurfaceType>, ret: SurfaceType) -> BuiltinFnDef {
    BuiltinFnDef {
        visibility: Visibility::Public,
        name: name.into(),
        type_params: vec!["A".into(), "B".into()],
        params: params
            .into_iter()
            .enumerate()
            .map(|(idx, ty)| Param {
                name: format!("p{idx}").into(),
                ty,
            })
            .collect(),
        return_type: ret,
        span: Span::default(),
    }
}

fn assert_proc_of(ty: &Type, expected_arg: &Type) {
    match ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.name, "Proc");
            assert_eq!(args, std::slice::from_ref(expected_arg));
        }
        other => panic!("expected Proc<...>, got {other:?}"),
    }
}

fn assert_handle_of(ty: &Type, expected_arg: &Type) {
    match ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.name, "P");
            assert_eq!(args, std::slice::from_ref(expected_arg));
        }
        other => panic!("expected P<...>, got {other:?}"),
    }
}

#[test]
fn proc_unit_signature_typechecks_as_value_to_proc() {
    let env = TypeEnv::with_builtin_types();
    let sig = builtin(
        "unit",
        vec![SurfaceType::Name("Int".into())],
        constructor("Proc", vec![SurfaceType::Name("Int".into())]),
    );

    let ty = builtin_fn_signature_type(&env, &sig).expect("proc::unit signature should resolve");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };
    assert_eq!(params, vec![Type::Int]);
    assert_proc_of(&ret, &Type::Int);
}

#[test]
fn proc_bind_signature_typechecks_dependent_sequencing_shape() {
    let env = TypeEnv::with_builtin_types();
    let sig = builtin(
        "bind",
        vec![
            constructor("Proc", vec![SurfaceType::Name("Int".into())]),
            SurfaceType::Fn(
                vec![SurfaceType::Name("Int".into())],
                Box::new(constructor(
                    "Proc",
                    vec![SurfaceType::Name("String".into())],
                )),
            ),
        ],
        constructor("Proc", vec![SurfaceType::Name("String".into())]),
    );

    let ty = builtin_fn_signature_type(&env, &sig).expect("proc::bind signature should resolve");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };
    assert_proc_of(&params[0], &Type::Int);
    assert!(
        matches!(&params[1], Type::Fn(fn_params, fn_ret) if *fn_params == vec![Type::Int] && matches!(&**fn_ret, Type::Constructor { name, args, .. } if name.name == "Proc" && args == &vec![Type::String]))
    );
    assert_proc_of(&ret, &Type::String);
}

#[test]
fn proc_then_signature_typechecks_discarding_left_value() {
    let env = TypeEnv::with_builtin_types();
    let sig = builtin(
        "then",
        vec![
            constructor("Proc", vec![SurfaceType::Name("Int".into())]),
            constructor("Proc", vec![SurfaceType::Name("Bool".into())]),
        ],
        constructor("Proc", vec![SurfaceType::Name("Bool".into())]),
    );

    let ty = builtin_fn_signature_type(&env, &sig).expect("proc::then signature should resolve");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };
    assert_proc_of(&params[0], &Type::Int);
    assert_proc_of(&params[1], &Type::Bool);
    assert_proc_of(&ret, &Type::Bool);
}

#[test]
fn proc_yield_signature_typechecks_unit_surface_as_proc_null_runtime_shape() {
    let env = TypeEnv::with_builtin_types();
    let sig = builtin(
        "yield",
        vec![],
        constructor("Proc", vec![SurfaceType::Name("Unit".into())]),
    );

    let ty = builtin_fn_signature_type(&env, &sig).expect("proc::yield signature should resolve");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };
    assert!(params.is_empty());
    match *ret {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.name, "Proc");
            assert_eq!(args.len(), 1);
            assert!(match &args[0] {
                Type::Null => true,
                Type::Constructor { name, args, .. } => name.name == "Null" && args.is_empty(),
                _ => false,
            });
        }
        other => panic!("expected Proc<...>, got {other:?}"),
    }
}

#[test]
fn proc_yield_expression_typechecks_as_nullary_proc_returning_null() {
    let env = TypeEnv::with_builtin_types();
    let expr = SurfaceExpr::Call {
        func: "yield".into(),
        module: Some("proc".into()),
        args: vec![],
        span: Span::default(),
    };

    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "proc::yield() should typecheck: {result:?}");
    assert_proc_of(&result.substitution.apply(&result.ty), &Type::Null);
}

#[test]
fn proc_par_builtin_is_registered_as_ordered_two_handle_admission() {
    let env = TypeEnv::with_builtin_types();

    let ty = env
        .lookup_variable("proc::par")
        .expect("TypeEnv should register proc::par");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };

    assert_eq!(params.len(), 2);
    let left_result = match &params[0] {
        Type::Constructor { name, args, .. } if name.name == "Proc" && args.len() == 1 => {
            args[0].clone()
        }
        other => panic!("expected left proc parameter, got {other:?}"),
    };
    let right_result = match &params[1] {
        Type::Constructor { name, args, .. } if name.name == "Proc" && args.len() == 1 => {
            args[0].clone()
        }
        other => panic!("expected right proc parameter, got {other:?}"),
    };

    match *ret {
        Type::Constructor { name, args, .. } if name.name == "Proc" && args.len() == 1 => {
            match &args[0] {
                Type::Record(fields) => {
                    assert_eq!(
                        fields.len(),
                        2,
                        "par should return exactly two ordered child handles"
                    );
                    assert_eq!(fields[0].0.as_ref(), "_0");
                    assert_handle_of(&fields[0].1, &left_result);
                    assert_eq!(fields[1].0.as_ref(), "_1");
                    assert_handle_of(&fields[1].1, &right_result);
                }
                other => {
                    panic!("expected tuple-record payload for proc::par result, got {other:?}")
                }
            }
        }
        other => panic!("expected Proc<...>, got {other:?}"),
    }
}

#[test]
fn proc_scatter_builtin_is_registered_as_ordered_handle_list_admission() {
    let env = TypeEnv::with_builtin_types();

    let ty = env
        .lookup_variable("proc::scatter")
        .expect("TypeEnv should register proc::scatter");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };

    assert_eq!(params.len(), 2);
    let item_ty = match &params[0] {
        Type::List(item) => item.as_ref().clone(),
        Type::Constructor { name, args, .. } if name.name == "List" && args.len() == 1 => {
            args[0].clone()
        }
        other => panic!("expected List<A> input, got {other:?}"),
    };

    let child_result = match &params[1] {
        Type::Fn(fn_params, fn_ret) if fn_params.len() == 1 => {
            assert_eq!(fn_params[0], item_ty);
            match fn_ret.as_ref() {
                Type::Constructor { name, args, .. } if name.name == "Proc" && args.len() == 1 => {
                    args[0].clone()
                }
                other => panic!("expected mapper to return Proc<B>, got {other:?}"),
            }
        }
        other => panic!("expected A -> Proc<B> mapper, got {other:?}"),
    };

    match *ret {
        Type::Constructor { name, args, .. } if name.name == "Proc" && args.len() == 1 => {
            match &args[0] {
                Type::List(inner) => assert_handle_of(inner, &child_result),
                Type::Constructor { name, args, .. } if name.name == "List" && args.len() == 1 => {
                    assert_handle_of(&args[0], &child_result)
                }
                other => panic!("expected Proc<List<P<B>>>, got {other:?}"),
            }
        }
        other => panic!("expected Proc<...>, got {other:?}"),
    }
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, ..Config::default() })]

    #[test]
    fn proc_unit_preserves_primitive_result_type(primitive in prop_oneof![Just("Int"), Just("String"), Just("Bool")]) {
        let env = TypeEnv::with_builtin_types();
        let expected = match primitive {
            "Int" => Type::Int,
            "String" => Type::String,
            "Bool" => Type::Bool,
            other => unreachable!("unexpected primitive generated: {other}"),
        };
        let sig = builtin(
            "unit",
            vec![SurfaceType::Name(primitive.into())],
            constructor("Proc", vec![SurfaceType::Name(primitive.into())]),
        );

        let ty = builtin_fn_signature_type(&env, &sig).expect("proc::unit signature should resolve");
        let Type::Fn(_, ret) = ty else {
            prop_assert!(false, "expected function type");
            return Ok(());
        };
        match *ret {
            Type::Constructor { name, args, .. } => {
                prop_assert_eq!(name.name, "Proc");
                prop_assert_eq!(args, vec![expected]);
            }
            other => prop_assert!(false, "expected Proc<...>, got {other:?}"),
        }
    }
}
