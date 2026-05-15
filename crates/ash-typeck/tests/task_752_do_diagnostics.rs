use ash_core::ast::{TypeBody, TypeDef as CoreTypeDef, VariantDef, VariantPayload, Visibility};
use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{ActStmt, DoStmt, DoTarget, Expr, Literal, Name, Type as SurfaceType};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::check_expr::{
    check_expr, do_notation_diagnostics, legacy_act_migration_diagnostics,
};
use ash_typeck::error::ConstructorError;
use winnow::prelude::*;

fn span() -> Span {
    Span::default()
}

fn target(name: &str) -> DoTarget {
    DoTarget {
        name: Name::from(name),
        args: vec![],
        span: span(),
    }
}

fn result_target_with_value_hole() -> DoTarget {
    DoTarget {
        name: Name::from("Result"),
        args: vec![
            SurfaceType::Hole { span: span() },
            SurfaceType::Name(Name::from("Int")),
        ],
        span: span(),
    }
}

fn do_block(target_name: &str, stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target: target(target_name),
        stmts,
        span: span(),
    }
}

fn do_block_with_target(target: DoTarget, stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target,
        stmts,
        span: span(),
    }
}

fn ret(value: Expr) -> DoStmt {
    DoStmt::Return {
        value: Box::new(value),
        span: span(),
    }
}

