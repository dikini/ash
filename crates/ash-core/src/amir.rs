//! Minimal AMIR and bytecode logical schema for alpha execution artifacts.
//!
//! The schema is intentionally structural. It provides a stable, traceable
//! execution-artifact spine from TCIR to AMIR to bytecode metadata without
//! implementing VM execution, runtime dispatch, or JIT compilation.

use crate::FailureBoundary;
use crate::semantic_summary::SourceAnchor;
use crate::type_ir::{
    TcirComputationExpression, TcirStatement, TcirStatementId, TcirStatementKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Current AMIR logical schema version.
pub const AMIR_SCHEMA_VERSION: u16 = 1;

/// Current bytecode logical schema version.
pub const BYTECODE_SCHEMA_VERSION: u16 = 1;

/// Debug/provenance edge from AMIR or bytecode back to TCIR and source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TcirSourceProvenance {
    /// Statement-level TCIR edge when the artifact item came from one statement.
    pub tcir_statement: Option<TcirStatementId>,
    /// Source anchor copied from TCIR. Consumers must not reparse source to fill it.
    pub source_anchor: Option<SourceAnchor>,
    /// Typed whole-computation identity copied from TCIR.
    ///
    /// Source anchors alone are diagnostic facts, not artifact identity. This
    /// fingerprint lets verifiers reject stale AMIR/bytecode copied from a
    /// different typed computation that happens to share the same source span.
    pub computation: Option<TcirComputationProvenance>,
}

impl TcirSourceProvenance {
    /// Creates a provenance edge for a whole computation expression.
    #[must_use]
    pub fn computation(tcir: &TcirComputationExpression) -> Self {
        Self {
            tcir_statement: None,
            source_anchor: Some(tcir.source_anchor.clone()),
            computation: Some(TcirComputationProvenance::from_tcir(tcir)),
        }
    }

    /// Creates a provenance edge for one TCIR statement.
    #[must_use]
    pub fn statement(statement: &TcirStatement) -> Self {
        Self {
            tcir_statement: Some(statement.id),
            source_anchor: Some(statement.source_anchor.clone()),
            computation: None,
        }
    }
}

/// Stable typed-computation fingerprint retained as artifact provenance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TcirComputationProvenance {
    /// Source-facing target display retained by TCIR.
    pub target_display: String,
    /// Selected evidence key used for `return`/`bind` lowering.
    pub evidence_key: String,
    /// Semantic boundary attributed to the computation.
    pub boundary_level: FailureBoundary,
    /// Ordered TCIR statement identities in the computation artifact.
    pub statement_ids: Vec<TcirStatementId>,
}

impl TcirComputationProvenance {
    /// Builds the fingerprint from the typed computation artifact.
    #[must_use]
    pub fn from_tcir(tcir: &TcirComputationExpression) -> Self {
        Self {
            target_display: tcir.target.display.clone(),
            evidence_key: tcir.evidence.evidence_key.clone(),
            boundary_level: tcir.boundary_level,
            statement_ids: tcir
                .statements
                .iter()
                .map(|statement| statement.id)
                .collect(),
        }
    }
}

/// AMIR logical-schema verifier.
pub struct AmirVerifier;

