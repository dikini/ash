use ash_parser::new_input;
use ash_parser::surface::{Definition, Expr, Pattern};

fn parse_fn(source: &str) -> Definition {
    let mut input = new_input(source);
    let result = ash_parser::parse_module::parse_fn_definition(&mut input);
    result.expect("parse_fn should succeed")
}

#[test]
fn let_single_identifier_works() {
    let def = parse_fn(r#"fn test() -> Int { let x = 42; x }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function");
    };
    match &f.body {
        Expr::Block { statements, .. } => {
            assert_eq!(statements.len(), 1);
            let ash_parser::surface::BlockStmt::Let { pattern, .. } = &statements[0] else {
                panic!("expected let statement, got: {:?}", statements[0]);
            };
            assert!(matches!(pattern, Pattern::Variable { name, .. } if name.as_ref() == "x"));
        }
        _ => panic!("expected Block body"),
    }
}

#[test]
fn let_record_destructor_works() {
    // NOTE: Shorthand `let { x, y } = p` is NOT supported.
    // Must use explicit rename: `let { x: x, y: y } = p`
    let def = parse_fn(
        r#"fn test() -> Int { let p = Point { x: 10, y: 20 }; let { x: x, y: y } = p; x + y }"#,
    );
    let Definition::Function(f) = def else {
        panic!("expected Function");
    };
    match &f.body {
        Expr::Block { statements, .. } => {
            assert_eq!(statements.len(), 2);
            let ash_parser::surface::BlockStmt::Let { pattern, .. } = &statements[1] else {
                panic!("expected let statement, got: {:?}", statements[1]);
            };
            match pattern {
                Pattern::Record(fields) => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].0.as_ref(), "x");
                    assert!(matches!(
                            &fields[0].1, Pattern::Variable { name, .. } if name.as_ref() == "x"));
                    assert_eq!(fields[1].0.as_ref(), "y");
                    assert!(matches!(
                            &fields[1].1, Pattern::Variable { name, .. } if name.as_ref() == "y"));
                }
                _ => panic!("expected Record pattern, got: {:?}", pattern),
            }
        }
        _ => panic!("expected Block body"),
    }
}

#[test]
fn let_record_destructor_with_rename_works() {
    let def = parse_fn(
        r#"fn test() -> Int { let p = Point { x: 10, y: 20 }; let { x: a, y: b } = p; a + b }"#,
    );
    let Definition::Function(f) = def else {
        panic!("expected Function");
    };
    match &f.body {
        Expr::Block { statements, .. } => {
            assert_eq!(statements.len(), 2);
            let ash_parser::surface::BlockStmt::Let { pattern, .. } = &statements[1] else {
                panic!("expected let statement, got: {:?}", statements[1]);
            };
            match pattern {
                Pattern::Record(fields) => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].0.as_ref(), "x");
                    assert!(matches!(
                            &fields[0].1, Pattern::Variable { name, .. } if name.as_ref() == "a"));
                    assert_eq!(fields[1].0.as_ref(), "y");
                    assert!(matches!(
                            &fields[1].1, Pattern::Variable { name, .. } if name.as_ref() == "b"));
                }
                _ => panic!("expected Record pattern, got: {:?}", pattern),
            }
        }
        _ => panic!("expected Block body"),
    }
}

#[test]
fn record_accessor_works() {
    let def = parse_fn(r#"fn test() -> Int { let p = Point { x: 10, y: 20 }; p.x }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function");
    };
    match &f.body {
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            assert_eq!(statements.len(), 1);
            match tail_expr.as_ref().unwrap().as_ref() {
                Expr::FieldAccess { base, field, .. } => {
                    assert_eq!(field.as_ref(), "x");
                    match base.as_ref() {
                        Expr::Variable { name, .. } => {
                            assert_eq!(name.as_ref(), "p");
                        }
                        _ => panic!("expected Variable base"),
                    }
                }
                _ => panic!("expected FieldAccess, got: {:?}", tail_expr),
            }
        }
        _ => panic!("expected Block body"),
    }
}
#[test]
fn record_accessor_on_fn_field_works() {
    let def =
        parse_fn(r#"fn test() -> Int { let s = Strategy { gen: fn(_ctx) { 42 } }; s.gen(0) }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function");
    };
    match &f.body {
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            assert_eq!(statements.len(), 1);
            // Check that tail_expr is a function application on field access
            match tail_expr.as_ref().unwrap().as_ref() {
                Expr::FnApply { func, args, .. } => {
                    assert_eq!(args.len(), 1);
                    match func.as_ref() {
                        Expr::FieldAccess { base, field, .. } => {
                            assert_eq!(field.as_ref(), "gen");
                            match base.as_ref() {
                                Expr::Variable { name, .. } => {
                                    assert_eq!(name.as_ref(), "s");
                                }
                                _ => panic!("expected Variable base"),
                            }
                        }
                        _ => panic!("expected FieldAccess func, got: {:?}", func),
                    }
                }
                _ => panic!("expected FnApply, got: {:?}", tail_expr),
            }
        }
        _ => panic!("expected Block body"),
    }
}

#[test]
fn let_record_destructor_shorthand_works() {
    // Shorthand `let { x, y } = p` IS supported.
    // Equivalent to `let { x: x, y: y } = p`
    let def =
        parse_fn(r#"fn test() -> Int { let p = Point { x: 10, y: 20 }; let { x, y } = p; x + y }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function");
    };
    match &f.body {
        Expr::Block { statements, .. } => {
            assert_eq!(statements.len(), 2);
            let ash_parser::surface::BlockStmt::Let { pattern, .. } = &statements[1] else {
                panic!("expected let statement, got: {:?}", statements[1]);
            };
            match pattern {
                Pattern::Record(fields) => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].0.as_ref(), "x");
                    assert!(matches!(
                            &fields[0].1, Pattern::Variable { name, .. } if name.as_ref() == "x"));
                    assert_eq!(fields[1].0.as_ref(), "y");
                    assert!(matches!(
                            &fields[1].1, Pattern::Variable { name, .. } if name.as_ref() == "y"));
                }
                _ => panic!("expected Record pattern, got: {:?}", pattern),
            }
        }
        _ => panic!("expected Block body"),
    }
}
