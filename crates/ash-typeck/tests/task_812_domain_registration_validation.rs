//! TASK-812: Sealed-domain TypeEnv registration and validation tests.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorSummary, DomainFieldSummary, ModuleIdentity, ModuleSemanticSummary,
    ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin,
    StructuralFieldStatus, SummaryVersion,
};
use ash_typeck::TypeEnv;

fn module_identity(id: u32, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id as usize),
        path.iter().map(|s| (*s).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-812-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-812-test".into(),
        },
        None,
        label,
    )
}

fn make_domain_summary(
    module: &ModuleIdentity,
    name: &str,
    visibility: CoreVisibility,
    constructors: Vec<DomainConstructorSummary>,
) -> SealedDomainSummary {
    let domain_id = SealedDomainId::new(module.clone(), name);
    let mut summary = SealedDomainSummary::new(domain_id, name, visibility, anchor(name));
    for ctor in constructors {
        summary = summary.with_constructor(ctor);
    }
    summary
}

fn unit_ctor(domain_id: &SealedDomainId, name: &str) -> DomainConstructorSummary {
    DomainConstructorSummary::new(
        ash_core::semantic_summary::DomainConstructorId::new(domain_id.clone(), name),
        name,
        vec![],
        anchor(name),
    )
}

fn fielded_ctor(
    domain_id: &SealedDomainId,
    name: &str,
    fields: Vec<DomainFieldSummary>,
) -> DomainConstructorSummary {
    DomainConstructorSummary::new(
        ash_core::semantic_summary::DomainConstructorId::new(domain_id.clone(), name),
        name,
        fields,
        anchor(name),
    )
}

fn make_v2_summary_with_domain(
    module: &ModuleIdentity,
    domain: SealedDomainSummary,
) -> ModuleSemanticSummary {
    let mut summary =
        ModuleSemanticSummary::new(module.clone()).with_exported_sealed_domain(domain);
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;
    summary
}

// --- Registration tests ---

#[test]
fn register_public_sealed_domain() {
    let module = module_identity(1, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Color");
    let domain = make_domain_summary(
        &module,
        "Color",
        CoreVisibility::Public,
        vec![
            unit_ctor(&domain_id, "Red"),
            unit_ctor(&domain_id, "Green"),
            unit_ctor(&domain_id, "Blue"),
        ],
    );
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("registration should succeed");

    let found = env
        .lookup_sealed_domain("Color")
        .expect("domain should be found");
    assert_eq!(found.constructors.len(), 3);
    assert_eq!(found.visibility, CoreVisibility::Public);
}

#[test]
fn register_sealed_domain_with_fields() {
    let module = module_identity(2, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Nat");
    let domain = make_domain_summary(
        &module,
        "Nat",
        CoreVisibility::Public,
        vec![
            unit_ctor(&domain_id, "Zero"),
            fielded_ctor(
                &domain_id,
                "Succ",
                vec![DomainFieldSummary::constrained_to(
                    "pred",
                    &SealedDomainId::new(module.clone(), "Nat"),
                    SealedDomainId::new(module.clone(), "Nat"),
                )],
            ),
        ],
    );
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("registration should succeed");

    let found = env
        .lookup_sealed_domain("Nat")
        .expect("domain should be found");
    assert_eq!(found.constructors.len(), 2);
    let succ = &found.constructors[1];
    assert_eq!(succ.fields.len(), 1);
    assert_eq!(
        succ.fields[0].structural_status,
        StructuralFieldStatus::StructuralSelfDomain
    );
}

#[test]
fn lookup_sealed_domain_by_id() {
    let module = module_identity(3, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "MyDomain");
    let domain = make_domain_summary(
        &module,
        "MyDomain",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_id, "X")],
    );
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("registration should succeed");

    let found = env
        .lookup_sealed_domain_by_id(&SealedDomainId::new(module, "MyDomain"))
        .expect("domain should be found by id");
    assert_eq!(found.exported_name, "MyDomain");
}

#[test]
fn v1_summary_still_accepted() {
    let module = module_identity(4, &["test"]);
    let summary = ModuleSemanticSummary::new(module);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("V1 summary without sealed domains should still be accepted");
}

