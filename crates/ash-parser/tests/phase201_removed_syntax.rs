//! Phase 201 parser gates for removed surface forms.

fn removed_keyword(parts: &[&str]) -> String {
    parts.concat()
}

fn removed_callable_type(parts: &[&str]) -> String {
    parts.concat()
}

#[test]
fn target_fn_main_module_remains_parseable() {
    let source = "fn main() -> Int { 1 }\n";

    let module = ash_parser::parse_surface_file(source).expect("target fn main should parse");

    assert_eq!(module.definitions.len(), 1);
}

#[test]
fn removed_entry_keyword_is_not_a_current_module_item() {
    let keyword = removed_keyword(&["work", "flow"]);
    let source = format!("{keyword} main {{ return 0 }}");

    let error = ash_parser::parse_surface_file(&source)
        .expect_err("removed entry keyword must not parse as a current module item");
    let text = format!("{error:?}");

    assert!(text.contains("parse error"), "{text}");
}

#[test]
fn removed_capability_declaration_is_not_a_current_module_item() {
    let keyword = removed_keyword(&["cap", "ability"]);
    let source = format!("pub {keyword} Store: observe read() returns String;");

    let error = ash_parser::parse_surface_file(&source)
        .expect_err("removed capability declaration must not parse as a current module item");

    assert_eq!(
        error[0].message,
        "`capability` declarations are removed from target Ash"
    );
}

#[test]
fn removed_capability_interface_is_not_a_current_module_item() {
    let keyword = removed_keyword(&["cap", "ability"]);
    let source = format!("pub {keyword} interface Sensor:\n  observe read() returns Int;");

    let error = ash_parser::parse_surface_file(&source)
        .expect_err("removed capability interface syntax must not parse");

    assert_eq!(
        error[0].message,
        "`capability` declarations are removed from target Ash"
    );
}

#[test]
fn removed_capability_implementation_is_not_a_current_module_item() {
    let keyword = removed_keyword(&["cap", "ability"]);
    let source = format!("{keyword} impl Noop for Sensor {{ observe read() returns Int {{ 0 }} }}");

    let error = ash_parser::parse_surface_file(&source)
        .expect_err("removed capability implementation syntax must not parse");

    assert_eq!(
        error[0].message,
        "`capability` declarations are removed from target Ash"
    );
}

#[test]
fn target_parenthesized_callable_type_remains_parseable() {
    let source = "fn keep(f: (Int, String) -> Bool) -> Bool { true }";

    let module = ash_parser::parse_surface_file(source).expect("target callable type should parse");

    assert_eq!(module.definitions.len(), 1);
}

#[test]
fn removed_fn_constructor_callable_type_is_not_current_syntax() {
    let callable = removed_callable_type(&["F", "n", "(", "Int, String", ")", " -> ", "Bool"]);
    let source = format!("fn keep(f: {callable}) -> Bool {{ true }}");

    let error = ash_parser::parse_surface_file(&source)
        .expect_err("removed callable constructor spelling must not parse");
    let text = format!("{error:?}");

    assert!(text.contains("parse error"), "{text}");
}

#[test]
fn unary_callable_arrow_remains_current_syntax() {
    let callable = removed_callable_type(&["Int", " -> ", "Bool"]);
    let source = format!("fn keep(f: {callable}) -> Bool {{ true }}");

    let module = ash_parser::parse_surface_file(&source)
        .expect("SPEC-072 retains A -> B as the unary callable arrow form");

    assert_eq!(module.definitions.len(), 1);
}
