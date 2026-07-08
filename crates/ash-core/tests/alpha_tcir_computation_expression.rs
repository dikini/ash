use ash_core::FailureBoundary;
use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, PartialTypeArg, PartialTypeConstructorApp, TcirBinder, TcirClosure,
    TcirComputationExpression, TcirDoTarget, TcirEntryArtifactProvenance,
    TcirExplicitLiftProvenance, TcirFailureBoundaryProvenance, TcirOperation, TcirOperationKind,
    TcirSelectedEvidence, TcirStatement, TcirStatementId, TcirStatementKind, TypeConstructorExpr,
    TypeConstructorHeadId, TypeHoleAmbiguity, TypeHoleId, TypeHoleMetadata,
};
use ash_core::workflow_carrier::{
    ProjectionEvent, ProjectionEventKind, ProjectionKind, SourceOrigin as WorkflowSourceOrigin,
    WorkflowNodeId, WorkflowObligation,
};
use ash_core::workflow_contract::{Effect, Requirement};
use ash_core::{Expr, Span};

fn source(label: &str, start: usize, end: usize) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::File("task-925.ash".to_string()),
        Some(Span { start, end }),
        label,
    )
}

fn type_decl(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(925),
            vec!["task_925".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "TASK-925 test identity".to_string(),
            },
        ),
        name,
    )
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.to_string(),
        span: Span::default(),
    }
}

fn result_partial_target() -> TypeConstructorExpr {
    let result_hole = TypeHoleId::new(925);
    TypeConstructorExpr::PartialApplication(PartialTypeConstructorApp::new_with_hole_metadata(
        TypeConstructorHeadId::nominal(type_decl("Result"), "Result"),
        vec![
            PartialTypeArg::Hole(result_hole),
            PartialTypeArg::Applied(Box::new(CanonicalTypeExpr::NominalApp {
                origin: type_decl("TaskError"),
                visible_name: "TaskError".to_string(),
                args: Vec::new(),
                kind: Kind::Type,
            })),
        ],
        Kind::arrow(Kind::Type, Kind::Type),
        vec![TypeHoleMetadata::new(
            result_hole,
            source("result-value-hole", 10, 11),
            Some(Kind::Type),
            TypeHoleAmbiguity::ExpectedValueSlot,
        )],
        Some(source("do-result-target", 3, 18)),
    ))
}

#[test]
fn tcir_records_source_do_target_and_selected_evidence() {
    let return_op = TcirOperation::evidence_intrinsic(
        "Monad<Result<_, TaskError>>",
        "return",
        vec!["result".to_string()],
        "ok",
        Some(source("result-return-evidence", 41, 47)),
    );
    let bind_op = TcirOperation::evidence_intrinsic(
        "Monad<Result<_, TaskError>>",
        "bind",
        vec!["result".to_string()],
        "and_then",
        Some(source("result-bind-evidence", 21, 39)),
    );
    let tcir = TcirComputationExpression {
        source_anchor: source("do-result-block", 0, 80),
        target: TcirDoTarget {
            constructor: result_partial_target(),
            display: "Result<_, TaskError>".to_string(),
            source_anchor: source("do-result-target", 3, 18),
        },
        evidence: TcirSelectedEvidence {
            interface: "Monad".to_string(),
            evidence_key: "Monad<Result<_, TaskError>>".to_string(),
            return_op: return_op.clone(),
            bind_op: bind_op.clone(),
        },
        boundary_level: FailureBoundary::Effectful,
        result_type: CanonicalTypeExpr::NominalApp {
            origin: type_decl("Result"),
            visible_name: "Result".to_string(),
            args: vec![
                CanonicalTypeExpr::Primitive("Int".to_string()),
                CanonicalTypeExpr::NominalApp {
                    origin: type_decl("TaskError"),
                    visible_name: "TaskError".to_string(),
                    args: Vec::new(),
                    kind: Kind::Type,
                },
            ],
            kind: Kind::Type,
        },
        statements: vec![
            TcirStatement {
                id: TcirStatementId::new(0),
                source_anchor: source("bind-statement", 20, 50),
                kind: TcirStatementKind::Bind {
                    binder: TcirBinder {
                        name: "value".to_string(),
                        source_anchor: Some(source("bind-value", 20, 25)),
                    },
                    source: Box::new(var("selected")),
                    bind_op: Box::new(bind_op.clone()),
                    closure: TcirClosure {
                        source_anchor: source("bind-continuation", 30, 50),
                        params: vec![TcirBinder {
                            name: "value".to_string(),
                            source_anchor: Some(source("bind-value-param", 30, 35)),
                        }],
                        body_statement_ids: vec![TcirStatementId::new(1)],
                    },
                },
            },
            TcirStatement {
                id: TcirStatementId::new(1),
                source_anchor: source("return-statement", 51, 70),
                kind: TcirStatementKind::Return {
                    value: Box::new(var("value")),
                    return_op: Box::new(return_op),
                },
            },
        ],
        explicit_lifts: Vec::new(),
        failure_boundaries: vec![TcirFailureBoundaryProvenance {
            boundary: FailureBoundary::Effectful,
            entity: None,
            source_anchor: source("result-failure-boundary", 0, 80),
            notes: vec![
                "Result domain failures remain selected Monad evidence, not Act failure"
                    .to_string(),
            ],
        }],
        entry_artifact: None,
    };

    assert_eq!(tcir.source_anchor.label, "do-result-block");
    assert_eq!(tcir.target.display, "Result<_, TaskError>");
    assert!(!matches!(
        tcir.target.constructor,
        TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::Nominal { ref visible_name, .. })
            if matches!(visible_name.as_str(), "Act" | "Proc" | "Workflow")
    ));
    let TypeConstructorExpr::PartialApplication(partial) = &tcir.target.constructor else {
        panic!("Result<_, E> target must preserve partial constructor identity");
    };
    assert!(matches!(partial.args[0], PartialTypeArg::Hole(_)));
    assert_eq!(
        partial
            .metadata_for_hole(TypeHoleId::new(925))
            .map(|metadata| metadata.source_anchor.label.as_str()),
        Some("result-value-hole")
    );
    assert_eq!(tcir.evidence.evidence_key, "Monad<Result<_, TaskError>>");
    assert_eq!(tcir.evidence.bind_op, bind_op);
    assert_eq!(tcir.statements[0].source_anchor.label, "bind-statement");
    assert_eq!(tcir.statements[1].source_anchor.label, "return-statement");
    assert_eq!(
        tcir.failure_boundaries[0].boundary,
        FailureBoundary::Effectful
    );
}

