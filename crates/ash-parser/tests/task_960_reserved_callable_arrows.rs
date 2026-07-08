//! TASK-960 parser coverage for removed callable arrows.

use ash_parser::surface::{Definition, Expr};

fn parse_error_text(source: &str) -> String {
    ash_parser::parse_surface_file(source)
        .expect_err("source should be rejected")
        .into_iter()
        .map(|err| err.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_removed_type_arrow(source: &str, arrow: &str) {
    let text = parse_error_text(source);
    assert!(
        text.contains("removed callable arrow syntax is not accepted"),
        "expected removed callable arrow diagnostic for {arrow}, got:\n{text}"
    );
    assert!(
        text.contains(arrow),
        "diagnostic should mention removed arrow {arrow}, got:\n{text}"
    );
}

fn assert_removed_closure_arrow(source: &str, arrow: &str) {
    let text = parse_error_text(source);
    assert!(
        text.contains("removed callable arrow syntax is not accepted"),
        "expected removed closure arrow diagnostic for {arrow}, got:\n{text}"
    );
    assert!(
        text.contains(arrow),
        "diagnostic should mention removed arrow {arrow}, got:\n{text}"
    );
}

fn assert_removed_in_type_contexts(arrow: &str) {
    assert_removed_type_arrow(&format!("type Handler = (Int) {arrow} Bool;"), arrow);
    assert_removed_type_arrow(&format!("type UnaryHandler = Int {arrow} Bool;"), arrow);
    assert_removed_type_arrow(
        &format!("type GenericUnaryHandler = List<Int> {arrow} Bool;"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("type NestedGenericHandler = List<Int {arrow} Bool>;"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("type NestedGenericParenHandler = List<(Int) {arrow} Bool>;"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("type NestedGenericSecondHandler = Map<String, Int {arrow} Bool>;"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("type NestedGenericSecondParenHandler = Map<String, (Int) {arrow} Bool>;"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("type ListElementHandler = [Int {arrow} Bool];"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("type ListElementParenHandler = [(Int) {arrow} Bool];"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("fn accept(handler: (Int) {arrow} Bool) -> Bool {{ true }}"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("fn make() -> (Int) {arrow} Bool {{ true }}"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("builtin fn accept(handler: (Int) {arrow} Bool) -> Bool;"),
        arrow,
    );
    assert_removed_type_arrow(
        &format!("interface Callable {{ invoke((Int) {arrow} Bool) -> Bool }}"),
        arrow,
    );
}

fn assert_removed_in_closure_contexts(arrow: &str) {
    assert_removed_closure_arrow(
        &format!("fn bad() -> Int {{ let f = |x: Int| {arrow} {{ x }}; 0 }}"),
        arrow,
    );
    assert_removed_closure_arrow(
        &format!("fn bad() -> Int {{ apply(|x: Int| {arrow} {{ x }}) }}"),
        arrow,
    );
    assert_removed_closure_arrow(
        &format!("fn bad() -> Int {{ |x: Int| {arrow} {{ x }} }}"),
        arrow,
    );
}

#[test]
fn dash_star_type_arrow_is_removed() {
    assert_removed_in_type_contexts("-*>");
}

#[test]
fn fat_type_arrow_is_removed() {
    assert_removed_in_type_contexts("=>");
}

#[test]
fn equals_star_type_arrow_is_removed() {
    assert_removed_in_type_contexts("=*>");
}

#[test]
fn dash_star_closure_arrow_is_removed() {
    assert_removed_in_closure_contexts("-*>");
}

#[test]
fn fat_closure_arrow_is_removed() {
    assert_removed_in_closure_contexts("=>");
}

#[test]
fn equals_star_closure_arrow_is_removed() {
    assert_removed_in_closure_contexts("=*>");
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
    assert_removed_type_arrow("type Handler = (Int) /* reserved */ => Bool;", "=>");
    assert_removed_closure_arrow(
        "fn bad() -> Int { let f = |x: Int| /* reserved */ => { x }; 0 }",
        "=>",
    );
}

#[test]
fn unrelated_parse_error_does_not_steal_parenthesized_match_arm_fat_arrow() {
    let text = parse_error_text("fn f(n: Int) -> Int { match n { (0) => 1, _ => 2 } }\nfn broken(");

    assert!(
        !text.contains("removed callable arrow syntax is not accepted"),
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
            !text.contains("removed callable arrow syntax is not accepted"),
            "reserved-looking text in strings/comments must not become a callable diagnostic: {text}"
        );
    }
}
