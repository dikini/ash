//! Checked module-aware Core-to-CPS lowering.
//!
//! This boundary accepts only a finalizer-issued module interface, checked
//! import-resolution facts, and checker-owned finalized declaration bodies. It
//! resolves and snapshots imports before validating or lowering Core, then
//! delegates exclusively to the checked Core-to-CPS bridge. Import metadata is
//! not installed as callable authority and this module does not evaluate CPS.

use ash_core::core_ash::{CoreAtom, CoreExpr, CoreRow};
use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_typecheck::{
    CoreCheckedLoweringError, CoreTypeCheckEnv, type_check_and_lower_core_program,
};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};
use ash_core::cps::ContRef;
use ash_core::module_graph::ModuleKey;
use ash_core::module_interface::ModuleInterfaceDefiningIdentity;
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact, ResolvedModuleImport};
use ash_parser::CanonicalExpandedModuleGraph;
use std::fmt;
use thiserror::Error;

use crate::canonical_checked_module_finalizer::CanonicalCheckedModuleFinalization;
use crate::canonical_module_collection::{CanonicalDeclarationKind, CanonicalModuleCollection};
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

    /// The requested canonical module is absent from the checked closure.
    #[error("checked module {module} is absent from the finalization closure")]
    MissingCheckedModule { module: ModuleKey },

    /// The requested callable is absent from the checked module.
    #[error("checked callable {module}::{name:?} is absent from the finalization closure")]
    MissingCheckedDefinition { module: ModuleKey, name: String },

    /// The selected declaration is not a supported callable body in this slice.
    #[error("checked declaration {module}::{name:?} has unsupported kind {kind:?}")]
    UnsupportedDefinition {
        module: ModuleKey,
        name: String,
        kind: CanonicalDeclarationKind,
    },

    /// The selected declaration has no retained checker-owned surface body.
    #[error("checked callable {module}::{name:?} has no retained definition body")]
    MissingDefinitionBody { module: ModuleKey, name: String },

    /// Surface-to-Core lowering rejected the checker-owned body.
    #[error("checked callable {module}::{name:?} surface lowering failed: {reason}")]
    SurfaceLowering {
        module: ModuleKey,
        name: String,
        reason: String,
    },

    /// The finalizer and expanded graph disagree about source provenance.
    #[error("checked module {module} has mismatched source provenance")]
    ProvenanceMismatch { module: ModuleKey },
}

/// One checker-owned ordinary definition lowered to non-authorizing Core/CPS
/// transport artifacts.
///
/// The declaration name is retained alongside the existing module artifact
/// because a module may contain more than one ordinary function. The Core and
/// CPS carriers remain public data and do not authorize admission or execution.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredCheckedModuleDefinition {
    declaration_name: String,
    core: ModuleCoreArtifact,
    cps: ModuleCpsArtifact,
}

impl LoweredCheckedModuleDefinition {
    fn new(
        declaration_name: impl Into<String>,
        core: ModuleCoreArtifact,
        cps: ModuleCpsArtifact,
    ) -> Self {
        Self {
            declaration_name: declaration_name.into(),
            core,
            cps,
        }
    }

    /// Returns the checked declaration name owning these artifacts.
    #[must_use]
    pub fn declaration_name(&self) -> &str {
        &self.declaration_name
    }

    /// Returns the non-authorizing checked Core carrier.
    #[must_use]
    pub const fn core(&self) -> &ModuleCoreArtifact {
        &self.core
    }

    /// Returns the non-authorizing checked CPS carrier.
    #[must_use]
    pub const fn cps(&self) -> &ModuleCpsArtifact {
        &self.cps
    }
}

