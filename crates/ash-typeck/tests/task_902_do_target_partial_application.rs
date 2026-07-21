use ash_core::ast::{TypeBody, TypeDef, Visibility};
use ash_parser::surface::{DoTarget, Name, Type as SurfaceType};
use ash_parser::token::Span;
use ash_typeck::{TypeEnv, resolve_do_target_for_test};

fn name(name: &str) -> Box<str> {
    name.into()
}

fn target(name: &str, args: Vec<SurfaceType>) -> DoTarget {
    DoTarget {
        name: Name::from(name),
        args,
        span: Span::default(),
    }
}

fn hole() -> SurfaceType {
    SurfaceType::Hole {
        span: Span::default(),
    }
}

fn proper(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn type_def(name: &str, params: &[&str]) -> TypeDef {
    TypeDef {
        name: name.into(),
        params: params.iter().copied().map(Into::into).collect(),
        body: TypeBody::Struct(vec![]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn fixture_env() -> TypeEnv {
    let mut env = TypeEnv::new();
    for def in [
        type_def("Result", &["T", "E"]),
        type_def("E", &[]),
        type_def("List", &["T"]),
        type_def("Act", &["T"]),
        type_def("Proc", &["T"]),
    ] {
        env.register_type(&def).expect("register fixture type");
    }
    env
}

fn error_text(err: impl std::fmt::Display) -> String {
    err.to_string()
}

#[test]
fn task_902_result_hole_error_reaches_missing_monad_evidence() {
    let env = fixture_env();
    let err = resolve_do_target_for_test(&env, &target("Result", vec![hole(), proper("E")]))
        .expect_err("Result<_, E> has shape but no SPEC-067 Monad evidence");
    let message = error_text(err);

    assert!(message.contains("missing Monad evidence"), "{message}");
    assert!(message.contains("Result<_, E>"), "{message}");
    assert!(!message.contains("explicit type arguments"), "{message}");
    assert!(!message.contains("wrong target shape"), "{message}");
}

#[test]
fn task_902_bare_result_reports_wrong_shape_with_hole_hint() {
    let env = fixture_env();
    let err = resolve_do_target_for_test(&env, &target("Result", vec![]))
        .expect_err("bare Result is not implicitly curried");
    let message = error_text(err);

    assert!(message.contains("wrong target shape"), "{message}");
    assert!(message.contains("Result<_, E>"), "{message}");
    assert!(!message.contains("missing Monad evidence"), "{message}");
}

#[test]
fn task_902_result_with_two_holes_reports_multiple_holes() {
    let env = fixture_env();
    let err = resolve_do_target_for_test(&env, &target("Result", vec![hole(), hole()]))
        .expect_err("multiple holes remain outside the MVP");
    let message = error_text(err);

    assert!(message.contains("multiple type holes"), "{message}");
    assert!(message.contains("2"), "{message}");
    assert!(!message.contains("missing Monad evidence"), "{message}");
}

#[test]
fn task_902_nested_result_hole_reports_unsupported_shape_not_missing_evidence() {
    let env = fixture_env();
    let nested_list_hole = SurfaceType::Constructor {
        name: name("List"),
        args: vec![hole()],
    };
    let err =
        resolve_do_target_for_test(&env, &target("Result", vec![nested_list_hole, proper("E")]))
            .expect_err("nested holes are unsupported in MVP do targets");
    let message = error_text(err);

    assert!(
        message.contains("unsupported") || message.contains("non-inverting"),
        "{message}"
    );
    assert!(
        message.contains("nested holes") || message.contains("inverting"),
        "{message}"
    );
    assert!(!message.contains("missing Monad evidence"), "{message}");
}

#[test]
fn task_902_associated_family_hole_reports_no_inversion_not_missing_evidence() {
    let env = fixture_env();
    let projection_with_hole = SurfaceType::AssociatedFamilyProjection {
        interface: name("Iterator"),
        args: vec![hole()],
        member: name("Item"),
        span: Span::default(),
    };
    let err = resolve_do_target_for_test(
        &env,
        &target("Result", vec![projection_with_hole, proper("E")]),
    )
    .expect_err("holes under associated-family outputs are not invertible do targets");
    let message = error_text(err);

    assert!(message.contains("non-inverting"), "{message}");
    assert!(message.contains("inverting"), "{message}");
    assert!(!message.contains("missing Monad evidence"), "{message}");
}

#[test]
fn task_902_tower_carriers_require_explicit_monad_evidence() {
    let env = fixture_env();

    for target_name in ["Act", "Proc"] {
        let err = resolve_do_target_for_test(&env, &target(target_name, vec![]))
            .expect_err("{target_name} must not resolve without Monad evidence");
        let message = error_text(err);
        assert!(message.contains("missing Monad evidence"), "{message}");
        assert!(
            message.contains(&format!("Monad<{target_name}>")),
            "{message}"
        );
    }
}
