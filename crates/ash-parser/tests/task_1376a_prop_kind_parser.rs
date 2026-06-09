use ash_core::Kind;
use ash_parser::surface::Definition;

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

#[test]
fn parses_prop_as_explicit_kind_atom() {
    let module = parse("prop Holds<P : Prop>;\n");

    let predicate = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::PropositionPredicate(predicate) => Some(predicate),
            _ => None,
        })
        .expect("proposition predicate should be present");

    let annotation = predicate.params[0]
        .kind
        .as_ref()
        .expect("Prop annotation should be preserved");
    assert_eq!(annotation.kind, Kind::Prop);
    assert_eq!(annotation.kind.to_string(), "Prop");
    assert!(annotation.span.end > annotation.span.start);
}

#[test]
fn parses_prop_in_kind_arrow_domain() {
    let module = parse("prop MapsProof<F : Prop -> *>;\n");

    let predicate = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::PropositionPredicate(predicate) => Some(predicate),
            _ => None,
        })
        .expect("proposition predicate should be present");

    let annotation = predicate.params[0]
        .kind
        .as_ref()
        .expect("Prop arrow annotation should be preserved");
    assert_eq!(annotation.kind, Kind::arrow(Kind::Prop, Kind::Type));
    assert_eq!(annotation.kind.to_string(), "Prop -> *");
}

#[test]
fn prop_keyword_does_not_swallow_longer_identifiers() {
    let module = parse("prop Holds<P : Property>;\n");

    let predicate = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::PropositionPredicate(predicate) => Some(predicate),
            _ => None,
        })
        .expect("proposition predicate should be present");

    assert!(
        predicate.params[0].kind.is_none(),
        "Property should remain an ordinary domain annotation, not a Prop kind prefix"
    );
}
