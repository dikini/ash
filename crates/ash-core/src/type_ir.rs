//! Canonical internal type-expression substrate for Phase 110.
//!
//! This module intentionally introduces only the minimal computation-grade carriers
//! needed by the first substrate slice:
//! - shared computation-head identity namespace
//! - canonical distinction between nominal applications, projections, and
//!   future computation-head applications
//! - projection rigidity metadata
//!
//! It does not yet define lowering, normalization, equality, or diagnostics.

use crate::kind::Kind;
use crate::semantic_summary::{
    AssociatedMemberIdentityId, InterfaceIdentityId, ModuleIdentity, TypeDeclId,
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
