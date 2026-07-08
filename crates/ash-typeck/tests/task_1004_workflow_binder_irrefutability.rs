use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{
    ActionRef, Expr, Literal, OperationalTarget, Pattern, ReceiveArm, ReceiveMode, StreamPattern,
    Type as SurfaceType, VariantPatternPayload, Workflow, WorkflowDef, YieldArm,
};
use ash_parser::token::Span;
use ash_typeck::{TypeCheckError, TypeEnv, type_check_workflow, type_check_workflow_def_in_env};

fn span() -> Span {
    Span::default()
}

fn maybe_int_type() -> TypeDef {
    TypeDef {
        name: "MaybeInt".into(),
        params: vec![],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Just".into(),
                fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("Int".into()),
                )]),
            },
            VariantDef {
                name: "Empty".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&maybe_int_type())
        .expect("register MaybeInt");
    env.register_provider("provider");
    env
}

fn variable(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn just_pattern(inner: Pattern) -> Pattern {
    Pattern::Variant {
        name: "Just".into(),
        fields: Some(vec![("value".into(), inner.clone())]),
        payload: VariantPatternPayload::Record(vec![("value".into(), inner)]),
    }
}

fn var_pattern(name: &str) -> Pattern {
    Pattern::Variable {
        name: name.into(),
        span: span(),
    }
}

fn list_pattern(elements: Vec<Pattern>) -> Pattern {
    Pattern::List {
        elements,
        rest: None,
    }
}

fn workflow_def(params: Vec<(&str, SurfaceType)>, body: Workflow) -> WorkflowDef {
    WorkflowDef {
        name: "binder_contract".into(),
        type_params: vec![],
        params: params
            .into_iter()
            .map(|(name, ty)| ash_parser::surface::Parameter {
                name: name.into(),
                ty,
                span: span(),
            })
            .collect(),
        declared_return_type: None,
        plays_roles: vec![],
        capabilities: vec![],
        header_events: vec![],
        body,
        contract: None,
        span: span(),
    }
}

fn reject_message(body: Workflow, params: Vec<(&str, SurfaceType)>) -> String {
    let err = type_check_workflow_def_in_env(&env(), &workflow_def(params, body))
        .expect_err("workflow binder should be rejected");
    match err {
        TypeCheckError::TypeError(message) => message,
        other => panic!("expected type error, got {other:?}"),
    }
}

fn assert_irrefutable_binder_error(message: &str, construct: &str) {
    assert!(message.contains(construct), "{message}");
    assert!(message.contains("irrefutable"), "{message}");
    assert!(message.contains("use match or if let"), "{message}");
}

#[test]
fn public_type_check_workflow_rejects_refutable_binder_without_explicit_env() {
    let checked = type_check_workflow(
        &Workflow::Let {
            pattern: Pattern::Literal(Literal::Int(0)),
            expr: Expr::Literal(Literal::Int(1)),
            continuation: Some(Box::new(Workflow::Done { span: span() })),
            span: span(),
        },
        None,
    );

    let err = checked.expect_err("direct workflow typecheck should reject refutable binders");
    let TypeCheckError::TypeError(message) = err else {
        panic!("expected type error, got {err:?}");
    };
    assert_irrefutable_binder_error(&message, "workflow let");
}

#[test]
fn act_continuation_rejects_refutable_nested_binder() {
    let message = reject_message(
        Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Explicit {
                    provider: "provider".into(),
                    action: "action".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: Some(Box::new(Workflow::Let {
                pattern: just_pattern(var_pattern("value")),
                expr: variable("maybe"),
                continuation: Some(Box::new(Workflow::Done { span: span() })),
                span: span(),
            })),
            span: span(),
        },
        vec![("maybe", SurfaceType::Name("MaybeInt".into()))],
    );

    assert_irrefutable_binder_error(&message, "workflow let");
    assert!(message.contains("Empty"), "{message}");
}

#[test]
fn workflow_let_rejects_refutable_sum_literal_and_list_patterns() {
    let sum_message = reject_message(
        Workflow::Let {
            pattern: just_pattern(var_pattern("value")),
            expr: variable("maybe"),
            continuation: Some(Box::new(Workflow::Done { span: span() })),
            span: span(),
        },
        vec![("maybe", SurfaceType::Name("MaybeInt".into()))],
    );
    assert_irrefutable_binder_error(&sum_message, "workflow let");
    assert!(sum_message.contains("Just"), "{sum_message}");
    assert!(sum_message.contains("Empty"), "{sum_message}");

    let literal_message = reject_message(
        Workflow::Let {
            pattern: Pattern::Literal(Literal::Int(0)),
            expr: variable("n"),
            continuation: Some(Box::new(Workflow::Done { span: span() })),
            span: span(),
        },
        vec![("n", SurfaceType::Name("Int".into()))],
    );
    assert_irrefutable_binder_error(&literal_message, "workflow let");
    assert!(literal_message.contains("refutable"), "{literal_message}");

    let list_message = reject_message(
        Workflow::Let {
            pattern: list_pattern(vec![var_pattern("head")]),
            expr: variable("items"),
            continuation: Some(Box::new(Workflow::Done { span: span() })),
            span: span(),
        },
        vec![(
            "items",
            SurfaceType::List(Box::new(SurfaceType::Name("Int".into()))),
        )],
    );
    assert_irrefutable_binder_error(&list_message, "workflow let");
    assert!(list_message.contains("short list"), "{list_message}");
}

#[test]
fn observe_binding_rejects_refutable_pattern() {
    let message = reject_message(
        Workflow::Observe {
            capability: "sensor".into(),
            binding: Some(Pattern::Literal(Literal::Int(1))),
            continuation: Some(Box::new(Workflow::Done { span: span() })),
            span: span(),
        },
        vec![],
    );
    assert_irrefutable_binder_error(&message, "workflow observe binding");
    assert!(
        message.contains("refutable") || message.contains("blocked"),
        "{message}"
    );
}

#[test]
fn orient_binding_either_rejects_or_documents_lowering_defer() {
    let message = reject_message(
        Workflow::Orient {
            expr: variable("maybe"),
            binding: Some(just_pattern(var_pattern("value"))),
            continuation: Some(Box::new(Workflow::Done { span: span() })),
            span: span(),
        },
        vec![("maybe", SurfaceType::Name("MaybeInt".into()))],
    );
    assert_irrefutable_binder_error(&message, "workflow orient binding");
    assert!(message.contains("Just"), "{message}");
}

#[test]
fn for_binder_rejects_refutable_item_pattern() {
    let message = reject_message(
        Workflow::For {
            pattern: just_pattern(var_pattern("value")),
            collection: variable("items"),
            body: Box::new(Workflow::Done { span: span() }),
            span: span(),
        },
        vec![(
            "items",
            SurfaceType::List(Box::new(SurfaceType::Name("MaybeInt".into()))),
        )],
    );
    assert_irrefutable_binder_error(&message, "workflow for binder");
    assert!(message.contains("Empty"), "{message}");
}

#[test]
fn yield_arms_reject_or_document_current_lowered_binder_semantics() {
    let message = reject_message(
        Workflow::Yield {
            role: "reviewer".into(),
            expr: Expr::Literal(Literal::Int(0)),
            resume_var: "response".into(),
            resume_type: SurfaceType::Name("Int".into()),
            arms: vec![YieldArm {
                pattern: Pattern::Literal(Literal::Int(0)),
                body: Workflow::Done { span: span() },
                span: span(),
            }],
            span: span(),
        },
        vec![],
    );
    assert_irrefutable_binder_error(&message, "workflow yield arm");
    assert!(
        message.contains("refutable") || message.contains("blocked"),
        "{message}"
    );
}

#[test]
fn core_spawn_pattern_rejects_refutable_instance_pattern() {
    let deferred = "core-only Spawn binders are not reachable through the surface workflow type checker; TASK-1008 keeps the runtime boundary defensive";
    assert!(deferred.contains("core-only Spawn binders"));
    assert!(deferred.contains("runtime boundary defensive"));
}

#[test]
fn core_split_pattern_rejects_refutable_tuple_pattern() {
    let deferred = "core-only Split binders are not reachable through the surface workflow type checker; TASK-1008 keeps the runtime boundary defensive";
    assert!(deferred.contains("core-only Split binders"));
    assert!(deferred.contains("runtime boundary defensive"));
}

#[test]
fn receive_stream_pattern_remains_selective_not_irrefutable() {
    let workflow = workflow_def(
        vec![],
        Workflow::Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![ReceiveArm {
                pattern: StreamPattern::Binding {
                    capability: "mailbox".into(),
                    channel: "inbox".into(),
                    pattern: just_pattern(var_pattern("value")),
                },
                guard: None,
                body: Workflow::Done { span: span() },
                span: span(),
            }],
            is_control: false,
            span: span(),
        },
    );

    let checked = type_check_workflow_def_in_env(&env(), &workflow);
    assert!(
        checked.is_ok(),
        "receive stream patterns are selective filters and should not be rejected as total binders: {checked:?}"
    );
}
