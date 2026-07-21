use ash_core::amir::{AMIR_SCHEMA_VERSION, BYTECODE_SCHEMA_VERSION};
use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::runtime_kernel::{
    RuntimeArtifactBuildIdentity, RuntimeArtifactBuildInput, RuntimeArtifactVerifierResult,
    RuntimeConfigId, RuntimeKernelArtifactBuilder, RuntimeProfileId, RuntimeProfileIdentity,
    RuntimeRootSetId, RuntimeTcirCarrierScope,
};
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TcirComputationExpression, TcirDoTarget, TcirOperation,
    TcirSelectedEvidence, TcirStatement, TcirStatementId, TcirStatementKind, TypeConstructorExpr,
    TypeConstructorHeadId,
};
use ash_core::{Expr, FailureBoundary, Span, Value};

fn source(label: &str, start: usize, end: usize) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::File("/tmp/task-935-source-is-not-reparsed.ash".to_string()),
        Some(Span { start, end }),
        label,
    )
}

fn type_decl(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(935),
            vec!["task_935".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "TASK-935 artifact-builder test identity".to_string(),
            },
        ),
        name,
    )
}

fn tcir_computation() -> TcirComputationExpression {
    let return_op = TcirOperation::evidence_intrinsic(
        "Monad<Proc>",
        "return",
        vec!["proc".to_string()],
        "proc_return",
        Some(source("proc-return-evidence", 40, 46)),
    );

    TcirComputationExpression {
        source_anchor: source("proc-block", 0, 64),
        target: TcirDoTarget {
            constructor: TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(
                type_decl("Proc"),
                "Proc",
            )),
            display: "Proc".to_string(),
            source_anchor: source("proc-target", 3, 7),
        },
        evidence: TcirSelectedEvidence {
            interface: "Monad".to_string(),
            evidence_key: "Monad<Proc>".to_string(),
            return_op: return_op.clone(),
            bind_op: return_op.clone(),
        },
        boundary_level: FailureBoundary::Application,
        result_type: CanonicalTypeExpr::NominalApp {
            origin: type_decl("Proc"),
            visible_name: "Proc".to_string(),
            args: vec![CanonicalTypeExpr::Primitive("Int".to_string())],
            kind: Kind::Type,
        },
        statements: vec![TcirStatement {
            id: TcirStatementId::new(935),
            source_anchor: source("return-statement", 40, 58),
            kind: TcirStatementKind::Return {
                value: Box::new(Expr::Literal(Value::Int(7))),
                return_op: Box::new(return_op),
            },
        }],
        explicit_lifts: Vec::new(),
        failure_boundaries: Vec::new(),
    }
}

fn profile() -> RuntimeProfileIdentity {
    RuntimeProfileIdentity::new(
        RuntimeProfileId::new("alpha-local"),
        RuntimeConfigId::new("local-config"),
        vec!["profile=alpha-local".to_string()],
    )
}

fn input(source: &str) -> RuntimeArtifactBuildInput {
    RuntimeArtifactBuildInput::new(
        RuntimeArtifactBuildIdentity::new(
            RuntimeRootSetId::new("workspace:/task-935"),
            profile(),
            "applications/demo.ash",
            "main",
        ),
        source,
        "engine-check:ok;warnings=0",
        tcir_computation(),
        RuntimeTcirCarrierScope::CheckedTcir,
    )
}

#[test]
fn builder_produces_deterministic_verified_language_artifact_summary() {
    let source = "this source text is hashed but not parsed by bytecode verification";
    let first = RuntimeKernelArtifactBuilder::new()
        .build(input(source))
        .expect("artifact builds");
    let second = RuntimeKernelArtifactBuilder::new()
        .build(input(source))
        .expect("artifact builds deterministically");

    assert_eq!(first, second);
    assert_eq!(
        first.artifact_version.as_str(),
        "runtime-kernel-artifact-v1"
    );
    assert!(first.source_hash.starts_with("sha256:"));
    assert!(first.check_summary_hash.starts_with("sha256:"));
    assert_eq!(first.cache_key.source_hash, first.source_hash);
    assert_eq!(first.cache_key.check_summary_hash, first.check_summary_hash);
    assert_eq!(first.definition.entry_name, "main");
    assert_eq!(first.definition.source_identity, first.source_hash);
    assert_eq!(first.artifact.version, first.artifact_version);
    assert_eq!(
        first.tcir.carrier_scope,
        RuntimeTcirCarrierScope::CheckedTcir
    );
    assert_eq!(first.tcir.target_display, "Proc");
    assert_eq!(first.tcir.evidence_key, "Monad<Proc>");
    assert_eq!(first.tcir.statement_ids, vec![TcirStatementId::new(935)]);
    assert_eq!(first.amir.schema_version, AMIR_SCHEMA_VERSION);
    assert_eq!(first.amir.instruction_count, 1);
    assert_eq!(first.bytecode.schema_version, BYTECODE_SCHEMA_VERSION);
    assert_eq!(first.bytecode.instruction_count, 1);
    assert!(
        !first.bytecode.requires_source_reparse,
        "bytecode verification must use carried TCIR/source provenance"
    );
    assert_eq!(first.verifier, RuntimeArtifactVerifierResult::Verified);
}

#[test]
fn builder_hashes_changed_source_without_reparsing_it_for_verification() {
    let valid_artifact = RuntimeKernelArtifactBuilder::new()
        .build(input("fn main() { return 7 }"))
        .expect("artifact builds");
    let unparsable_source_artifact = RuntimeKernelArtifactBuilder::new()
        .build(input(
            "not valid ash syntax, but bytecode verification uses supplied TCIR",
        ))
        .expect("artifact still builds because verifier does not reparse source");

    assert_ne!(
        valid_artifact.source_hash,
        unparsable_source_artifact.source_hash
    );
    assert_eq!(
        valid_artifact.bytecode, unparsable_source_artifact.bytecode,
        "the verifier-normalized bytecode summary is derived from TCIR, not a second source parse"
    );
    assert_eq!(
        unparsable_source_artifact.verifier,
        RuntimeArtifactVerifierResult::Verified
    );
}
