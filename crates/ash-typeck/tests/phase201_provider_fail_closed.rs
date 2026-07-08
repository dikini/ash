//! Phase 201 provider validation regressions.

use ash_parser::surface::{ActionRef, Expr, Literal, OperationalTarget, Workflow, WorkflowDef};
use ash_parser::token::Span;
use ash_typeck::{TypeCheckError, TypeEnv, type_check_workflow_def_in_env};

fn span() -> Span {
    Span::default()
}

fn main_workflow(body: Workflow) -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: None,
        plays_roles: vec![],
        capabilities: vec![],
        header_events: vec![],
        body,
        contract: None,
        span: span(),
    }
}

#[test]
fn explicit_provider_action_requires_registered_provider() {
    let workflow = main_workflow(Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Explicit {
                provider: "missing_provider".into(),
                action: "noop".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: Some("ignored".into()),
        continuation: Some(Box::new(Workflow::Ret {
            expr: Expr::Literal(Literal::Null),
            span: span(),
        })),
        span: span(),
    });

    let error = type_check_workflow_def_in_env(&TypeEnv::with_builtin_types(), &workflow)
        .expect_err("unregistered explicit provider action must fail closed");

    let TypeCheckError::ResolutionError(message) = error else {
        panic!("expected provider resolution error, got {error:?}");
    };
    assert!(
        message.contains("unknown provider 'missing_provider'"),
        "unexpected error: {message}"
    );
}
