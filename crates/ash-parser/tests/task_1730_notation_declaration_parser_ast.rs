use ash_parser::surface::{Definition, NotationAssociativity, NotationFixity, Visibility};

#[test]
fn parses_representative_notation_declarations() {
    let module = ash_parser::parse_surface_file(
        r#"
        prefix ! = not
        infixl 6 <+> = combine
        infixr 5 <**> = pow
        infix 4 == = eq
        suffix ? = is_present
        mixfix _ between _ and _ = between
        "#,
    )
    .expect("notation declarations parse");

    assert_eq!(module.definitions.len(), 6);
    let Definition::Notation(prefix) = &module.definitions[0] else {
        panic!("expected prefix notation")
    };
    assert_eq!(prefix.visibility, Visibility::Inherited);
    assert!(matches!(
        prefix.fixity,
        NotationFixity::Prefix { precedence: None }
    ));
    assert_eq!(prefix.pattern.raw.as_ref(), "!");
    assert_eq!(prefix.pattern.tokens[0].spelling.as_ref(), "!");
    assert_eq!(prefix.target.name.as_ref(), "not");

    let Definition::Notation(infixl) = &module.definitions[1] else {
        panic!("expected infixl notation")
    };
    assert!(matches!(
        infixl.fixity,
        NotationFixity::Infix {
            associativity: NotationAssociativity::Left,
            precedence: 6
        }
    ));
    assert_eq!(infixl.pattern.raw.as_ref(), "<+>");
    assert_eq!(infixl.pattern.tokens[0].spelling.as_ref(), "<+>");
    assert!(infixl.pattern.span.start < infixl.pattern.span.end);

    let Definition::Notation(infixr) = &module.definitions[2] else {
        panic!("expected infixr notation")
    };
    assert!(matches!(
        infixr.fixity,
        NotationFixity::Infix {
            associativity: NotationAssociativity::Right,
            precedence: 5
        }
    ));

    let Definition::Notation(infix) = &module.definitions[3] else {
        panic!("expected infix notation")
    };
    assert!(matches!(
        infix.fixity,
        NotationFixity::Infix {
            associativity: NotationAssociativity::Nonassoc,
            precedence: 4
        }
    ));

    let Definition::Notation(suffix) = &module.definitions[4] else {
        panic!("expected suffix notation")
    };
    assert!(matches!(
        suffix.fixity,
        NotationFixity::Suffix { precedence: None }
    ));

    let Definition::Notation(mixfix) = &module.definitions[5] else {
        panic!("expected mixfix notation")
    };
    assert!(matches!(mixfix.fixity, NotationFixity::Mixfix));
    assert_eq!(mixfix.pattern.raw.as_ref(), "_ between _ and _");
}

#[test]
fn parses_qualified_callable_target_without_resolving_it() {
    let module = ash_parser::parse_surface_file("pub infixl 7 <+> = Math::combine")
        .expect("qualified notation target parses");
    let Definition::Notation(decl) = &module.definitions[0] else {
        panic!("expected notation")
    };
    assert_eq!(decl.visibility, Visibility::Public);
    assert_eq!(
        decl.target.module.as_ref().map(|name| name.as_ref()),
        Some("Math")
    );
    assert_eq!(decl.target.name.as_ref(), "combine");
}
