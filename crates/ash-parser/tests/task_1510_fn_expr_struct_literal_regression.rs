//! TASK-1510 regression tests: fn expressions and closures in multi-field struct literals.
//!
//! These tests verify that anonymous `fn` expressions and closure shorthand work correctly
//! when used as field values in struct literals with two or more fields.

use ash_parser::surface::Expr;

fn parse_ok(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("source should parse")
}

// ---------------------------------------------------------------------------
// Single-field struct literals (baseline — should already work)
// ---------------------------------------------------------------------------

#[test]
fn single_field_struct_with_fn_expr_parses() {
    let defs = parse_ok(
        "type Box = Box { value: (Int) -> Int };\n\
         fn make() -> Box { Box { value: fn(x: Int) -> Int { 42 } } }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

#[test]
fn single_field_struct_with_closure_parses() {
    let defs = parse_ok(
        "type Box = Box { value: (Int) -> Int };\n\
         fn make() -> Box { Box { value: |x: Int| -> 42 } }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

// ---------------------------------------------------------------------------
// Multi-field struct literals with fn expressions (the bug)
// ---------------------------------------------------------------------------

#[test]
fn two_field_struct_with_fn_expr_parses() {
    let defs = parse_ok(
        "type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };\n\
         fn make() -> Pair { \
             Pair { \
                 first: fn(x: Int) -> Int { 42 }, \
                 second: fn(x: Int) -> Int { 43 } \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

#[test]
fn two_field_struct_with_closure_parses() {
    let defs = parse_ok(
        "type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };\n\
         fn make() -> Pair { \
             Pair { \
                 first: |x: Int| -> 42, \
                 second: |x: Int| -> 43 \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

#[test]
fn three_field_struct_with_fn_expr_parses() {
    let defs = parse_ok(
        "type Triple = Triple { a: (Int) -> Int, b: (Int) -> Int, c: (Int) -> Int };\n\
         fn make() -> Triple { \
             Triple { \
                 a: fn(x: Int) -> Int { 1 }, \
                 b: fn(x: Int) -> Int { 2 }, \
                 c: fn(x: Int) -> Int { 3 } \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

#[test]
fn mixed_field_types_with_fn_expr_parses() {
    let defs = parse_ok(
        "type Mixed = Mixed { name: String, f: (Int) -> Int };\n\
         fn make() -> Mixed { \
             Mixed { \
                 name: \"hello\", \
                 f: fn(x: Int) -> Int { 42 } \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

// ---------------------------------------------------------------------------
// Generic struct with fn expressions (Strategy-like)
// ---------------------------------------------------------------------------

#[test]
fn generic_struct_with_fn_expr_parses() {
    let defs = parse_ok(
        "type Strategy<T> = Strategy { gen: (Int) -> T, shrink: (T) -> List<T> };\n\
         fn make() -> Strategy<Int> { \
             Strategy { \
                 gen: fn(x: Int) -> Int { 42 }, \
                 shrink: fn(x: Int) -> List<Int> { [] } \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

#[test]
fn generic_struct_with_closure_parses() {
    let defs = parse_ok(
        "type Strategy<T> = Strategy { gen: (Int) -> T, shrink: (T) -> List<T> };\n\
         fn make() -> Strategy<Int> { \
             Strategy { \
                 gen: |x: Int| -> 42, \
                 shrink: |x: Int| -> [] \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

// ---------------------------------------------------------------------------
// Nested struct literals with fn expressions
// ---------------------------------------------------------------------------

#[test]
fn nested_struct_with_fn_expr_parses() {
    let defs = parse_ok(
        "type Inner = Inner { f: (Int) -> Int };\n\
         type Outer = Outer { inner: Inner };\n\
         fn make() -> Outer { \
             Outer { \
                 inner: Inner { f: fn(x: Int) -> Int { 42 } } \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 3);
}

// ---------------------------------------------------------------------------
// Trailing comma cases
// ---------------------------------------------------------------------------

#[test]
fn two_field_struct_with_trailing_comma_and_fn_expr_parses() {
    let defs = parse_ok(
        "type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };\n\
         fn make() -> Pair { \
             Pair { \
                 first: fn(x: Int) -> Int { 42 }, \
                 second: fn(x: Int) -> Int { 43 }, \
             } \
         }",
    );
    assert_eq!(defs.definitions.len(), 2);
}

// ---------------------------------------------------------------------------
// Verify parsed AST structure for fn expressions in struct fields
// ---------------------------------------------------------------------------

#[test]
fn fn_expr_in_struct_field_has_correct_ast() {
    let defs = parse_ok(
        "type Box = Box { value: (Int) -> Int };\n\
         fn make() -> Box { Box { value: fn(x: Int) -> Int { 42 } } }",
    );

    let fn_def = match &defs.definitions[1] {
        ash_parser::surface::Definition::Function(f) => f,
        _ => panic!("expected function definition"),
    };

    match &fn_def.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => match tail.as_ref() {
            Expr::Constructor { name, fields, .. } => {
                assert_eq!(name.as_ref(), "Box");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0.as_ref(), "value");
                match &fields[0].1 {
                    Expr::FnDef {
                        params,
                        return_type,
                        ..
                    } => {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].0.as_ref(), "x");
                        assert_eq!(return_type.as_ref().map(|t| t.as_ref()), Some("Int"));
                    }
                    other => panic!("expected FnDef, got {:?}", other),
                }
            }
            other => panic!("expected Constructor, got {:?}", other),
        },
        other => panic!("expected Block with tail, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Verify closure shorthand in struct fields has correct AST
// ---------------------------------------------------------------------------

#[test]
fn closure_in_struct_field_has_correct_ast() {
    let defs = parse_ok(
        "type Box = Box { value: (Int) -> Int };\n\
         fn make() -> Box { Box { value: |x: Int| -> 42 } }",
    );

    let fn_def = match &defs.definitions[1] {
        ash_parser::surface::Definition::Function(f) => f,
        _ => panic!("expected function definition"),
    };

    match &fn_def.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => match tail.as_ref() {
            Expr::Constructor { name, fields, .. } => {
                assert_eq!(name.as_ref(), "Box");
                assert_eq!(fields.len(), 1);
                match &fields[0].1 {
                    Expr::FnDef {
                        params,
                        return_type,
                        ..
                    } => {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].0.as_ref(), "x");
                        assert!(
                            return_type.is_none(),
                            "closure shorthand has no return type annotation"
                        );
                    }
                    other => panic!("expected FnDef, got {:?}", other),
                }
            }
            other => panic!("expected Constructor, got {:?}", other),
        },
        other => panic!("expected Block with tail, got {:?}", other),
    }
}
