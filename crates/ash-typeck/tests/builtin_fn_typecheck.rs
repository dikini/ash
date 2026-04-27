//! Tests for TASK-619: Typechecker resolves `builtin fn` type signatures.
//!
//! Builtin fns type identically to `pub fn` (pure, `Type::Fn(params, ret)`).
//! The typechecker should recognize builtin fn definitions and resolve their
//! type signatures so calls to builtin fns typecheck correctly.

use ash_parser::surface::{
    BuiltinFnDef, Definition, Expr, Literal, Param, Program, Type as SurfaceType, Visibility,
    Workflow, WorkflowDef,
};
use ash_parser::token::Span;
use ash_typeck::type_check_program;

fn span() -> Span {
    Span::default()
}

fn workflow_returning(expr: Expr, return_ty: SurfaceType) -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(return_ty),
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        body: Workflow::Ret { expr, span: span() },
        contract: None,
        span: span(),
    }
}

/// builtin fn add(a: Int, b: Int) -> Int;
fn builtin_add() -> BuiltinFnDef {
    BuiltinFnDef {
        visibility: Visibility::Public,
        name: "add".into(),
        type_params: vec![],
        params: vec![
            Param {
                name: "a".into(),
                ty: SurfaceType::Name("Int".into()),
            },
            Param {
                name: "b".into(),
                ty: SurfaceType::Name("Int".into()),
            },
        ],
        return_type: SurfaceType::Name("Int".into()),
        span: span(),
    }
}

#[test]
fn builtin_fn_call_typechecks_correctly() {
    // builtin fn add(a: Int, b: Int) -> Int;
    // workflow main -> Int { return add(1, 2); }
    let program = Program {
        definitions: vec![Definition::BuiltinFn(builtin_add())],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "add".into(),
                module: None,
                args: vec![
                    Expr::Literal(Literal::Int(1)),
                    Expr::Literal(Literal::Int(2)),
                ],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected builtin fn call to typecheck, got {result:?}"
    );
}

#[test]
fn builtin_fn_return_type_mismatch_fails() {
    // builtin fn add(a: Int, b: Int) -> Int;
    // workflow main -> String { return add(1, 2); }  // return type mismatch
    let program = Program {
        definitions: vec![Definition::BuiltinFn(builtin_add())],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "add".into(),
                module: None,
                args: vec![
                    Expr::Literal(Literal::Int(1)),
                    Expr::Literal(Literal::Int(2)),
                ],
                span: span(),
            },
            SurfaceType::Name("String".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_err(),
        "expected return type mismatch for builtin fn call to fail, got {result:?}"
    );
}

#[test]
fn builtin_fn_wrong_arg_count_fails_typecheck() {
    // builtin fn add(a: Int, b: Int) -> Int;
    // workflow main -> Int { return add(1); }  // wrong arg count
    let program = Program {
        definitions: vec![Definition::BuiltinFn(builtin_add())],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "add".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(1))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_err(),
        "expected wrong-arg-count call to builtin fn to fail, got {result:?}"
    );
}

#[test]
fn builtin_fn_wrong_arg_type_fails_typecheck() {
    // builtin fn add(a: Int, b: Int) -> Int;
    // workflow main -> Int { return add("hello", 2); }  // wrong type
    let program = Program {
        definitions: vec![Definition::BuiltinFn(builtin_add())],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "add".into(),
                module: None,
                args: vec![
                    Expr::Literal(Literal::String("hello".into())),
                    Expr::Literal(Literal::Int(2)),
                ],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_err(),
        "expected wrong-type arg call to builtin fn to fail, got {result:?}"
    );
}

#[test]
fn builtin_fn_with_generic_type_params() {
    // builtin fn id<T>(x: T) -> T;
    // workflow main -> Int { return id(42); }
    let builtin_id = BuiltinFnDef {
        visibility: Visibility::Public,
        name: "id".into(),
        type_params: vec!["T".into()],
        params: vec![Param {
            name: "x".into(),
            ty: SurfaceType::Name("T".into()),
        }],
        return_type: SurfaceType::Name("T".into()),
        span: span(),
    };

    let program = Program {
        definitions: vec![Definition::BuiltinFn(builtin_id)],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "id".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(42))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected generic builtin fn call to typecheck, got {result:?}"
    );
}

#[test]
fn builtin_fn_coexists_with_regular_fn() {
    // builtin fn add(a: Int, b: Int) -> Int;
    // fn double(x: Int) -> Int { add(x, x) }
    // workflow main -> Int { return double(3); }
    use ash_parser::surface::FnDef;

    let double_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "double".into(),
        type_params: vec![],
        params: vec![Param {
            name: "x".into(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        contract: None,
        body: Expr::Call {
            func: "add".into(),
            module: None,
            args: vec![
                Expr::Variable {
                    name: "x".into(),
                    span: span(),
                },
                Expr::Variable {
                    name: "x".into(),
                    span: span(),
                },
            ],
            span: span(),
        },
        span: span(),
    };

    let program = Program {
        definitions: vec![
            Definition::BuiltinFn(builtin_add()),
            Definition::Function(double_fn),
        ],
        helper_workflows: vec![],
        workflow: workflow_returning(
            Expr::Call {
                func: "double".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(3))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    };

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected builtin fn + regular fn coexisting to typecheck, got {result:?}"
    );
}
