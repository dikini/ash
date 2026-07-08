use ash_parser::surface::{Definition, Type};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn assert_parse_rejected(source: &str) {
    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "source should be rejected: {source}"
    );
}

fn first_interface(source: &str) -> ash_parser::surface::InterfaceDef {
    parse(source)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("interface should be parsed")
}

fn assert_name(ty: &Type, expected: &str) {
    match ty {
        Type::Name(name) => assert_eq!(name.as_ref(), expected),
        other => panic!("expected type name {expected}, got {other:?}"),
    }
}

#[test]
fn parses_single_interface_evidence_constraint() {
    let interface = first_interface(
        r#"
        interface Monad<M : * -> *> where M: Applicative {
            unit(Int) -> M<Int>
            bind(M<Int>, (Int) -> M<Int>) -> M<Int>
        }
        "#,
    );

    assert_eq!(interface.name.as_ref(), "Monad");
    assert_eq!(interface.evidence_constraints.len(), 1);
    let constraint = &interface.evidence_constraints[0];
    assert_name(&constraint.subject, "M");
    assert_name(&constraint.interface, "Applicative");
    assert!(constraint.colon_span.end > constraint.colon_span.start);
    assert!(constraint.span.end > constraint.span.start);
}

#[test]
fn parses_comma_separated_interface_evidence_constraints() {
    let interface = first_interface(
        r#"
        interface Traversable<T : * -> *> where T: Functor, T: Foldable {
            traverse(T<Int>) -> List<Int>
        }
        "#,
    );

    let constraints = &interface.evidence_constraints;
    assert_eq!(constraints.len(), 2);
    assert_name(&constraints[0].subject, "T");
    assert_name(&constraints[0].interface, "Functor");
    assert_name(&constraints[1].subject, "T");
    assert_name(&constraints[1].interface, "Foldable");
}

#[test]
fn rejects_generalized_propositions_in_interface_constraint_tail() {
    for source in [
        "interface Bad<T> where T == U { method(T) -> T }",
        "interface Bad<T> where T != U { method(T) -> T }",
        "interface Bad<T> where NonEmpty<T> { method(T) -> T }",
        "interface Bad<T> where T: Applicative + Monad { method(T) -> T }",
    ] {
        assert_parse_rejected(source);
    }
}

#[test]
fn rejects_object_style_interface_extension_syntax() {
    for source in [
        "interface Monad<M> : Applicative<M> { unit(Int) -> M<Int> }",
        "interface Monad<M> extends Applicative<M> { unit(Int) -> M<Int> }",
    ] {
        assert_parse_rejected(source);
    }
}
