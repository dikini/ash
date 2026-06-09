use super::support::*;

#[test]
fn parse_simple_fn() {
    let def = parse_fn(r#"fn add(a: Int, b: Int) -> Int { a + b }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "add");
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].name.as_ref(), "a");
    assert_eq!(f.params[1].name.as_ref(), "b");
    assert!(f.return_type.is_some());
}

#[test]
fn parse_fn_with_keyword_name_then() {
    let def = parse_fn(r#"fn then(a: Int, b: Int) -> Int { b }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "then");
    assert_eq!(f.params.len(), 2);
}

#[test]
fn parse_fn_with_keyword_name_guard() {
    let def = parse_fn(r#"fn guard(a: Int) -> Int { a }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "guard");
    assert_eq!(f.params.len(), 1);
}

#[test]
fn parse_pub_fn() {
    let def = parse_fn(r#"pub fn helper(n: Int) -> Int { n + 1 }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "helper");
    // Visibility should not be Inherited (the default)
    assert!(
        !matches!(f.visibility, ash_parser::surface::Visibility::Inherited),
        "expected pub visibility"
    );
}

// ---------------------------------------------------------------------------
// 10. fn accepts nested fn at parse time; lowering desugars Expr::Block to nested Expr::Let
// ---------------------------------------------------------------------------
