//! Canonical internal type-expression and normal-form substrate.
//!
//! This module owns the shared computation-grade carriers introduced by Phase 110
//! plus the Phase 112 normal-form carrier needed by the typechecker normalizer:
//! - shared computation-head identity namespace;
//! - canonical distinction between nominal applications, projections, and
//!   computation-head applications;
//! - projection rigidity metadata;
//! - sealed-domain constructor normal forms backed by `DomainConstructorId` and
//!   `SealedDomainId`, not ordinary runtime `ConstructorId` values.
//!
//! It defines only carriers. Lowering, normalization algorithms, definitional
//! equality, and diagnostics are owned by later Phase 112 tasks.

use crate::kind::Kind;
use crate::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, InterfaceIdentityId, ModuleIdentity,
    SealedDomainId, TypeDeclId,
};
use serde::{Deserialize, Serialize};

/// Identity for future explicit type-computation heads.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeComputationHeadId {
    pub module: ModuleIdentity,
    pub name: String,
}

impl TypeComputationHeadId {
    #[must_use]
    pub fn new(module: ModuleIdentity, name: impl Into<String>) -> Self {
        Self {
            module,
            name: name.into(),
        }
    }
}

/// Whether a projection is currently rigid or neutral/stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionRigidity {
    Rigid,
    Neutral,
}

/// Canonical internal type-expression substrate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalTypeExpr {
    Primitive(String),
    Var(String),
    NominalApp {
        origin: TypeDeclId,
        visible_name: String,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
    },
    Projection {
        interface: InterfaceIdentityId,
        member: AssociatedMemberIdentityId,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
        rigidity: ProjectionRigidity,
    },
    ComputationHeadApp {
        head: TypeComputationHeadId,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
    },
}

/// Reason metadata for normal forms that are stuck/blocked rather than reducible.
///
/// The normalizer and diagnostics may refine which reason applies in later tasks;
/// the carrier lives in `ash-core` so neutral computation applications and rigid
/// or neutral projections can transport that distinction structurally.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalFormBlockReason {
    AbstractScrutinee,
    NeutralScrutinee,
    RigidProjection,
    MissingAssociatedEvidence,
    Unsupported,
}

/// Shared normal-form carrier produced by the Phase 112 type-expression normalizer.
///
/// This is a structural carrier only: it does not perform normalization or encode
/// reduction rules. It deliberately keeps sealed-domain marker constructors in a
/// dedicated `DomainConstructorApp` variant backed by `DomainConstructorId` and
/// `SealedDomainId`, so they cannot be confused with ordinary runtime/ADT
/// constructor identities. Neutral computation heads and projections retain their
/// normalized argument spines for later definitional equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NormalTypeExpr {
    Primitive(String),
    Var(String),
    NominalApp {
        origin: TypeDeclId,
        visible_name: String,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
    },
    DomainConstructorApp {
        constructor: DomainConstructorId,
        domain: SealedDomainId,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
    },
    NeutralComputationApp {
        head: TypeComputationHeadId,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
        reason: Option<NormalFormBlockReason>,
    },
    Projection {
        interface: InterfaceIdentityId,
        member: AssociatedMemberIdentityId,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
        rigidity: ProjectionRigidity,
        reason: Option<NormalFormBlockReason>,
    },
}
