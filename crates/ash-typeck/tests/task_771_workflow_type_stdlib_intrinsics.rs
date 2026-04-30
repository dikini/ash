use ash_core::workflow_carrier::{
    AlignmentKey, ContractPlan, CoverageEvidence, ProjectionEvent, ProjectionEventKind,
    ProjectionKind, SourceOrigin, WorkflowBinder, WorkflowForm, WorkflowNodeId, WorkflowObligation,
};
use ash_core::workflow_contract::{Requirement, RolePolicy};
use ash_typeck::{
    Kind, QualifiedName, Type, TypeEnv, WorkflowIntrinsicKind, WorkflowIntrinsicParameterClass,
};

fn ctor(name: &str, arg: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![arg],
        kind: Kind::Type,
    }
}

#[test]
fn workflow_is_public_unary_builtin_constructor() {
    let env = TypeEnv::with_builtin_types();
    assert!(env.has_type("Workflow"));
    assert!(
        env.check_type_constructor_arity(&QualifiedName::root("Workflow"), 1)
            .is_ok()
    );

    let err = env
        .check_type_constructor_arity(&QualifiedName::root("Workflow"), 2)
        .expect_err("Workflow<C, A> must not be accepted");
    assert!(err.to_string().contains("Workflow"));
    assert!(err.to_string().contains("expected 1"));
}

#[test]
fn workflow_builtins_are_qualified_only() {
    let env = TypeEnv::with_builtin_types();
    for name in [
        "unit",
        "bind",
        "then",
        "from_proc",
        "from_act",
        "requires",
        "ensures",
    ] {
        assert!(
            env.lookup_variable(name).is_none(),
            "{name} leaked unqualified"
        );
        assert!(
            env.lookup_variable(&format!("workflow::{name}")).is_some()
                || env
                    .lookup_workflow_intrinsic(&format!("workflow::{name}"))
                    .is_some(),
            "workflow::{name} is missing"
        );
    }
}

#[test]
fn workflow_value_builtin_signatures_match_first_slice_surface() {
    let env = TypeEnv::with_builtin_types();
    let a = Type::Var(ash_typeck::types::TypeVar(0));
    let b = Type::Var(ash_typeck::types::TypeVar(1));
    let workflow_a = ctor("Workflow", a.clone());
    let workflow_b = ctor("Workflow", b.clone());
    let proc_a = ctor("Proc", a.clone());
    let act_a = ctor("Act", a.clone());

    assert_eq!(
        env.lookup_variable("workflow::unit"),
        Some(Type::Fn(vec![a.clone()], Box::new(workflow_a.clone())))
    );
    assert_eq!(
        env.lookup_variable("workflow::from_proc"),
        Some(Type::Fn(vec![proc_a], Box::new(workflow_a.clone())))
    );
    assert_eq!(
        env.lookup_variable("workflow::from_act"),
        Some(Type::Fn(vec![act_a], Box::new(workflow_a.clone())))
    );
    assert_eq!(
        env.lookup_variable("workflow::then"),
        Some(Type::Fn(
            vec![workflow_a.clone(), workflow_b.clone()],
            Box::new(workflow_b.clone())
        ))
    );
    assert_eq!(
        env.lookup_variable("workflow::bind"),
        Some(Type::Fn(
            vec![
                workflow_a.clone(),
                Type::Fn(vec![a], Box::new(workflow_b.clone()))
            ],
            Box::new(workflow_b)
        ))
    );
}

#[test]
fn contract_intrinsic_parameter_classes_are_not_source_types_or_plain_values() {
    let env = TypeEnv::with_builtin_types();
    assert!(!env.has_type("Requirement"));
    assert!(!env.has_type("OpenPostcondition"));
    assert!(env.lookup_variable("workflow::requires").is_none());
    assert!(env.lookup_variable("workflow::ensures").is_none());

    let requires = env.lookup_workflow_intrinsic("workflow::requires").unwrap();
    let ensures = env.lookup_workflow_intrinsic("workflow::ensures").unwrap();
    assert_eq!(requires.kind, WorkflowIntrinsicKind::Requires);
    assert_eq!(requires.qualified_name, "workflow::requires");
    assert_eq!(
        requires.parameter_class(),
        WorkflowIntrinsicParameterClass::Requirement
    );
    assert_eq!(requires.parameter_class().as_str(), "Requirement");
    assert_eq!(ensures.kind, WorkflowIntrinsicKind::Ensures);
    assert_eq!(ensures.qualified_name, "workflow::ensures");
    assert_eq!(
        ensures.parameter_class(),
        WorkflowIntrinsicParameterClass::OpenPostcondition
    );
    assert_eq!(ensures.parameter_class().as_str(), "OpenPostcondition");
}

#[test]
fn shared_workflow_carriers_preserve_spec_056_alignment_shape() {
    let node = WorkflowNodeId(7);
    let origin = SourceOrigin::Synthetic {
        parent_span: Some("1:1..1:12".to_string()),
        reason: "test governance node".to_string(),
    };
    let requirement = Requirement::AnyRole(RolePolicy {
        roles: vec!["admin".to_string(), "maintainer".to_string()],
    });

    let form: WorkflowForm<()> = WorkflowForm::Bind {
        node,
        source: Box::new(WorkflowForm::Requires {
            node,
            requirement: requirement.clone(),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::Unit { node, value: () }),
    };

    assert!(matches!(
        form,
        WorkflowForm::Bind {
            binder: WorkflowBinder::Ignored,
            ..
        }
    ));

    let event = ProjectionEvent {
        node,
        projection: ProjectionKind::Contract,
        origin,
        kind: ProjectionEventKind::Requires {
            requirement: requirement.clone(),
        },
    };
    assert_eq!(
        AlignmentKey {
            node,
            projection: ProjectionKind::Contract
        },
        AlignmentKey {
            node: event.node,
            projection: event.projection
        }
    );

    let plan: ContractPlan<()> = ContractPlan::RequirementContract { node, requirement };
    assert!(matches!(plan, ContractPlan::RequirementContract { .. }));
}

#[test]
fn coverage_evidence_has_non_flat_spec_056_components() {
    let node = WorkflowNodeId(11);
    let key = AlignmentKey {
        node,
        projection: ProjectionKind::Provenance,
    };
    let evidence = CoverageEvidence {
        provenance: vec![key.clone()],
        obligations: vec![WorkflowObligation::OpaqueSummaryRejected {
            node,
            imported_name: "opaque.workflow".to_string(),
        }],
        ..CoverageEvidence::default()
    };

    assert_eq!(evidence.provenance, vec![key]);
    assert!(matches!(
        evidence.obligations.as_slice(),
        [WorkflowObligation::OpaqueSummaryRejected { imported_name, .. }]
            if imported_name == "opaque.workflow"
    ));
}
