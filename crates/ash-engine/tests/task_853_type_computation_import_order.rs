//! TASK-853: import-order and re-export determinism for type-computation summaries.

use ash_core::ast::Visibility as CoreVisibility;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin, SummaryVersion, TypeFunctionClosureMetadata, TypeFunctionExportMode,
    TypeFunctionParamSummary, TypeFunctionRevalidationMetadata, TypeFunctionSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalTypeExpr, TypeComputationHeadId, TypeFunctionEquation,
    TypeFunctionPattern, TypeFunctionPatternConstraint, TypeFunctionResultConstraint,
    TypeFunctionResultExpr, TypeFunctionSourceAnchors,
};
use ash_engine::module_loader::load_ordinary_file;
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::Normalizer;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-853-test".into(),
        },
        None,
        label,
    )
}

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(853)),
        ModuleId(id),
        vec!["task853".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "task-853 import-order determinism tests".to_string(),
        },
    )
}

fn domain(module: &ModuleIdentity, name: &str) -> SealedDomainId {
    SealedDomainId::new(module.clone(), name)
}

fn ctor(domain: &SealedDomainId, name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain.clone(), name)
}

fn head(module: &ModuleIdentity, name: &str) -> TypeComputationHeadId {
    TypeComputationHeadId::new(module.clone(), name)
}

fn param(name: &str, domain: &SealedDomainId) -> TypeFunctionParamSummary {
    TypeFunctionParamSummary {
        name: name.to_string(),
        ty: CanonicalTypeExpr::Primitive("Type".to_string()),
        kind: Kind::Type,
        domain_constraint: Some(domain.clone()),
        source_anchor: anchor(name),
    }
}

fn var_pattern(name: &str, domain: &SealedDomainId) -> TypeFunctionPattern {
    TypeFunctionPattern::Var {
        name: name.to_string(),
        constraint: TypeFunctionPatternConstraint::Domain(domain.clone()),
        source_anchor: anchor(name),
    }
}

fn var_result(name: &str, domain: &SealedDomainId) -> TypeFunctionResultExpr {
    TypeFunctionResultExpr::Var {
        name: name.to_string(),
        kind: Kind::Type,
        constraint: TypeFunctionResultConstraint::Domain(domain.clone()),
        source_anchor: anchor(name),
    }
}

fn ctor_result(domain: &SealedDomainId, ctor_name: &str) -> TypeFunctionResultExpr {
    TypeFunctionResultExpr::DomainConstructorApp {
        constructor: ctor(domain, ctor_name),
        domain: domain.clone(),
        args: vec![],
        kind: Kind::Type,
        constraint: TypeFunctionResultConstraint::Domain(domain.clone()),
        source_anchor: anchor(ctor_name),
    }
}

fn normal_ctor(domain: &SealedDomainId, ctor_name: &str) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor(domain, ctor_name),
        domain: domain.clone(),
        args: vec![],
        kind: Kind::Type,
    }
}

const fn summary_metadata(
    type_function_count: usize,
    sealed_domain_count: usize,
) -> (
    TypeFunctionClosureMetadata,
    TypeFunctionRevalidationMetadata,
) {
    (
        TypeFunctionClosureMetadata {
            public_closure_checked: true,
            public_ordinary_type_count: 0,
            public_sealed_domain_count: sealed_domain_count,
            public_type_function_count: type_function_count,
            public_projection_count: 0,
        },
        TypeFunctionRevalidationMetadata {
            spec_version: SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
            structural_recursion_checked: true,
            kind_and_domain_checked: true,
            coverage_and_overlap_checked: true,
            decreases_param: None,
        },
    )
}

fn type_function_summary(
    module: &ModuleIdentity,
    name: &str,
    params: Vec<TypeFunctionParamSummary>,
    result_domain: &SealedDomainId,
    equations: Vec<TypeFunctionEquation>,
    closure_type_function_count: usize,
    closure_sealed_domain_count: usize,
) -> TypeFunctionSummary {
    let (closure_metadata, revalidation_metadata) =
        summary_metadata(closure_type_function_count, closure_sealed_domain_count);
    TypeFunctionSummary {
        exported_name: name.to_string(),
        head: head(module, name),
        visibility: CoreVisibility::Public,
        params,
        return_type: CanonicalTypeExpr::Primitive("Type".to_string()),
        return_kind: Kind::Type,
        result_constraint: TypeFunctionResultConstraint::Domain(result_domain.clone()),
        export_mode: TypeFunctionExportMode::TransparentEquations,
        source_anchors: TypeFunctionSourceAnchors {
            definition: anchor(name),
            decreases: None,
        },
        equations,
        dependency_summary_refs: vec![],
        closure_metadata,
        revalidation_metadata,
    }
}