impl AmirVerifier {
    /// Verifies AMIR logical sections, blocks, instructions, and typed TCIR provenance.
    ///
    /// # Errors
    ///
    /// Returns an [`AmirVerificationError`] when the AMIR schema, layout, or any
    /// carried TCIR provenance is missing or stale relative to the supplied typed
    /// computation expression.
    pub fn verify(
        module: &AmirModule,
        tcir: &TcirComputationExpression,
    ) -> Result<(), AmirVerificationError> {
        if module.schema_version != AMIR_SCHEMA_VERSION {
            return Err(AmirVerificationError::UnsupportedSchemaVersion {
                version: module.schema_version,
            });
        }

        reject_duplicate_tcir_statement_ids(tcir)
            .map_err(|statement| AmirVerificationError::DuplicateTcirStatementId { statement })?;

        verify_computation_provenance(&module.provenance, tcir)
            .map_err(AmirVerificationError::from_shared)?;

        let expected_sections = [
            AmirSectionKind::Header,
            AmirSectionKind::Blocks,
            AmirSectionKind::DebugTrace,
        ];
        let actual_sections = module
            .sections
            .iter()
            .map(|section| section.kind)
            .collect::<Vec<_>>();
        if actual_sections != expected_sections {
            return Err(AmirVerificationError::UnstableSectionLayout);
        }

        for section in &module.sections {
            let provenance = section.provenance.as_ref().ok_or(
                AmirVerificationError::MissingSectionProvenance {
                    section: section.kind,
                },
            )?;
            verify_computation_provenance(provenance, tcir).map_err(|error| {
                AmirVerificationError::from_shared_for_section(section.kind, error)
            })?;
        }

        let expected_statement_count = tcir.statements.len();
        let actual_instruction_count: usize = module
            .blocks
            .iter()
            .map(|block| block.instructions.len())
            .sum();
        if expected_statement_count > 0 && actual_instruction_count == 0 {
            return Err(AmirVerificationError::EmptyInstructionsForNonEmptyTcir);
        }

        let mut covered_statements = HashSet::with_capacity(actual_instruction_count);
        for block in &module.blocks {
            let provenance = block
                .provenance
                .as_ref()
                .ok_or(AmirVerificationError::MissingBlockProvenance { block: block.id })?;
            verify_computation_provenance(provenance, tcir)
                .map_err(|error| AmirVerificationError::from_shared_for_block(block.id, error))?;

            for instruction in &block.instructions {
                let provenance = instruction.provenance.as_ref().ok_or(
                    AmirVerificationError::MissingInstructionProvenance { block: block.id },
                )?;
                let statement =
                    verify_instruction_provenance(provenance, tcir).map_err(|error| {
                        AmirVerificationError::from_shared_for_instruction(block.id, error)
                    })?;
                if !covered_statements.insert(statement.id) {
                    return Err(AmirVerificationError::DuplicateTcirStatementReference {
                        block: block.id,
                        statement: statement.id,
                    });
                }
                let expected_opcode = AmirOpcode::from_statement_kind(&statement.kind);
                if instruction.opcode != expected_opcode {
                    return Err(AmirVerificationError::StaleInstructionOpcode {
                        block: block.id,
                        statement: statement.id,
                        expected: expected_opcode,
                        actual: instruction.opcode,
                    });
                }
            }
        }
        for statement in &tcir.statements {
            if !covered_statements.contains(&statement.id) {
                return Err(AmirVerificationError::MissingTcirStatementCoverage {
                    statement: statement.id,
                });
            }
        }

        Ok(())
    }
}

/// Minimal sectioned AMIR module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmirModule {
    /// Logical schema version.
    pub schema_version: u16,
    /// Whole-computation provenance.
    pub provenance: TcirSourceProvenance,
    /// Stable logical AMIR sections.
    pub sections: Vec<AmirSection>,
    /// Basic blocks preserving statement-order lowering.
    pub blocks: Vec<AmirBlock>,
}

impl AmirModule {
    /// Builds a minimal traceable AMIR module from a TCIR computation expression.
    ///
    /// This bridge copies source anchors and TCIR statement IDs from the typed
    /// carrier. It does not inspect or reparse source text.
    #[must_use]
    pub fn from_tcir(tcir: &TcirComputationExpression) -> Self {
        let provenance = TcirSourceProvenance::computation(tcir);
        let sections = vec![
            AmirSection::new(AmirSectionKind::Header, provenance.clone()),
            AmirSection::new(AmirSectionKind::Blocks, provenance.clone()),
            AmirSection::new(AmirSectionKind::DebugTrace, provenance.clone()),
        ];
        let instructions = tcir
            .statements
            .iter()
            .map(|statement| AmirInstruction {
                opcode: AmirOpcode::from_statement_kind(&statement.kind),
                provenance: Some(TcirSourceProvenance::statement(statement)),
            })
            .collect();

        Self {
            schema_version: AMIR_SCHEMA_VERSION,
            provenance: provenance.clone(),
            sections,
            blocks: vec![AmirBlock {
                id: AmirBlockId(0),
                provenance: Some(provenance),
                instructions,
            }],
        }
    }
}

/// Stable AMIR logical section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmirSection {
    /// Section role.
    pub kind: AmirSectionKind,
    /// Required provenance copied from TCIR.
    pub provenance: Option<TcirSourceProvenance>,
}

impl AmirSection {
    fn new(kind: AmirSectionKind, provenance: TcirSourceProvenance) -> Self {
        Self {
            kind,
            provenance: Some(provenance),
        }
    }
}

/// AMIR logical section kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmirSectionKind {
    /// Module header and versioning facts.
    Header,
    /// Block/register executable spine.
    Blocks,
    /// Debug/provenance trace table.
    DebugTrace,
}

/// Stable AMIR block identity within one module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AmirBlockId(pub u32);

/// Minimal AMIR block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmirBlock {
    /// Block identity.
    pub id: AmirBlockId,
    /// Required provenance for the block as a logical section of execution.
    pub provenance: Option<TcirSourceProvenance>,
    /// Statement-order AMIR instructions.
    pub instructions: Vec<AmirInstruction>,
}

/// Minimal AMIR instruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmirInstruction {
    /// Logical instruction role.
    pub opcode: AmirOpcode,
    /// Required TCIR/source provenance.
    pub provenance: Option<TcirSourceProvenance>,
}

/// AMIR operation categories retained before concrete bytecode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmirOpcode {
    /// Pure lexical binding.
    Let,
    /// Evidence-selected monadic bind.
    Bind,
    /// Evidence-selected return.
    Return,
    /// Explicit cross-boundary lift.
    ExplicitLift,
    /// Failure-boundary marker.
    FailureBoundary,
}

