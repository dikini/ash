//! TASK-847: public type-function export-closure validation.

use ash_core::ast::{TypeBody, TypeDef, Visibility as CoreVisibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, DomainConstructorId,
    DomainConstructorSummary, DomainFieldSummary, InterfaceIdentityId, InterfaceIdentitySummary,
    ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin, RepresentationExposure,
    SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId,
    TypeDeclSummary, TypeRepresentationSummary,
};
use ash_parser::surface::{AssociatedTypeDecl, Definition, InterfaceDef, Visibility};
use ash_typeck::TypeEnv;

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(847)),
        ModuleId(id),
        vec!["task847".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-847-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-847-test".into(),
        },
        None,
        label,
    )
}

fn list_domain(
    module: &ModuleIdentity,
    name: &str,
    nil: &str,
    cons: &str,
    visibility: CoreVisibility,
) -> SealedDomainSummary {
    let domain = SealedDomainId::new(module.clone(), name);
    let nil = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), nil),
        nil,
        vec![],
        anchor(nil),
    );
    let cons = DomainConstructorSummary::new(
        DomainConstructorId::new(domain.clone(), cons),
        cons,
        vec![
            DomainFieldSummary::unconstrained("head"),
            DomainFieldSummary::constrained_to("tail", &domain, domain.clone()),
        ],
        anchor(cons),
    );
    SealedDomainSummary::new(domain, name, visibility, anchor(name))
        .with_constructor(nil)
        .with_constructor(cons)
}

fn register_public_domains(env: &mut TypeEnv, module: &ModuleIdentity) {
    let mut summary = ModuleSemanticSummary::new(module.clone()).with_exported_sealed_domain(
        list_domain(module, "TypeList", "Nil", "Cons", CoreVisibility::Public),
    );
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    env.register_module_semantic_summary(&summary)
        .expect("public domains register");
}

fn exported_type_summary(module: ModuleIdentity, name: &str, params: &[&str]) -> TypeDeclSummary {
    TypeDeclSummary::new(
        TypeDeclId::ordinary(module, name),
        name,
        CoreVisibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor(name),
    )
    .with_params(params.iter().map(|param| (*param).to_string()).collect())
}

fn interface_identity(module: &ModuleIdentity, name: &str) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module.clone(), name)
}

fn member_identity(interface: &InterfaceIdentityId, name: &str) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        name,
        vec![interface.name.to_string(), name.to_string()],
    )
}

fn register_projection_metadata(
    env: &mut TypeEnv,
    module: &ModuleIdentity,
    visibility: Visibility,
) {
    let interface = interface_identity(module, "Pair");
    let member = member_identity(&interface, "Item");
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_type(exported_type_summary(module.clone(), "Pair", &["A", "B"]))
        .with_interface_identity(InterfaceIdentitySummary::new(
            interface.clone(),
            "Pair",
            vec!["Pair".into()],
            anchor("interface Pair"),
        ))
        .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
            member,
            "Item",
            anchor("associated type Pair::Item"),
        ));
    env.register_module_semantic_summary(&summary)
        .expect("projection carrier summary registers");
    env.register_interface(&InterfaceDef {
        visibility,
        name: "Pair".into(),
        type_params: vec!["A".into(), "B".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            kind: ash_parser::surface::AssociatedTypeKind::Ordinary,
            span: ash_parser::token::Span::default(),
        }],
        methods: vec![],
        span: ash_parser::token::Span::default(),
    })
    .expect("projection interface registers");
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

fn assert_rejects(source: &str, expected: &str) {
    let module = module_identity(expected.len());
    let mut env = TypeEnv::new();
    register_public_domains(&mut env, &module);
    let defs = type_fns(source);
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("definition should reject");
    let actual = format!("{err}");
    assert!(
        actual.contains(expected),
        "expected diagnostic containing {expected:?}, got {actual}"
    );
}

#[test]
fn accepts_export_closed_public_type_function() {
    let module = module_identity(1);
    let mut env = TypeEnv::new();
    register_public_domains(&mut env, &module);

    let defs = type_fns(
        r#"
        pub type fn Id(xs: TypeList) -> TypeList {
            case Id<xs> = xs;
        }
        "#,
    );

    env.register_local_type_functions(&module, &defs)
        .expect("export-closed public type fn registers");
    let lowered = env
        .lookup_local_type_function("Id")
        .expect("published head");
    assert_eq!(lowered.visibility, CoreVisibility::Public);
}

