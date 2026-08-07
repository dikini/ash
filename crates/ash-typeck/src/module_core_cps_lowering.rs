//! Checked module-aware Core-to-CPS lowering.
//!
//! This boundary accepts only a finalizer-issued module interface, checked
//! import-resolution facts, and checker-owned finalized declaration bodies. It
//! resolves and snapshots imports before validating or lowering Core, then
//! delegates exclusively to the checked Core-to-CPS bridge. Import metadata is
//! not installed as callable authority and this module does not evaluate CPS.

use ash_core::Visibility as CoreVisibility;
use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreParam, CorePrimOp,
    CoreRow, CoreType, CoreValue,
};
use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_typecheck::{
    CoreCheckedLoweringError, CoreTypeCheckEnv, type_check_and_lower_core_program,
};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};
use ash_core::cps::ContRef;
use ash_core::module_graph::ModuleKey;
use ash_core::module_interface::ModuleInterfaceDefiningIdentity;
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceBindingKind, ModuleInterfaceDependency,
    ModuleInterfaceTypedIdentity, PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION, PublicModuleInterface,
};
use ash_core::module_lowering::{
    ModuleCoreArtifact, ModuleCpsArtifact, ModuleImportVisibility, ResolvedModuleImport,
};
use ash_parser::CanonicalExpandedModuleGraph;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;

use crate::canonical_checked_module_finalizer::CanonicalCheckedModuleFinalization;
use crate::canonical_module_collection::{
    CanonicalDeclarationKind, CanonicalModuleCollection, CanonicalNamespace,
};
use crate::canonical_parsed_import_resolver::{
    CanonicalParsedImportBinding, CanonicalParsedImportResult, CanonicalParsedNotationImport,
};
use crate::interface_import_resolver::{InterfaceImportDiagnostic, InterfaceImportEnvironment};
use crate::module_interface_finalization::FinalizedModuleInterface;

/// Continuation installed by the Engine's checked-CPS entry admission.
const ENTRY_ANSWER_CONTINUATION: &str = "__answer";

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

    /// A per-module Engine transport entry was not selected explicitly.
    #[error("checked module {module} has no selected lowering entry")]
    MissingSelectedEntry { module: ModuleKey },

    /// The entry-selection carrier names a module outside the checked closure.
    #[error("selected lowering entry names module {module} outside the checked closure")]
    UnexpectedSelectedEntry { module: ModuleKey },

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

    /// The lowering caller supplied a different checked import carrier than
    /// the one retained by finalization.
    #[error("checked module import transport does not match finalized facts")]
    ImportTransportMismatch,

    /// A checked import fact has no lossless Core transport mapping in this
    /// bounded lowering slice.
    #[error(
        "checked import {local_name:?} uses unsupported lowering fact {namespace:?}/{kind:?}: {reason}"
    )]
    UnsupportedImportFact {
        /// Local alias introduced by the import.
        local_name: String,
        /// Canonical namespace of the imported declaration.
        namespace: crate::canonical_module_collection::CanonicalNamespace,
        /// Canonical declaration kind of the imported declaration.
        kind: CanonicalDeclarationKind,
        /// Why this bounded carrier cannot represent the fact without loss.
        reason: String,
    },

    /// A finalized public export has no lossless checked interface mapping in
    /// this transport slice.
    #[error(
        "checked public export {module}::{name:?} uses unsupported interface namespace {namespace:?}: {reason}"
    )]
    UnsupportedInterfaceExport {
        /// Module publishing the export.
        module: ModuleKey,
        /// Visible exported name.
        name: String,
        /// Canonical namespace of the exported declaration.
        namespace: crate::canonical_module_collection::CanonicalNamespace,
        /// Why the interface carrier cannot represent the export.
        reason: String,
    },

    /// The checked public-interface constructor rejected a projected closure.
    #[error("checked public interface construction failed for module {module}: {reason}")]
    InterfaceConstruction {
        /// Module whose interface failed construction.
        module: ModuleKey,
        /// Core interface validation detail.
        reason: String,
    },
}

/// One checker-owned definition or neutral metadata module lowered to
/// non-authorizing Core/CPS transport artifacts.
///
/// The declaration name is retained alongside the existing module artifact
/// because a module may contain more than one ordinary function. The Core and
/// CPS carriers remain public data and do not authorize admission or execution.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredCheckedModuleDefinition {
    declaration_name: String,
    parameter_names: Vec<String>,
    core: ModuleCoreArtifact,
    cps: ModuleCpsArtifact,
    callable_entry: bool,
}

/// Projects the canonical finalizer's public exports into the checked Core
/// public-interface carrier used by Engine transport.
///
/// The finalized export map and parser-owned artifacts are the only semantic
/// inputs.  Parsed imports contribute dependency identities, while source
/// paths remain origin diagnostics only.  This function publishes no partial
/// interface vector: one unsupported export or malformed artifact rejects the
/// complete projection.
///
/// The interface carrier intentionally exposes only namespaces represented by
/// [`ModuleInterfaceBindingKind`].  Unrepresented metadata is rejected rather
/// than collapsed into a value or callable binding.
///
/// # Errors
///
/// Returns [`ModuleCoreCpsLoweringError::UnsupportedInterfaceExport`] when a
/// public namespace cannot be represented without loss, or
/// [`ModuleCoreCpsLoweringError::InterfaceConstruction`] when Core interface
/// validation rejects the projected artifact.
#[allow(clippy::result_large_err)]
pub fn build_checked_public_module_interface_closure(
    finalized: &CanonicalCheckedModuleFinalization,
    expanded: &CanonicalExpandedModuleGraph,
    imports: &CanonicalParsedImportResult,
) -> Result<Vec<PublicModuleInterface>, ModuleCoreCpsLoweringError> {
    if imports != finalized.imports() {
        return Err(ModuleCoreCpsLoweringError::ImportTransportMismatch);
    }

    let mut interfaces = Vec::new();
    for finalized_module in finalized.modules() {
        let module_key = finalized_module.module_key().clone();
        let artifact = expanded
            .parsed_graph()
            .module_unit(&module_key)
            .ok_or_else(|| ModuleCoreCpsLoweringError::MissingCheckedModule {
                module: module_key.clone(),
            })?
            .artifact()
            .clone();
        let bindings = finalized_module
            .public_exports()
            .map(|export| interface_binding_for_export(&module_key, export, expanded))
            .collect::<Result<Vec<_>, _>>()?;
        let typed_identities = finalized_module
            .public_exports()
            .filter_map(|export| {
                let is_constructor = matches!(
                    export.declaration().fact(),
                    crate::canonical_checked_module_finalizer::CanonicalCheckedDeclarationFact::Constructor {
                        ..
                    }
                );
                let kind = match export.declaration().namespace() {
                    crate::canonical_module_collection::CanonicalNamespace::TypeDomain
                    | crate::canonical_module_collection::CanonicalNamespace::Interface
                    | crate::canonical_module_collection::CanonicalNamespace::RowName => {
                        interface_binding_kind(&module_key, export).ok()?
                    }
                    crate::canonical_module_collection::CanonicalNamespace::ValueCallable
                        if is_constructor => interface_binding_kind(&module_key, export).ok()?,
                    _ => return None,
                };
                kind.requires_typed_identity().then(|| {
                    ModuleInterfaceTypedIdentity::new(
                        export.defining_identity().module_key().clone(),
                        export.declaration().name(),
                        kind,
                    )
                })
            })
            .collect();
        let dependencies = dependency_snapshot_for_module(imports, &module_key);
        let interface = PublicModuleInterface::with_dependencies_and_typed_identities(
            artifact,
            bindings,
            dependencies,
            None,
            typed_identities,
        )
        .map_err(|error| ModuleCoreCpsLoweringError::InterfaceConstruction {
            module: module_key,
            reason: error.to_string(),
        })?;
        interfaces.push(interface);
    }

    Ok(interfaces)
}

fn interface_binding_for_export(
    module_key: &ModuleKey,
    export: &crate::canonical_checked_module_finalizer::CanonicalCheckedExport,
    expanded: &CanonicalExpandedModuleGraph,
) -> Result<ModuleInterfaceBinding, ModuleCoreCpsLoweringError> {
    let visibility = CoreVisibility::Public;
    if export.declaration().namespace()
        == crate::canonical_module_collection::CanonicalNamespace::StructuralModule
    {
        let crate::canonical_checked_module_finalizer::CanonicalCheckedDeclarationFact::StructuralModule { module: child } =
            export.declaration().fact()
        else {
            return Err(ModuleCoreCpsLoweringError::UnsupportedInterfaceExport {
                module: module_key.clone(),
                name: export.local_name().to_owned(),
                namespace: export.declaration().namespace(),
                reason: "structural export has no checked child-module identity".to_owned(),
            });
        };
        let origin = expanded
            .parsed_graph()
            .module_unit(child)
            .ok_or_else(|| ModuleCoreCpsLoweringError::MissingCheckedModule {
                module: child.clone(),
            })?
            .artifact()
            .origin()
            .clone();
        return Ok(ModuleInterfaceBinding::child(
            export.local_name(),
            child.clone(),
            visibility,
            origin,
        ));
    }

    let origin = export.declaration().origin().clone();
    let kind = interface_binding_kind(module_key, export)?;
    Ok(ModuleInterfaceBinding::declaration(
        export.local_name(),
        export.defining_identity().module_key().clone(),
        export.declaration().name(),
        kind,
        visibility,
        origin,
    ))
}

fn interface_binding_kind(
    module_key: &ModuleKey,
    export: &crate::canonical_checked_module_finalizer::CanonicalCheckedExport,
) -> Result<ModuleInterfaceBindingKind, ModuleCoreCpsLoweringError> {
    use crate::canonical_module_collection::CanonicalNamespace;

    let namespace = export.declaration().namespace();
    if matches!(
        export.declaration().fact(),
        crate::canonical_checked_module_finalizer::CanonicalCheckedDeclarationFact::Constructor { .. }
    ) {
        return Ok(ModuleInterfaceBindingKind::Constructor);
    }
    let kind = match namespace {
        CanonicalNamespace::Macro => ModuleInterfaceBindingKind::SyntaxMacro,
        CanonicalNamespace::Notation => ModuleInterfaceBindingKind::SyntaxNotation,
        CanonicalNamespace::TypeDomain => ModuleInterfaceBindingKind::Type,
        CanonicalNamespace::TypeComputation => ModuleInterfaceBindingKind::TypeFunction,
        CanonicalNamespace::Proposition => ModuleInterfaceBindingKind::Proposition,
        CanonicalNamespace::PromotedKind => ModuleInterfaceBindingKind::PromotedKind,
        CanonicalNamespace::RowName => ModuleInterfaceBindingKind::EffectRow,
        CanonicalNamespace::Interface => ModuleInterfaceBindingKind::Interface,
        CanonicalNamespace::ImplementationRegistry => ModuleInterfaceBindingKind::Implementation,
        CanonicalNamespace::ValueCallable => ModuleInterfaceBindingKind::Callable,
        CanonicalNamespace::Evidence => ModuleInterfaceBindingKind::Evidence,
        unsupported => {
            return Err(ModuleCoreCpsLoweringError::UnsupportedInterfaceExport {
                module: module_key.clone(),
                name: export.local_name().to_owned(),
                namespace: unsupported,
                reason: "the checked public-interface carrier has no lossless binding kind"
                    .to_owned(),
            });
        }
    };
    Ok(kind)
}

impl LoweredCheckedModuleDefinition {
    fn new(
        declaration_name: impl Into<String>,
        parameter_names: impl IntoIterator<Item = impl Into<String>>,
        core: ModuleCoreArtifact,
        cps: ModuleCpsArtifact,
    ) -> Self {
        Self {
            declaration_name: declaration_name.into(),
            parameter_names: parameter_names.into_iter().map(Into::into).collect(),
            core,
            cps,
            callable_entry: true,
        }
    }

    fn metadata_only(core: ModuleCoreArtifact, cps: ModuleCpsArtifact) -> Self {
        Self {
            declaration_name: String::new(),
            parameter_names: Vec::new(),
            core,
            cps,
            callable_entry: false,
        }
    }

    /// Returns whether this carrier represents a checker-lowered callable.
    ///
    /// A false result denotes a neutral metadata-only module carrier. It has
    /// no selected entry or local callable authority and exists only so a
    /// structural metadata module can remain in the canonical closure.
    #[must_use]
    pub const fn is_callable_entry(&self) -> bool {
        self.callable_entry
    }

    /// Returns the checked declaration name owning these artifacts.
    #[must_use]
    pub fn declaration_name(&self) -> &str {
        &self.declaration_name
    }

