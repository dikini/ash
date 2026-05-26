//! TASK-959 coverage for preferred pure closure arrow syntax.

use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::parse_module::parse_fn_definition;
use ash_parser::surface::{BinaryOp, Definition, Expr};

fn parse_expr_complete(source: &str) -> Result<Expr, String> {
    let mut input = new_input(source);
    let parsed = expr(&mut input).map_err(|err| format!("{err:?}"))?;
    if !input.input.trim().is_empty() {
        return Err(format!("parser left trailing input: {:?}", input.input));
    }
    Ok(parsed)
}

#[test]
fn pure_closure_arrow_single_param_parses_as_fn_def() {
    let parsed = parse_expr_complete("|x| -> x + 1").expect("pure closure arrow should parse");

    let Expr::FnDef {
        params,
        return_type,
        body,
        ..
    } = parsed
    else {
        panic!("expected closure shorthand to lower to surface FnDef");
    };

    assert_eq!(params.len(), 1);
    assert_eq!(params[0].0.as_ref(), "x");
    assert!(params[0].1.is_none());
    assert!(return_type.is_none());
    assert!(
        matches!(
            body.as_ref(),
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ),
        "expected closure body to parse as addition, got {body:?}"
    );
}

#[test]
fn pure_closure_arrow_two_params_parses_as_fn_def() {
    let parsed = parse_expr_complete("|x, y| -> x + y").expect("pure closure arrow should parse");

    let Expr::FnDef { params, body, .. } = parsed else {
        panic!("expected closure shorthand to lower to surface FnDef");
    };

    assert_eq!(params.len(), 2);
    assert_eq!(params[0].0.as_ref(), "x");
    assert_eq!(params[1].0.as_ref(), "y");
    assert!(
        matches!(
            body.as_ref(),
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ),
        "expected closure body to parse as addition, got {body:?}"
    );
}

#[test]
fn old_fat_arrow_closure_is_not_silent_pure_shorthand() {
    let result = parse_expr_complete("|x| => x + 1");

    assert!(
        result.is_err(),
        "old fat-arrow closure shorthand must not parse as pure FnDef: {result:?}"
    );
}

#[test]
fn closure_arrow_does_not_steal_match_arm_fat_arrow() {
    let mut input = new_input("fn describe(n: Int) -> Int { match n { 0 => 1 } }");
    let parsed = parse_fn_definition(&mut input).expect("match arm fat-arrow should still parse");

    let Definition::Function(function) = parsed else {
        panic!("expected function definition");
    };
    let Expr::Block { tail_expr, .. } = function.body else {
        panic!("expected function body block");
    };
    let tail = tail_expr.expect("function body should have tail expression");
    let Expr::Match { arms, .. } = tail.as_ref() else {
        panic!("expected match expression, got {tail:?}");
    };

    assert_eq!(arms.len(), 1);
}