#[test]
fn rejects_public_type_function_calling_private_helper() {
    assert_rejects(
        r#"
        type fn Helper(xs: TypeList) -> TypeList { case Helper<xs> = xs; }
        pub type fn UseHelper(xs: TypeList) -> TypeList { case UseHelper<xs> = Helper<xs>; }
        "#,
        "public type function 'UseHelper' depends on private type function 'Helper'",
    );
}

#[test]
fn accepts_public_type_function_calling_public_helper() {
    let module = module_identity(2);
    let mut env = TypeEnv::new();
    register_public_domains(&mut env, &module);
    let defs = type_fns(
        r#"
        pub type fn Helper(xs: TypeList) -> TypeList { case Helper<xs> = xs; }
        pub type fn UseHelper(xs: TypeList) -> TypeList { case UseHelper<xs> = Helper<xs>; }
        "#,
    );
    env.register_local_type_functions(&module, &defs)
        .expect("public helper preserves export closure");
}

#[test]
fn rejects_public_type_function_over_private_domain() {
    let module = module_identity(3);
    let mut env = TypeEnv::new();
    env.register_local_sealed_domain_summary(&list_domain(
        &module,
        "PrivateList",
        "PrivateNil",
        "PrivateCons",
        CoreVisibility::Private,
    ))
    .expect("private local domain registers for type function lowering");

    let defs = type_fns(
        r#"
        pub type fn Leak(xs: PrivateList) -> PrivateList {
            case Leak<xs> = xs;
        }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("public type fn over private domain rejects");
    let actual = format!("{err}");
    assert!(
        actual
            .contains("public type function 'Leak' depends on private sealed domain 'PrivateList'"),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_public_type_function_using_private_marker_constructor_in_rhs() {
    let module = module_identity(4);
    let mut env = TypeEnv::new();
    env.register_local_sealed_domain_summary(&list_domain(
        &module,
        "PrivateList",
        "PrivateNil",
        "PrivateCons",
        CoreVisibility::Private,
    ))
    .expect("private local domain registers for type function lowering");

    let defs = type_fns(
        r#"
        pub type fn Make(xs: PrivateList) -> PrivateList {
            case Make<PrivateNil> = PrivateNil;
            case Make<PrivateCons<h, t>> = PrivateNil;
        }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("public type fn constructing private marker rejects");
    let actual = format!("{err}");
    assert!(
        actual.contains(
            "public type function 'Make' depends on private marker constructor 'PrivateNil'"
        ),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_public_type_function_with_private_ordinary_type_in_signature() {
    let module = module_identity(5);
    let mut env = TypeEnv::new();
    register_public_domains(&mut env, &module);
    env.register_type(&TypeDef {
        name: "Secret".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Private,
        builtin: false,
    })
    .expect("private ordinary type registers");

    let defs = type_fns(
        r#"
        pub type fn Leak(xs: TypeList, secret: Secret) -> TypeList {
            case Leak<xs, secret> = xs;
        }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("private ordinary type in public type fn signature rejects");
    let actual = format!("{err}");
    assert!(
        actual.contains("public type function 'Leak' depends on private ordinary type 'Secret'"),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_public_type_function_with_private_ordinary_type_in_rhs() {
    let module = module_identity(6);
    let mut env = TypeEnv::new();
    register_public_domains(&mut env, &module);
    env.register_type(&TypeDef {
        name: "Secret".to_string(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: CoreVisibility::Private,
        builtin: false,
    })
    .expect("private ordinary type registers");

    let defs = type_fns(
        r#"
        pub type fn Leak(xs: TypeList) -> Type {
            case Leak<xs> = Secret;
        }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("private ordinary type in public type fn result rejects");
    let actual = format!("{err}");
    assert!(
        actual.contains("public type function 'Leak' depends on private ordinary type 'Secret'"),
        "unexpected diagnostic: {actual}"
    );
}

#[test]
fn rejects_public_type_function_with_private_projection_result() {
    let module = module_identity(7);
    let mut env = TypeEnv::new();
    register_public_domains(&mut env, &module);
    register_projection_metadata(&mut env, &module, Visibility::Inherited);

    let defs = type_fns(
        r#"
        pub type fn Project(xs: TypeList) -> Pair<TypeList, TypeList>::Item {
            case Project<xs> = Pair<TypeList, TypeList>::Item;
        }
        "#,
    );
    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("private projection in public type fn result rejects");
    let actual = format!("{err}");
    assert!(
        actual
            .contains("public type function 'Project' depends on private projection 'Pair::Item'"),
        "unexpected diagnostic: {actual}"
    );
}