fn import_order_summaries() -> (
    ModuleSemanticSummary,
    ModuleSemanticSummary,
    TypeComputationHeadId,
    SealedDomainId,
) {
    let module_a = module("a", 1);
    let module_b = module("b", 2);
    let domain_a = domain(&module_a, "ABox");
    let domain_b = domain(&module_b, "BBox");
    let head_id_a = head(&module_a, "IdA");
    let head_use_a = head(&module_b, "UseA");

    let summary_a = ModuleSemanticSummary::new(module_a.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_sealed_domain(
            SealedDomainSummary::new(
                domain_a.clone(),
                "ABox",
                CoreVisibility::Public,
                anchor("ABox"),
            )
            .with_constructor(DomainConstructorSummary::new(
                ctor(&domain_a, "AOnly"),
                "AOnly",
                vec![],
                anchor("AOnly"),
            ))
            .with_constructor(DomainConstructorSummary::new(
                ctor(&domain_a, "AFromB"),
                "AFromB",
                vec![DomainFieldSummary::constrained_to(
                    "child",
                    &domain_a,
                    domain_b.clone(),
                )],
                anchor("AFromB"),
            )),
        )
        .with_exported_type_function(type_function_summary(
            &module_a,
            "IdA",
            vec![param("x", &domain_a)],
            &domain_a,
            vec![TypeFunctionEquation {
                head: head_id_a.clone(),
                ordinal: 0,
                patterns: vec![var_pattern("x", &domain_a)],
                result: var_result("x", &domain_a),
                source_anchor: anchor("case IdA<x>"),
                case_head_anchor: anchor("IdA"),
            }],
            1,
            2,
        ));

    let summary_b = ModuleSemanticSummary::new(module_b.clone())
        .with_version(SummaryVersion::SPEC062_TYPE_COMPUTATION_V3)
        .with_exported_sealed_domain(
            SealedDomainSummary::new(
                domain_b.clone(),
                "BBox",
                CoreVisibility::Public,
                anchor("BBox"),
            )
            .with_constructor(DomainConstructorSummary::new(
                ctor(&domain_b, "BOnly"),
                "BOnly",
                vec![],
                anchor("BOnly"),
            )),
        )
        .with_exported_type_function(type_function_summary(
            &module_b,
            "UseA",
            vec![param("x", &domain_b)],
            &domain_a,
            vec![TypeFunctionEquation {
                head: head_use_a.clone(),
                ordinal: 0,
                patterns: vec![var_pattern("x", &domain_b)],
                result: TypeFunctionResultExpr::ComputationHeadApp {
                    head: head_id_a,
                    args: vec![ctor_result(&domain_a, "AOnly")],
                    kind: Kind::Type,
                    constraint: TypeFunctionResultConstraint::Domain(domain_a.clone()),
                    source_anchor: anchor("IdA<AOnly>"),
                },
                source_anchor: anchor("case UseA<x>"),
                case_head_anchor: anchor("UseA"),
            }],
            2,
            2,
        ));

    (summary_a, summary_b, head_use_a, domain_a)
}

fn type_function_names(loaded: &ash_engine::module_loader::LoadedOrdinaryFile) -> Vec<String> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|type_function| type_function.exported_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn find_type_function<'a>(
    loaded: &'a ash_engine::module_loader::LoadedOrdinaryFile,
    name: &str,
) -> &'a TypeFunctionSummary {
    loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .find(|type_function| type_function.exported_name == name)
        .unwrap_or_else(|| panic!("missing type function {name}"))
}

const PROVIDER_WITH_HELPERS: &str = r"
pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }
pub sealed type domain Unrelated { Other; }

pub type fn Helper(xs: TypeList) -> TypeList { case Helper<xs> = xs; }
pub type fn UseHelper(xs: TypeList) -> TypeList { case UseHelper<xs> = Helper<xs>; }
pub type fn Sibling(xs: TypeList) -> TypeList { case Sibling<xs> = xs; }
";

#[test]
fn import_order_permutations_batch_register_cross_summary_dependencies() {
    let (summary_a, summary_b, use_a_head, domain_a) = import_order_summaries();

    // These single-summary registrations document the regression this test guards:
    // source-order one-at-a-time import cannot validate the cross-summary sealed
    // domain field and computation-head reference atomically.
    let mut source_order_env = TypeEnv::new();
    assert!(
        source_order_env
            .register_module_semantic_summary(&summary_a)
            .is_err(),
        "A alone should fail because its public sealed-domain field references B"
    );
    let mut reverse_order_env = TypeEnv::new();
    assert!(
        reverse_order_env
            .register_module_semantic_summary(&summary_b)
            .is_err(),
        "B alone should fail because its public equation references A's head"
    );

    for summaries in [
        vec![summary_a.clone(), summary_b.clone()],
        vec![summary_b, summary_a],
    ] {
        let mut env = TypeEnv::new();
        env.register_module_semantic_summaries(&summaries)
            .expect("batch summary import is order-independent");
        let reduced = Normalizer::new(&env)
            .normalize_known_computation_app(
                &use_a_head,
                vec![normal_ctor(&domain(&module("b", 2), "BBox"), "BOnly")],
                &Kind::Type,
            )
            .expect("normalization succeeds");
        assert_eq!(reduced, normal_ctor(&domain_a, "AOnly"));
    }
}

