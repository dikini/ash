//! Type environment for tracking type definitions and constructor mappings
//!
//! Provides `TypeEnv` for managing type definitions and looking up constructors.

#![allow(clippy::result_large_err)]

use crate::error::{PropositionDiagnosticKind, TypeEnvError};
use crate::exhaustiveness::{MatchCoverage, check_match_exhaustive};
use crate::normalizer::{DefinitionalEqualityResult, Normalizer};
use crate::solver::TypeError;
use crate::types::{Substitution, Type, TypeVar, UnifyError, unify};
use crate::{Kind, QualifiedName};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{
    Pattern as CorePattern, TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload,
};
use ash_core::contract::{Contract as ContractMetadata, RuntimePostconditionContract};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedFamilyClosureMetadata, AssociatedFamilyDependencySummaryRef,
    AssociatedFamilyExportMode, AssociatedFamilyRevalidationMetadata, AssociatedFamilySummary,
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, ConstructorPayloadKind,
    ConstructorSummary, DomainConstructorId, DomainConstructorSummary, EffectRowAuthority,
    EffectRowExportClassification, EffectRowExportId, EffectRowExportSummary, EffectRowItemSummary,
    InterfaceIdentityId, InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary,
    ModuleSemanticSummaryValidationError, ModuleSummaryRef, PromotedConstructorId,
    PromotedConstructorSummary, PromotedDataKindId, PromotedDataKindSummary,
    PropositionFactSummary, PropositionPredicateId, PropositionPredicateParamSummary,
    PropositionPredicateSummary, RepresentationExposure, SealedDomainId, SealedDomainSummary,
    SourceAnchor, SourceOrigin, StructuralFieldStatus, SummaryVersion, TypeDeclId, TypeDeclSummary,
    TypeDeclarationKind, TypeFunctionClosureMetadata, TypeFunctionDependencySummaryRef,
    TypeFunctionExportMode, TypeFunctionParamSummary, TypeFunctionRevalidationMetadata,
    TypeFunctionSummary, TypeRepresentationSummary, ValidatedDecreasesSummary, ValueExportKind,
    ValueExportSummary,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyHeadId, AssociatedFamilyPattern,
    AssociatedFamilyProjection, AssociatedFamilyProjectionMode, AssociatedFamilyResultConstraint,
    AssociatedFamilyResultExpr, AssociatedFamilyScheme, AssociatedFamilySchemeParam,
    CanonicalTypeExpr, ConstructorVariableApp, ConstructorVariableRef, InterfaceBoundProposition,
    NamedPredicateProposition, NormalFormBlockReason, NormalTypeExpr, PartialTypeArg,
    PartialTypeConstructorApp, ProjectionRigidity, PropositionBoundary, PropositionDeferredKind,
    PropositionDeferredReason, PropositionEvidence, PropositionEvidenceRule, PropositionOutcome,
    PropositionRefutation, PropositionRefutationReason, PropositionTypeComparisonEvidence,
    TypeComputationHeadId, TypeConstructorExpr, TypeConstructorHeadId, TypeDisequalityProposition,
    TypeEqualityProposition, TypeFunctionDef, TypeFunctionEquation, TypeFunctionParam,
    TypeFunctionPattern, TypeFunctionPatternConstraint, TypeFunctionResultConstraint,
    TypeFunctionResultExpr, TypeFunctionSourceAnchors, TypeHoleAmbiguity, TypeHoleId,
    TypeHoleMetadata, TypeProposition, TypePropositionTerm,
};
use ash_parser::lower_pattern;
use ash_parser::surface::{
    AssociatedTypeKind, BlockStmt, ComprehensionQualifier, ConstructorPayload, Definition, DoStmt,
    Expr, ImplDef, InterfaceDef, InterfaceMethodSig, InterfaceTypeParam, LawDef, MatchArm, Name,
    Pattern, ProofBody, ProofDef, PropositionClause, PropositionClauseKind,
    PropositionPredicateDecl, PropositionPredicateParam, PropositionTail, ResourceTypeDef,
    Type as SurfaceType, TypeFnDef as SurfaceTypeFnDef, TypePattern as SurfaceTypePattern,
    Visibility as SurfaceVisibility,
};
use ash_parser::token::Span;
use std::collections::{BTreeMap, HashMap, HashSet};

pub use ash_core::semantic_summary::PropositionFactRole;

mod support;
pub use support::*;

/// Declaration kind retained for module-level callables.
///
/// A handler has the same surface callable payload as a function, but its
/// declaration marker is an admission boundary and must not be erased during
/// module registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableDeclarationKind {
    /// An ordinary `fn` declaration.
    Function,
    /// A declaration introduced with the `handler` marker.
    Handler,
}

/// Nominal metadata retained for a parsed `newtype` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominalNewtype {
    type_name: String,
    constructor: String,
    representation_name: String,
    representation: Option<crate::types::Type>,
    identity: TypeDeclId,
}

/// Provenance retained for an imported nominal-newtype pattern binding.
///
/// This is intentionally narrower than general summary import metadata: it
/// proves only the exact provider identity and bounded public-facade depth
/// required by the source pattern bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportedNominalNewtypePatternBinding {
    identity: TypeDeclId,
    public_reexport_hops: u8,
}

impl NominalNewtype {
    /// Return the source-visible nominal type name.
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Return the sole value constructor for this newtype.
    #[must_use]
    pub fn constructor(&self) -> &str {
        &self.constructor
    }

    /// Return the source-visible representation type name.
    #[must_use]
    pub fn representation_name(&self) -> &str {
        &self.representation_name
    }

    /// Return the distinct nominal identity of this wrapper.
    #[must_use]
    pub fn identity(&self) -> TypeDeclId {
        self.identity.clone()
    }

    /// Return the checked representation type once ordinary program checking
    /// has admitted this local declaration.
    #[must_use]
    pub fn representation(&self) -> Option<&crate::types::Type> {
        self.representation.as_ref()
    }
}

