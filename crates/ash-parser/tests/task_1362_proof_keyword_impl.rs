use ash_parser::module::ModuleSource;
use ash_parser::surface::{Definition, ProofBody};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn find_impl(definitions: &[Definition]) -> Option<ash_parser::surface::ImplDef> {
    definitions.iter().find_map(|definition| match definition {
        Definition::Impl(impl_def) => Some(impl_def.clone()),
        _ => None,
    })
}

fn first_impl(source: &str) -> ash_parser::surface::ImplDef {
    let module = parse(source);
    find_impl(&module.definitions)
        .or_else(|| {
            module
                .module_decls
                .iter()
                .find_map(|decl| match &decl.source {
                    ModuleSource::Inline(definitions) => find_impl(definitions),
                    ModuleSource::File => None,
                })
        })
        .expect("impl should be parsed")
}

#[test]
fn parses_proof_by_definition_inside_impl() {
    let impl_def = first_impl(
        r#"
        mod m {
            impl Semigroup<Int> {
                append(a, b) = a + b
                proof associativity(a: Int, b: Int, c: Int) where Semigroup(Int) {
                    by_definition
                }
            }
        }
        "#,
    );

    assert_eq!(impl_def.interface.as_ref(), "Semigroup");
    assert_eq!(impl_def.proofs.len(), 1);

    let proof = &impl_def.proofs[0];
    assert_eq!(proof.name.as_ref(), "associativity");
    assert_eq!(proof.params.len(), 3);
    assert_eq!(proof.params[0].name.as_ref(), "a");
    assert_eq!(proof.params[1].name.as_ref(), "b");
    assert_eq!(proof.params[2].name.as_ref(), "c");
    assert_eq!(proof.constraints.len(), 1);
    assert_eq!(proof.constraints[0].predicate.name.as_ref(), "Semigroup");
    assert!(matches!(proof.body, ProofBody::ByDefinition));
}

#[test]
fn parses_proof_by_test_inside_impl() {
    let impl_def = first_impl(
        r#"
        mod m {
            impl Monoid<String> {
                empty() = ""
                append(a, b) = concat(a, b)
                proof left_identity(x: String) where monoid_string(x) {
                    by test "monoid_left_identity"
                }
            }
        }
        "#,
    );

    assert_eq!(impl_def.proofs.len(), 1);
    let proof = &impl_def.proofs[0];
    assert_eq!(proof.name.as_ref(), "left_identity");
    assert_eq!(proof.params.len(), 1);

    match &proof.body {
        ProofBody::ByTest { test_name } => {
            assert_eq!(test_name, "monoid_left_identity");
        }
        _ => panic!("expected ByTest proof body"),
    }
}

#[test]
fn parses_impl_with_methods_and_proofs() {
    let impl_def = first_impl(
        r#"
        mod m {
            impl Eq<Int> {
                equiv(a, b) = a == b
                proof reflexivity(x: Int) {
                    by_definition
                }
                proof symmetry(x: Int, y: Int) where Eq(Int) {
                    by_definition
                }
            }
        }
        "#,
    );

    assert_eq!(impl_def.methods.len(), 1);
    assert_eq!(impl_def.proofs.len(), 2);
    assert_eq!(impl_def.proofs[0].name.as_ref(), "reflexivity");
    assert_eq!(impl_def.proofs[1].name.as_ref(), "symmetry");
}

#[test]
fn parses_expression_body_inside_impl_proof() {
    let source = r#"
        impl Eq<String> {
            equiv(a, b) = string_eq(a, b)
            proof reflexivity(x: String) {
                equiv(x, x)
            }
        }
        "#;

    let impl_def = first_impl(source);
    assert_eq!(impl_def.proofs.len(), 1);
    assert!(matches!(impl_def.proofs[0].body, ProofBody::Expr(_)));
}

#[test]
fn parses_proof_without_constraints() {
    let impl_def = first_impl(
        r#"
        mod m {
            impl Functor<List> {
                map(fa, f) = list_map(fa, f)
                proof identity(fa: List<A>) {
                    by_definition
                }
            }
        }
        "#,
    );

    assert_eq!(impl_def.proofs.len(), 1);
    let proof = &impl_def.proofs[0];
    assert!(proof.constraints.is_empty());
    assert!(matches!(proof.body, ProofBody::ByDefinition));
}
