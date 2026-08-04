//! Checked module-aware Core-to-CPS lowering.
//!
//! This boundary accepts only a finalizer-issued module interface, checked
//! import-resolution facts, and an already materialized raw Core program. It
//! resolves and snapshots imports before validating or lowering Core, then
//! delegates exclusively to the checked Core-to-CPS bridge. Import metadata is
//! not installed as callable authority and this module does not evaluate CPS.

use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_typecheck::{
    CoreCheckedLoweringError, CoreTypeCheckEnv, type_check_and_lower_core_program,
};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};
use ash_core::module_interface::ModuleInterfaceDefiningIdentity;
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact, ResolvedModuleImport};
use std::fmt;
use thiserror::Error;

use crate::interface_import_resolver::{InterfaceImportDiagnostic, InterfaceImportEnvironment};
use crate::module_interface_finalization::FinalizedModuleInterface;

/// An import fact expected by the module-aware lowering boundary.
///
/// The expected defining identity protects the immutable resolver result from
/// being reused against a different public binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedResolvedImport {
    local_name: String,
    defining_identity: ModuleInterfaceDefiningIdentity,
}

impl ExpectedResolvedImport {
    /// Creates an expected local alias and defining binding identity.
    #[must_use]
    pub fn new(
        local_name: impl Into<String>,
        defining_identity: ModuleInterfaceDefiningIdentity,
    ) -> Self {
        Self {
            local_name: local_name.into(),
            defining_identity,
        }
    }

    /// Returns the importing module's requested local alias.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the defining identity the resolver result must retain.
    #[must_use]
    pub fn defining_identity(&self) -> &ModuleInterfaceDefiningIdentity {
        &self.defining_identity
    }
}

/// Heap-backed defining identity retained by a stale-import diagnostic.
///
/// This wrapper keeps [`ModuleCoreCpsLoweringError`] compact while preserving
/// the complete identity in its diagnostics. It compares directly with an
/// unboxed [`ModuleInterfaceDefiningIdentity`] for ergonomic error handling.
#[derive(Clone, Eq)]
pub struct BoxedModuleInterfaceDefiningIdentity(Box<ModuleInterfaceDefiningIdentity>);

impl BoxedModuleInterfaceDefiningIdentity {
    /// Returns the retained defining identity.
    #[must_use]
    pub fn as_inner(&self) -> &ModuleInterfaceDefiningIdentity {
        &self.0
    }

    /// Consumes the diagnostic wrapper and returns its defining identity.
    #[must_use]
    pub fn into_inner(self) -> ModuleInterfaceDefiningIdentity {
        *self.0
    }
}

impl From<ModuleInterfaceDefiningIdentity> for BoxedModuleInterfaceDefiningIdentity {
    fn from(identity: ModuleInterfaceDefiningIdentity) -> Self {
        Self(Box::new(identity))
    }
}

impl fmt::Debug for BoxedModuleInterfaceDefiningIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl PartialEq for BoxedModuleInterfaceDefiningIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<ModuleInterfaceDefiningIdentity> for BoxedModuleInterfaceDefiningIdentity {
    fn eq(&self, other: &ModuleInterfaceDefiningIdentity) -> bool {
        self.0.as_ref() == other
    }
}

/// Failure while preparing imports or lowering a finalized module to Core/CPS.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ModuleCoreCpsLoweringError {
    /// Checked-interface import resolution could not satisfy a requested alias.
    #[error(transparent)]
    ImportResolution(#[from] InterfaceImportDiagnostic),

    /// A resolved alias no longer has its expected defining identity.
    #[error(
        "resolved import {local_name:?} has stale defining identity: expected {expected:?}, got {actual:?}"
    )]
    StaleResolvedImportIdentity {
        /// Importing module local alias.
        local_name: String,
        /// Identity supplied by the lowering caller.
        expected: BoxedModuleInterfaceDefiningIdentity,
        /// Identity from the checked resolver binding.
        actual: BoxedModuleInterfaceDefiningIdentity,
    },

    /// The materialized Core program failed its structural validation boundary.
    #[error(transparent)]
    CoreValidation(#[from] CoreValidationError),

    /// The checked Core-to-CPS bridge rejected or could not lower Core content.
    #[error(transparent)]
    CoreLowering(#[from] CoreCheckedLoweringError),
}

/// Resolves expected imports and lowers a finalized module's Core program to CPS.
///
/// Requested local aliases are resolved and checked against their expected
/// defining identities before raw Core validation or type checking starts.
/// Their cloned binding snapshots remain provenance metadata only: they do not
/// populate the Core type-checking environment or any runtime callable scope.
///
/// # Errors
///
/// Returns [`ModuleCoreCpsLoweringError::ImportResolution`] for unresolved or
/// ambiguous aliases, [`ModuleCoreCpsLoweringError::StaleResolvedImportIdentity`]
/// if a resolver result has an unexpected defining identity, and the existing
/// Core validation or checked Core-to-CPS lowering errors otherwise.
pub fn lower_finalized_module_to_core_cps(
    finalized_module: &FinalizedModuleInterface,
    import_environment: &InterfaceImportEnvironment,
    expected_imports: &[ExpectedResolvedImport],
    raw_core_program: RawCoreProgram,
    type_environment: CoreTypeCheckEnv,
    lowering_context: CoreLoweringContext,
) -> Result<(ModuleCoreArtifact, ModuleCpsArtifact), ModuleCoreCpsLoweringError> {
    let resolved_imports = snapshot_expected_imports(import_environment, expected_imports)?;

    let validated_program = validate_core_program(raw_core_program)?;
    let lowered_program =
        type_check_and_lower_core_program(validated_program, &type_environment, lowering_context)?;
    let (checked_core_program, cps_program) = lowered_program.into_parts();

    let core_artifact = ModuleCoreArtifact::new(
        finalized_module.module_artifact().clone(),
        resolved_imports,
        checked_core_program,
    );
    let cps_artifact = ModuleCpsArtifact::from_core_artifact(&core_artifact, cps_program);

    Ok((core_artifact, cps_artifact))
}

fn snapshot_expected_imports(
    import_environment: &InterfaceImportEnvironment,
    expected_imports: &[ExpectedResolvedImport],
) -> Result<Vec<ResolvedModuleImport>, ModuleCoreCpsLoweringError> {
    expected_imports
        .iter()
        .map(|expected_import| {
            let binding = import_environment.lookup(expected_import.local_name())?;
            let actual = binding.defining_identity().clone();
            if actual != *expected_import.defining_identity() {
                return Err(ModuleCoreCpsLoweringError::StaleResolvedImportIdentity {
                    local_name: expected_import.local_name().to_owned(),
                    expected: expected_import.defining_identity().clone().into(),
                    actual: actual.into(),
                });
            }

            Ok(ResolvedModuleImport::new(
                expected_import.local_name(),
                binding.clone(),
            ))
        })
        .collect()
}