/// Error returned when a handler-only admission receives another callable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HandlerCallableRequirementError {
    /// The named callable was not registered.
    #[error("unknown callable '{0}'")]
    UnknownCallable(String),
    /// The named callable is an ordinary function rather than a handler.
    #[error("callable '{0}' is an ordinary function, not a handler")]
    OrdinaryFunction(String),
}

/// Type environment for tracking type definitions and constructor mappings
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Type definitions by name (stored as AST TypeDef)
    ast_types: HashMap<TypeName, TypeDef>,
    /// Internal type info (converted from AST)
    type_info: HashMap<TypeName, TypeInfo>,
    /// Constructor mappings: constructor name -> (type name, variant index)
    constructors: HashMap<String, (TypeName, VariantIndex)>,
    /// Module-level callable declaration markers.
    callable_declarations: HashMap<String, CallableDeclarationKind>,
    /// Imported named effect rows. Their summary authority remains explicitly non-granting.
    imported_effect_rows: HashMap<String, EffectRowExportSummary>,
    /// Nominal newtype metadata, intentionally separate from transparent aliases.
    nominal_newtypes: HashMap<String, NominalNewtype>,
    /// Public imported nominal newtypes visible at this source boundary.
    ///
    /// This is narrow source-boundary provenance for the source-pattern
    /// bridge. Eligibility requires the exact provider-owned identity and no
    /// more than one intervening public facade.
    visible_imported_nominal_newtypes: HashMap<TypeName, ImportedNominalNewtypePatternBinding>,
    /// Public alias names whose underlying representation is intentionally transparent.
    transparent_aliases: HashSet<TypeName>,
    /// Explicit declaration state, avoiding structural placeholder guesses.
    type_declaration_states: HashMap<TypeName, TypeDeclarationState>,
    /// Visible type-name aliases to canonical ordinary type identities.
    type_alias_identities: HashMap<TypeName, TypeDeclId>,
    /// Preferred visible name for a canonical ordinary type identity.
    canonical_type_names: HashMap<TypeDeclId, TypeName>,
    /// Preferred visible interface name to canonical interface identity.
    interface_identity_aliases: HashMap<String, InterfaceIdentityId>,
    /// Tracks whether a visible interface alias came from imported summary metadata.
    interface_identity_alias_is_imported: HashMap<String, bool>,
    /// Preferred visible name for a canonical interface identity.
    canonical_interface_names: HashMap<InterfaceIdentityId, String>,
    /// Minimal TASK-799 local interface arity registry keyed by canonical identity.
    local_interface_arities: HashMap<InterfaceIdentityId, usize>,
    /// Every known interface identity, including imported and source-local registrations.
    known_interface_identities: HashSet<InterfaceIdentityId>,
    /// Preferred visible `(interface, member)` pair to canonical associated-member identity.
    associated_member_identity_aliases: HashMap<(String, String), AssociatedMemberIdentityId>,
    /// Tracks whether a visible associated-member alias came from imported summary metadata.
    associated_member_identity_alias_is_imported: HashMap<(String, String), bool>,
    /// Every known associated-member identity, including imported and source-local registrations.
    known_associated_member_identities: HashSet<AssociatedMemberIdentityId>,
    /// Registered interfaces by name.
    pub(crate) interfaces: HashMap<String, InterfaceInfo>,
    /// Registered capability interfaces by name.
    capability_interfaces: HashMap<String, CapabilityInterfaceInfo>,
    /// Registered resource types by name.
    resource_types: HashMap<String, ResourceTypeInfo>,
    /// Registered capability implementation recipes by name.
    capability_implementations: HashMap<String, CapabilityImplementationInfo>,
    /// Application-admitted capability bindings by local binding name.
    capability_bindings: HashMap<String, CapabilityBindingInfo>,
    /// Registered closed-world impls.
    impls: Vec<ImplScheme>,
    /// Assumed proposition facts available as inputs to later proposition solvers.
    proposition_assumptions: Vec<PropositionFactRecord>,
    /// Required proposition obligations that later task-owned solvers must discharge.
    proposition_obligations: Vec<PropositionFactRecord>,
    /// Source-visible named proposition predicate aliases.
    proposition_predicate_aliases: HashMap<String, PropositionPredicateId>,
    /// Registered named proposition predicates keyed by canonical identity.
    proposition_predicates: HashMap<PropositionPredicateId, PropositionPredicateInfo>,
    /// Interface bounds attached to workflow type variables.
    pub(crate) type_var_interface_bounds: HashMap<TypeVar, HashSet<String>>,
    /// Source-visible type parameter names classified by kind for HKT lowering.
    type_parameter_kinds: HashMap<String, Kind>,
    /// Variable bindings: variable name -> type
    variables: HashMap<String, crate::types::Type>,
    /// Source-only computation facts attached to their originating lexical
    /// parameter bindings. These are never callable signatures or runtime
    /// authority, and ordinary bindings with the same name shadow them.
    source_computation_facts: HashMap<String, crate::checked_computation::CheckedComputation>,
    /// Compiler-known contract intrinsics whose parameters are not source-denotable types.
    contract_intrinsics: HashMap<String, ContractIntrinsic>,
    /// Lowered pure-function contracts kept at the type/runtime boundary.
    fn_contracts: HashMap<String, StoredFnContract>,
    /// Capability symbols known to be capability targets, not pure functions.
    capability_symbols: HashSet<String>,
    /// Parent environment for nested scopes (None for root)
    parent: Option<Box<TypeEnv>>,
    /// Registered capability providers (e.g., "io", "http", "db")
    providers: HashSet<String>,
    /// Sealed-domain identities registered in this environment.
    sealed_domain_identities: HashSet<SealedDomainId>,
    /// Visible alias -> canonical sealed-domain identity.
    sealed_domain_aliases: HashMap<String, SealedDomainId>,
    /// Sealed-domain identity -> domain summary metadata.
    sealed_domain_summaries: HashMap<SealedDomainId, SealedDomainSummary>,
    /// Promoted data-kind identities registered in this environment.
    promoted_data_kind_identities: HashSet<PromotedDataKindId>,
    /// Visible promoted data-kind alias -> canonical promoted data-kind identity.
    promoted_data_kind_aliases: HashMap<String, PromotedDataKindId>,
    /// Promoted data-kind identity -> validated summary metadata.
    promoted_data_kind_summaries: HashMap<PromotedDataKindId, PromotedDataKindSummary>,
    /// Promoted constructor identity -> validated summary metadata.
    promoted_constructor_summaries: HashMap<PromotedConstructorId, PromotedConstructorSummary>,
    /// Promoted constructor identity -> TypeEnv-owned kinding/domain metadata.
    promoted_constructor_kinds: HashMap<PromotedConstructorId, PromotedConstructorKindInfo>,
    /// Source-visible local/imported type-function names keyed to canonical heads.
    local_type_function_heads: HashMap<String, TypeComputationHeadId>,
    /// Checked local/imported type-function carriers keyed by canonical computation head.
    local_type_functions: HashMap<TypeComputationHeadId, TypeFunctionDef>,
    /// Current module identity used to assign source-local interface/family ownership.
    current_module_identity: Option<ModuleIdentity>,
    /// Sealed associated-family declarations keyed by canonical head.
    associated_family_declarations:
        HashMap<AssociatedFamilyHeadId, AssociatedFamilyDeclarationInfo>,
    /// Source-visible `(interface, member)` lookup for sealed associated-family heads.
    associated_family_name_index: HashMap<(String, String), AssociatedFamilyHeadId>,
    /// Coherence-checked associated-family schemes keyed by canonical head.
    associated_family_schemes:
        HashMap<AssociatedFamilyHeadId, Vec<RegisteredAssociatedFamilyScheme>>,
    /// Ambient effect context for the three-vertex boundary (SPEC-031 §4.8).
    ///
    /// `Some(effect)` means expression checking is operating under an ambient
    /// effect level; closures (`Expr::FnDef`) are therefore typed as
    /// `Type::Fun(params, ret, effect)` rather than the pure `Type::Fn(params, ret)`.
    /// `None` means we are in a pure-fn or module-level context.
    ambient_effect: Option<ash_core::Effect>,
}