impl AmirOpcode {
    fn from_statement_kind(kind: &TcirStatementKind) -> Self {
        match kind {
            TcirStatementKind::Let { .. } => Self::Let,
            TcirStatementKind::Bind { .. } => Self::Bind,
            TcirStatementKind::Return { .. } => Self::Return,
            TcirStatementKind::ExplicitLift { .. } => Self::ExplicitLift,
            TcirStatementKind::FailureBoundary { .. } => Self::FailureBoundary,
        }
    }
}

/// Minimal bytecode logical schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BytecodeModule {
    /// Logical schema version.
    pub schema_version: u16,
    /// Stable bytecode sections.
    pub sections: Vec<BytecodeSection>,
    /// Register-shaped instruction stream.
    pub instructions: Vec<BytecodeInstruction>,
}

impl BytecodeModule {
    /// Builds a sectioned bytecode schema from AMIR without implementing runtime execution.
    ///
    /// # Errors
    ///
    /// Returns an [`AmirVerificationError`] if the AMIR artifact is not verified
    /// against the typed TCIR computation before bytecode is created.
    pub fn from_amir(
        amir: &AmirModule,
        tcir: &TcirComputationExpression,
    ) -> Result<Self, AmirVerificationError> {
        AmirVerifier::verify(amir, tcir)?;
        let provenance = amir.provenance.clone();
        let sections = vec![
            BytecodeSection::new(BytecodeSectionKind::Header, provenance.clone()),
            BytecodeSection::new(BytecodeSectionKind::Constants, provenance.clone()),
            BytecodeSection::new(BytecodeSectionKind::Functions, provenance.clone()),
            BytecodeSection::new(BytecodeSectionKind::DebugTrace, provenance),
        ];
        let instructions = amir
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
            .enumerate()
            .map(|(offset, instruction)| BytecodeInstruction {
                offset: offset as u32,
                opcode: BytecodeOpcode::from_amir_opcode(instruction.opcode),
                operands: BytecodeOperand::for_opcode(instruction.opcode),
                provenance: instruction.provenance.clone(),
            })
            .collect();

        Ok(Self {
            schema_version: BYTECODE_SCHEMA_VERSION,
            sections,
            instructions,
        })
    }

    /// Returns false because bytecode verification is defined over carried TCIR/source provenance.
    #[must_use]
    pub const fn requires_source_reparse(&self) -> bool {
        false
    }

    /// Returns the provenance attached to an instruction offset.
    #[must_use]
    pub fn trace_for_instruction(&self, offset: u32) -> Option<&TcirSourceProvenance> {
        self.instructions
            .iter()
            .find(|instruction| instruction.offset == offset)
            .and_then(|instruction| instruction.provenance.as_ref())
    }
}

/// Stable bytecode section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BytecodeSection {
    /// Section role.
    pub kind: BytecodeSectionKind,
    /// Required provenance copied through AMIR from TCIR.
    pub provenance: Option<TcirSourceProvenance>,
}

impl BytecodeSection {
    fn new(kind: BytecodeSectionKind, provenance: TcirSourceProvenance) -> Self {
        Self {
            kind,
            provenance: Some(provenance),
        }
    }
}

/// Stable bytecode logical section kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BytecodeSectionKind {
    /// Module header and schema version.
    Header,
    /// Constant-pool section reserved for value interning.
    Constants,
    /// Function/block instruction section.
    Functions,
    /// Debug/provenance table.
    DebugTrace,
}

/// Register-shaped bytecode instruction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BytecodeInstruction {
    /// Instruction offset in the logical stream.
    pub offset: u32,
    /// Operation code.
    pub opcode: BytecodeOpcode,
    /// Logical operands, shaped for later register/block verification and JIT lowering.
    pub operands: Vec<BytecodeOperand>,
    /// Required TCIR/source provenance.
    pub provenance: Option<TcirSourceProvenance>,
}

/// Minimal bytecode opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BytecodeOpcode {
    /// Evaluate a pure expression into a register.
    EvalPure,
    /// Invoke selected monadic bind/effect helper.
    InvokeBind,
    /// Return through selected evidence.
    Return,
    /// Invoke an explicit cross-boundary lift helper.
    ExplicitLift,
    /// Mark a failure boundary.
    FailureBoundary,
}

impl BytecodeOpcode {
    fn from_amir_opcode(opcode: AmirOpcode) -> Self {
        match opcode {
            AmirOpcode::Let => Self::EvalPure,
            AmirOpcode::Bind => Self::InvokeBind,
            AmirOpcode::Return => Self::Return,
            AmirOpcode::ExplicitLift => Self::ExplicitLift,
            AmirOpcode::FailureBoundary => Self::FailureBoundary,
        }
    }
}