#[test]
fn tcir_preserves_boundary_level_and_entry_artifact_provenance() {
    let node = WorkflowNodeId(925);
    let lift_anchor = source("workflow-from-proc", 12, 33);
    let lift = TcirExplicitLiftProvenance {
        operation: TcirOperation::visible_operation(
            vec!["workflow".to_string()],
            "from_proc",
            Some(lift_anchor.clone()),
        ),
        from_boundary: FailureBoundary::Process,
        to_boundary: FailureBoundary::Application,
        source_anchor: lift_anchor.clone(),
    };
    let projection = ProjectionEvent {
        node,
        projection: ProjectionKind::Proc,
        origin: WorkflowSourceOrigin::SourceSpan {
            span: "task-925.ash:12..33".to_string(),
        },
        kind: ProjectionEventKind::FromProc {
            summary: Default::default(),
        },
    };
    let obligation = WorkflowObligation::RequiredCapabilityCovered {
        node,
        capability: "payments.charge".to_string(),
        mode: "required".to_string(),
    };

    let tcir = TcirComputationExpression {
        source_anchor: source("do-workflow-block", 0, 64),
        target: TcirDoTarget {
            constructor: TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(
                type_decl("Workflow"),
                "Workflow",
            )),
            display: "Workflow".to_string(),
            source_anchor: source("do-workflow-target", 3, 11),
        },
        evidence: TcirSelectedEvidence {
            interface: "Monad".to_string(),
            evidence_key: "compiler-prelude::Workflow".to_string(),
            return_op: TcirOperation::hidden_compiler_prelude("workflow::unit", None),
            bind_op: TcirOperation::hidden_compiler_prelude("workflow::bind", None),
        },
        boundary_level: FailureBoundary::Application,
        result_type: CanonicalTypeExpr::NominalApp {
            origin: type_decl("Workflow"),
            visible_name: "Workflow".to_string(),
            args: vec![CanonicalTypeExpr::Primitive("Int".to_string())],
            kind: Kind::Type,
        },
        statements: vec![
            TcirStatement {
                id: TcirStatementId::new(0),
                source_anchor: source("workflow-lift-statement", 12, 33),
                kind: TcirStatementKind::ExplicitLift { lift: lift.clone() },
            },
            TcirStatement {
                id: TcirStatementId::new(1),
                source_anchor: source("entry-artifact", 34, 60),
                kind: TcirStatementKind::EntryArtifact {
                    node,
                    event: ProjectionEventKind::Requires {
                        requirement: Requirement::HasCapability {
                            cap: "payments.charge".to_string(),
                            min_effect: Effect::Operational,
                        },
                    },
                },
            },
        ],
        explicit_lifts: vec![lift.clone()],
        failure_boundaries: vec![TcirFailureBoundaryProvenance {
            boundary: FailureBoundary::Application,
            entity: None,
            source_anchor: source("workflow-failure-boundary", 0, 64),
            notes: vec!["workflow governance reinterprets lower failures at boundary".to_string()],
        }],
        entry_artifact: Some(TcirEntryArtifactProvenance {
            source_origin: WorkflowSourceOrigin::SourceSpan {
                span: "task-925.ash:0..64".to_string(),
            },
            nodes: vec![node],
            projection_events: vec![projection.clone()],
            obligations: vec![obligation.clone()],
        }),
    };

    assert_eq!(tcir.boundary_level, FailureBoundary::Application);
    assert_eq!(tcir.explicit_lifts, vec![lift]);
    assert!(matches!(
        tcir.explicit_lifts[0].operation.kind,
        TcirOperationKind::VisibleOperation { ref module_path, ref name }
            if module_path == &["workflow".to_string()] && name == "from_proc"
    ));
    let artifact = tcir
        .entry_artifact
        .as_ref()
        .expect("entry artifact provenance is retained separately");
    assert_eq!(artifact.nodes, vec![node]);
    assert_eq!(artifact.projection_events, vec![projection]);
    assert_eq!(artifact.obligations, vec![obligation]);
    assert!(tcir.statements.iter().any(|statement| matches!(
        statement.kind,
        TcirStatementKind::EntryArtifact { node: artifact_node, .. } if artifact_node == node
    )));
}

