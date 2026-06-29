use ash_parser::surface::{Expr, visit_exprs_in_module};

#[test]
fn reusable_traversal_visits_contract_workflow_and_capability_sites() {
    let module = ash_parser::parse_surface_file(
        r#"
        fn checked(x: Int) -> Int requires: (+ x) { x }
        capability impl NoopKV for KVStore {
            observe get(key: String) returns Option<String> { (+ key) }
        }
        workflow main { ret (+ 1) }
        "#,
    )
    .expect("module with expression-bearing sites parses");

    let mut sections = Vec::new();
    visit_exprs_in_module(&module, &mut |expr| {
        if let Expr::OperatorSection { section } = expr {
            sections.push(section.operator.spelling.to_string());
        }
    });

    assert_eq!(sections, vec!["+", "+", "+"]);
}

#[test]
fn expansion_boundary_uses_traversal_for_non_function_sites() {
    let module = ash_parser::parse_surface_file(
        r#"
        capability impl NoopKV for KVStore {
            observe get(key: String) returns Option<String> { (+ key) }
        }
        "#,
    )
    .expect("module parses");

    let expanded = ash_parser::surface::expand_surface_module(module)
        .expect("built-in operator section elaborates before expanded boundary");
    let mut raw_sections = 0usize;
    visit_exprs_in_module(&expanded.module, &mut |expr| {
        if matches!(expr, Expr::OperatorSection { .. }) {
            raw_sections += 1;
        }
    });
    assert_eq!(raw_sections, 0);
}
