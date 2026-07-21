use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_core::type_ir::{
    TcirOperationKind, TcirStatementKind, TypeConstructorExpr, TypeConstructorHeadId,
};
use ash_parser::surface::{
    ConstructorPayload, Definition, DoStmt, DoTarget, Expr, ImplDef, InterfaceDef, Literal,
    Type as SurfaceType,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::check_expr::elaborate_typed_do_block;
use ash_typeck::types::Type;

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module file should parse: {source}\nerrors: {errors:?}"))
}

fn interface_named(module: &ash_parser::surface::ModuleFile, name: &str) -> InterfaceDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == name => {
                Some(interface.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("interface {name} should be present"))
}

fn impl_named(module: &ash_parser::surface::ModuleFile, name: &str) -> ImplDef {
    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Impl(implementation) if implementation.interface.as_ref() == name => {
                Some(implementation.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("impl {name} should be present"))
}

fn pair_type_def() -> TypeDef {
    TypeDef {
        name: "Pair".into(),
        params: Vec::new(),
        body: TypeBody::Enum(vec![VariantDef {
            name: "Pair".into(),
            fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
            payload: VariantPayload::Record(vec![("value".into(), TypeExpr::Named("Int".into()))]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env_with_monad_option_methods() -> TypeEnv {
    let module = parse(
        r#"
        interface Monad<M : * -> *> {
            unit(Int) -> M<Int>
            bind(M<Int>, (Int) -> M<Int>) -> M<Int>
        }

        impl Monad<Option> {
            unit(value) = Some { value: value }
            bind(value, _f) = value
        }
        "#,
    );
    let interface = interface_named(&module, "Monad");
    let implementation = impl_named(&module, "Monad");

    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&pair_type_def())
        .expect("Pair fixture type should register");
    env.register_interface(&interface)
        .expect("Monad interface should register");
    env.register_impl(&implementation)
        .expect("Monad<Option> implementation should register");
    env
}

fn do_option_return() -> Expr {
    Expr::DoBlock {
        target: DoTarget {
            name: "Option".into(),
            args: Vec::<SurfaceType>::new(),
            span: Span {
                start: 3,
                end: 9,
                line: 1,
                column: 4,
            },
        },
        stmts: vec![DoStmt::Return {
            value: Box::new(Expr::Literal(Literal::Int(1))),
            span: Span {
                start: 12,
                end: 17,
                line: 2,
                column: 5,
            },
        }],
        span: Span {
            start: 0,
            end: 20,
            line: 1,
            column: 1,
        },
    }
}

#[test]
fn typeck_attaches_tcir_without_collapsing_user_constructor_to_runtime_bridge() {
    let env = env_with_monad_option_methods();
    let elaborated = elaborate_typed_do_block(&env, &do_option_return())
        .expect("do:Option should elaborate through selected Monad<Option> evidence");
    let tcir = elaborated
        .tcir
        .as_ref()
        .expect("typed do elaboration should attach TCIR");

    assert_eq!(
        tcir.source_anchor.span.map(|span| (span.start, span.end)),
        Some((0, 20))
    );
    assert_eq!(tcir.target.display, "Option");
    assert_eq!(
        tcir.target
            .source_anchor
            .span
            .map(|span| (span.start, span.end)),
        Some((3, 9))
    );
    assert!(matches!(
        tcir.target.constructor,
        TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::Nominal { ref visible_name, .. })
            if visible_name == "Option"
    ));
    assert!(!matches!(
        tcir.target.constructor,
        TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::Nominal { ref visible_name, .. })
            if matches!(visible_name.as_str(), "Act" | "Proc")
    ));
    assert_eq!(tcir.evidence.interface, "Monad");
    assert_eq!(tcir.evidence.evidence_key, "Monad<Option>");
    assert!(matches!(
        tcir.evidence.return_op.kind,
        TcirOperationKind::EvidenceMethod { ref evidence_key, ref method, .. }
            if evidence_key == "Monad<Option>" && method == "unit"
    ));
    assert!(!matches!(
        tcir.evidence.return_op.kind,
        TcirOperationKind::HiddenCompilerPrelude { .. }
    ));
    assert_eq!(tcir.statements.len(), 1);
    assert_eq!(
        tcir.statements[0]
            .source_anchor
            .span
            .map(|span| (span.start, span.end)),
        Some((12, 17))
    );
    assert!(matches!(
        tcir.statements[0].kind,
        TcirStatementKind::Return { .. }
    ));
    assert!(tcir.failure_boundaries.len() == 1);
    assert_eq!(tcir.failure_boundaries[0].entity, None);
}

#[test]
fn typeck_tcir_attachment_does_not_reject_non_nominal_result_types() {
    let env = env_with_monad_option_methods();
    let mut expr = do_option_return();
    let Expr::DoBlock { stmts, .. } = &mut expr else {
        panic!("test helper must produce do block");
    };
    let DoStmt::Return { value, .. } = &mut stmts[0] else {
        panic!("test helper must produce return statement");
    };
    **value = Expr::Constructor {
        name: "Pair".into(),
        fields: vec![("value".into(), Expr::Literal(Literal::Int(1)))],
        payload: ConstructorPayload::Record(vec![("value".into(), Expr::Literal(Literal::Int(1)))]),
        span: Span {
            start: 12,
            end: 31,
            line: 2,
            column: 5,
        },
    };

    let elaborated = elaborate_typed_do_block(&env, &expr)
        .expect("TCIR attachment must not reject valid record result do blocks");
    let tcir = elaborated.tcir.expect("typed do elaboration attaches TCIR");

    assert!(matches!(elaborated.ty, Type::Constructor { .. }));
    assert!(matches!(
        tcir.result_type,
        ash_core::CanonicalTypeExpr::NominalApp { .. }
    ));
    assert_eq!(tcir.failure_boundaries[0].entity, None);
}