#[test]
fn tcir_user_constructor_evidence_is_not_collapsed_to_runtime_bridge_terms() {
    let return_op = TcirOperation::evidence_method(
        "Monad<Option>",
        "return",
        vec!["value".to_string()],
        Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), var("value"))],
        },
        Some(source("option-return-method", 20, 40)),
    );
    let bind_op = TcirOperation::evidence_method(
        "Monad<Option>",
        "bind",
        vec!["value".to_string(), "next".to_string()],
        var("value"),
        Some(source("option-bind-method", 41, 60)),
    );
    let tcir = TcirComputationExpression {
        source_anchor: source("do-option-block", 0, 64),
        target: TcirDoTarget {
            constructor: TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(
                type_decl("Option"),
                "Option",
            )),
            display: "Option".to_string(),
            source_anchor: source("do-option-target", 3, 9),
        },
        evidence: TcirSelectedEvidence {
            interface: "Monad".to_string(),
            evidence_key: "Monad<Option>".to_string(),
            return_op: return_op.clone(),
            bind_op: bind_op.clone(),
        },
        boundary_level: FailureBoundary::Effectful,
        result_type: CanonicalTypeExpr::NominalApp {
            origin: type_decl("Option"),
            visible_name: "Option".to_string(),
            args: vec![CanonicalTypeExpr::Primitive("Int".to_string())],
            kind: Kind::Type,
        },
        statements: vec![TcirStatement {
            id: TcirStatementId::new(0),
            source_anchor: source("return-statement", 10, 18),
            kind: TcirStatementKind::Return {
                value: Box::new(var("answer")),
                return_op: Box::new(return_op),
            },
        }],
        explicit_lifts: Vec::new(),
        failure_boundaries: vec![TcirFailureBoundaryProvenance {
            boundary: FailureBoundary::Effectful,
            entity: None,
            source_anchor: source("option-failure-boundary", 0, 64),
            notes: vec!["user Monad<Option> evidence remains selected evidence".to_string()],
        }],
        entry_artifact: None,
    };

    assert!(matches!(
        tcir.target.constructor,
        TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::Nominal { ref visible_name, .. })
            if visible_name == "Option"
    ));
    assert!(!matches!(
        tcir.target.constructor,
        TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::Nominal { ref visible_name, .. })
            if matches!(visible_name.as_str(), "Act" | "Proc" | "Workflow")
    ));
    assert!(matches!(
        tcir.evidence.return_op.kind,
        TcirOperationKind::EvidenceMethod { ref evidence_key, ref method, .. }
            if evidence_key == "Monad<Option>" && method == "return"
    ));
    assert!(matches!(
        tcir.evidence.bind_op.kind,
        TcirOperationKind::EvidenceMethod { ref evidence_key, ref method, .. }
            if evidence_key == "Monad<Option>" && method == "bind"
    ));
    assert!(!matches!(
        tcir.evidence.return_op.kind,
        TcirOperationKind::HiddenCompilerPrelude { .. }
    ));
    assert!(!matches!(
        tcir.evidence.bind_op.kind,
        TcirOperationKind::HiddenCompilerPrelude { .. }
    ));
}
