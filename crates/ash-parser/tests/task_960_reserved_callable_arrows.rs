//! TASK-960 parser coverage for reserved tower callable arrows.

use ash_parser::surface::{Definition, Expr};

fn parse_error_text(source: &str) -> String {
    ash_parser::parse_surface_file(source)
        .expect_err("source should be rejected")
        .into_iter()
        .map(|err| err.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_reserved(source: &str, arrow: &str, stratum: &str) {
    let text = parse_error_text(source);
    assert!(
        text.contains(&format!("{stratum} callable syntax is reserved")),
        "expected reserved {stratum} callable diagnostic for {arrow}, got:\n{text}"
    );
    assert!(
        text.contains(arrow),
        "diagnostic should mention reserved arrow {arrow}, got:\n{text}"
    );
}

fn assert_closure_reserved(source: &str, arrow: &str, stratum: &str) {
    let text = parse_error_text(source);
    assert!(
        text.contains(&format!("{stratum} closures are reserved")),
        "expected reserved {stratum} closure diagnostic for {arrow}, got:\n{text}"
    );
    assert!(
        text.contains(arrow),
        "diagnostic should mention reserved arrow {arrow}, got:\n{text}"
    );
}

fn assert_reserved_in_type_contexts(arrow: &str, stratum: &str) {
    assert_reserved(
        &format!("type Handler = (Int) {arrow} Bool;"),
        arrow,
        stratum,
    );
    assert_reserved(
        &format!("fn accept(handler: (Int) {arrow} Bool) -> Bool {{ true }}"),
        arrow,
        stratum,
    );
    assert_reserved(
        &format!("fn make() -> (Int) {arrow} Bool {{ true }}"),
        arrow,
        stratum,
    );
    assert_reserved(
        &format!("builtin fn accept(handler: (Int) {arrow} Bool) -> Bool;"),
        arrow,
        stratum,
    );
    assert_reserved(
        &format!("interface Callable {{ invoke((Int) {arrow} Bool) -> Bool }}"),
        arrow,
        stratum,
    );
}

fn assert_reserved_in_closure_contexts(arrow: &str, stratum: &str) {
    assert_closure_reserved(
        &format!("fn bad() -> Int {{ let f = |x: Int| {arrow} {{ x }}; 0 }}"),
        arrow,
        stratum,
    );
    assert_closure_reserved(
        &format!("fn bad() -> Int {{ apply(|x: Int| {arrow} {{ x }}) }}"),
        arrow,
        stratum,
    );
    assert_closure_reserved(
        &format!("fn bad() -> Int {{ |x: Int| {arrow} {{ x }} }}"),
        arrow,
        stratum,
    );
}

#[test]
fn act_callable_type_arrow_is_reserved() {
    assert_reserved_in_type_contexts("-*>", "Act");
}

#[test]
fn proc_callable_type_arrow_is_reserved() {
    assert_reserved_in_type_contexts("=>", "Proc");
}

#[test]
fn workflow_callable_type_arrow_is_reserved() {
    assert_reserved_in_type_contexts("=*>", "Workflow");
}

#[test]
fn act_closure_arrow_is_reserved() {
    assert_reserved_in_closure_contexts("-*>", "Act");
}

#[test]
fn proc_closure_arrow_is_reserved() {
    assert_reserved_in_closure_contexts("=>", "Proc");
}

#[test]
fn workflow_closure_arrow_is_reserved() {
    assert_reserved_in_closure_contexts("=*>", "Workflow");
}

#[test]
fn match_arm_fat_arrow_remains_legal() {
    let module =
        ash_parser::parse_surface_file("fn describe(n: Int) -> Int { match n { 0 => 1, _ => 2 } }")
            .expect("match-arm fat arrows should remain legal outside closure contexts");

    let Definition::Function(function) = &module.definitions[0] else {
        panic!("expected function definition");
    };
    let Expr::Block { tail_expr, .. } = &function.body else {
        panic!("expected function body block");
    };
    let Expr::Match { arms, .. } = tail_expr.as_deref().expect("function should have tail") else {
        panic!("expected match tail expression");
    };

    assert_eq!(arms.len(), 2);
}

#[test]
fn reserved_arrows_allow_comments_between_callable_tokens() {
    assert_reserved("type Handler = (Int) /* reserved */ => Bool;", "=>", "Proc");
    assert_closure_reserved(
        "fn bad() -> Int { let f = |x: Int| /* reserved */ => { x }; 0 }",
        "=>",
        "Proc",
    );
}

#[test]
fn unrelated_parse_error_does_not_steal_parenthesized_match_arm_fat_arrow() {
    let text = parse_error_text("fn f(n: Int) -> Int { match n { (0) => 1, _ => 2 } }\nfn broken(");

    assert!(
        !text.contains("Proc callable syntax is reserved"),
        "parenthesized match-arm fat arrow must not be reported as reserved callable syntax: {text}"
    );
}

#[test]
fn unrelated_parse_error_ignores_reserved_looking_strings_and_comments() {
    for source in [
        "fn f() -> Int { let s = \"(Int) => Bool\"; 0 }\nfn broken(",
        "// (Int) => Bool\nfn broken(",
    ] {
        let text = parse_error_text(source);

        assert!(
            !text.contains("Proc callable syntax is reserved"),
            "reserved-looking text in strings/comments must not become a callable diagnostic: {text}"
        );
    }
}
