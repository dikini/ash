//! Checked, export-closed module facts for the canonical module pipeline.
//!
//! This boundary consumes the checker-internal collection and the atomically
//! staged parsed-import result.  It never performs source acquisition or name
//! lookup.  All declarations are staged privately first; only after every
//! callable signature/body and public export has been validated is an opaque final set
//! returned to the caller.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{
    BuiltinFnDef, ComputationRow, ComputationRowItem, DataKindDef, Definition, DomainConstructor,
    EffectAliasDef, EffectGroupDef, FnDef, HandlerDef, ImplDef, ImplMethodDef, InterfaceDef,
    LawDef, MacroSummary, ModuleFile, NotationDecl, PolicyDef, ProofDef, PropositionPredicateDecl,
    RoleDef, SealedDomainDef, Type as SurfaceType, TypeBody, TypeDef as SurfaceTypeDef, TypeFnDef,
    TypeParam, Visibility,
};
use ash_parser::{CanonicalExpandedModuleGraph, Span, Spanned, collect_public_macro_summaries};
use thiserror::Error;

use crate::canonical_module_collection::{
    CanonicalCollectedEntry, CanonicalDeclarationIdentity, CanonicalDeclarationKind,
    CanonicalDeclarationOriginKey, CanonicalModuleCollection, CanonicalNamespace,
};
use crate::canonical_parsed_import_resolver::{
    CanonicalParsedImportBinding, CanonicalParsedImportResult,
};
use crate::canonical_provisional_module_scopes::CanonicalProvisionalModuleScopes;
use crate::check_expr::check_expr;
use crate::types::unify;
use crate::{
    Kind, Type, TypeEnv, TypeVar, builtin_fn_signature_type, check_function_body_in_env,
    check_handler_body_in_env, fn_signature_type, handler_signature_type,
    workflow_surface_type_to_type,
};

/// Checked visibility metadata for evidence nested under a public declaration.
///
/// Interface laws and implementation proofs are carried through their parent
/// summary.  They never become standalone module exports merely because the
/// parent declaration is public.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCheckedNestedEvidenceSummary {
    name: Box<str>,
    kind: CanonicalDeclarationKind,
    visibility: Visibility,
}

impl CanonicalCheckedNestedEvidenceSummary {
    /// Returns the nested evidence name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the nested evidence declaration kind.
    #[must_use]
    pub const fn kind(&self) -> CanonicalDeclarationKind {
        self.kind
    }

    /// Returns the parser-carried nested evidence visibility.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

/// Body-free checked metadata for one public implementation declaration.
///
/// Implementation members remain parent-scoped and are never copied into the
/// provisional name view.  This summary carries only the interface head,
/// dependency-relevant type metadata, and member names needed by a public
/// interface; method bodies remain private to the checker.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCheckedImplementationSummary {
    interface: Box<str>,
    type_params: Box<[Box<str>]>,
    type_args: Box<[SurfaceType]>,
    where_bounds: Box<[(Box<str>, Box<str>)]>,
    associated_types: Box<[(Box<str>, SurfaceType)]>,
    methods: Box<[Box<str>]>,
    handlers: Box<[Box<str>]>,
    proofs: Box<[CanonicalCheckedNestedEvidenceSummary]>,
}

impl CanonicalCheckedImplementationSummary {
    /// Returns the implemented interface name.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the implementation's source-visible type parameters.
    #[must_use]
    pub fn type_params(&self) -> &[Box<str>] {
        &self.type_params
    }

    /// Returns the concrete or parameterized interface head arguments.
    #[must_use]
    pub fn type_args(&self) -> &[SurfaceType] {
        &self.type_args
    }

    /// Returns interface bounds attached to implementation type parameters.
    #[must_use]
    pub fn where_bounds(&self) -> &[(Box<str>, Box<str>)] {
        &self.where_bounds
    }

    /// Returns associated-type names and their checked surface expressions.
    #[must_use]
    pub fn associated_types(&self) -> &[(Box<str>, SurfaceType)] {
        &self.associated_types
    }

    /// Returns implementation method names without their private bodies.
    #[must_use]
    pub fn methods(&self) -> &[Box<str>] {
        &self.methods
    }

    /// Returns co-located handler names without their private bodies.
    #[must_use]
    pub fn handlers(&self) -> &[Box<str>] {
        &self.handlers
    }

    /// Returns matched proof summaries without proof-body authority.
    #[must_use]
    pub fn proofs(&self) -> &[CanonicalCheckedNestedEvidenceSummary] {
        &self.proofs
    }
}

/// One complete checked declaration retained in a module's private view.
/// Checked namespace metadata retained for declarations whose facts are not
/// callable signatures or bodies.
///
/// These values are copied from the checker-internal collection only after
/// declaration registration succeeds. They are metadata for later interface
/// validation, never a source or import authority.
#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalCheckedDeclarationFact {
    /// No namespace-specific metadata is available in this bounded slice.
    Opaque,
    /// Checked ordinary type declaration shape.
    Type {
        /// Source-visible generic parameter names.
        params: Box<[Box<str>]>,
        /// Checked declaration body shape retained for public validation.
        body: TypeBody,
        /// Whether the declaration is runtime-managed builtin substrate.
        builtin: bool,
    },
    /// Checked nominal newtype declaration shape.
    Newtype {
        /// Source-visible generic parameters and bounds.
        type_params: Box<[TypeParam]>,
        /// Sole value constructor name.
        constructor: Box<str>,
        /// Representation type retained for export-closure validation.
        representation: SurfaceType,
    },
    /// Checked resource type field schema.
    ResourceType {
        /// Ordered public schema fields and their source types.
        fields: Box<[(Box<str>, SurfaceType)]>,
    },
    /// Checked public interface declaration shape and method metadata.
    Interface {
        /// Raw parser-owned interface declaration retained after collection checking.
        definition: InterfaceDef,
        /// Parent-scoped law visibility retained in the public interface summary.
        evidence: Box<[CanonicalCheckedNestedEvidenceSummary]>,
    },
    /// Checked public implementation metadata without private member bodies.
    Implementation {
        /// Body-free implementation summary retained for public closure.
        summary: CanonicalCheckedImplementationSummary,
    },
    /// Checked public computation-effect row alias metadata.
    EffectAlias {
        /// Raw parser-owned effect-row alias retained after collection checking.
        definition: EffectAliasDef,
    },
    /// Checked public computation-effect row group metadata.
    EffectGroup {
        /// Raw parser-owned effect-row group retained after collection checking.
        definition: EffectGroupDef,
    },
    /// Checked promoted data-kind declaration metadata.
    DataKind {
        /// Raw parser-owned promoted-kind declaration retained after collection checking.
        definition: DataKindDef,
    },
    /// Checked named proposition-predicate declaration metadata.
    PropositionPredicate {
        /// Raw parser-owned predicate declaration retained after collection checking.
        definition: PropositionPredicateDecl,
    },
    /// Checked public role declaration metadata.
    Role {
        /// Raw parser-owned role declaration retained after collection checking.
        definition: RoleDef,
    },
    /// Checked public policy schema metadata.
    Policy {
        /// Raw parser-owned policy schema retained after collection checking.
        definition: Box<PolicyDef>,
    },
    /// Bounded checked type-function declaration metadata.
    TypeFn {
        /// Raw parser-owned type-function declaration retained after collection checking.
        definition: Box<TypeFnDef>,
    },
    /// Checked public syntax-notation declaration metadata.
    Notation {
        /// Raw parser-owned notation declaration retained after collection checking.
        definition: NotationDecl,
    },
    /// Checked public syntax-phase macro summary metadata.
    Macro {
        /// Parser-owned summary that carries no runtime callable authority.
        summary: MacroSummary,
    },
    /// Checked public module-law evidence metadata.
    Law {
        /// Parser-owned law declaration retained after proposition checking.
        definition: Box<LawDef>,
    },
    /// Checked public module-proof evidence metadata.
    Proof {
        /// Parser-owned proof declaration retained after proof checking.
        definition: Box<ProofDef>,
    },
    /// Checked sealed type-level domain declaration shape.
    SealedDomain {
        /// Raw parser-owned domain declaration retained after collection checking.
        definition: SealedDomainDef,
    },
    /// A constructor nested under a checked type or newtype declaration.
    Constructor {
        /// Defining parent type identity.
        parent: CanonicalDeclarationIdentity,
        /// Source-visible constructor name.
        name: Box<str>,
    },
    /// A marker constructor nested under a sealed type-level domain.
    SealedDomainConstructor {
        /// Defining parent sealed-domain identity.
        parent: CanonicalDeclarationIdentity,
        /// Checked marker-constructor metadata.
        constructor: DomainConstructor,
    },
    /// A checked structural child-module identity retained in its parent.
    StructuralModule {
        /// Canonical child module declared by the parent module item.
        module: ModuleKey,
    },
}

/// One complete checked declaration retained in a module's private view.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCheckedDeclaration {
    identity: CanonicalDeclarationIdentity,
    name: Box<str>,
    kind: CanonicalDeclarationKind,
    namespace: CanonicalNamespace,
    declaration_span: Span,
    body_span: Option<Span>,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
    signature: Option<Type>,
    body_type: Option<Type>,
    fact: CanonicalCheckedDeclarationFact,
}

impl CanonicalCheckedDeclaration {
    /// Returns the stable declaration identity retained from collection.
    #[must_use]
    pub fn identity(&self) -> &CanonicalDeclarationIdentity {
        &self.identity
    }

    /// Returns the defining declaration name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the collected declaration kind.
    #[must_use]
    pub const fn kind(&self) -> CanonicalDeclarationKind {
        self.kind
    }

    /// Returns the collected namespace bucket.
    #[must_use]
    pub const fn namespace(&self) -> CanonicalNamespace {
        self.namespace
    }

    /// Returns the declaration source anchor.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns the checked callable body anchor, when this declaration has a body.
    #[must_use]
    pub const fn body_span(&self) -> Option<Span> {
        self.body_span
    }

    /// Returns the acquisition origin of the defining module.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the source visibility retained by collection.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Returns the checked callable signature, when this declaration is callable.
    #[must_use]
    pub fn signature(&self) -> Option<&Type> {
        self.signature.as_ref()
    }

    /// Returns the inferred and declared-compatible body type, when checked.
    #[must_use]
    pub fn body_type(&self) -> Option<&Type> {
        self.body_type.as_ref()
    }

    /// Returns checked namespace metadata retained for this declaration.
    #[must_use]
    pub fn fact(&self) -> &CanonicalCheckedDeclarationFact {
        &self.fact
    }

    fn is_exported(&self) -> bool {
        if matches!(
            self.fact,
            CanonicalCheckedDeclarationFact::SealedDomainConstructor { .. }
        ) {
            return false;
        }
        if self.identity.canonical_parent().is_some_and(|parent| {
            matches!(
                parent.kind(),
                CanonicalDeclarationKind::Interface | CanonicalDeclarationKind::Impl
            )
        }) {
            return false;
        }
        matches!(self.visibility, Visibility::Public)
    }
}

/// Checked public declaration facts projected from a private declaration.
///
/// The public projection deliberately omits the private body's span and inferred body type.  A
/// consumer can use its checked signature and provenance without receiving the private checking
/// carrier.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCheckedPublicDeclaration {
    identity: CanonicalDeclarationIdentity,
    name: Box<str>,
    kind: CanonicalDeclarationKind,
    namespace: CanonicalNamespace,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
    signature: Option<Type>,
    fact: CanonicalCheckedDeclarationFact,
}

impl CanonicalCheckedPublicDeclaration {
    fn from_private(declaration: &CanonicalCheckedDeclaration) -> Self {
        Self {
            identity: declaration.identity.clone(),
            name: declaration.name.clone(),
            kind: declaration.kind,
            namespace: declaration.namespace,
            declaration_span: declaration.declaration_span,
            origin: declaration.origin.clone(),
            visibility: declaration.visibility.clone(),
            signature: declaration.signature.clone(),
            fact: declaration.fact.clone(),
        }
    }

    /// Returns the stable defining declaration identity.
    #[must_use]
    pub fn identity(&self) -> &CanonicalDeclarationIdentity {
        &self.identity
    }

    /// Returns the local name under which the defining declaration is projected.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the declaration kind retained by the public projection.
    #[must_use]
    pub const fn kind(&self) -> CanonicalDeclarationKind {
        self.kind
    }

    /// Returns the namespace retained by the public projection.
    #[must_use]
    pub const fn namespace(&self) -> CanonicalNamespace {
        self.namespace
    }

    /// Returns the defining declaration span.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns the defining module's acquisition origin.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the visibility that authorized this public projection.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Returns the checked public callable signature, when one exists.
    #[must_use]
    pub fn signature(&self) -> Option<&Type> {
        self.signature.as_ref()
    }

    /// Returns checked public namespace metadata for this declaration.
    #[must_use]
    pub fn fact(&self) -> &CanonicalCheckedDeclarationFact {
        &self.fact
    }
}

/// One final public export, including a public defining projection and optional re-export span.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCheckedExport {
    local_name: Box<str>,
    declaration: CanonicalCheckedPublicDeclaration,
    import_span: Option<Span>,
}

impl CanonicalCheckedExport {
    /// Returns the name exported by the receiving module.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the original defining declaration identity.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalDeclarationIdentity {
        self.declaration.identity()
    }

    /// Returns the checked defining declaration.
    #[must_use]
    pub fn declaration(&self) -> &CanonicalCheckedPublicDeclaration {
        &self.declaration
    }

    /// Returns the source span of the staged public use, when this is a re-export.
    #[must_use]
    pub const fn import_span(&self) -> Option<Span> {
        self.import_span
    }
}

/// One atomically finalized module interface.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCheckedModuleInterface {
    module_key: ModuleKey,
    origin: ModuleArtifactOrigin,
    private_declarations: Box<[CanonicalCheckedDeclaration]>,
    public_exports: BTreeMap<(CanonicalNamespace, Box<str>), CanonicalCheckedExport>,
}

impl CanonicalCheckedModuleInterface {
    /// Returns this interface's canonical module identity.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the source-acquisition origin for this module.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Iterates over the complete checked private declaration view.
    pub fn private_declarations(&self) -> impl Iterator<Item = &CanonicalCheckedDeclaration> {
        self.private_declarations.iter()
    }

    /// Looks up one checked private declaration by its defining name.
    #[must_use]
    pub fn private_declaration(&self, name: &str) -> Option<&CanonicalCheckedDeclaration> {
        self.private_declarations
            .iter()
            .find(|declaration| declaration.name() == name)
    }

    /// Looks up one export-closed public projection by local exported name.
    ///
    /// Returns `None` when more than one namespace exports the spelling. Use
    /// [`Self::public_export_in_namespace`] for an unambiguous lookup.
    #[must_use]
    pub fn public_export(&self, name: &str) -> Option<&CanonicalCheckedExport> {
        let mut matches = self
            .public_exports
            .values()
            .filter(|export| export.local_name() == name);
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }

    /// Looks up one export-closed public projection by namespace and local
    /// exported name.
    #[must_use]
    pub fn public_export_in_namespace(
        &self,
        namespace: CanonicalNamespace,
        name: &str,
    ) -> Option<&CanonicalCheckedExport> {
        self.public_exports.get(&(namespace, name.into()))
    }

    /// Iterates over exports in deterministic namespace/name order.
    pub fn public_exports(&self) -> impl Iterator<Item = &CanonicalCheckedExport> {
        self.public_exports.values()
    }
}

/// Atomically finalized checked interfaces for all collected modules.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCheckedModuleFinalization {
    modules: BTreeMap<ModuleKey, CanonicalCheckedModuleInterface>,
}

impl CanonicalCheckedModuleFinalization {
    /// Returns the final interface for one canonical module.
    #[must_use]
    pub fn module(&self, module_key: &ModuleKey) -> Option<&CanonicalCheckedModuleInterface> {
        self.modules.get(module_key)
    }

    /// Iterates over all finalized interfaces in canonical-key order.
    pub fn modules(&self) -> impl Iterator<Item = &CanonicalCheckedModuleInterface> {
        self.modules.values()
    }
}

/// Diagnostic context for a public use that crosses a non-public module path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonExportedPublicUseModulePathDetails {
    /// Source span of the public use.
    pub span: Span,
    /// Source span of the offending module declaration.
    pub declaration_span: Span,
    /// Canonical attempted access path.
    pub path: Box<str>,
    /// First module segment that is not publicly reachable.
    pub offending_segment: Box<str>,
    /// Human-readable visibility boundary that was violated.
    pub violated_visibility: Box<str>,
}

/// Failure that prevents publication of any final checked interface.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CanonicalCheckedModuleFinalizationError {
    /// The expanded and collected module key sets differ.
    #[error("checked finalization module keys do not match")]
    GraphMismatch,
    /// A collected module has no durable parser artifact.
    #[error("checked finalization is missing parser artifact for module {module}")]
    MissingArtifact { module: ModuleKey },
    /// A structural module declaration points at a child absent from the collected closure.
    #[error("checked module declaration {module}::{name:?} has no collected child module {child}")]
    MissingStructuralModuleTarget {
        module: ModuleKey,
        name: Box<str>,
        child: Box<ModuleKey>,
        span: Span,
    },
    /// A structural module declaration does not point at its canonical child identity.
    #[error("checked module declaration {module}::{name:?} points at {child}, expected {expected}")]
    StructuralModuleIdentityMismatch {
        module: ModuleKey,
        name: Box<str>,
        child: Box<ModuleKey>,
        expected: Box<ModuleKey>,
        span: Span,
    },
    /// A staged binding points to no collected declaration.
    #[error("checked import {name:?} in {module} has no collected defining identity")]
    MissingBindingTarget { module: ModuleKey, name: Box<str> },
    /// A callable identity has no matching checked declaration skeleton.
    #[error("checked callable {name:?} in {module} has no declaration fact")]
    MissingCheckedDeclaration {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
    },
    /// A staged binding's defining-module origin disagrees with the acquired artifact.
    #[error("checked import {name:?} in {module} has mismatched defining origin")]
    BindingOriginMismatch { module: ModuleKey, name: Box<str> },
    /// A staged binding's declaration visibility disagrees with the acquired declaration.
    #[error("checked import {name:?} in {module} has mismatched declaration visibility")]
    BindingVisibilityMismatch { module: ModuleKey, name: Box<str> },
    /// A staged binding's defining module path is not visible from its importing module.
    #[error("checked import {name:?} in {module} has an inaccessible defining module path")]
    BindingModuleVisibilityMismatch { module: ModuleKey, name: Box<str> },
    /// A staged binding's declaration metadata disagrees with its identity target.
    #[error("checked import {name:?} in {module} has mismatched declaration metadata")]
    BindingDeclarationMetadataMismatch { module: ModuleKey, name: Box<str> },
    /// A staged public-use carrier disagrees with its authoritative import binding.
    #[error("checked public use {name:?} in {module} has mismatched binding")]
    PublicUseBindingMismatch { module: ModuleKey, name: Box<str> },
    /// A staged binding's local name disagrees with its authoritative import-map key.
    #[error(
        "checked import in {module} has mismatched local name: authoritative map name {authoritative_name:?}, binding local name {binding_local_name:?}"
    )]
    BindingLocalNameMismatch {
        module: ModuleKey,
        authoritative_name: Box<str>,
        binding_local_name: Box<str>,
    },
    /// A staged binding's namespace or declaration kind disagrees with the acquired declaration.
    #[error(
        "checked import {name:?} in {module} has mismatched declaration shape: binding namespace {binding_namespace:?}, target namespace {target_namespace:?}, binding kind {binding_kind:?}, target kind {target_kind:?}"
    )]
    BindingShapeMismatch {
        module: ModuleKey,
        name: Box<str>,
        binding_namespace: CanonicalNamespace,
        target_namespace: CanonicalNamespace,
        binding_kind: CanonicalDeclarationKind,
        target_kind: CanonicalDeclarationKind,
    },
    /// The current bounded checker does not yet support this public declaration form.
    #[error("checked finalization does not support {kind:?} declaration {name:?} in {module}")]
    UnsupportedDefinition {
        module: ModuleKey,
        name: Box<str>,
        kind: CanonicalDeclarationKind,
        span: Span,
    },
    /// A public syntax-phase macro summary could not be collected or validated.
    #[error("macro summary checking failed for {module}::{name}: {reason}")]
    InvalidMacroSummary {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
        reason: Box<str>,
    },
    /// Signature checking rejected one callable.
    #[error("signature checking failed for {module}::{name}: {reason}")]
    Signature {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
        reason: Box<str>,
    },
    /// Body checking rejected one callable.
    #[error("body checking failed for {module}::{name}: {reason}")]
    Body {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
        reason: Box<str>,
    },
    /// Policy field defaults and invariants failed checked expression validation.
    #[error("policy checking failed for {module}::{name}: {reason}")]
    Policy {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
        reason: Box<str>,
    },
    /// A public signature mentions a private declaration.
    #[error("public export {module}::{name} mentions private declaration {dependency:?}")]
    PrivateExportDependency {
        module: ModuleKey,
        name: Box<str>,
        dependency: Box<str>,
        span: Span,
    },
    /// A public metadata declaration names no local or imported dependency.
    #[error("public export {module}::{name} names missing declaration {dependency:?}")]
    MissingPublicExportDependency {
        module: ModuleKey,
        name: Box<str>,
        dependency: Box<str>,
        span: Span,
    },
    /// A public namespace dependency graph contains a cycle.
    #[error("public export {module}::{name} has cyclic dependency {dependency:?}")]
    CyclicPublicExportDependency {
        module: ModuleKey,
        name: Box<str>,
        dependency: Box<str>,
        span: Span,
    },
    /// A staged public use does not target an export-closed declaration.
    #[error("public use {module}::{name:?} targets a non-exported declaration")]
    NonExportedPublicUse {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
    },
    /// A staged public use crosses a non-public defining module path.
    #[error(
        "public use {module}::{name:?} crosses a non-exported defining module path: {details:?}"
    )]
    NonExportedPublicUseModulePath {
        module: ModuleKey,
        name: Box<str>,
        defining_module: ModuleKey,
        details: Box<NonExportedPublicUseModulePathDetails>,
    },
    /// A staged public use would publish a duplicate local name.
    #[error("public export {module}::{name:?} is duplicated")]
    DuplicatePublicExport {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
    },
}