/// Logical bytecode operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BytecodeOperand {
    /// Register index.
    Register(u32),
    /// Basic-block index.
    Block(u32),
    /// Constant-pool index.
    Constant(u32),
}

impl BytecodeOperand {
    fn for_opcode(opcode: AmirOpcode) -> Vec<Self> {
        match opcode {
            AmirOpcode::Let => vec![Self::Register(0)],
            AmirOpcode::Bind => vec![Self::Register(0), Self::Block(0)],
            AmirOpcode::Return => vec![Self::Register(0)],
            AmirOpcode::ExplicitLift | AmirOpcode::FailureBoundary => vec![Self::Constant(0)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SharedProvenanceError {
    MissingComputationSource,
    StaleComputationSource,
    MissingComputationFingerprint,
    StaleComputationFingerprint,
    MissingInstructionSource,
    MissingInstructionTcirStatement,
    StaleTcirStatementReference { statement: TcirStatementId },
    StaleTcirStatementSource { statement: TcirStatementId },
}

fn verify_computation_provenance(
    provenance: &TcirSourceProvenance,
    tcir: &TcirComputationExpression,
) -> Result<(), SharedProvenanceError> {
    let source_anchor = provenance
        .source_anchor
        .as_ref()
        .ok_or(SharedProvenanceError::MissingComputationSource)?;
    if source_anchor != &tcir.source_anchor {
        return Err(SharedProvenanceError::StaleComputationSource);
    }

    let computation = provenance
        .computation
        .as_ref()
        .ok_or(SharedProvenanceError::MissingComputationFingerprint)?;
    if computation != &TcirComputationProvenance::from_tcir(tcir) {
        return Err(SharedProvenanceError::StaleComputationFingerprint);
    }

    Ok(())
}

fn verify_instruction_provenance<'a>(
    provenance: &TcirSourceProvenance,
    tcir: &'a TcirComputationExpression,
) -> Result<&'a TcirStatement, SharedProvenanceError> {
    let source_anchor = provenance
        .source_anchor
        .as_ref()
        .ok_or(SharedProvenanceError::MissingInstructionSource)?;
    let statement = provenance
        .tcir_statement
        .ok_or(SharedProvenanceError::MissingInstructionTcirStatement)?;

    verify_statement_reference(statement, source_anchor, tcir)
}

fn verify_statement_reference<'a>(
    statement: TcirStatementId,
    source_anchor: &SourceAnchor,
    tcir: &'a TcirComputationExpression,
) -> Result<&'a TcirStatement, SharedProvenanceError> {
    let Some(tcir_statement) = tcir
        .statements
        .iter()
        .find(|candidate| candidate.id == statement)
    else {
        return Err(SharedProvenanceError::StaleTcirStatementReference { statement });
    };

    if source_anchor != &tcir_statement.source_anchor {
        return Err(SharedProvenanceError::StaleTcirStatementSource { statement });
    }

    Ok(tcir_statement)
}

fn reject_duplicate_tcir_statement_ids(
    tcir: &TcirComputationExpression,
) -> Result<(), TcirStatementId> {
    let mut statement_ids = HashSet::with_capacity(tcir.statements.len());
    for statement in &tcir.statements {
        if !statement_ids.insert(statement.id) {
            return Err(statement.id);
        }
    }
    Ok(())
}

/// Bytecode logical-schema verifier.
pub struct BytecodeVerifier;

