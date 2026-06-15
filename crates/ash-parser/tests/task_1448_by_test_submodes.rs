use ash_parser::surface::{Definition, ProofBody};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn first_proof_body(source: &str) -> ProofBody {
    let module = parse(source);
    let Definition::Proof(proof) = &module.definitions[0] else {
        panic!("expected module-scoped proof definition");
    };
    proof.body.clone()
}

#[test]
fn parses_explicit_authored_by_test_submode() {
    match first_proof_body(
        r#"
        proof identity(x: Int) {
            by test authored "identity_examples"
        }
        "#,
    ) {
        ProofBody::ByTest { test_name } => assert_eq!(test_name, "identity_examples"),
        other => panic!("expected authored by-test proof body, got {other:?}"),
    }
}

#[test]
fn parses_property_by_test_submode() {
    assert!(matches!(
        first_proof_body(
            r#"
            proof identity(x: Int) {
                by test property
            }
            "#,
        ),
        ProofBody::ByTestProperty
    ));
}

#[test]
fn parses_small_world_by_test_submode() {
    assert!(matches!(
        first_proof_body(
            r#"
            proof identity(x: Int) {
                by test small_world
            }
            "#,
        ),
        ProofBody::ByTestSmallWorld
    ));
}
