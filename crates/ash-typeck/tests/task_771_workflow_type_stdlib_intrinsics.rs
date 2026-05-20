use ash_core::workflow_carrier::{
    AlignmentKey, ContractPlan, CoverageEvidence, ProjectionEvent, ProjectionEventKind,
    ProjectionKind, SourceOrigin, WorkflowBinder, WorkflowForm, WorkflowNodeId, WorkflowObligation,
};
use ash_core::workflow_contract::{Requirement, RolePolicy};
use ash_typeck::{
    QualifiedName, Type, TypeEnv, WorkflowIntrinsicKind, WorkflowIntrinsicParameterClass,
};

fn lookup_fn(env: &TypeEnv, name: &str) -> (Vec<Type>, Type) {
    match env.lookup_variable(name) {
        Some(Type::Fn(params, ret)) => (params, *ret),
        other => panic!("expected {name} to be a function type, got {other:?}"),
    }
}

fn assert_unary_constructor<'a>(ty: &'a Type, expected_name: &str) -> &'a Type {
    match ty {
        Type::Constructor { name, args, .. } if name.display() == expected_name => {
            assert_eq!(args.len(), 1, "{expected_name} should be unary");
            &args[0]
        }
        other => panic!("expected {expected_name}<...>, got {other:?}"),
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

    let (unit_params, unit_ret) = lookup_fn(&env, "workflow::unit");
    assert_eq!(unit_params.len(), 1);
    assert_eq!(
        assert_unary_constructor(&unit_ret, "Workflow"),
        &unit_params[0]
    );

    let (from_proc_params, from_proc_ret) = lookup_fn(&env, "workflow::from_proc");
    assert_eq!(from_proc_params.len(), 1);
    let from_proc_input = assert_unary_constructor(&from_proc_params[0], "Proc");
    assert_eq!(
        assert_unary_constructor(&from_proc_ret, "Workflow"),
        from_proc_input
    );

    let (from_act_params, from_act_ret) = lookup_fn(&env, "workflow::from_act");
    assert_eq!(from_act_params.len(), 1);
    let from_act_input = assert_unary_constructor(&from_act_params[0], "Act");
    assert_eq!(
        assert_unary_constructor(&from_act_ret, "Workflow"),
        from_act_input
    );

    let (then_params, then_ret) = lookup_fn(&env, "workflow::then");
    assert_eq!(then_params.len(), 2);
    assert_unary_constructor(&then_params[0], "Workflow");
    let then_right = assert_unary_constructor(&then_params[1], "Workflow");
    assert_eq!(assert_unary_constructor(&then_ret, "Workflow"), then_right);

    let (bind_params, bind_ret) = lookup_fn(&env, "workflow::bind");
    assert_eq!(bind_params.len(), 2);
    let bind_input = assert_unary_constructor(&bind_params[0], "Workflow");
    let Type::Fn(callback_params, callback_ret) = &bind_params[1] else {
        panic!(
            "workflow::bind callback should be a function: {:?}",
            bind_params[1]
        );
    };
    assert_eq!(callback_params, std::slice::from_ref(bind_input));
    let callback_output = assert_unary_constructor(callback_ret, "Workflow");
    assert_eq!(
        assert_unary_constructor(&bind_ret, "Workflow"),
        callback_output
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