impl BytecodeVerifier {
    /// Verifies bytecode logical sections and TCIR/source provenance.
    ///
    /// # Errors
    ///
    /// Returns a [`BytecodeVerificationError`] when the schema version, logical
    /// section layout, or carried provenance is missing or no longer matches the
    /// TCIR computation expression supplied by the compiler pipeline.
    pub fn verify(
        module: &BytecodeModule,
        tcir: &TcirComputationExpression,
    ) -> Result<(), BytecodeVerificationError> {
        if module.schema_version != BYTECODE_SCHEMA_VERSION {
            return Err(BytecodeVerificationError::UnsupportedSchemaVersion {
                version: module.schema_version,
            });
        }

        reject_duplicate_tcir_statement_ids(tcir).map_err(|statement| {
            BytecodeVerificationError::DuplicateTcirStatementId { statement }
        })?;

        let expected_sections = [
            BytecodeSectionKind::Header,
            BytecodeSectionKind::Constants,
            BytecodeSectionKind::Functions,
            BytecodeSectionKind::DebugTrace,
        ];
        let actual_sections = module
            .sections
            .iter()
            .map(|section| section.kind)
            .collect::<Vec<_>>();
        if actual_sections != expected_sections {
            return Err(BytecodeVerificationError::UnstableSectionLayout);
        }

        for section in &module.sections {
            let provenance = section.provenance.as_ref().ok_or(
                BytecodeVerificationError::MissingSectionProvenance {
                    section: section.kind,
                },
            )?;
            Self::verify_section_provenance(section.kind, provenance, tcir)?;
        }

        if !tcir.statements.is_empty() && module.instructions.is_empty() {
            return Err(BytecodeVerificationError::EmptyInstructionsForNonEmptyTcir);
        }

        let mut seen_offsets = HashSet::with_capacity(module.instructions.len());
        for (expected_offset, instruction) in module.instructions.iter().enumerate() {
            if !seen_offsets.insert(instruction.offset) {
                return Err(BytecodeVerificationError::DuplicateBytecodeOffset {
                    offset: instruction.offset,
                });
            }
            let expected_offset = expected_offset as u32;
            if instruction.offset != expected_offset {
                if instruction.offset > expected_offset {
                    return Err(BytecodeVerificationError::SkippedBytecodeOffset {
                        expected: expected_offset,
                        actual: instruction.offset,
                    });
                }
                return Err(BytecodeVerificationError::DuplicateBytecodeOffset {
                    offset: instruction.offset,
                });
            }
        }

        let mut covered_statements = HashSet::with_capacity(module.instructions.len());
        for instruction in &module.instructions {
            let provenance = instruction.provenance.as_ref().ok_or(
                BytecodeVerificationError::MissingInstructionProvenance {
                    offset: instruction.offset,
                },
            )?;
            let statement =
                Self::verify_instruction_provenance(instruction.offset, provenance, tcir)?;
            if !covered_statements.insert(statement.id) {
                return Err(BytecodeVerificationError::DuplicateTcirStatementReference {
                    offset: instruction.offset,
                    statement: statement.id,
                });
            }
            let expected_opcode =
                BytecodeOpcode::from_amir_opcode(AmirOpcode::from_statement_kind(&statement.kind));
            if instruction.opcode != expected_opcode {
                return Err(BytecodeVerificationError::StaleInstructionOpcode {
                    offset: instruction.offset,
                    statement: statement.id,
                    expected: expected_opcode,
                    actual: instruction.opcode,
                });
            }
        }
        for statement in &tcir.statements {
            if !covered_statements.contains(&statement.id) {
                return Err(BytecodeVerificationError::MissingTcirStatementCoverage {
                    statement: statement.id,
                });
            }
        }

        Ok(())
    }

    fn verify_section_provenance(
        section: BytecodeSectionKind,
        provenance: &TcirSourceProvenance,
        tcir: &TcirComputationExpression,
    ) -> Result<(), BytecodeVerificationError> {
        let source_anchor = provenance
            .source_anchor
            .as_ref()
            .ok_or(BytecodeVerificationError::MissingSectionSource { section })?;
        if provenance.tcir_statement.is_none() {
            verify_computation_provenance(provenance, tcir).map_err(|error| match error {
                SharedProvenanceError::MissingComputationSource => {
                    BytecodeVerificationError::MissingSectionSource { section }
                }
                SharedProvenanceError::StaleComputationSource => {
                    BytecodeVerificationError::StaleComputationSource { section }
                }
                SharedProvenanceError::MissingComputationFingerprint => {
                    BytecodeVerificationError::MissingComputationFingerprint { section }
                }
                SharedProvenanceError::StaleComputationFingerprint => {
                    BytecodeVerificationError::StaleComputationFingerprint { section }
                }
                SharedProvenanceError::MissingInstructionSource
                | SharedProvenanceError::MissingInstructionTcirStatement
                | SharedProvenanceError::StaleTcirStatementReference { .. }
                | SharedProvenanceError::StaleTcirStatementSource { .. } => {
                    BytecodeVerificationError::StaleComputationSource { section }
                }
            })?;
            return Ok(());
        }
        let statement = provenance.tcir_statement.expect("checked above");
        verify_statement_reference(statement, source_anchor, tcir).map_err(
            |error| match error {
                SharedProvenanceError::StaleTcirStatementReference { statement } => {
                    BytecodeVerificationError::StaleTcirStatementReference {
                        offset: None,
                        statement,
                    }
                }
                SharedProvenanceError::StaleTcirStatementSource { statement } => {
                    BytecodeVerificationError::StaleTcirStatementSource {
                        offset: None,
                        statement,
                    }
                }
                _ => BytecodeVerificationError::StaleComputationSource { section },
            },
        )?;
        Ok(())
    }

    fn verify_instruction_provenance<'a>(
        offset: u32,
        provenance: &TcirSourceProvenance,
        tcir: &'a TcirComputationExpression,
    ) -> Result<&'a TcirStatement, BytecodeVerificationError> {
        verify_instruction_provenance(provenance, tcir).map_err(|error| match error {
            SharedProvenanceError::MissingInstructionSource => {
                BytecodeVerificationError::MissingInstructionSource { offset }
            }
            SharedProvenanceError::MissingInstructionTcirStatement => {
                BytecodeVerificationError::MissingInstructionTcirStatement { offset }
            }
            SharedProvenanceError::StaleTcirStatementReference { statement } => {
                BytecodeVerificationError::StaleTcirStatementReference {
                    offset: Some(offset),
                    statement,
                }
            }
            SharedProvenanceError::StaleTcirStatementSource { statement } => {
                BytecodeVerificationError::StaleTcirStatementSource {
                    offset: Some(offset),
                    statement,
                }
            }
            _ => BytecodeVerificationError::MissingInstructionSource { offset },
        })
    }
}

