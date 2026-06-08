use ash_parser::module::ModuleSource;
use ash_parser::surface::{Definition, ProofBody};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

#[test]
fn parses_proof_at_module_scope() {
    let module = parse(
        r#"
        proof monoid_left_identity(x: String) where monoid_string(x) {
            by_definition
        }
        "#,
    );

    assert_eq!(module.definitions.len(), 1);
    let Definition::Proof(proof) = &module.definitions[0] else {
        panic!("expected module-scoped proof definition");
    };

    assert_eq!(proof.name.as_ref(), "monoid_left_identity");
    assert_eq!(proof.params.len(), 1);
    assert_eq!(proof.params[0].name.as_ref(), "x");
    assert_eq!(proof.constraints.len(), 1);
    assert_eq!(
        proof.constraints[0].predicate.name.as_ref(),
        "monoid_string"
    );
    assert!(matches!(proof.body, ProofBody::ByDefinition));
}

#[test]
fn parses_proof_in_inline_module_scope() {
    let module = parse(
        r#"
        mod laws {
            proof list_functor_identity(xs: List<Int>) {
                by test "list_functor_identity"
            }
        }
        "#,
    );

    let inline = module
        .module_decls
        .iter()
        .find_map(|decl| match &decl.source {
            ModuleSource::Inline(definitions) => Some(definitions),
            ModuleSource::File => None,
        })
        .expect("inline module definitions should exist");

    assert_eq!(inline.len(), 1);
    let Definition::Proof(proof) = &inline[0] else {
        panic!("expected proof in inline module");
    };

    assert_eq!(proof.name.as_ref(), "list_functor_identity");
    match &proof.body {
        ProofBody::ByTest { test_name } => assert_eq!(test_name, "list_functor_identity"),
        _ => panic!("expected by test proof body"),
    }
}

#[test]
fn parses_proofs_alongside_laws_functions_and_types() {
    let module = parse(
        r#"
        type Marker = Done | Pending;

        fn id(x: Int) -> Int {
            x
        }

        law id_reflexive(x: Int): eq(id(x), x)

        proof id_reflexive(x: Int) {
            eq(id(x), x)
        }
        "#,
    );

    assert!(
        module
            .definitions
            .iter()
            .any(|definition| matches!(definition, Definition::Type(_)))
    );
    assert!(
        module
            .definitions
            .iter()
            .any(|definition| matches!(definition, Definition::Function(_)))
    );
    assert!(
        module
            .definitions
            .iter()
            .any(|definition| matches!(definition, Definition::Law(_)))
    );
    assert!(
        module
            .definitions
            .iter()
            .any(|definition| matches!(definition, Definition::Proof(_)))
    );
}
