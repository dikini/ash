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
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedFamilyClosureMetadata, AssociatedFamilyDependencySummaryRef,
    AssociatedFamilyExportMode, AssociatedFamilyRevalidationMetadata, AssociatedFamilySummary,
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, ConstructorPayloadKind,
    ConstructorSummary, DomainConstructorId, DomainConstructorSummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary,
    ModuleSemanticSummaryValidationError, ModuleSummaryRef, PromotedConstructorId,
    PromotedConstructorSummary, PromotedDataKindId, PromotedDataKindSummary,
    PropositionFactSummary, PropositionPredicateId, PropositionPredicateParamSummary,
    PropositionPredicateSummary, RepresentationExposure, SealedDomainId, SealedDomainSummary,
    SourceAnchor, SourceOrigin, StructuralFieldStatus, SummaryVersion, TypeDeclId, TypeDeclSummary,
    TypeFunctionClosureMetadata, TypeFunctionDependencySummaryRef, TypeFunctionExportMode,
    TypeFunctionParamSummary, TypeFunctionRevalidationMetadata, TypeFunctionSummary,
    TypeRepresentationSummary, ValidatedDecreasesSummary,
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
use ash_core::workflow_contract::{Contract as WorkflowContract, RuntimePostconditionContract};
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

/// Type environment for tracking type definitions and constructor mappings
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Type definitions by name (stored as AST TypeDef)
    ast_types: HashMap<TypeName, TypeDef>,
    /// Internal type info (converted from AST)
    type_info: HashMap<TypeName, TypeInfo>,
    /// Constructor mappings: constructor name -> (type name, variant index)
    constructors: HashMap<String, (TypeName, VariantIndex)>,
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
    /// Workflow-admitted capability bindings by local binding name.
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
    /// Compiler-known contract intrinsics whose parameters are not source-denotable types.
    contract_intrinsics: HashMap<String, ContractIntrinsic>,
    /// Public Workflow summaries imported from module metadata by binding name.
    public_workflow_summaries: HashMap<String, ash_core::workflow_carrier::PublicWorkflowSummary>,
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
        let TypeRepresentationSummary::Exposed(TypeBody::Enum(variants)) =
            &parent_summary.representation
        else {
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
        ModuleSemanticSummaryValidationError::UnsupportedSummaryVersion { version } => {
            TypeEnvError::UnsupportedSummaryVersion {
                version,
                expected: format!(
                    "{}, {}, {}, {}, or {}",
                    SummaryVersion::SPEC057_ORDINARY_TYPE_V1.0,
                    SummaryVersion::SPEC059_SEALED_DOMAIN_V2.0,
                    SummaryVersion::SPEC062_TYPE_COMPUTATION_V3.0,
                    SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4.0,
                    SummaryVersion::SPEC064_PROPOSITIONS_V5.0
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
