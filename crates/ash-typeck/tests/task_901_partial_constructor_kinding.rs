use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_core::kind::Kind;
use ash_core::type_ir::{PartialTypeArg, TypeConstructorExpr, TypeHoleAmbiguity};
use ash_parser::surface::Type as SurfaceType;
use ash_parser::token::Span;
use ash_typeck::{PartialConstructorElaborationError, TypeEnv};

fn name(name: &str) -> Box<str> {
    name.into()
}

fn result_type() -> TypeDef {
    TypeDef {
        name: "Result".into(),
        params: vec!["T".into(), "E".into()],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Ok".into(),
                fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                payload: VariantPayload::Tuple(vec![TypeExpr::Named("T".into())]),
            },
            VariantDef {
                name: "Err".into(),
                fields: vec![("error".into(), TypeExpr::Named("E".into()))],
                payload: VariantPayload::Tuple(vec![TypeExpr::Named("E".into())]),
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn nullary_type(name: &str) -> TypeDef {
    TypeDef {
        name: name.into(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn unary_type(name: &str) -> TypeDef {
    TypeDef {
        name: name.into(),
        params: vec!["T".into()],
        body: TypeBody::Struct(vec![]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn foo_type() -> TypeDef {
    TypeDef {
        name: "Foo".into(),
        params: vec!["A".into(), "B".into(), "C".into()],
        body: TypeBody::Struct(vec![]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn fixture_env() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_type(&result_type()).expect("register Result");
    env.register_type(&nullary_type("E")).expect("register E");
    env.register_type(&nullary_type("Extra"))
        .expect("register Extra");
    env.register_type(&unary_type("List"))
        .expect("register List");
    env.register_type(&foo_type()).expect("register Foo");
    env
}

fn result_with(args: Vec<SurfaceType>) -> SurfaceType {
    SurfaceType::Constructor {
        name: name("Result"),
        args,
    }
}

fn hole() -> SurfaceType {
    SurfaceType::Hole {
        span: Span::default(),
    }
}

#[test]
fn task_901_result_hole_error_elaborates_to_unary_partial_application() {
    let env = fixture_env();
    let surface = result_with(vec![hole(), SurfaceType::Name(name("E"))]);

    let elaborated = env
        .elaborate_do_target_constructor_expr(&surface)
        .expect("Result<_, E> is an MVP unary partial target");

    let TypeConstructorExpr::PartialApplication(app) = elaborated else {
        panic!("expected partial application, got {elaborated:?}");
    };
    assert_eq!(app.result_kind, Kind::n_ary(1));
    assert_eq!(app.args.len(), 2);
    assert!(matches!(app.args[0], PartialTypeArg::Hole(_)));
    assert!(matches!(app.args[1], PartialTypeArg::Applied(_)));
    assert_eq!(app.hole_metadata.len(), 1);
    assert_eq!(app.hole_metadata[0].expected_kind, Some(Kind::Type));
    assert_eq!(
        app.hole_metadata[0].ambiguity,
        TypeHoleAmbiguity::ExpectedValueSlot
    );
}

#[test]
fn task_901_saturated_result_is_rejected_when_partial_target_required() {
    let env = fixture_env();
    let surface = result_with(vec![
        SurfaceType::Name(name("Int")),
        SurfaceType::Name(name("E")),
    ]);

    let err = env
        .elaborate_do_target_constructor_expr(&surface)
        .expect_err("saturated Result<Int, E> has no partial target hole");

    assert!(matches!(
        err,
        PartialConstructorElaborationError::MissingHole { .. }
    ));
    assert!(err.to_string().contains("exactly one type hole"));
}

#[test]
fn task_901_multiple_holes_are_rejected() {
    let env = fixture_env();
    let result_two_holes = result_with(vec![hole(), hole()]);
    let foo_two_holes = SurfaceType::Constructor {
        name: name("Foo"),
        args: vec![hole(), hole(), SurfaceType::Name(name("E"))],
    };

    for surface in [result_two_holes, foo_two_holes] {
        let err = env
            .elaborate_do_target_constructor_expr(&surface)
            .expect_err("MVP rejects multiple partial-target holes");
        assert!(matches!(
            err,
            PartialConstructorElaborationError::MultipleHoles { count: 2, .. }
        ));
    }
}

#[test]
fn task_901_bare_higher_arity_constructor_suggests_explicit_hole() {
    let env = fixture_env();
    let surface = SurfaceType::Name(name("Result"));

    let err = env
        .elaborate_do_target_constructor_expr(&surface)
        .expect_err("bare Result is not implicit currying");

    assert!(matches!(
        err,
        PartialConstructorElaborationError::BareHigherArityConstructor { arity: 2, .. }
    ));
    assert!(err.to_string().contains("Result<_, E>"));
}

#[test]
fn task_901_wrong_arity_is_rejected_before_partial_elaboration() {
    let env = fixture_env();
    let too_many = result_with(vec![
        hole(),
        SurfaceType::Name(name("E")),
        SurfaceType::Name(name("Extra")),
    ]);
    let too_few = result_with(vec![hole()]);

    for surface in [too_many, too_few] {
        let err = env
            .elaborate_do_target_constructor_expr(&surface)
            .expect_err("wrong constructor arity should reject before partial elaboration");
        assert!(matches!(
            err,
            PartialConstructorElaborationError::WrongArity { .. }
        ));
    }
}

#[test]
fn task_901_nested_holes_are_unsupported_positions_not_inversion() {
    let env = fixture_env();
    let nested_hole = result_with(vec![
        SurfaceType::Constructor {
            name: name("List"),
            args: vec![hole()],
        },
        SurfaceType::Name(name("E")),
    ]);

    let err = env
        .elaborate_do_target_constructor_expr(&nested_hole)
        .expect_err("nested holes are not enabled for MVP partial targets");
    assert!(matches!(
        err,
        PartialConstructorElaborationError::UnsupportedHolePosition { .. }
    ));
}

#[test]
fn task_901_bare_hole_is_an_unsupported_position() {
    let env = fixture_env();

    let err = env
        .elaborate_do_target_constructor_expr(&hole())
        .expect_err("bare `_` has no constructor head");
    assert!(matches!(
        err,
        PartialConstructorElaborationError::UnsupportedHolePosition { .. }
    ));
}

#[test]
fn task_901_non_hole_associated_family_argument_is_not_an_inversion_boundary() {
    let env = fixture_env();
    let projection_arg = SurfaceType::AssociatedFamilyProjection {
        interface: name("Iterator"),
        args: vec![SurfaceType::Name(name("E"))],
        member: name("Item"),
        span: Span::default(),
    };
    let surface = result_with(vec![hole(), projection_arg]);

    let err = env
        .elaborate_do_target_constructor_expr(&surface)
        .expect_err("unknown projection metadata may still fail lowering");
    assert!(
        !matches!(
            err,
            PartialConstructorElaborationError::NoInversionBoundary { .. }
        ),
        "non-hole associated-family arguments must not be classified as hole inversion: {err:?}"
    );
}

#[test]
fn task_901_holes_under_associated_family_projection_do_not_invert_outputs() {
    let env = fixture_env();
    let surface = SurfaceType::AssociatedFamilyProjection {
        interface: name("Iterator"),
        args: vec![hole()],
        member: name("Item"),
        span: Span::default(),
    };

    let err = env
        .elaborate_do_target_constructor_expr(&surface)
        .expect_err("holes under associated-family outputs are out of scope");

    assert!(matches!(
        err,
        PartialConstructorElaborationError::NoInversionBoundary { .. }
    ));
    assert!(err.to_string().contains("inverting"));
}