    /// Returns checker-retained parameter names for the selected callable.
    ///
    /// This is transport metadata for the later non-authorizing linked-call
    /// handoff. It does not install a runtime callable or grant authority.
    #[must_use]
    pub fn parameter_names(&self) -> &[String] {
        &self.parameter_names
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
/// This single-definition API supports ordinary function bodies with the
/// finalized declaration's parameter environment. Callers lowering a complete
/// module closure should use [`lower_complete_checked_module_definition_closure`]
/// or [`lower_complete_checked_module_entry_closure`], which add checked local
/// and imported callable signatures plus dependency snapshots.
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
    if finalized_declaration
        .identity()
        .canonical_parent()
        .is_some()
    {
        return Err(ModuleCoreCpsLoweringError::UnsupportedDefinition {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            kind: finalized_declaration.kind(),
        });
    }
    let type_environment = core_type_environment_for_declaration(finalized_declaration)?;
    lower_complete_checked_module_definition_body_with_environment(
        finalized,
        collection,
        expanded,
        module_key,
        declaration_name,
        finalized_declaration,
        &type_environment,
    )
}

#[allow(clippy::result_large_err)]
fn lower_complete_checked_module_definition_body_with_environment(
    finalized: &CanonicalCheckedModuleFinalization,
    collection: &CanonicalModuleCollection,
    expanded: &CanonicalExpandedModuleGraph,
    module_key: &ModuleKey,
    declaration_name: &str,
    finalized_declaration: &crate::canonical_checked_module_finalizer::CanonicalCheckedDeclaration,
    type_environment: &CoreTypeCheckEnv,
) -> Result<(ModuleCoreArtifact, ModuleCpsArtifact), ModuleCoreCpsLoweringError> {
    let finalized_module = finalized.module(module_key).ok_or_else(|| {
        ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        }
    })?;
    if !matches!(
        finalized_declaration.kind(),
        CanonicalDeclarationKind::Function | CanonicalDeclarationKind::Handler
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

    if finalized_declaration.kind() == CanonicalDeclarationKind::Handler {
        return lower_checked_handler_definition_body(
            module_artifact,
            module_key,
            declaration_name,
            finalized_declaration,
            type_environment,
        );
    }

    let core_expr = ash_parser::lower_expr(body).map_err(|error| {
        ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason: error.to_string(),
        }
    })?;
    let core_expr =
        surface_core_expr_to_checked_core(core_expr, type_environment).map_err(|reason| {
            ModuleCoreCpsLoweringError::SurfaceLowering {
                module: module_key.clone(),
                name: declaration_name.to_owned(),
                reason,
            }
        })?;
    let validated_program = validate_core_program(RawCoreProgram::new(core_expr))?;
    let lowered_program = type_check_and_lower_core_program(
        validated_program,
        type_environment,
        CoreLoweringContext::new(
            ContRef::Label(ENTRY_ANSWER_CONTINUATION.to_owned()),
            CoreRow::default(),
        ),
    )?;
    let (checked_core_program, cps_program) = lowered_program.into_parts();
    let core_artifact = ModuleCoreArtifact::new(module_artifact, Vec::new(), checked_core_program);
    let cps_artifact = ModuleCpsArtifact::from_core_artifact(&core_artifact, cps_program);
    Ok((core_artifact, cps_artifact))
}

/// Lower the bounded identity/direct-resume source-handler declaration slice.
///
/// Handler clauses arrive with checker-retained operation identities and
/// continuation facts.  This bridge deliberately accepts only the shapes the
/// Core carrier can represent without inventing a provider frame: one checked
/// operation clause, an identity `done` clause, and either a payload identity,
/// direct resume, or the same operation raised from the payload.  Richer
/// handler rows remain an explicit lowering rejection.
#[allow(clippy::result_large_err)]
fn lower_checked_handler_definition_body(
    module_artifact: ash_core::module_graph::ModuleArtifact,
    module_key: &ModuleKey,
    declaration_name: &str,
    declaration: &crate::canonical_checked_module_finalizer::CanonicalCheckedDeclaration,
    type_environment: &CoreTypeCheckEnv,
) -> Result<(ModuleCoreArtifact, ModuleCpsArtifact), ModuleCoreCpsLoweringError> {
    let Some(crate::CheckedHandlerDeclaration {
        clauses,
        answer_type,
        ..
    }) = declaration.handler_fact()
    else {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason: "checked handler has no retained typed clause fact".to_owned(),
        });
    };
    if clauses.len() != 1 {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason: "checked handler lowering requires exactly one operation clause".to_owned(),
        });
    }
    let Some(ash_parser::surface::Expr::On {
        computation,
        clauses: source_clauses,
        ..
    }) = declaration.body()
    else {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason: "checked handler lowering requires a canonical on body".to_owned(),
        });
    };
    let computation_name = match computation.as_ref() {
        ash_parser::surface::Expr::Variable { name, .. } => name.to_string(),
        _ => {
            return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
                module: module_key.clone(),
                name: declaration_name.to_owned(),
                reason: "checked handler lowering requires a variable computation binder"
                    .to_owned(),
            });
        }
    };
    let checked_clause = &clauses[0];
    let mut done_identity = false;
    let mut operation_source = None;
    for source_clause in source_clauses {
        match source_clause {
            ash_parser::surface::HandlerClause::Done { binding, body, .. } => {
                done_identity = matches!(
                    body.as_ref(),
                    ash_parser::surface::Expr::Variable { name, .. } if name == binding
                );
            }
            ash_parser::surface::HandlerClause::Operation {
                impl_type,
                operation,
                pattern,
                resume,
                body,
                ..
            } if impl_type.as_ref() == checked_clause.operation.impl_type
                && operation.as_ref() == checked_clause.operation.operation =>
            {
                operation_source = Some((pattern, resume, body));
            }
            ash_parser::surface::HandlerClause::Operation { .. } => {}
        }
    }
    if !done_identity {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason: "checked handler lowering requires an identity done clause".to_owned(),
        });
    }
    let Some((
        ash_parser::surface::Pattern::Variable {
            name: payload_name, ..
        },
        resume_name,
        clause_body,
    )) = operation_source
    else {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
            reason: "checked handler lowering requires a variable operation binder".to_owned(),
        });
    };

    let clause_core_body = match clause_body.as_ref() {
        ash_parser::surface::Expr::Variable { name, .. } if name == payload_name => {
            CoreExpr::Atom(CoreAtom::Var(payload_name.to_string()))
        }
        ash_parser::surface::Expr::Call {
            func,
            module: None,
            args,
            ..
        } if func == resume_name && args.len() == 1 => CoreExpr::Jump {
            cont: CoreContRef::Var(resume_name.to_string()),
            arg: crate::surface_handler_resume_argument_to_core_atom(&args[0]).map_err(
                |error| ModuleCoreCpsLoweringError::SurfaceLowering {
                    module: module_key.clone(),
                    name: declaration_name.to_owned(),
                    reason: error.to_string(),
                },
            )?,
        },
        ash_parser::surface::Expr::Call {
            func,
            module: Some(impl_type),
            args,
            ..
        } if checked_clause.local_effect.as_ref().is_some_and(|effect| {
            impl_type.as_ref() == effect.impl_type
                && func.as_ref() == effect.operation
                && matches!(
                    args.as_slice(),
                    [ash_parser::surface::Expr::Variable { name, .. }]
                        if name == payload_name
                )
        }) =>
        {
            CoreExpr::Raise {
                op: crate::declared_operation_to_core_effect_op(
                    checked_clause
                        .local_effect
                        .as_ref()
                        .expect("guarded by local effect presence"),
                ),
                args: vec![CoreAtom::Var(payload_name.to_string())],
            }
        }
        _ => {
            return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
                module: module_key.clone(),
                name: declaration_name.to_owned(),
                reason: "checked handler lowering requires identity, direct resume, or local raise clause body"
                    .to_owned(),
            });
        }
    };

    let operation = crate::declared_operation_to_core_effect_op(&checked_clause.operation);
    let mut handler_environment = type_environment.clone();
    handler_environment
        .operations_mut()
        .insert(operation.clone());
    let core_expr = CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: operation.clone(),
            params: vec![CoreParam {
                name: payload_name.to_string(),
                ty: crate::type_to_core_type(&checked_clause.payload_type),
            }],
            resume: CoreParam {
                name: resume_name.to_string(),
                ty: CoreType::Cont {
                    input: Box::new(crate::type_to_core_type(
                        &checked_clause.operation.result_type,
                    )),
                    answer: Box::new(crate::type_to_core_type(answer_type)),
                    row: CoreRow::default(),
                    multiplicity: match checked_clause.continuation_multiplicity {
                        crate::ContinuationMultiplicity::MultiShotPure => {
                            CoreMultiplicity::MultiShotPure
                        }
                        crate::ContinuationMultiplicity::Affine => CoreMultiplicity::Affine,
                    },
                },
            },
            body: Box::new(clause_core_body),
            row: checked_clause
                .local_effect
                .as_ref()
                .map_or_else(CoreRow::default, crate::operation_effect_row),
        },
        body: Box::new(CoreExpr::Call {
            func: CoreAtom::Var(computation_name),
            args: Vec::new(),
        }),
    };
    let validated_program = validate_core_program(RawCoreProgram::new(core_expr))?;
    let lowered_program = type_check_and_lower_core_program(
        validated_program,
        &handler_environment,
        CoreLoweringContext::new(
            ContRef::Label(ENTRY_ANSWER_CONTINUATION.to_owned()),
            CoreRow::default(),
        ),
    )?;
    let (checked_core_program, cps_program) = lowered_program.into_parts();
    let core_artifact = ModuleCoreArtifact::new(module_artifact, Vec::new(), checked_core_program);
    let cps_artifact = ModuleCpsArtifact::from_core_artifact(&core_artifact, cps_program);
    Ok((core_artifact, cps_artifact))
}

fn core_type_environment_for_declaration(
    declaration: &crate::canonical_checked_module_finalizer::CanonicalCheckedDeclaration,
) -> Result<CoreTypeCheckEnv, ModuleCoreCpsLoweringError> {
    let mut environment = CoreTypeCheckEnv::default();
    let Some(parameter_names) = declaration.parameter_names() else {
        return Ok(environment);
    };
    let Some(signature) = declaration.signature() else {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: declaration.identity().module_key().clone(),
            name: declaration.name().to_owned(),
            reason: "finalized callable has parameter names but no checked signature".to_owned(),
        });
    };
    let parameter_types = match signature {
        crate::Type::Fn(parameter_types, _) | crate::Type::Fun(parameter_types, _, _) => {
            parameter_types
        }
        other => {
            return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
                module: declaration.identity().module_key().clone(),
                name: declaration.name().to_owned(),
                reason: format!("finalized callable has non-function signature {other:?}"),
            });
        }
    };
    if parameter_names.len() != parameter_types.len() {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: declaration.identity().module_key().clone(),
            name: declaration.name().to_owned(),
            reason: format!(
                "finalized parameter/signature arity mismatch: {} names, {} types",
                parameter_names.len(),
                parameter_types.len()
            ),
        });
    }
    for (name, parameter_type) in parameter_names.iter().zip(parameter_types) {
        environment
            .values_mut()
            .insert(name.to_string(), core_type_for_checked_type(parameter_type));
    }
    Ok(environment)
}

fn core_type_environment_for_module(
    finalized: &CanonicalCheckedModuleFinalization,
    module_key: &ModuleKey,
    declaration: &crate::canonical_checked_module_finalizer::CanonicalCheckedDeclaration,
) -> Result<CoreTypeCheckEnv, ModuleCoreCpsLoweringError> {
    let mut environment = core_type_environment_for_declaration(declaration)?;
    let module = finalized.module(module_key).ok_or_else(|| {
        ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        }
    })?;

    for candidate in module.private_declarations() {
        if matches!(
            candidate.kind(),
            CanonicalDeclarationKind::Function
                | CanonicalDeclarationKind::Handler
                | CanonicalDeclarationKind::BuiltinFn
        ) && let Some(signature) = candidate.signature()
        {
            insert_core_callable_signature(
                &mut environment,
                candidate.name(),
                signature,
                module_key,
            )?;
        }
    }

    for (importing_module, _, binding) in finalized.imports().bindings() {
        if importing_module != module_key {
            continue;
        }
        let Some(target_module) = finalized.module(binding.defining_identity().module_key()) else {
            return Err(ModuleCoreCpsLoweringError::MissingCheckedModule {
                module: binding.defining_identity().module_key().clone(),
            });
        };
        let signature = target_module
            .public_export_in_namespace(
                binding.lookup_key().namespace(),
                binding.lookup_key().visible_local_key(),
            )
            .and_then(|export| export.declaration().signature())
            .or_else(|| {
                target_module
                    .private_declarations()
                    .find(|candidate| candidate.identity() == binding.defining_identity())
                    .and_then(|candidate| candidate.signature())
            });
        let Some(signature) = signature else {
            continue;
        };
        insert_core_callable_signature(
            &mut environment,
            binding.local_name(),
            signature,
            module_key,
        )?;
    }

    for import in
        structural_module_alias_callable_imports(finalized, finalized.imports(), module_key)
    {
        insert_core_callable_signature(
            &mut environment,
            &import.local_name,
            &import.signature,
            module_key,
        )?;
    }

    Ok(environment)
}

