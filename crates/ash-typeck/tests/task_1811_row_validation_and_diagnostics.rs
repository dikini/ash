//! TASK-1811 row validation diagnostics before Core lowering.

use ash_parser::surface::ComputationRowItem;
use ash_parser::token::Span;
use ash_typeck::TypeCheckError;

fn parse_program(source: &str) -> ash_parser::surface::Program {
    let module = ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("source should parse before row validation: {errors:?}"));
    ash_parser::surface::Program {
        definitions: module.definitions,
        helper_workflows: Vec::new(),
        workflow: module.workflow.expect("fixture should include workflow"),
    }
}

fn typecheck_error_text(source: &str) -> String {
    let program = parse_program(source);
    ash_typeck::type_check_program(&program)
        .expect_err("fixture should fail row validation")
        .to_string()
}

fn assert_typecheck_err_contains(source: &str, expected: &[&str]) {
    let text = typecheck_error_text(source);
    for fragment in expected {
        assert!(
            text.contains(fragment),
            "expected error to contain {fragment:?}, got:\n{text}"
        );
    }
}

fn first_where_row_items_mut(
    program: &mut ash_parser::surface::Program,
) -> &mut Vec<ComputationRowItem> {
    let ash_parser::surface::Definition::Function(function) = &mut program.definitions[0] else {
        panic!("expected first definition to be a function");
    };
    &mut function
        .proposition_tail
        .as_mut()
        .and_then(|tail| tail.row.as_mut())
        .expect("fixture should have where row")
        .row
        .items
}

#[test]
fn duplicate_inline_and_expanded_callable_rows_are_rejected() {
    assert_typecheck_err_contains(
        r#"
        fn read(path: String) -> {fs.read} Int
        where
            row { evidence read_allowed }
        {
            0
        }

        workflow main { done }
        "#,
        &["row", "specified twice", "read", "inline", "where"],
    );
}

#[test]
fn predicate_like_row_families_fail_closed() {
    for (family, item) in [
        ("requires", "requires_fact"),
        ("ensures", "ensures_fact"),
        ("invariant", "invariant_fact"),
        ("law", "law_fact"),
        ("proof", "proof_fact"),
    ] {
        assert_typecheck_err_contains(
            &format!(
                r#"
                fn guarded() -> Int
                where
                    row {{ {item} }}
                {{
                    0
                }}

                workflow main {{ done }}
                "#
            ),
            &["unsupported", "row", family, "evidence"],
        );
    }
}

#[test]
fn row_tail_must_be_final_even_for_malformed_surface_carriers() {
    let mut program = parse_program(
        r#"
        fn guarded() -> Int
        where
            row { | r }
        {
            0
        }

        workflow main { done }
        "#,
    );
    first_where_row_items_mut(&mut program).push(ComputationRowItem::Operation {
        path: vec!["fs".into(), "read".into()],
        span: Span::default(),
    });

    let text = ash_typeck::type_check_program(&program)
        .expect_err("malformed carrier should fail row validation")
        .to_string();
    assert!(
        text.contains("row tail") && text.contains("final"),
        "expected row tail final diagnostic, got:\n{text}"
    );
}

#[test]
fn duplicate_row_tails_fail_closed_even_for_malformed_surface_carriers() {
    let mut program = parse_program(
        r#"
        fn guarded() -> Int
        where
            row { | r }
        {
            0
        }

        workflow main { done }
        "#,
    );
    first_where_row_items_mut(&mut program).push(ComputationRowItem::Tail {
        variable: "s".into(),
        span: Span::default(),
    });

    let text = ash_typeck::type_check_program(&program)
        .expect_err("malformed carrier should fail row validation")
        .to_string();
    assert!(
        text.contains("duplicate row tail"),
        "expected duplicate row tail diagnostic, got:\n{text}"
    );
}

#[test]
fn supported_row_families_are_not_rejected_by_validation() {
    let program = parse_program(
        r#"
        fn guarded() -> Int
        where
            row {
                fs.read,
                resource File read,
                role Reader,
                policy AllowRead,
                fail ReadFailure,
                evidence read_allowed,
                group FsReads
            }
        {
            0
        }

        workflow main { done }
        "#,
    );

    let result = ash_typeck::type_check_program(&program);
    assert!(
        !matches!(&result, Err(TypeCheckError::TypeEnv(err)) if err.to_string().contains("row")),
        "supported row families should not fail row validation, got: {result:?}"
    );
}