#[derive(Debug)]
enum CallableDefinition {
    Function(FnDef),
    Builtin(BuiltinFnDef),
    Handler(HandlerDef),
}

impl CallableDefinition {
    fn name(&self) -> &str {
        match self {
            Self::Function(function) => function.name.as_ref(),
            Self::Builtin(function) => function.name.as_ref(),
            Self::Handler(handler) => handler.name.as_ref(),
        }
    }
}

#[derive(Debug)]
struct ModuleStage {
    module_key: ModuleKey,
    origin: ModuleArtifactOrigin,
    raw_definitions: Vec<Definition>,
    definitions: Vec<CanonicalCheckedDeclaration>,
    callable_definitions: Vec<(CanonicalDeclarationIdentity, CallableDefinition)>,
    implementation_members: Vec<(CanonicalDeclarationIdentity, ImplementationMember)>,
}

#[derive(Debug)]
enum ImplementationMember {
    Method {
        implementation: ImplDef,
        method: ImplMethodDef,
    },
    Handler {
        handler: HandlerDef,
    },
}

/// Check and atomically publish all collected modules and staged public uses.
///
/// Only the internal collection entries supply declaration bodies and names.
/// The parsed-import result supplies already-resolved identities and spans; it
/// is never used to recover callable or type facts. The current executable
/// slice checks ordinary functions, bodyless builtins, and canonical handlers
/// with explicit public-signature dependency closure,
/// ordinary types, nominal newtypes, resource schemas, public interface
/// metadata, and nested constructors; unsupported public namespace forms are
/// rejected before publication.
///
/// # Errors
///
/// Returns an error before publishing any interface when the key sets, staged
/// identities, callable signatures/bodies, or public export closure disagree.
#[allow(clippy::result_large_err)]
pub fn finalize_canonical_module_collection(
    expanded: &CanonicalExpandedModuleGraph,
    collection: &CanonicalModuleCollection,
    imports: &CanonicalParsedImportResult,
) -> Result<CanonicalCheckedModuleFinalization, CanonicalCheckedModuleFinalizationError> {
    let expanded_keys = expanded
        .parsed_graph()
        .module_units()
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let collection_keys = collection
        .modules()
        .map(|module| module.module_key().clone())
        .collect::<Vec<_>>();
    if expanded_keys != collection_keys {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    }
    collection
        .revalidate_against(expanded)
        .map_err(|_| CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
    validate_import_binding_local_names(imports)?;
    validate_public_use_bindings(imports)?;

    let mut stages = Vec::new();
    for module in collection.modules() {
        let module_key = module.module_key().clone();
        let artifact = expanded
            .parsed_graph()
            .module_unit(&module_key)
            .ok_or_else(
                || CanonicalCheckedModuleFinalizationError::MissingArtifact {
                    module: module_key.clone(),
                },
            )?
            .artifact();
        let raw_definitions = module
            .internal_snapshot()
            .entries()
            .filter_map(|entry| entry.raw_definition().cloned())
            .fold(Vec::new(), |mut definitions, definition| {
                if !definitions.contains(&definition) {
                    definitions.push(definition);
                }
                definitions
            });
        let macro_summaries = collect_public_macro_summaries(
            &ModuleFile {
                definitions: raw_definitions.clone(),
                ..ModuleFile::default()
            },
            module_key.to_string(),
        )
        .map_err(|error| macro_summary_error(&module_key, &raw_definitions, error))?;
        let mut definitions = module
            .internal_snapshot()
            .entries()
            .map(|entry| {
                checked_declaration_skeleton(entry, artifact.origin().clone(), &macro_summaries)
            })
            .collect::<Vec<_>>();
        for declaration in &mut definitions {
            if !matches!(declaration.kind(), CanonicalDeclarationKind::ModuleDecl) {
                continue;
            }
            if let Some(module_declaration) = expanded
                .parsed_graph()
                .module_unit(&module_key)
                .into_iter()
                .flat_map(|unit| unit.body().module_decls())
                .find(|module_declaration| module_declaration.name.as_ref() == declaration.name())
            {
                declaration.visibility = module_declaration.visibility.clone();
            }
        }
        let callable_definitions = module
            .internal_snapshot()
            .entries()
            .filter_map(|entry| {
                let callable = match entry.raw_definition()? {
                    Definition::Function(function) => {
                        CallableDefinition::Function(function.clone())
                    }
                    Definition::BuiltinFn(function) => {
                        CallableDefinition::Builtin(function.clone())
                    }
                    Definition::Handler(handler) => CallableDefinition::Handler(handler.clone()),
                    _ => return None,
                };
                Some((entry.identity().clone(), callable))
            })
            .collect::<Vec<_>>();
        let implementation_members = module
            .internal_snapshot()
            .entries()
            .filter_map(|entry| {
                let Definition::Impl(implementation) = entry.raw_definition()? else {
                    return None;
                };
                let name = entry.declared_name()?;
                let member = match entry.kind() {
                    CanonicalDeclarationKind::Function => implementation
                        .methods
                        .iter()
                        .find(|method| method.name.as_ref() == name)
                        .cloned()
                        .map(|method| ImplementationMember::Method {
                            implementation: implementation.clone(),
                            method,
                        }),
                    CanonicalDeclarationKind::Handler => implementation
                        .handlers
                        .iter()
                        .find(|handler| handler.name.as_ref() == name)
                        .cloned()
                        .map(|handler| ImplementationMember::Handler { handler }),
                    _ => None,
                }?;
                Some((entry.identity().clone(), member))
            })
            .collect::<Vec<_>>();
        stages.push(ModuleStage {
            module_key,
            origin: artifact.origin().clone(),
            raw_definitions,
            definitions,
            callable_definitions,
            implementation_members,
        });
    }

    validate_import_binding_declaration_metadata(imports, &stages)?;
    validate_import_binding_module_visibility(expanded.parsed_graph(), imports)?;
    validate_import_binding_shapes(imports, &stages)?;
    validate_public_use_module_export_closure(imports, &stages)?;

    let imported_type_definitions = stages
        .iter()
        .map(|stage| imported_type_identity_definitions(stage, &stages, imports))
        .collect::<Vec<_>>();

    let mut signatures = Vec::<(CanonicalDeclarationIdentity, Type)>::new();
    for (stage_index, stage) in stages.iter().enumerate() {
        validate_public_declaration_support(stage)?;
        validate_structural_module_declarations(stage, &stages)?;
        validate_public_declaration_dependencies(stage, &stages, imports)?;
        validate_public_signatures(stage, &stages, imports)?;
        let mut environment =
            stage_type_environment(stage, &imported_type_definitions[stage_index])?;
        for (identity, callable) in &stage.callable_definitions {
            let (name, signature) = match callable {
                CallableDefinition::Function(function) => (
                    function.name.as_ref(),
                    fn_signature_type(&environment, function),
                ),
                CallableDefinition::Builtin(function) => (
                    function.name.as_ref(),
                    builtin_fn_signature_type(&environment, function),
                ),
                CallableDefinition::Handler(handler) => (
                    handler.name.as_ref(),
                    handler_signature_type(&environment, handler),
                ),
            };
            let signature = signature.map_err(|error| {
                signature_error(
                    &stage.module_key,
                    name,
                    callable_span(callable),
                    error.to_string(),
                )
            })?;
            environment.bind_variable(name, signature.clone());
            signatures.push((identity.clone(), signature));
        }
    }

    validate_import_targets(imports, &stages, &signatures)?;

    validate_public_effect_row_dependency_closure(&stages, imports)?;
    validate_public_type_alias_dependency_closure(&stages, imports)?;

    for (stage_index, stage) in stages.iter_mut().enumerate() {
        let mut environment =
            stage_type_environment(stage, &imported_type_definitions[stage_index])?;
        for (importing_module, _, binding) in imports.bindings() {
            if importing_module != &stage.module_key {
                continue;
            }
            let Some((_, signature)) = signatures
                .iter()
                .find(|(identity, _)| identity == binding.defining_identity())
            else {
                continue;
            };
            environment.bind_variable(binding.local_name(), signature.clone());
        }
        for (identity, callable) in &stage.callable_definitions {
            if let Some((_, signature)) = signatures
                .iter()
                .find(|(candidate, _)| candidate == identity)
            {
                environment.bind_variable(callable.name(), signature.clone());
            }
        }
        for definition in &stage.raw_definitions {
            if let Definition::Interface(interface) = definition {
                environment
                    .register_interface_laws(interface)
                    .map_err(|error| {
                        signature_error(
                            &stage.module_key,
                            "<interface-laws>",
                            interface.span,
                            error.to_string(),
                        )
                    })?;
            }
        }
        environment
            .register_module_laws(&stage.raw_definitions)
            .map_err(|error| {
                signature_error(
                    &stage.module_key,
                    "<laws>",
                    Span::default(),
                    error.to_string(),
                )
            })?;
        environment
            .register_module_proofs(&stage.raw_definitions)
            .map_err(|error| {
                signature_error(
                    &stage.module_key,
                    "<proofs>",
                    Span::default(),
                    error.to_string(),
                )
            })?;
        validate_policy_definitions(&environment, &stage.raw_definitions, &stage.module_key)?;
        for (identity, member) in &stage.implementation_members {
            let (signature, body_type, body_span, name) = match member {
                ImplementationMember::Method {
                    implementation,
                    method,
                } => {
                    let (signature, body_type) = check_implementation_method_body_in_env(
                        &environment,
                        implementation,
                        method,
                    )
                    .map_err(|reason| {
                        CanonicalCheckedModuleFinalizationError::Body {
                            module: stage.module_key.clone(),
                            name: method.name.to_string().into_boxed_str(),
                            span: method.span,
                            reason,
                        }
                    })?;
                    (
                        signature,
                        body_type,
                        method.body.span(),
                        method.name.clone(),
                    )
                }
                ImplementationMember::Handler { handler } => {
                    let signature =
                        handler_signature_type(&environment, handler).map_err(|error| {
                            CanonicalCheckedModuleFinalizationError::Body {
                                module: stage.module_key.clone(),
                                name: handler.name.to_string().into_boxed_str(),
                                span: handler.span,
                                reason: error.to_string().into_boxed_str(),
                            }
                        })?;
                    let mut handler_environment = environment.clone();
                    handler_environment.bind_variable(handler.name.as_ref(), signature.clone());
                    let mut definitions = stage.raw_definitions.clone();
                    definitions.push(Definition::Handler(handler.clone()));
                    let body_type =
                        check_handler_body_in_env(&handler_environment, &definitions, handler)
                            .map_err(|error| CanonicalCheckedModuleFinalizationError::Body {
                                module: stage.module_key.clone(),
                                name: handler.name.to_string().into_boxed_str(),
                                span: handler.span,
                                reason: error.to_string().into_boxed_str(),
                            })?;
                    (
                        signature,
                        body_type,
                        handler.body.span(),
                        handler.name.clone(),
                    )
                }
            };
            let declaration = stage
                .definitions
                .iter_mut()
                .find(|declaration| declaration.identity() == identity)
                .ok_or_else(|| {
                    CanonicalCheckedModuleFinalizationError::MissingCheckedDeclaration {
                        module: stage.module_key.clone(),
                        name: name.to_string().into_boxed_str(),
                        span: body_span,
                    }
                })?;
            declaration.signature = Some(signature);
            declaration.body_span = Some(body_span);
            declaration.body_type = Some(body_type);
        }
        for declaration in &mut stage.definitions {
            declaration.signature = signatures
                .iter()
                .find(|(identity, _)| identity == &declaration.identity)
                .map(|(_, signature)| signature.clone());
        }
        for (identity, callable) in &stage.callable_definitions {
            let Some(declaration) = stage
                .definitions
                .iter_mut()
                .find(|declaration| declaration.identity() == identity)
            else {
                continue;
            };
            let body_type = match callable {
                CallableDefinition::Function(function) => {
                    check_function_body_in_env(&environment, function).map_err(|error| {
                        CanonicalCheckedModuleFinalizationError::Body {
                            module: stage.module_key.clone(),
                            name: function.name.to_string().into_boxed_str(),
                            span: function.body.span(),
                            reason: error.to_string().into_boxed_str(),
                        }
                    })?
                }
                CallableDefinition::Handler(handler) => {
                    check_handler_body_in_env(&environment, &stage.raw_definitions, handler)
                        .map_err(|error| CanonicalCheckedModuleFinalizationError::Body {
                            module: stage.module_key.clone(),
                            name: handler.name.to_string().into_boxed_str(),
                            span: handler.body.span(),
                            reason: error.to_string().into_boxed_str(),
                        })?
                }
                CallableDefinition::Builtin(_) => continue,
            };
            declaration.body_span = Some(match callable {
                CallableDefinition::Function(function) => function.body.span(),
                CallableDefinition::Handler(handler) => handler.body.span(),
                CallableDefinition::Builtin(_) => unreachable!(),
            });
            declaration.body_type = Some(body_type);
        }
    }

    for stage in &stages {
        validate_public_signatures(stage, &stages, imports)?;
    }

    let mut interfaces = stages
        .into_iter()
        .map(|stage| {
            let mut public_exports = BTreeMap::new();
            for declaration in &stage.definitions {
                if declaration.is_exported() {
                    let public_declaration =
                        CanonicalCheckedPublicDeclaration::from_private(declaration);
                    public_exports.insert(
                        (public_declaration.namespace(), declaration.name.clone()),
                        CanonicalCheckedExport {
                            local_name: declaration.name.clone(),
                            declaration: public_declaration,
                            import_span: None,
                        },
                    );
                }
            }
            (
                stage.module_key.clone(),
                CanonicalCheckedModuleInterface {
                    module_key: stage.module_key,
                    origin: stage.origin,
                    private_declarations: stage.definitions.into_boxed_slice(),
                    public_exports,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    for public_use in imports
        .public_uses()
        .iter()
        .filter(|public_use| is_public_reexport(public_use.binding()))
    {
        let importing_module = public_use.importing_module();
        let binding = public_use.binding();
        let target = interfaces
            .values()
            .flat_map(|interface| interface.private_declarations())
            .find(|declaration| declaration.identity() == binding.defining_identity())
            .ok_or_else(
                || CanonicalCheckedModuleFinalizationError::MissingBindingTarget {
                    module: importing_module.clone(),
                    name: binding.local_name().into(),
                },
            )?;
        if !target.is_exported() {
            return Err(
                CanonicalCheckedModuleFinalizationError::NonExportedPublicUse {
                    module: importing_module.clone(),
                    name: binding.local_name().into(),
                    span: binding.use_span(),
                },
            );
        }
        let target = target.clone();
        let interface = interfaces
            .get_mut(importing_module)
            .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
        let export_key = (target.namespace(), binding.local_name().into());
        if interface.public_exports.contains_key(&export_key) {
            return Err(
                CanonicalCheckedModuleFinalizationError::DuplicatePublicExport {
                    module: importing_module.clone(),
                    name: binding.local_name().into(),
                    span: binding.use_span(),
                },
            );
        }
        interface.public_exports.insert(
            export_key,
            CanonicalCheckedExport {
                local_name: binding.local_name().into(),
                declaration: CanonicalCheckedPublicDeclaration::from_private(&target),
                import_span: Some(binding.use_span()),
            },
        );
    }

    Ok(CanonicalCheckedModuleFinalization {
        modules: interfaces,
    })
}

fn validate_public_declaration_support(
    stage: &ModuleStage,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for declaration in &stage.definitions {
        match declaration.fact() {
            CanonicalCheckedDeclarationFact::Constructor { parent, .. } => {
                validate_public_constructor_parent(stage, declaration, parent, false)?;
            }
            CanonicalCheckedDeclarationFact::SealedDomainConstructor { parent, .. } => {
                validate_public_constructor_parent(stage, declaration, parent, true)?;
            }
            _ => {}
        }
    }

    for declaration in &stage.definitions {
        if !declaration.is_exported() {
            continue;
        }
        let supported = match declaration.kind() {
            CanonicalDeclarationKind::ModuleDecl => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::StructuralModule { .. }
            ),
            CanonicalDeclarationKind::Handler | CanonicalDeclarationKind::BuiltinFn => stage
                .callable_definitions
                .iter()
                .any(|(identity, _)| identity == declaration.identity()),
            CanonicalDeclarationKind::Function => {
                stage
                    .callable_definitions
                    .iter()
                    .any(|(identity, _)| identity == declaration.identity())
                    || matches!(
                        declaration.fact(),
                        CanonicalCheckedDeclarationFact::Constructor { .. }
                    )
            }
            CanonicalDeclarationKind::Type
            | CanonicalDeclarationKind::Newtype
            | CanonicalDeclarationKind::ResourceType => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Type { .. }
                    | CanonicalCheckedDeclarationFact::Newtype { .. }
                    | CanonicalCheckedDeclarationFact::ResourceType { .. }
            ),
            CanonicalDeclarationKind::Interface => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Interface { .. }
            ),
            CanonicalDeclarationKind::Impl => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Implementation { .. }
            ),
            CanonicalDeclarationKind::EffectAlias => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::EffectAlias { .. }
            ),
            CanonicalDeclarationKind::EffectGroup => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::EffectGroup { .. }
            ),
            CanonicalDeclarationKind::DataKind => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::DataKind { .. }
            ),
            CanonicalDeclarationKind::PropositionPredicate => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::PropositionPredicate { .. }
            ),
            CanonicalDeclarationKind::Role => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Role { .. }
            ),
            CanonicalDeclarationKind::Policy => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Policy { .. }
            ),
            CanonicalDeclarationKind::TypeFn => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::TypeFn { .. }
            ),
            CanonicalDeclarationKind::Macro => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Macro { .. }
            ),
            CanonicalDeclarationKind::Law => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Law { .. }
            ),
            CanonicalDeclarationKind::Proof => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Proof { .. }
            ),
            CanonicalDeclarationKind::Notation => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Notation { .. }
            ),
            CanonicalDeclarationKind::SealedDomain => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::SealedDomain { .. }
                    | CanonicalCheckedDeclarationFact::SealedDomainConstructor { .. }
            ),
            _ => matches!(
                declaration.fact(),
                CanonicalCheckedDeclarationFact::Constructor { .. }
            ),
        };
        if !supported {
            return Err(
                CanonicalCheckedModuleFinalizationError::UnsupportedDefinition {
                    module: stage.module_key.clone(),
                    name: declaration.name.clone(),
                    kind: declaration.kind(),
                    span: declaration.declaration_span(),
                },
            );
        }
    }
    Ok(())
}

fn validate_structural_module_declarations(
    stage: &ModuleStage,
    stages: &[ModuleStage],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for declaration in &stage.definitions {
        if declaration.kind() != CanonicalDeclarationKind::ModuleDecl {
            continue;
        }
        let CanonicalCheckedDeclarationFact::StructuralModule { module: child } =
            declaration.fact()
        else {
            return Err(
                CanonicalCheckedModuleFinalizationError::UnsupportedDefinition {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    kind: declaration.kind(),
                    span: declaration.declaration_span(),
                },
            );
        };
        let expected_child = stage
            .module_key
            .child(declaration.name())
            .map_err(|_| CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
        if child != &expected_child {
            return Err(
                CanonicalCheckedModuleFinalizationError::StructuralModuleIdentityMismatch {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    child: Box::new(child.clone()),
                    expected: Box::new(expected_child),
                    span: declaration.declaration_span(),
                },
            );
        }
        if !stages
            .iter()
            .any(|candidate| candidate.module_key == *child)
        {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingStructuralModuleTarget {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    child: Box::new(child.clone()),
                    span: declaration.declaration_span(),
                },
            );
        }
    }
    Ok(())
}