impl TypeEnv {
    /// Register the declaration-level facts needed before a module crosses a
    /// later handler/newtype checking or lowering boundary.
    ///
    /// This deliberately records no expression semantics: it preserves only
    /// callable markers, nominal newtype identities, and newtype constructors.
    pub fn register_surface_module_declarations(
        &mut self,
        module: &ash_parser::surface::ModuleFile,
    ) -> Result<(), TypeEnvError> {
        self.register_surface_declarations(&module.definitions)
    }

    /// Register declaration-only facts for an entry program before callable
    /// signatures and bodies are checked.
    ///
    /// This is deliberately separate from ordinary ADT registration: a
    /// `newtype` has a fresh nominal identity and its tuple constructor is
    /// checked by the dedicated newtype path rather than alias unfolding.
    pub fn register_surface_declarations(
        &mut self,
        definitions: &[Definition],
    ) -> Result<(), TypeEnvError> {
        let mut staged = self.clone();
        let local_type_names = definitions
            .iter()
            .filter_map(|definition| match definition {
                Definition::Type(ty) => Some(ty.name.to_string()),
                _ => None,
            })
            .collect::<HashSet<_>>();
        let local_constructor_names = definitions
            .iter()
            .filter_map(|definition| match definition {
                Definition::Type(ty) => match &ty.body {
                    ash_parser::surface::TypeBody::Enum(variants) => Some(variants),
                    _ => None,
                },
                _ => None,
            })
            .flatten()
            .map(|variant| variant.name.to_string())
            .collect::<HashSet<_>>();
        let mut preceding_local_type_names = HashSet::new();
        let mut preceding_local_constructor_names = HashSet::new();
        for definition in definitions {
            match definition {
                Definition::Type(ty) => {
                    preceding_local_type_names.insert(ty.name.to_string());
                    if let ash_parser::surface::TypeBody::Enum(variants) = &ty.body {
                        preceding_local_constructor_names
                            .extend(variants.iter().map(|variant| variant.name.to_string()));
                    }
                }
                Definition::Function(function) => {
                    preflight_callable_row_kind_uses(
                        &staged,
                        &function.type_params,
                        &function.params,
                        function.return_type.as_ref(),
                        function.span,
                    )?;
                    staged
                        .callable_declarations
                        .insert(function.name.to_string(), CallableDeclarationKind::Function);
                }
                Definition::Handler(handler) => {
                    preflight_callable_row_kind_uses(
                        &staged,
                        &handler.type_params,
                        &handler.params,
                        Some(&handler.return_type),
                        handler.span,
                    )?;
                    staged
                        .callable_declarations
                        .insert(handler.name.to_string(), CallableDeclarationKind::Handler);
                }
                Definition::Impl(implementation) => {
                    // A `derive handler` declaration has no independent
                    // callable body or value binding.  It nevertheless owns
                    // the same declaration marker as an explicit `handler`,
                    // so handler-only source admission can resolve its name
                    // through the normal value-namespace query.  The checked
                    // declaration fact and any lowering remain separate.
                    for derived in &implementation.derived_handlers {
                        staged
                            .callable_declarations
                            .insert(derived.name.to_string(), CallableDeclarationKind::Handler);
                    }
                }
                Definition::BuiltinFn(function) => {
                    preflight_callable_row_kind_uses(
                        &staged,
                        &function.type_params,
                        &function.params,
                        Some(&function.return_type),
                        function.span,
                    )?;
                }
                Definition::Newtype(newtype) => {
                    let type_name = newtype.name.to_string();
                    if builtin_nominal_type_name(&type_name) {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "newtype '{type_name}' conflicts with existing primitive or prelude type"
                            ),
                            newtype.span,
                        ));
                    }
                    if preceding_local_type_names.contains(&type_name) {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "conflicting local type declaration '{type_name}'; newtype '{type_name}' conflicts with existing local type or constructor"
                            ),
                            newtype.span,
                        ));
                    }
                    if staged.nominal_newtypes.contains_key(&type_name)
                        || (staged.has_type(&type_name) && !local_type_names.contains(&type_name))
                    {
                        return Err(TypeEnvError::DuplicateType(type_name, newtype.span));
                    }
                    if preceding_local_constructor_names.contains(newtype.constructor.as_ref()) {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "newtype '{}' conflicts with existing local type or constructor",
                                newtype.name
                            ),
                            newtype.span,
                        ));
                    }
                    if staged
                        .constructors
                        .contains_key(newtype.constructor.as_ref())
                        && !local_constructor_names.contains(newtype.constructor.as_ref())
                    {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "newtype constructor '{}' is already registered",
                                newtype.constructor
                            ),
                            newtype.span,
                        ));
                    }

                    let identity = self
                        .current_module_identity
                        .as_ref()
                        .cloned()
                        .map(|module| TypeDeclId::ordinary(module, type_name.clone()))
                        .unwrap_or_else(|| fallback_canonical_type_decl_id(&type_name));
                    staged
                        .type_alias_identities
                        .insert(type_name.clone(), identity.clone());
                    staged
                        .canonical_type_names
                        .insert(identity.clone(), type_name.clone());
                    staged
                        .constructors
                        .insert(newtype.constructor.to_string(), (type_name.clone(), 0));
                    staged.nominal_newtypes.insert(
                        type_name.clone(),
                        NominalNewtype {
                            type_name,
                            constructor: newtype.constructor.to_string(),
                            representation_name: newtype_representation_name(
                                &newtype.representation,
                            ),
                            representation: None,
                            identity,
                        },
                    );
                }
                _ => {}
            }
        }
        *self = staged;
        Ok(())
    }

    /// Complete a declaration-only newtype registration with its checked
    /// representation.  The caller has already resolved the source type in
    /// the same local environment, so this cannot introduce a textual or
    /// representation-based constructor fallback.
    pub fn set_nominal_newtype_representation(
        &mut self,
        type_name: &str,
        representation: crate::types::Type,
    ) -> Result<(), TypeEnvError> {
        let Some(newtype) = self.nominal_newtypes.get_mut(type_name) else {
            return Err(TypeEnvError::TypeNotFound(
                type_name.to_string(),
                Span::default(),
            ));
        };
        newtype.representation = Some(representation);
        Ok(())
    }

    /// Resolve a newtype by its sole declared constructor identity.
    #[must_use]
    pub fn nominal_newtype_for_constructor(&self, constructor: &str) -> Option<&NominalNewtype> {
        self.nominal_newtypes
            .values()
            .find(|newtype| newtype.constructor == constructor)
    }

    /// Register local effect aliases and groups before callable-row validation.
    ///
    /// These declarations are requirement descriptions only.  Their summaries
    /// retain the mandatory [`EffectRowAuthority::NonGranting`] marker and do
    /// not install a capability, provider, admission, or discharge fact.
    pub fn register_local_effect_row_declarations(
        &mut self,
        definitions: &[Definition],
    ) -> Result<(), TypeEnvError> {
        let module = self.current_module_identity().cloned().ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                "local effect-row declarations require a current module identity".to_string(),
                Span::default(),
            )
        })?;
        let mut staged = self.clone();
        for definition in definitions {
            let (name, visibility, classification, items, span, label) = match definition {
                Definition::EffectAlias(alias) => (
                    alias.name.clone(),
                    alias.visibility.clone(),
                    EffectRowExportClassification::TransparentAlias,
                    &alias.row.items,
                    alias.span,
                    format!("effect alias {}", alias.name),
                ),
                Definition::EffectGroup(group) => (
                    group.name.clone(),
                    group.visibility.clone(),
                    EffectRowExportClassification::DiagnosticGroup,
                    &group.row.items,
                    group.span,
                    format!("effect group {}", group.name),
                ),
                _ => continue,
            };
            let exported_name = name.to_string();
            let row = EffectRowExportSummary::new(
                EffectRowExportId::new(module.clone(), name),
                exported_name.clone(),
                match visibility {
                    SurfaceVisibility::Public => ash_core::ast::Visibility::Public,
                    SurfaceVisibility::Crate => ash_core::ast::Visibility::Crate,
                    SurfaceVisibility::Inherited
                    | SurfaceVisibility::Super { .. }
                    | SurfaceVisibility::Self_
                    | SurfaceVisibility::Restricted { .. } => ash_core::ast::Visibility::Private,
                },
                classification,
                items
                    .iter()
                    .map(|item| {
                        EffectRowItemSummary::new(ash_parser::surface::format_row_item(item))
                    })
                    .collect(),
                SourceAnchor::new(
                    SourceOrigin::Synthetic {
                        reason: "local effect-row declaration".to_string(),
                    },
                    Some(ash_core::Span {
                        start: span.start,
                        end: span.end,
                    }),
                    label,
                ),
            );
            match staged.imported_effect_rows.get(&exported_name) {
                Some(existing) if existing == &row => {}
                Some(_) => {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("duplicate visible effect-row declaration '{exported_name}'"),
                        span,
                    ));
                }
                None => {
                    staged.imported_effect_rows.insert(exported_name, row);
                }
            }
        }
        *self = staged;
        Ok(())
    }

    /// Return the declaration marker recorded for a module-level callable.
    #[must_use]
    pub fn callable_declaration_kind(&self, name: &str) -> Option<CallableDeclarationKind> {
        self.callable_declarations.get(name).copied()
    }

    /// Record one local callable's declaration marker.
    pub fn register_callable_declaration_kind(
        &mut self,
        name: impl Into<String>,
        kind: CallableDeclarationKind,
    ) {
        self.callable_declarations.insert(name.into(), kind);
    }

    /// Require that a named callable was declared with the `handler` marker.
    pub fn require_handler_callable(
        &self,
        name: &str,
    ) -> Result<(), HandlerCallableRequirementError> {
        match self.callable_declaration_kind(name) {
            Some(CallableDeclarationKind::Handler) => Ok(()),
            Some(CallableDeclarationKind::Function) => Err(
                HandlerCallableRequirementError::OrdinaryFunction(name.to_string()),
            ),
            None => Err(HandlerCallableRequirementError::UnknownCallable(
                name.to_string(),
            )),
        }
    }

    /// Return imported effect-row metadata by its visible exported name.
    #[must_use]
    pub fn lookup_effect_row_export(&self, name: &str) -> Option<&EffectRowExportSummary> {
        self.imported_effect_rows.get(name).or_else(|| {
            // A named import may bind a provider row as `X` while its row text
            // still refers to the provider's canonical name.  Resolve that
            // reference by declaration identity without manufacturing another
            // caller-visible export.
            self.imported_effect_rows
                .values()
                .find(|row| row.id.name == name || row.provider.declaration_name == name)
        })
    }

    /// Expand a registered imported effect row into its source-order item metadata.
    pub fn expand_effect_row_export(
        &self,
        name: &str,
    ) -> Result<Vec<EffectRowItemSummary>, TypeEnvError> {
        self.lookup_effect_row_export(name)
            .map(|row| row.row_items.clone())
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!("unknown imported effect-row export '{name}'"),
                    Span::default(),
                )
            })
    }

    /// Return the nominal registration for a source-visible newtype name.
    #[must_use]
    pub fn nominal_newtype(&self, name: &str) -> Option<&NominalNewtype> {
        self.nominal_newtypes.get(name)
    }

    /// Return whether `visible_name` is a public imported nominal newtype with
    /// the exact provider-owned declaration identity and an admitted direct or
    /// one-hop public-facade provenance.
    #[must_use]
    pub fn is_visible_imported_nominal_newtype(
        &self,
        visible_name: &str,
        identity: &TypeDeclId,
    ) -> bool {
        self.visible_imported_nominal_newtypes
            .get(visible_name)
            .is_some_and(|binding| {
                binding.identity == *identity && binding.public_reexport_hops <= 1
            })
    }

    /// Return a nominal type identity without transparent-alias expansion.
    #[must_use]
    pub fn nominal_type_identity(&self, name: &str) -> Option<TypeDeclId> {
        self.type_identity_for_name(name)
            .cloned()
            .or_else(|| {
                self.nominal_newtypes
                    .get(name)
                    .map(NominalNewtype::identity)
            })
            .or_else(|| {
                builtin_nominal_type_name(name).then(|| fallback_canonical_type_decl_id(name))
            })
    }

    /// Return whether a type is registered as a transparent alias.
    #[must_use]
    pub fn is_transparent_alias(&self, name: &str) -> bool {
        self.transparent_aliases.contains(name)
    }
}