fn insert_core_callable_signature(
    environment: &mut CoreTypeCheckEnv,
    name: &str,
    signature: &crate::Type,
    module_key: &ModuleKey,
) -> Result<(), ModuleCoreCpsLoweringError> {
    let core_type = core_type_for_checked_type(signature);
    if !matches!(core_type, CoreType::Function { .. }) {
        return Err(ModuleCoreCpsLoweringError::SurfaceLowering {
            module: module_key.clone(),
            name: name.to_owned(),
            reason: format!("callable signature has no Core function shape: {signature:?}"),
        });
    }
    environment.values_mut().insert(name.to_owned(), core_type);
    Ok(())
}

fn core_type_for_checked_type(ty: &crate::Type) -> CoreType {
    match ty {
        crate::Type::Fn(parameters, result) | crate::Type::Fun(parameters, result, _) => {
            CoreType::Function {
                params: parameters.iter().map(core_type_for_checked_type).collect(),
                result: Box::new(core_type_for_checked_type(result)),
                row: CoreRow::default(),
            }
        }
        crate::Type::Record(fields) => CoreType::Record(
            fields
                .iter()
                .map(|(name, field_type)| {
                    (name.to_string(), core_type_for_checked_type(field_type))
                })
                .collect(),
        ),
        crate::Type::List(item) => CoreType::App {
            name: "List".to_owned(),
            args: vec![core_type_for_checked_type(item)],
        },
        crate::Type::Constructor { name, args, .. } => CoreType::App {
            name: name.to_string(),
            args: args.iter().map(core_type_for_checked_type).collect(),
        },
        other => crate::type_to_core_type(other),
    }
}

/// Lowers every supported ordinary function body in the finalized module
/// closure, publishing no partial vector when any body fails.
///
/// The finalized private view is the declaration/body authority. The paired
/// collection is used only to require the same canonical module closure, and
/// each returned item retains the exact module provenance through its Core and
/// CPS carriers. Only the bounded handler forms accepted by the checked handler
/// lowering bridge are admitted; unsupported handler shapes reject rather than
/// being silently omitted.
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
    imports: &CanonicalParsedImportResult,
) -> Result<Vec<LoweredCheckedModuleDefinition>, ModuleCoreCpsLoweringError> {
    if imports != finalized.imports() {
        return Err(ModuleCoreCpsLoweringError::ImportTransportMismatch);
    }

    for finalized_module in finalized.modules() {
        if collection.module(finalized_module.module_key()).is_none() {
            return Err(ModuleCoreCpsLoweringError::MissingCheckedModule {
                module: finalized_module.module_key().clone(),
            });
        }
    }

    let mut lowered = Vec::new();
    for finalized_module in finalized.modules() {
        let resolved_imports =
            resolved_imports_for_module(finalized, imports, finalized_module.module_key())?;
        let dependencies = dependency_snapshot_for_module(imports, finalized_module.module_key());
        for declaration in finalized_module.private_declarations() {
            // Constructor declarations are checked interface facts, not
            // source bodies.  They must remain available to import transport
            // without being mistaken for ordinary definitions that require a
            // second Core/CPS body artifact.
            // Parent-scoped interface and implementation callables are also
            // checked private facts, not standalone module entries. Their
            // bodies remain available on the finalized declaration for a
            // future parent-aware lowering boundary.
            if matches!(
                declaration.kind(),
                CanonicalDeclarationKind::Function | CanonicalDeclarationKind::Handler
            ) && declaration.identity().canonical_parent().is_some()
            {
                continue;
            }
            if declaration.kind() == CanonicalDeclarationKind::Function
                && declaration.body().is_none()
            {
                continue;
            }
            match declaration.kind() {
                CanonicalDeclarationKind::Function => {
                    let (core, cps) = lower_checked_definition_with_imports(
                        finalized,
                        collection,
                        expanded,
                        finalized_module.module_key(),
                        declaration.name(),
                        &resolved_imports,
                        &dependencies,
                    )?;
                    lowered.push(LoweredCheckedModuleDefinition::new(
                        declaration.name(),
                        declaration
                            .parameter_names()
                            .into_iter()
                            .flatten()
                            .map(|name| name.as_ref()),
                        core,
                        cps,
                    ));
                }
                CanonicalDeclarationKind::Handler => {
                    let (core, cps) = lower_checked_definition_with_imports(
                        finalized,
                        collection,
                        expanded,
                        finalized_module.module_key(),
                        declaration.name(),
                        &resolved_imports,
                        &dependencies,
                    )?;
                    lowered.push(LoweredCheckedModuleDefinition::new(
                        declaration.name(),
                        declaration
                            .parameter_names()
                            .into_iter()
                            .flatten()
                            .map(|name| name.as_ref()),
                        core,
                        cps,
                    ));
                }
                _ => {}
            }
        }
    }

    Ok(lowered)
}

/// Builds a neutral Core/CPS carrier for a finalized module with no standalone
/// callable body.
///
/// Structural modules that contain only checked metadata still belong to the
/// canonical module closure. They must not be dropped, because doing so would
/// make the structural graph incomplete, but they also must not acquire a
/// selected callable entry. The neutral literal is checked and lowered solely
/// to keep the non-authorizing transport shape total; Engine linking never
/// selects it as a callable.
///
/// # Errors
///
/// Returns the same checked-module, Core-validation, or Core/CPS-lowering
/// errors as the ordinary module lowering boundary.
#[allow(clippy::result_large_err)]
pub fn lower_checked_metadata_only_module(
    finalized: &CanonicalCheckedModuleFinalization,
    collection: &CanonicalModuleCollection,
    expanded: &CanonicalExpandedModuleGraph,
    imports: &CanonicalParsedImportResult,
    module_key: &ModuleKey,
) -> Result<LoweredCheckedModuleDefinition, ModuleCoreCpsLoweringError> {
    if imports != finalized.imports() {
        return Err(ModuleCoreCpsLoweringError::ImportTransportMismatch);
    }
    if collection.module(module_key).is_none() || finalized.module(module_key).is_none() {
        return Err(ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        });
    }
    let module_artifact = expanded
        .parsed_graph()
        .module_unit(module_key)
        .ok_or_else(|| ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        })?
        .artifact()
        .clone();
    let resolved_imports = resolved_imports_for_module(finalized, imports, module_key)?;
    let dependencies = dependency_snapshot_for_module(imports, module_key);
    let validated =
        validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(0))))?;
    let lowered = type_check_and_lower_core_program(
        validated,
        &CoreTypeCheckEnv::default(),
        CoreLoweringContext::new(
            ContRef::Label(ENTRY_ANSWER_CONTINUATION.to_owned()),
            CoreRow::default(),
        ),
    )?;
    let (checked_core_program, cps_program) = lowered.into_parts();
    let core = ModuleCoreArtifact::new_with_interface_metadata(
        module_artifact,
        resolved_imports,
        PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        dependencies,
        checked_core_program,
    );
    let cps = ModuleCpsArtifact::from_core_artifact(&core, cps_program);
    Ok(LoweredCheckedModuleDefinition::metadata_only(core, cps))
}

/// Lowers one explicitly selected ordinary entry for every finalized module.
///
/// [`lower_complete_checked_module_definition_closure`] deliberately retains
/// one result per declaration so typechecking can inspect every body. Engine
/// transport, however, has one canonical artifact slot per [`ModuleKey`].
/// This entry-oriented handoff makes that boundary explicit: callers provide
/// exactly one declaration name for every finalized module, and the returned
/// vector therefore contains at most one artifact per canonical module.
///
/// The selection is metadata about which already-checked body is exposed as a
/// module entry; it does not grant callable, handler, or runtime authority.
/// Missing selections and non-function selections reject before a partial
/// transport vector is published.
///
/// # Errors
///
/// Returns [`ModuleCoreCpsLoweringError::MissingSelectedEntry`] when a module
/// has no selected declaration, or any of the same checked lowering errors as
/// [`lower_complete_checked_module_definition_closure`].
#[allow(clippy::result_large_err)]
pub fn lower_complete_checked_module_entry_closure(
    finalized: &CanonicalCheckedModuleFinalization,
    collection: &CanonicalModuleCollection,
    expanded: &CanonicalExpandedModuleGraph,
    imports: &CanonicalParsedImportResult,
    selected_entries: &BTreeMap<ModuleKey, String>,
) -> Result<Vec<LoweredCheckedModuleDefinition>, ModuleCoreCpsLoweringError> {
    if imports != finalized.imports() {
        return Err(ModuleCoreCpsLoweringError::ImportTransportMismatch);
    }

    if let Some((module, _)) = selected_entries.iter().find(|(module, _)| {
        !finalized
            .modules()
            .any(|candidate| candidate.module_key() == *module)
    }) {
        return Err(ModuleCoreCpsLoweringError::UnexpectedSelectedEntry {
            module: module.clone(),
        });
    }

    let mut lowered = Vec::new();
    for finalized_module in finalized.modules() {
        let module_key = finalized_module.module_key();
        let declaration_name = selected_entries.get(module_key).ok_or_else(|| {
            ModuleCoreCpsLoweringError::MissingSelectedEntry {
                module: module_key.clone(),
            }
        })?;
        let declaration = finalized_module
            .private_declarations()
            .find(|declaration| {
                declaration.name() == declaration_name
                    && declaration.identity().canonical_parent().is_none()
            })
            .or_else(|| {
                finalized_module
                    .private_declarations()
                    .find(|declaration| declaration.name() == declaration_name)
            })
            .ok_or_else(|| ModuleCoreCpsLoweringError::MissingCheckedDefinition {
                module: module_key.clone(),
                name: declaration_name.clone(),
            })?;
        if declaration.identity().canonical_parent().is_some() {
            return Err(ModuleCoreCpsLoweringError::UnsupportedDefinition {
                module: module_key.clone(),
                name: declaration_name.clone(),
                kind: declaration.kind(),
            });
        }
        let resolved_imports = resolved_imports_for_module(finalized, imports, module_key)?;
        let dependencies = dependency_snapshot_for_module(imports, module_key);
        let (core, cps) = lower_checked_definition_with_imports(
            finalized,
            collection,
            expanded,
            module_key,
            declaration_name,
            &resolved_imports,
            &dependencies,
        )?;
        lowered.push(LoweredCheckedModuleDefinition::new(
            declaration_name,
            finalized_module
                .private_declaration(declaration_name)
                .and_then(|declaration| declaration.parameter_names())
                .into_iter()
                .flatten()
                .map(|name| name.as_ref()),
            core,
            cps,
        ));
    }

    Ok(lowered)
}

/// Lowers the complete non-authorizing route closure for one selected root.
///
/// Ordinary callable declarations are retained as local callable entries and
/// the caller's selected name identifies the entry exposed by each callable
/// module. A structural module with no standalone ordinary function receives
/// a neutral metadata-only carrier instead. This keeps structural children
/// and handler-only modules in the canonical closure without inventing a
/// callable selection for them.
///
/// The route map must contain exactly one entry for every finalized module.
/// An empty selection is valid only for a non-root module; all checker-lowered
/// callable bodies remain local non-authorizing entries alongside the neutral
/// carrier. This is a transport handoff, not Engine admission or runtime
/// authority.
///
/// # Errors
///
/// Returns the same checked lowering errors as
/// [`lower_complete_checked_module_definition_closure`], plus selection
/// errors when the route map is incomplete, names an unexpected module, or
/// leaves an ordinary callable module without a selected entry.
#[allow(clippy::result_large_err)]
pub fn lower_complete_checked_module_route_closure(
    root: ModuleKey,
    finalized: &CanonicalCheckedModuleFinalization,
    collection: &CanonicalModuleCollection,
    expanded: &CanonicalExpandedModuleGraph,
    imports: &CanonicalParsedImportResult,
    selected_entries: &BTreeMap<ModuleKey, String>,
) -> Result<Vec<LoweredCheckedModuleDefinition>, ModuleCoreCpsLoweringError> {
    if imports != finalized.imports() {
        return Err(ModuleCoreCpsLoweringError::ImportTransportMismatch);
    }
    if let Some((module, _)) = selected_entries.iter().find(|(module, _)| {
        finalized
            .modules()
            .all(|candidate| candidate.module_key() != *module)
    }) {
        return Err(ModuleCoreCpsLoweringError::UnexpectedSelectedEntry {
            module: module.clone(),
        });
    }
    if selected_entries.get(&root).is_none_or(String::is_empty) {
        return Err(ModuleCoreCpsLoweringError::MissingSelectedEntry { module: root });
    }
    if let Some(module) = finalized
        .modules()
        .map(|module| module.module_key())
        .find(|module| !selected_entries.contains_key(*module))
    {
        return Err(ModuleCoreCpsLoweringError::MissingSelectedEntry {
            module: module.clone(),
        });
    }

    let mut lowered =
        lower_complete_checked_module_definition_closure(finalized, collection, expanded, imports)?;
    for finalized_module in finalized.modules() {
        let module_key = finalized_module.module_key();
        let selected_name = selected_entries.get(module_key).ok_or_else(|| {
            ModuleCoreCpsLoweringError::MissingSelectedEntry {
                module: module_key.clone(),
            }
        })?;
        let module_lowered = lowered
            .iter()
            .filter(|definition| definition.core().module_artifact().key() == module_key)
            .collect::<Vec<_>>();

        if selected_name.is_empty() {
            if !module_lowered.iter().any(|definition| {
                !definition.is_callable_entry()
                    && definition.core().module_artifact().key() == module_key
            }) {
                lowered.push(lower_checked_metadata_only_module(
                    finalized, collection, expanded, imports, module_key,
                )?);
            }
            continue;
        }

        let selected_definition = module_lowered.iter().find(|definition| {
            definition.is_callable_entry() && definition.declaration_name() == selected_name
        });
        if selected_definition.is_none() {
            return Err(ModuleCoreCpsLoweringError::MissingCheckedDefinition {
                module: module_key.clone(),
                name: selected_name.clone(),
            });
        }
    }

    Ok(lowered)
}