fn validate_public_declaration_dependencies(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let builtin_types = TypeEnv::with_builtin_types();
    for declaration in &stage.definitions {
        if !declaration.is_exported() {
            continue;
        }

        let mut dependencies = Vec::new();
        match declaration.fact() {
            CanonicalCheckedDeclarationFact::Type { params, body, .. } => {
                let mut type_dependencies = Vec::new();
                collect_surface_type_body_names(body, &mut type_dependencies);
                let type_parameters = params
                    .iter()
                    .map(ToString::to_string)
                    .collect::<HashSet<_>>();
                type_dependencies.retain(|name| !type_parameters.contains(name));
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
            }
            CanonicalCheckedDeclarationFact::Newtype {
                type_params,
                representation,
                ..
            } => {
                let mut type_dependencies = Vec::new();
                collect_surface_type_names(representation, &mut type_dependencies);
                let type_parameters = type_params
                    .iter()
                    .map(|parameter| parameter.name.to_string())
                    .collect::<HashSet<_>>();
                type_dependencies.retain(|name| !type_parameters.contains(name));
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
            }
            CanonicalCheckedDeclarationFact::ResourceType { fields } => {
                let mut type_dependencies = Vec::new();
                for (_, field_type) in fields.iter() {
                    collect_surface_type_names(field_type, &mut type_dependencies);
                }
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
            }
            CanonicalCheckedDeclarationFact::Interface { definition, .. } => {
                let mut type_dependencies = Vec::new();
                collect_interface_type_names(definition, &mut type_dependencies);
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
                let mut value_dependencies = Vec::new();
                for law in &definition.laws {
                    collect_expr_dependency_names(&law.proposition, &mut value_dependencies);
                }
                validate_public_interface_law_value_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &value_dependencies,
                )?;
            }
            CanonicalCheckedDeclarationFact::Implementation { summary } => {
                let type_parameters = summary
                    .type_params()
                    .iter()
                    .map(|parameter| parameter.to_string())
                    .collect::<HashSet<_>>();
                let mut type_dependencies = Vec::new();
                for type_argument in summary.type_args() {
                    collect_surface_type_names(type_argument, &mut type_dependencies);
                }
                for (_, associated_type) in summary.associated_types() {
                    collect_surface_type_names(associated_type, &mut type_dependencies);
                }
                type_dependencies.retain(|name| !type_parameters.contains(name));
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                validate_public_namespace_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    summary.interface(),
                    CanonicalNamespace::Interface,
                    declaration.declaration_span(),
                )?;
                for (_, bound) in summary.where_bounds() {
                    validate_public_namespace_dependency(
                        stage,
                        stages,
                        imports,
                        declaration,
                        bound,
                        CanonicalNamespace::Interface,
                        declaration.declaration_span(),
                    )?;
                }
                validate_public_implementation_proof_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &type_parameters,
                )?;
                dependencies.push(summary.interface().to_owned());
                dependencies.extend(type_dependencies);
            }
            CanonicalCheckedDeclarationFact::EffectAlias { definition } => {
                validate_effect_row_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &definition.row,
                    &mut dependencies,
                )?;
            }
            CanonicalCheckedDeclarationFact::EffectGroup { definition } => {
                validate_effect_row_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &definition.row,
                    &mut dependencies,
                )?;
            }
            CanonicalCheckedDeclarationFact::DataKind { definition } => {
                validate_public_namespace_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &definition.source_adt,
                    CanonicalNamespace::TypeDomain,
                    declaration.declaration_span(),
                )?;
                dependencies.push(definition.source_adt.to_string());
            }
            CanonicalCheckedDeclarationFact::PropositionPredicate { definition } => {
                let mut type_dependencies = Vec::new();
                for parameter in &definition.params {
                    collect_surface_type_names(&parameter.domain, &mut type_dependencies);
                }
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
            }
            CanonicalCheckedDeclarationFact::Role { .. } => {}
            CanonicalCheckedDeclarationFact::Policy { definition } => {
                let mut value_dependencies = Vec::new();
                let type_parameters = definition
                    .type_params
                    .iter()
                    .map(|parameter| parameter.to_string())
                    .collect::<HashSet<_>>();
                for field in &definition.fields {
                    let mut field_type_dependencies = Vec::new();
                    collect_surface_type_names(&field.ty, &mut field_type_dependencies);
                    field_type_dependencies.retain(|name| !type_parameters.contains(name));
                    validate_public_type_dependencies(
                        stage,
                        stages,
                        imports,
                        declaration,
                        &builtin_types,
                        &field_type_dependencies,
                    )?;
                    dependencies.extend(field_type_dependencies);
                    if let Some(default) = &field.default {
                        collect_expr_dependency_names(default, &mut value_dependencies);
                    }
                }
                if let Some(where_clause) = &definition.where_clause {
                    collect_expr_dependency_names(where_clause, &mut value_dependencies);
                }
                validate_public_expression_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &value_dependencies,
                )?;
            }
            CanonicalCheckedDeclarationFact::TypeFn { definition } => {
                let mut type_dependencies = Vec::new();
                for parameter in &definition.params {
                    collect_surface_type_names(&parameter.ty, &mut type_dependencies);
                }
                collect_surface_type_names(&definition.return_type, &mut type_dependencies);
                for equation in &definition.equations {
                    for pattern in &equation.patterns {
                        collect_type_pattern_names(pattern, &mut type_dependencies);
                    }
                    collect_surface_type_names(&equation.result, &mut type_dependencies);
                }
                if let Some(tail) = &definition.proposition_tail {
                    validate_public_proposition_tail_dependencies(
                        stage,
                        stages,
                        imports,
                        declaration,
                        tail,
                        &mut type_dependencies,
                        &mut dependencies,
                    )?;
                }
                let type_parameters = definition
                    .params
                    .iter()
                    .map(|parameter| parameter.name.to_string())
                    .collect::<HashSet<_>>();
                type_dependencies.retain(|name| !type_parameters.contains(name));
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
            }
            CanonicalCheckedDeclarationFact::Notation { definition } => {
                if let Some(module) = &definition.target.module {
                    validate_public_qualified_namespace_dependency(
                        stage,
                        stages,
                        declaration,
                        &[module.clone(), definition.target.name.clone()],
                        CanonicalNamespace::ValueCallable,
                        definition.target.span,
                    )?;
                } else {
                    validate_public_namespace_dependency(
                        stage,
                        stages,
                        imports,
                        declaration,
                        &definition.target.name,
                        CanonicalNamespace::ValueCallable,
                        declaration.declaration_span(),
                    )?;
                }
                dependencies.push(definition.target.name.to_string());
            }
            CanonicalCheckedDeclarationFact::Macro { summary } => {
                let mut value_dependencies = Vec::new();
                let mut type_dependencies = Vec::new();
                if let Some(signature) = &summary.typed_signature {
                    for parameter_type in signature.param_types.iter().flatten() {
                        collect_surface_type_names(parameter_type, &mut type_dependencies);
                    }
                    if let Some(return_type) = &signature.return_type {
                        collect_surface_type_names(return_type, &mut type_dependencies);
                    }
                }
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
                if let Some(Definition::Macro(definition)) =
                    stage.raw_definitions.iter().find(|candidate| {
                        matches!(
                            candidate,
                            Definition::Macro(definition)
                                if definition.name.as_ref() == declaration.name()
                        )
                    })
                {
                    ash_parser::surface::visit_expr(&definition.body, &mut |expression| {
                        match expression {
                            ash_parser::surface::Expr::Call {
                                func,
                                module: Some(implementation),
                                ..
                            } => value_dependencies.push(
                                PublicExpressionDependency::Implementation {
                                    implementation: implementation.clone(),
                                    operation: func.clone(),
                                },
                            ),
                            ash_parser::surface::Expr::Call {
                                func, module: None, ..
                            }
                            | ash_parser::surface::Expr::Constructor { name: func, .. } => {
                                value_dependencies.push(PublicExpressionDependency::Value(
                                    func.to_string().into_boxed_str(),
                                ));
                            }
                            ash_parser::surface::Expr::MacroInvocation { invocation } => {
                                dependencies.push(invocation.name.to_string());
                            }
                            _ => {}
                        }
                    });
                }
                validate_public_expression_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &value_dependencies,
                )?;
            }
            CanonicalCheckedDeclarationFact::Law { definition } => {
                let mut value_dependencies = Vec::new();
                let mut type_dependencies = Vec::new();
                for parameter in &definition.params {
                    collect_surface_type_names(&parameter.ty, &mut type_dependencies);
                }
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
                collect_expr_dependency_names(&definition.proposition, &mut value_dependencies);
                validate_public_expression_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &value_dependencies,
                )?;
            }
            CanonicalCheckedDeclarationFact::Proof { definition } => {
                let mut value_dependencies = Vec::new();
                let mut type_dependencies = Vec::new();
                for parameter in &definition.params {
                    collect_surface_type_names(&parameter.ty, &mut type_dependencies);
                }
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
                match &definition.body {
                    ash_parser::surface::ProofBody::Expr(expression) => {
                        collect_expr_dependency_names(expression, &mut value_dependencies);
                    }
                    ash_parser::surface::ProofBody::ByTestProperty { strategies } => {
                        for strategy in strategies {
                            collect_expr_dependency_names(
                                &strategy.strategy_expr,
                                &mut value_dependencies,
                            );
                        }
                    }
                    ash_parser::surface::ProofBody::ByDefinition
                    | ash_parser::surface::ProofBody::ByTest { .. }
                    | ash_parser::surface::ProofBody::ByTestSmallWorld => {}
                }
                validate_public_expression_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &value_dependencies,
                )?;
            }
            CanonicalCheckedDeclarationFact::SealedDomain { definition } => {
                let mut type_dependencies = Vec::new();
                for constructor in &definition.constructors {
                    for field in &constructor.fields {
                        if let ash_parser::surface::DomainSlot::DomainRef(name) = &field.slot {
                            type_dependencies.push(name.to_string());
                        }
                    }
                }
                validate_public_type_dependencies(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &builtin_types,
                    &type_dependencies,
                )?;
                dependencies.extend(type_dependencies);
            }
            CanonicalCheckedDeclarationFact::Constructor { .. }
            | CanonicalCheckedDeclarationFact::SealedDomainConstructor { .. }
            | CanonicalCheckedDeclarationFact::StructuralModule { .. } => {}
            CanonicalCheckedDeclarationFact::Opaque => {}
        }

        for dependency in dependencies {
            if let Some(private) = stage
                .definitions
                .iter()
                .find(|candidate| candidate.name() == dependency)
                && !private.is_exported()
            {
                return Err(
                    CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                        module: stage.module_key.clone(),
                        name: declaration.name().into(),
                        dependency: dependency.into_boxed_str(),
                        span: declaration.declaration_span(),
                    },
                );
            }
        }
    }
    Ok(())
}

/// Validate the parent identity carried by a public parent-scoped constructor.
///
/// Ordinary and newtype constructors are exported as value entries only when
/// their parent type is itself present and publicly reachable. Sealed-domain
/// constructors remain parent-scoped and are never standalone public exports,
/// but a forged public carrier must still agree with its checked parent.
fn validate_public_constructor_parent(
    stage: &ModuleStage,
    declaration: &CanonicalCheckedDeclaration,
    parent: &CanonicalDeclarationIdentity,
    sealed_domain: bool,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    if declaration.identity().canonical_parent() != Some(parent)
        || parent.module_key() != &stage.module_key
        || parent.canonical_parent().is_some()
    {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    }

    let Some(parent_declaration) = stage
        .definitions
        .iter()
        .find(|candidate| candidate.identity() == parent)
    else {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    };
    let valid_parent_kind = if sealed_domain {
        declaration.kind() == CanonicalDeclarationKind::SealedDomain
            && declaration.namespace() == CanonicalNamespace::TypeDomain
            && parent_declaration.kind() == CanonicalDeclarationKind::SealedDomain
    } else {
        declaration.kind() == CanonicalDeclarationKind::Function
            && declaration.namespace() == CanonicalNamespace::ValueCallable
            && matches!(
                parent_declaration.kind(),
                CanonicalDeclarationKind::Type | CanonicalDeclarationKind::Newtype
            )
    };
    if !valid_parent_kind || parent_declaration.namespace() != CanonicalNamespace::TypeDomain {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    }
    let constructor_name = match declaration.fact() {
        CanonicalCheckedDeclarationFact::Constructor { name, .. } => name.as_ref(),
        CanonicalCheckedDeclarationFact::SealedDomainConstructor { constructor, .. } => {
            constructor.name.as_ref()
        }
        _ => return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch),
    };
    if constructor_name != declaration.name()
        || !parent_contains_constructor(parent_declaration, constructor_name, sealed_domain)
    {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    }
    if declaration.is_exported() && !parent_declaration.is_exported() {
        return Err(
            CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: parent_declaration.name().into(),
                span: declaration.declaration_span(),
            },
        );
    }
    Ok(())
}

fn parent_contains_constructor(
    parent: &CanonicalCheckedDeclaration,
    constructor_name: &str,
    sealed_domain: bool,
) -> bool {
    match parent.fact() {
        CanonicalCheckedDeclarationFact::Type {
            body: TypeBody::Enum(variants),
            ..
        } if !sealed_domain => variants
            .iter()
            .any(|variant| variant.name.as_ref() == constructor_name),
        CanonicalCheckedDeclarationFact::Newtype { constructor, .. } if !sealed_domain => {
            constructor.as_ref() == constructor_name
        }
        CanonicalCheckedDeclarationFact::SealedDomain { definition } if sealed_domain => definition
            .constructors
            .iter()
            .any(|constructor| constructor.name.as_ref() == constructor_name),
        _ => false,
    }
}

/// Validate that one named namespace dependency is publicly reachable.
///
/// Internal imports may legally access `pub(crate)` or restricted declarations,
/// but those declarations cannot satisfy a public interface dependency. Keep
/// this check separate from type lowering so row, promoted-kind, notation, and
/// interface metadata use the same export-closure rule without acquiring
/// runtime or policy authority.
fn validate_public_namespace_dependency(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    dependency: &str,
    namespace: CanonicalNamespace,
    span: Span,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    if validate_public_namespace_dependency_if_present(
        stage,
        stages,
        imports,
        declaration,
        dependency,
        namespace,
        span,
    )? {
        return Ok(());
    }

    Err(
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: dependency.to_owned().into_boxed_str(),
            span,
        },
    )
}

/// Validate a qualified dependency against the checker-owned module stages.
///
/// Qualified row and notation paths do not create ordinary parsed-import
/// bindings, so they cannot be checked through the local binding map. Resolve
/// their canonical module identity from the staged declarations instead and
/// require both the structural path and the target declaration to be public.
fn validate_public_qualified_namespace_dependency(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    declaration: &CanonicalCheckedDeclaration,
    path: &[Box<str>],
    namespace: CanonicalNamespace,
    span: Span,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    resolve_public_qualified_namespace_dependency(stage, stages, declaration, path, namespace, span)
        .map(|_| ())
}

/// Resolve one qualified public namespace dependency and validate every
/// enclosing module declaration on its path.
fn resolve_public_qualified_namespace_dependency<'a>(
    stage: &ModuleStage,
    stages: &'a [ModuleStage],
    declaration: &CanonicalCheckedDeclaration,
    path: &[Box<str>],
    namespace: CanonicalNamespace,
    span: Span,
) -> Result<
    (&'a ModuleStage, &'a CanonicalCheckedDeclaration),
    CanonicalCheckedModuleFinalizationError,
> {
    let Some((dependency, path_prefix)) = path.split_last() else {
        return Err(
            CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: "<empty>".into(),
                span,
            },
        );
    };
    let module_segments = if path_prefix
        .first()
        .is_some_and(|segment| segment.as_ref() == "crate")
    {
        &path_prefix[1..]
    } else {
        path_prefix
    };
    let target_stage = if module_segments.is_empty() {
        None
    } else if path_prefix
        .first()
        .is_some_and(|segment| segment.as_ref() == "crate")
    {
        stages.iter().find(|candidate| {
            candidate
                .module_key
                .segments()
                .iter()
                .map(String::as_str)
                .eq(module_segments.iter().map(|segment| segment.as_ref()))
        })
    } else {
        let matches = stages
            .iter()
            .filter(|candidate| {
                candidate.module_key.segments().len() >= module_segments.len()
                    && candidate
                        .module_key
                        .segments()
                        .iter()
                        .rev()
                        .zip(module_segments.iter().rev())
                        .all(|(candidate, requested)| candidate.as_str() == requested.as_ref())
            })
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| matches[0])
    };
    let Some(target_stage) = target_stage else {
        return Err(
            CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.clone(),
                span,
            },
        );
    };

    for (depth, segment) in target_stage.module_key.segments().iter().enumerate() {
        let Some(parent) = target_stage
            .module_key
            .segments()
            .get(..depth)
            .and_then(|segments| module_key_with_segments(&target_stage.module_key, segments))
        else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.clone(),
                    span,
                },
            );
        };
        let Some(parent_stage) = stages
            .iter()
            .find(|candidate| candidate.module_key == parent)
        else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.clone(),
                    span,
                },
            );
        };
        let child_identity = target_stage
            .module_key
            .segments()
            .get(..=depth)
            .and_then(|segments| module_key_with_segments(&target_stage.module_key, segments));
        let Some(child_identity) = child_identity else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.clone(),
                    span,
                },
            );
        };
        let Some(child) = parent_stage.definitions.iter().find(|candidate| {
            candidate.kind() == CanonicalDeclarationKind::ModuleDecl
                && candidate.name() == segment
                && candidate.identity().module_key() == &child_identity
        }) else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.clone(),
                    span,
                },
            );
        };
        if !child.is_exported() {
            return Err(
                CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.clone(),
                    span,
                },
            );
        }
    }

    let Some(target) = target_stage.definitions.iter().find(|candidate| {
        candidate.namespace() == namespace && candidate.name() == dependency.as_ref()
    }) else {
        return Err(
            CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.clone(),
                span,
            },
        );
    };
    if !target.is_exported() {
        return Err(
            CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.clone(),
                span,
            },
        );
    }
    Ok((target_stage, target))
}

fn module_key_with_segments(module: &ModuleKey, segments: &[String]) -> Option<ModuleKey> {
    let mut root = module.clone();
    while let Some(parent) = root.parent() {
        root = parent;
    }
    let mut key = root;
    for segment in segments {
        key = key.child(segment).ok()?;
    }
    Some(key)
}

/// Validate a namespace dependency when the source expression names one.
///
/// Row metadata may also mention checker-owned variables or external resource
/// operations that do not have a declaration/import entry in this stage. Those
/// names remain with the existing row checker; only a resolved local or
/// imported declaration participates in this optional namespace closure.
fn validate_public_namespace_dependency_if_present(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    dependency: &str,
    namespace: CanonicalNamespace,
    span: Span,
) -> Result<bool, CanonicalCheckedModuleFinalizationError> {
    if let Some(local) = stage
        .definitions
        .iter()
        .find(|candidate| candidate.namespace() == namespace && candidate.name() == dependency)
    {
        if !local.is_exported() {
            return Err(
                CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.to_owned().into_boxed_str(),
                    span,
                },
            );
        }
        return Ok(true);
    }

    if let Some((_, _, binding)) = imports.bindings().find(|(module, _, binding)| {
        *module == &stage.module_key
            && binding.lookup_key().namespace() == namespace
            && binding.local_name() == dependency
    }) {
        if !matches!(
            binding.declaration_visibility(),
            ash_parser::surface::Visibility::Public
        ) {
            return Err(
                CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.to_owned().into_boxed_str(),
                    span,
                },
            );
        }
        validate_public_defining_module_path(
            stage,
            stages,
            declaration,
            binding.defining_identity().module_key(),
            dependency,
            span,
        )?;
        return Ok(true);
    }

    Ok(false)
}

fn validate_public_expression_dependencies(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    dependencies: &[PublicExpressionDependency],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for dependency in dependencies {
        match dependency {
            PublicExpressionDependency::Value(dependency) => {
                validate_public_namespace_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    dependency,
                    CanonicalNamespace::ValueCallable,
                    declaration.declaration_span(),
                )?;
            }
            PublicExpressionDependency::Implementation {
                implementation,
                operation,
            } => {
                validate_public_expression_implementation_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    implementation,
                    operation,
                    declaration.declaration_span(),
                )?;
            }
        }
    }
    Ok(())
}

fn validate_public_interface_law_value_dependencies(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    dependencies: &[PublicExpressionDependency],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for dependency in dependencies {
        match dependency {
            PublicExpressionDependency::Value(dependency) => {
                let is_parent_scoped_method = stage.definitions.iter().any(|candidate| {
                    candidate.identity().canonical_parent() == Some(declaration.identity())
                        && candidate.namespace() == CanonicalNamespace::ValueCallable
                        && candidate.name() == dependency.as_ref()
                });
                if is_parent_scoped_method {
                    continue;
                }
                validate_public_namespace_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    dependency,
                    CanonicalNamespace::ValueCallable,
                    declaration.declaration_span(),
                )?;
            }
            PublicExpressionDependency::Implementation {
                implementation,
                operation,
            } => {
                let is_parent_scoped_method = implementation.as_ref() == declaration.name()
                    && stage.definitions.iter().any(|candidate| {
                        candidate.identity().canonical_parent() == Some(declaration.identity())
                            && candidate.namespace() == CanonicalNamespace::ValueCallable
                            && candidate.name() == operation.as_ref()
                    });
                if is_parent_scoped_method {
                    continue;
                }
                validate_public_expression_implementation_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    implementation,
                    operation,
                    declaration.declaration_span(),
                )?;
            }
        }
    }
    Ok(())
}