fn preflight_callable_row_kind_uses(
    env: &TypeEnv,
    type_params: &[ash_parser::surface::TypeParam],
    params: &[ash_parser::surface::Param],
    return_type: Option<&SurfaceType>,
    span: Span,
) -> Result<(), TypeEnvError> {
    crate::surface_type_lowering::preflight_row_kinded_proper_type_use(
        env,
        type_params,
        params,
        return_type,
    )
    .map_err(|error| TypeEnvError::InvalidDefinition(error.to_string(), span))
}

fn newtype_representation_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Name(name) => name.to_string(),
        other => format!("{other:?}"),
    }
}

fn builtin_nominal_type_name(name: &str) -> bool {
    matches!(
        name,
        "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" | "()"
    )
}

fn duplicate_summary_identity_diagnostic(
    visible_name: &str,
    existing: &TypeDeclId,
    duplicate: &TypeDeclSummary,
) -> String {
    format!(
        "duplicate ordinary type summary identity for visible type '{visible_name}': \
         existing origin '{}::{}', duplicate origin '{}::{}' from module '{}' at {:?}",
        existing.module.path.join("::"),
        existing.name,
        duplicate.id.module.path.join("::"),
        duplicate.id.name,
        duplicate.id.module.path.join("::"),
        duplicate.source_anchor,
    )
}