fn bind_stmt(name: &str, value: Expr) -> DoStmt {
    DoStmt::Bind {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn let_stmt(name: &str, value: Expr) -> DoStmt {
    DoStmt::Let {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn call(module: Option<&str>, func: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        module: module.map(Into::into),
        func: func.into(),
        args,
        span: span(),
    }
}

fn unit_call(module: &str, value: Expr) -> Expr {
    call(Some(module), "unit", vec![value])
}

fn first_error_text(expr: &Expr) -> String {
    let result = check_expr(&TypeEnv::with_builtin_types(), expr);
    assert!(!result.is_ok(), "expected diagnostic error, got {result:?}");
    result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_first_error_has_span(expr: &Expr) {
    let result = check_expr(&TypeEnv::with_builtin_types(), expr);
    let Some(error) = result.errors.first() else {
        panic!("expected at least one error, got {result:?}");
    };
    match error {
        ConstructorError::UnsupportedExpression { span, .. } => {
            let _ = span;
        }
        other => panic!("expected unsupported-expression diagnostic, got {other:?}"),
    }
}

fn parse_expr_source(source: &str) -> Expr {
    let mut input = new_input(source);
    let parsed = expr.parse_next(&mut input).expect(source);
    assert!(
        input.input.is_empty(),
        "parser left trailing input: {:?}",
        input.input
    );
    parsed
}

fn computation_boxed_type_def() -> CoreTypeDef {
    CoreTypeDef {
        name: "Boxed".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Boxed".to_string(),
            fields: vec![],
            payload: VariantPayload::Unit,
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

#[test]
fn unknown_do_target_names_target_and_fix_shape() {
    let expr = do_block("Missing", vec![ret(int_lit(1))]);
    let text = first_error_text(&expr);
    assert!(text.contains("unknown do target 'Missing'"), "{text}");
    assert!(text.contains("Act, Proc, or Workflow"), "{text}");
    assert_first_error_has_span(&expr);
}

#[test]
fn wrong_kind_target_names_expected_computation_kind() {
    let expr = do_block("Int", vec![ret(int_lit(1))]);
    let text = first_error_text(&expr);
    assert!(text.contains("do target Int has kind *"), "{text}");
    assert!(text.contains("expected * -> *"), "{text}");
    assert!(text.contains("Act, Proc, or Workflow"), "{text}");
    assert_first_error_has_span(&expr);
}

#[test]
fn unsupported_target_mentions_missing_dictionary_and_deferred_monad() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&computation_boxed_type_def())
        .expect("Boxed computation constructor should register");
    let expr = do_block("Boxed", vec![ret(int_lit(1))]);
    let result = check_expr(&env, &expr);
    let text = result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("do target Boxed has no MVP dictionary"),
        "{text}"
    );
    assert!(text.contains("Monad<K>"), "{text}");
    assert!(text.contains("deferred"), "{text}");
}

#[test]
fn explicit_target_args_report_deferred_hole_target() {
    let expr = do_block_with_target(result_target_with_value_hole(), vec![ret(int_lit(1))]);
    let text = first_error_text(&expr);
    assert!(text.contains("missing Monad evidence"), "{text}");
    assert!(text.contains("Result<_, Int>"), "{text}");
    assert!(text.contains("SPEC-067 Monad<K>"), "{text}");
}

#[test]
fn pure_rhs_with_bind_suggests_let() {
    let expr = do_block("Act", vec![bind_stmt("x", int_lit(1)), ret(var("x"))]);
    let text = first_error_text(&expr);
    assert!(text.contains("<-"), "{text}");
    assert!(text.contains("found Int"), "{text}");
    assert!(text.contains("use let"), "{text}");
}

#[test]
fn wrong_constructor_in_bind_names_expected_found_and_lift() {
    let expr = do_block(
        "Proc",
        vec![
            bind_stmt("x", do_block("Act", vec![ret(int_lit(1))])),
            ret(var("x")),
        ],
    );
    let text = first_error_text(&expr);
    assert!(text.contains("do:Proc"), "{text}");
    assert!(text.contains("Proc<T>"), "{text}");
    assert!(text.contains("found Act<Int>"), "{text}");
    assert!(text.contains("proc::from_act"), "{text}");
}

#[test]
fn let_monadic_value_warns_to_use_bind_when_sequencing() {
    let expr = do_block(
        "Act",
        vec![let_stmt("x", unit_call("act", int_lit(1))), ret(var("x"))],
    );
    let diagnostics = do_notation_diagnostics(&TypeEnv::with_builtin_types(), &expr);
    let text = diagnostics.join("\n");
    assert!(
        text.contains("let `x` binds monadic value Act<Int>"),
        "{text}"
    );
    assert!(text.contains("x <-"), "{text}");
    assert!(
        text.contains("intentionally want the computation value"),
        "{text}"
    );
}

#[test]
fn missing_final_return_and_return_before_end_are_teaching_oriented() {
    let missing = first_error_text(&do_block("Act", vec![let_stmt("x", int_lit(1))]));
    assert!(
        missing.contains("do block must end with a return statement"),
        "{missing}"
    );

    let early = first_error_text(&do_block(
        "Act",
        vec![ret(int_lit(1)), let_stmt("x", int_lit(2))],
    ));
    assert!(
        early.contains("return must be the last statement in a do block"),
        "{early}"
    );
}

#[test]
fn legacy_ret_and_legacy_bind_emit_migration_diagnostics() {
    let legacy = Expr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(unit_call("act", int_lit(1))),
                span: span(),
            },
            ActStmt::Return {
                value: Box::new(var("x")),
                span: span(),
            },
        ],
        span: span(),
    };
    let text = legacy_act_migration_diagnostics(&legacy).join("\n");
    assert!(text.contains("legacy act bind"), "{text}");
    assert!(text.contains("x <-"), "{text}");
    assert!(text.contains("let x"), "{text}");
    assert!(text.contains("legacy act return"), "{text}");
    assert!(text.contains("ret"), "{text}");
    assert!(text.contains("return"), "{text}");
}

#[test]
fn parsed_act_to_proc_mismatch_has_from_act_hint() {
    let expr = parse_expr_source("do:Proc { x <- do:Act { return 1 }; return x }");
    let text = first_error_text(&expr);
    assert!(text.contains("Act<Int>"), "{text}");
    assert!(text.contains("proc::from_act"), "{text}");
}
