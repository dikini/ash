use ash_parser::surface::{Definition, Expr};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn first_law(source: &str) -> ash_parser::surface::LawDef {
    parse(source)
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Law(law) => Some(law),
            _ => None,
        })
        .expect("law should be parsed at module scope")
}

#[test]
fn parses_law_at_module_scope() {
    let module = parse(
        r#"
        law from_string_consistent(s: String, eq: Eq<PathBuf>)
          : eq.equiv(from_string(s), from_string(s))
        "#,
    );

    let law = module
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Law(law) => Some(law),
            _ => None,
        })
        .expect("law should be parsed at module scope");

    assert_eq!(law.name.as_ref(), "from_string_consistent");
    assert_eq!(law.params.len(), 2);
    assert_eq!(law.params[0].name.as_ref(), "s");
    assert_eq!(law.params[1].name.as_ref(), "eq");
    assert!(law.constraints.is_empty());
    assert!(!matches!(&law.proposition, Expr::Variable { name, .. } if name.as_ref().is_empty()));
}

#[test]
fn parses_law_with_where_constraints_at_module_scope() {
    let law = first_law(
        r#"
        law commutativity(a: Int, b: Int, eq: Eq<Int>) where eq(a, b): eq.equiv(add(a, b), add(b, a))
        "#,
    );

    assert_eq!(law.name.as_ref(), "commutativity");
    assert_eq!(law.params.len(), 3);
    assert_eq!(law.constraints.len(), 1);
    assert_eq!(law.constraints[0].predicate.name.as_ref(), "eq");
    assert_eq!(law.constraints[0].predicate.args.len(), 2);
}

#[test]
fn parses_multiple_laws_at_module_scope() {
    let module = parse(
        r#"
        law reflexivity(x: Int): eq(x, x)
        law symmetry(x: Int, y: Int): eq(x, y) == eq(y, x)
        "#,
    );

    let laws: Vec<_> = module
        .definitions
        .into_iter()
        .filter_map(|definition| match definition {
            Definition::Law(law) => Some(law),
            _ => None,
        })
        .collect();

    assert_eq!(laws.len(), 2);
    assert_eq!(laws[0].name.as_ref(), "reflexivity");
    assert_eq!(laws[1].name.as_ref(), "symmetry");
}

#[test]
fn parses_law_alongside_other_definitions() {
    let module = parse(
        r#"
        fn add(a: Int, b: Int) -> Int { a + b }
        law commutativity(a: Int, b: Int): add(a, b) == add(b, a)
        fn mul(a: Int, b: Int) -> Int { a * b }
        "#,
    );

    let defs = module.definitions;
    assert_eq!(defs.len(), 3);
    assert!(matches!(&defs[0], Definition::Function(_)));
    assert!(matches!(&defs[1], Definition::Law(_)));
    assert!(matches!(&defs[2], Definition::Function(_)));

    if let Definition::Law(law) = &defs[1] {
        assert_eq!(law.name.as_ref(), "commutativity");
        assert_eq!(law.params.len(), 2);
    }
}

#[test]
fn parses_law_in_inline_module() {
    let module = parse(
        r#"
        mod math {
            law associativity(a: Int, b: Int, c: Int): add(add(a, b), c) == add(a, add(b, c))
        }
        "#,
    );

    let inline_module = module
        .module_decls
        .into_iter()
        .find(|decl| decl.name.as_ref() == "math")
        .expect("math module should exist");

    let definitions = match inline_module.source {
        ash_parser::module::ModuleSource::Inline(defs) => defs,
        _ => panic!("expected inline module"),
    };

    assert_eq!(definitions.len(), 1);
    if let Definition::Law(law) = &definitions[0] {
        assert_eq!(law.name.as_ref(), "associativity");
        assert_eq!(law.params.len(), 3);
    } else {
        panic!("expected law definition inside inline module");
    }
}