/// Lowers one checker-owned callable body without accepting caller-materialized Core.
///
/// This is the first TASK-2069 source-to-Core-to-CPS handoff. The body comes
/// from the private TASK-2073 finalization and is selected only after the
/// collected TASK-2075 closure contains the same module. The returned public
/// artifacts retain the exact expanded [`ash_core::module_graph::ModuleArtifact`]
/// and remain non-sealed, non-authorizing data carriers.
///
/// This initial slice supports ordinary function bodies with a direct-style
/// default Core environment. Import-environment wiring and reachable dependency
/// transport remain explicit follow-up slices.
///
/// # Errors
///
/// Returns an error when the module, callable, retained body, provenance, or
/// surface/Core lowering boundary is incomplete or inconsistent.
#[allow(clippy::result_large_err)]
pub fn lower_complete_checked_module_definition_bodies(
    finalized: &CanonicalCheckedModuleFinalization,
    collection: &CanonicalModuleCollection,
    expanded: &CanonicalExpandedModuleGraph,
    module_key: &ModuleKey,
    declaration_name: &str,
) -> Result<(ModuleCoreArtifact, ModuleCpsArtifact), ModuleCoreCpsLoweringError> {
    let finalized_module = finalized.module(module_key).ok_or_else(|| {
        ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        }
    })?;
    let finalized_declaration = finalized_module
        .private_declaration(declaration_name)
        .ok_or_else(|| ModuleCoreCpsLoweringError::MissingCheckedDefinition {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
        })?;
    if !matches!(
        finalized_declaration.kind(),
        CanonicalDeclarationKind::Function
    ) {
        return Err(ModuleCoreCpsLoweringError::UnsupportedDefinition {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            kind: finalized_declaration.kind(),
        });
    }

    collection.module(module_key).ok_or_else(|| {
        ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        }
    })?;
    let body = finalized_declaration.body().ok_or_else(|| {
        ModuleCoreCpsLoweringError::MissingDefinitionBody {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
        }
    })?;

    let module_artifact = expanded
        .parsed_graph()
        .module_unit(module_key)
        .ok_or_else(|| ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        })?
        .artifact()
        .clone();
    if finalized_module.origin() != module_artifact.origin() {
        return Err(ModuleCoreCpsLoweringError::ProvenanceMismatch {
            module: module_key.clone(),
        });
    }

    let core_expr = ash_parser::lower_expr(body).map_err(|error| {
        ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason: error.to_string(),
        }
    })?;
    let core_expr = surface_core_expr_to_checked_core(core_expr).map_err(|reason| {
        ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason,
        }
    })?;
    let validated_program = validate_core_program(RawCoreProgram::new(core_expr))?;
    let lowered_program = type_check_and_lower_core_program(
        validated_program,
        &CoreTypeCheckEnv::default(),
        CoreLoweringContext::new(ContRef::Label("halt".into()), CoreRow::default()),
    )?;
    let (checked_core_program, cps_program) = lowered_program.into_parts();
    let core_artifact = ModuleCoreArtifact::new(module_artifact, Vec::new(), checked_core_program);
    let cps_artifact = ModuleCpsArtifact::from_core_artifact(&core_artifact, cps_program);
    Ok((core_artifact, cps_artifact))
}

/// Lowers every supported ordinary function body in the finalized module
/// closure, publishing no partial vector when any body fails.
///
/// The finalized private view is the declaration/body authority. The paired
/// collection is used only to require the same canonical module closure, and
/// each returned item retains the exact module provenance through its Core and
/// CPS carriers. Handlers remain outside this bounded lowering slice and are
/// rejected rather than silently omitted.
///
/// # Errors
///
/// Returns the first checked-module, unsupported-definition, body, provenance,
/// surface-lowering, Core-validation, or Core/CPS-lowering error encountered.
#[allow(clippy::result_large_err)]
pub fn lower_complete_checked_module_definition_closure(
    finalized: &CanonicalCheckedModuleFinalization,
    collection: &CanonicalModuleCollection,
    expanded: &CanonicalExpandedModuleGraph,
) -> Result<Vec<LoweredCheckedModuleDefinition>, ModuleCoreCpsLoweringError> {
    for finalized_module in finalized.modules() {
        if collection.module(finalized_module.module_key()).is_none() {
            return Err(ModuleCoreCpsLoweringError::MissingCheckedModule {
                module: finalized_module.module_key().clone(),
            });
        }
    }

    let mut lowered = Vec::new();
    for finalized_module in finalized.modules() {
        for declaration in finalized_module.private_declarations() {
            match declaration.kind() {
                CanonicalDeclarationKind::Function => {
                    let (core, cps) = lower_complete_checked_module_definition_bodies(
                        finalized,
                        collection,
                        expanded,
                        finalized_module.module_key(),
                        declaration.name(),
                    )?;
                    lowered.push(LoweredCheckedModuleDefinition::new(
                        declaration.name(),
                        core,
                        cps,
                    ));
                }
                CanonicalDeclarationKind::Handler => {
                    return Err(ModuleCoreCpsLoweringError::UnsupportedDefinition {
                        module: finalized_module.module_key().clone(),
                        name: declaration.name().to_owned(),
                        kind: declaration.kind(),
                    });
                }
                _ => {}
            }
        }
    }

    Ok(lowered)
}

fn surface_core_expr_to_checked_core(expr: ash_core::Expr) -> Result<CoreExpr, String> {
    match expr {
        ash_core::Expr::Literal(ash_core::Value::Int(value)) => {
            Ok(CoreExpr::Atom(CoreAtom::LitInt(value)))
        }
        ash_core::Expr::Literal(ash_core::Value::String(value)) => {
            Ok(CoreExpr::Atom(CoreAtom::LitString(value)))
        }
        ash_core::Expr::Literal(ash_core::Value::Bool(value)) => {
            Ok(CoreExpr::Atom(CoreAtom::LitBool(value)))
        }
        other => Err(format!(
            "surface expression `{other:?}` has no checked Core projection in this slice"
        )),
    }
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