fn conflicting_summary_contract_diagnostic(visible_name: &str) -> String {
    format!("conflicting ordinary type summary metadata for visible type '{visible_name}'")
}

fn is_builtin_prelude_ordinary_type_compatibility_name(name: &str) -> bool {
    matches!(name, "Option" | "Result")
}

fn summary_contract_matches(left: &TypeDeclSummary, right: &TypeDeclSummary) -> bool {
    identity_summary_contract_matches(left, right) && left.exported_name == right.exported_name
}

fn identity_summary_contract_matches(left: &TypeDeclSummary, right: &TypeDeclSummary) -> bool {
    left.id == right.id
        && left.visibility == right.visibility
        && left.params == right.params
        && left.declaration_kind == right.declaration_kind
        && left.representation_exposure == right.representation_exposure
        && left.representation == right.representation
}

fn variant_payload_kind(payload: &VariantPayload) -> ConstructorPayloadKind {
    match payload {
        VariantPayload::Unit => ConstructorPayloadKind::Unit,
        VariantPayload::Record(_) => ConstructorPayloadKind::Record,
        VariantPayload::Tuple(_) => ConstructorPayloadKind::Tuple,
    }
}

fn validate_summary_visibility_and_duplicates(
    summary: &ModuleSemanticSummary,
) -> Result<(), TypeEnvError> {
    summary
        .validate_summary_version_contract()
        .map_err(summary_version_contract_error)?;

    if summary.version == SummaryVersion::SPEC057_ORDINARY_TYPE_V1
        && !summary.exported_sealed_domains.is_empty()
    {
        return Err(TypeEnvError::InvalidDefinition(
            "V1 module semantic summary cannot carry sealed domain metadata".to_string(),
            Span::default(),
        ));
    }

    for (index, row) in summary.exported_effect_rows.iter().enumerate() {
        if row.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public effect-row summary '{}' is not valid public metadata",
                    row.exported_name
                ),
                anchor_span(&row.source_anchor),
            ));
        }
        if row.id.module != summary.module {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "effect-row summary '{}' identity does not match enclosing module",
                    row.exported_name
                ),
                anchor_span(&row.source_anchor),
            ));
        }
        if row.authority != EffectRowAuthority::NonGranting {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "effect-row summary '{}' must retain non-granting authority",
                    row.exported_name
                ),
                anchor_span(&row.source_anchor),
            ));
        }
        for duplicate in summary.exported_effect_rows.iter().skip(index + 1) {
            if row.exported_name == duplicate.exported_name && row != duplicate {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate effect-row exported name '{}' has conflicting metadata",
                        row.exported_name
                    ),
                    anchor_span(&duplicate.source_anchor),
                ));
            }
        }
    }

    for (index, value) in summary.exported_values.iter().enumerate() {
        if value.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public value summary '{}' is not valid public metadata",
                    value.exported_name
                ),
                anchor_span(&value.source_anchor),
            ));
        }
        for duplicate in summary.exported_values.iter().skip(index + 1) {
            if value.exported_name == duplicate.exported_name && value != duplicate {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate value exported name '{}' has conflicting metadata",
                        value.exported_name
                    ),
                    anchor_span(&duplicate.source_anchor),
                ));
            }
        }
    }

    for (index, ty) in summary.exported_types.iter().enumerate() {
        if ty.visibility != ash_core::ast::Visibility::Public
            && !matches!(
                ty.representation,
                TypeRepresentationSummary::Opaque { builtin: true }
            )
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public ordinary type summary '{}' is not valid public metadata",
                    ty.exported_name
                ),
                Span::default(),
            ));
        }
        match (&ty.representation_exposure, &ty.representation) {
            (RepresentationExposure::Exposed, TypeRepresentationSummary::Exposed(_)) => {
                if ty.visibility != ash_core::ast::Visibility::Public {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "non-public exposed ordinary type summary '{}' is not valid public metadata",
                            ty.exported_name
                        ),
                        Span::default(),
                    ));
                }
            }
            (RepresentationExposure::Opaque, TypeRepresentationSummary::Opaque { .. }) => {}
            (RepresentationExposure::Exposed, TypeRepresentationSummary::Opaque { .. }) => {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type '{}' has exposed representation exposure without an exposed body",
                        ty.exported_name
                    ),
                    Span::default(),
                ));
            }
            (RepresentationExposure::Opaque, TypeRepresentationSummary::Exposed(_)) => {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type '{}' has opaque representation exposure with an exposed body",
                        ty.exported_name
                    ),
                    Span::default(),
                ));
            }
        }

        for duplicate in summary.exported_types.iter().skip(index + 1) {
            if ty.exported_name != duplicate.exported_name {
                continue;
            }
            if ty.id != duplicate.id {
                return Err(TypeEnvError::InvalidDefinition(
                    duplicate_summary_identity_diagnostic(&ty.exported_name, &ty.id, duplicate),
                    Span::default(),
                ));
            }
            if !summary_contract_matches(ty, duplicate) {
                return Err(TypeEnvError::InvalidDefinition(
                    conflicting_summary_contract_diagnostic(&ty.exported_name),
                    Span::default(),
                ));
            }
        }
        for duplicate in summary.exported_types.iter().skip(index + 1) {
            if ty.id != duplicate.id || ty.exported_name == duplicate.exported_name {
                continue;
            }
            if !identity_summary_contract_matches(ty, duplicate) {
                return Err(TypeEnvError::InvalidDefinition(
                    conflicting_summary_contract_diagnostic(&duplicate.exported_name),
                    Span::default(),
                ));
            }
        }
    }

    for (index, constructor) in summary.exported_constructors.iter().enumerate() {
        for duplicate in summary.exported_constructors.iter().skip(index + 1) {
            if constructor.exported_name != duplicate.exported_name {
                continue;
            }
            if constructor.id != duplicate.id
                || constructor.parent != duplicate.parent
                || constructor.payload_kind != duplicate.payload_kind
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate exported constructor summary '{}' has conflicting metadata",
                        constructor.exported_name
                    ),
                    Span::default(),
                ));
            }
        }
        if constructor.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public exported constructor summary '{}' is not valid public metadata",
                    constructor.exported_name
                ),
                Span::default(),
            ));
        }
        if constructor.id.parent != constructor.parent {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor summary '{}' parent identity does not match constructor id",
                    constructor.exported_name
                ),
                Span::default(),
            ));
        }
        if constructor.id.payload_kind != constructor.payload_kind {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor summary '{}' payload kind does not match constructor id",
                    constructor.exported_name
                ),
                Span::default(),
            ));
        }
        let Some(parent_summary) = summary
            .exported_types
            .iter()
            .find(|ty| ty.id == constructor.parent)
        else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor summary '{}' references a non-exported parent type",
                    constructor.exported_name
                ),
                Span::default(),
            ));
        };
        let TypeRepresentationSummary::Exposed(body) = &parent_summary.representation else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor summary '{}' references a parent without an exposed enum body",
                    constructor.exported_name
                ),
                Span::default(),
            ));
        };
        if imported_summaries_and_domains::imported_nominal_newtype_constructor(
            parent_summary,
            &summary.exported_constructors,
        )?
        .is_some_and(|newtype_constructor| newtype_constructor == constructor)
        {
            continue;
        }
        let TypeBody::Enum(variants) = body else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor summary '{}' references a parent without an exposed enum body",
                    constructor.exported_name
                ),
                Span::default(),
            ));
        };
        let Some(variant) = variants
            .iter()
            .find(|variant| variant.name == constructor.id.name)
        else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor summary '{}' does not match any exposed variant on type '{}'",
                    constructor.exported_name, parent_summary.exported_name
                ),
                Span::default(),
            ));
        };
        let actual_payload_kind = variant_payload_kind(&variant.payload);
        if actual_payload_kind != constructor.payload_kind {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor summary '{}' payload kind {:?} conflicts with exposed enum body {:?}",
                    constructor.exported_name, constructor.payload_kind, actual_payload_kind
                ),
                Span::default(),
            ));
        }
    }

    let same_summary_domain_ids: HashSet<&SealedDomainId> = summary
        .exported_sealed_domains
        .iter()
        .map(|domain| &domain.id)
        .collect();
    let mut same_summary_edges: HashMap<&SealedDomainId, HashSet<&SealedDomainId>> = HashMap::new();
    for domain in &summary.exported_sealed_domains {
        for constructor in &domain.constructors {
            for field in &constructor.fields {
                let Some(target) = field.domain_constraint.as_ref() else {
                    continue;
                };
                if target != &domain.id && same_summary_domain_ids.contains(target) {
                    same_summary_edges
                        .entry(&domain.id)
                        .or_default()
                        .insert(target);
                }
            }
        }
    }
    for domain in &summary.exported_sealed_domains {
        let Some(targets) = same_summary_edges.get(&domain.id) else {
            continue;
        };
        let mut visited = HashSet::new();
        let mut stack: Vec<&SealedDomainId> = targets.iter().copied().collect();
        while let Some(current) = stack.pop() {
            if current == &domain.id {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "sealed domain '{}' participates in a same-summary mutual recursion cycle",
                        domain.exported_name
                    ),
                    Span::default(),
                ));
            }
            if visited.insert(current)
                && let Some(next_targets) = same_summary_edges.get(current)
            {
                stack.extend(next_targets.iter().copied());
            }
        }
    }

    // Sealed-domain structural validation.
    for (index, domain) in summary.exported_sealed_domains.iter().enumerate() {
        // Non-public domains should not appear in imported summaries.
        if domain.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public sealed domain summary '{}' is not valid public metadata",
                    domain.exported_name
                ),
                Span::default(),
            ));
        }
        // Check for duplicate exported domain names.
        for duplicate in summary.exported_sealed_domains.iter().skip(index + 1) {
            if domain.exported_name == duplicate.exported_name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate sealed domain exported name '{}'",
                        domain.exported_name
                    ),
                    Span::default(),
                ));
            }
        }
        // Check for duplicate exported domain identities under different names.
        for duplicate in summary.exported_sealed_domains.iter().skip(index + 1) {
            if domain.id == duplicate.id && domain.exported_name != duplicate.exported_name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "sealed domain identity '{}' appears under multiple exported names",
                        domain.exported_name
                    ),
                    Span::default(),
                ));
            }
        }
        // Validate constructor name uniqueness within this domain.
        let mut constructor_names: HashSet<&str> = HashSet::new();
        for constructor in &domain.constructors {
            if !constructor_names.insert(constructor.exported_name.as_str()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate constructor '{}' in sealed domain '{}'",
                        constructor.exported_name, domain.exported_name
                    ),
                    Span::default(),
                ));
            }
        }
    }

    for (index, data_kind) in summary.exported_promoted_data_kinds.iter().enumerate() {
        if data_kind.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public promoted data-kind summary '{}' is not valid public metadata",
                    data_kind.exported_name
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }
        if data_kind.id.module != summary.module {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind summary '{}' identity module does not match enclosing summary module",
                    data_kind.exported_name
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }
        if data_kind.id.name != data_kind.exported_name
            && !is_dependency_metadata_name(&data_kind.exported_name)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind summary '{}' identity name does not match exported name",
                    data_kind.exported_name
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }
        if data_kind.id.source_type != data_kind.source_type {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind summary '{}' source type does not match its identity",
                    data_kind.exported_name
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }
        for duplicate in summary.exported_promoted_data_kinds.iter().skip(index + 1) {
            if data_kind.exported_name == duplicate.exported_name && data_kind.id != duplicate.id {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate promoted data-kind exported name '{}' has conflicting identities",
                        data_kind.exported_name
                    ),
                    anchor_span(&duplicate.source_anchor),
                ));
            }
            if data_kind.id == duplicate.id && data_kind != duplicate {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate promoted data-kind identity '{}' has conflicting metadata",
                        data_kind.exported_name
                    ),
                    anchor_span(&duplicate.source_anchor),
                ));
            }
        }

        let mut constructor_names = HashSet::new();
        let mut constructor_ids = HashSet::new();
        for constructor in &data_kind.constructors {
            if constructor.visibility != ash_core::ast::Visibility::Public {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "non-public promoted constructor summary '{}' is not valid public metadata",
                        constructor.exported_name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if constructor.id.kind != data_kind.id {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor '{}' belongs to a different promoted data kind",
                        constructor.exported_name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if constructor.id.name != constructor.exported_name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor summary '{}' identity name does not match exported name",
                        constructor.exported_name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if constructor.id.source_constructor != constructor.source_constructor {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor summary '{}' source constructor does not match its identity",
                        constructor.exported_name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if constructor.source_constructor.parent != data_kind.source_type {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "source constructor for promoted constructor '{}' does not belong to source ADT '{}'",
                        constructor.exported_name, data_kind.source_type.name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if !constructor_names.insert(constructor.exported_name.as_str()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate promoted constructor '{}' in data kind '{}'",
                        constructor.exported_name, data_kind.exported_name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if !constructor_ids.insert(&constructor.id) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate promoted constructor identity '{}' in data kind '{}'",
                        constructor.exported_name, data_kind.exported_name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
        }
    }

    for (index, predicate) in summary.exported_proposition_predicates.iter().enumerate() {
        if predicate.visibility != ash_core::ast::Visibility::Public {
            return Err(private_proposition_dependency_error(
                "public proposition metadata",
                "proposition predicate",
                predicate.exported_name.as_ref(),
                anchor_span(&predicate.source_anchor),
            ));
        }
        if predicate.id.name != predicate.exported_name {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "proposition predicate summary '{}' identity does not match exported name '{}'",
                    predicate.id.name, predicate.exported_name
                ),
                anchor_span(&predicate.source_anchor),
            ));
        }
        for duplicate in summary
            .exported_proposition_predicates
            .iter()
            .skip(index + 1)
        {
            if predicate.exported_name == duplicate.exported_name && predicate.id != duplicate.id {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate proposition predicate exported name '{}' has conflicting identities",
                        predicate.exported_name
                    ),
                    anchor_span(&duplicate.source_anchor),
                ));
            }
            if predicate.id == duplicate.id && predicate != duplicate {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate proposition predicate identity '{}' has conflicting metadata",
                        predicate.exported_name
                    ),
                    anchor_span(&duplicate.source_anchor),
                ));
            }
        }
    }

    Ok(())
}

