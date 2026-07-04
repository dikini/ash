use ash_core::core_ash::{CoreSourceSpan, CoreType};
use ash_core::core_ash_contract::{
    BoundaryKind, ContractRecoverability, CoreBlameLabel, CoreBlameParty, CoreBlamePolarity,
    DiagnosticShape, DynamicPredicatePlan, LoweredPredicateBuilder, PredicateBinder,
    PredicateBinderKind, PredicateClassification, PredicateEnvironment, PredicateFunctionRef,
    PredicateNode, ProofFragment, RuntimeCheckPlan, SnapshotRef,
};

fn bool_ty() -> CoreType {
    CoreType::Base("Bool".into())
}

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn span(start: usize, end: usize) -> CoreSourceSpan {
    CoreSourceSpan {
        file: Some("task_1694.ash".into()),
        start,
        end,
    }
}

fn binder(boundary: &str, id: &str, name: &str, ty: CoreType) -> PredicateBinder {
    PredicateBinder::new(
        boundary,
        id,
        name,
        PredicateBinderKind::Parameter,
        ty,
        span(0, 1),
    )
}

fn snapshot(boundary: &str, root: &PredicateBinder) -> SnapshotRef {
    SnapshotRef::new(
        boundary,
        root.id().clone(),
        vec!["len".into()],
        int_ty(),
        span(10, 20),
    )
}

#[test]
fn stable_identity_depends_on_binders_snapshots_predicate_fns_and_types_not_source_text() {
    let boundary = "fn:push:ensures";
    let xs = binder(
        boundary,
        "xs",
        "xs",
        CoreType::App {
            name: "List".into(),
            args: vec![int_ty()],
        },
    );
    let old_len = snapshot(boundary, &xs);
    let predicate_fn = PredicateFunctionRef::new(
        vec!["contracts".into(), "sorted".into()],
        vec![xs.ty().clone()],
        bool_ty(),
    );
    let env = PredicateEnvironment::new(
        boundary,
        vec![xs.clone()],
        vec![old_len.clone()],
        vec![predicate_fn.clone()],
    );
    let root = PredicateNode::And(
        Box::new(PredicateNode::PredicateCall {
            callee: predicate_fn,
            args: vec![PredicateNode::Binder(xs.ref_())],
        }),
        Box::new(PredicateNode::Ge(
            Box::new(PredicateNode::Snapshot(old_len.clone())),
            Box::new(PredicateNode::IntLit(0)),
        )),
    );

    let first = LoweredPredicateBuilder::new(boundary, env.clone(), root.clone(), bool_ty())
        .source(span(0, 40), "sorted(xs) && old(xs.len) >= 0")
        .classification(PredicateClassification::Dynamic)
        .dynamic_plan(DynamicPredicatePlan::Interpreter {
            boundary_kind: BoundaryKind::Ensures,
            environment_binders: Vec::new(),
            predicate_node: PredicateNode::BoolLit(true),
        })
        .diagnostic_shape(DiagnosticShape::predicate_false(
            "sorted-list-postcondition",
        ))
        .build();

    let same_semantics_different_text =
        LoweredPredicateBuilder::new(boundary, env, root.clone(), bool_ty())
            .source(
                span(100, 180),
                " /* comment */ sorted ( xs ) && old ( xs . len ) >= 0",
            )
            .classification(PredicateClassification::Dynamic)
            .dynamic_plan(DynamicPredicatePlan::Interpreter {
                boundary_kind: BoundaryKind::Ensures,
                environment_binders: Vec::new(),
                predicate_node: PredicateNode::BoolLit(true),
            })
            .diagnostic_shape(DiagnosticShape::predicate_false("different wording"))
            .build();

    assert_eq!(
        first.predicate_ref().stable_hash,
        same_semantics_different_text.predicate_ref().stable_hash
    );
    assert_eq!(first.id(), same_semantics_different_text.id());
    assert_ne!(
        first.contract_text(),
        same_semantics_different_text.contract_text()
    );

    let changed_type_binder = binder(
        boundary,
        "xs",
        "xs",
        CoreType::App {
            name: "List".into(),
            args: vec![CoreType::Base("String".into())],
        },
    );
    let changed_env = PredicateEnvironment::new(
        boundary,
        vec![changed_type_binder.clone()],
        vec![snapshot(boundary, &changed_type_binder)],
        Vec::new(),
    );
    let changed_type = LoweredPredicateBuilder::new(
        boundary,
        changed_env,
        PredicateNode::Binder(changed_type_binder.ref_()),
        bool_ty(),
    )
    .source(span(0, 3), "xs")
    .build();

    assert_ne!(
        first.predicate_ref().stable_hash,
        changed_type.predicate_ref().stable_hash
    );
}

