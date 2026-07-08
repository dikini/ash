//! TASK-880: proposition checking points integrate required discharge without inversion.

use ash_core::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    DomainConstructorId, DomainConstructorSummary, DomainFieldSummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    PropositionFactSummary, PropositionPredicateId, PropositionPredicateParamSummary,
    PropositionPredicateSummary, SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin,
    SummaryVersion,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NamedPredicateProposition, PropositionDeferredKind, PropositionOutcome,
    PropositionRefutationReason, TypeProposition, TypePropositionTerm,
};
use ash_parser::surface::{Definition, InterfaceDef, Visibility, Workflow, WorkflowDef};
use ash_parser::token::Span;
use ash_typeck::type_env::{
    PropositionCheckingSite, PropositionCheckingSiteKind, PropositionFactRole,
};
use ash_typeck::{Kind, TypeEnv, TypeVar, type_check_program_in_env_for_module};

fn module_identity(id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(880)),
        ModuleId(id),
        vec!["task880".to_string(), format!("m{id}")],
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-880-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-880-typeck-test".into(),
    }
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
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

fn register_type_list(env: &mut TypeEnv, module: &ModuleIdentity) {
    let summary = ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::SPEC059_SEALED_DOMAIN_V2)
        .with_exported_sealed_domain(list_domain(module));
    env.register_module_semantic_summary(&summary)
        .expect("public TypeList domain should import for type-fn fixtures");
}

fn type_fns(source: &str) -> Vec<ash_parser::surface::TypeFnDef> {
    let parsed = ash_parser::parse_surface_file(source).expect("source parses");
    parsed
        .definitions
        .into_iter()
        .filter_map(|definition| match definition {
            Definition::TypeFn(type_fn) => Some(type_fn),
            _ => None,
        })
        .collect()
}

fn proposition_tail(source: &str) -> ash_parser::surface::PropositionTail {
    let parsed = ash_parser::parse_surface_file(source).expect("source parses");
    parsed
        .definitions
        .into_iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => function.proposition_tail,
            Definition::BuiltinFn(function) => function.proposition_tail,
            Definition::TypeFn(type_fn) => type_fn.proposition_tail,
            _ => None,
        })
        .expect("fixture should carry a proposition tail")
}

fn parse_program(source: &str) -> ash_parser::surface::Program {
    let module = ash_parser::parse_surface_file(source).expect("program source parses");
    ash_parser::surface::Program {
        definitions: module.definitions,
        helper_workflows: Vec::new(),
        workflow: WorkflowDef {
            name: "main".into(),
            type_params: vec![],
            params: vec![],
            declared_return_type: None,
            plays_roles: vec![],
            capabilities: vec![],
            header_events: vec![],
            body: Workflow::Done {
                span: Span::default(),
            },
            contract: None,
            span: Span::default(),
        },
    }
}

fn register_marker_interface(env: &mut TypeEnv, name: &str) {
    let module = module_identity(50_000 + name.len());
    let id = InterfaceIdentityId::new(module, name);
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        id,
        name,
        vec![name.into()],
        anchor(&format!("interface {name}")),
    ))
    .expect("marker interface identity registers");
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Public,
        name: name.into(),
        type_params: vec![],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![],
        laws: Vec::new(),
        span: Default::default(),
    })
    .expect("marker interface registers");
}

#[test]
fn task_880_public_type_fn_generates_and_discharges_satisfied_proposition_tail() {
    let module = module_identity(1);
    let mut env = TypeEnv::new();
    register_type_list(&mut env, &module);
    let defs = type_fns(
        r#"
        pub type fn Id(xs: TypeList) -> TypeList where Int == Int {
            case Id<xs> = xs;
        }
        "#,
    );

    env.register_local_type_functions(&module, &defs)
        .expect("satisfied proposition tail should pass required type-fn checking point");

    let obligations = env.proposition_obligations();
    assert_eq!(
        obligations.len(),
        1,
        "type-fn tail should generate one obligation"
    );
    assert_eq!(
        obligations[0].owner_site.kind,
        PropositionCheckingSiteKind::ExplicitRequirement
    );
    assert!(matches!(
        obligations[0].outcome,
        Some(PropositionOutcome::Satisfied(_))
    ));
}