fn summary_version_contract_error(error: ModuleSemanticSummaryValidationError) -> TypeEnvError {
    match error {
        ModuleSemanticSummaryValidationError::TypeFunctionsRequireV3 { version } => {
            TypeEnvError::MalformedImportedComputationSummary {
                message: format!(
                    "module semantic summary version {} cannot carry public type-function summaries; expected {}, {}, or {}",
                    version.0,
                    SummaryVersion::SPEC062_TYPE_COMPUTATION_V3.0,
                    SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4.0,
                    SummaryVersion::SPEC064_PROPOSITIONS_V5.0
                ),
                version,
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::AssociatedFamiliesRequireV4 { version } => {
            TypeEnvError::MalformedImportedComputationSummary {
                message: format!(
                    "module semantic summary version {} cannot carry public associated-family summaries; expected {} or {}",
                    version.0,
                    SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4.0,
                    SummaryVersion::SPEC064_PROPOSITIONS_V5.0
                ),
                version,
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::PropositionFactsRequireV5 { version } => {
            TypeEnvError::PropositionDiagnostic {
                kind: PropositionDiagnosticKind::MalformedPropositionSummary,
                proposition: "public proposition summary payload".into(),
                expected: format!(
                    "summary version {} for public proposition facts",
                    SummaryVersion::SPEC064_PROPOSITIONS_V5.0
                ),
                found: format!("summary version {}", version.0),
                solver_rule: "fail-closed semantic-summary version validation".into(),
                help: "emit proposition facts only from V5/SPEC-064 summaries or drop the proposition payload".into(),
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::PromotedDataKindsRequireV6 { version } => {
            TypeEnvError::MalformedImportedComputationSummary {
                message: format!(
                    "module semantic summary version {} cannot carry public promoted data-kind summaries; expected {}",
                    version.0,
                    SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6.0
                ),
                version,
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::EffectRowProviderBindingsRequireV7 { version } => {
            TypeEnvError::MalformedImportedEffectRowSummary {
                message: format!(
                    "module semantic summary version {} cannot carry provider-binding effect-row summaries; expected {}",
                    version.0,
                    SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7.0
                ),
                version,
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::EffectRowProviderBindingClosureIncomplete { version } => {
            TypeEnvError::MalformedImportedEffectRowSummary {
                message: format!(
                    "module semantic summary version {} has incomplete provider-binding effect-row closure metadata",
                    version.0,
                ),
                version,
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::UnsupportedEffectRowSanitizerSchemaVersion {
            version: sanitizer_schema_version,
        } => TypeEnvError::MalformedImportedEffectRowSummary {
            message: format!(
                "provider-binding effect-row closure uses unsupported sanitizer schema version {sanitizer_schema_version}"
            ),
            version: SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7,
            span: Span::default(),
        },
        ModuleSemanticSummaryValidationError::EffectRowProviderBindingIncoherent { version } => {
            TypeEnvError::MalformedImportedEffectRowSummary {
                message: "provider-binding effect-row identity is incoherent at public boundary"
                    .to_string(),
                version,
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::EffectRowProviderBindingOpaqueInaccessible { version } => {
            TypeEnvError::MalformedImportedEffectRowSummary {
                message: "provider-binding effect-row closure is inaccessible at public boundary"
                    .to_string(),
                version,
                span: Span::default(),
            }
        }
        ModuleSemanticSummaryValidationError::UnsupportedSummaryVersion { version } => {
            TypeEnvError::UnsupportedSummaryVersion {
                version,
                expected: format!(
                    "{}, {}, {}, {}, {}, {}, or {}",
                    SummaryVersion::SPEC057_ORDINARY_TYPE_V1.0,
                    SummaryVersion::SPEC059_SEALED_DOMAIN_V2.0,
                    SummaryVersion::SPEC062_TYPE_COMPUTATION_V3.0,
                    SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4.0,
                    SummaryVersion::SPEC064_PROPOSITIONS_V5.0,
                    SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6.0,
                    SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7.0
                ),
                span: Span::default(),
            }
        }
    }
}

fn anchor_span(anchor: &SourceAnchor) -> Span {
    anchor
        .span
        .map_or_else(Span::default, |span| Span::new(span.start, span.end, 0, 0))
}

fn imported_type_function_def(summary: &TypeFunctionSummary) -> TypeFunctionDef {
    TypeFunctionDef {
        visibility: summary.visibility,
        head: summary.head.clone(),
        name: summary.head.name.clone(),
        params: summary
            .params
            .iter()
            .map(|param| TypeFunctionParam {
                name: param.name.clone(),
                ty: param.ty.clone(),
                kind: param.kind.clone(),
                domain_constraint: param.domain_constraint.clone(),
                source_anchor: param.source_anchor.clone(),
            })
            .collect(),
        return_type: summary.return_type.clone(),
        return_kind: summary.return_kind.clone(),
        result_constraint: summary.result_constraint.clone(),
        decreases: summary.revalidation_metadata.decreases_param.clone(),
        source_anchors: summary.source_anchors.clone(),
        equations: summary.equations.clone(),
    }
}

mod proofs;
pub use proofs::*;

mod associated_families_and_capabilities;
mod imported_summaries_and_domains;
mod interfaces_and_summary_types;
mod lookup_and_unfold;
mod surface_types_laws_and_prelude;
mod type_function_lowering_and_propositions;
mod type_functions;

#[cfg(test)]
mod tests;