fn lower_checked_definition_with_imports(
    finalized: &CanonicalCheckedModuleFinalization,
    collection: &CanonicalModuleCollection,
    expanded: &CanonicalExpandedModuleGraph,
    module_key: &ModuleKey,
    declaration_name: &str,
    imports: &[ResolvedModuleImport],
    dependencies: &[ModuleInterfaceDependency],
) -> Result<(ModuleCoreArtifact, ModuleCpsArtifact), ModuleCoreCpsLoweringError> {
    let finalized_module = finalized.module(module_key).ok_or_else(|| {
        ModuleCoreCpsLoweringError::MissingCheckedModule {
            module: module_key.clone(),
        }
    })?;
    let declaration = finalized_module
        .private_declaration(declaration_name)
        .ok_or_else(|| ModuleCoreCpsLoweringError::MissingCheckedDefinition {
            module: module_key.clone(),
            name: declaration_name.to_owned(),
        })?;
    let type_environment = core_type_environment_for_module(finalized, module_key, declaration)?;
    let (core, cps) = lower_complete_checked_module_definition_body_with_environment(
        finalized,
        collection,
        expanded,
        module_key,
        declaration_name,
        declaration,
        &type_environment,
    )?;
    let checked_core_program = core.checked_core_program().clone();
    let cps_program = cps.cps_program().clone();
    let core_artifact = ModuleCoreArtifact::new_with_interface_metadata(
        core.module_artifact().clone(),
        imports.to_vec(),
        PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        dependencies.to_vec(),
        checked_core_program,
    );
    let cps_artifact = ModuleCpsArtifact::from_core_artifact(&core_artifact, cps_program);
    Ok((core_artifact, cps_artifact))
}

fn dependency_snapshot_for_module(
    imports: &CanonicalParsedImportResult,
    module_key: &ModuleKey,
) -> Vec<ModuleInterfaceDependency> {
    let mut import_edges = BTreeMap::<ModuleKey, BTreeSet<ModuleKey>>::new();
    for (importing_module, _, binding) in imports.bindings() {
        let defining_module = binding.defining_identity().module_key();
        if defining_module != importing_module {
            import_edges
                .entry(importing_module.clone())
                .or_default()
                .insert(defining_module.clone());
        }
    }
    for notation in imports.notation_imports() {
        // Notation imports are intentionally absent from ordinary bindings,
        // but their provider is still a checked syntax-phase dependency of
        // any Core/CPS carrier that retains the notation metadata.
        let provider_module = notation.provider_module();
        if provider_module != notation.importing_module() {
            import_edges
                .entry(notation.importing_module().clone())
                .or_default()
                .insert(provider_module.clone());
        }
    }

    // A checked module's Core/CPS carrier must name every module reachable
    // through its resolved import graph, not only the first imported target.
    // Keep the traversal keyed by canonical ModuleKey so cycles (which should
    // already have been rejected by TASK-2072) cannot duplicate entries or
    // make this transport preparation loop.
    let mut modules = BTreeSet::new();
    let mut pending = import_edges
        .remove(module_key)
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    while let Some(dependency) = pending.pop() {
        if !modules.insert(dependency.clone()) {
            continue;
        }
        if let Some(transitive) = import_edges.get(&dependency) {
            pending.extend(transitive.iter().cloned());
        }
    }

    modules
        .into_iter()
        .map(|module| {
            ModuleInterfaceDependency::new(module, PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION)
        })
        .collect()
}

fn resolved_imports_for_module(
    finalized: &CanonicalCheckedModuleFinalization,
    imports: &CanonicalParsedImportResult,
    module_key: &ModuleKey,
) -> Result<Vec<ResolvedModuleImport>, ModuleCoreCpsLoweringError> {
    let mut resolved = imports
        .bindings()
        .filter(|(importing_module, _, _)| *importing_module == module_key)
        .map(|(_, _, binding)| resolved_import(binding))
        .collect::<Result<Vec<_>, _>>()?;
    // Keep notation in the carrier as a phase-marked metadata fact rather
    // than manufacturing an ordinary local binding or runtime callable.
    resolved.extend(
        imports
            .notation_imports()
            .iter()
            .filter(|notation| notation.importing_module() == module_key)
            .map(resolved_notation_import),
    );
    resolved.extend(
        structural_module_alias_callable_imports(finalized, imports, module_key)
            .into_iter()
            .map(|import| {
                ResolvedModuleImport::with_visibility(
                    import.local_name,
                    import.binding,
                    ModuleImportVisibility::Public,
                )
            }),
    );
    Ok(resolved)
}

struct StructuralModuleAliasCallableImport {
    local_name: String,
    binding: ModuleInterfaceBinding,
    signature: crate::Type,
}

fn structural_module_alias_callable_imports(
    finalized: &CanonicalCheckedModuleFinalization,
    imports: &CanonicalParsedImportResult,
    module_key: &ModuleKey,
) -> Vec<StructuralModuleAliasCallableImport> {
    let mut result = Vec::new();
    for (_, alias, module_binding) in imports.bindings().filter(|(module, _, binding)| {
        *module == module_key
            && binding.lookup_key().namespace() == CanonicalNamespace::StructuralModule
    }) {
        let Some(target_module) = finalized.module(module_binding.defining_identity().module_key())
        else {
            continue;
        };
        for export in target_module.public_exports().filter(|export| {
            export.declaration().identity().canonical_parent().is_none()
                && export.declaration().namespace() == CanonicalNamespace::ValueCallable
        }) {
            let Some(signature) = export.declaration().signature().cloned() else {
                continue;
            };
            let local_name = format!("{alias}::{}", export.local_name());
            let binding = ModuleInterfaceBinding::declaration(
                &local_name,
                export.defining_identity().module_key().clone(),
                export.declaration().name(),
                ModuleInterfaceBindingKind::Callable,
                CoreVisibility::Public,
                export.declaration().origin().clone(),
            );
            result.push(StructuralModuleAliasCallableImport {
                local_name,
                binding,
                signature,
            });
        }
    }
    result
}

fn resolved_notation_import(notation: &CanonicalParsedNotationImport) -> ResolvedModuleImport {
    let visible_name = notation.lookup_key().visible_local_key();
    let binding = ModuleInterfaceBinding::declaration(
        visible_name,
        notation.defining_identity().module_key().clone(),
        visible_name,
        ModuleInterfaceBindingKind::SyntaxNotation,
        core_visibility(notation.declaration_visibility()),
        notation.origin().clone(),
    );
    ResolvedModuleImport::with_visibility(
        visible_name,
        binding,
        module_import_visibility(notation.declaration_visibility()),
    )
}

