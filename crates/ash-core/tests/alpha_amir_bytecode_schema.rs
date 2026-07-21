use ash_core::amir::{
    AmirModule, AmirOpcode, AmirSectionKind, AmirVerificationError, AmirVerifier, BytecodeModule,
    BytecodeOpcode, BytecodeSectionKind, BytecodeVerificationError, BytecodeVerifier,
};
use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TcirBinder, TcirClosure, TcirComputationExpression, TcirDoTarget,
    TcirFailureBoundaryProvenance, TcirOperation, TcirSelectedEvidence, TcirStatement,
    TcirStatementId, TcirStatementKind, TypeConstructorExpr, TypeConstructorHeadId,
};
use ash_core::{Expr, FailureBoundary, Span};

fn assert_error_contains<E: std::fmt::Display, T: std::fmt::Debug>(
    result: Result<T, E>,
    needle: &str,
) {
    let error = result.expect_err("verification should reject malformed artifact");
    let message = error.to_string();
    assert!(
        message.contains(needle),
        "expected error containing '{needle}', got '{message}'"
    );
}

fn source(label: &str, start: usize, end: usize) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::File("/tmp/task-926-source-is-not-read.ash".to_string()),
        Some(Span { start, end }),
        label,
    )
}

fn type_decl(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(926),
            vec!["task_926".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "TASK-926 test identity".to_string(),
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

fn tcir_computation() -> TcirComputationExpression {
    let return_op = TcirOperation::evidence_intrinsic(
        "Monad<Option>",
        "return",
        vec!["option".to_string()],
        "some",
        Some(source("option-return-evidence", 51, 57)),
    );
    let bind_op = TcirOperation::evidence_intrinsic(
        "Monad<Option>",
        "bind",
        vec!["option".to_string()],
        "and_then",
        Some(source("option-bind-evidence", 20, 35)),
    );

    TcirComputationExpression {
        source_anchor: source("do-option-block", 0, 80),
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
        statements: vec![
            TcirStatement {
                id: TcirStatementId::new(10),
                source_anchor: source("bind-statement", 20, 50),
                kind: TcirStatementKind::Bind {
                    binder: TcirBinder {
                        name: "value".to_string(),
                        source_anchor: Some(source("bind-value", 20, 25)),
                    },
                    source: Box::new(var("maybe_value")),
                    bind_op: Box::new(bind_op),
                    closure: TcirClosure {
                        source_anchor: source("bind-continuation", 30, 50),
                        params: vec![TcirBinder {
                            name: "value".to_string(),
                            source_anchor: Some(source("bind-param", 30, 35)),
                        }],
                        body_statement_ids: vec![TcirStatementId::new(11)],
                    },
                },
            },
            TcirStatement {
                id: TcirStatementId::new(11),
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
            source_anchor: source("option-failure-boundary", 0, 80),
            notes: vec!["user Monad<Option> failures remain domain-local".to_string()],
        }],
    }
}

#[test]
fn amir_sections_keep_tcir_source_provenance() {
    let tcir = tcir_computation();

    let amir = AmirModule::from_tcir(&tcir);

    assert_eq!(
        amir.sections
            .iter()
            .map(|section| section.kind)
            .collect::<Vec<_>>(),
        vec![
            AmirSectionKind::Header,
            AmirSectionKind::Blocks,
            AmirSectionKind::DebugTrace,
        ]
    );
    assert!(amir.sections.iter().all(|section| {
        let provenance = section
            .provenance
            .as_ref()
            .expect("AMIR sections carry provenance");
        provenance.tcir_statement.is_none()
            && provenance.source_anchor.as_ref() == Some(&tcir.source_anchor)
            && provenance
                .computation
                .as_ref()
                .map(|computation| computation.evidence_key.as_str())
                == Some("Monad<Option>")
    }));
    AmirVerifier::verify(&amir, &tcir).expect("valid AMIR verifies against TCIR");

    let mut missing_section = amir.clone();
    missing_section.sections[0].provenance = None;
    assert!(matches!(
        AmirVerifier::verify(&missing_section, &tcir),
        Err(AmirVerificationError::MissingSectionProvenance {
            section: AmirSectionKind::Header
        })
    ));

    let mut stale_section = amir.clone();
    stale_section.sections[0]
        .provenance
        .as_mut()
        .expect("valid AMIR section provenance")
        .computation
        .as_mut()
        .expect("valid AMIR computation provenance")
        .evidence_key = "Monad<Result>".to_string();
    assert!(matches!(
        AmirVerifier::verify(&stale_section, &tcir),
        Err(AmirVerificationError::StaleSectionComputationFingerprint {
            section: AmirSectionKind::Header
        })
    ));

    let mut missing_block = amir.clone();
    missing_block.blocks[0].provenance = None;
    assert!(matches!(
        AmirVerifier::verify(&missing_block, &tcir),
        Err(AmirVerificationError::MissingBlockProvenance { .. })
    ));

    let mut stale_block = amir.clone();
    stale_block.blocks[0]
        .provenance
        .as_mut()
        .expect("valid AMIR block provenance")
        .source_anchor = Some(source("stale-block", 0, 80));
    assert!(matches!(
        AmirVerifier::verify(&stale_block, &tcir),
        Err(AmirVerificationError::StaleBlockSource { .. })
    ));

    let mut missing_instruction = amir.clone();
    missing_instruction.blocks[0].instructions[0].provenance = None;
    assert!(matches!(
        AmirVerifier::verify(&missing_instruction, &tcir),
        Err(AmirVerificationError::MissingInstructionProvenance { .. })
    ));

    let mut stale_instruction = amir.clone();
    stale_instruction.blocks[0].instructions[0]
        .provenance
        .as_mut()
        .expect("valid AMIR instruction provenance")
        .source_anchor = Some(source("stale-instruction", 20, 50));
    assert!(matches!(
        AmirVerifier::verify(&stale_instruction, &tcir),
        Err(AmirVerificationError::StaleTcirStatementSource { .. })
    ));

    let mut unsupported_schema = amir.clone();
    unsupported_schema.schema_version = 926;
    assert!(matches!(
        AmirVerifier::verify(&unsupported_schema, &tcir),
        Err(AmirVerificationError::UnsupportedSchemaVersion { version: 926 })
    ));

    let mut stale_opcode = amir.clone();
    stale_opcode.blocks[0].instructions[0].opcode = AmirOpcode::Return;
    assert!(matches!(
        AmirVerifier::verify(&stale_opcode, &tcir),
        Err(AmirVerificationError::StaleInstructionOpcode {
            statement,
            expected: AmirOpcode::Bind,
            actual: AmirOpcode::Return,
            ..
        }) if statement == TcirStatementId::new(10)
    ));

    assert_eq!(amir.blocks.len(), 1);
    assert_eq!(
        amir.blocks[0]
            .instructions
            .iter()
            .map(|instruction| {
                instruction
                    .provenance
                    .as_ref()
                    .and_then(|provenance| provenance.tcir_statement)
            })
            .collect::<Vec<_>>(),
        vec![
            Some(TcirStatementId::new(10)),
            Some(TcirStatementId::new(11))
        ]
    );
    assert_eq!(
        amir.blocks[0].instructions[0]
            .provenance
            .as_ref()
            .and_then(|provenance| provenance.source_anchor.as_ref())
            .map(|anchor| anchor.label.as_str()),
        Some("bind-statement")
    );
}

#[test]
fn amir_verifier_rejects_non_bijective_statement_coverage() {
    let tcir = tcir_computation();
    let amir = AmirModule::from_tcir(&tcir);

    let mut missing_statement = amir.clone();
    missing_statement.blocks[0].instructions.pop();
    assert_error_contains(
        AmirVerifier::verify(&missing_statement, &tcir),
        "missing TCIR statement coverage",
    );

    let mut duplicate_statement = amir.clone();
    duplicate_statement.blocks[0].instructions[1].opcode = AmirOpcode::Bind;
    duplicate_statement.blocks[0].instructions[1]
        .provenance
        .as_mut()
        .expect("valid AMIR instruction provenance")
        .tcir_statement = Some(TcirStatementId::new(10));
    duplicate_statement.blocks[0].instructions[1]
        .provenance
        .as_mut()
        .expect("valid AMIR instruction provenance")
        .source_anchor = Some(source("bind-statement", 20, 50));
    assert_error_contains(
        AmirVerifier::verify(&duplicate_statement, &tcir),
        "duplicate TCIR statement reference",
    );

    let mut empty_instructions_for_non_empty_tcir = amir;
    empty_instructions_for_non_empty_tcir.blocks[0]
        .instructions
        .clear();
    assert_error_contains(
        AmirVerifier::verify(&empty_instructions_for_non_empty_tcir, &tcir),
        "empty AMIR instructions for non-empty TCIR",
    );
}

#[test]
fn bytecode_verifier_rejects_missing_or_stale_provenance() {
    let tcir = tcir_computation();
    let amir = AmirModule::from_tcir(&tcir);
    let valid = BytecodeModule::from_amir(&amir, &tcir).expect("valid AMIR builds bytecode");

    let mut missing_section_provenance = valid.clone();
    missing_section_provenance.sections[0].provenance = None;
    assert!(matches!(
        BytecodeVerifier::verify(&missing_section_provenance, &tcir),
        Err(BytecodeVerificationError::MissingSectionProvenance {
            section: BytecodeSectionKind::Header
        })
    ));

    let mut missing_instruction_source = valid.clone();
    missing_instruction_source.instructions[0]
        .provenance
        .as_mut()
        .expect("test starts from valid bytecode")
        .source_anchor = None;
    assert!(matches!(
        BytecodeVerifier::verify(&missing_instruction_source, &tcir),
        Err(BytecodeVerificationError::MissingInstructionSource { offset: 0 })
    ));

    let mut missing_instruction_provenance = valid.clone();
    missing_instruction_provenance.instructions[0].provenance = None;
    assert!(matches!(
        BytecodeVerifier::verify(&missing_instruction_provenance, &tcir),
        Err(BytecodeVerificationError::MissingInstructionProvenance { offset: 0 })
    ));

    let mut stale_statement = valid.clone();
    stale_statement.instructions[0]
        .provenance
        .as_mut()
        .expect("test starts from valid bytecode")
        .tcir_statement = Some(TcirStatementId::new(926_000));
    assert!(matches!(
        BytecodeVerifier::verify(&stale_statement, &tcir),
        Err(BytecodeVerificationError::StaleTcirStatementReference {
            offset: Some(0),
            statement
        }) if statement == TcirStatementId::new(926_000)
    ));

    let mut stale_computation = valid.clone();
    stale_computation.sections[0]
        .provenance
        .as_mut()
        .expect("valid bytecode section provenance")
        .computation
        .as_mut()
        .expect("valid bytecode computation provenance")
        .target_display = "Result".to_string();
    assert!(matches!(
        BytecodeVerifier::verify(&stale_computation, &tcir),
        Err(BytecodeVerificationError::StaleComputationFingerprint {
            section: BytecodeSectionKind::Header
        })
    ));

    let mut stale_opcode = valid;
    stale_opcode.instructions[0].opcode = BytecodeOpcode::Return;
    assert!(matches!(
        BytecodeVerifier::verify(&stale_opcode, &tcir),
        Err(BytecodeVerificationError::StaleInstructionOpcode {
            offset: 0,
            statement,
            expected: BytecodeOpcode::InvokeBind,
            actual: BytecodeOpcode::Return,
        }) if statement == TcirStatementId::new(10)
    ));
}

#[test]
fn bytecode_verifier_rejects_non_bijective_statement_coverage_and_offsets() {
    let tcir = tcir_computation();
    let amir = AmirModule::from_tcir(&tcir);
    let valid = BytecodeModule::from_amir(&amir, &tcir).expect("valid AMIR builds bytecode");

    let mut missing_statement = valid.clone();
    missing_statement.instructions.pop();
    assert_error_contains(
        BytecodeVerifier::verify(&missing_statement, &tcir),
        "missing TCIR statement coverage",
    );

    let mut duplicate_statement = valid.clone();
    duplicate_statement.instructions[1].opcode = BytecodeOpcode::InvokeBind;
    duplicate_statement.instructions[1]
        .provenance
        .as_mut()
        .expect("valid bytecode instruction provenance")
        .tcir_statement = Some(TcirStatementId::new(10));
    duplicate_statement.instructions[1]
        .provenance
        .as_mut()
        .expect("valid bytecode instruction provenance")
        .source_anchor = Some(source("bind-statement", 20, 50));
    assert_error_contains(
        BytecodeVerifier::verify(&duplicate_statement, &tcir),
        "duplicate TCIR statement reference",
    );

    let mut empty_instructions_for_non_empty_tcir = valid.clone();
    empty_instructions_for_non_empty_tcir.instructions.clear();
    assert_error_contains(
        BytecodeVerifier::verify(&empty_instructions_for_non_empty_tcir, &tcir),
        "empty bytecode instructions for non-empty TCIR",
    );

    let mut duplicate_offset = valid.clone();
    duplicate_offset.instructions[1].offset = 0;
    assert_error_contains(
        BytecodeVerifier::verify(&duplicate_offset, &tcir),
        "duplicate bytecode offset",
    );

    let mut skipped_offset = valid;
    skipped_offset.instructions[1].offset = 2;
    assert_error_contains(
        BytecodeVerifier::verify(&skipped_offset, &tcir),
        "skipped bytecode offset",
    );
}

#[test]
fn verifiers_reject_duplicate_tcir_statement_ids() {
    let mut tcir = tcir_computation();
    tcir.statements[1].id = tcir.statements[0].id;

    let amir = AmirModule::from_tcir(&tcir_computation());
    assert_error_contains(
        AmirVerifier::verify(&amir, &tcir),
        "duplicate TCIR statement id",
    );

    let valid_tcir = tcir_computation();
    let valid_amir = AmirModule::from_tcir(&valid_tcir);
    let bytecode =
        BytecodeModule::from_amir(&valid_amir, &valid_tcir).expect("valid AMIR builds bytecode");
    assert_error_contains(
        BytecodeVerifier::verify(&bytecode, &tcir),
        "duplicate TCIR statement id",
    );
}

#[test]
fn bytecode_schema_validates_without_source_reparse() {
    let tcir = tcir_computation();
    let amir = AmirModule::from_tcir(&tcir);
    let bytecode = BytecodeModule::from_amir(&amir, &tcir).expect("valid AMIR builds bytecode");

    BytecodeVerifier::verify(&bytecode, &tcir).expect("valid bytecode verifies from TCIR only");

    assert!(!bytecode.requires_source_reparse());
    assert_eq!(
        bytecode
            .sections
            .iter()
            .map(|section| section.kind)
            .collect::<Vec<_>>(),
        vec![
            BytecodeSectionKind::Header,
            BytecodeSectionKind::Constants,
            BytecodeSectionKind::Functions,
            BytecodeSectionKind::DebugTrace,
        ]
    );
    assert_eq!(
        bytecode
            .trace_for_instruction(0)
            .and_then(|provenance| provenance.source_anchor.as_ref())
            .map(|anchor| (&anchor.origin, anchor.label.as_str())),
        Some((
            &SourceOrigin::File("/tmp/task-926-source-is-not-read.ash".to_string()),
            "bind-statement"
        ))
    );
}
