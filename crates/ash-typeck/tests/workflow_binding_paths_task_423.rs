use ash_parser::surface::{
    ActionRef, Definition, Expr, ImplDef, ImplMethodDef, InterfaceDef, InterfaceMethodSig, Literal,
    Parameter, Pattern, Program, Type as SurfaceType, Visibility as SurfaceVisibility, Workflow,
    WorkflowDef,
};
use ash_parser::token::Span;

fn test_span() -> Span {
    Span::default()
}

fn explain_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Explain".into(),
        type_params: vec!["T".into()],
        methods: vec![InterfaceMethodSig {
            name: "explain".into(),
            params: vec![SurfaceType::Name("T".into())],
            return_type: SurfaceType::Name("String".into()),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn explain_string_impl() -> ImplDef {
    ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Explain".into(),
        type_args: vec![SurfaceType::Name("String".into())],
        methods: vec![ImplMethodDef {
            name: "explain".into(),
            param: "value".into(),
            body: Expr::Literal(Literal::String("policy".into())),
            span: test_span(),
        }],
        span: test_span(),
    }
}

fn for_bound_interface_method_call_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "explain_all".into(),
        type_params: vec![],
        params: vec![Parameter {
            name: "items".into(),
            ty: SurfaceType::List(Box::new(SurfaceType::Name("String".into()))),
            span: test_span(),
        }],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::For {
            pattern: Pattern::Variable("item".into()),
            collection: Expr::Variable("items".into()),
            body: Box::new(Workflow::Ret {
                expr: Expr::InterfaceMethodCall {
                    interface: "Explain".into(),
                    method: "explain".into(),
                    argument: Box::new(Expr::Variable("item".into())),
                    span: test_span(),
                },
                span: test_span(),
            }),
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn for_bound_declared_return_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "first_item".into(),
        type_params: vec![],
        params: vec![Parameter {
            name: "items".into(),
            ty: SurfaceType::List(Box::new(SurfaceType::Name("String".into()))),
            span: test_span(),
        }],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::For {
            pattern: Pattern::Variable("item".into()),
            collection: Expr::Variable("items".into()),
            body: Box::new(Workflow::Ret {
                expr: Expr::Variable("item".into()),
                span: test_span(),
            }),
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn observe_bound_declared_return_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "observed_value".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::Observe {
            capability: "read_policy".into(),
            binding: Some(Pattern::Variable("observed".into())),
            continuation: Some(Box::new(Workflow::Ret {
                expr: Expr::Variable("observed".into()),
                span: test_span(),
            })),
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn propose_binding_workflow() -> WorkflowDef {
    WorkflowDef {
        name: "proposed_value".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::Propose {
            action: ActionRef {
                name: "draft_policy".into(),
                args: vec![],
            },
            binding: Some(Pattern::Variable("proposal".into())),
            continuation: Some(Box::new(Workflow::Ret {
                expr: Expr::Variable("proposal".into()),
                span: test_span(),
            })),
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    }
}

fn program_with_interface_impl_and_workflow(workflow: WorkflowDef) -> Program {
    Program {
        definitions: vec![
            Definition::Interface(explain_interface_def()),
            Definition::Impl(explain_string_impl()),
        ],
        workflow,
    }
}

#[test]
fn program_typecheck_accepts_interface_method_call_on_for_bound_name() {
    let program =
        program_with_interface_impl_and_workflow(for_bound_interface_method_call_workflow());

    let result = ash_typeck::type_check_program(&program);

    assert!(
        result.is_ok(),
        "program typechecking should let collection element types drive Explain::explain(item) typing inside For, got: {:?}",
        result
    );
}

#[test]
fn workflow_typecheck_accepts_declared_return_type_on_for_bound_name() {
    let workflow = for_bound_declared_return_workflow();

    let result = ash_typeck::type_check_workflow_def(&workflow);

    assert!(
        result.is_ok(),
        "workflow typechecking should treat For-bound items as visible to declared return inference, got: {:?}",
        result
    );
}

#[test]
fn workflow_typecheck_rejects_observe_bound_declared_return_without_honest_result_type() {
    let workflow = observe_bound_declared_return_workflow();

    let result = ash_typeck::type_check_workflow_def(&workflow);

    assert!(
        result.is_err(),
        "observe-bound declared returns should fail honestly until observed result typing exists"
    );
}

#[test]
fn workflow_typecheck_rejects_propose_binding_until_result_semantics_exist() {
    let workflow = propose_binding_workflow();

    let result = ash_typeck::type_check_workflow_def(&workflow);

    assert!(
        result.is_err(),
        "surfaced Propose bindings should be rejected explicitly until result semantics are implemented"
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, ash_typeck::TypeCheckError::TypeError(_)),
        "expected explicit type error for unsupported Propose binding, got: {error:?}"
    );
    let error = error.to_string();
    assert!(
        error.contains("Propose") && error.contains("binding") && error.contains("MVP"),
        "expected unsupported Propose binding error, got: {error}"
    );
}

#[test]
fn workflow_typecheck_rejects_non_list_for_collection_honestly() {
    let workflow = WorkflowDef {
        name: "bad_for".into(),
        type_params: vec![],
        params: vec![Parameter {
            name: "item".into(),
            ty: SurfaceType::Name("String".into()),
            span: test_span(),
        }],
        declared_return_type: Some(SurfaceType::Name("String".into())),
        plays_roles: vec![],
        capabilities: vec![],
        body: Workflow::For {
            pattern: Pattern::Variable("x".into()),
            collection: Expr::Variable("item".into()),
            body: Box::new(Workflow::Ret {
                expr: Expr::Variable("x".into()),
                span: test_span(),
            }),
            span: test_span(),
        },
        contract: None,
        span: test_span(),
    };

    let result = ash_typeck::type_check_workflow_def(&workflow);

    assert!(
        result.is_err(),
        "non-list For collections should be rejected honestly instead of fabricating a fresh element type"
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("for collection") && error.contains("list"),
        "expected non-list For collection error, got: {error}"
    );
}