fn resolved_import(
    binding: &CanonicalParsedImportBinding,
) -> Result<ResolvedModuleImport, ModuleCoreCpsLoweringError> {
    let namespace = binding.lookup_key().namespace();
    let kind = binding.defining_identity().kind();
    let visibility_source = if binding.is_reexport() {
        binding.import_visibility()
    } else {
        binding.declaration_visibility()
    };
    let visibility = core_visibility(visibility_source);
    let module = binding.defining_identity().module_key().clone();
    let origin = binding.origin().clone();
    let core_binding = match namespace {
        crate::canonical_module_collection::CanonicalNamespace::StructuralModule => {
            ModuleInterfaceBinding::child(binding.local_name(), module, visibility, origin)
        }
        crate::canonical_module_collection::CanonicalNamespace::ValueCallable
            if matches!(
                kind,
                CanonicalDeclarationKind::Function
                    | CanonicalDeclarationKind::Handler
                    | CanonicalDeclarationKind::BuiltinFn
            ) && binding.defining_identity().canonical_parent().is_none() =>
        {
            ModuleInterfaceBinding::declaration(
                binding.local_name(),
                module,
                binding.lookup_key().visible_local_key(),
                ModuleInterfaceBindingKind::Callable,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::ValueCallable
            if kind == CanonicalDeclarationKind::Function
                && binding
                    .defining_identity()
                    .canonical_parent()
                    .is_some_and(|parent| {
                        matches!(
                            parent.kind(),
                            CanonicalDeclarationKind::Type | CanonicalDeclarationKind::Newtype
                        )
                    }) =>
        {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::Constructor,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::Macro => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::SyntaxMacro,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::Notation => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::SyntaxNotation,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::TypeDomain => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::Type,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::TypeComputation => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::TypeFunction,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::Proposition => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::Proposition,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::PromotedKind => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::PromotedKind,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::RowName => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::EffectRow,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::Interface => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::Interface,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::ImplementationRegistry => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::Implementation,
                visibility,
                origin,
            )
        }
        crate::canonical_module_collection::CanonicalNamespace::Evidence => {
            declaration_import_binding(
                binding,
                module,
                ModuleInterfaceBindingKind::Evidence,
                visibility,
                origin,
            )
        }
        _ => {
            return Err(unsupported_import_fact(
                binding,
                "namespace or identity is not lossless",
            ));
        }
    };
    Ok(ResolvedModuleImport::with_visibility(
        binding.local_name(),
        core_binding,
        module_import_visibility(visibility_source),
    ))
}

fn module_import_visibility(
    visibility: &ash_parser::surface::Visibility,
) -> ModuleImportVisibility {
    match visibility {
        ash_parser::surface::Visibility::Public => ModuleImportVisibility::Public,
        ash_parser::surface::Visibility::Crate => ModuleImportVisibility::Crate,
        ash_parser::surface::Visibility::Inherited | ash_parser::surface::Visibility::Self_ => {
            ModuleImportVisibility::Private
        }
        ash_parser::surface::Visibility::Super { levels } => {
            ModuleImportVisibility::Super { levels: *levels }
        }
        ash_parser::surface::Visibility::Restricted { path } => {
            ModuleImportVisibility::Restricted {
                path: path.to_string(),
            }
        }
    }
}

fn declaration_import_binding(
    binding: &CanonicalParsedImportBinding,
    module: ModuleKey,
    kind: ModuleInterfaceBindingKind,
    visibility: CoreVisibility,
    origin: ash_core::module_graph::ModuleArtifactOrigin,
) -> ModuleInterfaceBinding {
    ModuleInterfaceBinding::declaration(
        binding.local_name(),
        module,
        binding.lookup_key().visible_local_key(),
        kind,
        visibility,
        origin,
    )
}

fn unsupported_import_fact(
    binding: &CanonicalParsedImportBinding,
    reason: &str,
) -> ModuleCoreCpsLoweringError {
    ModuleCoreCpsLoweringError::UnsupportedImportFact {
        local_name: binding.local_name().to_owned(),
        namespace: binding.lookup_key().namespace(),
        kind: binding.defining_identity().kind(),
        reason: reason.to_owned(),
    }
}

fn core_visibility(visibility: &ash_parser::surface::Visibility) -> CoreVisibility {
    // The checker has already enforced the exact parent/region boundary. The
    // Core import carrier only needs to retain that this binding is not public;
    // private transport cannot publish through PublicModuleInterface.
    match visibility {
        ash_parser::surface::Visibility::Public => CoreVisibility::Public,
        ash_parser::surface::Visibility::Crate => CoreVisibility::Crate,
        ash_parser::surface::Visibility::Inherited
        | ash_parser::surface::Visibility::Super { .. }
        | ash_parser::surface::Visibility::Self_
        | ash_parser::surface::Visibility::Restricted { .. } => CoreVisibility::Private,
    }
}

struct PrimitiveBinding {
    name: String,
    op: CorePrimOp,
    args: Vec<CoreAtom>,
}

struct ValueBinding {
    name: String,
    ty: CoreType,
    value: CoreValue,
}

#[derive(Default)]
struct SurfaceCoreLoweringState {
    type_environment: CoreTypeCheckEnv,
    next_primitive: usize,
    next_call: usize,
    next_discard: usize,
    next_value: usize,
}

fn surface_core_expr_to_checked_core(
    expr: ash_core::Expr,
    type_environment: &CoreTypeCheckEnv,
) -> Result<CoreExpr, String> {
    let mut state = SurfaceCoreLoweringState {
        type_environment: type_environment.clone(),
        ..Default::default()
    };
    surface_core_expr_to_checked_core_with_state(expr, &mut state)
}

fn surface_core_expr_to_checked_core_with_state(
    expr: ash_core::Expr,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    let mut bindings = Vec::new();
    let mut value_bindings = Vec::new();
    let mut core = match expr {
        ash_core::Expr::Let {
            pattern,
            expr,
            body,
            ..
        } => {
            let name = match pattern {
                ash_core::Pattern::Variable { name, .. } if name != "_" => name,
                ash_core::Pattern::Variable { .. } | ash_core::Pattern::Wildcard => {
                    let name = format!("__ash_discard_{}", state.next_discard);
                    state.next_discard += 1;
                    name
                }
                pattern => {
                    return Err(format!(
                        "let pattern `{pattern:?}` has no checked Core binding projection"
                    ));
                }
            };
            let mut value_bindings = Vec::new();
            let mut let_bindings = Vec::new();
            let body = surface_core_expr_to_checked_core_with_state(*body, state)?;
            let mut let_expr = if matches!(
                &*expr,
                ash_core::Expr::Binary {
                    op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
                    ..
                }
            ) {
                let then_branch = CoreExpr::LetVal {
                    name: name.clone(),
                    ty: CoreType::Base("Bool".to_owned()),
                    value: CoreValue::Atom(CoreAtom::LitBool(true)),
                    body: Box::new(body.clone()),
                };
                let else_branch = CoreExpr::LetVal {
                    name,
                    ty: CoreType::Base("Bool".to_owned()),
                    value: CoreValue::Atom(CoreAtom::LitBool(false)),
                    body: Box::new(body),
                };
                lower_boolean_control_expression(
                    *expr,
                    then_branch,
                    else_branch,
                    &mut value_bindings,
                    &mut let_bindings,
                    state,
                )?
            } else {
                match *expr {
                    ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. } => {
                        let (func, args, call_bindings) = surface_application_to_core_parts(
                            *expr,
                            &mut value_bindings,
                            &mut let_bindings,
                            state,
                        )?;
                        let mut call = CoreExpr::LetCall {
                            name,
                            func,
                            args,
                            body: Box::new(body),
                        };
                        for binding in call_bindings.into_iter().rev() {
                            call = CoreExpr::LetCall {
                                name: binding.name,
                                func: binding.func,
                                args: binding.args,
                                body: Box::new(call),
                            };
                        }
                        call
                    }
                    ash_core::Expr::Record { fields }
                        if record_requires_sequenced_field_lowering(&fields) =>
                    {
                        lower_record_let_with_short_circuit(name, fields, body, state)?
                    }
                    other => {
                        let value = surface_expr_to_primitive_atom(
                            other,
                            &mut value_bindings,
                            &mut let_bindings,
                            state,
                        )?;
                        if let CoreAtom::Var(generated) = &value
                            && let Some(binding_index) = let_bindings
                                .iter()
                                .position(|binding| binding.name == *generated)
                            && matches!(
                                let_bindings[binding_index].op,
                                CorePrimOp::RecordGet(_) | CorePrimOp::TupleGet(_)
                            )
                        {
                            let binding = let_bindings.remove(binding_index);
                            CoreExpr::LetPrim {
                                name,
                                op: binding.op,
                                args: binding.args,
                                body: Box::new(body),
                            }
                        } else {
                            let value_ty =
                                core_type_for_bound_atom(&value, &let_bindings, &value_bindings)?;
                            CoreExpr::LetVal {
                                name,
                                ty: value_ty,
                                value: CoreValue::Atom(value),
                                body: Box::new(body),
                            }
                        }
                    }
                }
            };
            for binding in let_bindings.into_iter().rev() {
                let_expr = CoreExpr::LetPrim {
                    name: binding.name,
                    op: binding.op,
                    args: binding.args,
                    body: Box::new(let_expr),
                };
            }
            for binding in value_bindings.into_iter().rev() {
                let_expr = CoreExpr::LetVal {
                    name: binding.name,
                    ty: binding.ty,
                    value: binding.value,
                    body: Box::new(let_expr),
                };
            }
            let_expr
        }
        call @ ash_core::Expr::Call { .. } if application_has_short_circuit_argument(&call) => {
            surface_application_with_short_circuit_arguments(call, state)?
        }
        ash_core::Expr::Call { .. } => {
            let (func, args, call_bindings) =
                surface_application_to_core_parts(expr, &mut value_bindings, &mut bindings, state)?;
            let mut call = CoreExpr::Call { func, args };
            for binding in call_bindings.into_iter().rev() {
                call = CoreExpr::LetCall {
                    name: binding.name,
                    func: binding.func,
                    args: binding.args,
                    body: Box::new(call),
                };
            }
            call
        }
        apply @ ash_core::Expr::FnApply { .. }
            if application_has_short_circuit_argument(&apply) =>
        {
            surface_application_with_short_circuit_arguments(apply, state)?
        }
        ash_core::Expr::FnApply { func, args } => {
            let (func, args, call_bindings) = surface_application_to_core_parts(
                ash_core::Expr::FnApply { func, args },
                &mut value_bindings,
                &mut bindings,
                state,
            )?;
            let mut call = CoreExpr::Call { func, args };
            for binding in call_bindings.into_iter().rev() {
                call = CoreExpr::LetCall {
                    name: binding.name,
                    func: binding.func,
                    args: binding.args,
                    body: Box::new(call),
                };
            }
            call
        }
        ash_core::Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
        } => {
            let expected = match pattern {
                ash_core::Pattern::Literal(ash_core::Value::Bool(expected)) => expected,
                unsupported => {
                    return Err(format!(
                        "if-let pattern `{unsupported:?}` has no checked boolean Core projection"
                    ));
                }
            };
            if matches!(
                &*expr,
                ash_core::Expr::Binary {
                    op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
                    ..
                }
            ) {
                let then_branch =
                    surface_core_expr_to_checked_core_with_state(*then_branch, state)?;
                let else_branch =
                    surface_core_expr_to_checked_core_with_state(*else_branch, state)?;
                let (matched_branch, unmatched_branch) = if expected {
                    (then_branch, else_branch)
                } else {
                    (else_branch, then_branch)
                };
                return lower_boolean_control_expression(
                    *expr,
                    matched_branch,
                    unmatched_branch,
                    &mut value_bindings,
                    &mut bindings,
                    state,
                );
            }
            if application_has_short_circuit_argument(&expr) {
                let then_branch =
                    surface_core_expr_to_checked_core_with_state(*then_branch, state)?;
                let else_branch =
                    surface_core_expr_to_checked_core_with_state(*else_branch, state)?;
                let (matched_branch, unmatched_branch) = if expected {
                    (then_branch, else_branch)
                } else {
                    (else_branch, then_branch)
                };
                return lower_application_control_expression(
                    *expr,
                    matched_branch,
                    unmatched_branch,
                    &mut value_bindings,
                    &mut bindings,
                    state,
                );
            }
            let mut call_bindings = Vec::new();
            let condition = if matches!(
                &*expr,
                ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. }
            ) {
                let (func, args, nested_bindings) = surface_application_to_core_parts(
                    *expr,
                    &mut value_bindings,
                    &mut bindings,
                    state,
                )?;
                call_bindings.extend(nested_bindings);
                let name = format!("__ash_lowered_if_let_call_{}", state.next_call);
                state.next_call += 1;
                call_bindings.push(CallBinding {
                    name: name.clone(),
                    func,
                    args,
                });
                CoreAtom::Var(name)
            } else {
                surface_expr_to_primitive_atom(*expr, &mut value_bindings, &mut bindings, state)?
            };
            let inverted_name =
                (!expected).then(|| format!("__ash_lowered_if_let_not_{}", state.next_primitive));
            if inverted_name.is_some() {
                state.next_primitive += 1;
            }
            let condition_for_if = inverted_name
                .as_ref()
                .map_or_else(|| condition.clone(), |name| CoreAtom::Var(name.clone()));
            let mut conditional = CoreExpr::If {
                cond: condition_for_if,
                then_branch: Box::new(surface_core_expr_to_checked_core_with_state(
                    *then_branch,
                    state,
                )?),
                else_branch: Box::new(surface_core_expr_to_checked_core_with_state(
                    *else_branch,
                    state,
                )?),
            };
            for binding in call_bindings.into_iter().rev() {
                conditional = CoreExpr::LetCall {
                    name: binding.name,
                    func: binding.func,
                    args: binding.args,
                    body: Box::new(conditional),
                };
            }
            if !expected {
                conditional = CoreExpr::LetPrim {
                    name: inverted_name.expect("false if-let allocates a negation name"),
                    op: CorePrimOp::Not,
                    args: vec![condition],
                    body: Box::new(conditional),
                };
            }
            conditional
        }
        ash_core::Expr::Match { scrutinee, arms } => {
            let mut then_branch = None;
            let mut else_branch = None;
            for arm in arms {
                match arm.pattern {
                    ash_core::Pattern::Literal(ash_core::Value::Bool(true)) => {
                        then_branch = Some(arm.body);
                    }
                    ash_core::Pattern::Literal(ash_core::Value::Bool(false)) => {
                        else_branch = Some(arm.body);
                    }
                    pattern => {
                        return Err(format!(
                            "match pattern `{pattern:?}` has no checked boolean Core projection"
                        ));
                    }
                }
            }
            let then_branch = then_branch.ok_or_else(|| {
                "boolean match is missing its true branch in checked Core projection".to_owned()
            })?;
            let else_branch = else_branch.ok_or_else(|| {
                "boolean match is missing its false branch in checked Core projection".to_owned()
            })?;
            if matches!(
                &*scrutinee,
                ash_core::Expr::Binary {
                    op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
                    ..
                }
            ) {
                let then_branch = surface_core_expr_to_checked_core_with_state(then_branch, state)?;
                let else_branch = surface_core_expr_to_checked_core_with_state(else_branch, state)?;
                return lower_boolean_control_expression(
                    *scrutinee,
                    then_branch,
                    else_branch,
                    &mut value_bindings,
                    &mut bindings,
                    state,
                );
            }
            if application_has_short_circuit_argument(&scrutinee) {
                let then_branch = surface_core_expr_to_checked_core_with_state(then_branch, state)?;
                let else_branch = surface_core_expr_to_checked_core_with_state(else_branch, state)?;
                return lower_application_control_expression(
                    *scrutinee,
                    then_branch,
                    else_branch,
                    &mut value_bindings,
                    &mut bindings,
                    state,
                );
            }
            let mut call_bindings = Vec::new();
            let cond = if matches!(
                &*scrutinee,
                ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. }
            ) {
                let (func, args, nested_bindings) = surface_application_to_core_parts(
                    *scrutinee,
                    &mut value_bindings,
                    &mut bindings,
                    state,
                )?;
                call_bindings.extend(nested_bindings);
                let name = format!("__ash_lowered_match_call_{}", state.next_call);
                state.next_call += 1;
                call_bindings.push(CallBinding {
                    name: name.clone(),
                    func,
                    args,
                });
                CoreAtom::Var(name)
            } else {
                surface_expr_to_primitive_atom(
                    *scrutinee,
                    &mut value_bindings,
                    &mut bindings,
                    state,
                )?
            };
            let mut conditional = CoreExpr::If {
                cond,
                then_branch: Box::new(surface_core_expr_to_checked_core_with_state(
                    then_branch,
                    state,
                )?),
                else_branch: Box::new(surface_core_expr_to_checked_core_with_state(
                    else_branch,
                    state,
                )?),
            };
            for binding in call_bindings.into_iter().rev() {
                conditional = CoreExpr::LetCall {
                    name: binding.name,
                    func: binding.func,
                    args: binding.args,
                    body: Box::new(conditional),
                };
            }
            conditional
        }
        ash_core::Expr::Binary { op, left, right }
            if matches!(op, ash_core::BinaryOp::And | ash_core::BinaryOp::Or) =>
        {
            lower_boolean_control_expression(
                ash_core::Expr::Binary { op, left, right },
                CoreExpr::Atom(CoreAtom::LitBool(true)),
                CoreExpr::Atom(CoreAtom::LitBool(false)),
                &mut value_bindings,
                &mut bindings,
                state,
            )?
        }
        ash_core::Expr::Binary { op, left, right } => {
            let (left, mut call_bindings) = surface_operand_to_atom_with_calls(
                *left,
                &mut value_bindings,
                &mut bindings,
                state,
            )?;
            let (right, right_call_bindings) = surface_operand_to_atom_with_calls(
                *right,
                &mut value_bindings,
                &mut bindings,
                state,
            )?;
            call_bindings.extend(right_call_bindings);
            let core_op = match op {
                ash_core::BinaryOp::Add => CorePrimOp::Add,
                ash_core::BinaryOp::Sub => CorePrimOp::Sub,
                ash_core::BinaryOp::Mul => CorePrimOp::Mul,
                ash_core::BinaryOp::Div => CorePrimOp::Div,
                ash_core::BinaryOp::Mod => CorePrimOp::Rem,
                ash_core::BinaryOp::Eq => CorePrimOp::Eq,
                ash_core::BinaryOp::Ne => CorePrimOp::Ne,
                ash_core::BinaryOp::Lt => CorePrimOp::Lt,
                ash_core::BinaryOp::Le => CorePrimOp::Le,
                ash_core::BinaryOp::Gt => CorePrimOp::Gt,
                ash_core::BinaryOp::Ge => CorePrimOp::Ge,
                unsupported => {
                    return Err(format!(
                        "binary operator `{unsupported:?}` has no checked Core primitive"
                    ));
                }
            };
            let name = format!("__ash_primitive_{}", state.next_primitive);
            state.next_primitive += 1;
            let mut binary = CoreExpr::LetPrim {
                name: name.clone(),
                op: core_op,
                args: vec![left, right],
                body: Box::new(CoreExpr::Atom(CoreAtom::Var(name))),
            };
            for binding in call_bindings.into_iter().rev() {
                binary = CoreExpr::LetCall {
                    name: binding.name,
                    func: binding.func,
                    args: binding.args,
                    body: Box::new(binary),
                };
            }
            binary
        }
        other => {
            let atom =
                surface_expr_to_primitive_atom(other, &mut value_bindings, &mut bindings, state)?;
            CoreExpr::Atom(atom)
        }
    };
    for binding in bindings.into_iter().rev() {
        core = CoreExpr::LetPrim {
            name: binding.name,
            op: binding.op,
            args: binding.args,
            body: Box::new(core),
        };
    }
    for binding in value_bindings.into_iter().rev() {
        core = CoreExpr::LetVal {
            name: binding.name,
            ty: binding.ty,
            value: binding.value,
            body: Box::new(core),
        };
    }
    Ok(core)
}