#[test]
fn task_880_required_type_fn_deferred_no_inversion_errors_without_publishing_append() {
    let module = module_identity(2);
    let mut env = TypeEnv::new();
    register_type_list(&mut env, &module);
    let defs = type_fns(
        r#"
        pub type fn Append(xs: TypeList, ys: TypeList) -> TypeList
            decreases xs
            where Append<xs, ys> == Cons<A, Nil>
        {
            case Append<Nil, ys> = ys;
            case Append<Cons<h, t>, ys> = Cons<h, Append<t, ys>>;
        }
        "#,
    );

    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("required open Append equality must be rejected, not inverted");

    let message = err.to_string();
    assert!(
        message.contains("proposition")
            && (message.contains("deferred") || message.contains("no-inversion"))
            && message.contains("Append"),
        "expected required-proposition no-inversion diagnostic, got {message}"
    );
    assert!(
        env.lookup_local_type_function("Append").is_none(),
        "failed required propositions must not publish a partially checked type function"
    );
    assert!(
        env.proposition_obligations().is_empty(),
        "failed checking point should not leak obligations into the outer environment"
    );
}

#[test]
fn task_880_refuted_required_proposition_errors_at_required_checking_point() {
    let module = module_identity(3);
    let mut env = TypeEnv::new();
    register_type_list(&mut env, &module);
    let defs = type_fns(
        r#"
        pub type fn Id(xs: TypeList) -> TypeList where Int == String {
            case Id<xs> = xs;
        }
        "#,
    );

    let err = env
        .register_local_type_functions(&module, &defs)
        .expect_err("refuted required equality must fail the checking point");

    let message = err.to_string();
    assert!(
        message.contains("proposition")
            && (message.contains("refuted")
                || message.contains(&format!(
                    "{:?}",
                    PropositionRefutationReason::DefinitionalEquality
                ))),
        "expected required-proposition refutation diagnostic, got {message}"
    );
}

#[test]
fn task_880_public_fn_refuted_tail_fails_real_program_checking_point() {
    let module = module_identity(31);
    let env = TypeEnv::with_builtin_types();
    let program = parse_program(
        r#"
        pub fn bad(x: Int) -> Int where Int == String { x }
        fn main() -> Int { 0 }
        "#,
    );

    let err = type_check_program_in_env_for_module(&env, &program, module).expect_err(
        "public fn proposition tail must be required-discharged by source type checking",
    );
    let message = err.to_string();
    assert!(
        message.contains("proposition") && message.contains("refuted") && message.contains("bad"),
        "expected public fn required proposition refutation, got {message}"
    );
}

#[test]
fn task_880_public_builtin_deferred_tail_fails_real_program_checking_point() {
    let module = module_identity(32);
    let env = TypeEnv::with_builtin_types();
    let program = parse_program(
        r#"
        pub interface Marker {}
        pub builtin fn opaque(x: Int) -> Int where Int: Marker;
        fn main() -> Int { 0 }
        "#,
    );

    let err = type_check_program_in_env_for_module(&env, &program, module).expect_err(
        "public builtin fn proposition tail must be required-discharged by source type checking",
    );
    let message = err.to_string();
    assert!(
        message.contains("proposition")
            && (message.contains("deferred") || message.contains("MissingInterfaceEvidence"))
            && message.contains("opaque"),
        "expected public builtin required proposition deferral, got {message}"
    );
}

#[test]
fn task_880_deferred_named_predicate_remains_non_error_until_required_discharge() {
    let mut env = TypeEnv::with_builtin_types();
    let module = module_identity(4);
    let predicate = ash_core::semantic_summary::PropositionPredicateSummary {
        id: ash_core::semantic_summary::PropositionPredicateId::new(module, "Opaque"),
        exported_name: "Opaque".into(),
        visibility: CoreVisibility::Public,
        params: vec![
            ash_core::semantic_summary::PropositionPredicateParamSummary {
                name: "T".into(),
                ty: ash_core::type_ir::CanonicalTypeExpr::Primitive("Int".into()),
                kind: Kind::Type,
                source_anchor: anchor("Opaque<T> param"),
            },
        ],
        source_anchor: anchor("prop Opaque<T: Int>"),
    };
    env.register_proposition_predicate_summary(&predicate)
        .expect("ordinary named predicate summary registers");
    let tail = proposition_tail("fn needs(x: Int) -> Int where Opaque<Int> { x }");
    env.add_proposition_obligations_from_tail(
        &tail,
        origin(),
        PropositionCheckingSite::new(
            880_004,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("required named predicate".into()),
        ),
    )
    .expect("named predicate tail lowers");

    let outcomes = env
        .solve_proposition_obligations()
        .expect("plain solving may return a deferred outcome without error");
    assert!(matches!(
        &outcomes[0],
        PropositionOutcome::Deferred(reason)
            if reason.kind == PropositionDeferredKind::UnsupportedNamedPredicate
    ));
}

