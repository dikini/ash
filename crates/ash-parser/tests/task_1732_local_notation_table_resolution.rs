use ash_parser::Span;
use ash_parser::surface::{
    ExpansionError, NotationAssociativity, NotationFixity, NotationPatternPart,
    build_local_notation_table, normalized_notation_pattern_key,
};

#[test]
fn typed_notation_keys_distinguish_holes_tokens_and_part_boundaries() {
    let span = Span::new(0, 1, 1, 1);
    let hole = normalized_notation_pattern_key(&[NotationPatternPart::Hole { span }]);
    let underscore_token = normalized_notation_pattern_key(&[NotationPatternPart::Token {
        spelling: "_".into(),
        span,
    }]);
    assert_ne!(hole, underscore_token);

    let embedded_space = normalized_notation_pattern_key(&[NotationPatternPart::Token {
        spelling: "a b".into(),
        span,
    }]);
    let two_tokens = normalized_notation_pattern_key(&[
        NotationPatternPart::Token {
            spelling: "a".into(),
            span,
        },
        NotationPatternPart::Token {
            spelling: "b".into(),
            span,
        },
    ]);
    assert_ne!(embedded_space, two_tokens);
}

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
fn diagnostic_raw_mutation_does_not_change_mixfix_duplicate_identity() {
    let mut module = ash_parser::parse_surface_file(
        r#"
        mixfix _ between _ = between
        mixfix _ between _ = between_again
        "#,
    )
    .expect("duplicate mixfix declarations parse before expansion");
    let ash_parser::surface::Definition::Notation(second) = &mut module.definitions[1] else {
        panic!("expected the second notation declaration")
    };
    second.pattern.raw = "diagnostic spelling must not be semantic".into();

    let error = build_local_notation_table(&module)
        .expect_err("structured duplicate identity must ignore diagnostic raw spelling");
    assert!(matches!(
        error,
        ExpansionError::DuplicateNotationDeclaration { operator, .. }
            if operator.as_ref() == "_ between _"
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