fn surface_operand_to_atom_with_calls(
    expr: ash_core::Expr,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<(CoreAtom, Vec<CallBinding>), String> {
    if matches!(
        &expr,
        ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. }
    ) {
        let (func, args, mut call_bindings) =
            surface_application_to_core_parts(expr, value_bindings, bindings, state)?;
        let name = format!("__ash_lowered_call_{}", state.next_call);
        state.next_call += 1;
        call_bindings.push(CallBinding {
            name: name.clone(),
            func,
            args,
        });
        Ok((CoreAtom::Var(name), call_bindings))
    } else if let ash_core::Expr::Record { fields } = expr {
        if record_contains_callable_call(&fields) {
            surface_record_to_atom_with_calls(fields, value_bindings, bindings, state)
        } else {
            Ok((
                surface_expr_to_primitive_atom(
                    ash_core::Expr::Record { fields },
                    value_bindings,
                    bindings,
                    state,
                )?,
                Vec::new(),
            ))
        }
    } else if expression_contains_callable_call(&expr) {
        match expr {
            ash_core::Expr::FieldAccess { expr, field } => {
                let (base, call_bindings) =
                    surface_operand_to_atom_with_calls(*expr, value_bindings, bindings, state)?;
                let atom = push_primitive_binding(
                    bindings,
                    state,
                    CorePrimOp::RecordGet(field),
                    vec![base],
                );
                Ok((atom, call_bindings))
            }
            ash_core::Expr::Unary { op, expr } => {
                let (operand, call_bindings) =
                    surface_operand_to_atom_with_calls(*expr, value_bindings, bindings, state)?;
                let core_op = match op {
                    ash_core::UnaryOp::Neg => CorePrimOp::Neg,
                    ash_core::UnaryOp::Not => CorePrimOp::Not,
                };
                let atom = push_primitive_binding(bindings, state, core_op, vec![operand]);
                Ok((atom, call_bindings))
            }
            ash_core::Expr::Binary { op, left, right } => {
                let (left, mut call_bindings) =
                    surface_operand_to_atom_with_calls(*left, value_bindings, bindings, state)?;
                let (right, right_calls) =
                    surface_operand_to_atom_with_calls(*right, value_bindings, bindings, state)?;
                call_bindings.extend(right_calls);
                let core_op = match op {
                    ash_core::BinaryOp::Add => CorePrimOp::Add,
                    ash_core::BinaryOp::Sub => CorePrimOp::Sub,
                    ash_core::BinaryOp::Mul => CorePrimOp::Mul,
                    ash_core::BinaryOp::Div => CorePrimOp::Div,
                    ash_core::BinaryOp::Mod => CorePrimOp::Rem,
                    ash_core::BinaryOp::Eq => CorePrimOp::Eq,
                    ash_core::BinaryOp::Ne => CorePrimOp::Ne,
                    ash_core::BinaryOp::Lt => CorePrimOp::Lt,
                    ash_core::BinaryOp::Le => CorePrimOp::Le,
                    ash_core::BinaryOp::Gt => CorePrimOp::Gt,
                    ash_core::BinaryOp::Ge => CorePrimOp::Ge,
                    unsupported => {
                        return Err(format!(
                            "binary operator `{unsupported:?}` has no checked Core primitive"
                        ));
                    }
                };
                let atom = push_primitive_binding(bindings, state, core_op, vec![left, right]);
                Ok((atom, call_bindings))
            }
            other => Ok((
                surface_expr_to_primitive_atom(other, value_bindings, bindings, state)?,
                Vec::new(),
            )),
        }
    } else {
        Ok((
            surface_expr_to_primitive_atom(expr, value_bindings, bindings, state)?,
            Vec::new(),
        ))
    }
}

fn surface_record_to_atom_with_calls(
    fields: Vec<(String, ash_core::Expr)>,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<(CoreAtom, Vec<CallBinding>), String> {
    let mut core_fields = Vec::with_capacity(fields.len());
    let mut field_types = Vec::with_capacity(fields.len());
    let mut call_bindings = Vec::new();
    for (name, field_expr) in fields {
        let (atom, nested_calls) =
            surface_operand_to_atom_with_calls(field_expr, value_bindings, bindings, state)?;
        let field_ty = core_type_for_bound_atom(&atom, bindings, value_bindings).or_else(|_| {
            nested_calls
                .last()
                .map(|binding| core_type_for_call_binding(binding, state))
                .unwrap_or_else(|| Err(format!("record field `{name}` has no checked Core type")))
        })?;
        call_bindings.extend(nested_calls);
        core_fields.push((name.clone(), atom));
        field_types.push((name, field_ty));
    }
    let name = format!("__ash_value_{}", state.next_value);
    state.next_value += 1;
    value_bindings.push(ValueBinding {
        name: name.clone(),
        ty: CoreType::Record(field_types),
        value: CoreValue::Record {
            fields: core_fields,
        },
    });
    Ok((CoreAtom::Var(name), call_bindings))
}

fn surface_callable_atom(
    expr: ash_core::Expr,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreAtom, String> {
    match expr {
        ash_core::Expr::Variable { name, .. } => Ok(CoreAtom::Var(name)),
        other => surface_expr_to_primitive_atom(other, value_bindings, bindings, state),
    }
}

struct CallBinding {
    name: String,
    func: CoreAtom,
    args: Vec<CoreAtom>,
}

fn application_has_short_circuit_argument(expr: &ash_core::Expr) -> bool {
    let arguments = match expr {
        ash_core::Expr::Call { arguments, .. }
        | ash_core::Expr::FnApply {
            args: arguments, ..
        } => arguments,
        _ => return false,
    };
    arguments.iter().any(|argument| {
        matches!(
            argument,
            ash_core::Expr::Binary {
                op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
                ..
            }
        ) || matches!(
            argument,
            ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. }
        ) && application_has_short_circuit_argument(argument)
    })
}

fn record_requires_sequenced_field_lowering(fields: &[(String, ash_core::Expr)]) -> bool {
    fields.iter().any(|(_, field)| {
        matches!(
            field,
            ash_core::Expr::Binary {
                op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
                ..
            }
        ) || expression_contains_callable_call(field)
    })
}

fn record_contains_callable_call(fields: &[(String, ash_core::Expr)]) -> bool {
    fields
        .iter()
        .any(|(_, expression)| expression_contains_callable_call(expression))
}

fn expression_contains_callable_call(expr: &ash_core::Expr) -> bool {
    match expr {
        ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. } => true,
        ash_core::Expr::Record { fields } => record_contains_callable_call(fields),
        ash_core::Expr::FieldAccess { expr, .. }
        | ash_core::Expr::Unary { expr, .. }
        | ash_core::Expr::Split(expr)
        | ash_core::Expr::Fail { payload: expr }
        | ash_core::Expr::Spawn { init: expr, .. } => expression_contains_callable_call(expr),
        ash_core::Expr::IndexAccess { expr, index } => {
            expression_contains_callable_call(expr) || expression_contains_callable_call(index)
        }
        ash_core::Expr::Binary { left, right, .. } => {
            expression_contains_callable_call(left) || expression_contains_callable_call(right)
        }
        ash_core::Expr::Let { expr, body, .. } => {
            expression_contains_callable_call(expr) || expression_contains_callable_call(body)
        }
        ash_core::Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            expression_contains_callable_call(expr)
                || expression_contains_callable_call(then_branch)
                || expression_contains_callable_call(else_branch)
        }
        ash_core::Expr::Match { scrutinee, arms } => {
            expression_contains_callable_call(scrutinee)
                || arms
                    .iter()
                    .any(|arm| expression_contains_callable_call(&arm.body))
        }
        ash_core::Expr::Literal(_)
        | ash_core::Expr::Variable { .. }
        | ash_core::Expr::Constructor { .. }
        | ash_core::Expr::CheckObligation { .. }
        | ash_core::Expr::WithError { .. }
        | ash_core::Expr::FnDef { .. } => false,
    }
}

fn lower_record_let_with_short_circuit(
    name: String,
    fields: Vec<(String, ash_core::Expr)>,
    body: CoreExpr,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    lower_record_field_sequence(&name, &fields, 0, Vec::new(), body, state)
}

fn lower_record_field_sequence(
    name: &str,
    fields: &[(String, ash_core::Expr)],
    index: usize,
    collected: Vec<(String, CoreAtom, CoreType)>,
    body: CoreExpr,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    if index == fields.len() {
        let field_types = collected
            .iter()
            .map(|(field, _, ty)| (field.clone(), ty.clone()))
            .collect();
        let record_fields = collected
            .into_iter()
            .map(|(field, atom, _)| (field, atom))
            .collect();
        return Ok(CoreExpr::LetVal {
            name: name.to_owned(),
            ty: CoreType::Record(field_types),
            value: CoreValue::Record {
                fields: record_fields,
            },
            body: Box::new(body),
        });
    }

    let (field, expression) = &fields[index];
    if matches!(
        expression,
        ash_core::Expr::Binary {
            op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
            ..
        }
    ) {
        let then_collected = {
            let mut collected = collected.clone();
            collected.push((
                field.clone(),
                CoreAtom::LitBool(true),
                CoreType::Base("Bool".to_owned()),
            ));
            collected
        };
        let else_collected = {
            let mut collected = collected;
            collected.push((
                field.clone(),
                CoreAtom::LitBool(false),
                CoreType::Base("Bool".to_owned()),
            ));
            collected
        };
        let then_branch = lower_record_field_sequence(
            name,
            fields,
            index + 1,
            then_collected,
            body.clone(),
            state,
        )?;
        let else_branch =
            lower_record_field_sequence(name, fields, index + 1, else_collected, body, state)?;
        let mut value_bindings = Vec::new();
        let mut bindings = Vec::new();
        let mut core = lower_boolean_control_expression(
            expression.clone(),
            then_branch,
            else_branch,
            &mut value_bindings,
            &mut bindings,
            state,
        )?;
        for binding in bindings.into_iter().rev() {
            core = CoreExpr::LetPrim {
                name: binding.name,
                op: binding.op,
                args: binding.args,
                body: Box::new(core),
            };
        }
        for binding in value_bindings.into_iter().rev() {
            core = CoreExpr::LetVal {
                name: binding.name,
                ty: binding.ty,
                value: binding.value,
                body: Box::new(core),
            };
        }
        return Ok(core);
    }

    let mut value_bindings = Vec::new();
    let mut bindings = Vec::new();
    let (atom, call_bindings) = surface_operand_to_atom_with_calls(
        expression.clone(),
        &mut value_bindings,
        &mut bindings,
        state,
    )?;
    let ty = core_type_for_bound_atom(&atom, &bindings, &value_bindings).or_else(|_| {
        call_bindings
            .last()
            .map(|binding| core_type_for_call_binding(binding, state))
            .unwrap_or_else(|| Err("record field has no checked Core type".to_owned()))
    })?;
    let mut collected = collected;
    collected.push((field.clone(), atom, ty));
    let mut core = lower_record_field_sequence(name, fields, index + 1, collected, body, state)?;
    if value_bindings_depend_on_calls(&value_bindings, &call_bindings) {
        for binding in bindings.into_iter().rev() {
            core = CoreExpr::LetPrim {
                name: binding.name,
                op: binding.op,
                args: binding.args,
                body: Box::new(core),
            };
        }
        for binding in value_bindings.into_iter().rev() {
            core = CoreExpr::LetVal {
                name: binding.name,
                ty: binding.ty,
                value: binding.value,
                body: Box::new(core),
            };
        }
        for binding in call_bindings.into_iter().rev() {
            core = CoreExpr::LetCall {
                name: binding.name,
                func: binding.func,
                args: binding.args,
                body: Box::new(core),
            };
        }
    } else {
        for binding in bindings.into_iter().rev() {
            core = CoreExpr::LetPrim {
                name: binding.name,
                op: binding.op,
                args: binding.args,
                body: Box::new(core),
            };
        }
        for binding in call_bindings.into_iter().rev() {
            core = CoreExpr::LetCall {
                name: binding.name,
                func: binding.func,
                args: binding.args,
                body: Box::new(core),
            };
        }
        for binding in value_bindings.into_iter().rev() {
            core = CoreExpr::LetVal {
                name: binding.name,
                ty: binding.ty,
                value: binding.value,
                body: Box::new(core),
            };
        }
    }
    Ok(core)
}