#[test]
fn task_880_failed_required_import_discharge_rolls_back_imported_proposition_state() {
    let mut env = TypeEnv::with_builtin_types();
    let module = module_identity(33);
    let predicate_id = PropositionPredicateId::new(module.clone(), "ImportedOpaque");
    let predicate = PropositionPredicateSummary {
        id: predicate_id.clone(),
        exported_name: "ImportedOpaque".into(),
        visibility: CoreVisibility::Public,
        params: vec![PropositionPredicateParamSummary {
            name: "T".into(),
            ty: CanonicalTypeExpr::Primitive("Int".into()),
            kind: Kind::Type,
            source_anchor: anchor("ImportedOpaque<T> param"),
        }],
        source_anchor: anchor("prop ImportedOpaque<T: Int>"),
    };
    let fact = PropositionFactSummary {
        proposition: TypeProposition::NamedPredicate(NamedPredicateProposition {
            predicate: predicate_id.clone(),
            args: vec![TypePropositionTerm::Canonical(
                CanonicalTypeExpr::Primitive("Int".into()),
            )],
        }),
        role: PropositionFactRole::Requirement,
        source_anchor: anchor("imported opaque proposition fact"),
        predicate_dependencies: vec![predicate_id.clone()],
        dependency_summary_refs: Vec::new(),
        outcome: None,
    };
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC064_PROPOSITIONS_V5)
        .with_exported_proposition_predicate(predicate)
        .with_exported_proposition_fact(fact);

    let err = env
        .register_module_semantic_summaries_and_discharge_required_propositions(&[summary])
        .expect_err("deferred imported required proposition must fail");
    let message = err.to_string();
    assert!(
        message.contains("proposition")
            && (message.contains("deferred") || message.contains("UnsupportedNamedPredicate")),
        "expected required import discharge diagnostic, got {message}"
    );
    assert!(
        env.proposition_predicate_by_id(&predicate_id).is_none(),
        "failed required import discharge must not leave imported predicate state"
    );
    assert!(
        env.proposition_obligations().is_empty(),
        "failed required import discharge must not leave imported proposition obligations"
    );
}

#[test]
fn task_880_assumptions_only_discharge_interface_bounds_from_explicitly_allowed_sites() {
    let mut env = TypeEnv::with_builtin_types();
    register_marker_interface(&mut env, "Displayable");
    let explicit_assumption =
        proposition_tail("fn assume(x: Int) -> Int where Int: Displayable { x }");
    env.add_proposition_assumptions_from_tail(
        &explicit_assumption,
        origin(),
        PropositionCheckingSite::new(
            880_005,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("not an assumption context".into()),
        ),
    )
    .expect("tail can be stored as a fact, but the site is not solver-trusted");
    let required = proposition_tail("fn require(x: Int) -> Int where Int: Displayable { x }");
    env.add_proposition_obligations_from_tail(
        &required,
        origin(),
        PropositionCheckingSite::new(
            880_006,
            PropositionCheckingSiteKind::ExplicitRequirement,
            Some("required interface bound".into()),
        ),
    )
    .expect("interface-bound obligation lowers");

    let outcomes = env
        .solve_proposition_obligations()
        .expect("plain solving should report missing evidence as deferred");
    assert!(matches!(
        &outcomes[0],
        PropositionOutcome::Deferred(reason)
            if reason.kind == PropositionDeferredKind::MissingInterfaceEvidence
    ));

    let mut trusted_env = TypeEnv::with_builtin_types();
    register_marker_interface(&mut trusted_env, "Displayable");
    trusted_env.bind_type_var_interface_bound(TypeVar(7), "Displayable");
    let type_var_bound = ash_core::type_ir::TypeProposition::InterfaceBound(
        ash_core::type_ir::InterfaceBoundProposition {
            subject: ash_core::type_ir::TypePropositionTerm::Canonical(
                ash_core::type_ir::CanonicalTypeExpr::Var("type_var_7".into()),
            ),
            interface: trusted_env
                .interface_identity_for_name("Displayable")
                .expect("interface id")
                .clone(),
            interface_args: vec![],
        },
    );
    let trusted = trusted_env
        .solve_proposition(&type_var_bound, Some(anchor("type_var_7: Displayable")))
        .expect("type-var where-bound assumption is an explicit solver input");
    assert!(matches!(trusted, PropositionOutcome::Satisfied(_)));
}
