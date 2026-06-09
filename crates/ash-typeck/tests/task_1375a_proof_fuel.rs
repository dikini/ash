use ash_parser::surface::{Definition, ProofDef};

fn first_module_proof(source: &str) -> ProofDef {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Proof(proof) => Some(proof),
            _ => None,
        })
        .expect("source should contain a module-scoped proof")
}

#[test]
fn proof_expr_with_zero_fuel_returns_untested_not_error() {
    let proof = first_module_proof(
        r#"
        law reflexive(x: Int): x == x
        proof reflexive(x: Int) {
            x == x
        }
        "#,
    );

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality_with_fuel(&proof, 0)
        .expect("fuel exhaustion should be reported as an untested proof result, not an error");

    assert_eq!(
        result.status,
        ash_typeck::ProofTotalityStatus::Untested(
            ash_typeck::ProofTotalityUntestedReason::FuelExhausted
        )
    );
    assert_eq!(result.fuel_limit, 0);
    assert_eq!(result.fuel_remaining, 0);
}

#[test]
fn default_proof_totality_fuel_is_1000_and_counts_expr_steps() {
    let proof = first_module_proof(
        r#"
        law reflexive(x: Int): x == x
        proof reflexive(x: Int) {
            x == x
        }
        "#,
    );

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("default proof totality check should not reject a finite proof expression");

    assert_eq!(ash_typeck::DEFAULT_PROOF_FUEL, 1000);
    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
    assert_eq!(result.fuel_limit, ash_typeck::DEFAULT_PROOF_FUEL);
    assert!(
        result.fuel_remaining < result.fuel_limit,
        "proof expression traversal should consume at least one normalization step: {result:?}"
    );
}
