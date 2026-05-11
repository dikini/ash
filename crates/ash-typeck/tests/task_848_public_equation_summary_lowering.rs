//! TASK-848: transparent public equation summary lowering.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion, TypeFunctionExportMode,
};
use ash_core::type_ir::{TypeFunctionResultConstraint, TypeFunctionResultExpr};
use ash_parser::surface::Definition;
use ash_typeck::TypeEnv;

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(848)),
        ModuleId(id),
        vec!["task848".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-848-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-848-test".into(),
        },
        None,
        label,
    )
}

fn list_domain(module: &ModuleIdentity) -> SealedDomainSummary {
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
    SealedDomainSummary::new(
        domain,
        "TypeList",
        CoreVisibility::Public,
        anchor("TypeList"),
    )
    .with_constructor(nil)
    .with_constructor(cons)
}

fn register_domains(env: &mut TypeEnv, module: &ModuleIdentity) {
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC059_SEALED_DOMAIN_V2)
        .with_exported_sealed_domain(list_domain(module));
    env.register_module_semantic_summary(&summary)
        .expect("domain summary registers");
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

#[test]
fn lowers_only_public_transparent_type_functions_with_source_ordered_equations() {
    let module = module_identity(1);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);

    let defs = type_fns(
        r#"
        pub type fn Tail(xs: TypeList) -> TypeList {
            case Tail<Nil> = Nil;
            case Tail<Cons<h, t>> = t;
        }

        type fn PrivateId(xs: TypeList) -> TypeList {
            case PrivateId<xs> = xs;
        }
        "#,
    );
    env.register_local_type_functions(&module, &defs)
        .expect("type functions register");

    let summaries = env
        .export_public_type_function_summaries(&module)
        .expect("public summaries lower");
    assert_eq!(summaries.len(), 1, "private type fn must not be exported");

    let summary = &summaries[0];
    assert_eq!(summary.exported_name, "Tail");
    assert_eq!(summary.visibility, CoreVisibility::Public);
    assert_eq!(
        summary.export_mode,
        TypeFunctionExportMode::TransparentEquations
    );
    assert_eq!(
        summary.head,
        env.lookup_local_type_function("Tail").unwrap().head
    );
    assert_eq!(
        summary
            .equations
            .iter()
            .map(|eq| eq.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1],
        "checked source equation order is preserved"
    );
    assert!(summary.equations.iter().all(|eq| eq.head == summary.head));
    assert_eq!(summary.params[0].name, "xs");
    assert_eq!(summary.params[0].kind, Kind::Type);
    assert!(summary.params[0].domain_constraint.is_some());
    assert!(matches!(
        summary.result_constraint,
        TypeFunctionResultConstraint::Domain(_)
    ));
    assert!(summary.closure_metadata.public_closure_checked);
    assert_eq!(summary.closure_metadata.public_sealed_domain_count, 1);
    assert_eq!(summary.closure_metadata.public_type_function_count, 1);
    assert_eq!(
        summary.revalidation_metadata.spec_version,
        SummaryVersion::SPEC062_TYPE_COMPUTATION_V3
    );
    assert!(summary.revalidation_metadata.structural_recursion_checked);
    assert!(summary.revalidation_metadata.kind_and_domain_checked);
    assert!(summary.revalidation_metadata.coverage_and_overlap_checked);
    assert!(summary.dependency_summary_refs.iter().any(|dep| {
        dep.summary_ref.module == module
            && dep.summary_ref.version == SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    }));
}

#[test]
fn closure_metadata_counts_public_helper_head_dependencies() {
    let module = module_identity(2);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);

    let defs = type_fns(
        r#"
        pub type fn Helper(xs: TypeList) -> TypeList { case Helper<xs> = xs; }
        pub type fn UseHelper(xs: TypeList) -> TypeList { case UseHelper<xs> = Helper<xs>; }
        "#,
    );
    env.register_local_type_functions(&module, &defs)
        .expect("public helper type functions register");

    let summaries = env
        .export_public_type_function_summaries(&module)
        .expect("public summaries lower");
    let use_helper = summaries
        .iter()
        .find(|summary| summary.exported_name == "UseHelper")
        .expect("UseHelper summary is exported");

    assert_eq!(use_helper.closure_metadata.public_sealed_domain_count, 1);
    assert_eq!(use_helper.closure_metadata.public_type_function_count, 2);
    assert!(use_helper.equations.iter().any(|eq| matches!(
        eq.result,
        TypeFunctionResultExpr::ComputationHeadApp { ref head, .. } if head.name == "Helper"
    )));
    assert!(use_helper.dependency_summary_refs.iter().any(|dep| {
        dep.summary_ref.module == module
            && dep.summary_ref.version == SummaryVersion::SPEC062_TYPE_COMPUTATION_V3
    }));
}

#[test]
fn closure_metadata_counts_transitive_public_helper_head_dependencies() {
    let module = module_identity(3);
    let mut env = TypeEnv::new();
    register_domains(&mut env, &module);

    let defs = type_fns(
        r#"
        pub type fn C(xs: TypeList) -> TypeList { case C<xs> = xs; }
        pub type fn B(xs: TypeList) -> TypeList { case B<xs> = C<xs>; }
        pub type fn A(xs: TypeList) -> TypeList { case A<xs> = B<xs>; }
        "#,
    );
    env.register_local_type_functions(&module, &defs)
        .expect("transitive public helper type functions register");

    let summaries = env
        .export_public_type_function_summaries(&module)
        .expect("public summaries lower");
    let a = summaries
        .iter()
        .find(|summary| summary.exported_name == "A")
        .expect("A summary is exported");

    assert_eq!(
        a.closure_metadata.public_type_function_count, 3,
        "A's normalizer-availability closure must include A, B, and C"
    );
    assert_eq!(a.closure_metadata.public_sealed_domain_count, 1);
    assert!(a.dependency_summary_refs.iter().any(|dep| {
        dep.summary_ref.module == module
            && dep.summary_ref.version == SummaryVersion::SPEC062_TYPE_COMPUTATION_V3
    }));
}
