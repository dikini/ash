use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{ComprehensionQualifier, DoTarget, Expr, Literal, Name};
use ash_parser::token::Span;
use ash_typeck::check_expr::{check_expr, do_notation_diagnostics};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};
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

fn bind_qual(name: &str, value: Expr) -> ComprehensionQualifier {
    ComprehensionQualifier::Bind {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn let_qual(name: &str, value: Expr) -> ComprehensionQualifier {
    ComprehensionQualifier::Let {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn comprehension(
    target_name: Option<&str>,
    result: Expr,
    qualifiers: Vec<ComprehensionQualifier>,
) -> Expr {
    Expr::Comprehension {
        result: Box::new(result),
        qualifiers,
        target: target_name.map(target),
        span: span(),
    }
}

fn env_with_act_unit() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable(
        "act::unit",
        Type::Fn(
            vec![Type::Int],
            Box::new(Type::Constructor {
                name: QualifiedName::root("Act"),
                args: vec![Type::Int],
                kind: Kind::Type,
            }),
        ),
    );
    env
}

fn error_text(env: &TypeEnv, expr: &Expr) -> String {
    let result = check_expr(env, expr);
    assert!(!result.is_ok(), "expected diagnostic error, got {result:?}");
    result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_error_contains(env: &TypeEnv, expr: &Expr, needles: &[&str]) {
    let text = error_text(env, expr);
    for needle in needles {
        assert!(text.contains(needle), "expected {needle:?} in:\n{text}");
    }
}

#[test]
fn missing_target_names_comprehension_and_suggests_shape() {
    let env = TypeEnv::with_builtin_types();
    let expr = comprehension(
        None,
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );

    assert_error_contains(
        &env,
        &expr,
        &[
            "comprehension",
            "explicit target",
            "[x | x <- xs]: Act",
            "target inference is deferred",
        ],
    );
}

#[test]
fn wrong_kind_target_names_target_and_expected_kind() {
    let env = TypeEnv::with_builtin_types();
    let expr = comprehension(Some("Int"), var("x"), vec![bind_qual("x", int_lit(1))]);

    assert_error_contains(
        &env,
        &expr,
        &["comprehension", "do target Int", "expected * -> *"],
    );
}

#[test]
fn pure_rhs_with_bind_suggests_let() {
    let env = TypeEnv::with_builtin_types();
    let expr = comprehension(Some("Act"), var("x"), vec![bind_qual("x", int_lit(1))]);

    assert_error_contains(
        &env,
        &expr,
        &["comprehension", "<-", "found Int", "use let"],
    );
}

#[test]
fn wrong_constructor_rhs_names_expected_found_and_explicit_lift() {
    let env = env_with_act_unit();
    let expr = comprehension(
        Some("Proc"),
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );

    assert_error_contains(
        &env,
        &expr,
        &[
            "comprehension",
            "do:Proc",
            "Proc<T>",
            "found Act<Int>",
            "proc::from_act",
        ],
    );
}

#[test]
fn let_monadic_value_reports_nonfatal_comprehension_diagnostic() {
    let env = env_with_act_unit();
    let expr = comprehension(
        Some("Act"),
        var("x"),
        vec![let_qual("x", unit_call("act", int_lit(1)))],
    );

    let text = do_notation_diagnostics(&env, &expr).join("\n");
    assert!(text.contains("comprehension"), "{text}");
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
fn missing_dictionary_does_not_overclaim_future_dictionaries() {
    let env = TypeEnv::with_builtin_types();
    let expr = comprehension(
        Some("Option"),
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );

    let text = error_text(&env, &expr);
    assert!(text.contains("comprehension"), "{text}");
    assert!(text.contains("do target Option"), "{text}");
    assert!(text.contains("missing Monad evidence"), "{text}");
    assert!(text.contains("SPEC-067 Monad<K> evidence"), "{text}");
    assert!(text.contains("Monad<Option>"), "{text}");
    assert!(!text.contains("no MVP dictionary"), "{text}");
    assert!(!text.contains("inferred target"), "{text}");
}

#[test]
fn bare_boolean_qualifier_is_not_accepted_by_parser() {
    let mut input = new_input("[x | x <- xs, x > 0]: List");
    let parsed = expr.parse_next(&mut input);

    let accepted_full_expr = parsed.is_ok() && input.input.is_empty();
    assert!(
        !accepted_full_expr,
        "bare boolean qualifier must not parse as valid MVP comprehension"
    );
}