/// Bytecode logical-schema verification errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BytecodeVerificationError {
    /// The bytecode schema version is not supported by this verifier.
    #[error("unsupported bytecode schema version {version}")]
    UnsupportedSchemaVersion {
        /// Observed schema version.
        version: u16,
    },
    /// The bytecode section layout is not the stable alpha layout.
    #[error("unstable bytecode section layout")]
    UnstableSectionLayout,
    /// The bytecode instruction stream is empty even though TCIR has statements.
    #[error("empty bytecode instructions for non-empty TCIR")]
    EmptyInstructionsForNonEmptyTcir,
    /// The TCIR input contains the same statement identity more than once.
    #[error("duplicate TCIR statement id {statement:?}")]
    DuplicateTcirStatementId {
        /// Duplicate statement identity in the TCIR input.
        statement: TcirStatementId,
    },
    /// A bytecode instruction offset appears more than once or goes backwards.
    #[error("duplicate bytecode offset {offset}")]
    DuplicateBytecodeOffset {
        /// Duplicate or regressing offset.
        offset: u32,
    },
    /// The bytecode instruction stream skipped the next logical offset.
    #[error("skipped bytecode offset: expected {expected}, found {actual}")]
    SkippedBytecodeOffset {
        /// Next expected logical offset.
        expected: u32,
        /// Actual offset found in the instruction stream.
        actual: u32,
    },
    /// A bytecode section has no provenance record.
    #[error("missing provenance for bytecode section {section:?}")]
    MissingSectionProvenance {
        /// Section missing provenance.
        section: BytecodeSectionKind,
    },
    /// A bytecode section has no source anchor.
    #[error("missing source anchor for bytecode section {section:?}")]
    MissingSectionSource {
        /// Section missing source provenance.
        section: BytecodeSectionKind,
    },
    /// A bytecode section claims a whole-computation source anchor that no longer matches TCIR.
    #[error("stale computation source for bytecode section {section:?}")]
    StaleComputationSource {
        /// Section with stale computation source.
        section: BytecodeSectionKind,
    },
    /// A bytecode section has no typed computation fingerprint.
    #[error("missing typed computation fingerprint for bytecode section {section:?}")]
    MissingComputationFingerprint {
        /// Section missing typed computation provenance.
        section: BytecodeSectionKind,
    },
    /// A bytecode section claims a typed computation fingerprint that no longer matches TCIR.
    #[error("stale typed computation fingerprint for bytecode section {section:?}")]
    StaleComputationFingerprint {
        /// Section with stale typed computation provenance.
        section: BytecodeSectionKind,
    },
    /// A bytecode instruction has no provenance record.
    #[error("missing provenance for bytecode instruction at offset {offset}")]
    MissingInstructionProvenance {
        /// Instruction offset missing provenance.
        offset: u32,
    },
    /// A bytecode instruction has no source anchor.
    #[error("missing source anchor for bytecode instruction at offset {offset}")]
    MissingInstructionSource {
        /// Instruction offset missing source provenance.
        offset: u32,
    },
    /// A bytecode instruction has no TCIR statement edge.
    #[error("missing TCIR statement for bytecode instruction at offset {offset}")]
    MissingInstructionTcirStatement {
        /// Instruction offset missing a TCIR statement edge.
        offset: u32,
    },
    /// A provenance record points at a statement not present in the supplied TCIR artifact.
    #[error("stale TCIR statement reference {statement:?}")]
    StaleTcirStatementReference {
        /// Instruction offset when the stale edge came from an instruction.
        offset: Option<u32>,
        /// Stale TCIR statement identity.
        statement: TcirStatementId,
    },
    /// A provenance record points at an existing statement but with a stale source anchor.
    #[error("stale source anchor for TCIR statement {statement:?}")]
    StaleTcirStatementSource {
        /// Instruction offset when the stale edge came from an instruction.
        offset: Option<u32>,
        /// TCIR statement identity with mismatched source.
        statement: TcirStatementId,
    },
    /// A bytecode instruction opcode no longer matches the cited TCIR statement.
    #[error("stale opcode for bytecode instruction at offset {offset}")]
    StaleInstructionOpcode {
        /// Instruction offset with stale opcode.
        offset: u32,
        /// TCIR statement cited by provenance.
        statement: TcirStatementId,
        /// Opcode expected from the current TCIR statement kind.
        expected: BytecodeOpcode,
        /// Opcode carried by the bytecode instruction.
        actual: BytecodeOpcode,
    },
    /// A bytecode instruction stream references the same TCIR statement more than once.
    #[error("duplicate TCIR statement reference {statement:?} at bytecode offset {offset}")]
    DuplicateTcirStatementReference {
        /// Instruction offset carrying the duplicate statement edge.
        offset: u32,
        /// Duplicate TCIR statement identity.
        statement: TcirStatementId,
    },
    /// A TCIR statement has no corresponding bytecode instruction.
    #[error("missing TCIR statement coverage for {statement:?}")]
    MissingTcirStatementCoverage {
        /// TCIR statement identity with no bytecode instruction coverage.
        statement: TcirStatementId,
    },
}