#[test]
fn stable_identity_ignores_proof_and_runtime_discharge_metadata() {
    let boundary = "fn:push:ensures";
    let x = binder(boundary, "x", "x", int_ty());
    let env = PredicateEnvironment::new(boundary, vec![x.clone()], Vec::new(), Vec::new());
    let root = PredicateNode::Gt(
        Box::new(PredicateNode::Binder(x.ref_())),
        Box::new(PredicateNode::IntLit(0)),
    );

    let static_predicate =
        LoweredPredicateBuilder::new(boundary, env.clone(), root.clone(), bool_ty())
            .classification(PredicateClassification::Static)
            .proof_fragment(ProofFragment::SmtSafe)
            .build();
    let dynamic_predicate = LoweredPredicateBuilder::new(boundary, env, root, bool_ty())
        .classification(PredicateClassification::Dynamic)
        .dynamic_plan(DynamicPredicatePlan::Interpreter {
            boundary_kind: BoundaryKind::Ensures,
            environment_binders: Vec::new(),
            predicate_node: PredicateNode::BoolLit(true),
        })
        .build();

    assert_eq!(
        static_predicate.predicate_ref().stable_hash,
        dynamic_predicate.predicate_ref().stable_hash
    );
    assert_eq!(static_predicate.id(), dynamic_predicate.id());
}

#[test]
fn snapshot_refs_are_boundary_local_even_for_same_root_and_path() {
    let entry_root = binder("fn:push:requires", "xs", "xs", int_ty());
    let exit_root = binder("fn:push:ensures", "xs", "xs", int_ty());

    let entry_snapshot = SnapshotRef::new(
        "fn:push:requires",
        entry_root.id().clone(),
        vec!["len".into()],
        int_ty(),
        span(1, 5),
    );
    let exit_snapshot = SnapshotRef::new(
        "fn:push:ensures",
        exit_root.id().clone(),
        vec!["len".into()],
        int_ty(),
        span(1, 5),
    );

    assert_ne!(entry_snapshot, exit_snapshot);
    assert_eq!(entry_snapshot.boundary().as_str(), "fn:push:requires");
    assert_eq!(exit_snapshot.boundary().as_str(), "fn:push:ensures");
}

#[test]
fn runtime_check_plan_reuses_lowered_predicate_ref_and_captured_environment() {
    let boundary = "fn:push:ensures";
    let result = binder(boundary, "result", "result", int_ty());
    let env = PredicateEnvironment::new(boundary, vec![result.clone()], Vec::new(), Vec::new());
    let predicate = LoweredPredicateBuilder::new(
        boundary,
        env.clone(),
        PredicateNode::Gt(
            Box::new(PredicateNode::Binder(result.ref_())),
            Box::new(PredicateNode::IntLit(0)),
        ),
        bool_ty(),
    )
    .classification(PredicateClassification::Dynamic)
    .dynamic_plan(DynamicPredicatePlan::Interpreter {
        boundary_kind: BoundaryKind::Ensures,
        environment_binders: Vec::new(),
        predicate_node: PredicateNode::BoolLit(true),
    })
    .source(span(0, 10), "result > 0")
    .build();

    let plan = RuntimeCheckPlan::new(
        predicate.predicate_ref().clone(),
        env.ref_(),
        DynamicPredicatePlan::Interpreter {
            boundary_kind: BoundaryKind::Ensures,
            environment_binders: Vec::new(),
            predicate_node: PredicateNode::BoolLit(true),
        },
        CoreBlameLabel::new(
            CoreBlameParty::Callee,
            CoreBlamePolarity::Positive,
            boundary,
        ),
        Vec::new(),
        DiagnosticShape::predicate_false("positive-result"),
        ContractRecoverability::TrapDefault,
    );

    assert_eq!(plan.predicate(), predicate.predicate_ref());
    assert_eq!(plan.environment(), &env.ref_());
    assert_eq!(plan.boundary().as_str(), boundary);
    assert!(matches!(
        plan.recoverability(),
        ContractRecoverability::TrapDefault
    ));
}

#[test]
fn builder_requires_boundary_environment_and_bool_type_at_construction_site() {
    let boundary = "law:identity";
    let env = PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new());
    let predicate =
        LoweredPredicateBuilder::new(boundary, env, PredicateNode::BoolLit(true), bool_ty())
            .classification(PredicateClassification::Static)
            .source(span(0, 4), "true")
            .build();

    assert_eq!(predicate.boundary().as_str(), boundary);
    assert_eq!(predicate.ty(), &bool_ty());
    assert_eq!(predicate.classification(), PredicateClassification::Static);
    assert_eq!(predicate.free_vars().len(), 0);
    assert_eq!(predicate.snapshot_refs().len(), 0);
    assert_eq!(predicate.boundary_kind(), BoundaryKind::Unspecified);
}
