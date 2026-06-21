use ash_core::core_ash::{CoreEvalMode, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, check_core_type_well_formed, core_types_equivalent,
};

#[test]
fn strict_mode_rejects_latent_row() {
    let mode = CoreType::Mode {
        mode: CoreEvalMode::Strict,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: Some(CoreRow::default()),
    };

    let err = check_core_type_well_formed(&mode, &CoreTypeCheckEnv::default())
        .expect_err("strict mode cannot carry latent row");

    assert!(matches!(err, CoreTypeCheckError::InvalidModeType { .. }));
}

#[test]
fn lazy_mode_requires_latent_row() {
    let mode = CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: None,
    };

    let err = check_core_type_well_formed(&mode, &CoreTypeCheckEnv::default())
        .expect_err("lazy mode must carry latent row");

    assert!(matches!(err, CoreTypeCheckError::InvalidModeType { .. }));
}

#[test]
fn memo_mode_requires_latent_row() {
    let mode = CoreType::Mode {
        mode: CoreEvalMode::Memo,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: None,
    };

    let err = check_core_type_well_formed(&mode, &CoreTypeCheckEnv::default())
        .expect_err("memo mode must carry latent row");

    assert!(matches!(err, CoreTypeCheckError::InvalidModeType { .. }));
}

#[test]
fn lazy_mode_nested_row_carries_unknown_tail_error() {
    let mode = CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: Some(CoreRow::open(vec![], "missing_row")),
    };

    let err = check_core_type_well_formed(&mode, &CoreTypeCheckEnv::default())
        .expect_err("latent rows are checked recursively");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownRowVariable {
            name: "missing_row".into()
        }
    );
}

#[test]
fn mode_types_compare_invariantly_and_treat_mode_as_distinct() {
    let strict = CoreType::Mode {
        mode: CoreEvalMode::Strict,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: None,
    };
    let lazy = CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: Some(CoreRow::default()),
    };
    let memo = CoreType::Mode {
        mode: CoreEvalMode::Memo,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: Some(CoreRow::default()),
    };
    let env = CoreTypeCheckEnv::default();

    assert!(
        core_types_equivalent(&strict, &strict, &env).expect("same strict mode must match"),
        "exact mode wrapper should match"
    );
    assert!(
        !core_types_equivalent(&strict, &lazy, &env).expect("mode mismatch is not equivalent"),
        "strict must not match lazy"
    );
    assert!(
        !core_types_equivalent(&lazy, &memo, &env).expect("mode mismatch is not equivalent"),
        "lazy must not match memo"
    );
}

#[test]
fn mode_type_equivalence_uses_row_equivalence_for_nested_record_payloads() {
    let left = CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(CoreType::Base("Unit".into())),
        latent_row: Some(CoreRow::closed(vec![CoreRowItem::Channel {
            path: vec!["jobs".into()],
            mode: "send".into(),
            payload_type: Box::new(CoreType::Record(vec![
                ("a".into(), CoreType::Base("Int".into())),
                ("b".into(), CoreType::Base("String".into())),
            ])),
        }])),
    };
    let right = CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(CoreType::Base("Unit".into())),
        latent_row: Some(CoreRow::closed(vec![CoreRowItem::Channel {
            path: vec!["jobs".into()],
            mode: "send".into(),
            payload_type: Box::new(CoreType::Record(vec![
                ("b".into(), CoreType::Base("String".into())),
                ("a".into(), CoreType::Base("Int".into())),
            ])),
        }])),
    };
    let env = CoreTypeCheckEnv::default();

    assert!(core_types_equivalent(&left, &right, &env).expect(
        "mode latent rows should compare channel payload record fields using type equivalence"
    ));
}

#[test]
fn mode_type_checks_inner_refinement_predicate() {
    let mode = CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(CoreType::Refinement {
            base: Box::new(CoreType::Base("Int".into())),
            predicate: "positive-result".into(),
        }),
        latent_row: Some(CoreRow::default()),
    };

    let err = check_core_type_well_formed(&mode, &CoreTypeCheckEnv::default())
        .expect_err("unknown refinement predicate in mode wrappers should reject");

    assert_eq!(
        err,
        CoreTypeCheckError::UnknownRefinementPredicate {
            predicate: "positive-result".into()
        }
    );
}