/// AMIR logical-schema verification errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AmirVerificationError {
    /// The AMIR schema version is not supported by this verifier.
    #[error("unsupported AMIR schema version {version}")]
    UnsupportedSchemaVersion {
        /// Observed schema version.
        version: u16,
    },
    /// The AMIR section layout is not the stable alpha layout.
    #[error("unstable AMIR section layout")]
    UnstableSectionLayout,
    /// The AMIR instruction stream is empty even though TCIR has statements.
    #[error("empty AMIR instructions for non-empty TCIR")]
    EmptyInstructionsForNonEmptyTcir,
    /// The TCIR input contains the same statement identity more than once.
    #[error("duplicate TCIR statement id {statement:?}")]
    DuplicateTcirStatementId {
        /// Duplicate statement identity in the TCIR input.
        statement: TcirStatementId,
    },
    /// The module-level provenance has no source anchor.
    #[error("missing AMIR module source provenance")]
    MissingModuleSource,
    /// The module-level source anchor no longer matches TCIR.
    #[error("stale AMIR module source provenance")]
    StaleModuleSource,
    /// The module-level provenance has no typed computation fingerprint.
    #[error("missing AMIR module typed computation fingerprint")]
    MissingModuleComputationFingerprint,
    /// The module-level typed computation fingerprint no longer matches TCIR.
    #[error("stale AMIR module typed computation fingerprint")]
    StaleModuleComputationFingerprint,
    /// An AMIR section has no provenance record.
    #[error("missing provenance for AMIR section {section:?}")]
    MissingSectionProvenance {
        /// Section missing provenance.
        section: AmirSectionKind,
    },
    /// An AMIR section has no source anchor.
    #[error("missing source anchor for AMIR section {section:?}")]
    MissingSectionSource {
        /// Section missing source provenance.
        section: AmirSectionKind,
    },
    /// An AMIR section claims stale whole-computation source.
    #[error("stale computation source for AMIR section {section:?}")]
    StaleSectionSource {
        /// Section with stale source provenance.
        section: AmirSectionKind,
    },
    /// An AMIR section has no typed computation fingerprint.
    #[error("missing typed computation fingerprint for AMIR section {section:?}")]
    MissingSectionComputationFingerprint {
        /// Section missing typed computation provenance.
        section: AmirSectionKind,
    },
    /// An AMIR section claims stale typed computation provenance.
    #[error("stale typed computation fingerprint for AMIR section {section:?}")]
    StaleSectionComputationFingerprint {
        /// Section with stale typed computation provenance.
        section: AmirSectionKind,
    },
    /// An AMIR block has no provenance record.
    #[error("missing provenance for AMIR block {block:?}")]
    MissingBlockProvenance {
        /// Block missing provenance.
        block: AmirBlockId,
    },
    /// An AMIR block has no source anchor.
    #[error("missing source anchor for AMIR block {block:?}")]
    MissingBlockSource {
        /// Block missing source provenance.
        block: AmirBlockId,
    },
    /// An AMIR block claims stale whole-computation source.
    #[error("stale computation source for AMIR block {block:?}")]
    StaleBlockSource {
        /// Block with stale source provenance.
        block: AmirBlockId,
    },
    /// An AMIR block has no typed computation fingerprint.
    #[error("missing typed computation fingerprint for AMIR block {block:?}")]
    MissingBlockComputationFingerprint {
        /// Block missing typed computation provenance.
        block: AmirBlockId,
    },
    /// An AMIR block claims stale typed computation provenance.
    #[error("stale typed computation fingerprint for AMIR block {block:?}")]
    StaleBlockComputationFingerprint {
        /// Block with stale typed computation provenance.
        block: AmirBlockId,
    },
    /// An AMIR instruction has no provenance record.
    #[error("missing provenance for AMIR instruction in block {block:?}")]
    MissingInstructionProvenance {
        /// Block containing the instruction.
        block: AmirBlockId,
    },
    /// An AMIR instruction has no source anchor.
    #[error("missing source anchor for AMIR instruction in block {block:?}")]
    MissingInstructionSource {
        /// Block containing the instruction.
        block: AmirBlockId,
    },
    /// An AMIR instruction has no TCIR statement edge.
    #[error("missing TCIR statement for AMIR instruction in block {block:?}")]
    MissingInstructionTcirStatement {
        /// Block containing the instruction.
        block: AmirBlockId,
    },
    /// An AMIR provenance record points at a missing TCIR statement.
    #[error("stale TCIR statement reference {statement:?}")]
    StaleTcirStatementReference {
        /// Block containing the stale edge when instruction-scoped.
        block: Option<AmirBlockId>,
        /// Stale TCIR statement identity.
        statement: TcirStatementId,
    },
    /// An AMIR provenance record points at a stale statement source anchor.
    #[error("stale source anchor for TCIR statement {statement:?}")]
    StaleTcirStatementSource {
        /// Block containing the stale edge when instruction-scoped.
        block: Option<AmirBlockId>,
        /// TCIR statement identity with mismatched source.
        statement: TcirStatementId,
    },
    /// An AMIR instruction opcode no longer matches the cited TCIR statement.
    #[error("stale opcode for AMIR instruction in block {block:?}")]
    StaleInstructionOpcode {
        /// Block containing the stale opcode.
        block: AmirBlockId,
        /// TCIR statement cited by provenance.
        statement: TcirStatementId,
        /// Opcode expected from the current TCIR statement kind.
        expected: AmirOpcode,
        /// Opcode carried by the AMIR instruction.
        actual: AmirOpcode,
    },
    /// An AMIR instruction stream references the same TCIR statement more than once.
    #[error("duplicate TCIR statement reference {statement:?} in AMIR block {block:?}")]
    DuplicateTcirStatementReference {
        /// Block containing the duplicate statement edge.
        block: AmirBlockId,
        /// Duplicate TCIR statement identity.
        statement: TcirStatementId,
    },
    /// A TCIR statement has no corresponding AMIR instruction.
    #[error("missing TCIR statement coverage for {statement:?}")]
    MissingTcirStatementCoverage {
        /// TCIR statement identity with no AMIR instruction coverage.
        statement: TcirStatementId,
    },
}