#[test]
fn pub_use_reexport_preserves_original_head_and_equation_order() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let direct = dir.path().join("direct.ash");
    let reexported = dir.path().join("reexported.ash");

    write_file(
        &provider,
        r"
pub sealed type domain Bits { Zero; One; }
pub type fn Prefer(x: Bits) -> Bits {
    case Prefer<Zero> = One;
    case Prefer<x> = x;
}
",
    );
    write_file(&facade, "pub use provider::{Prefer};\n");
    write_file(&direct, "use provider::{Prefer}\nworkflow main { ret 0 }\n");
    write_file(
        &reexported,
        "use facade::{Prefer}\nworkflow main { ret 0 }\n",
    );

    let direct = load_ordinary_file(&direct).expect("direct import loads");
    let reexported = load_ordinary_file(&reexported).expect("re-exported import loads");
    let direct_prefer = find_type_function(&direct, "Prefer");
    let reexported_prefer = find_type_function(&reexported, "Prefer");

    assert_eq!(reexported_prefer.head, direct_prefer.head);
    assert_eq!(
        reexported_prefer
            .equations
            .iter()
            .map(|equation| equation.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(reexported_prefer.equations, direct_prefer.equations);
}

#[test]
fn repeated_imports_are_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(&provider, PROVIDER_WITH_HELPERS);
    write_file(
        &caller,
        r"use provider::{UseHelper}
use provider::{UseHelper}
use provider::{Helper}
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("repeated imports load");
    let names = type_function_names(&loaded);
    assert_eq!(names, vec!["$ash_dependency$Helper", "UseHelper"]);
    assert_eq!(
        loaded
            .imported_type_function_heads
            .iter()
            .filter(|(name, _)| name == "Helper")
            .count(),
        1
    );
}

#[test]
fn named_import_does_not_source_expose_siblings_or_dependency_helpers() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    let helper_caller = dir.path().join("helper_caller.ash");
    let sibling_caller = dir.path().join("sibling_caller.ash");
    write_file(&provider, PROVIDER_WITH_HELPERS);
    write_file(&facade, "pub use provider::{UseHelper};\n");
    write_file(
        &caller,
        "use facade::{UseHelper}\nworkflow main { ret 0 }\n",
    );
    write_file(
        &helper_caller,
        "use facade::{Helper}\nworkflow main { ret 0 }\n",
    );
    write_file(
        &sibling_caller,
        "use facade::{Sibling}\nworkflow main { ret 0 }\n",
    );

    let loaded = load_ordinary_file(&caller).expect("selected re-export import loads");
    assert_eq!(
        type_function_names(&loaded),
        vec!["$ash_dependency$Helper", "UseHelper"]
    );
    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .flat_map(|summary| summary.exported_sealed_domains.iter())
            .all(|domain| domain.exported_name != "Unrelated"),
        "unrelated sibling sealed-domain summaries must not leak"
    );
    assert!(
        loaded
            .imported_type_defs
            .iter()
            .all(|type_def| type_def.name != "Helper" && type_def.name != "Sibling"),
        "type-function helper/sibling names must not become ordinary source-visible types"
    );

    let err = load_ordinary_file(&helper_caller)
        .expect_err("dependency helper is normalizer-available but not re-export source-visible");
    assert!(
        err.to_string().contains("item 'Helper' not found"),
        "unexpected error: {err}"
    );

    let err = load_ordinary_file(&sibling_caller)
        .expect_err("unrelated sibling is not re-export source-visible");
    assert!(
        err.to_string().contains("item 'Sibling' not found"),
        "unexpected error: {err}"
    );
}

#[test]
fn glob_import_imports_all_public_heads_deterministically() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let first = dir.path().join("first.ash");
    let second = dir.path().join("second.ash");
    write_file(&provider, PROVIDER_WITH_HELPERS);
    write_file(&first, "use provider::*\nworkflow main { ret 0 }\n");
    write_file(
        &second,
        "use provider::*\nuse provider::*\nworkflow main { ret 0 }\n",
    );

    let first = load_ordinary_file(&first).expect("glob import loads");
    let second = load_ordinary_file(&second).expect("repeated glob import loads");
    let expected = vec![
        "Helper".to_string(),
        "Sibling".to_string(),
        "UseHelper".to_string(),
    ];

    assert_eq!(type_function_names(&first), expected);
    assert_eq!(type_function_names(&second), expected);
    assert_eq!(
        first
            .imported_semantic_summaries
            .iter()
            .flat_map(|summary| summary.exported_type_functions.iter())
            .map(|type_function| type_function.head.clone())
            .collect::<Vec<_>>(),
        second
            .imported_semantic_summaries
            .iter()
            .flat_map(|summary| summary.exported_type_functions.iter())
            .map(|type_function| type_function.head.clone())
            .collect::<Vec<_>>()
    );
}