/// Validate a qualified implementation operation used by a public expression.
///
/// Unlike qualified operation rows, expression calls are export-bearing and
/// therefore cannot defer an unresolved implementation to the checker.  The
/// implementation itself must be publicly reachable, and the operation must
/// be present as a parent-scoped member of that implementation.  Imported
/// implementations additionally retain the defining-module-path check used by
/// the ordinary namespace closure.
fn validate_public_expression_implementation_dependency(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    implementation: &str,
    operation: &str,
    span: Span,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let missing_implementation =
        || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: implementation.to_owned().into_boxed_str(),
            span,
        };
    let private_implementation =
        || CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: implementation.to_owned().into_boxed_str(),
            span,
        };
    let missing_operation =
        || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: format!("{implementation}::{operation}").into_boxed_str(),
            span,
        };

    let (target_stage, target) = if let Some(target) = stage.definitions.iter().find(|candidate| {
        candidate.namespace() == CanonicalNamespace::ImplementationRegistry
            && candidate.name() == implementation
    }) {
        (stage, target)
    } else if let Some((_, _, binding)) = imports.bindings().find(|(module, _, binding)| {
        *module == &stage.module_key
            && binding.lookup_key().namespace() == CanonicalNamespace::ImplementationRegistry
            && binding.local_name() == implementation
    }) {
        if !matches!(
            binding.declaration_visibility(),
            ash_parser::surface::Visibility::Public
        ) {
            return Err(private_implementation());
        }
        validate_public_defining_module_path(
            stage,
            stages,
            declaration,
            binding.defining_identity().module_key(),
            implementation,
            span,
        )?;
        let Some((target_stage, target)) = stages.iter().find_map(|candidate_stage| {
            candidate_stage
                .definitions
                .iter()
                .find(|candidate| {
                    candidate.identity() == binding.defining_identity()
                        && candidate.name() == binding.lookup_key().visible_local_key()
                        && candidate.namespace() == CanonicalNamespace::ImplementationRegistry
                })
                .map(|target| (candidate_stage, target))
        }) else {
            return Err(missing_implementation());
        };
        (target_stage, target)
    } else {
        return Err(missing_implementation());
    };

    if !target.is_exported() {
        return Err(private_implementation());
    }

    if target_stage.definitions.iter().any(|candidate| {
        candidate.identity().canonical_parent() == Some(target.identity())
            && candidate.namespace() == CanonicalNamespace::ValueCallable
            && candidate.name() == operation
    }) {
        Ok(())
    } else {
        Err(missing_operation())
    }
}

/// Validate dependencies carried by proofs nested under a public implementation.
///
/// Proofs remain parent-scoped summary metadata, but their checked parameter types and
/// value-bearing proof terms are part of the public implementation's export surface. They must
/// therefore satisfy the same closure boundary as a standalone public evidence declaration.
fn validate_public_implementation_proof_dependencies(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    implementation: &CanonicalCheckedDeclaration,
    implementation_type_parameters: &HashSet<String>,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for proof in stage.definitions.iter().filter(|candidate| {
        candidate.kind() == CanonicalDeclarationKind::Proof
            && candidate.identity().canonical_parent() == Some(implementation.identity())
    }) {
        let CanonicalCheckedDeclarationFact::Proof { definition } = proof.fact() else {
            return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
        };

        let mut type_dependencies = Vec::new();
        for parameter in &definition.params {
            collect_surface_type_names(&parameter.ty, &mut type_dependencies);
        }
        type_dependencies.retain(|name| !implementation_type_parameters.contains(name));
        validate_public_type_dependencies(
            stage,
            stages,
            imports,
            proof,
            &TypeEnv::with_builtin_types(),
            &type_dependencies,
        )?;

        let mut value_dependencies = Vec::new();
        for constraint in &definition.constraints {
            for argument in &constraint.predicate.args {
                collect_expr_dependency_names(argument, &mut value_dependencies);
            }
        }
        match &definition.body {
            ash_parser::surface::ProofBody::Expr(expression) => {
                collect_expr_dependency_names(expression, &mut value_dependencies);
            }
            ash_parser::surface::ProofBody::ByTestProperty { strategies } => {
                for strategy in strategies {
                    collect_expr_dependency_names(&strategy.strategy_expr, &mut value_dependencies);
                }
            }
            ash_parser::surface::ProofBody::ByDefinition
            | ash_parser::surface::ProofBody::ByTest { .. }
            | ash_parser::surface::ProofBody::ByTestSmallWorld => {}
        }
        validate_public_expression_dependencies(
            stage,
            stages,
            imports,
            proof,
            &value_dependencies,
        )?;
    }
    Ok(())
}

fn validate_public_type_dependencies(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    builtins: &TypeEnv,
    dependencies: &[String],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for dependency in dependencies {
        if dependency == "Prop" || builtins.resolve_type(dependency).is_ok() {
            continue;
        }

        if let Some(local) = stage.definitions.iter().find(|candidate| {
            candidate.name() == dependency
                && matches!(
                    candidate.namespace(),
                    CanonicalNamespace::TypeDomain | CanonicalNamespace::Interface
                )
        }) {
            if !local.is_exported() {
                return Err(
                    CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                        module: stage.module_key.clone(),
                        name: declaration.name().into(),
                        dependency: dependency.clone().into_boxed_str(),
                        span: declaration.declaration_span(),
                    },
                );
            }
            continue;
        }

        if let Some((_, _, binding)) = imports.bindings().find(|(module, _, binding)| {
            *module == &stage.module_key
                && binding.local_name() == dependency
                && matches!(
                    binding.lookup_key().namespace(),
                    CanonicalNamespace::TypeDomain | CanonicalNamespace::Interface
                )
        }) {
            if !matches!(binding.declaration_visibility(), Visibility::Public) {
                return Err(
                    CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                        module: stage.module_key.clone(),
                        name: declaration.name().into(),
                        dependency: dependency.clone().into_boxed_str(),
                        span: declaration.declaration_span(),
                    },
                );
            }
            validate_public_defining_module_path(
                stage,
                stages,
                declaration,
                binding.defining_identity().module_key(),
                dependency,
                declaration.declaration_span(),
            )?;
            continue;
        }

        return Err(
            CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.clone().into_boxed_str(),
                span: declaration.declaration_span(),
            },
        );
    }
    Ok(())
}

/// Validate the transitive closure of public ordinary type aliases.
///
/// Ordinary enum and struct bodies are deliberately excluded from this walk:
/// their recursive references describe nominal recursive data and are not
/// alias-expansion edges.  Only a checked `TypeBody::Alias` can add another
/// declaration to the traversal, and every edge is resolved through the
/// canonical staged declaration identity before it is followed.
fn validate_public_type_alias_dependency_closure(
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let builtins = TypeEnv::with_builtin_types();
    let mut visiting = HashSet::<CanonicalDeclarationIdentity>::new();
    let mut validated = HashSet::<CanonicalDeclarationIdentity>::new();

    for stage in stages {
        for declaration in &stage.definitions {
            if !declaration.is_exported()
                || !matches!(
                    declaration.fact(),
                    CanonicalCheckedDeclarationFact::Type {
                        body: TypeBody::Alias(_),
                        ..
                    }
                )
            {
                continue;
            }
            validate_public_type_alias_declaration(
                stage,
                stages,
                imports,
                declaration,
                &builtins,
                &mut visiting,
                &mut validated,
            )?;
        }
    }
    Ok(())
}

/// Walk one public type alias and recurse only into alias targets.
fn validate_public_type_alias_declaration(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    builtins: &TypeEnv,
    visiting: &mut HashSet<CanonicalDeclarationIdentity>,
    validated: &mut HashSet<CanonicalDeclarationIdentity>,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    if validated.contains(declaration.identity()) {
        return Ok(());
    }
    if !visiting.insert(declaration.identity().clone()) {
        return Err(
            CanonicalCheckedModuleFinalizationError::CyclicPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: declaration.name().into(),
                span: declaration.declaration_span(),
            },
        );
    }

    let CanonicalCheckedDeclarationFact::Type { params, body, .. } = declaration.fact() else {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    };
    let TypeBody::Alias(alias) = body else {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    };

    let type_parameters = params
        .iter()
        .map(ToString::to_string)
        .collect::<HashSet<_>>();
    let mut dependencies = Vec::new();
    collect_surface_type_names(alias, &mut dependencies);
    dependencies.retain(|dependency| !type_parameters.contains(dependency));

    for dependency in dependencies {
        if dependency == "Prop" || builtins.resolve_type(&dependency).is_ok() {
            continue;
        }

        validate_public_type_dependencies(
            stage,
            stages,
            imports,
            declaration,
            builtins,
            std::slice::from_ref(&dependency),
        )?;
        let (target_stage, target) = resolve_public_type_dependency_target(
            stage,
            stages,
            imports,
            declaration,
            &dependency,
            declaration.declaration_span(),
        )?;
        if !matches!(
            target.fact(),
            CanonicalCheckedDeclarationFact::Type {
                body: TypeBody::Alias(_),
                ..
            }
        ) {
            continue;
        }
        if visiting.contains(target.identity()) {
            return Err(
                CanonicalCheckedModuleFinalizationError::CyclicPublicExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.into_boxed_str(),
                    span: declaration.declaration_span(),
                },
            );
        }
        validate_public_type_alias_declaration(
            target_stage,
            stages,
            imports,
            target,
            builtins,
            visiting,
            validated,
        )?;
    }

    visiting.remove(declaration.identity());
    validated.insert(declaration.identity().clone());
    Ok(())
}

/// Resolve one public type dependency to its canonical staged declaration.
///
/// Local names are resolved in the canonical stage for the importing module;
/// imported names must match the binding's defining identity exactly.  Both
/// paths enforce public declaration visibility and, for imports, the existing
/// publicly reachable defining-module path check.
fn resolve_public_type_dependency_target<'a>(
    stage: &ModuleStage,
    stages: &'a [ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    dependency: &str,
    span: Span,
) -> Result<
    (&'a ModuleStage, &'a CanonicalCheckedDeclaration),
    CanonicalCheckedModuleFinalizationError,
> {
    let local_stage = stages
        .iter()
        .find(|candidate| candidate.module_key == stage.module_key)
        .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
    if let Some(target) = local_stage.definitions.iter().find(|candidate| {
        candidate.name() == dependency
            && matches!(
                candidate.namespace(),
                CanonicalNamespace::TypeDomain | CanonicalNamespace::Interface
            )
    }) {
        if target.identity().module_key() != &local_stage.module_key {
            return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
        }
        return Ok((local_stage, target));
    }

    let Some((_, _, binding)) = imports.bindings().find(|(module, _, binding)| {
        *module == &stage.module_key
            && binding.local_name() == dependency
            && matches!(
                binding.lookup_key().namespace(),
                CanonicalNamespace::TypeDomain | CanonicalNamespace::Interface
            )
    }) else {
        return Err(
            CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.to_owned().into_boxed_str(),
                span,
            },
        );
    };

    let target_stage = stages
        .iter()
        .find(|candidate| candidate.module_key == *binding.defining_identity().module_key())
        .ok_or_else(
            || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.to_owned().into_boxed_str(),
                span,
            },
        )?;
    let target = target_stage
        .definitions
        .iter()
        .find(|candidate| {
            candidate.identity() == binding.defining_identity()
                && candidate.namespace() == binding.lookup_key().namespace()
        })
        .ok_or_else(
            || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.to_owned().into_boxed_str(),
                span,
            },
        )?;
    if !target.is_exported() {
        return Err(
            CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.to_owned().into_boxed_str(),
                span,
            },
        );
    }
    Ok((target_stage, target))
}

/// Validate that an imported dependency's defining module path is publicly
/// reachable from the crate root.
fn validate_public_defining_module_path(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    declaration: &CanonicalCheckedDeclaration,
    defining_module: &ModuleKey,
    dependency: &str,
    span: Span,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let mut parent = defining_module.clone();
    while let Some(ancestor) = parent.parent() {
        parent = ancestor;
    }

    for segment in defining_module.segments() {
        let child = parent
            .child(segment.as_str())
            .map_err(|_| CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
        let parent_stage = stages
            .iter()
            .find(|candidate| candidate.module_key == parent)
            .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
        let child_declaration = parent_stage
            .definitions
            .iter()
            .find(|candidate| {
                candidate.kind() == CanonicalDeclarationKind::ModuleDecl
                    && candidate.identity().module_key() == &child
            })
            .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
        if !child_declaration.is_exported() {
            return Err(
                CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.to_owned().into_boxed_str(),
                    span,
                },
            );
        }
        parent = child;
    }

    if &parent != defining_module {
        return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
    }
    Ok(())
}

fn collect_surface_type_body_names(body: &TypeBody, names: &mut Vec<String>) {
    match body {
        TypeBody::Struct(fields) => {
            for field in fields {
                collect_surface_type_names(&field.ty, names);
            }
        }
        TypeBody::Enum(variants) => {
            for variant in variants {
                for field in &variant.fields {
                    collect_surface_type_names(&field.ty, names);
                }
                match &variant.payload {
                    ash_parser::surface::VariantPayload::Unit => {}
                    ash_parser::surface::VariantPayload::Record(fields) => {
                        for field in fields {
                            collect_surface_type_names(&field.ty, names);
                        }
                    }
                    ash_parser::surface::VariantPayload::Tuple(types) => {
                        for ty in types {
                            collect_surface_type_names(ty, names);
                        }
                    }
                }
            }
        }
        TypeBody::Alias(ty) => collect_surface_type_names(ty, names),
    }
}

fn collect_interface_type_names(interface: &InterfaceDef, names: &mut Vec<String>) {
    let type_parameters = interface
        .type_params
        .iter()
        .map(|parameter| parameter.name.to_string())
        .collect::<HashSet<_>>();
    let mut collected = Vec::new();
    for parameter in &interface.type_params {
        if let Some(domain) = &parameter.domain {
            collect_surface_type_names(domain, &mut collected);
        }
    }
    for constraint in &interface.evidence_constraints {
        collect_surface_type_names(&constraint.subject, &mut collected);
        collect_surface_type_names(&constraint.interface, &mut collected);
    }
    for associated_type in &interface.associated_types {
        if let ash_parser::surface::AssociatedTypeKind::SealedFamily { result_domain, .. } =
            &associated_type.kind
        {
            collect_surface_type_names(result_domain, &mut collected);
        }
    }
    for method in &interface.methods {
        for parameter in &method.params {
            collect_surface_type_names(parameter, &mut collected);
        }
        collect_surface_type_names(&method.return_type, &mut collected);
    }
    for law in &interface.laws {
        for parameter in &law.params {
            collect_surface_type_names(&parameter.ty, &mut collected);
        }
    }
    names.extend(
        collected
            .into_iter()
            .filter(|name| !type_parameters.contains(name)),
    );
}

fn validate_effect_row_dependencies(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    row: &ComputationRow,
    names: &mut Vec<String>,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for item in &row.items {
        match item {
            ComputationRowItem::Group { path, span } => {
                let Some(name) = path.last() else {
                    continue;
                };
                if path.len() == 1 {
                    validate_public_namespace_dependency(
                        stage,
                        stages,
                        imports,
                        declaration,
                        name,
                        CanonicalNamespace::RowName,
                        *span,
                    )?;
                } else {
                    validate_public_qualified_namespace_dependency(
                        stage,
                        stages,
                        declaration,
                        path,
                        CanonicalNamespace::RowName,
                        *span,
                    )?;
                }
                names.push(name.to_string());
            }
            ComputationRowItem::WholeRow { variable, span } => {
                validate_public_namespace_dependency_if_present(
                    stage,
                    stages,
                    imports,
                    declaration,
                    variable,
                    CanonicalNamespace::RowName,
                    *span,
                )?;
                names.push(variable.to_string());
            }
            ComputationRowItem::Operation {
                path,
                separator: None,
                span,
            } if path.len() == 1 => {
                let variable = &path[0];
                validate_public_namespace_dependency_if_present(
                    stage,
                    stages,
                    imports,
                    declaration,
                    variable,
                    CanonicalNamespace::RowName,
                    *span,
                )?;
                names.push(variable.to_string());
            }
            ComputationRowItem::Operation {
                path,
                separator: Some(ash_parser::surface::RowPathSeparator::DoubleColon),
                span,
            } if path.len() == 2 => {
                validate_public_qualified_impl_operation_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    &path[0],
                    &path[1],
                    *span,
                )?;
            }
            ComputationRowItem::Role { path, span } => {
                let Some(name) = path.last() else {
                    continue;
                };
                if path.len() == 1 {
                    validate_public_namespace_dependency(
                        stage,
                        stages,
                        imports,
                        declaration,
                        name,
                        CanonicalNamespace::Role,
                        *span,
                    )?;
                } else {
                    validate_public_qualified_namespace_dependency(
                        stage,
                        stages,
                        declaration,
                        path,
                        CanonicalNamespace::Role,
                        *span,
                    )?;
                }
            }
            ComputationRowItem::Policy { path, span } => {
                let Some(name) = path.last() else {
                    continue;
                };
                if path.len() == 1 {
                    validate_public_namespace_dependency(
                        stage,
                        stages,
                        imports,
                        declaration,
                        name,
                        CanonicalNamespace::Policy,
                        *span,
                    )?;
                } else {
                    validate_public_qualified_namespace_dependency(
                        stage,
                        stages,
                        declaration,
                        path,
                        CanonicalNamespace::Policy,
                        *span,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Validate a public `Impl::operation` row item when its qualifier resolves to
/// a checked implementation declaration. Other qualified operation rows are
/// retained as non-authorizing row metadata for their owning checker.
fn validate_public_qualified_impl_operation_dependency(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    implementation: &str,
    operation: &str,
    span: Span,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let missing_implementation =
        || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: implementation.to_owned().into_boxed_str(),
            span,
        };
    let private_implementation =
        || CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: implementation.to_owned().into_boxed_str(),
            span,
        };
    let missing_operation =
        || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: format!("{implementation}::{operation}").into_boxed_str(),
            span,
        };

    let (target_stage, target) = if let Some(target) = stage.definitions.iter().find(|candidate| {
        candidate.namespace() == CanonicalNamespace::ImplementationRegistry
            && candidate.name() == implementation
    }) {
        (stage, target)
    } else if let Some((_, _, binding)) = imports.bindings().find(|(module, _, binding)| {
        *module == &stage.module_key
            && binding.lookup_key().namespace() == CanonicalNamespace::ImplementationRegistry
            && binding.local_name() == implementation
    }) {
        if !matches!(
            binding.declaration_visibility(),
            ash_parser::surface::Visibility::Public
        ) {
            return Err(private_implementation());
        }
        validate_public_defining_module_path(
            stage,
            stages,
            declaration,
            binding.defining_identity().module_key(),
            implementation,
            span,
        )?;
        let Some((target_stage, target)) = stages.iter().find_map(|candidate_stage| {
            candidate_stage
                .definitions
                .iter()
                .find(|candidate| {
                    candidate.identity() == binding.defining_identity()
                        && candidate.name() == binding.lookup_key().visible_local_key()
                        && candidate.namespace() == CanonicalNamespace::ImplementationRegistry
                })
                .map(|target| (candidate_stage, target))
        }) else {
            return Err(missing_implementation());
        };
        (target_stage, target)
    } else {
        return Ok(());
    };

    if !target.is_exported() {
        return Err(private_implementation());
    }

    if target_stage.definitions.iter().any(|candidate| {
        candidate.identity().canonical_parent() == Some(target.identity())
            && candidate.namespace() == CanonicalNamespace::ValueCallable
            && candidate.name() == operation
    }) {
        Ok(())
    } else {
        Err(missing_operation())
    }
}

/// Resolve a row alias/group target from the staged checker view.
///
/// Row names are a namespace of their own.  An unqualified name can be local
/// or come from a parsed binding; a qualified name is resolved against the
/// canonical module stages.  In both cases the target must be exportable
/// because it is reachable from a public row declaration.
fn resolve_public_row_dependency_target<'a>(
    stage: &ModuleStage,
    stages: &'a [ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    path: &[Box<str>],
    span: Span,
) -> Result<
    (&'a ModuleStage, &'a CanonicalCheckedDeclaration),
    CanonicalCheckedModuleFinalizationError,
> {
    if path.len() > 1 {
        return resolve_public_qualified_namespace_dependency(
            stage,
            stages,
            declaration,
            path,
            CanonicalNamespace::RowName,
            span,
        );
    }

    let dependency = path.last().map(|segment| segment.as_ref()).ok_or_else(|| {
        CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: "<empty>".into(),
            span,
        }
    })?;
    resolve_public_unqualified_row_dependency_target_if_present(
        stage,
        stages,
        imports,
        declaration,
        dependency,
        span,
    )?
    .ok_or_else(
        || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
            module: stage.module_key.clone(),
            name: declaration.name().into(),
            dependency: dependency.into(),
            span,
        },
    )
}

/// Resolve an unqualified row target when the spelling is a named row carrier.
///
/// A bare row item may also be a whole-row variable. The existing row checker
/// owns those variables, so this helper distinguishes an absent name from a
/// staged alias/group and lets the finalizer validate only the latter.
fn resolve_public_unqualified_row_dependency_target_if_present<'a>(
    stage: &ModuleStage,
    stages: &'a [ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    dependency: &str,
    span: Span,
) -> Result<
    Option<(&'a ModuleStage, &'a CanonicalCheckedDeclaration)>,
    CanonicalCheckedModuleFinalizationError,
