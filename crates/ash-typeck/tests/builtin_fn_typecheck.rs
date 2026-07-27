//! Tests for TASK-619: Typechecker resolves `builtin fn` type signatures.
//!
//! Builtin fns type identically to `pub fn` (pure, `Type::Fn(params, ret)`).
//! The typechecker should recognize builtin fn definitions and resolve their
//! type signatures so calls to builtin fns typecheck correctly.

use ash_parser::surface::{
    BuiltinFnDef, Definition, Expr, FnDef, Literal, Param, Program, ProgramEntry,
    Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::type_check_program;

fn span() -> Span {
    Span::default()
}

fn entry_returning(expr: Expr, return_ty: SurfaceType) -> FnDef {
    FnDef {
        visibility: Visibility::Inherited,
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        return_type: Some(return_ty),
        proposition_tail: None,
        contract: None,
        body: expr,
        span: span(),
    }
}

fn program_with_entry(mut definitions: Vec<Definition>, entry: FnDef) -> Program {
    let entry_name = entry.name.clone();
    let entry_span = entry.span;
    definitions.push(Definition::Function(entry));
    Program {
        definitions,
        entry: ProgramEntry {
            function: entry_name,
            span: entry_span,
        },
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
                name_span: span(),
                ty: SurfaceType::Name("Int".into()),
            },
            Param {
                name: "b".into(),
                name_span: span(),
                ty: SurfaceType::Name("Int".into()),
            },
        ],
        return_type: SurfaceType::Name("Int".into()),
        proposition_tail: None,
        span: span(),
    }
}

#[test]
fn builtin_fn_call_typechecks_correctly() {
    // builtin fn add(a: Int, b: Int) -> Int;
    // workflow main -> Int { return add(1, 2); }
    let program = program_with_entry(
        vec![Definition::BuiltinFn(builtin_add())],
        entry_returning(
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
    );

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
    let program = program_with_entry(
        vec![Definition::BuiltinFn(builtin_add())],
        entry_returning(
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
    );

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
    let program = program_with_entry(
        vec![Definition::BuiltinFn(builtin_add())],
        entry_returning(
            Expr::Call {
                func: "add".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(1))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    );

    let result = type_check_program(&program);
    assert!(
        result.is_err(),
        "expected wrong-arg-count call to builtin fn to fail, got {result:?}"
    );
}

#[test]
fn builtin_fn_wrong_arg_type_fails_typecheck() {
    // builtin fn add(a: Int, b: Int) -> Int;
    // fn main() -> Int { add("hello", 2) }  // wrong type
    let program = program_with_entry(
        vec![Definition::BuiltinFn(builtin_add())],
        entry_returning(
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
    );

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
            name_span: span(),
            ty: SurfaceType::Name("T".into()),
        }],
        return_type: SurfaceType::Name("T".into()),
        proposition_tail: None,
        span: span(),
    };

    let program = program_with_entry(
        vec![Definition::BuiltinFn(builtin_id)],
        entry_returning(
            Expr::Call {
                func: "id".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(42))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    );

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
    let double_fn = FnDef {
        visibility: Visibility::Inherited,
        name: "double".into(),
        type_params: vec![],
        params: vec![Param {
            name: "x".into(),
            name_span: span(),
            ty: SurfaceType::Name("Int".into()),
        }],
        return_type: Some(SurfaceType::Name("Int".into())),
        proposition_tail: None,
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

    let program = program_with_entry(
        vec![
            Definition::BuiltinFn(builtin_add()),
            Definition::Function(double_fn),
        ],
        entry_returning(
            Expr::Call {
                func: "double".into(),
                module: None,
                args: vec![Expr::Literal(Literal::Int(3))],
                span: span(),
            },
            SurfaceType::Name("Int".into()),
        ),
    );

    let result = type_check_program(&program);
    assert!(
        result.is_ok(),
        "expected builtin fn + regular fn coexisting to typecheck, got {result:?}"
    );
}