fn core_type_for_call_binding(
    binding: &CallBinding,
    state: &SurfaceCoreLoweringState,
) -> Result<CoreType, String> {
    let CoreAtom::Var(function_name) = &binding.func else {
        return Err(format!(
            "record field call target `{}` has no checked Core function type",
            format_core_atom(&binding.func)
        ));
    };
    let Some(CoreType::Function { result, .. }) =
        state.type_environment.values().lookup(function_name)
    else {
        return Err(format!(
            "record field call target `{function_name}` has no checked Core function type"
        ));
    };
    Ok((**result).clone())
}

fn value_bindings_depend_on_calls(
    value_bindings: &[ValueBinding],
    call_bindings: &[CallBinding],
) -> bool {
    let call_names = call_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect::<BTreeSet<_>>();
    value_bindings
        .iter()
        .any(|binding| core_value_references_any_name(&binding.value, &call_names))
}

fn core_value_references_any_name(value: &CoreValue, names: &BTreeSet<&str>) -> bool {
    match value {
        CoreValue::Atom(atom) => core_atom_references_any_name(atom, names),
        CoreValue::Record { fields } => fields
            .iter()
            .any(|(_, atom)| core_atom_references_any_name(atom, names)),
        CoreValue::Tuple { elems } => elems
            .iter()
            .any(|atom| core_atom_references_any_name(atom, names)),
        CoreValue::Lam { .. } | CoreValue::Thunk { .. } | CoreValue::DischargeMarker { .. } => {
            false
        }
    }
}

fn core_atom_references_any_name(atom: &CoreAtom, names: &BTreeSet<&str>) -> bool {
    matches!(atom, CoreAtom::Var(name) if names.contains(name.as_str()))
}

fn format_core_atom(atom: &CoreAtom) -> String {
    match atom {
        CoreAtom::Var(name) | CoreAtom::ConstructorName(name) => name.clone(),
        CoreAtom::PrimName(op) => format!("{op:?}"),
        CoreAtom::LitInt(value) => value.to_string(),
        CoreAtom::LitString(value) => format!("{value:?}"),
        CoreAtom::LitBool(value) => value.to_string(),
        CoreAtom::LitUnit => "()".to_owned(),
    }
}

fn surface_application_with_short_circuit_arguments(
    expr: ash_core::Expr,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    let (func, arguments, mut value_bindings, bindings) = match expr {
        ash_core::Expr::Call {
            func,
            module,
            arguments,
        } => {
            let func = match module {
                Some(module) => format!("{module}::{func}"),
                None => func,
            };
            (CoreAtom::Var(func), arguments, Vec::new(), Vec::new())
        }
        ash_core::Expr::FnApply { func, args } => {
            let mut value_bindings = Vec::new();
            let mut bindings = Vec::new();
            let func = surface_callable_atom(*func, &mut value_bindings, &mut bindings, state)?;
            (func, args, value_bindings, bindings)
        }
        other => {
            return Err(format!(
                "surface expression `{other:?}` has no checked Core application projection"
            ));
        }
    };

    let mut core = lower_application_argument_sequence(func, &arguments, 0, Vec::new(), state)?;
    for binding in bindings.into_iter().rev() {
        core = CoreExpr::LetPrim {
            name: binding.name,
            op: binding.op,
            args: binding.args,
            body: Box::new(core),
        };
    }
    for binding in value_bindings.drain(..).rev() {
        core = CoreExpr::LetVal {
            name: binding.name,
            ty: binding.ty,
            value: binding.value,
            body: Box::new(core),
        };
    }
    Ok(core)
}

fn lower_application_argument_sequence(
    func: CoreAtom,
    arguments: &[ash_core::Expr],
    index: usize,
    collected: Vec<CoreAtom>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    if index == arguments.len() {
        return Ok(CoreExpr::Call {
            func,
            args: collected,
        });
    }

    let argument = arguments[index].clone();
    if matches!(
        &argument,
        ash_core::Expr::Binary {
            op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
            ..
        }
    ) {
        let name = format!("__ash_lowered_argument_{}", state.next_value);
        state.next_value += 1;
        let mut then_arguments = collected.clone();
        then_arguments.push(CoreAtom::Var(name.clone()));
        let mut else_arguments = collected;
        else_arguments.push(CoreAtom::Var(name.clone()));
        let then_branch = lower_application_argument_sequence(
            func.clone(),
            arguments,
            index + 1,
            then_arguments,
            state,
        )?;
        let else_branch =
            lower_application_argument_sequence(func, arguments, index + 1, else_arguments, state)?;
        let then_branch = CoreExpr::LetVal {
            name: name.clone(),
            ty: CoreType::Base("Bool".to_owned()),
            value: CoreValue::Atom(CoreAtom::LitBool(true)),
            body: Box::new(then_branch),
        };
        let else_branch = CoreExpr::LetVal {
            name,
            ty: CoreType::Base("Bool".to_owned()),
            value: CoreValue::Atom(CoreAtom::LitBool(false)),
            body: Box::new(else_branch),
        };
        let mut value_bindings = Vec::new();
        let mut bindings = Vec::new();
        let mut core = lower_boolean_control_expression(
            argument,
            then_branch,
            else_branch,
            &mut value_bindings,
            &mut bindings,
            state,
        )?;
        for binding in bindings.into_iter().rev() {
            core = CoreExpr::LetPrim {
                name: binding.name,
                op: binding.op,
                args: binding.args,
                body: Box::new(core),
            };
        }
        for binding in value_bindings.into_iter().rev() {
            core = CoreExpr::LetVal {
                name: binding.name,
                ty: binding.ty,
                value: binding.value,
                body: Box::new(core),
            };
        }
        return Ok(core);
    }

    if application_has_short_circuit_argument(&argument) {
        let result_name = format!("__ash_lowered_nested_argument_{}", state.next_call);
        state.next_call += 1;
        let mut collected = collected;
        collected.push(CoreAtom::Var(result_name.clone()));
        let continuation =
            lower_application_argument_sequence(func, arguments, index + 1, collected, state)?;
        return lower_application_value_with_short_circuit(
            argument,
            result_name,
            continuation,
            state,
        );
    }

    let mut value_bindings = Vec::new();
    let mut bindings = Vec::new();
    let (atom, call_bindings) =
        surface_operand_to_atom_with_calls(argument, &mut value_bindings, &mut bindings, state)?;
    let mut collected = collected;
    collected.push(atom);
    let mut core =
        lower_application_argument_sequence(func, arguments, index + 1, collected, state)?;
    for binding in call_bindings.into_iter().rev() {
        core = CoreExpr::LetCall {
            name: binding.name,
            func: binding.func,
            args: binding.args,
            body: Box::new(core),
        };
    }
    for binding in bindings.into_iter().rev() {
        core = CoreExpr::LetPrim {
            name: binding.name,
            op: binding.op,
            args: binding.args,
            body: Box::new(core),
        };
    }
    for binding in value_bindings.into_iter().rev() {
        core = CoreExpr::LetVal {
            name: binding.name,
            ty: binding.ty,
            value: binding.value,
            body: Box::new(core),
        };
    }
    Ok(core)
}

fn lower_application_control_expression(
    expr: ash_core::Expr,
    then_branch: CoreExpr,
    else_branch: CoreExpr,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    let (func, arguments) = match expr {
        ash_core::Expr::Call {
            func,
            module,
            arguments,
        } => {
            let func = match module {
                Some(module) => format!("{module}::{func}"),
                None => func,
            };
            (CoreAtom::Var(func), arguments)
        }
        ash_core::Expr::FnApply { func, args } => (
            surface_callable_atom(*func, value_bindings, bindings, state)?,
            args,
        ),
        other => {
            return Err(format!(
                "surface expression `{other:?}` has no checked Core application projection"
            ));
        }
    };
    let result_name = format!("__ash_lowered_condition_call_{}", state.next_call);
    state.next_call += 1;
    lower_application_control_argument_sequence(
        func,
        &arguments,
        0,
        Vec::new(),
        result_name,
        then_branch,
        else_branch,
        state,
    )
}

#[allow(clippy::too_many_arguments)]
fn lower_application_control_argument_sequence(
    func: CoreAtom,
    arguments: &[ash_core::Expr],
    index: usize,
    collected: Vec<CoreAtom>,
    result_name: String,
    then_branch: CoreExpr,
    else_branch: CoreExpr,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    if index == arguments.len() {
        return Ok(CoreExpr::LetCall {
            name: result_name.clone(),
            func,
            args: collected,
            body: Box::new(CoreExpr::If {
                cond: CoreAtom::Var(result_name),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            }),
        });
    }

    let argument = arguments[index].clone();
    if matches!(
        &argument,
        ash_core::Expr::Binary {
            op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
            ..
        }
    ) {
        let name = format!("__ash_lowered_condition_argument_{}", state.next_value);
        state.next_value += 1;
        let mut then_arguments = collected.clone();
        then_arguments.push(CoreAtom::Var(name.clone()));
        let mut else_arguments = collected;
        else_arguments.push(CoreAtom::Var(name.clone()));
        let then_continuation = then_branch;
        let else_continuation = else_branch;
        let then_branch = lower_application_control_argument_sequence(
            func.clone(),
            arguments,
            index + 1,
            then_arguments,
            result_name.clone(),
            then_continuation.clone(),
            else_continuation.clone(),
            state,
        )?;
        let else_branch = lower_application_control_argument_sequence(
            func,
            arguments,
            index + 1,
            else_arguments,
            result_name,
            then_continuation,
            else_continuation,
            state,
        )?;
        let then_branch = CoreExpr::LetVal {
            name: name.clone(),
            ty: CoreType::Base("Bool".to_owned()),
            value: CoreValue::Atom(CoreAtom::LitBool(true)),
            body: Box::new(then_branch),
        };
        let else_branch = CoreExpr::LetVal {
            name,
            ty: CoreType::Base("Bool".to_owned()),
            value: CoreValue::Atom(CoreAtom::LitBool(false)),
            body: Box::new(else_branch),
        };
        let mut value_bindings = Vec::new();
        let mut bindings = Vec::new();
        let mut core = lower_boolean_control_expression(
            argument,
            then_branch,
            else_branch,
            &mut value_bindings,
            &mut bindings,
            state,
        )?;
        for binding in bindings.into_iter().rev() {
            core = CoreExpr::LetPrim {
                name: binding.name,
                op: binding.op,
                args: binding.args,
                body: Box::new(core),
            };
        }
        for binding in value_bindings.into_iter().rev() {
            core = CoreExpr::LetVal {
                name: binding.name,
                ty: binding.ty,
                value: binding.value,
                body: Box::new(core),
            };
        }
        return Ok(core);
    }

    if application_has_short_circuit_argument(&argument) {
        let nested_result_name = format!(
            "__ash_lowered_nested_condition_argument_{}",
            state.next_call
        );
        state.next_call += 1;
        let mut collected = collected;
        collected.push(CoreAtom::Var(nested_result_name.clone()));
        let continuation = lower_application_control_argument_sequence(
            func,
            arguments,
            index + 1,
            collected,
            result_name,
            then_branch,
            else_branch,
            state,
        )?;
        return lower_application_value_with_short_circuit(
            argument,
            nested_result_name,
            continuation,
            state,
        );
    }

    let mut value_bindings = Vec::new();
    let mut bindings = Vec::new();
    let (atom, call_bindings) =
        surface_operand_to_atom_with_calls(argument, &mut value_bindings, &mut bindings, state)?;
    let mut collected = collected;
    collected.push(atom);
    let mut core = lower_application_control_argument_sequence(
        func,
        arguments,
        index + 1,
        collected,
        result_name,
        then_branch,
        else_branch,
        state,
    )?;
    for binding in call_bindings.into_iter().rev() {
        core = CoreExpr::LetCall {
            name: binding.name,
            func: binding.func,
            args: binding.args,
            body: Box::new(core),
        };
    }
    for binding in bindings.into_iter().rev() {
        core = CoreExpr::LetPrim {
            name: binding.name,
            op: binding.op,
            args: binding.args,
            body: Box::new(core),
        };
    }
    for binding in value_bindings.into_iter().rev() {
        core = CoreExpr::LetVal {
            name: binding.name,
            ty: binding.ty,
            value: binding.value,
            body: Box::new(core),
        };
    }
    Ok(core)
}

