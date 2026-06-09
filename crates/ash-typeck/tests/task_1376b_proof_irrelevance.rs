use ash_core::type_ir::{TypeEqualityProposition, TypeProposition, TypePropositionTerm};
use ash_parser::surface::{Expr, Literal, ProofBody, ProofDef};
use ash_parser::token::Span;

fn proof(name: &str, body: ProofBody) -> ProofDef {
    ProofDef {
        name: name.into(),
        params: vec![],
        constraints: vec![],
        body,
        span: Span::default(),
    }
}

fn bool_equality_prop() -> TypeProposition {
    TypeProposition::Equality(TypeEqualityProposition {
        lhs: TypePropositionTerm::Canonical(ash_core::CanonicalTypeExpr::Primitive("Bool".into())),
        rhs: TypePropositionTerm::Canonical(ash_core::CanonicalTypeExpr::Primitive("Bool".into())),
    })
}

fn int_equality_prop() -> TypeProposition {
    TypeProposition::Equality(TypeEqualityProposition {
        lhs: TypePropositionTerm::Canonical(ash_core::CanonicalTypeExpr::Primitive("Int".into())),
        rhs: TypePropositionTerm::Canonical(ash_core::CanonicalTypeExpr::Primitive("Int".into())),
    })
}

#[test]
fn proof_irrelevance_makes_distinct_proof_bodies_equal_for_same_proposition() {
    let env = ash_typeck::TypeEnv::with_builtin_types();
    let proposition = bool_equality_prop();
    let by_definition = proof("bool_refl_by_definition", ProofBody::ByDefinition);
    let by_expr = proof(
        "bool_refl_by_expr",
        ProofBody::Expr(Expr::Literal(Literal::Bool(true))),
    );

    let left = env
        .erase_proof_for_proposition(&proposition, &by_definition)
        .expect("by-definition proof should erase");
    let right = env
        .erase_proof_for_proposition(&proposition, &by_expr)
        .expect("expression proof should erase");

    assert_eq!(
        left, right,
        "proof erasure must discard proof identity and body"
    );
    assert!(
        env.proofs_definitionally_equal_for_proposition(&proposition, &by_definition, &by_expr)
            .expect("total proof erasure should compare"),
        "proofs of the same proposition are definitionally equal"
    );
}

#[test]
fn proof_erasure_preserves_the_proved_proposition_boundary() {
    let env = ash_typeck::TypeEnv::with_builtin_types();
    let same_body = ProofBody::ByDefinition;
    let bool_erasure = env
        .erase_proof_for_proposition(
            &bool_equality_prop(),
            &proof("bool_refl", same_body.clone()),
        )
        .expect("Bool proof should erase");
    let int_erasure = env
        .erase_proof_for_proposition(&int_equality_prop(), &proof("int_refl", same_body))
        .expect("Int proof should erase");

    assert_ne!(
        bool_erasure, int_erasure,
        "erasure must retain the proposition so different propositions do not collapse"
    );
}

#[test]
fn inconclusive_proof_totality_is_not_erased() {
    let env = ash_typeck::TypeEnv::with_builtin_types();
    let proof = proof(
        "fuel_exhausted",
        ProofBody::Expr(Expr::Literal(Literal::Bool(true))),
    );

    let err = env
        .erase_proof_for_proposition_with_fuel(&bool_equality_prop(), &proof, 0)
        .expect_err("inconclusive proof totality should not erase");

    let message = err.to_string();
    assert!(
        message.contains("could not be erased") && message.contains("inconclusive"),
        "expected proof-erasure totality diagnostic, got: {message}"
    );
}
