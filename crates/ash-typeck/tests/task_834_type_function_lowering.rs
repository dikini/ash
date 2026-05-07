//! TASK-834: type-function lowering and module-local registration substrate.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
use ash_core::type_ir::{TypeFunctionPattern, TypeFunctionResultExpr};
use ash_parser::surface::Definition;
use ash_typeck::TypeEnv;

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(834)),
        ModuleId(id),
        vec!["task834".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-834-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-834-test".into(),
        },
        None,
        label,
    )
}

fn typelist_summary(module: &ModuleIdentity) -> ModuleSemanticSummary {
    let domain = SealedDomainId::new(module.clone(), "TypeList");
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Nil"),
        "Nil",
        vec![],
        anchor("Nil"),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), "Cons"),
        "Cons",
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor("Cons"),
    );
    let mut summary = ModuleSemanticSummary::new(module.clone()).with_exported_sealed_domain(
        SealedDomainSummary::new(
            domain,
            "TypeList",
            CoreVisibility::Public,
            anchor("TypeList"),
        )
        .with_constructor(nil)
        .with_constructor(cons),
    );
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    summary
}

fn type_fns(source: &str) -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(source).expect("source parses");
    parsed
        .definitions
        .into_iter()
        .filter_map(|def| match def {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect()
}

fn register_domain(env: &mut TypeEnv, module: &ModuleIdentity) {
    env.register_module_semantic_summary(&typelist_summary(module))
        .expect("domain registers");
}

#[test]
fn lowers_self_reference_to_current_provisional_head_and_preserves_equation_order() {
    let module = module_identity(1);
    let mut env = TypeEnv::new();
    register_domain(&mut env, &module);
    let defs = type_fns(
        r#"
        type fn Append(xs: TypeList, ys: TypeList) -> TypeList decreases xs {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
        "#,
    );

    env.register_local_type_functions(&module, &defs)
        .expect("Append should lower and publish");

    let append = env
        .lookup_local_type_function("Append")
        .expect("Append published");
    assert_eq!(append.equations.len(), 2);
    assert_eq!(append.equations[0].ordinal, 0);
    assert_eq!(append.equations[1].ordinal, 1);
    assert_eq!(append.equations[1].patterns.len(), 2);
    match &append.equations[1].result {
        TypeFunctionResultExpr::DomainConstructorApp { args, .. } => match &args[1] {
            TypeFunctionResultExpr::ComputationHeadApp {
                head, args, kind, ..
            } => {
                assert_eq!(head, &append.head);
                assert_eq!(args.len(), 2);
                assert_eq!(*kind, Kind::Type);
            }
            other => panic!("expected recursive computation-head app, got {other:?}"),
        },
        other => panic!("expected Cons domain-constructor RHS, got {other:?}"),
    }
}

#[test]
fn lowers_earlier_validated_dependency_to_computation_head_app() {
    let module = module_identity(2);
    let mut env = TypeEnv::new();
    register_domain(&mut env, &module);
    let defs = type_fns(
        r#"
        type fn IdList(xs: TypeList) -> TypeList {
            case IdList<xs> = xs;
        }
        type fn UseId(xs: TypeList) -> TypeList {
            case UseId<xs> = IdList<xs>;
        }
        "#,
    );

    env.register_local_type_functions(&module, &defs)
        .expect("source-ordered dependency should register");

    let id = env.lookup_local_type_function("IdList").unwrap();
    let use_id = env.lookup_local_type_function("UseId").unwrap();
    match &use_id.equations[0].result {
        TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
            assert_eq!(head, &id.head);
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected dependency computation-head app, got {other:?}"),
    }
}

#[test]
fn rejects_duplicate_type_function_names_without_publishing_duplicate() {
    let module = module_identity(3);
    let mut env = TypeEnv::new();
    register_domain(&mut env, &module);
    let defs = type_fns(
        r#"
        type fn Dup(xs: TypeList) -> TypeList { case Dup<xs> = xs; }
        type fn Dup(xs: TypeList) -> TypeList { case Dup<xs> = xs; }
        "#,
    );

    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("duplicate names are rejected");
    assert!(format!("{err}").contains("duplicate type function 'Dup'"));
    assert!(env.lookup_local_type_function("Dup").is_none());
}

#[test]
fn invalid_current_definition_is_not_published_to_later_registrations() {
    let module = module_identity(4);
    let mut env = TypeEnv::new();
    register_domain(&mut env, &module);
    let bad = type_fns(
        r#"
        type fn Bad(xs: TypeList) -> TypeList {
            case Bad<xs> = UnknownHead<xs>;
        }
        "#,
    );

    let err = env
        .register_local_type_functions(&module, &bad)
        .expect_err("bad RHS head rejects definition");
    assert!(format!("{err}").contains("unresolved type function or type head 'UnknownHead'"));
    assert!(env.lookup_local_type_function("Bad").is_none());

    let later = type_fns(
        r#"
        type fn Later(xs: TypeList) -> TypeList {
            case Later<xs> = Bad<xs>;
        }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &later)
        .expect_err("unpublished invalid head is not visible");
    assert!(format!("{err}").contains("unresolved type function or type head 'Bad'"));
    assert!(env.lookup_local_type_function("Later").is_none());
}

#[test]
fn rejects_later_same_module_forward_reference_before_publication() {
    let module = module_identity(5);
    let mut env = TypeEnv::new();
    register_domain(&mut env, &module);
    let defs = type_fns(
        r#"
        type fn UseLater(xs: TypeList) -> TypeList {
            case UseLater<xs> = Later<xs>;
        }
        type fn Later(xs: TypeList) -> TypeList {
            case Later<xs> = xs;
        }
        "#,
    );

    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("forward reference is SPEC-E unsupported");
    assert!(format!("{err}").contains("forward reference to later type function 'Later'"));
    assert!(env.lookup_local_type_function("UseLater").is_none());
    assert!(env.lookup_local_type_function("Later").is_none());
}

#[test]
fn preserves_pattern_variable_metadata_and_marker_constructor_rhs_carriers() {
    let module = module_identity(6);
    let mut env = TypeEnv::new();
    register_domain(&mut env, &module);
    let defs = type_fns(
        r#"
        type fn TailOrNil(xs: TypeList) -> TypeList {
            case TailOrNil<Nil> = Nil;
            case TailOrNil<Cons<h, t>> = Cons<h, t>;
        }
        "#,
    );

    env.register_local_type_functions(&module, &defs)
        .expect("definition lowers");
    let def = env.lookup_local_type_function("TailOrNil").unwrap();
    match &def.equations[1].patterns[0] {
        TypeFunctionPattern::DomainConstructor { fields, .. } => {
            assert!(matches!(fields[0], TypeFunctionPattern::Var { ref name, .. } if name == "h"));
            assert!(matches!(fields[1], TypeFunctionPattern::Var { ref name, .. } if name == "t"));
        }
        other => panic!("expected constructor pattern, got {other:?}"),
    }
    match &def.equations[1].result {
        TypeFunctionResultExpr::DomainConstructorApp { args, .. } => {
            assert!(matches!(args[0], TypeFunctionResultExpr::Var { ref name, .. } if name == "h"));
            assert!(matches!(args[1], TypeFunctionResultExpr::Var { ref name, .. } if name == "t"));
        }
        other => panic!("expected marker-constructor RHS app, got {other:?}"),
    }
}