impl AmirVerificationError {
    fn from_shared(error: SharedProvenanceError) -> Self {
        match error {
            SharedProvenanceError::MissingComputationSource => Self::MissingModuleSource,
            SharedProvenanceError::StaleComputationSource => Self::StaleModuleSource,
            SharedProvenanceError::MissingComputationFingerprint => {
                Self::MissingModuleComputationFingerprint
            }
            SharedProvenanceError::StaleComputationFingerprint => {
                Self::StaleModuleComputationFingerprint
            }
            SharedProvenanceError::StaleTcirStatementReference { statement } => {
                Self::StaleTcirStatementReference {
                    block: None,
                    statement,
                }
            }
            SharedProvenanceError::StaleTcirStatementSource { statement } => {
                Self::StaleTcirStatementSource {
                    block: None,
                    statement,
                }
            }
            SharedProvenanceError::MissingInstructionSource
            | SharedProvenanceError::MissingInstructionTcirStatement => Self::MissingModuleSource,
        }
    }

    fn from_shared_for_section(section: AmirSectionKind, error: SharedProvenanceError) -> Self {
        match error {
            SharedProvenanceError::MissingComputationSource => {
                Self::MissingSectionSource { section }
            }
            SharedProvenanceError::StaleComputationSource => Self::StaleSectionSource { section },
            SharedProvenanceError::MissingComputationFingerprint => {
                Self::MissingSectionComputationFingerprint { section }
            }
            SharedProvenanceError::StaleComputationFingerprint => {
                Self::StaleSectionComputationFingerprint { section }
            }
            _ => Self::StaleSectionSource { section },
        }
    }

    fn from_shared_for_block(block: AmirBlockId, error: SharedProvenanceError) -> Self {
        match error {
            SharedProvenanceError::MissingComputationSource => Self::MissingBlockSource { block },
            SharedProvenanceError::StaleComputationSource => Self::StaleBlockSource { block },
            SharedProvenanceError::MissingComputationFingerprint => {
                Self::MissingBlockComputationFingerprint { block }
            }
            SharedProvenanceError::StaleComputationFingerprint => {
                Self::StaleBlockComputationFingerprint { block }
            }
            _ => Self::StaleBlockSource { block },
        }
    }

    fn from_shared_for_instruction(block: AmirBlockId, error: SharedProvenanceError) -> Self {
        match error {
            SharedProvenanceError::MissingInstructionSource => {
                Self::MissingInstructionSource { block }
            }
            SharedProvenanceError::MissingInstructionTcirStatement => {
                Self::MissingInstructionTcirStatement { block }
            }
            SharedProvenanceError::StaleTcirStatementReference { statement } => {
                Self::StaleTcirStatementReference {
                    block: Some(block),
                    statement,
                }
            }
            SharedProvenanceError::StaleTcirStatementSource { statement } => {
                Self::StaleTcirStatementSource {
                    block: Some(block),
                    statement,
                }
            }
            _ => Self::MissingInstructionSource { block },
        }
    }
}