> {
    let target_stage = stages
        .iter()
        .find(|candidate| candidate.module_key == stage.module_key)
        .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
    if let Some(target) = target_stage.definitions.iter().find(|candidate| {
        candidate.namespace() == CanonicalNamespace::RowName && candidate.name() == dependency
    }) {
        if !target.is_exported() {
            return Err(
                CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: dependency.into(),
                    span,
                },
            );
        }
        return Ok(Some((target_stage, target)));
    }

    let Some((_, _, binding)) = imports.bindings().find(|(module, _, binding)| {
        *module == &stage.module_key
            && binding.lookup_key().namespace() == CanonicalNamespace::RowName
            && binding.local_name() == dependency
    }) else {
        return Ok(None);
    };
    if !matches!(binding.declaration_visibility(), Visibility::Public) {
        return Err(
            CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.into(),
                span,
            },
        );
    }
    let target_stage = stages
        .iter()
        .find(|candidate| candidate.module_key == binding.defining_identity().module_key().clone())
        .ok_or_else(
            || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.into(),
                span,
            },
        )?;
    let target = target_stage
        .definitions
        .iter()
        .find(|candidate| {
            candidate.identity() == binding.defining_identity()
                && candidate.namespace() == CanonicalNamespace::RowName
        })
        .ok_or_else(
            || CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.into(),
                span,
            },
        )?;
    if !target.is_exported() {
        return Err(
            CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: dependency.into(),
                span,
            },
        );
    }
    Ok(Some((target_stage, target)))
}

/// Validate transitive public effect-row closure and reject row cycles.
///
/// Direct visibility checks are intentionally kept separate from this walk:
/// the former validates each declaration's immediate syntax, while this walk
/// proves that a public row does not hide a private or incomplete row behind
/// an otherwise public alias/group.  The walk consumes only staged identities
/// and checked row facts.
fn validate_public_effect_row_dependency_closure(
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let mut visiting = HashSet::<CanonicalDeclarationIdentity>::new();
    let mut validated = HashSet::<CanonicalDeclarationIdentity>::new();
    for stage in stages {
        for declaration in &stage.definitions {
            if !declaration.is_exported()
                || !matches!(
                    declaration.fact(),
                    CanonicalCheckedDeclarationFact::EffectAlias { .. }
                        | CanonicalCheckedDeclarationFact::EffectGroup { .. }
                )
            {
                continue;
            }
            validate_public_effect_row_declaration(
                stage,
                stages,
                imports,
                declaration,
                &mut visiting,
                &mut validated,
            )?;
        }
    }
    Ok(())
}

fn validate_public_effect_row_declaration(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    visiting: &mut HashSet<CanonicalDeclarationIdentity>,
    validated: &mut HashSet<CanonicalDeclarationIdentity>,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    if validated.contains(declaration.identity()) {
        return Ok(());
    }
    if !visiting.insert(declaration.identity().clone()) {
        return Err(
            CanonicalCheckedModuleFinalizationError::CyclicPublicExportDependency {
                module: stage.module_key.clone(),
                name: declaration.name().into(),
                dependency: declaration.name().into(),
                span: declaration.declaration_span(),
            },
        );
    }

    let row = match declaration.fact() {
        CanonicalCheckedDeclarationFact::EffectAlias { definition } => &definition.row,
        CanonicalCheckedDeclarationFact::EffectGroup { definition } => &definition.row,
        _ => unreachable!("row closure only visits row declarations"),
    };
    for item in &row.items {
        let ((target_stage, target), span) = match item {
            ComputationRowItem::Group { path, span } => (
                resolve_public_row_dependency_target(
                    stage,
                    stages,
                    imports,
                    declaration,
                    path,
                    *span,
                )?,
                *span,
            ),
            ComputationRowItem::WholeRow { variable, span } => {
                let Some(target) = resolve_public_unqualified_row_dependency_target_if_present(
                    stage,
                    stages,
                    imports,
                    declaration,
                    variable,
                    *span,
                )?
                else {
                    continue;
                };
                (target, *span)
            }
            _ => continue,
        };
        if visiting.contains(target.identity()) {
            return Err(
                CanonicalCheckedModuleFinalizationError::CyclicPublicExportDependency {
                    module: stage.module_key.clone(),
                    name: declaration.name().into(),
                    dependency: target.name().into(),
                    span,
                },
            );
        }
        if matches!(
            target.kind(),
            CanonicalDeclarationKind::EffectAlias | CanonicalDeclarationKind::EffectGroup
        ) {
            if !matches!(
                target.fact(),
                CanonicalCheckedDeclarationFact::EffectAlias { .. }
                    | CanonicalCheckedDeclarationFact::EffectGroup { .. }
            ) {
                return Err(
                    CanonicalCheckedModuleFinalizationError::MissingPublicExportDependency {
                        module: stage.module_key.clone(),
                        name: declaration.name().into(),
                        dependency: target.name().into(),
                        span,
                    },
                );
            }
            validate_public_effect_row_declaration(
                target_stage,
                stages,
                imports,
                target,
                visiting,
                validated,
            )?;
        }
    }
    visiting.remove(declaration.identity());
    validated.insert(declaration.identity().clone());
    Ok(())
}

fn validate_policy_definitions(
    environment: &TypeEnv,
    definitions: &[Definition],
    module: &ModuleKey,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for policy in definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Policy(policy) => Some(policy),
            _ => None,
        })
    {
        let mut policy_environment = environment.clone();
        let mut type_parameters = HashMap::new();
        for parameter in &policy.type_params {
            policy_environment
                .register_type_parameter_kind(parameter.to_string(), Kind::Type)
                .map_err(|error| policy_error(module, policy, error.to_string()))?;
            type_parameters.insert(parameter.to_string(), Type::Var(TypeVar::fresh()));
        }

        for field in &policy.fields {
            let field_type =
                workflow_surface_type_to_type(&policy_environment, &field.ty, &type_parameters)
                    .map_err(|error| policy_error(module, policy, error.to_string()))?;
            policy_environment.bind_variable(field.name.as_ref(), field_type.clone());

            if let Some(default) = &field.default {
                let checked = check_expr(&policy_environment, default);
                if !checked.is_ok() {
                    return Err(policy_error(
                        module,
                        policy,
                        format!("field '{}' default: {}", field.name, checked.errors[0]),
                    ));
                }
                let default_type = checked.substitution.apply(&checked.ty);
                if !policy_default_matches(&field_type, &default_type) {
                    return Err(policy_error(
                        module,
                        policy,
                        format!(
                            "field '{}' default has type {}, expected {}",
                            field.name, default_type, field_type
                        ),
                    ));
                }
            }
        }

        if let Some(invariant) = &policy.where_clause {
            let checked = check_expr(&policy_environment, invariant);
            if !checked.is_ok() {
                return Err(policy_error(
                    module,
                    policy,
                    format!("where clause: {}", checked.errors[0]),
                ));
            }
            let invariant_type = checked.substitution.apply(&checked.ty);
            if invariant_type != Type::Bool {
                return Err(policy_error(
                    module,
                    policy,
                    format!("where clause must have type Bool, found {invariant_type}"),
                ));
            }
        }
    }
    Ok(())
}

fn policy_default_matches(expected: &Type, actual: &Type) -> bool {
    if matches!(expected, Type::Var(_)) {
        return true;
    }
    if matches!(actual, Type::Var(_)) {
        return false;
    }
    unify(expected, actual).is_ok()
}

fn policy_error(
    module: &ModuleKey,
    policy: &PolicyDef,
    reason: impl Into<Box<str>>,
) -> CanonicalCheckedModuleFinalizationError {
    CanonicalCheckedModuleFinalizationError::Policy {
        module: module.clone(),
        name: policy.name.to_string().into_boxed_str(),
        span: policy.span,
        reason: reason.into(),
    }
}

fn imported_type_identity_definitions(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
) -> Vec<ash_core::ast::TypeDef> {
    imports
        .bindings()
        .filter(|(module, _, binding)| {
            *module == &stage.module_key
                && binding.lookup_key().namespace() == CanonicalNamespace::TypeDomain
        })
        .filter_map(|(_, _, binding)| {
            let target = stages.iter().find_map(|target_stage| {
                target_stage
                    .definitions
                    .iter()
                    .zip(&target_stage.raw_definitions)
                    .find(|(declaration, _)| {
                        declaration.identity() == binding.defining_identity()
                            && declaration.name() == binding.lookup_key().visible_local_key()
                    })
                    .map(|(_, definition)| definition)
            })?;
            let identity = match target {
                Definition::Type(type_definition) => SurfaceTypeDef {
                    visibility: Visibility::Public,
                    name: binding.local_name().into(),
                    params: type_definition.params.clone(),
                    body: TypeBody::Struct(Vec::new()),
                    builtin: type_definition.builtin,
                    span: type_definition.span,
                    source: type_definition.source.clone(),
                },
                Definition::Newtype(newtype) => SurfaceTypeDef {
                    visibility: Visibility::Public,
                    name: binding.local_name().into(),
                    params: newtype
                        .type_params
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    body: TypeBody::Struct(Vec::new()),
                    builtin: false,
                    span: newtype.span,
                    source: newtype.source.clone(),
                },
                Definition::ResourceType(resource_type) => SurfaceTypeDef {
                    visibility: Visibility::Public,
                    name: binding.local_name().into(),
                    params: Vec::new(),
                    body: TypeBody::Struct(Vec::new()),
                    builtin: false,
                    span: resource_type.span,
                    source: None,
                },
                _ => return None,
            };
            Some(ash_parser::lower_surface_type_def(&identity))
        })
        .collect()
}

