#[test]
fn task_827_public_type_fn_declaration_syntax_is_rejected() {
    let result = ash_parser::parse_surface_file(
        r#"
        type fn Append<Xs, Ys> = Ys;
        "#,
    );

    assert!(
        result.is_err(),
        "Phase 112 must not add public source `type fn` declaration syntax"
    );
}

#[test]
fn task_827_public_type_fn_with_equation_body_is_rejected() {
    let result = ash_parser::parse_surface_file(
        r#"
        pub type fn Normalize<T> {
            Normalize<T> = T;
        }
        "#,
    );

    assert!(
        result.is_err(),
        "fixture equations remain internal and must not be parsed from source"
    );
}