#[test]
fn multiple_domains_from_different_modules() {
    let module_a = module_identity(10, &["mod_a"]);
    let module_b = module_identity(11, &["mod_b"]);

    let domain_a_id = SealedDomainId::new(module_a.clone(), "Color");
    let domain_a = make_domain_summary(
        &module_a,
        "Color",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_a_id, "Red")],
    );

    let domain_b_id = SealedDomainId::new(module_b.clone(), "Shape");
    let domain_b = make_domain_summary(
        &module_b,
        "Shape",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_b_id, "Circle")],
    );

    let summary_a = make_v2_summary_with_domain(&module_a, domain_a);
    let summary_b = make_v2_summary_with_domain(&module_b, domain_b);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary_a)
        .expect("first module should register");
    env.register_module_semantic_summary(&summary_b)
        .expect("second module should register");

    assert!(env.lookup_sealed_domain("Color").is_some());
    assert!(env.lookup_sealed_domain("Shape").is_some());
}

// --- Duplicate rejection tests ---

#[test]
fn reject_duplicate_domain_names_in_same_summary() {
    let module = module_identity(20, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Dup");
    let domain = make_domain_summary(
        &module,
        "Dup",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_id, "X")],
    );
    // Create a summary with the same domain exported twice
    let mut summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_sealed_domain(domain.clone())
        .with_exported_sealed_domain(domain);
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(result.is_err(), "duplicate domain names should be rejected");
}

#[test]
fn reject_duplicate_domain_visible_names_across_modules() {
    let module_a = module_identity(30, &["mod_a"]);
    let module_b = module_identity(31, &["mod_b"]);

    let domain_a = make_domain_summary(
        &module_a,
        "Same",
        CoreVisibility::Public,
        vec![unit_ctor(
            &SealedDomainId::new(module_a.clone(), "Same"),
            "X",
        )],
    );
    let domain_b = make_domain_summary(
        &module_b,
        "Same",
        CoreVisibility::Public,
        vec![unit_ctor(
            &SealedDomainId::new(module_b.clone(), "Same"),
            "Y",
        )],
    );

    let summary_a = make_v2_summary_with_domain(&module_a, domain_a);
    let summary_b = make_v2_summary_with_domain(&module_b, domain_b);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary_a)
        .expect("first module should register");
    let result = env.register_module_semantic_summary(&summary_b);
    assert!(
        result.is_err(),
        "duplicate visible domain names across modules should be rejected"
    );
}

// --- Non-interference ---

#[test]
fn ordinary_type_registration_still_works_with_v2_summary() {
    use ash_core::semantic_summary::{
        RepresentationExposure, TypeDeclId, TypeDeclSummary, TypeRepresentationSummary,
    };

    let module = module_identity(40, &["test"]);
    let domain_id = SealedDomainId::new(module.clone(), "Dir");
    let domain = make_domain_summary(
        &module,
        "Dir",
        CoreVisibility::Public,
        vec![unit_ctor(&domain_id, "North")],
    );

    let type_id = TypeDeclId::ordinary(module.clone(), "Foo");
    let type_summary = TypeDeclSummary::new(
        type_id,
        "Foo",
        CoreVisibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor("Foo"),
    );

    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(type_summary)
        .with_exported_sealed_domain(domain);
    // Set V2 version
    let mut summary = summary;
    summary.version = SummaryVersion::SPEC059_SEALED_DOMAIN_V2;

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("mixed ordinary type + sealed domain should register");

    // Both should be accessible
    assert!(
        env.lookup_type("Foo").is_some(),
        "ordinary type should be found"
    );
    assert!(
        env.lookup_sealed_domain("Dir").is_some(),
        "sealed domain should be found"
    );
}

#[test]
fn non_public_domain_rejected_in_imported_summary() {
    let module = module_identity(50, &["other_mod"]);
    let domain_id = SealedDomainId::new(module.clone(), "Private");
    let domain = make_domain_summary(
        &module,
        "Private",
        CoreVisibility::Private,
        vec![unit_ctor(&domain_id, "A")],
    );
    let summary = make_v2_summary_with_domain(&module, domain);

    let mut env = TypeEnv::new();
    let result = env.register_module_semantic_summary(&summary);
    assert!(
        result.is_err(),
        "non-public domain in imported summary should be rejected"
    );
}
