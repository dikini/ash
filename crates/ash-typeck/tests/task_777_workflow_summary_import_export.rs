use ash_core::workflow_carrier::{
    ProjectionEvent, ProjectionEventKind, ProjectionKind, PublicWorkflowSummary, SourceOrigin,
    WorkflowNodeId,
};
use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_typeck::QualifiedName;
use ash_typeck::check_expr::elaborate_typed_do_block;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;
use winnow::Parser;

fn workflow_int() -> Type {
    Type::Constructor {
        name: QualifiedName::root("Workflow"),
        args: vec![Type::Int],
        kind: ash_typeck::Kind::Type,
    }
}

fn fn_to_workflow_int() -> Type {
    Type::Fn(vec![], Box::new(workflow_int()))
}

fn imported_summary(anchor: &str) -> PublicWorkflowSummary {
    PublicWorkflowSummary {
        node_count: 1,
        projection_events: vec![ProjectionEvent {
            node: WorkflowNodeId(77),
            projection: ProjectionKind::Contract,
            origin: SourceOrigin::ImportedSummary {
                module: "flows".to_string(),
                public_anchor: anchor.to_string(),
            },
            kind: ProjectionEventKind::Neutral,
        }],
        coverage: Default::default(),
    }
}

#[test]
fn imported_workflow_summary_allows_do_workflow_to_sequence_call() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("guarded", fn_to_workflow_int());
    env.bind_public_workflow_summary("guarded", imported_summary("guarded"));

    let mut input = new_input(
        r#"do:Workflow {
            x <- guarded();
            return x
        }"#,
    );
    let parsed = expr.parse_next(&mut input).expect("do:Workflow parses");

    let elaborated =
        elaborate_typed_do_block(&env, &parsed).expect("imported workflow summary is accepted");
    let artifact = elaborated
        .workflow_artifact
        .expect("do:Workflow carries a live workflow artifact");

    assert_eq!(elaborated.ty, workflow_int());
    assert!(
        artifact
            .projection_events
            .iter()
            .any(|event| matches!(event.origin, SourceOrigin::ImportedSummary { ref module, ref public_anchor }
                if module == "flows" && public_anchor == "guarded")),
        "imported public workflow summary origin should survive workflow artifact construction"
    );
}

#[test]
fn imported_workflow_summary_allows_workflow_comprehension_to_sequence_call() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("guarded", fn_to_workflow_int());
    env.bind_public_workflow_summary("guarded", imported_summary("guarded"));

    let mut input = new_input("[x | x <- guarded()]: Workflow");
    let parsed = expr
        .parse_next(&mut input)
        .expect("workflow comprehension parses");

    let elaborated = ash_typeck::check_expr::elaborate_typed_comprehension(&env, &parsed)
        .expect("workflow comprehension accepts imported workflow summary");
    let artifact = elaborated
        .workflow_artifact
        .expect("workflow comprehension carries a live workflow artifact");

    assert_eq!(elaborated.ty, workflow_int());
    assert!(
        artifact
            .projection_events
            .iter()
            .any(|event| matches!(event.origin, SourceOrigin::ImportedSummary { ref module, ref public_anchor }
                if module == "flows" && public_anchor == "guarded")),
        "comprehension should reuse the same imported public workflow summary path"
    );
}

#[test]
fn imported_workflow_typed_call_without_summary_is_rejected_as_opaque() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("opaque", fn_to_workflow_int());

    let mut input = new_input(
        r#"do:Workflow {
            x <- opaque();
            return x
        }"#,
    );
    let parsed = expr.parse_next(&mut input).expect("do:Workflow parses");

    let errors = elaborate_typed_do_block(&env, &parsed)
        .expect_err("Workflow<T> imports without public summaries must remain opaque");
    let rendered = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        rendered.contains("public workflow summary") && rendered.contains("opaque Workflow<T>"),
        "missing public workflow summaries should use the existing opaque Workflow diagnostic, got: {rendered}"
    );
}