fn stage_type_environment(
    stage: &ModuleStage,
    imported_type_definitions: &[ash_core::ast::TypeDef],
) -> Result<TypeEnv, CanonicalCheckedModuleFinalizationError> {
    let mut environment = TypeEnv::with_builtin_types();

    for type_definition in imported_type_definitions {
        environment
            .register_type_identity(type_definition)
            .map_err(|error| {
                signature_error(
                    &stage.module_key,
                    "<imports>",
                    Span::default(),
                    error.to_string(),
                )
            })?;
    }

    // Callable preflight resolves user-defined types. Install identity
    // placeholders and then complete all ordinary type declarations before
    // registering callable declaration markers, matching the normal checker
    // ordering without consulting any source outside the staged snapshot.
    for definition in &stage.raw_definitions {
        if let Definition::Type(type_definition) = definition {
            environment.declare_type_name(type_definition.name.as_ref());
        }
    }
    for definition in &stage.raw_definitions {
        if let Definition::Type(type_definition) = definition {
            environment
                .register_type(&ash_parser::lower_surface_type_def(type_definition))
                .map_err(|error| {
                    signature_error(
                        &stage.module_key,
                        "<declarations>",
                        Span::default(),
                        error.to_string(),
                    )
                })?;
        }
    }
    environment
        .register_surface_declarations(&stage.raw_definitions)
        .map_err(|error| {
            signature_error(
                &stage.module_key,
                "<declarations>",
                Span::default(),
                error.to_string(),
            )
        })?;
    let mut registered_impls = HashSet::new();
    for definition in &stage.raw_definitions {
        let result = match definition {
            Definition::Interface(interface)
                if !environment.has_interface(interface.name.as_ref()) =>
            {
                environment.register_interface(interface)
            }
            Definition::Interface(_) => Ok(()),
            Definition::ResourceType(resource_type) => {
                environment.register_resource_type(resource_type)
            }
            Definition::Type(_) => Ok(()),
            Definition::Impl(implementation) => {
                let key = format!(
                    "{}::{:?}",
                    implementation.interface, implementation.type_args
                );
                if registered_impls.insert(key) {
                    match environment.register_impl(implementation) {
                        Ok(()) => environment.register_impl_proofs(implementation),
                        Err(error) => Err(error),
                    }
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        };
        result.map_err(|error| {
            signature_error(
                &stage.module_key,
                "<declarations>",
                Span::default(),
                error.to_string(),
            )
        })?;
    }
    Ok(environment)
}

fn check_implementation_method_body_in_env(
    environment: &TypeEnv,
    implementation: &ImplDef,
    method: &ImplMethodDef,
) -> Result<(Type, Type), Box<str>> {
    let scheme = environment
        .resolve_interface_evidence(&implementation.interface, &implementation.type_args)
        .map_err(|error| error.to_string().into_boxed_str())?;
    let method_info = scheme
        .methods
        .iter()
        .find(|candidate| candidate.name == method.name.as_ref())
        .ok_or_else(|| {
            format!(
                "implementation method '{}::{}' has no checked interface method",
                implementation.interface, method.name
            )
            .into_boxed_str()
        })?;
    if method_info.params.len() != method.params.len() {
        return Err(format!(
            "implementation method '{}::{}' has {} parameters, expected {}",
            implementation.interface,
            method.name,
            method.params.len(),
            method_info.params.len()
        )
        .into_boxed_str());
    }

    let mut method_environment = environment.clone();
    for (name, ty) in method.params.iter().zip(&method_info.params) {
        method_environment.bind_variable(name.as_ref(), ty.clone());
    }
    let result = check_expr(&method_environment, &method.body);
    if !result.is_ok() {
        return Err(result
            .errors
            .into_iter()
            .next()
            .map(|error| error.to_string().into_boxed_str())
            .unwrap_or_else(|| "implementation method body checking failed".into()));
    }
    let body_type = result.substitution.apply(&result.ty);
    let return_substitution = unify(&method_info.return_type, &body_type).map_err(|_| {
        format!(
            "implementation method '{}::{}' must return {}, found {}",
            implementation.interface, method.name, method_info.return_type, body_type
        )
        .into_boxed_str()
    })?;
    let checked_body_type = return_substitution.apply(&body_type);
    Ok((
        Type::Fn(
            method_info.params.clone(),
            Box::new(method_info.return_type.clone()),
        ),
        checked_body_type,
    ))
}

fn checked_declaration_skeleton(
    entry: &CanonicalCollectedEntry,
    origin: ModuleArtifactOrigin,
    macro_summaries: &[MacroSummary],
) -> CanonicalCheckedDeclaration {
    let (visibility, body_span) = entry
        .raw_definition()
        .map(|definition| {
            (
                if entry.identity().canonical_parent().is_some()
                    && matches!(
                        definition,
                        Definition::Interface(_) | Definition::SealedDomain(_)
                    )
                {
                    Visibility::Inherited
                } else {
                    definition_visibility(definition)
                },
                entry.callable_body().map(Spanned::span),
            )
        })
        .unwrap_or((Visibility::Inherited, None));
    CanonicalCheckedDeclaration {
        identity: entry.identity().clone(),
        name: entry
            .declared_name()
            .unwrap_or_else(|| entry.lookup_key().visible_local_key())
            .into(),
        kind: entry.kind(),
        namespace: entry.namespace(),
        declaration_span: entry.source_anchor(),
        body_span,
        origin,
        visibility,
        signature: None,
        body_type: None,
        fact: checked_declaration_fact(entry, macro_summaries),
    }
}

fn checked_declaration_fact(
    entry: &CanonicalCollectedEntry,
    macro_summaries: &[MacroSummary],
) -> CanonicalCheckedDeclarationFact {
    if entry.kind() == CanonicalDeclarationKind::ModuleDecl {
        return CanonicalCheckedDeclarationFact::StructuralModule {
            module: entry.identity().module_key().clone(),
        };
    }
    let Some(definition) = entry.raw_definition() else {
        return CanonicalCheckedDeclarationFact::Opaque;
    };

    match definition {
        Definition::Type(type_definition) => {
            if let Some(parent) = entry.identity().canonical_parent() {
                CanonicalCheckedDeclarationFact::Constructor {
                    parent: parent.clone(),
                    name: entry
                        .declared_name()
                        .unwrap_or_else(|| entry.lookup_key().visible_local_key())
                        .into(),
                }
            } else {
                CanonicalCheckedDeclarationFact::Type {
                    params: type_definition
                        .params
                        .iter()
                        .map(|parameter| parameter.as_ref().into())
                        .collect(),
                    body: type_definition.body.clone(),
                    builtin: type_definition.builtin,
                }
            }
        }
        Definition::Newtype(newtype) => {
            if let Some(parent) = entry.identity().canonical_parent() {
                CanonicalCheckedDeclarationFact::Constructor {
                    parent: parent.clone(),
                    name: entry
                        .declared_name()
                        .unwrap_or_else(|| entry.lookup_key().visible_local_key())
                        .into(),
                }
            } else {
                CanonicalCheckedDeclarationFact::Newtype {
                    type_params: newtype.type_params.clone().into_boxed_slice(),
                    constructor: newtype.constructor.as_ref().into(),
                    representation: newtype.representation.clone(),
                }
            }
        }
        Definition::ResourceType(resource_type) => CanonicalCheckedDeclarationFact::ResourceType {
            fields: resource_type
                .fields
                .iter()
                .map(|field| (field.name.as_ref().into(), field.ty.clone()))
                .collect(),
        },
        Definition::Interface(interface) if entry.identity().canonical_parent().is_none() => {
            CanonicalCheckedDeclarationFact::Interface {
                definition: interface.clone(),
                evidence: interface_evidence_summary(interface),
            }
        }
        Definition::Interface(interface) if entry.kind() == CanonicalDeclarationKind::Law => {
            let name = entry
                .declared_name()
                .unwrap_or_else(|| entry.lookup_key().visible_local_key());
            interface
                .laws
                .iter()
                .find(|law| law.name.as_ref() == name)
                .cloned()
                .map_or(CanonicalCheckedDeclarationFact::Opaque, |law| {
                    CanonicalCheckedDeclarationFact::Law {
                        definition: Box::new(law),
                    }
                })
        }
        Definition::Impl(implementation) if entry.identity().canonical_parent().is_none() => {
            CanonicalCheckedDeclarationFact::Implementation {
                summary: implementation_summary(implementation),
            }
        }
        Definition::SealedDomain(domain) => {
            if let Some(parent) = entry.identity().canonical_parent() {
                let Some(constructor) = domain
                    .constructors
                    .iter()
                    .find(|constructor| Some(constructor.name.as_ref()) == entry.declared_name())
                    .cloned()
                else {
                    return CanonicalCheckedDeclarationFact::Opaque;
                };
                CanonicalCheckedDeclarationFact::SealedDomainConstructor {
                    parent: parent.clone(),
                    constructor,
                }
            } else {
                CanonicalCheckedDeclarationFact::SealedDomain {
                    definition: domain.clone(),
                }
            }
        }
        Definition::EffectAlias(alias) => CanonicalCheckedDeclarationFact::EffectAlias {
            definition: alias.clone(),
        },
        Definition::EffectGroup(group) => CanonicalCheckedDeclarationFact::EffectGroup {
            definition: group.clone(),
        },
        Definition::DataKind(data_kind) => CanonicalCheckedDeclarationFact::DataKind {
            definition: data_kind.clone(),
        },
        Definition::PropositionPredicate(predicate) => {
            CanonicalCheckedDeclarationFact::PropositionPredicate {
                definition: predicate.clone(),
            }
        }
        Definition::Role(role) => CanonicalCheckedDeclarationFact::Role {
            definition: role.clone(),
        },
        Definition::Policy(policy) => CanonicalCheckedDeclarationFact::Policy {
            definition: Box::new(policy.clone()),
        },
        Definition::TypeFn(type_function) => CanonicalCheckedDeclarationFact::TypeFn {
            definition: Box::new(type_function.clone()),
        },
        Definition::Notation(notation) => CanonicalCheckedDeclarationFact::Notation {
            definition: notation.clone(),
        },
        Definition::Macro(macro_definition) => macro_summaries
            .iter()
            .find(|summary| summary.name == macro_definition.name)
            .cloned()
            .map_or(CanonicalCheckedDeclarationFact::Opaque, |summary| {
                CanonicalCheckedDeclarationFact::Macro { summary }
            }),
        Definition::Law(law) => CanonicalCheckedDeclarationFact::Law {
            definition: Box::new(law.clone()),
        },
        Definition::Impl(implementation) if entry.kind() == CanonicalDeclarationKind::Proof => {
            let name = entry
                .declared_name()
                .unwrap_or_else(|| entry.lookup_key().visible_local_key());
            implementation
                .proofs
                .iter()
                .find(|proof| proof.name.as_ref() == name)
                .cloned()
                .map_or(CanonicalCheckedDeclarationFact::Opaque, |proof| {
                    CanonicalCheckedDeclarationFact::Proof {
                        definition: Box::new(proof),
                    }
                })
        }
        Definition::Proof(proof) => CanonicalCheckedDeclarationFact::Proof {
            definition: Box::new(proof.clone()),
        },
        _ => CanonicalCheckedDeclarationFact::Opaque,
    }
}

fn implementation_summary(implementation: &ImplDef) -> CanonicalCheckedImplementationSummary {
    CanonicalCheckedImplementationSummary {
        interface: implementation.interface.clone(),
        type_params: implementation
            .type_params
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect(),
        type_args: implementation.type_args.clone().into_boxed_slice(),
        where_bounds: implementation
            .where_bounds
            .iter()
            .map(|bound| (bound.param.clone(), bound.bound.clone()))
            .collect(),
        associated_types: implementation
            .associated_type_bindings
            .iter()
            .map(|binding| (binding.name.clone(), binding.ty.clone()))
            .collect(),
        methods: implementation
            .methods
            .iter()
            .map(|method| method.name.clone())
            .collect(),
        handlers: implementation
            .handlers
            .iter()
            .map(|handler| handler.name.clone())
            .collect(),
        proofs: implementation
            .proofs
            .iter()
            .map(|proof| CanonicalCheckedNestedEvidenceSummary {
                name: proof.name.clone(),
                kind: CanonicalDeclarationKind::Proof,
                visibility: proof.visibility.clone(),
            })
            .collect(),
    }
}

fn interface_evidence_summary(
    interface: &InterfaceDef,
) -> Box<[CanonicalCheckedNestedEvidenceSummary]> {
    interface
        .laws
        .iter()
        .map(|law| CanonicalCheckedNestedEvidenceSummary {
            name: law.name.clone(),
            kind: CanonicalDeclarationKind::Law,
            visibility: law.visibility.clone(),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PublicExpressionDependency {
    /// An unqualified callable or constructor in the value namespace.
    Value(Box<str>),
    /// A qualified implementation operation, represented by its parent
    /// implementation name. The operation remains parent-scoped metadata.
    Implementation {
        /// The implementation-registry name used as the qualifier.
        implementation: Box<str>,
        /// The parent-scoped operation name.
        operation: Box<str>,
    },
}

fn collect_expr_dependency_names(
    expression: &ash_parser::surface::Expr,
    dependencies: &mut Vec<PublicExpressionDependency>,
) {
    ash_parser::surface::visit_expr(expression, &mut |expression| match expression {
        ash_parser::surface::Expr::Call {
            func,
            module: Some(implementation),
            ..
        } => dependencies.push(PublicExpressionDependency::Implementation {
            implementation: implementation.clone(),
            operation: func.clone(),
        }),
        ash_parser::surface::Expr::Call {
            func, module: None, ..
        }
        | ash_parser::surface::Expr::Constructor { name: func, .. } => {
            dependencies.push(PublicExpressionDependency::Value(
                func.to_string().into_boxed_str(),
            ));
        }
        _ => {}
    });
}

fn macro_summary_error(
    module: &ModuleKey,
    definitions: &[Definition],
    error: ash_parser::ExpansionError,
) -> CanonicalCheckedModuleFinalizationError {
    let (name, span) = match &error {
        ash_parser::ExpansionError::DuplicateMacroDeclaration {
            name,
            second_span: span,
            ..
        }
        | ash_parser::ExpansionError::UnknownMacroInvocation { name, span, .. }
        | ash_parser::ExpansionError::UnsupportedMacroInvocation { name, span, .. }
        | ash_parser::ExpansionError::MacroTokenTreeReparseFailed { name, span, .. }
        | ash_parser::ExpansionError::MacroArityMismatch { name, span, .. }
        | ash_parser::ExpansionError::UnsupportedMacroTemplate { name, span, .. }
        | ash_parser::ExpansionError::MacroTypeMismatch { name, span, .. }
        | ash_parser::ExpansionError::MacroExpansionDepthExceeded { name, span, .. }
        | ash_parser::ExpansionError::DeferredMacroInvocation { name, span, .. } => {
            (name.clone(), *span)
        }
        ash_parser::ExpansionError::DuplicateNotationDeclaration {
            operator,
            second_span,
            ..
        }
        | ash_parser::ExpansionError::ConflictingNotationDeclaration {
            operator,
            second_span,
            ..
        }
        | ash_parser::ExpansionError::UnresolvedOperatorSection {
            operator,
            span: second_span,
        } => (operator.clone(), *second_span),
    };
    let (name, span) = if name.is_empty() {
        definitions
            .iter()
            .find_map(|definition| match definition {
                Definition::Macro(definition)
                    if matches!(definition.visibility, Visibility::Public) =>
                {
                    Some((definition.name.clone(), definition.span))
                }
                _ => None,
            })
            .unwrap_or_else(|| ("<module>".into(), Span::default()))
    } else {
        (name, span)
    };
    CanonicalCheckedModuleFinalizationError::InvalidMacroSummary {
        module: module.clone(),
        name,
        span,
        reason: error.to_string().into_boxed_str(),
    }
}

fn validate_import_binding_local_names(
    imports: &CanonicalParsedImportResult,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for (module, name, binding) in imports.bindings() {
        if binding.local_name() != name {
            return Err(
                CanonicalCheckedModuleFinalizationError::BindingLocalNameMismatch {
                    module: module.clone(),
                    authoritative_name: name.into(),
                    binding_local_name: binding.local_name().into(),
                },
            );
        }
    }
    Ok(())
}

fn validate_public_use_bindings(
    imports: &CanonicalParsedImportResult,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for public_use in imports.public_uses() {
        let carrier = public_use.binding();
        let Some(authoritative) =
            imports.binding(public_use.importing_module(), carrier.local_name())
        else {
            return Err(
                CanonicalCheckedModuleFinalizationError::PublicUseBindingMismatch {
                    module: public_use.importing_module().clone(),
                    name: carrier.local_name().into(),
                },
            );
        };
        if !carrier.is_reexport() || carrier != authoritative {
            return Err(
                CanonicalCheckedModuleFinalizationError::PublicUseBindingMismatch {
                    module: public_use.importing_module().clone(),
                    name: carrier.local_name().into(),
                },
            );
        }
    }
    Ok(())
}

fn validate_import_binding_declaration_metadata(
    imports: &CanonicalParsedImportResult,
    stages: &[ModuleStage],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for (module, name, binding) in imports.bindings() {
        if !stages.iter().any(|stage| &stage.module_key == module) {
            return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
        }
        let Some(target) = stages.iter().find_map(|stage| {
            stage.definitions.iter().find(|declaration| {
                declaration.identity() == binding.defining_identity()
                    && declaration.name() == binding.lookup_key().visible_local_key()
            })
        }) else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingBindingTarget {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        };
        if target.visibility() != binding.declaration_visibility() {
            return Err(
                CanonicalCheckedModuleFinalizationError::BindingVisibilityMismatch {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        }
        let authoritative_source_ordinal = match binding.defining_identity().origin_key() {
            CanonicalDeclarationOriginKey::Source { source_ordinal }
            | CanonicalDeclarationOriginKey::Expanded { source_ordinal, .. } => *source_ordinal,
        };
        if binding.declaration_span() != target.declaration_span()
            || binding.source_ordinal() != authoritative_source_ordinal
        {
            return Err(
                CanonicalCheckedModuleFinalizationError::BindingDeclarationMetadataMismatch {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        }
    }
    Ok(())
}

fn validate_import_binding_module_visibility(
    graph: &ash_parser::CanonicalModuleGraph,
    imports: &CanonicalParsedImportResult,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let scopes = CanonicalProvisionalModuleScopes::from_graph(graph)
        .map_err(|_| CanonicalCheckedModuleFinalizationError::GraphMismatch)?;

    for (importing_module, name, binding) in imports.bindings() {
        let mut parent = graph.root_key().clone();
        for segment in binding.defining_identity().module_key().segments() {
            let scope = scopes
                .scope(&parent)
                .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
            let child = scope
                .child(segment)
                .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
            let visible = scopes
                // Structural module visibility is owned by the module that
                // declares the child, matching parsed-import resolution.
                .is_visible_from(child.visibility(), &parent, importing_module)
                .map_err(|_| CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
            if !visible {
                return Err(
                    CanonicalCheckedModuleFinalizationError::BindingModuleVisibilityMismatch {
                        module: importing_module.clone(),
                        name: name.into(),
                    },
                );
            }
            parent = child.module_key().clone();
        }
        if &parent != binding.defining_identity().module_key() {
            return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
        }
    }

    Ok(())
}

fn validate_import_binding_shapes(
    imports: &CanonicalParsedImportResult,
    stages: &[ModuleStage],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for (module, name, binding) in imports.bindings() {
        if !stages.iter().any(|stage| &stage.module_key == module) {
            return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
        }
        let Some(target) = stages.iter().find_map(|stage| {
            stage.definitions.iter().find(|declaration| {
                declaration.identity() == binding.defining_identity()
                    && declaration.name() == binding.lookup_key().visible_local_key()
            })
        }) else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingBindingTarget {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        };
        let binding_namespace = binding.lookup_key().namespace();
        let target_namespace = target.namespace();
        let binding_kind = binding.defining_identity().kind();
        let target_kind = target.kind();
        if binding_namespace != target_namespace || binding_kind != target_kind {
            return Err(
                CanonicalCheckedModuleFinalizationError::BindingShapeMismatch {
                    module: module.clone(),
                    name: name.into(),
                    binding_namespace,
                    target_namespace,
                    binding_kind,
                    target_kind,
                },
            );
        }
    }
    Ok(())
}

fn validate_public_use_module_export_closure(
    imports: &CanonicalParsedImportResult,
    stages: &[ModuleStage],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for public_use in imports
        .public_uses()
        .iter()
        .filter(|public_use| is_public_reexport(public_use.binding()))
    {
        let importing_module = public_use.importing_module();
        let binding = public_use.binding();
        let defining_module = binding.defining_identity().module_key();
        let attempted_path = binding
            .attempted_access_path()
            .iter()
            .map(|segment| segment.as_ref())
            .collect::<Vec<_>>()
            .join("::")
            .into_boxed_str();
        let mut current = defining_module.clone();
        let mut path = Vec::new();
        while let Some(parent) = current.parent() {
            path.push(
                current
                    .segments()
                    .last()
                    .cloned()
                    .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?,
            );
            current = parent;
        }
        path.reverse();

        let mut parent = current;
        for segment in path {
            let child = parent
                .child(segment.as_str())
                .map_err(|_| CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
            let parent_stage = stages
                .iter()
                .find(|stage| stage.module_key == parent)
                .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
            let child_declaration = parent_stage
                .definitions
                .iter()
                .find(|declaration| {
                    declaration.kind() == CanonicalDeclarationKind::ModuleDecl
                        && declaration.identity().module_key() == &child
                })
                .ok_or(CanonicalCheckedModuleFinalizationError::GraphMismatch)?;
            if !child_declaration.is_exported() {
                return Err(
                    CanonicalCheckedModuleFinalizationError::NonExportedPublicUseModulePath {
                        module: importing_module.clone(),
                        name: binding.local_name().into(),
                        defining_module: defining_module.clone(),
                        details: Box::new(NonExportedPublicUseModulePathDetails {
                            span: binding.use_span(),
                            declaration_span: child_declaration.declaration_span(),
                            path: attempted_path.clone(),
                            offending_segment: segment.into_boxed_str(),
                            violated_visibility: visibility_label(child_declaration.visibility()),
                        }),
                    },
                );
            }
            parent = child;
        }
        if &parent != defining_module {
            return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
        }
    }
    Ok(())
}

fn is_public_reexport(binding: &CanonicalParsedImportBinding) -> bool {
    binding.is_externally_public_reexport()
}

fn visibility_label(visibility: &Visibility) -> Box<str> {
    match visibility {
        Visibility::Inherited => "private".into(),
        Visibility::Public => "pub".into(),
        Visibility::Self_ => "pub(self)".into(),
        Visibility::Crate => "pub(crate)".into(),
        Visibility::Super { levels: 1 } => "pub(super)".into(),
        Visibility::Super { levels } => format!("pub(super^{levels})").into_boxed_str(),
        Visibility::Restricted { path } => format!("pub(in {path})").into_boxed_str(),
    }
}

fn validate_import_targets(
    imports: &CanonicalParsedImportResult,
    stages: &[ModuleStage],
    signatures: &[(CanonicalDeclarationIdentity, Type)],
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for (module, name, binding) in imports.bindings() {
        let Some(stage) = stages.iter().find(|stage| &stage.module_key == module) else {
            return Err(CanonicalCheckedModuleFinalizationError::GraphMismatch);
        };
        let Some(target_stage) = stages.iter().find(|stage| {
            stage.definitions.iter().any(|declaration| {
                declaration.identity() == binding.defining_identity()
                    && declaration.name() == binding.lookup_key().visible_local_key()
            })
        }) else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingBindingTarget {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        };
        let Some(target) = target_stage.definitions.iter().find(|declaration| {
            declaration.identity() == binding.defining_identity()
                && declaration.name() == binding.lookup_key().visible_local_key()
        }) else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingBindingTarget {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        };
        if target.origin() != binding.origin() {
            return Err(
                CanonicalCheckedModuleFinalizationError::BindingOriginMismatch {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        }
        if target.visibility() != binding.declaration_visibility() {
            return Err(
                CanonicalCheckedModuleFinalizationError::BindingVisibilityMismatch {
                    module: module.clone(),
                    name: name.into(),
                },
            );
        }
        if signatures
            .iter()
            .all(|(identity, _)| identity != binding.defining_identity())
            && matches!(
                target.kind(),
                CanonicalDeclarationKind::Function
                    | CanonicalDeclarationKind::Handler
                    | CanonicalDeclarationKind::BuiltinFn
            )
        {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingBindingTarget {
                    module: stage.module_key.clone(),
                    name: name.into(),
                },
            );
        }
    }
    Ok(())
}

fn validate_public_signatures(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    let builtin_types = TypeEnv::with_builtin_types();
    for (identity, callable) in &stage.callable_definitions {
        let Some(declaration) = stage
            .definitions
            .iter()
            .find(|declaration| declaration.identity() == identity)
        else {
            continue;
        };
        if !declaration.is_exported() {
            continue;
        }
        let (name, type_params, params, return_type, proposition_tail) = match callable {
            CallableDefinition::Function(function) => (
                function.name.as_ref(),
                function.type_params.as_slice(),
                function.params.as_slice(),
                function.return_type.as_ref(),
                function.proposition_tail.as_ref(),
            ),
            CallableDefinition::Builtin(function) => (
                function.name.as_ref(),
                function.type_params.as_slice(),
                function.params.as_slice(),
                Some(&function.return_type),
                function.proposition_tail.as_ref(),
            ),
            CallableDefinition::Handler(handler) => (
                handler.name.as_ref(),
                handler.type_params.as_slice(),
                handler.params.as_slice(),
                Some(&handler.return_type),
                handler.proposition_tail.as_ref(),
            ),
        };
        let mut names = Vec::new();
        for parameter in params {
            collect_surface_type_names(&parameter.ty, &mut names);
        }
        if let Some(return_type) = return_type {
            collect_surface_type_names(return_type, &mut names);
        }
        let mut proposition_dependencies = Vec::new();
        if let Some(tail) = proposition_tail {
            validate_public_proposition_tail_dependencies(
                stage,
                stages,
                imports,
                declaration,
                tail,
                &mut names,
                &mut proposition_dependencies,
            )?;
        }
        let type_parameters = type_params
            .iter()
            .map(|parameter| parameter.name.to_string())
            .collect::<HashSet<_>>();
        names.retain(|name| !type_parameters.contains(name));
        let Some(declaration) = stage
            .definitions
            .iter()
            .find(|declaration| declaration.identity() == identity)
        else {
            return Err(
                CanonicalCheckedModuleFinalizationError::MissingCheckedDeclaration {
                    module: stage.module_key.clone(),
                    name: name.into(),
                    span: callable_span(callable),
                },
            );
        };
        validate_public_type_dependencies(
            stage,
            stages,
            imports,
            declaration,
            &builtin_types,
            &names,
        )?;
        for dependency in proposition_dependencies {
            if let Some(private) = stage
                .definitions
                .iter()
                .find(|candidate| candidate.name() == dependency)
                && !private.is_exported()
            {
                return Err(
                    CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                        module: stage.module_key.clone(),
                        name: declaration.name().into(),
                        dependency: dependency.into_boxed_str(),
                        span: declaration.declaration_span(),
                    },
                );
            }
        }
    }
    Ok(())
}

fn callable_span(callable: &CallableDefinition) -> Span {
    match callable {
        CallableDefinition::Function(function) => function.span,
        CallableDefinition::Builtin(function) => function.span,
        CallableDefinition::Handler(handler) => handler.span,
    }
}

fn validate_public_proposition_tail_dependencies(
    stage: &ModuleStage,
    stages: &[ModuleStage],
    imports: &CanonicalParsedImportResult,
    declaration: &CanonicalCheckedDeclaration,
    tail: &ash_parser::surface::PropositionTail,
    type_dependencies: &mut Vec<String>,
    dependencies: &mut Vec<String>,
) -> Result<(), CanonicalCheckedModuleFinalizationError> {
    for clause in &tail.clauses {
        match &clause.kind {
            ash_parser::surface::PropositionClauseKind::Equality { lhs, rhs, .. }
            | ash_parser::surface::PropositionClauseKind::Disequality { lhs, rhs, .. } => {
                collect_surface_type_names(lhs, type_dependencies);
                collect_surface_type_names(rhs, type_dependencies);
            }
            ash_parser::surface::PropositionClauseKind::InterfaceBound {
                subject,
                interface,
                ..
            } => {
                collect_surface_type_names(subject, type_dependencies);
                collect_surface_type_names(interface, type_dependencies);
            }
            ash_parser::surface::PropositionClauseKind::NamedPredicate { name, args, .. } => {
                validate_public_namespace_dependency(
                    stage,
                    stages,
                    imports,
                    declaration,
                    name,
                    CanonicalNamespace::Proposition,
                    clause.span,
                )?;
                dependencies.push(name.to_string());
                for argument in args {
                    collect_surface_type_names(argument, type_dependencies);
                }
            }
        }
    }
    if let Some(where_row) = &tail.row {
        validate_effect_row_dependencies(
            stage,
            stages,
            imports,
            declaration,
            &where_row.row,
            dependencies,
        )?;
    }
    Ok(())
}

fn collect_surface_type_names(ty: &SurfaceType, names: &mut Vec<String>) {
    match ty {
        SurfaceType::Name(name) => names.push(name.to_string()),
        SurfaceType::List(element) => collect_surface_type_names(element, names),
        SurfaceType::Tuple(elements) => {
            for element in elements {
                collect_surface_type_names(element, names);
            }
        }
        SurfaceType::Record(fields) => {
            for (_, field_type) in fields {
                collect_surface_type_names(field_type, names);
            }
        }
        SurfaceType::Constructor { name, args } => {
            names.push(name.to_string());
            for argument in args {
                collect_surface_type_names(argument, names);
            }
        }
        SurfaceType::Fn(args, _, result) => {
            for argument in args {
                collect_surface_type_names(argument, names);
            }
            collect_surface_type_names(result, names);
        }
        SurfaceType::Associated { base, .. } => collect_surface_type_names(base, names),
        SurfaceType::AssociatedFamilyProjection {
            interface, args, ..
        } => {
            names.push(interface.to_string());
            for argument in args {
                collect_surface_type_names(argument, names);
            }
        }
        SurfaceType::Hole { .. } | SurfaceType::Capability(_) => {}
    }
}

fn collect_type_pattern_names(pattern: &ash_parser::surface::TypePattern, names: &mut Vec<String>) {
    match pattern {
        ash_parser::surface::TypePattern::Constructor { name, args, .. } => {
            names.push(name.to_string());
            for argument in args {
                collect_type_pattern_names(argument, names);
            }
        }
        ash_parser::surface::TypePattern::Var { .. }
        | ash_parser::surface::TypePattern::Wildcard { .. } => {}
    }
}

fn signature_error(
    module: &ModuleKey,
    name: &str,
    span: Span,
    reason: String,
) -> CanonicalCheckedModuleFinalizationError {
    CanonicalCheckedModuleFinalizationError::Signature {
        module: module.clone(),
        name: name.to_owned().into_boxed_str(),
        span,
        reason: reason.into_boxed_str(),
    }
}

fn definition_visibility(definition: &Definition) -> Visibility {
    match definition {
        Definition::Notation(definition) => definition.visibility.clone(),
        Definition::Macro(definition) => definition.visibility.clone(),
        Definition::Capability(definition) => definition.visibility.clone(),
        Definition::ResourceType(definition) => definition.visibility.clone(),
        Definition::Type(definition) => definition.visibility.clone(),
        Definition::Newtype(definition) => definition.visibility.clone(),
        Definition::EffectAlias(definition) => definition.visibility.clone(),
        Definition::EffectGroup(definition) => definition.visibility.clone(),
        Definition::DataKind(definition) => definition.visibility.clone(),
        Definition::TypeFn(definition) => definition.visibility.clone(),
        Definition::PropositionPredicate(definition) => definition.visibility.clone(),
        Definition::Policy(definition) => definition.visibility.clone(),
        Definition::Role(definition) => definition.visibility.clone(),
        Definition::Interface(definition) => definition.visibility.clone(),
        Definition::Impl(definition) => definition.visibility.clone(),
        Definition::Function(definition) => definition.visibility.clone(),
        Definition::Handler(definition) => definition.visibility.clone(),
        Definition::BuiltinFn(definition) => definition.visibility.clone(),
        Definition::SealedDomain(definition) => definition.visibility.clone(),
        Definition::Law(definition) => definition.visibility.clone(),
        Definition::Proof(definition) => definition.visibility.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use ash_core::module_graph::ModuleKey;
    use ash_parser::CanonicalExpandedModuleGraph;

    use super::*;
    use crate::canonical_module_collection::{
        CanonicalDeclarationKind, CanonicalNamespace, collect_canonical_expanded_module_graph,
    };
    use crate::canonical_parsed_import_resolver::{
        GraphResolver, clone_with_binding_declaration_span, clone_with_binding_defining_target,
        clone_with_binding_local_name, clone_with_binding_lookup_namespace,
        clone_with_binding_source_ordinal, clone_with_public_use_binding_declaration_span,
        clone_with_public_use_binding_reexport, resolve_parsed_imports_from_collection,
    };

    static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

    struct TempTree {
        root: PathBuf,
    }

    impl TempTree {
        fn new() -> Self {
            let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "ash-task-2073-forged-import-shape-{}-{serial}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("create forged-import fixture tree");
            Self { root }
        }

        fn write(&self, relative: &str, source: &str) -> PathBuf {
            let path = self.root.join(relative);
            fs::create_dir_all(path.parent().expect("fixture path has a parent"))
                .expect("create forged-import fixture parent");
            fs::write(&path, source).expect("write forged-import fixture");
            path
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn assert_public_use_through_non_public_nested_module_rejects(
        module_declaration: &str,
        fixture_label: &str,
    ) {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod outer;");
        tree.write("src/outer.ash", "pub mod inner;");
        tree.write(
            "src/inner.ash",
            format!(
                "{module_declaration} pub use crate::outer::inner::hidden::expose as exported;"
            )
            .as_str(),
        );
        tree.write(
            "src/hidden.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .unwrap_or_else(|error| {
                panic!("{fixture_label} fixture resolves through the parser graph: {error:?}")
            });
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed).unwrap_or_else(|error| {
            panic!("{fixture_label} fixture expands through the parser graph: {error:?}")
        });
        let collection =
            collect_canonical_expanded_module_graph(&expanded).unwrap_or_else(|error| {
                panic!("{fixture_label} fixture collection succeeds: {error:?}")
            });
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .unwrap_or_else(|error| {
                panic!("{fixture_label} fixture import resolution succeeds: {error:?}")
            });
        let inner = root
            .child("outer")
            .and_then(|key| key.child("inner"))
            .expect("nested fixture key is canonical");
        assert!(imports.binding(&inner, "exported").is_some());
        assert!(imports.public_uses().iter().any(|public_use| {
            public_use.importing_module() == &inner
                && public_use.binding().local_name() == "exported"
        }));

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(
            matches!(
                result,
                Err(CanonicalCheckedModuleFinalizationError::NonExportedPublicUseModulePath { .. })
            ),
            "{fixture_label} public use must reject its non-public nested defining path: {result:?}"
        );
    }

    #[test]
    fn public_use_nested_private_path_diagnostic_preserves_access_context() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod outer;");
        tree.write("src/outer.ash", "pub mod inner;");
        tree.write(
            "src/inner.ash",
            "mod hidden; pub use crate::outer::inner::hidden::expose as exported;",
        );
        tree.write(
            "src/hidden.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("nested-private public-use fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("nested-private public-use fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("nested-private public-use fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("nested-private public-use fixture import resolution succeeds");
        let inner = root
            .child("outer")
            .and_then(|key| key.child("inner"))
            .expect("nested fixture key is canonical");
        let binding = imports
            .binding(&inner, "exported")
            .expect("nested-private fixture retains the staged binding");
        let use_span = binding.use_span();

        let error = finalize_canonical_module_collection(&expanded, &collection, &imports)
            .expect_err("nested-private public use must be rejected");
        let diagnostic = error.to_string();
        assert!(
            diagnostic.contains(&format!("{use_span:?}")),
            "diagnostic must identify the use span {use_span:?}: {diagnostic}"
        );
        assert!(
            diagnostic.contains("crate::outer::inner::hidden::expose"),
            "diagnostic must identify the attempted access path: {diagnostic}"
        );
        assert!(
            diagnostic.contains("hidden"),
            "diagnostic must identify the offending path segment: {diagnostic}"
        );
        assert!(
            diagnostic.contains("private"),
            "diagnostic must identify the violated private visibility: {diagnostic}"
        );
    }

    #[test]
    fn public_use_projection_excludes_narrow_reexports() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api;");
        tree.write(
            "src/api.ash",
            "pub fn expose(value: Int) -> Int { value }\
             pub use crate::api::expose as public_alias;\
             pub(crate) use crate::api::expose as crate_alias;\
             pub(super) use crate::api::expose as super_alias;\
             pub(in crate::api) use crate::api::expose as restricted_alias;",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("narrow-reexport fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("narrow-reexport fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("narrow-reexport fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("narrow-reexport fixture import resolution succeeds");
        let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
            .expect("narrow re-export projection should finalize atomically");
        let api = root.child("api").expect("api fixture key is canonical");
        let interface = finalized
            .module(&api)
            .expect("narrow-reexport fixture publishes the api interface");

        assert!(interface.public_export("public_alias").is_some());
        for narrow_alias in ["crate_alias", "super_alias", "restricted_alias"] {
            assert!(
                interface.public_export(narrow_alias).is_none(),
                "{narrow_alias} must not enter the external public export projection"
            );
        }
    }

    #[test]
    fn public_use_projection_does_not_promote_narrow_reexport() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api;");
        tree.write(
            "src/api.ash",
            "pub fn expose(value: Int) -> Int { value }\
             pub(crate) use crate::api::expose as crate_alias;\
             pub use crate::api::crate_alias as promoted_alias;",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("transitive-narrow-reexport fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("transitive-narrow-reexport fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("transitive-narrow-reexport fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("transitive-narrow-reexport fixture import resolution succeeds");
        let api = root.child("api").expect("api fixture key is canonical");
        assert!(imports.binding(&api, "promoted_alias").is_some());

        let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
            .expect("transitive narrow-to-public re-export should finalize successfully");
        let interface = finalized
            .module(&api)
            .expect("transitive-narrow-reexport fixture publishes the api interface");
        assert!(
            interface.public_export("promoted_alias").is_none(),
            "a pub use must not promote an intermediate pub(crate) alias into external exports"
        );
    }

    #[test]
    fn public_use_nested_private_module_path_rejects() {
        assert_public_use_through_non_public_nested_module_rejects("mod hidden;", "private");
    }

    #[test]
    fn public_use_nested_pub_crate_module_path_rejects() {
        assert_public_use_through_non_public_nested_module_rejects(
            "pub(crate) mod hidden;",
            "pub-crate",
        );
    }

    #[test]
    fn public_use_nested_pub_super_module_path_rejects() {
        assert_public_use_through_non_public_nested_module_rejects(
            "pub(super) mod hidden;",
            "pub-super",
        );
    }

    #[test]
    fn public_use_nested_restricted_to_allowed_module_path_rejects() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod allowed;");
        tree.write("src/allowed.ash", "pub mod outer;");
        tree.write("src/outer.ash", "pub mod inner;");
        tree.write(
            "src/inner.ash",
            "pub(in crate::allowed) mod hidden;\
             pub use crate::allowed::outer::inner::hidden::expose as exported;",
        );
        tree.write(
            "src/hidden.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("restricted-module public-use fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("restricted-module public-use fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("restricted-module public-use fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("restricted-module public-use fixture import resolution succeeds");
        let inner = root
            .child("allowed")
            .and_then(|key| key.child("outer"))
            .and_then(|key| key.child("inner"))
            .expect("restricted nested fixture key is canonical");
        assert!(imports.binding(&inner, "exported").is_some());

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(matches!(
            result,
            Err(CanonicalCheckedModuleFinalizationError::NonExportedPublicUseModulePath { .. })
        ));
    }

    #[test]
    fn public_use_nested_restricted_module_path_rejects() {
        public_use_nested_restricted_to_allowed_module_path_rejects();
    }

    fn staged_constructor_fixture(
        expanded: &CanonicalExpandedModuleGraph,
        collection: &CanonicalModuleCollection,
        module_key: &ModuleKey,
    ) -> ModuleStage {
        let snapshot = collection
            .internal_snapshot(module_key)
            .expect("constructor fixture retains its internal snapshot");
        let origin = expanded
            .parsed_graph()
            .module_unit(module_key)
            .expect("constructor fixture retains its parsed module")
            .artifact()
            .origin()
            .clone();
        let raw_definitions = snapshot
            .entries()
            .filter_map(|entry| entry.raw_definition().cloned())
            .fold(Vec::new(), |mut definitions, definition| {
                if !definitions.contains(&definition) {
                    definitions.push(definition);
                }
                definitions
            });
        let definitions = snapshot
            .entries()
            .map(|entry| checked_declaration_skeleton(entry, origin.clone(), &[]))
            .collect();
        ModuleStage {
            module_key: module_key.clone(),
            origin,
            raw_definitions,
            definitions,
            callable_definitions: Vec::new(),
            implementation_members: Vec::new(),
        }
    }

    #[test]
    fn red_structural_module_identity_and_target_reject_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api;");
        tree.write("src/api.ash", "pub fn expose(value: Int) -> Int { value }");
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let api = root.child("api").expect("fixture child key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("structural-module forged fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("structural-module forged fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("structural-module forged fixture collection succeeds");
        let child_stage = staged_constructor_fixture(&expanded, &collection, &api);

        let mut forged_root_stage = staged_constructor_fixture(&expanded, &collection, &root);
        let module_declaration = forged_root_stage
            .definitions
            .iter_mut()
            .find(|declaration| declaration.kind() == CanonicalDeclarationKind::ModuleDecl)
            .expect("root stage retains its structural module declaration");
        module_declaration.fact = CanonicalCheckedDeclarationFact::StructuralModule {
            module: root.child("forged").expect("forged child key is canonical"),
        };
        let forged_stages = vec![forged_root_stage, child_stage];
        let forged_error =
            validate_structural_module_declarations(&forged_stages[0], &forged_stages)
                .expect_err("a forged structural child identity must reject before publication");
        assert!(matches!(
            forged_error,
            CanonicalCheckedModuleFinalizationError::StructuralModuleIdentityMismatch {
                ref module,
                ref name,
                ..
            } if module == &root && name.as_ref() == "api"
        ));

        let missing_stage = staged_constructor_fixture(&expanded, &collection, &root);
        let missing_stages = vec![missing_stage];
        let missing_error =
            validate_structural_module_declarations(&missing_stages[0], &missing_stages)
                .expect_err("a structural child absent from the collected stages must reject");
        assert!(matches!(
            missing_error,
            CanonicalCheckedModuleFinalizationError::MissingStructuralModuleTarget {
                ref module,
                ref name,
                ..
            } if module == &root && name.as_ref() == "api"
        ));
    }

    #[test]
    fn red_public_constructor_projection_preserves_parent_and_rejects_private_parent_forgery() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub type Choice = Left { value: Int } | Right; pub newtype UserId = UserId(Int); newtype Hidden = Hidden(Int);",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("constructor fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("constructor fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("constructor fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("constructor fixture import resolution succeeds");

        let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
            .expect("valid ordinary and newtype constructors remain export-closed");
        let interface = finalized
            .module(&root)
            .expect("constructor fixture publishes its root interface");
        let choice = interface
            .private_declaration("Choice")
            .expect("ordinary parent remains in the private view");
        let left = interface
            .public_export("Left")
            .expect("public ordinary constructor is publicly reachable");
        assert!(matches!(
            left.declaration().fact(),
            CanonicalCheckedDeclarationFact::Constructor { parent, name }
                if parent == choice.identity() && name.as_ref() == "Left"
        ));
        let user_id = interface
            .private_declaration("UserId")
            .expect("newtype parent remains in the private view");
        let user_id_constructor = interface
            .public_export_in_namespace(CanonicalNamespace::ValueCallable, "UserId")
            .expect("public newtype constructor is publicly reachable");
        assert!(matches!(
            user_id_constructor.declaration().fact(),
            CanonicalCheckedDeclarationFact::Constructor { parent, name }
                if parent == user_id.identity() && name.as_ref() == "UserId"
        ));

        let mut stage = staged_constructor_fixture(&expanded, &collection, &root);
        let hidden_parent = stage
            .definitions
            .iter()
            .find(|declaration| {
                declaration.kind() == CanonicalDeclarationKind::Newtype
                    && declaration.name() == "Hidden"
                    && declaration.identity().canonical_parent().is_none()
            })
            .expect("private newtype parent remains in the staged view")
            .identity()
            .clone();
        let hidden_constructor = stage
            .definitions
            .iter_mut()
            .find(|declaration| {
                declaration.kind() == CanonicalDeclarationKind::Function
                    && declaration.name() == "Hidden"
                    && declaration.identity().canonical_parent() == Some(&hidden_parent)
            })
            .expect("private newtype constructor remains parent-scoped in the staged view");
        hidden_constructor.visibility = Visibility::Public;

        let stages = [stage];
        let result = validate_public_declaration_support(&stages[0]);
        assert!(
            matches!(
                result,
                Err(CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                    ref dependency,
                    ..
                }) if dependency.as_ref() == "Hidden"
            ),
            "a forged public constructor under a private parent must reject atomically: {result:?}"
        );
    }

    #[test]
    fn red_forged_constructor_parent_identity_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub type Choice = Left { value: Int } | Right; pub newtype UserId = UserId(Int);",
        );
        let root = ModuleKey::root("app").expect("fixture key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("constructor identity fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("constructor identity fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("constructor identity fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("constructor identity fixture import resolution succeeds");

        let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
            .expect("valid constructor parent projections remain export-closed");
        let interface = finalized
            .module(&root)
            .expect("constructor identity fixture publishes its root interface");
        let choice = interface
            .private_declaration("Choice")
            .expect("ordinary parent remains in the private view");
        let user_id = interface
            .private_declaration("UserId")
            .expect("newtype parent remains in the private view");
        assert!(matches!(
            interface
                .public_export("Left")
                .expect("ordinary constructor remains publicly reachable")
                .declaration()
                .fact(),
            CanonicalCheckedDeclarationFact::Constructor { parent, .. }
                if parent == choice.identity()
        ));
        assert!(matches!(
            interface
                .public_export_in_namespace(CanonicalNamespace::ValueCallable, "UserId")
                .expect("newtype constructor remains publicly reachable")
                .declaration()
                .fact(),
            CanonicalCheckedDeclarationFact::Constructor { parent, .. }
                if parent == user_id.identity()
        ));

        let mut stage = staged_constructor_fixture(&expanded, &collection, &root);
        let user_id_identity = stage
            .definitions
            .iter()
            .find(|declaration| {
                declaration.kind() == CanonicalDeclarationKind::Newtype
                    && declaration.name() == "UserId"
                    && declaration.identity().canonical_parent().is_none()
            })
            .expect("newtype parent remains in the staged view")
            .identity()
            .clone();
        let left_constructor = stage
            .definitions
            .iter_mut()
            .find(|declaration| {
                declaration.kind() == CanonicalDeclarationKind::Function
                    && declaration.name() == "Left"
            })
            .expect("ordinary constructor remains in the staged view");
        left_constructor.fact = CanonicalCheckedDeclarationFact::Constructor {
            parent: user_id_identity,
            name: "Left".into(),
        };

        let stages = [stage];
        let result = validate_public_declaration_support(&stages[0]);
        assert!(
            matches!(
                result,
                Err(CanonicalCheckedModuleFinalizationError::GraphMismatch)
            ),
            "a constructor fact with a forged parent identity must reject atomically: {result:?}"
        );

        let mut stage = staged_constructor_fixture(&expanded, &collection, &root);
        let left_constructor = stage
            .definitions
            .iter_mut()
            .find(|declaration| {
                declaration.kind() == CanonicalDeclarationKind::Function
                    && declaration.name() == "Left"
            })
            .expect("ordinary constructor remains in the staged view");
        let parent = left_constructor
            .identity()
            .canonical_parent()
            .expect("ordinary constructor retains its parent identity")
            .clone();
        left_constructor.fact = CanonicalCheckedDeclarationFact::Constructor {
            parent,
            name: "Ghost".into(),
        };
        let stages = [stage];
        let result = validate_public_declaration_support(&stages[0]);
        assert!(
            matches!(
                result,
                Err(CanonicalCheckedModuleFinalizationError::GraphMismatch)
            ),
            "a constructor fact naming a non-member must reject atomically: {result:?}"
        );

        let mut stage = staged_constructor_fixture(&expanded, &collection, &root);
        let user_id_identity = stage
            .definitions
            .iter()
            .find(|declaration| {
                declaration.name() == "UserId"
                    && declaration.identity().canonical_parent().is_none()
            })
            .expect("newtype parent remains in the staged view")
            .identity()
            .clone();
        stage
            .definitions
            .retain(|declaration| declaration.identity() != &user_id_identity);
        let stages = [stage];
        let result = validate_public_declaration_support(&stages[0]);
        assert!(
            matches!(
                result,
                Err(CanonicalCheckedModuleFinalizationError::GraphMismatch)
            ),
            "a constructor with a missing parent must reject atomically: {result:?}"
        );
    }

    #[test]
    fn forged_imported_binding_namespace_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api; use crate::api::expose;");
        tree.write("src/api.ash", "pub fn expose(value: Int) -> Int { value }");
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("forged-import fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("forged-import fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("forged-import fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("forged-import fixture import resolution succeeds");
        let original = imports
            .binding(&root, "expose")
            .expect("the real import result contains the public function binding");
        assert_eq!(
            original.lookup_key().namespace(),
            CanonicalNamespace::ValueCallable
        );

        let forged = clone_with_binding_lookup_namespace(
            &imports,
            &root,
            "expose",
            CanonicalNamespace::TypeDomain,
        )
        .expect("the real import result contains the binding to forge");
        let forged_binding = forged
            .binding(&root, "expose")
            .expect("the forged import result retains the binding");
        assert_eq!(
            forged_binding.defining_identity(),
            original.defining_identity()
        );
        assert_eq!(forged_binding.local_name(), original.local_name());
        assert_eq!(forged_binding.origin(), original.origin());
        assert_eq!(
            forged_binding.declaration_visibility(),
            original.declaration_visibility()
        );
        assert_eq!(
            forged_binding.lookup_key().namespace(),
            CanonicalNamespace::TypeDomain
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &forged);
        assert!(matches!(
            result,
            Err(
                CanonicalCheckedModuleFinalizationError::BindingShapeMismatch {
                    module,
                    name,
                    binding_namespace: CanonicalNamespace::TypeDomain,
                    target_namespace: CanonicalNamespace::ValueCallable,
                    binding_kind: CanonicalDeclarationKind::Function,
                    target_kind: CanonicalDeclarationKind::Function,
                }
            ) if module == root && name.as_ref() == "expose"
        ));
    }

    #[test]
    fn imported_binding_through_inherited_module_is_accepted_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "mod provider; use crate::provider::expose;");
        tree.write(
            "src/provider.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("inherited-module fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("inherited-module fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("inherited-module fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("inherited-module fixture import resolution succeeds");
        assert!(imports.binding(&root, "expose").is_some());

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(
            result.is_ok(),
            "a root import through an inherited module should be accepted: {result:?}"
        );
    }

    #[test]
    fn imported_binding_through_pub_super_module_boundary_is_accepted_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub mod parent; use crate::parent::inner::expose;",
        );
        tree.write("src/parent.ash", "pub(super) mod inner;");
        tree.write(
            "src/inner.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("pub-super fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("pub-super fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("pub-super fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("pub-super fixture import resolution succeeds");
        assert!(imports.binding(&root, "expose").is_some());

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(
            result.is_ok(),
            "a root import across a pub(super) module boundary should be accepted: {result:?}"
        );
    }

    #[test]
    fn imported_binding_through_pub_self_module_is_accepted_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub(self) mod provider; use crate::provider::expose;",
        );
        tree.write(
            "src/provider.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("pub-self fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("pub-self fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("pub-self fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("pub-self fixture import resolution succeeds");
        let binding = imports
            .binding(&root, "expose")
            .expect("pub-self fixture retains its imported binding");
        assert_eq!(
            binding.defining_identity().module_key(),
            &root.child("provider").expect("provider key is canonical")
        );
        assert_eq!(
            expanded
                .parsed_graph()
                .module_unit(&root)
                .expect("root module is retained")
                .body()
                .module_decls()
                .first()
                .expect("provider declaration is retained")
                .visibility,
            Visibility::Self_
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(
            result.is_ok(),
            "a root import through a pub(self) module should be accepted: {result:?}"
        );
    }

    #[test]
    fn imported_binding_through_pub_crate_module_is_accepted_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub(crate) mod provider; use crate::provider::expose;",
        );
        tree.write(
            "src/provider.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("pub-crate fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("pub-crate fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("pub-crate fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("pub-crate fixture import resolution succeeds");
        assert!(imports.binding(&root, "expose").is_some());

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(
            result.is_ok(),
            "a root import through a pub(crate) module should be accepted: {result:?}"
        );
    }

    #[test]
    fn imported_binding_through_restricted_crate_module_is_accepted_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub(in crate) mod provider; use crate::provider::expose;",
        );
        tree.write(
            "src/provider.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("restricted-crate fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("restricted-crate fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("restricted-crate fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("restricted-crate fixture import resolution succeeds");
        assert!(imports.binding(&root, "expose").is_some());

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(
            result.is_ok(),
            "a root import through a restricted crate module should be accepted: {result:?}"
        );
    }

    #[test]
    fn public_use_through_private_module_path_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "mod hidden; pub use crate::hidden::expose as exported;",
        );
        tree.write(
            "src/hidden.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("private-module public-use fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("private-module public-use fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("private-module public-use fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("private-module public-use fixture import resolution succeeds");
        assert!(imports.binding(&root, "exported").is_some());
        assert!(
            imports
                .public_uses()
                .iter()
                .any(|public_use| public_use.importing_module() == &root
                    && public_use.binding().local_name() == "exported")
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(matches!(
            result,
            Err(
                CanonicalCheckedModuleFinalizationError::NonExportedPublicUseModulePath {
                    module,
                    name,
                    defining_module,
                    ..
                }
            ) if module == root
                && name.as_ref() == "exported"
                && defining_module == root.child("hidden").expect("hidden key is canonical")
        ));
    }

    #[test]
    fn public_use_through_public_module_path_preserves_closure() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub mod provider; pub use crate::provider::expose as exported;",
        );
        tree.write(
            "src/provider.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("public-module public-use fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("public-module public-use fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("public-module public-use fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("public-module public-use fixture import resolution succeeds");
        assert!(imports.binding(&root, "exported").is_some());
        assert!(
            imports
                .public_uses()
                .iter()
                .any(|public_use| public_use.importing_module() == &root
                    && public_use.binding().local_name() == "exported")
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &imports);
        assert!(
            result.is_ok(),
            "a public use through a public module path must be accepted: {result:?}"
        );
    }

    #[test]
    fn forged_imported_binding_private_defining_module_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write(
            "src/main.ash",
            "pub mod provider; pub mod outer; use crate::provider::expose;",
        );
        tree.write(
            "src/provider.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        tree.write("src/outer.ash", "mod hidden;");
        tree.write(
            "src/hidden.ash",
            "pub fn expose(value: Int) -> Int { value }",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let provider = root
            .child("provider")
            .expect("provider fixture key is canonical");
        let outer = root.child("outer").expect("outer fixture key is canonical");
        let hidden = outer
            .child("hidden")
            .expect("hidden fixture key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("forged-module-path fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("forged-module-path fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("forged-module-path fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("forged-module-path fixture import resolution succeeds");
        let original = imports
            .binding(&root, "expose")
            .expect("the real import result contains the public provider binding");
        assert_eq!(original.defining_identity().module_key(), &provider);
        let hidden_entry = collection
            .internal_snapshot(&hidden)
            .expect("the private hidden module has an internal snapshot")
            .entries()
            .find(|entry| {
                entry.namespace() == CanonicalNamespace::ValueCallable
                    && entry.declared_name() == Some("expose")
            })
            .expect("the private hidden module contains the same-named public function");
        let hidden_visibility = match hidden_entry
            .raw_definition()
            .expect("the private hidden function retains its definition")
        {
            Definition::Function(function) => &function.visibility,
            other => panic!("expected a collected function, got {other:?}"),
        };
        let hidden_origin = expanded
            .parsed_graph()
            .module_unit(&hidden)
            .expect("the private hidden module has a parsed unit")
            .artifact()
            .origin()
            .clone();

        let forged = clone_with_binding_defining_target(
            &imports,
            &root,
            "expose",
            hidden_entry,
            hidden_visibility,
            &hidden_origin,
        )
        .expect("the real import result contains the binding to forge");
        let forged_binding = forged
            .binding(&root, "expose")
            .expect("the forged import result retains the binding");
        assert_eq!(forged_binding.defining_identity().module_key(), &hidden);
        assert_eq!(forged_binding.lookup_key(), original.lookup_key());
        assert_eq!(forged_binding.local_name(), original.local_name());
        assert_eq!(forged_binding.use_span(), original.use_span());
        assert_eq!(forged_binding.member_span(), original.member_span());
        assert_eq!(
            forged_binding.import_visibility(),
            original.import_visibility()
        );
        assert_eq!(forged_binding.declaration_visibility(), hidden_visibility);
        assert_eq!(forged_binding.origin(), &hidden_origin);
        assert_eq!(
            forged_binding.declaration_span(),
            hidden_entry.source_anchor()
        );
        assert_eq!(
            forged_binding.source_ordinal(),
            match hidden_entry.identity().origin_key() {
                CanonicalDeclarationOriginKey::Source { source_ordinal }
                | CanonicalDeclarationOriginKey::Expanded { source_ordinal, .. } => *source_ordinal,
            }
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &forged);
        assert!(matches!(
            result,
            Err(
                CanonicalCheckedModuleFinalizationError::BindingModuleVisibilityMismatch {
                    module,
                    name,
                }
            ) if module == root && name.as_ref() == "expose"
        ));
    }

    #[test]
    fn imported_impl_operation_private_defining_module_path_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub(crate) mod provider; pub mod api;");
        tree.write(
            "src/provider.ash",
            "pub interface Eq<A> { equiv(A, A) -> Bool } pub impl Eq<Int> { equiv(a, b) = a == b }",
        );
        tree.write(
            "src/api.ash",
            "use crate::provider::Eq; pub effect alias Published = { Eq::equiv };",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let provider = root
            .child("provider")
            .expect("provider fixture key is canonical");
        let api = root.child("api").expect("api fixture key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("private-provider fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("private-provider fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("private-provider fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("private-provider fixture import resolution succeeds");
        let implementation = collection
            .internal_snapshot(&provider)
            .expect("provider internal snapshot is retained")
            .entries()
            .find(|entry| {
                entry.namespace() == CanonicalNamespace::ImplementationRegistry
                    && entry.declared_name() == Some("Eq")
            })
            .expect("provider implementation is retained in the internal snapshot");
        let implementation_visibility = match implementation
            .raw_definition()
            .expect("implementation retains its source definition")
        {
            Definition::Impl(definition) => &definition.visibility,
            other => panic!("expected implementation definition, got {other:?}"),
        };
        let provider_origin = expanded
            .parsed_graph()
            .module_unit(&provider)
            .expect("provider parsed module is retained")
            .artifact()
            .origin()
            .clone();
        let forged = clone_with_binding_defining_target(
            &imports,
            &api,
            "Eq",
            implementation,
            implementation_visibility,
            &provider_origin,
        )
        .expect("the real API Eq binding is available to forge");
        let forged = clone_with_binding_lookup_namespace(
            &forged,
            &api,
            "Eq",
            CanonicalNamespace::ImplementationRegistry,
        )
        .expect("the forged API Eq binding retains its map entry");
        assert_eq!(
            forged
                .binding(&api, "Eq")
                .expect("the forged API Eq binding is retained")
                .defining_identity()
                .module_key(),
            &provider
        );
        assert_eq!(
            forged
                .binding(&api, "Eq")
                .expect("the forged API Eq binding is retained")
                .lookup_key()
                .namespace(),
            CanonicalNamespace::ImplementationRegistry
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &forged);
        assert!(matches!(
            result,
            Err(CanonicalCheckedModuleFinalizationError::PrivateExportDependency {
                module,
                name,
                dependency,
                ..
            }) if module == api && name.as_ref() == "Published" && dependency.as_ref() == "Eq"
        ));
    }

    #[test]
    fn imported_impl_operation_public_defining_module_path_preserves_closure() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod provider; pub mod api;");
        tree.write(
            "src/provider.ash",
            "pub interface Eq<A> { equiv(A, A) -> Bool } pub impl Eq<Int> { equiv(a, b) = a == b }",
        );
        tree.write(
            "src/api.ash",
            "use crate::provider::Eq; pub effect alias Published = { Eq::equiv };",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let provider = root
            .child("provider")
            .expect("provider fixture key is canonical");
        let api = root.child("api").expect("api fixture key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("public-provider fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("public-provider fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("public-provider fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("public-provider fixture import resolution succeeds");
        let implementation = collection
            .internal_snapshot(&provider)
            .expect("provider internal snapshot is retained")
            .entries()
            .find(|entry| {
                entry.namespace() == CanonicalNamespace::ImplementationRegistry
                    && entry.declared_name() == Some("Eq")
            })
            .expect("provider implementation is retained in the internal snapshot");
        let implementation_visibility = match implementation
            .raw_definition()
            .expect("implementation retains its source definition")
        {
            Definition::Impl(definition) => &definition.visibility,
            other => panic!("expected implementation definition, got {other:?}"),
        };
        let provider_origin = expanded
            .parsed_graph()
            .module_unit(&provider)
            .expect("provider parsed module is retained")
            .artifact()
            .origin()
            .clone();
        let forged = clone_with_binding_defining_target(
            &imports,
            &api,
            "Eq",
            implementation,
            implementation_visibility,
            &provider_origin,
        )
        .expect("the real API Eq binding is available to forge");
        let forged = clone_with_binding_lookup_namespace(
            &forged,
            &api,
            "Eq",
            CanonicalNamespace::ImplementationRegistry,
        )
        .expect("the forged API Eq binding retains its map entry");
        assert_eq!(
            forged
                .binding(&api, "Eq")
                .expect("the forged API Eq binding is retained")
                .defining_identity()
                .module_key(),
            &provider
        );
        assert_eq!(
            forged
                .binding(&api, "Eq")
                .expect("the forged API Eq binding is retained")
                .lookup_key()
                .namespace(),
            CanonicalNamespace::ImplementationRegistry
        );

        let finalized = finalize_canonical_module_collection(&expanded, &collection, &forged)
            .expect("a public effect row may expose an imported public implementation operation");
        assert!(
            finalized
                .module(&api)
                .expect("API interface is finalized")
                .public_export("Published")
                .is_some(),
            "the public effect alias remains export-closed"
        );
        assert!(
            finalized
                .module(&provider)
                .expect("provider interface is finalized")
                .public_export_in_namespace(CanonicalNamespace::ImplementationRegistry, "Eq")
                .is_some(),
            "the forged implementation target remains export-closed in its defining module"
        );
    }

    #[test]
    fn forged_imported_effect_row_binding_shape_rejects_before_module_path_diagnostic() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub(crate) mod provider; pub mod api;");
        tree.write(
            "src/provider.ash",
            "pub fn combine(value: Int) -> Int { value }",
        );
        tree.write(
            "src/api.ash",
            "use crate::provider::combine; pub effect alias Published = { combine::missing };",
        );
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let provider = root
            .child("provider")
            .expect("provider fixture key is canonical");
        let api = root.child("api").expect("api fixture key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("malformed-row-carrier fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("malformed-row-carrier fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("malformed-row-carrier fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("malformed-row-carrier fixture import resolution succeeds");
        let original = imports
            .binding(&api, "combine")
            .expect("the real API import contains the public function binding");
        assert_eq!(original.defining_identity().module_key(), &provider);
        assert_eq!(
            original.lookup_key().namespace(),
            CanonicalNamespace::ValueCallable
        );

        let forged = clone_with_binding_lookup_namespace(
            &imports,
            &api,
            "combine",
            CanonicalNamespace::ImplementationRegistry,
        )
        .expect("the real API import contains the binding to forge");
        let forged_binding = forged
            .binding(&api, "combine")
            .expect("the forged API import retains the binding");
        assert_eq!(
            forged_binding.defining_identity(),
            original.defining_identity()
        );
        assert_eq!(
            forged_binding.lookup_key().namespace(),
            CanonicalNamespace::ImplementationRegistry
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &forged);
        let result_debug = format!("{result:?}");
        assert!(
            matches!(
                result,
                Err(CanonicalCheckedModuleFinalizationError::BindingShapeMismatch {
                    module,
                    name,
                    binding_namespace: CanonicalNamespace::ImplementationRegistry,
                    target_namespace: CanonicalNamespace::ValueCallable,
                    binding_kind: CanonicalDeclarationKind::Function,
                    target_kind: CanonicalDeclarationKind::Function,
                }) if module == api && name.as_ref() == "combine"
            ),
            "expected BindingShapeMismatch before path validation: {result_debug}"
        );
    }

    #[test]
    fn forged_imported_binding_local_name_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api; use crate::api::expose;");
        tree.write("src/api.ash", "pub fn expose(value: Int) -> Int { value }");
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("forged-import fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("forged-import fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("forged-import fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("forged-import fixture import resolution succeeds");
        let original = imports
            .binding(&root, "expose")
            .expect("the real import result contains the public function binding");
        assert_eq!(original.local_name(), "expose");

        let forged = clone_with_binding_local_name(&imports, &root, "expose", "forged")
            .expect("the real import result contains the binding to forge");
        let forged_binding = forged
            .binding(&root, "expose")
            .expect("the forged import result retains the original map key");
        assert_eq!(
            forged_binding.defining_identity(),
            original.defining_identity()
        );
        assert_eq!(forged_binding.local_name(), "forged");
        assert_eq!(forged_binding.lookup_key(), original.lookup_key());
        assert_eq!(forged_binding.origin(), original.origin());
        assert_eq!(
            forged_binding.declaration_visibility(),
            original.declaration_visibility()
        );

        let early_error = validate_import_binding_local_names(&forged)
            .expect_err("the forged local name must fail before imported binding consumers");
        assert!(matches!(
            early_error,
            CanonicalCheckedModuleFinalizationError::BindingLocalNameMismatch {
                module,
                authoritative_name,
                binding_local_name,
            } if module == root
                && authoritative_name.as_ref() == "expose"
                && binding_local_name.as_ref() == "forged"
        ));

        let result = finalize_canonical_module_collection(&expanded, &collection, &forged);
        assert!(matches!(
            result,
            Err(
                CanonicalCheckedModuleFinalizationError::BindingLocalNameMismatch {
                    module,
                    authoritative_name,
                    binding_local_name,
                }
            ) if module == root
                && authoritative_name.as_ref() == "expose"
                && binding_local_name.as_ref() == "forged"
        ));
    }

    #[test]
    fn forged_imported_binding_declaration_metadata_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api; use crate::api::expose;");
        tree.write("src/api.ash", "pub fn expose(value: Int) -> Int { value }");
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("forged-import fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("forged-import fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("forged-import fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("forged-import fixture import resolution succeeds");
        let original = imports
            .binding(&root, "expose")
            .expect("the real import result contains the public function binding");
        assert_eq!(original.local_name(), "expose");

        let original_span = original.declaration_span();
        let forged_span = Span::new(
            original_span.start + 1,
            original_span.end + 1,
            original_span.line + 1,
            original_span.column + 1,
        );
        let forged_span_imports =
            clone_with_binding_declaration_span(&imports, &root, "expose", forged_span)
                .expect("the real import result contains the binding to forge");
        let forged_span_binding = forged_span_imports
            .binding(&root, "expose")
            .expect("the forged span result retains the binding");
        assert_eq!(
            forged_span_binding.defining_identity(),
            original.defining_identity()
        );
        assert_eq!(forged_span_binding.lookup_key(), original.lookup_key());
        assert_eq!(forged_span_binding.local_name(), original.local_name());
        assert_eq!(forged_span_binding.origin(), original.origin());
        assert_eq!(
            forged_span_binding.declaration_visibility(),
            original.declaration_visibility()
        );
        assert_eq!(
            forged_span_binding.source_ordinal(),
            original.source_ordinal()
        );
        assert_ne!(forged_span_binding.declaration_span(), original_span);

        let declaration_span_error =
            finalize_canonical_module_collection(&expanded, &collection, &forged_span_imports)
                .expect_err("forged declaration span must reject before publication");
        assert!(matches!(
            declaration_span_error,
            CanonicalCheckedModuleFinalizationError::BindingDeclarationMetadataMismatch {
                module,
                name,
            } if module == root
                && name.as_ref() == "expose"
        ));

        let forged_source_ordinal_imports = clone_with_binding_source_ordinal(
            &imports,
            &root,
            "expose",
            original.source_ordinal() + 1,
        )
        .expect("the real import result contains the binding to forge");
        let forged_source_ordinal_binding = forged_source_ordinal_imports
            .binding(&root, "expose")
            .expect("the forged source ordinal result retains the binding");
        assert_eq!(
            forged_source_ordinal_binding.declaration_span(),
            original.declaration_span()
        );
        assert_ne!(
            forged_source_ordinal_binding.source_ordinal(),
            original.source_ordinal()
        );

        let source_ordinal_error = finalize_canonical_module_collection(
            &expanded,
            &collection,
            &forged_source_ordinal_imports,
        )
        .expect_err("forged source ordinal must reject before publication");
        assert!(matches!(
            source_ordinal_error,
            CanonicalCheckedModuleFinalizationError::BindingDeclarationMetadataMismatch {
                module,
                name,
            } if module == root
                && name.as_ref() == "expose"
        ));
    }

    #[test]
    fn forged_public_use_binding_reexport_flag_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api; pub use crate::api::expose;");
        tree.write("src/api.ash", "pub fn expose(value: Int) -> Int { value }");
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("forged-public-use fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("forged-public-use fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("forged-public-use fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("forged-public-use fixture import resolution succeeds");
        let authoritative = imports
            .binding(&root, "expose")
            .expect("the real import result contains the re-export binding");
        let staged_public_use = imports
            .public_uses()
            .iter()
            .find(|public_use| {
                public_use.importing_module() == &root
                    && public_use.binding().local_name() == "expose"
            })
            .expect("the real import result contains the staged public use");
        assert!(authoritative.is_reexport());
        assert!(staged_public_use.binding().is_reexport());

        let forged = clone_with_public_use_binding_reexport(&imports, &root, "expose", false)
            .expect("the real import result contains the public-use carrier to forge");
        assert!(
            forged
                .binding(&root, "expose")
                .expect("the forged result retains the authoritative binding")
                .is_reexport(),
            "the authoritative binding must remain unchanged"
        );
        let forged_public_use = forged
            .public_uses()
            .iter()
            .find(|public_use| {
                public_use.importing_module() == &root
                    && public_use.binding().local_name() == "expose"
            })
            .expect("the forged result retains the staged public use");
        assert!(!forged_public_use.binding().is_reexport());
        assert_eq!(
            forged_public_use.binding().defining_identity(),
            authoritative.defining_identity()
        );
        assert_eq!(
            forged_public_use.binding().local_name(),
            authoritative.local_name()
        );
        assert_eq!(
            forged_public_use.binding().lookup_key(),
            authoritative.lookup_key()
        );
        assert_eq!(forged_public_use.binding().origin(), authoritative.origin());
        assert_eq!(
            forged_public_use.binding().declaration_visibility(),
            authoritative.declaration_visibility()
        );
        assert_eq!(
            forged_public_use.binding().declaration_span(),
            authoritative.declaration_span()
        );
        assert_eq!(
            forged_public_use.binding().source_ordinal(),
            authoritative.source_ordinal()
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &forged);
        assert!(matches!(
            result,
            Err(CanonicalCheckedModuleFinalizationError::PublicUseBindingMismatch {
                module,
                name,
            }) if module == root && name.as_ref() == "expose"
        ));
    }

    #[test]
    fn forged_public_use_binding_declaration_span_rejects_atomically() {
        let tree = TempTree::new();
        let root_path = tree.write("src/main.ash", "pub mod api; pub use crate::api::expose;");
        tree.write("src/api.ash", "pub fn expose(value: Int) -> Int { value }");
        let root = ModuleKey::root("app").expect("fixture crate key is canonical");
        let parsed = GraphResolver::new()
            .resolve_root(root.clone(), root_path)
            .expect("forged-public-use fixture resolves through the parser graph");
        let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
            .expect("forged-public-use fixture expands through the parser graph");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("forged-public-use fixture collection succeeds");
        let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
            .expect("forged-public-use fixture import resolution succeeds");
        let authoritative = imports
            .binding(&root, "expose")
            .expect("the real import result contains the re-export binding");
        let staged_public_use = imports
            .public_uses()
            .iter()
            .find(|public_use| {
                public_use.importing_module() == &root
                    && public_use.binding().local_name() == "expose"
            })
            .expect("the real import result contains the staged public use");
        assert!(authoritative.is_reexport());
        assert!(staged_public_use.binding().is_reexport());

        let original_span = staged_public_use.binding().declaration_span();
        let forged_span = Span::new(
            original_span.start + 1,
            original_span.end + 1,
            original_span.line + 1,
            original_span.column + 1,
        );
        let forged =
            clone_with_public_use_binding_declaration_span(&imports, &root, "expose", forged_span)
                .expect("the real import result contains the public-use carrier to forge");
        assert_eq!(
            forged
                .binding(&root, "expose")
                .expect("the forged result retains the authoritative binding"),
            authoritative,
            "the authoritative binding must remain unchanged"
        );
        let forged_public_use = forged
            .public_uses()
            .iter()
            .find(|public_use| {
                public_use.importing_module() == &root
                    && public_use.binding().local_name() == "expose"
            })
            .expect("the forged result retains the staged public use");
        assert!(forged_public_use.binding().is_reexport());
        assert_eq!(
            forged_public_use.binding().defining_identity(),
            authoritative.defining_identity()
        );
        assert_eq!(
            forged_public_use.binding().lookup_key(),
            authoritative.lookup_key()
        );
        assert_eq!(
            forged_public_use.binding().local_name(),
            authoritative.local_name()
        );
        assert_ne!(
            forged_public_use.binding().declaration_span(),
            authoritative.declaration_span()
        );

        let result = finalize_canonical_module_collection(&expanded, &collection, &forged);
        assert!(matches!(
            result,
            Err(CanonicalCheckedModuleFinalizationError::PublicUseBindingMismatch {
                module,
                name,
            }) if module == root && name.as_ref() == "expose"
        ));
    }
}
