use ash_parser::surface::{
    ExpansionError, NotationAssociativity, NotationFixity, build_local_notation_table,
};

#[test]
fn local_notation_table_resolves_declared_infix_target() {
    let module = ash_parser::parse_surface_file("infixl 6 <+> = combine")
        .expect("notation declaration parses");
    let table = build_local_notation_table(&module).expect("local notation table builds");
    let entry = table.resolve_infix("<+>").expect("operator resolves");
    assert_eq!(entry.operator.as_ref(), "<+>");
    assert_eq!(entry.target.name.as_ref(), "combine");
    assert!(matches!(
        entry.fixity,
        NotationFixity::Infix {
            associativity: NotationAssociativity::Left,
            precedence: 6
        }
    ));
}

#[test]
fn public_notation_declaration_is_still_only_a_local_table_entry() {
    let module = ash_parser::parse_surface_file("pub infixl 6 <+> = combine")
        .expect("public notation declaration parses");
    let table = build_local_notation_table(&module).expect("local notation table builds");
    let entry = table
        .resolve_infix("<+>")
        .expect("public notation is active locally");
    assert_eq!(entry.operator.as_ref(), "<+>");
    assert_eq!(entry.target.name.as_ref(), "combine");
}

#[test]
fn duplicate_local_notation_declarations_fail_closed() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = combine
        infixl 6 <+> = combine_again
        "#,
    )
    .expect("duplicate notation declarations parse before expansion");
    let err = build_local_notation_table(&module).expect_err("duplicates reject during expansion");
    assert!(matches!(
        err,
        ExpansionError::DuplicateNotationDeclaration { operator, .. } if operator.as_ref() == "<+>"
    ));
}

#[test]
fn conflicting_local_notation_declarations_fail_closed() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = combine
        infixr 7 <+> = combine_right
        "#,
    )
    .expect("conflicting notation declarations parse before expansion");
    let err = build_local_notation_table(&module).expect_err("conflicts reject during expansion");
    assert!(matches!(
        err,
        ExpansionError::DuplicateNotationDeclaration { operator, .. }
        | ExpansionError::ConflictingNotationDeclaration { operator, .. } if operator.as_ref() == "<+>"
    ));
}

#[test]
fn inline_module_notation_declarations_do_not_leak_to_parent_table() {
    let module = ash_parser::parse_surface_file(
        r#"
        mod inner { infixl 6 <+> = combine }
        fn parent(x: Int) -> Int { (x <+>) }
        "#,
    )
    .expect("inline notation and parent section parse");
    let table = build_local_notation_table(&module).expect("parent table builds");
    assert!(table.resolve_infix("<+>").is_none());
    let err = ash_parser::surface::expand_surface_module(module)
        .expect_err("parent cannot use inline-module-local notation");
    assert!(err.to_string().contains("operator section `<+>`"));
}

#[test]
fn parent_notation_declarations_do_not_leak_to_inline_modules() {
    let module = ash_parser::parse_surface_file(
        r#"
        infixl 6 <+> = combine
        mod inner { fn child(x: Int) -> Int { (x <+>) } }
        "#,
    )
    .expect("parent notation and inline section parse");
    let err = ash_parser::surface::expand_surface_module(module)
        .expect_err("inline module cannot use parent-local notation");
    assert!(err.to_string().contains("operator section `<+>`"));
}