/// Lowers an application containing a short-circuit argument and resumes a
/// value-producing continuation with the application's result. The
/// continuation is deliberately identical on both result branches: the
/// nested application is a value argument, so its boolean result—not its
/// truth value—must be passed to the enclosing call.
fn lower_application_value_with_short_circuit(
    expr: ash_core::Expr,
    result_name: String,
    continuation: CoreExpr,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    let mut value_bindings = Vec::new();
    let mut bindings = Vec::new();
    let (func, arguments) = match expr {
        ash_core::Expr::Call {
            func,
            module,
            arguments,
        } => {
            let func = match module {
                Some(module) => format!("{module}::{func}"),
                None => func,
            };
            (CoreAtom::Var(func), arguments)
        }
        ash_core::Expr::FnApply { func, args } => (
            surface_callable_atom(*func, &mut value_bindings, &mut bindings, state)?,
            args,
        ),
        other => {
            return Err(format!(
                "surface expression `{other:?}` has no checked Core application projection"
            ));
        }
    };

    let mut core = lower_application_control_argument_sequence(
        func,
        &arguments,
        0,
        Vec::new(),
        result_name,
        continuation.clone(),
        continuation,
        state,
    )?;
    for binding in bindings.into_iter().rev() {
        core = CoreExpr::LetPrim {
            name: binding.name,
            op: binding.op,
            args: binding.args,
            body: Box::new(core),
        };
    }
    for binding in value_bindings.into_iter().rev() {
        core = CoreExpr::LetVal {
            name: binding.name,
            ty: binding.ty,
            value: binding.value,
            body: Box::new(core),
        };
    }
    Ok(core)
}

fn surface_application_to_core_parts(
    expr: ash_core::Expr,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<(CoreAtom, Vec<CoreAtom>, Vec<CallBinding>), String> {
    let (func, arguments) = match expr {
        ash_core::Expr::Call {
            func,
            module,
            arguments,
        } => {
            let func = match module {
                Some(module) => format!("{module}::{func}"),
                None => func,
            };
            (CoreAtom::Var(func), arguments)
        }
        ash_core::Expr::FnApply { func, args } => (
            surface_callable_atom(*func, value_bindings, bindings, state)?,
            args,
        ),
        other => {
            return Err(format!(
                "surface expression `{other:?}` has no checked Core application projection"
            ));
        }
    };

    let mut call_bindings = Vec::new();
    let mut atoms = Vec::with_capacity(arguments.len());
    for argument in arguments {
        if matches!(
            &argument,
            ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. }
        ) {
            let (nested_func, nested_args, nested_bindings) =
                surface_application_to_core_parts(argument, value_bindings, bindings, state)?;
            call_bindings.extend(nested_bindings);
            let name = format!("__ash_lowered_call_{}", state.next_call);
            state.next_call += 1;
            call_bindings.push(CallBinding {
                name: name.clone(),
                func: nested_func,
                args: nested_args,
            });
            atoms.push(CoreAtom::Var(name));
        } else {
            atoms.push(surface_expr_to_primitive_atom(
                argument,
                value_bindings,
                bindings,
                state,
            )?);
        }
    }

    Ok((func, atoms, call_bindings))
}

fn core_type_for_bound_atom(
    atom: &CoreAtom,
    bindings: &[PrimitiveBinding],
    value_bindings: &[ValueBinding],
) -> Result<CoreType, String> {
    match atom {
        CoreAtom::Var(name) => bindings
            .iter()
            .find(|binding| binding.name == *name)
            .map(|binding| core_type_for_primitive_op(&binding.op))
            .or_else(|| {
                value_bindings
                    .iter()
                    .find(|binding| binding.name == *name)
                    .map(|binding| binding.ty.clone())
            })
            .ok_or_else(|| format!("let initializer variable `{name}` has no checked Core type")),
        other => Ok(core_type_for_atom(other)),
    }
}

fn core_type_for_primitive_op(op: &CorePrimOp) -> CoreType {
    match op {
        CorePrimOp::Add
        | CorePrimOp::Sub
        | CorePrimOp::Mul
        | CorePrimOp::Div
        | CorePrimOp::Rem
        | CorePrimOp::Neg => CoreType::Base("Int".to_owned()),
        CorePrimOp::Eq
        | CorePrimOp::Ne
        | CorePrimOp::Lt
        | CorePrimOp::Le
        | CorePrimOp::Gt
        | CorePrimOp::Ge
        | CorePrimOp::Not => CoreType::Base("Bool".to_owned()),
        CorePrimOp::RecordGet(_) | CorePrimOp::TupleGet(_) | CorePrimOp::ConstructorTag(_) => {
            CoreType::Named("Unknown".to_owned())
        }
    }
}

fn core_type_for_atom(atom: &CoreAtom) -> CoreType {
    match atom {
        CoreAtom::LitInt(_) => CoreType::Base("Int".to_owned()),
        CoreAtom::LitString(_) => CoreType::Base("String".to_owned()),
        CoreAtom::LitBool(_) => CoreType::Base("Bool".to_owned()),
        CoreAtom::LitUnit => CoreType::Base("Unit".to_owned()),
        CoreAtom::Var(_) | CoreAtom::PrimName(_) | CoreAtom::ConstructorName(_) => {
            unreachable!("let initializers only admit literal Core atoms in this slice")
        }
    }
}

fn surface_expr_to_primitive_atom(
    expr: ash_core::Expr,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreAtom, String> {
    match expr {
        ash_core::Expr::Literal(ash_core::Value::Int(value)) => Ok(CoreAtom::LitInt(value)),
        ash_core::Expr::Literal(ash_core::Value::String(value)) => Ok(CoreAtom::LitString(value)),
        ash_core::Expr::Literal(ash_core::Value::Bool(value)) => Ok(CoreAtom::LitBool(value)),
        ash_core::Expr::Literal(ash_core::Value::Null) => Ok(CoreAtom::LitUnit),
        ash_core::Expr::Variable { name, .. } => Ok(CoreAtom::Var(name)),
        ash_core::Expr::FieldAccess { expr, field } => {
            let base = surface_expr_to_primitive_atom(*expr, value_bindings, bindings, state)?;
            Ok(push_primitive_binding(
                bindings,
                state,
                CorePrimOp::RecordGet(field),
                vec![base],
            ))
        }
        ash_core::Expr::Record { fields } => {
            let mut core_fields = Vec::with_capacity(fields.len());
            let mut field_types = Vec::with_capacity(fields.len());
            for (name, field_expr) in fields {
                let atom =
                    surface_expr_to_primitive_atom(field_expr, value_bindings, bindings, state)?;
                let field_ty = core_type_for_bound_atom(&atom, bindings, value_bindings)?;
                core_fields.push((name, atom));
                field_types.push((core_fields.last().unwrap().0.clone(), field_ty));
            }
            let name = format!("__ash_value_{}", state.next_value);
            state.next_value += 1;
            value_bindings.push(ValueBinding {
                name: name.clone(),
                ty: CoreType::Record(field_types),
                value: CoreValue::Record {
                    fields: core_fields,
                },
            });
            Ok(CoreAtom::Var(name))
        }
        ash_core::Expr::Unary { op, expr } => {
            let operand = surface_expr_to_primitive_atom(*expr, value_bindings, bindings, state)?;
            let core_op = match op {
                ash_core::UnaryOp::Neg => CorePrimOp::Neg,
                ash_core::UnaryOp::Not => CorePrimOp::Not,
            };
            Ok(push_primitive_binding(
                bindings,
                state,
                core_op,
                vec![operand],
            ))
        }
        ash_core::Expr::Binary { op, left, right } => {
            let left = surface_expr_to_primitive_atom(*left, value_bindings, bindings, state)?;
            let right = surface_expr_to_primitive_atom(*right, value_bindings, bindings, state)?;
            let core_op = match op {
                ash_core::BinaryOp::Add => CorePrimOp::Add,
                ash_core::BinaryOp::Sub => CorePrimOp::Sub,
                ash_core::BinaryOp::Mul => CorePrimOp::Mul,
                ash_core::BinaryOp::Div => CorePrimOp::Div,
                ash_core::BinaryOp::Mod => CorePrimOp::Rem,
                ash_core::BinaryOp::Eq => CorePrimOp::Eq,
                ash_core::BinaryOp::Ne => CorePrimOp::Ne,
                ash_core::BinaryOp::Lt => CorePrimOp::Lt,
                ash_core::BinaryOp::Le => CorePrimOp::Le,
                ash_core::BinaryOp::Gt => CorePrimOp::Gt,
                ash_core::BinaryOp::Ge => CorePrimOp::Ge,
                unsupported => {
                    return Err(format!(
                        "binary operator `{unsupported:?}` has no checked Core primitive"
                    ));
                }
            };
            Ok(push_primitive_binding(
                bindings,
                state,
                core_op,
                vec![left, right],
            ))
        }
        other => Err(format!(
            "surface expression `{other:?}` has no checked Core projection in this slice"
        )),
    }
}

fn lower_boolean_control_expression(
    expr: ash_core::Expr,
    then_branch: CoreExpr,
    else_branch: CoreExpr,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<CoreExpr, String> {
    if let ash_core::Expr::Binary { op, left, right } = expr {
        if matches!(op, ash_core::BinaryOp::And | ash_core::BinaryOp::Or) {
            let is_and = op == ash_core::BinaryOp::And;
            let (nested_then, nested_else) = if is_and {
                (
                    lower_boolean_control_expression(
                        *right,
                        then_branch,
                        else_branch.clone(),
                        value_bindings,
                        bindings,
                        state,
                    )?,
                    else_branch,
                )
            } else {
                (
                    then_branch.clone(),
                    lower_boolean_control_expression(
                        *right,
                        then_branch,
                        else_branch,
                        value_bindings,
                        bindings,
                        state,
                    )?,
                )
            };
            if matches!(
                &*left,
                ash_core::Expr::Binary {
                    op: ash_core::BinaryOp::And | ash_core::BinaryOp::Or,
                    ..
                }
            ) {
                return lower_boolean_control_expression(
                    *left,
                    nested_then,
                    nested_else,
                    value_bindings,
                    bindings,
                    state,
                );
            }
            let (condition, call_bindings) =
                surface_boolean_condition_atom(*left, value_bindings, bindings, state)?;
            let mut conditional = CoreExpr::If {
                cond: condition,
                then_branch: Box::new(nested_then),
                else_branch: Box::new(nested_else),
            };
            for binding in call_bindings.into_iter().rev() {
                conditional = CoreExpr::LetCall {
                    name: binding.name,
                    func: binding.func,
                    args: binding.args,
                    body: Box::new(conditional),
                };
            }
            return Ok(conditional);
        }
        let expr = ash_core::Expr::Binary { op, left, right };
        let (condition, call_bindings) =
            surface_boolean_condition_atom(expr, value_bindings, bindings, state)?;
        let mut conditional = CoreExpr::If {
            cond: condition,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        };
        for binding in call_bindings.into_iter().rev() {
            conditional = CoreExpr::LetCall {
                name: binding.name,
                func: binding.func,
                args: binding.args,
                body: Box::new(conditional),
            };
        }
        return Ok(conditional);
    }

    let (condition, call_bindings) =
        surface_boolean_condition_atom(expr, value_bindings, bindings, state)?;
    let mut conditional = CoreExpr::If {
        cond: condition,
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    };
    for binding in call_bindings.into_iter().rev() {
        conditional = CoreExpr::LetCall {
            name: binding.name,
            func: binding.func,
            args: binding.args,
            body: Box::new(conditional),
        };
    }
    Ok(conditional)
}

fn surface_boolean_condition_atom(
    expr: ash_core::Expr,
    value_bindings: &mut Vec<ValueBinding>,
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
) -> Result<(CoreAtom, Vec<CallBinding>), String> {
    if matches!(
        &expr,
        ash_core::Expr::Call { .. } | ash_core::Expr::FnApply { .. }
    ) {
        let (func, args, call_bindings) =
            surface_application_to_core_parts(expr, value_bindings, bindings, state)?;
        let name = format!("__ash_lowered_boolean_call_{}", state.next_call);
        state.next_call += 1;
        let mut call_bindings = call_bindings;
        call_bindings.push(CallBinding {
            name: name.clone(),
            func,
            args,
        });
        Ok((CoreAtom::Var(name), call_bindings))
    } else {
        Ok((
            surface_expr_to_primitive_atom(expr, value_bindings, bindings, state)?,
            Vec::new(),
        ))
    }
}

fn push_primitive_binding(
    bindings: &mut Vec<PrimitiveBinding>,
    state: &mut SurfaceCoreLoweringState,
    op: CorePrimOp,
    args: Vec<CoreAtom>,
) -> CoreAtom {
    let name = format!("__ash_primitive_{}", state.next_primitive);
    state.next_primitive += 1;
    bindings.push(PrimitiveBinding {
        name: name.clone(),
        op,
        args,
    });
    CoreAtom::Var(name)
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
