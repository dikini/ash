use ash_core::core_ash::{CoreMultiplicity, CoreRow, CoreType};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, check_core_type_well_formed, core_types_equivalent,
};

#[test]
fn known_base_and_named_types_are_well_formed() {
    let mut env = CoreTypeCheckEnv::default();
    env.types_mut().insert_name("UserId");

    check_core_type_well_formed(&CoreType::Base("Int".into()), &env)
        .expect("known base type is well formed");
    check_core_type_well_formed(&CoreType::Named("UserId".into()), &env)
        .expect("known named type is well formed");
}

#[test]
fn unknown_named_type_fails_with_structured_error() {
    let err = check_core_type_well_formed(&CoreType::Named("Missing".into()), &Default::default())
        .expect_err("unknown named type is rejected");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownType {
            name: "Missing".into()
        }
    );
}

#[test]
fn type_application_arity_mismatch_fails() {
    let mut env = CoreTypeCheckEnv::default();
    env.types_mut().insert_constructor("Result", 2);

    let err = check_core_type_well_formed(
        &CoreType::App {
            name: "Result".into(),
            args: vec![CoreType::Base("Int".into())],
        },
        &env,
    )
    .expect_err("wrong type constructor arity is rejected");

    assert_eq!(
        err,
        CoreTypeCheckError::TypeApplicationArityMismatch {
            name: "Result".into(),
            expected: 2,
            actual: 1
        }
    );
}

#[test]
fn record_type_equivalence_is_field_name_based() {
    let env = CoreTypeCheckEnv::default();
    let first = CoreType::Record(vec![
        ("name".into(), CoreType::Base("String".into())),
        ("age".into(), CoreType::Base("Int".into())),
    ]);
    let second = CoreType::Record(vec![
        ("age".into(), CoreType::Base("Int".into())),
        ("name".into(), CoreType::Base("String".into())),
    ]);

    check_core_type_well_formed(&first, &env).expect("first record is well formed");
    check_core_type_well_formed(&second, &env).expect("second record is well formed");
    assert!(core_types_equivalent(&first, &second, &env).expect("record comparison succeeds"));
}

#[test]
fn duplicate_record_field_names_are_rejected_by_well_formedness() {
    let env = CoreTypeCheckEnv::default();
    let duplicate = CoreType::Record(vec![
        ("a".into(), CoreType::Base("Int".into())),
        ("a".into(), CoreType::Base("Int".into())),
    ]);

    let err = check_core_type_well_formed(&duplicate, &env)
        .expect_err("duplicate record field names are ambiguous in Core type metadata");

    assert_eq!(
        err,
        CoreTypeCheckError::DuplicateRecordField { field: "a".into() }
    );
}

#[test]
fn duplicate_record_field_names_are_not_equivalent_to_ambiguous_records() {
    let env = CoreTypeCheckEnv::default();
    let expected = CoreType::Record(vec![
        ("a".into(), CoreType::Base("Int".into())),
        ("b".into(), CoreType::Base("Int".into())),
    ]);
    let actual = CoreType::Record(vec![
        ("a".into(), CoreType::Base("Int".into())),
        ("a".into(), CoreType::Base("String".into())),
    ]);

    assert_eq!(
        core_types_equivalent(&expected, &actual, &env)
            .expect_err("duplicate record field names are rejected before equality"),
        CoreTypeCheckError::DuplicateRecordField { field: "a".into() }
    );
}

#[test]
fn refinement_checks_base_type_recursively() {
    let mut env = CoreTypeCheckEnv::default();
    env.discharges_mut()
        .insert_refinement_predicate("positive-result");
    let refinement = CoreType::Refinement {
        base: Box::new(CoreType::Named("Missing".into())),
        predicate: "positive-result".into(),
    };

    let err = check_core_type_well_formed(&refinement, &env)
        .expect_err("refinement base type must be well formed");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownType {
            name: "Missing".into()
        }
    );
}

#[test]
fn textual_refinement_requires_tracked_predicate_placeholder() {
    let refinement = CoreType::Refinement {
        base: Box::new(CoreType::Base("Int".into())),
        predicate: "positive-result".into(),
    };

    let err = check_core_type_well_formed(&refinement, &Default::default())
        .expect_err("textual predicate alone is not enough");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownRefinementPredicate {
            predicate: "positive-result".into()
        }
    );
}

#[test]
fn function_and_continuation_rows_require_known_row_tails() {
    let function_type = CoreType::Function {
        params: vec![CoreType::Base("Int".into())],
        result: Box::new(CoreType::Base("Int".into())),
        row: CoreRow::open(vec![], "r"),
    };

    let err = check_core_type_well_formed(&function_type, &Default::default())
        .expect_err("unknown row tail is rejected");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownRowVariable { name: "r".into() }
    );

    let mut env = CoreTypeCheckEnv::default();
    env.rows_mut().insert("r", CoreRow::default());
    let continuation_type = CoreType::Cont {
        input: Box::new(CoreType::Base("Int".into())),
        answer: Box::new(CoreType::Base("Unit".into())),
        row: CoreRow::open(vec![], "r"),
        multiplicity: CoreMultiplicity::Affine,
    };

    check_core_type_well_formed(&continuation_type, &env)
        .expect("known row tail makes continuation type well formed");
}
