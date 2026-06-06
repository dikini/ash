//! Type environment for tracking type definitions and constructor mappings
//!
//! Provides `TypeEnv` for managing type definitions and looking up constructors.

#![allow(clippy::result_large_err)]

use crate::error::{PropositionDiagnosticKind, TypeEnvError};
use crate::normalizer::{DefinitionalEqualityResult, Normalizer};
use crate::solver::TypeError;
use crate::types::{Substitution, Type, TypeVar, UnifyError, unify};
use crate::{Kind, QualifiedName};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload};
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
use ash_parser::surface::{
    AssociatedTypeKind, CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationDependencyKind, CapabilityImplementationOperation,
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, ImplDef, InterfaceDef,
    InterfaceMethodSig, InterfaceTypeParam, PropositionClause, PropositionClauseKind,
    PropositionPredicateDecl, PropositionPredicateParam, PropositionTail, ResourceTypeDef,
    Type as SurfaceType, TypeFnDef as SurfaceTypeFnDef, TypePattern as SurfaceTypePattern,
    Visibility as SurfaceVisibility,
};
use ash_parser::token::Span;
use std::collections::{BTreeMap, HashMap, HashSet};

pub use ash_core::semantic_summary::PropositionFactRole;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TypeFunctionCoverageValue {
    constructor: ash_core::semantic_summary::DomainConstructorId,
    fields: Vec<Option<TypeFunctionCoverageValue>>,
}

#[derive(Debug, Clone)]
struct TypeFunctionCoverageAlt {
    constructor: ash_core::semantic_summary::DomainConstructorId,
    fields: Vec<Option<TypeFunctionCoverageSpace>>,
}

#[derive(Debug, Clone)]
struct TypeFunctionCoverageSpace {
    domain: SealedDomainId,
    alts: Vec<TypeFunctionCoverageAlt>,
}

#[derive(Debug, Clone, Default)]
struct PublicTypeFunctionClosure {
    ordinary_types: HashSet<TypeDeclId>,
    sealed_domains: HashSet<SealedDomainId>,
    promoted_data_kinds: HashSet<PromotedDataKindId>,
    promoted_constructors: HashSet<PromotedConstructorId>,
    type_functions: HashSet<TypeComputationHeadId>,
    projections: HashSet<(InterfaceIdentityId, AssociatedMemberIdentityId)>,
}

#[derive(Debug, Clone, Default)]
struct PublicAssociatedFamilyClosure {
    ordinary_types: HashSet<TypeDeclId>,
    sealed_domains: HashSet<SealedDomainId>,
    domain_constructors: HashSet<DomainConstructorId>,
    type_functions: HashSet<TypeComputationHeadId>,
    projections: HashSet<AssociatedFamilyProjection>,
    associated_families: HashSet<AssociatedFamilyHeadId>,
}

impl PublicAssociatedFamilyClosure {
    fn associated_family_summary_refs(
        &self,
        current_head: &AssociatedFamilyHeadId,
        module: &ModuleIdentity,
        env: &TypeEnv,
    ) -> Vec<AssociatedFamilyDependencySummaryRef> {
        let mut refs = self
            .associated_families
            .iter()
            .filter(|head| *head != current_head)
            .map(|head| {
                let source_visible = env
                    .associated_family_declarations
                    .get(head)
                    .map(|declaration| {
                        env.interfaces
                            .get(declaration.head.interface.name.as_str())
                            .is_some_and(|info| {
                                matches!(info.visibility, ash_core::ast::Visibility::Public)
                            })
                    })
                    .unwrap_or(false);
                AssociatedFamilyDependencySummaryRef {
                    summary_ref: ModuleSummaryRef {
                        module: head.interface.module.clone(),
                        version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
                    },
                    family: (*head).clone(),
                    digest: None,
                    compiler_algorithm_version: Some("spec-063-mvp".to_string()),
                    source_visible,
                    normalizer_available: true,
                }
            })
            .collect::<Vec<_>>();
        refs.sort_by(|left, right| {
            left.family
                .interface
                .module
                .path
                .cmp(&right.family.interface.module.path)
                .then_with(|| left.family.interface.name.cmp(&right.family.interface.name))
                .then_with(|| left.family.member.name.cmp(&right.family.member.name))
        });
        refs.dedup_by(|left, right| {
            left.family == right.family && left.summary_ref == right.summary_ref
        });
        if self.associated_families.contains(current_head) {
            // Self recursion is represented by the exported scheme/decreases metadata, not by
            // an extra dependency summary ref.
        }
        if refs
            .iter()
            .all(|reference| reference.summary_ref.module != *module)
        {
            return refs;
        }
        refs
    }
}

impl PublicTypeFunctionClosure {
    fn dependency_summary_refs(&self) -> Vec<TypeFunctionDependencySummaryRef> {
        let mut refs = Vec::new();
        for ty in &self.ordinary_types {
            push_dependency_summary_ref(
                &mut refs,
                ty.module.clone(),
                SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
            );
        }
        for domain in &self.sealed_domains {
            push_dependency_summary_ref(
                &mut refs,
                domain.module.clone(),
                SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
            );
        }
        for data_kind in &self.promoted_data_kinds {
            push_dependency_summary_ref(
                &mut refs,
                data_kind.module.clone(),
                SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6,
            );
        }
        for constructor in &self.promoted_constructors {
            push_dependency_summary_ref(
                &mut refs,
                constructor.kind.module.clone(),
                SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6,
            );
        }
        for head in &self.type_functions {
            push_dependency_summary_ref(
                &mut refs,
                head.module.clone(),
                SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
            );
        }
        for (interface, member) in &self.projections {
            push_dependency_summary_ref(
                &mut refs,
                interface.module.clone(),
                SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
            );
            push_dependency_summary_ref(
                &mut refs,
                member.interface.module.clone(),
                SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
            );
        }
        refs.sort_by(|left, right| {
            left.summary_ref
                .module
                .path
                .cmp(&right.summary_ref.module.path)
                .then_with(|| {
                    left.summary_ref
                        .module
                        .module_id
                        .0
                        .cmp(&right.summary_ref.module.module_id.0)
                })
                .then_with(|| left.summary_ref.version.0.cmp(&right.summary_ref.version.0))
        });
        refs
    }
}

fn push_dependency_summary_ref(
    refs: &mut Vec<TypeFunctionDependencySummaryRef>,
    module: ModuleIdentity,
    version: SummaryVersion,
) {
    let summary_ref = ModuleSummaryRef { module, version };
    if refs.iter().any(|dep| dep.summary_ref == summary_ref) {
        return;
    }
    refs.push(TypeFunctionDependencySummaryRef {
        summary_ref,
        digest: None,
        compiler_algorithm_version: Some("spec-062-mvp".to_string()),
    });
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoredFnContract {
    pub param_names: Vec<String>,
    pub contract: WorkflowContract,
    pub runtime_postconditions: RuntimePostconditionContract,
}

/// Type name (e.g., "Option", "Result")
pub type TypeName = String;

/// Field name in a variant
pub type FieldName = String;

/// Index of a variant within an enum type
pub type VariantIndex = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeDeclarationState {
    Placeholder,
    IdentityOnly,
    Full,
}

/// Convert a type expression to an internal type
///
/// This conversion maps:
/// - Primitive types (Int, String, Bool, Null, Time, Ref) to their Type equivalents
/// - Type parameters to their corresponding TypeVar
/// - User-defined type constructors to Type::Constructor with resolved names
/// - Lists, tuples, and records to their corresponding Type variants
pub fn type_expr_to_type(
    expr: &TypeExpr,
    param_mapping: &HashMap<String, TypeVar>,
    type_env: &TypeEnv,
) -> Result<Type, TypeError> {
    match expr {
        TypeExpr::Named(name) => {
            // Check if it's a type parameter
            if let Some(&var) = param_mapping.get(name) {
                if let Some(kind) = type_env.type_parameter_kind(name)
                    && !kind.is_type()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "constructor variable '{name}' has kind {kind}; expected a fully applied proper type"
                        ),
                        Span::default(),
                    )
                    .into());
                }
                return Ok(Type::Var(var));
            }

            // Check for primitive types
            match name.as_str() {
                "Int" => Ok(Type::Int),
                "String" => Ok(Type::String),
                "Bool" => Ok(Type::Bool),
                "Float" => Ok(Type::Float),
                "Null" | "Unit" => Ok(Type::Null),
                "Time" => Ok(Type::Time),
                "Ref" => Ok(Type::Ref),
                _ => {
                    // User-defined type with no args - look it up
                    let (qualified, _) = type_env.resolve_type(name)?;
                    type_env.check_type_constructor_arity(&qualified, 0)?;
                    Ok(Type::Constructor {
                        name: qualified,
                        args: vec![],
                        kind: Kind::Type,
                    })
                }
            }
        }

        TypeExpr::Constructor { name, args } => {
            if name == "Fn" {
                let mut arg_types: Vec<_> = args
                    .iter()
                    .map(|arg| type_expr_to_type(arg, param_mapping, type_env))
                    .collect::<Result<Vec<_>, _>>()?;
                let ret = match arg_types.pop() {
                    Some(ret) => ret,
                    None => {
                        return Err(TypeError::ConstructorArityMismatch {
                            name: "Fn".to_string(),
                            expected_arity: 1,
                            found_arity: 0,
                            span: Span::default(),
                        });
                    }
                };
                Ok(Type::Fn(arg_types, Box::new(ret)))
            } else if let Some(kind) = type_env.type_parameter_kind(name) {
                constructor_variable_application_to_type(name, kind, args.len(), || {
                    args.iter()
                        .map(|arg| type_expr_to_type(arg, param_mapping, type_env))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|err| {
                            TypeEnvError::InvalidDefinition(err.to_string(), Span::default())
                        })
                })
                .map_err(TypeError::from)
            } else if param_mapping.contains_key(name) {
                Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "proper type variable '{name}' of kind * cannot be applied as a constructor"
                    ),
                    Span::default(),
                )
                .into())
            } else {
                let (qualified, _) = type_env.resolve_type(name)?;
                type_env.check_type_constructor_arity(&qualified, args.len())?;

                // Convert all arguments
                let arg_types: Result<Vec<_>, _> = args
                    .iter()
                    .map(|arg| type_expr_to_type(arg, param_mapping, type_env))
                    .collect();

                Ok(Type::Constructor {
                    name: qualified,
                    args: arg_types?,
                    kind: Kind::Type,
                })
            }
        }

        TypeExpr::Tuple(elems) => {
            // Convert tuple to record with numeric field names
            let field_types: Result<Vec<_>, _> = elems
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    type_expr_to_type(t, param_mapping, type_env)
                        .map(|ty| (Box::from(format!("_{}", i).as_str()), ty))
                })
                .collect();
            Ok(Type::Record(field_types?))
        }

        TypeExpr::Record(fields) => {
            let field_types: Result<Vec<_>, _> = fields
                .iter()
                .map(|(n, t)| {
                    type_expr_to_type(t, param_mapping, type_env)
                        .map(|ty| (Box::from(n.as_str()), ty))
                })
                .collect();
            Ok(Type::Record(field_types?))
        }
        TypeExpr::Associated { base, name } => {
            if let TypeExpr::Constructor {
                name: interface,
                args,
            } = base.as_ref()
                && type_env
                    .lookup_associated_family_declaration(interface, name)
                    .is_some()
            {
                return lower_core_explicit_associated_family_projection_to_type(
                    type_env,
                    interface,
                    args,
                    name,
                    param_mapping,
                );
            }
            let base_ty = match base.as_ref() {
                TypeExpr::Named(base_name) if !param_mapping.contains_key(base_name) => {
                    match type_env.resolve_type(base_name) {
                        Ok(_) => type_expr_to_type(base, param_mapping, type_env)?,
                        Err(_) if looks_like_unbound_type_var_name(base_name) => {
                            return Err(TypeError::TypeEnv(Box::new(
                                TypeEnvError::InvalidDefinition(
                                    format!("unresolved associated type '{name}'"),
                                    Span::default(),
                                ),
                            )));
                        }
                        Err(err) => return Err(err),
                    }
                }
                _ => type_expr_to_type(base, param_mapping, type_env)?,
            };
            let interface = resolve_associated_interface_from_type_var_bounds(
                type_env,
                &base_ty,
                &core_projection_base_spelling(base),
                name,
            )?;
            Ok(Type::Associated {
                interface,
                base: Box::new(base_ty),
                name: name.clone(),
            })
        }
    }
}

/// Internal representation of a variant definition with converted types
#[derive(Debug, Clone, PartialEq)]
pub struct VariantInfo {
    /// Name of the variant (e.g., "Some", "None")
    pub name: String,
    /// Fields of the variant: (field_name, field_type)
    /// Types are converted from TypeExpr to Type
    pub fields: Vec<(FieldName, Type)>,
    /// Canonical payload shape for the variant.
    pub payload_shape: VariantPayloadShape,
}

/// Internal representation of a type definition with converted types
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    /// Enum type with multiple variants
    Enum {
        /// Name of the type
        name: TypeName,
        /// Type parameters (for generic types)
        params: Vec<TypeVar>,
        /// Variants of the enum
        variants: Vec<VariantInfo>,
    },
    /// Struct type with fields
    Struct {
        /// Name of the type
        name: TypeName,
        /// Type parameters (for generic types)
        params: Vec<TypeVar>,
        /// Fields of the struct
        fields: Vec<(FieldName, Type)>,
    },
}

impl TypeInfo {
    pub(crate) fn type_arg_count(&self) -> usize {
        match self {
            Self::Enum { params, .. } | Self::Struct { params, .. } => params.len(),
        }
    }
}

/// Pattern-specific canonicalization outcome for a scrutinee type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternCanonicalization {
    /// The scrutinee has a concrete ordinary ADT identity and constructor universe.
    Matchable(PatternCanonicalType),
    /// The scrutinee cannot be matched as an ordinary ADT pattern universe.
    Blocked {
        /// Source type passed to the pattern canonicalization API.
        source_type: Type,
        /// Typed reason pattern canonicalization did not produce an ADT universe.
        reason: PatternCanonicalizationBlockedReason,
    },
}

/// Canonical ADT type and constructor universe used by pattern consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternCanonicalType {
    /// Source type passed to the pattern canonicalization API.
    pub source_type: Type,
    /// Canonical concrete ADT type after transparent alias/projection normalization.
    pub canonical_type: Type,
    /// Canonical ordinary ADT name.
    pub canonical_name: QualifiedName,
    /// Canonical type arguments applied to the ordinary ADT.
    pub canonical_type_args: Vec<Type>,
    /// Constructor universe for the canonical ADT, in variant order.
    pub constructors: Vec<PatternCanonicalConstructor>,
}

/// Canonical constructor entry for a pattern-matchable ADT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternCanonicalConstructor {
    /// Source-visible constructor name.
    pub name: String,
    /// Variant index within the canonical ADT.
    pub variant_index: VariantIndex,
    /// Payload fields after substituting the canonical ADT type arguments.
    pub fields: Vec<(FieldName, Type)>,
    /// Canonical payload shape.
    pub payload_shape: VariantPayloadShape,
}

/// Typed reason why a scrutinee type did not yield a matchable ADT universe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternCanonicalizationBlockedReason {
    /// The type is known but is not an ordinary enum ADT.
    NonAdt,
    /// The type head is an unresolved type variable.
    TypeVariable,
    /// The canonical ADT application still contains an unresolved type argument.
    NonConcreteTypeArgument,
    /// The type is an associated projection that is rigid, neutral, or unresolved.
    RigidAssociatedProjection {
        /// Source-visible interface name.
        interface: String,
        /// Source-visible associated member name.
        member: String,
    },
    /// The type is headed by a constructor variable such as `M<A>`.
    ConstructorVariableApplication {
        /// Source-visible constructor-variable name.
        constructor: String,
    },
    /// The nominal type head is not known to this environment.
    UnknownType {
        /// Source-visible type name.
        name: QualifiedName,
    },
    /// The ADT representation exists but its exported constructor universe is incomplete.
    UnknownConstructorUniverse {
        /// Canonical ordinary ADT name.
        name: QualifiedName,
    },
    /// The type shape is outside the runtime pattern ADT surface.
    UnsupportedType,
}

fn primitive_pattern_type(name: &str) -> Option<Type> {
    match name {
        "Int" => Some(Type::Int),
        "String" => Some(Type::String),
        "Bool" => Some(Type::Bool),
        "Float" => Some(Type::Float),
        "Null" => Some(Type::Null),
        "Time" => Some(Type::Time),
        "Ref" => Some(Type::Ref),
        _ => None,
    }
}

/// Errors reported while elaborating explicit type holes and partial
/// type-constructor applications into the core constructor-expression carrier.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PartialConstructorElaborationError {
    /// A partial target context requires exactly one explicit `_` hole.
    #[error("partial constructor target for `{constructor}` requires exactly one type hole `_`")]
    MissingHole { constructor: String, span: Span },
    /// The MVP accepts only one value-position hole in a partial target.
    #[error(
        "partial constructor target for `{constructor}` has {count} type holes; the MVP accepts exactly one"
    )]
    MultipleHoles {
        constructor: String,
        count: usize,
        span: Span,
    },
    /// Bare higher-arity constructors are not implicitly curried.
    #[error(
        "bare higher-arity constructor `{constructor}` has arity {arity}; write `{hint}` with an explicit `_` hole"
    )]
    BareHigherArityConstructor {
        constructor: String,
        arity: usize,
        hint: String,
        span: Span,
    },
    /// The supplied argument count does not match the constructor arity.
    #[error(
        "wrong constructor arity for `{constructor}` after hole elaboration: expected {expected_arity}, found {found_arity}"
    )]
    WrongArity {
        constructor: String,
        expected_arity: usize,
        found_arity: usize,
        span: Span,
    },
    /// A named type constructor could not be resolved.
    #[error("unknown type constructor `{constructor}`")]
    UnknownConstructor { constructor: String, span: Span },
    /// A hole appeared somewhere the MVP does not enable.
    #[error("unsupported type-hole position: {reason}")]
    UnsupportedHolePosition { reason: String, span: Span },
    /// A hole would require type-function or associated-family output inversion.
    #[error("cannot elaborate type hole by inverting {context}; this boundary is non-inverting")]
    NoInversionBoundary { context: String, span: Span },
    /// Lowering a non-hole argument failed.
    #[error("failed to elaborate type argument for `{constructor}`: {reason}")]
    ArgumentLoweringFailed {
        constructor: String,
        reason: String,
        span: Span,
    },
}

/// Internal representation of an interface method signature.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethodInfo {
    /// Interface-level type variables corresponding to the interface head.
    pub type_params: Vec<TypeVar>,
    /// Canonical single-argument parameter types.
    pub params: Vec<Type>,
    /// Declared return type.
    pub return_type: Type,
}

/// Internal representation of an interface definition.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceInfo {
    /// Interface name.
    pub name: String,
    /// Source/interface visibility used by public export-closure checks.
    pub visibility: ash_core::ast::Visibility,
    /// Interface-level type parameter names.
    pub type_params: Vec<String>,
    /// Interface-level type parameter kinds.
    pub type_param_kinds: Vec<Kind>,
    /// Associated types declared by the interface.
    pub associated_types: Vec<String>,
    /// Methods declared by the interface.
    pub methods: HashMap<String, InterfaceMethodInfo>,
}

/// Typed interface evidence argument used for impl coherence keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceEvidenceArg {
    /// A proper type argument of kind `*`.
    Proper(Type),
    /// A constructor argument such as `Option` for kind `* -> *`.
    Constructor(Box<TypeConstructorExpr>),
}

/// Internal representation of a capability interface operation signature.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityOperationInfo {
    /// Operation effect mode.
    pub mode: CapabilityOperationMode,
    /// Declared parameter names in source order.
    pub param_names: Vec<String>,
    /// Declared parameter types in source order.
    pub params: Vec<Type>,
    /// Declared return type.
    pub return_type: Type,
}

/// Internal representation of a capability interface definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityInterfaceInfo {
    /// Capability interface name.
    pub name: String,
    /// Operations declared by the capability interface.
    pub operations: HashMap<String, CapabilityOperationInfo>,
}

/// Internal representation of a resource type declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceTypeInfo {
    /// Resource type name.
    pub name: String,
    /// Metadata fields carried by resource instances.
    pub fields: Vec<(String, Type)>,
}

/// Static authority provenance category for capability/resource metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorityProvenanceKind {
    /// Authority supplied by host/runtime admission outside Ash-defined recipes.
    ///
    /// The static checker does not infer this category for Ash-defined
    /// `capability impl` recipes; host authority must be attached by a future
    /// runtime admission/provider path.
    Host,
    /// Authority over Ash-owned resources allocated or admitted explicitly.
    Internal,
    /// Authority derived from declared capability/resource dependencies.
    Derived,
    /// No static authority source is required by the recipe.
    NoAuthority,
}

/// Kind of dependency that participates in authority provenance metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProvenanceSourceKind {
    /// Resource dependency source.
    Resource,
    /// Capability dependency source.
    Capability,
    /// Config dependency metadata; not itself an authority source.
    Config,
}

/// Static implementation-level authority source metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationAuthoritySourceInfo {
    /// Source kind.
    pub kind: ProvenanceSourceKind,
    /// Declared dependency name.
    pub dependency_name: String,
    /// Resource type, capability interface, or config type target.
    pub target_name: String,
}

/// Workflow-owned resource provenance metadata for runtime admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBindingProvenanceInfo {
    /// Workflow resource binding name.
    pub name: String,
    /// Registered resource type name.
    pub resource_type: String,
    /// Static authority category for this resource binding.
    pub authority: AuthorityProvenanceKind,
}

/// Workflow capability-binding provenance source metadata for runtime admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingProvenanceSourceInfo {
    /// Source kind.
    pub kind: ProvenanceSourceKind,
    /// Declared dependency name in the selected implementation recipe.
    pub dependency_name: String,
    /// Concrete workflow binding/resource/config expression name where available.
    pub binding_name: String,
    /// Resource type, capability interface, or config type target.
    pub target_name: String,
}

/// Workflow capability-binding provenance metadata for runtime admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityBindingProvenanceInfo {
    /// Workflow capability binding name.
    pub name: String,
    /// Annotated capability interface name.
    pub interface: String,
    /// Selected implementation recipe name.
    pub implementation: String,
    /// Static authority category for this admitted binding.
    pub authority: AuthorityProvenanceKind,
    /// Concrete provenance source links.
    pub sources: Vec<BindingProvenanceSourceInfo>,
}

/// Workflow-admitted capability binding metadata for static operation resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityBindingInfo {
    /// Workflow binding name.
    pub name: String,
    /// Capability interface admitted for this binding.
    pub interface: String,
    /// Implementation recipe selected by the workflow header.
    pub implementation: String,
    /// Static authority category for this binding.
    pub authority: AuthorityProvenanceKind,
}

/// Workflow-level authority provenance metadata for runtime admission.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthorityProvenanceReport {
    /// Workflow-owned resource bindings.
    pub resource_bindings: Vec<ResourceBindingProvenanceInfo>,
    /// Workflow-used capability bindings.
    pub capability_bindings: Vec<CapabilityBindingProvenanceInfo>,
}

/// Internal representation of a capability implementation dependency.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityImplementationDependencyInfo {
    /// Dependency kind declared by the implementation recipe.
    pub kind: CapabilityImplementationDependencyKind,
    /// Binding name visible to operation bodies.
    pub name: String,
    /// Lowered dependency type.
    pub ty: Type,
    /// Resource type or capability interface target for metadata dependencies.
    pub target_name: Option<String>,
}

/// Internal representation of a capability implementation operation.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityImplementationOperationInfo {
    /// Operation effect mode.
    pub mode: CapabilityOperationMode,
    /// Declared parameter names in source order.
    pub param_names: Vec<String>,
    /// Declared parameter types in source order.
    pub params: Vec<Type>,
    /// Declared return type.
    pub return_type: Type,
}

/// Internal representation of a capability implementation recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityImplementationInfo {
    /// Implementation recipe name.
    pub name: String,
    /// Target capability interface name.
    pub interface: String,
    /// Explicit dependencies available to operation bodies.
    pub dependencies: Vec<CapabilityImplementationDependencyInfo>,
    /// Operations implemented by this recipe.
    pub operations: HashMap<String, CapabilityImplementationOperationInfo>,
    /// Static authority provenance classification inferred from declared dependencies.
    pub authority_provenance: AuthorityProvenanceKind,
    /// Static authority/config source metadata inferred from declared dependencies.
    pub authority_sources: Vec<ImplementationAuthoritySourceInfo>,
}

/// Internal representation of a where-bound for type checking.
#[derive(Debug, Clone, PartialEq)]
pub struct WhereBound {
    pub type_var: TypeVar,
    pub interface: String,
}

/// Internal representation of an impl method signature.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethodInfo {
    pub name: String,
    pub param_names: Vec<String>,
    pub type_params: Vec<TypeVar>,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub body: ash_core::ast::Expr,
}

/// Internal representation of a generic impl scheme.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplScheme {
    pub interface: String,
    pub type_params: Vec<TypeVar>,
    pub head: Type,
    pub head_args: Vec<InterfaceEvidenceArg>,
    pub where_bounds: Vec<WhereBound>,
    pub associated_type_bindings: HashMap<String, Type>,
    pub methods: Vec<ImplMethodInfo>,
}

/// Typed owner category for propositions generated or assumed during type checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropositionCheckingSiteKind {
    ExplicitRequirement,
    TypeVariableInterfaceBound,
    ImplWhereBound,
    ConcreteImpl,
    Synthetic,
}

/// Typed owner/provenance for a proposition fact.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropositionCheckingSite {
    pub id: u64,
    pub kind: PropositionCheckingSiteKind,
    pub label: Option<String>,
}

impl PropositionCheckingSite {
    #[must_use]
    pub const fn new(id: u64, kind: PropositionCheckingSiteKind, label: Option<String>) -> Self {
        Self { id, kind, label }
    }
}

/// Canonical proposition clause plus source-local classification outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredPropositionClause {
    pub proposition: TypeProposition,
    pub source_anchor: SourceAnchor,
    pub outcome: Option<PropositionOutcome>,
}

/// TypeEnv-owned proposition fact record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropositionFactRecord {
    pub proposition: TypeProposition,
    pub source_anchor: SourceAnchor,
    pub owner_site: PropositionCheckingSite,
    pub role: PropositionFactRole,
    pub outcome: Option<PropositionOutcome>,
}

/// Solver treatment for a registered named proposition predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropositionPredicateSolverKind {
    /// Ordinary source/imported predicates are opaque in TASK-878 and must defer.
    DeferredUnsupported,
    /// Compiler-owned builtin predicate explicitly registered in this TypeEnv.
    CompilerBuiltinSatisfied,
}

/// TypeEnv-owned named proposition predicate metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropositionPredicateInfo {
    pub summary: PropositionPredicateSummary,
    solver_kind: PropositionPredicateSolverKind,
}

impl PropositionPredicateInfo {
    #[must_use]
    pub fn is_compiler_builtin(&self) -> bool {
        matches!(
            self.solver_kind,
            PropositionPredicateSolverKind::CompilerBuiltinSatisfied
        )
    }
}

/// TypeEnv-owned kinding/domain metadata for a promoted data constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotedConstructorKindInfo {
    pub kind: Kind,
    pub result_data_kind: PromotedDataKindId,
    pub field_data_kind_constraints: Vec<Option<PromotedDataKindId>>,
}

/// Domain metadata preserved for an interface parameter of a sealed associated family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedFamilyInterfaceParamInfo {
    pub name: String,
    pub domain_constraint: Option<SealedDomainId>,
}

/// Registered declaration metadata for a sealed associated-family member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedFamilyDeclarationInfo {
    pub defining_module: ModuleIdentity,
    pub result_domain: AssociatedFamilyResultConstraint,
    pub decreases: Option<String>,
    pub interface_params: Vec<AssociatedFamilyInterfaceParamInfo>,
    pub head: AssociatedFamilyHeadId,
}

/// Coherence-checked associated-family scheme plus defining-module provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredAssociatedFamilyScheme {
    pub defining_module: ModuleIdentity,
    pub scheme: AssociatedFamilyScheme,
}

/// Structured blocker for one-way associated-family scheme selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssociatedFamilySelectionBlocker {
    NoApplicableScheme,
    AbstractScrutinee,
    NeutralScrutinee,
    RigidProjection,
    Ambiguous,
}

/// Evidence for a uniquely selected associated-family scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedAssociatedFamilyScheme<'env> {
    pub family_head: AssociatedFamilyHeadId,
    pub registered: &'env RegisteredAssociatedFamilyScheme,
    pub equation: &'env AssociatedFamilyEquation,
    pub scheme_param_bindings: BTreeMap<String, CanonicalTypeExpr>,
}

/// Result of one-way associated-family scheme selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssociatedFamilySelection<'env> {
    Selected(SelectedAssociatedFamilyScheme<'env>),
    Blocked {
        family: AssociatedFamilyHeadId,
        reason: AssociatedFamilySelectionBlocker,
    },
    Ambiguous {
        family: AssociatedFamilyHeadId,
        candidate_count: usize,
    },
    NoMatch {
        family: AssociatedFamilyHeadId,
    },
}

/// One-step associated-family reduction evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedFamilyReduction<'env> {
    pub selected: SelectedAssociatedFamilyScheme<'env>,
    pub result: AssociatedFamilyResultExpr,
}

/// Evidence for a uniquely selected associated-family scheme over already-normalized
/// normalizer arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedNormalizedAssociatedFamilyScheme<'env> {
    pub family_head: AssociatedFamilyHeadId,
    pub registered: &'env RegisteredAssociatedFamilyScheme,
    pub equation: &'env AssociatedFamilyEquation,
    pub scheme_param_bindings: BTreeMap<String, NormalTypeExpr>,
}

/// One-step local associated-family reduction over normalizer arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAssociatedFamilyReduction<'env> {
    pub selected: SelectedNormalizedAssociatedFamilyScheme<'env>,
    pub result: AssociatedFamilyResultExpr,
}

/// Result of consulting the local associated-family table from the normalizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalAssociatedFamilyProjectionLookup<'env> {
    Reduced(Box<LocalAssociatedFamilyReduction<'env>>),
    Blocked {
        family: Box<AssociatedFamilyHeadId>,
        reason: NormalFormBlockReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssociatedFamilyMatchFailure {
    NoMatch,
    Blocked(AssociatedFamilySelectionBlocker),
}

#[derive(Debug, Clone)]
pub struct SelectedScheme {
    pub substitution: Substitution,
}

#[derive(Debug, Default)]
struct AliasCanonicalVarBridge {
    next_var: u32,
    args: HashMap<TypeVar, CanonicalTypeExpr>,
}

impl AliasCanonicalVarBridge {
    fn placeholder_for_arg(&mut self, expr: &CanonicalTypeExpr) -> Type {
        let var = TypeVar(0x8230_0000u32.wrapping_add(self.next_var));
        self.next_var = self.next_var.wrapping_add(1);
        self.args.insert(var, expr.clone());
        Type::Var(var)
    }
}

fn fallback_canonical_type_decl_id(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(
        ModuleIdentity::new(
            Some(CrateId(usize::MAX)),
            ModuleId(usize::MAX),
            vec!["typeenv".to_string(), "defeq_fallback".to_string()],
            ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
                reason: "TASK-826 guarded TypeEnv defeq fallback identity".to_string(),
            },
        ),
        name.to_string(),
    )
}

fn resolve_associated_interface_from_type_var_bounds(
    type_env: &TypeEnv,
    base_ty: &Type,
    base_spelling: &str,
    name: &str,
) -> Result<String, TypeEnvError> {
    let Type::Var(var) = base_ty else {
        return Err(TypeEnvError::InvalidDefinition(
            format!("unresolved associated type '{name}'"),
            Span::default(),
        ));
    };

    let Some(bounds) = type_env.type_var_interface_bounds.get(var) else {
        return Err(TypeEnvError::InvalidDefinition(
            format!("unresolved associated type '{name}'"),
            Span::default(),
        ));
    };

    let mut candidates = Vec::new();
    for bound_iface in bounds {
        match type_env.interfaces.get(bound_iface) {
            Some(iface_info)
                if iface_info
                    .associated_types
                    .iter()
                    .any(|assoc| assoc == name) =>
            {
                candidates.push(bound_iface.clone());
            }
            _ => {}
        }
    }

    if candidates.len() == 1 {
        Ok(candidates.into_iter().next().expect("single candidate"))
    } else if candidates.len() > 1 {
        let mut candidate_bounds = candidates;
        candidate_bounds.sort();
        Err(TypeEnvError::AmbiguousAssociatedType {
            name: format!(
                "{name}' for projection '{}::{}' with candidate bounds [{}]",
                base_spelling,
                name,
                candidate_bounds.join(", ")
            ),
            span: Span::default(),
        })
    } else {
        Err(TypeEnvError::InvalidDefinition(
            format!("unresolved associated type '{name}'"),
            Span::default(),
        ))
    }
}

fn lower_explicit_associated_family_projection_to_type(
    type_env: &TypeEnv,
    interface: &str,
    args: &[SurfaceType],
    member: &str,
    param_mapping: &HashMap<String, TypeVar>,
    span: Span,
) -> Result<Type, TypeEnvError> {
    let declaration = type_env
        .lookup_associated_family_declaration(interface, member)
        .ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "unknown sealed associated-family projection '<{interface}<...>>::{member}'"
                ),
                span,
            )
        })?;
    if declaration.interface_params.len() != args.len() {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "associated-family projection '{}::{}' expects {} interface arguments, found {}",
                interface,
                member,
                declaration.interface_params.len(),
                args.len()
            ),
            span,
        ));
    }
    let args = args
        .iter()
        .map(|arg| surface_type_to_type(arg, param_mapping, type_env))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Type::Associated {
        interface: interface.to_string(),
        base: Box::new(Type::Constructor {
            name: QualifiedName::root(interface),
            args,
            kind: Kind::Type,
        }),
        name: member.to_string(),
    })
}

fn lower_core_explicit_associated_family_projection_to_type(
    type_env: &TypeEnv,
    interface: &str,
    args: &[TypeExpr],
    member: &str,
    param_mapping: &HashMap<String, TypeVar>,
) -> Result<Type, TypeError> {
    let declaration = type_env
        .lookup_associated_family_declaration(interface, member)
        .ok_or_else(|| {
            TypeError::TypeEnv(Box::new(TypeEnvError::InvalidDefinition(
                format!(
                    "unknown sealed associated-family projection '<{interface}<...>>::{member}'"
                ),
                Span::default(),
            )))
        })?;
    if declaration.interface_params.len() != args.len() {
        return Err(TypeError::ConstructorArityMismatch {
            name: format!("{}::{}", interface, member),
            expected_arity: declaration.interface_params.len(),
            found_arity: args.len(),
            span: Span::default(),
        });
    }
    let args = args
        .iter()
        .map(|arg| type_expr_to_type(arg, param_mapping, type_env))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Type::Associated {
        interface: interface.to_string(),
        base: Box::new(Type::Constructor {
            name: QualifiedName::root(interface),
            args,
            kind: Kind::Type,
        }),
        name: member.to_string(),
    })
}

fn constructor_variable_application_to_type(
    constructor: &str,
    kind: &Kind,
    found_arity: usize,
    lower_args: impl FnOnce() -> Result<Vec<Type>, TypeEnvError>,
) -> Result<Type, TypeEnvError> {
    if kind.is_type() {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "proper type variable '{constructor}' of kind * cannot be applied as a constructor"
            ),
            Span::default(),
        ));
    }
    let expected_arity = kind.arity();
    if found_arity != expected_arity {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "wrong arity for constructor variable '{constructor}': expected {expected_arity}, found {found_arity}"
            ),
            Span::default(),
        ));
    }
    Ok(Type::ConstructorVariableApp {
        constructor: constructor.to_string(),
        args: lower_args()?,
        kind: Kind::Type,
    })
}

fn surface_type_to_type(
    ty: &SurfaceType,
    param_mapping: &HashMap<String, TypeVar>,
    type_env: &TypeEnv,
) -> Result<Type, TypeEnvError> {
    match ty {
        SurfaceType::Hole { span } => Err(TypeEnvError::InvalidDefinition(
            "type holes are only accepted in audited SPEC-066 do-target positions; this semantic lowering path does not accept source holes"
                .to_string(),
            *span,
        )),
        SurfaceType::Name(name) => {
            if let Some(var) = param_mapping.get(name.as_ref()) {
                if let Some(kind) = type_env.type_parameter_kind(name.as_ref())
                    && !kind.is_type()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "constructor variable '{}' has kind {}; expected a fully applied proper type",
                            name, kind
                        ),
                        Span::default(),
                    ));
                }
                return Ok(Type::Var(*var));
            }

            match name.as_ref() {
                "Int" => Ok(Type::Int),
                "String" => Ok(Type::String),
                "Bool" => Ok(Type::Bool),
                "Float" => Ok(Type::Float),
                "Null" | "Unit" => Ok(Type::Null),
                "Time" => Ok(Type::Time),
                "Ref" => Ok(Type::Ref),
                "()" => Ok(Type::Constructor {
                    name: QualifiedName::root("()"),
                    args: vec![],
                    kind: Kind::Type,
                }),
                _ => {
                    let (qualified, _) = type_env.resolve_type(name.as_ref()).map_err(|e| {
                        TypeEnvError::InvalidDefinition(format!("{e}"), Span::default())
                    })?;
                    type_env
                        .check_type_constructor_arity(&qualified, 0)
                        .map_err(|e| {
                            TypeEnvError::InvalidDefinition(format!("{e}"), Span::default())
                        })?;
                    Ok(Type::Constructor {
                        name: qualified,
                        args: vec![],
                        kind: Kind::Type,
                    })
                }
            }
        }
        SurfaceType::List(item) => surface_type_to_type(item, param_mapping, type_env)
            .map(|item| Type::List(Box::new(item))),
        SurfaceType::Tuple(items) => {
            let items = items
                .iter()
                .enumerate()
                .map(|(index, ty)| {
                    surface_type_to_type(ty, param_mapping, type_env)
                        .map(|ty| (tuple_field_name(index).into_boxed_str(), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Record(items))
        }
        SurfaceType::Record(fields) => {
            let fields = fields
                .iter()
                .map(|(name, ty)| {
                    surface_type_to_type(ty, param_mapping, type_env)
                        .map(|ty| (Box::from(name.as_ref()), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Record(fields))
        }
        SurfaceType::Capability(name) => Ok(Type::Cap {
            name: Box::from(name.as_ref()),
            effect: ash_core::Effect::Operational,
        }),
        SurfaceType::Constructor { name, args } => {
            if name.as_ref() == "List" && args.len() == 1 {
                surface_type_to_type(&args[0], param_mapping, type_env)
                    .map(|item| Type::List(Box::new(item)))
            } else if let Some(kind) = type_env.type_parameter_kind(name.as_ref()) {
                constructor_variable_application_to_type(name, kind, args.len(), || {
                    args.iter()
                        .map(|arg| surface_type_to_type(arg, param_mapping, type_env))
                        .collect::<Result<Vec<_>, _>>()
                })
            } else if param_mapping.contains_key(name.as_ref()) {
                Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "proper type variable '{}' of kind * cannot be applied as a constructor",
                        name
                    ),
                    Span::default(),
                ))
            } else {
                let (qualified, _) = type_env.resolve_type(name.as_ref()).map_err(|e| {
                    TypeEnvError::InvalidDefinition(format!("{e}"), Span::default())
                })?;
                type_env
                    .check_type_constructor_arity(&qualified, args.len())
                    .map_err(|e| {
                        TypeEnvError::InvalidDefinition(format!("{e}"), Span::default())
                    })?;
                let args = args
                    .iter()
                    .map(|arg| surface_type_to_type(arg, param_mapping, type_env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Constructor {
                    name: qualified,
                    args,
                    kind: Kind::Type,
                })
            }
        }

        SurfaceType::Fn(params, ret) => {
            let params = params
                .iter()
                .map(|param| surface_type_to_type(param, param_mapping, type_env))
                .collect::<Result<Vec<_>, _>>()?;
            let ret = surface_type_to_type(ret, param_mapping, type_env)?;
            Ok(Type::Fn(params, Box::new(ret)))
        }
        SurfaceType::Associated { base, name } => {
            let base_ty = surface_type_to_type(base, param_mapping, type_env)?;
            let interface = resolve_associated_interface_from_type_var_bounds(
                type_env,
                &base_ty,
                &surface_projection_base_spelling(base),
                name,
            )?;

            Ok(Type::Associated {
                interface,
                base: Box::new(base_ty),
                name: name.to_string(),
            })
        }
        SurfaceType::AssociatedFamilyProjection {
            interface,
            args,
            member,
            span,
        } => lower_explicit_associated_family_projection_to_type(
            type_env,
            interface,
            args,
            member,
            param_mapping,
            *span,
        ),
    }
}

fn surface_type_name(ty: &SurfaceType) -> Option<String> {
    match ty {
        SurfaceType::Name(name) => Some(name.to_string()),
        SurfaceType::Capability(name) => Some(name.to_string()),
        _ => None,
    }
}

fn core_projection_base_spelling(base: &TypeExpr) -> String {
    match base {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Constructor { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}<{}>",
                    name,
                    args.iter()
                        .map(core_projection_base_spelling)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        TypeExpr::Tuple(items) => format!("Tuple({})", items.len()),
        TypeExpr::Record(fields) => format!("Record({})", fields.len()),
        TypeExpr::Associated { base, name } => {
            format!("{}::{}", core_projection_base_spelling(base), name)
        }
    }
}

fn surface_projection_base_spelling(base: &SurfaceType) -> String {
    match base {
        SurfaceType::Hole { .. } => "_".to_string(),
        SurfaceType::Name(name) => name.to_string(),
        SurfaceType::Constructor { name, args } => {
            if args.is_empty() {
                name.to_string()
            } else {
                format!(
                    "{}<{}>",
                    name,
                    args.iter()
                        .map(surface_projection_base_spelling)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        SurfaceType::Tuple(items) => format!("Tuple({})", items.len()),
        SurfaceType::Record(fields) => format!("Record({})", fields.len()),
        SurfaceType::List(_) => "List".to_string(),
        SurfaceType::Capability(name) => format!("Capability({name})"),
        SurfaceType::Fn(_, _) => "Fn".to_string(),
        SurfaceType::Associated { base, name } => {
            format!("{}::{}", surface_projection_base_spelling(base), name)
        }
        SurfaceType::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => format!(
            "<{}<{}>>::{}",
            interface,
            args.iter()
                .map(surface_projection_base_spelling)
                .collect::<Vec<_>>()
                .join(", "),
            member
        ),
    }
}

fn source_span_from_parser_span(span: Span) -> ash_core::ast::Span {
    ash_core::ast::Span {
        start: span.start,
        end: span.end,
    }
}

fn proposition_source_anchor(
    origin: SourceOrigin,
    span: Span,
    label: impl Into<String>,
) -> SourceAnchor {
    SourceAnchor::new(origin, Some(source_span_from_parser_span(span)), label)
}

fn proposition_module_source_origin(module: &ModuleIdentity) -> SourceOrigin {
    match &module.source {
        ash_core::semantic_summary::ModuleSourceOrigin::File(path) => {
            SourceOrigin::File(path.clone())
        }
        ash_core::semantic_summary::ModuleSourceOrigin::Inline { parent, offset } => {
            SourceOrigin::InlineModule {
                module: *parent,
                offset: *offset,
            }
        }
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic { reason } => {
            SourceOrigin::Synthetic {
                reason: reason.clone(),
            }
        }
    }
}

fn synthetic_proposition_source_anchor(label: impl Into<String>) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "typeenv proposition environment".to_string(),
        },
        None,
        label,
    )
}

fn type_var_proposition_term(var: TypeVar) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Var(format!("type_var_{}", var.0)))
}

fn proposition_term_from_canonical(expr: CanonicalTypeExpr) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(expr)
}

fn proposition_normalization_error(error: crate::normalizer::NormalizationError) -> TypeError {
    TypeEnvError::InvalidDefinition(
        format!("proposition normalization failed: {error:?}"),
        Span::default(),
    )
    .into()
}

fn proposition_revalidation_error(error: TypeError) -> TypeEnvError {
    match error {
        TypeError::TypeEnv(error) => *error,
        other => TypeEnvError::InvalidDefinition(
            format!("proposition fact revalidation failed: {other}"),
            Span::default(),
        ),
    }
}

fn constructor_kinded_binder_error(
    site: &str,
    name: &str,
    kind: &Kind,
    task: &str,
    span: Span,
) -> TypeEnvError {
    TypeEnvError::InvalidDefinition(
        format!(
            "{site} kinded binders are parsed by TASK-906 but require {task}; binder '{name}' has kind {kind}"
        ),
        span,
    )
}

fn reject_constructor_kinded_interface_params(
    params: &[InterfaceTypeParam],
    site: &str,
    task: &str,
) -> Result<(), TypeEnvError> {
    for param in params {
        if let Some(annotation) = &param.kind
            && annotation.kind != Kind::Type
        {
            return Err(constructor_kinded_binder_error(
                site,
                param.name.as_ref(),
                &annotation.kind,
                task,
                param.span,
            ));
        }
    }

    Ok(())
}

fn reject_constructor_kinded_proposition_params(
    params: &[PropositionPredicateParam],
    site: &str,
    task: &str,
) -> Result<(), TypeEnvError> {
    for param in params {
        if let Some(annotation) = &param.kind
            && annotation.kind != Kind::Type
        {
            return Err(constructor_kinded_binder_error(
                site,
                param.name.as_ref(),
                &annotation.kind,
                task,
                param.span,
            ));
        }
    }

    Ok(())
}

fn required_proposition_discharge_error(
    owner_site: &PropositionCheckingSite,
    source_anchor: &SourceAnchor,
    outcome: &PropositionOutcome,
) -> TypeEnvError {
    let site = owner_site
        .label
        .as_deref()
        .unwrap_or("unlabelled proposition checking point");
    TypeEnvError::PropositionDiagnostic {
        kind: proposition_diagnostic_kind_from_outcome(outcome),
        proposition: proposition_shape_from_outcome(outcome),
        expected: format!("a proposition discharged at checking point '{site}'"),
        found: proposition_found_shape_from_outcome(outcome),
        solver_rule: proposition_solver_rule_from_outcome(outcome).into(),
        help: proposition_help_from_outcome(outcome),
        span: anchor_span(source_anchor),
    }
}

fn proposition_help_from_outcome(outcome: &PropositionOutcome) -> String {
    let mut help = "add explicit evidence, use a closed proposition, or move unsupported proof search behind an assumption/imported evidence boundary".to_string();
    if matches!(
        outcome,
        PropositionOutcome::Deferred(reason)
            if !matches!(reason.proposition, TypeProposition::Disequality(_))
                && reason.no_inversion_boundary
                && matches!(
                    reason.kind,
                    PropositionDeferredKind::BlockedByNeutrality { .. }
                        | PropositionDeferredKind::RigidAssociatedProjection
                        | PropositionDeferredKind::RequiresTypeFunctionInversion
                        | PropositionDeferredKind::RequiresAssociatedFamilyInversion
                )
    ) {
        help.push_str("; Ash normalized both sides but did not solve under type functions or associated families");
    }
    help
}

fn proposition_shape_from_outcome(outcome: &PropositionOutcome) -> String {
    match outcome {
        PropositionOutcome::Satisfied(evidence) => format!("{:?}", evidence.proposition),
        PropositionOutcome::Refuted(refutation) => format!("{:?}", refutation.proposition),
        PropositionOutcome::Deferred(reason) => format!("{:?}", reason.proposition),
    }
}

fn proposition_found_shape_from_outcome(outcome: &PropositionOutcome) -> String {
    match outcome {
        PropositionOutcome::Satisfied(_) => {
            format!(
                "satisfied by {}",
                proposition_solver_rule_from_outcome(outcome)
            )
        }
        PropositionOutcome::Refuted(refutation) => {
            format!("refuted by {:?}", refutation.reason)
        }
        PropositionOutcome::Deferred(reason) => {
            format!("deferred by {:?}", reason.kind)
        }
    }
}

fn proposition_solver_rule_from_outcome(outcome: &PropositionOutcome) -> &'static str {
    match outcome {
        PropositionOutcome::Satisfied(evidence) => match evidence.rule {
            PropositionEvidenceRule::DefinitionalEquality => "normalize-and-compare equality",
            PropositionEvidenceRule::SealedDomainConstructorDisjointness => {
                "sealed-domain constructor disjointness"
            }
            PropositionEvidenceRule::NominalHeadDisjointness => "nominal-head disjointness",
            PropositionEvidenceRule::InScopeInterfaceBound => "in-scope interface-bound evidence",
            PropositionEvidenceRule::ConcreteImplEvidence => "concrete impl evidence",
            PropositionEvidenceRule::NamedPredicateAssumption => "named-predicate assumption",
            PropositionEvidenceRule::ImportedSummaryFact => "imported proposition summary fact",
        },
        PropositionOutcome::Refuted(refutation) => match refutation.reason {
            PropositionRefutationReason::DefinitionalEquality => {
                "normalize-and-compare disequality refutation"
            }
            PropositionRefutationReason::ClosedHeadMismatch => "closed-head mismatch",
            PropositionRefutationReason::InterfaceEvidenceNotFound => "interface evidence lookup",
            PropositionRefutationReason::NamedPredicateRefuted => "named-predicate refutation",
            PropositionRefutationReason::ImportedSummaryRefutation => "imported summary refutation",
        },
        PropositionOutcome::Deferred(reason) => match reason.kind {
            PropositionDeferredKind::BlockedByNeutrality { .. } => {
                "normalize-and-compare deferred on neutral head"
            }
            PropositionDeferredKind::RigidAssociatedProjection => {
                "normalize-and-compare deferred on rigid associated projection"
            }
            PropositionDeferredKind::RequiresTypeFunctionInversion => {
                "normalize-and-compare without type-function inversion"
            }
            PropositionDeferredKind::RequiresAssociatedFamilyInversion => {
                "normalize-and-compare without associated-family inversion"
            }
            PropositionDeferredKind::UnsupportedNamedPredicate => {
                "defer unsupported named predicate solving"
            }
            PropositionDeferredKind::MissingInterfaceEvidence => "interface evidence lookup",
            PropositionDeferredKind::UnsupportedProofSearch => "unsupported proof search boundary",
        },
    }
}

fn proposition_diagnostic_kind_from_outcome(
    outcome: &PropositionOutcome,
) -> PropositionDiagnosticKind {
    match outcome {
        PropositionOutcome::Satisfied(_) => {
            unreachable!("satisfied proposition outcome cannot produce required-discharge error")
        }
        PropositionOutcome::Refuted(refutation) => match refutation.reason {
            PropositionRefutationReason::DefinitionalEquality => {
                PropositionDiagnosticKind::DisequalityRefutedByEquality
            }
            PropositionRefutationReason::InterfaceEvidenceNotFound => {
                PropositionDiagnosticKind::InterfaceBoundNotFound
            }
            PropositionRefutationReason::ClosedHeadMismatch
            | PropositionRefutationReason::NamedPredicateRefuted
            | PropositionRefutationReason::ImportedSummaryRefutation => {
                PropositionDiagnosticKind::DisequalityOpenOrNeutral
            }
        },
        PropositionOutcome::Deferred(reason) => match reason.kind {
            PropositionDeferredKind::BlockedByNeutrality { .. } => {
                if matches!(reason.proposition, TypeProposition::Disequality(_)) {
                    PropositionDiagnosticKind::DisequalityOpenOrNeutral
                } else {
                    PropositionDiagnosticKind::EqualityBlockedByNeutralHead
                }
            }
            PropositionDeferredKind::RigidAssociatedProjection => {
                if matches!(reason.proposition, TypeProposition::Disequality(_)) {
                    PropositionDiagnosticKind::DisequalityOpenOrNeutral
                } else {
                    PropositionDiagnosticKind::EqualityBlockedByRigidProjection
                }
            }
            PropositionDeferredKind::RequiresTypeFunctionInversion
            | PropositionDeferredKind::RequiresAssociatedFamilyInversion => {
                PropositionDiagnosticKind::NoInversionBoundary
            }
            PropositionDeferredKind::UnsupportedNamedPredicate => {
                PropositionDiagnosticKind::UnsupportedNamedPredicateSolving
            }
            PropositionDeferredKind::MissingInterfaceEvidence => {
                PropositionDiagnosticKind::InterfaceBoundNotFound
            }
            PropositionDeferredKind::UnsupportedProofSearch => {
                PropositionDiagnosticKind::DisequalityOpenOrNeutral
            }
        },
    }
}

fn private_proposition_dependency_error(
    public_item: &str,
    dependency_kind: &str,
    dependency: &str,
    span: Span,
) -> TypeEnvError {
    TypeEnvError::PropositionDiagnostic {
        kind: PropositionDiagnosticKind::PrivatePropositionDependencyLeak,
        proposition: format!("public {public_item} proposition summary"),
        expected: "a public proposition summary containing only public dependencies".into(),
        found: format!("private {dependency_kind} '{dependency}'"),
        solver_rule: "fail-closed proposition summary export validation".into(),
        help: "make the dependency public, remove it from the public proposition, or keep the proposition private".into(),
        span,
    }
}

fn proposition_comparison_terms(
    lhs: NormalTypeExpr,
    rhs: NormalTypeExpr,
) -> PropositionTypeComparisonEvidence {
    PropositionTypeComparisonEvidence { lhs, rhs }
}

fn proposition_satisfaction(
    proposition: &TypeProposition,
    normalized_terms: Option<PropositionTypeComparisonEvidence>,
    rule: PropositionEvidenceRule,
    source_anchor: Option<SourceAnchor>,
) -> PropositionOutcome {
    PropositionOutcome::Satisfied(PropositionEvidence {
        proposition: proposition.clone(),
        normalized_terms,
        rule,
        source_anchor,
        boundary: PropositionBoundary::Local,
    })
}

fn proposition_refutation(
    proposition: &TypeProposition,
    normalized_terms: Option<PropositionTypeComparisonEvidence>,
    reason: PropositionRefutationReason,
    source_anchor: Option<SourceAnchor>,
) -> PropositionOutcome {
    PropositionOutcome::Refuted(PropositionRefutation {
        proposition: proposition.clone(),
        normalized_terms,
        reason,
        source_anchor,
        boundary: PropositionBoundary::Local,
    })
}

fn proposition_deferral(
    proposition: &TypeProposition,
    kind: PropositionDeferredKind,
    source_anchor: Option<SourceAnchor>,
    no_inversion_boundary: bool,
) -> PropositionOutcome {
    PropositionOutcome::Deferred(PropositionDeferredReason {
        proposition: proposition.clone(),
        kind,
        source_anchor,
        no_inversion_boundary,
    })
}

fn proposition_deferred_kind_from_blocked_normals(
    lhs_norm: &NormalTypeExpr,
    rhs_norm: &NormalTypeExpr,
) -> PropositionDeferredKind {
    let mut blockers = Vec::new();
    collect_proposition_blockers(lhs_norm, &mut blockers);
    collect_proposition_blockers(rhs_norm, &mut blockers);
    proposition_deferred_kind_from_blockers(&blockers)
}

fn proposition_deferred_kind_from_blockers(blockers: &[NormalTypeExpr]) -> PropositionDeferredKind {
    if blockers.iter().any(|blocker| {
        matches!(
            blocker,
            NormalTypeExpr::Projection {
                rigidity: ProjectionRigidity::Rigid,
                ..
            } | NormalTypeExpr::Projection {
                reason: Some(NormalFormBlockReason::RigidProjection),
                ..
            }
        )
    }) {
        return PropositionDeferredKind::RigidAssociatedProjection;
    }

    if let Some(blocker) = blockers.iter().find_map(normal_form_block_reason) {
        return PropositionDeferredKind::BlockedByNeutrality { blocker };
    }

    PropositionDeferredKind::UnsupportedProofSearch
}

fn normal_form_block_reason(normal: &NormalTypeExpr) -> Option<NormalFormBlockReason> {
    match normal {
        NormalTypeExpr::NeutralComputationApp { reason, .. } => Some(reason.clone()),
        NormalTypeExpr::ConstructorVariableApp { reason, .. } => Some(reason.clone()),
        NormalTypeExpr::Projection { reason, .. } => {
            Some(reason.clone().unwrap_or(NormalFormBlockReason::Unsupported))
        }
        NormalTypeExpr::Primitive(_)
        | NormalTypeExpr::Var(_)
        | NormalTypeExpr::NominalApp { .. }
        | NormalTypeExpr::DomainConstructorApp { .. }
        | NormalTypeExpr::PromotedDataConstructorApp { .. } => None,
    }
}

fn collect_proposition_blockers(normal: &NormalTypeExpr, blockers: &mut Vec<NormalTypeExpr>) {
    match normal {
        NormalTypeExpr::ConstructorVariableApp { args, .. } => {
            blockers.push(normal.clone());
            for arg in args {
                collect_proposition_blockers(arg, blockers);
            }
        }
        NormalTypeExpr::NeutralComputationApp { .. } | NormalTypeExpr::Projection { .. } => {
            blockers.push(normal.clone());
        }
        NormalTypeExpr::NominalApp { args, .. }
        | NormalTypeExpr::DomainConstructorApp { args, .. }
        | NormalTypeExpr::PromotedDataConstructorApp { args, .. } => {
            for arg in args {
                collect_proposition_blockers(arg, blockers);
            }
        }
        NormalTypeExpr::Primitive(_) | NormalTypeExpr::Var(_) => {}
    }
}

fn proposition_normal_form_is_open_or_blocked(normal: &NormalTypeExpr) -> bool {
    match normal {
        NormalTypeExpr::Var(_)
        | NormalTypeExpr::ConstructorVariableApp { .. }
        | NormalTypeExpr::NeutralComputationApp { .. }
        | NormalTypeExpr::Projection { .. } => true,
        NormalTypeExpr::NominalApp { args, .. }
        | NormalTypeExpr::DomainConstructorApp { args, .. }
        | NormalTypeExpr::PromotedDataConstructorApp { args, .. } => {
            args.iter().any(proposition_normal_form_is_open_or_blocked)
        }
        NormalTypeExpr::Primitive(_) => false,
    }
}

fn sealed_domain_constructor_heads_are_disjoint(
    lhs_norm: &NormalTypeExpr,
    rhs_norm: &NormalTypeExpr,
) -> bool {
    matches!(
        (lhs_norm, rhs_norm),
        (
            NormalTypeExpr::DomainConstructorApp {
                constructor: lhs_constructor,
                domain: lhs_domain,
                ..
            },
            NormalTypeExpr::DomainConstructorApp {
                constructor: rhs_constructor,
                domain: rhs_domain,
                ..
            }
        ) if lhs_domain == rhs_domain && lhs_constructor != rhs_constructor
    )
}

fn synthetic_proposition_module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(usize::MAX)),
        ModuleId(usize::MAX - 875),
        vec!["typeenv".to_string(), "propositions".to_string()],
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
            reason: "TASK-875 proposition predicate fallback identity".to_string(),
        },
    )
}

fn canonical_expr_contains_var(expr: &CanonicalTypeExpr) -> bool {
    match expr {
        CanonicalTypeExpr::Var(_) => true,
        CanonicalTypeExpr::ConstructorVariableApp(_) => true,
        CanonicalTypeExpr::NominalApp { args, .. }
        | CanonicalTypeExpr::Projection { args, .. }
        | CanonicalTypeExpr::ComputationHeadApp { args, .. } => {
            args.iter().any(canonical_expr_contains_var)
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
            app.args.iter().any(canonical_expr_contains_var)
        }
        CanonicalTypeExpr::Primitive(_) => false,
    }
}

fn projection_rigidity_for_canonical_args(args: &[CanonicalTypeExpr]) -> ProjectionRigidity {
    if args.iter().any(canonical_expr_contains_var) {
        ProjectionRigidity::Neutral
    } else {
        ProjectionRigidity::Rigid
    }
}

fn associated_family_result_contains_var(expr: &AssociatedFamilyResultExpr) -> bool {
    match expr {
        AssociatedFamilyResultExpr::Var { .. } => true,
        AssociatedFamilyResultExpr::NominalApp { args, .. }
        | AssociatedFamilyResultExpr::DomainConstructorApp { args, .. }
        | AssociatedFamilyResultExpr::AssociatedFamilyProjection {
            interface_args: args,
            ..
        }
        | AssociatedFamilyResultExpr::Projection { args, .. }
        | AssociatedFamilyResultExpr::ComputationHeadApp { args, .. } => {
            args.iter().any(associated_family_result_contains_var)
        }
        AssociatedFamilyResultExpr::Primitive { .. } => false,
    }
}

fn projection_rigidity_for_associated_family_args(
    args: &[AssociatedFamilyResultExpr],
) -> ProjectionRigidity {
    if args.iter().any(associated_family_result_contains_var) {
        ProjectionRigidity::Neutral
    } else {
        ProjectionRigidity::Rigid
    }
}

fn canonical_projection_base_spelling(base: &CanonicalTypeExpr) -> String {
    match base {
        CanonicalTypeExpr::Var(name) | CanonicalTypeExpr::Primitive(name) => name.clone(),
        CanonicalTypeExpr::NominalApp {
            visible_name, args, ..
        } => {
            if args.is_empty() {
                visible_name.clone()
            } else {
                format!(
                    "{}<{}>",
                    visible_name,
                    args.iter()
                        .map(canonical_projection_base_spelling)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args,
            ..
        } => {
            if args.is_empty() {
                format!("{}::{}", interface.name, member.name)
            } else {
                format!(
                    "{}<{}>::{}",
                    interface.name,
                    args.iter()
                        .map(canonical_projection_base_spelling)
                        .collect::<Vec<_>>()
                        .join(", "),
                    member.name
                )
            }
        }
        CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
            if args.is_empty() {
                head.name.clone()
            } else {
                format!(
                    "{}<{}>",
                    head.name,
                    args.iter()
                        .map(canonical_projection_base_spelling)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
            if app.args.is_empty() {
                app.constructor.name.clone()
            } else {
                format!(
                    "{}<{}>",
                    app.constructor.name,
                    app.args
                        .iter()
                        .map(canonical_projection_base_spelling)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        CanonicalTypeExpr::ConstructorVariableApp(app) => {
            if app.args.is_empty() {
                app.constructor.name.clone()
            } else {
                format!(
                    "{}<{}>",
                    app.constructor.name,
                    app.args
                        .iter()
                        .map(canonical_projection_base_spelling)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn provenance_source_kind(kind: CapabilityImplementationDependencyKind) -> ProvenanceSourceKind {
    match kind {
        CapabilityImplementationDependencyKind::Resource => ProvenanceSourceKind::Resource,
        CapabilityImplementationDependencyKind::Capability => ProvenanceSourceKind::Capability,
        CapabilityImplementationDependencyKind::Config => ProvenanceSourceKind::Config,
    }
}

fn classify_authority_provenance(
    dependencies: &[CapabilityImplementationDependencyInfo],
) -> AuthorityProvenanceKind {
    if dependencies
        .iter()
        .any(|dep| dep.kind == CapabilityImplementationDependencyKind::Capability)
    {
        AuthorityProvenanceKind::Derived
    } else if dependencies
        .iter()
        .any(|dep| dep.kind == CapabilityImplementationDependencyKind::Resource)
    {
        AuthorityProvenanceKind::Internal
    } else {
        AuthorityProvenanceKind::NoAuthority
    }
}

fn implementation_authority_sources(
    dependencies: &[CapabilityImplementationDependencyInfo],
) -> Vec<ImplementationAuthoritySourceInfo> {
    dependencies
        .iter()
        .map(|dependency| ImplementationAuthoritySourceInfo {
            kind: provenance_source_kind(dependency.kind),
            dependency_name: dependency.name.clone(),
            target_name: dependency
                .target_name
                .clone()
                .unwrap_or_else(|| dependency.ty.to_string()),
        })
        .collect()
}

fn looks_like_unbound_type_var_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn span_anchor(span: Span, label: impl Into<String>) -> SourceAnchor {
    let core_span = ash_core::ast::Span {
        start: span.start,
        end: span.end,
    };
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "type expression lowering".to_string(),
        },
        Some(core_span),
        label,
    )
}

fn surface_type_contains_hole(ty: &SurfaceType) -> bool {
    surface_type_hole_count(ty) > 0
}

fn surface_type_hole_count(ty: &SurfaceType) -> usize {
    match ty {
        SurfaceType::Hole { .. } => 1,
        SurfaceType::Name(_) | SurfaceType::Capability(_) => 0,
        SurfaceType::List(item) => surface_type_hole_count(item),
        SurfaceType::Tuple(items) | SurfaceType::Fn(items, _) => {
            items.iter().map(surface_type_hole_count).sum::<usize>()
                + match ty {
                    SurfaceType::Fn(_, ret) => surface_type_hole_count(ret),
                    _ => 0,
                }
        }
        SurfaceType::Record(fields) => fields
            .iter()
            .map(|(_, field_ty)| surface_type_hole_count(field_ty))
            .sum(),
        SurfaceType::Constructor { args, .. }
        | SurfaceType::AssociatedFamilyProjection { args, .. } => {
            args.iter().map(surface_type_hole_count).sum()
        }
        SurfaceType::Associated { base, .. } => surface_type_hole_count(base),
    }
}

fn bare_constructor_hole_hint(constructor: &str, arity: usize) -> String {
    match (constructor, arity) {
        ("Result", 2) => "Result<_, E>".to_string(),
        (_, 0) => constructor.to_string(),
        (_, 1) => format!("{constructor}<_>"),
        _ => {
            let args = std::iter::once("_".to_string())
                .chain((1..arity).map(|index| format!("T{index}")))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{constructor}<{args}>")
        }
    }
}

fn core_visibility_from_surface(visibility: &SurfaceVisibility) -> ash_core::ast::Visibility {
    match visibility {
        SurfaceVisibility::Inherited => ash_core::ast::Visibility::Private,
        SurfaceVisibility::Public => ash_core::ast::Visibility::Public,
        SurfaceVisibility::Crate => ash_core::ast::Visibility::Crate,
        SurfaceVisibility::Super { .. }
        | SurfaceVisibility::Self_
        | SurfaceVisibility::Restricted { .. } => ash_core::ast::Visibility::Private,
    }
}

fn constraint_for_param(param: &TypeFunctionParam) -> TypeFunctionPatternConstraint {
    param
        .domain_constraint
        .clone()
        .map(TypeFunctionPatternConstraint::Domain)
        .unwrap_or_else(|| TypeFunctionPatternConstraint::Kind(param.kind.clone()))
}

fn associated_family_constraint_to_type_function_pattern(
    constraint: &AssociatedFamilyResultConstraint,
) -> TypeFunctionPatternConstraint {
    match constraint {
        AssociatedFamilyResultConstraint::Kind(kind) => {
            TypeFunctionPatternConstraint::Kind(kind.clone())
        }
        AssociatedFamilyResultConstraint::Domain(domain) => {
            TypeFunctionPatternConstraint::Domain(domain.clone())
        }
    }
}

type CurrentTypeFunctionHead<'a> = (
    &'a str,
    &'a TypeComputationHeadId,
    &'a [TypeFunctionParam],
    &'a TypeFunctionResultConstraint,
);

struct TypeFunctionResultLoweringContext<'a> {
    pattern_vars: &'a HashMap<String, TypeFunctionPatternConstraint>,
    current_head: Option<CurrentTypeFunctionHead<'a>>,
    later_names: &'a HashSet<String>,
}

fn result_constraint_from_pattern(
    constraint: &TypeFunctionPatternConstraint,
) -> TypeFunctionResultConstraint {
    match constraint {
        TypeFunctionPatternConstraint::Kind(kind) => {
            TypeFunctionResultConstraint::Kind(kind.clone())
        }
        TypeFunctionPatternConstraint::Domain(domain) => {
            TypeFunctionResultConstraint::Domain(domain.clone())
        }
    }
}

fn type_function_result_from_canonical(
    canonical: CanonicalTypeExpr,
    span: Span,
) -> Result<TypeFunctionResultExpr, TypeEnvError> {
    match canonical {
        CanonicalTypeExpr::Primitive(name) => Ok(TypeFunctionResultExpr::Primitive {
            name: name.clone(),
            kind: Kind::Type,
            constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
            source_anchor: span_anchor(span, format!("primitive type {name}")),
        }),
        CanonicalTypeExpr::Var(name) => Ok(TypeFunctionResultExpr::Var {
            name: name.clone(),
            kind: Kind::Type,
            constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
            source_anchor: span_anchor(span, format!("type variable {name}")),
        }),
        CanonicalTypeExpr::NominalApp {
            origin,
            visible_name,
            args,
            kind,
        } => Ok(TypeFunctionResultExpr::NominalApp {
            origin,
            visible_name: visible_name.clone(),
            args: args
                .into_iter()
                .map(|arg| type_function_result_from_canonical(arg, span))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            constraint: TypeFunctionResultConstraint::Kind(kind),
            source_anchor: span_anchor(span, format!("nominal type {visible_name}")),
        }),
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args,
            kind,
            rigidity,
        } => Ok(TypeFunctionResultExpr::Projection {
            interface,
            member,
            args: args
                .into_iter()
                .map(|arg| type_function_result_from_canonical(arg, span))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            constraint: TypeFunctionResultConstraint::Kind(kind),
            rigidity,
            source_anchor: span_anchor(span, "associated projection"),
        }),
        CanonicalTypeExpr::ComputationHeadApp { head, args, kind } => {
            Ok(TypeFunctionResultExpr::ComputationHeadApp {
                head,
                args: args
                    .into_iter()
                    .map(|arg| type_function_result_from_canonical(arg, span))
                    .collect::<Result<Vec<_>, _>>()?,
                kind: kind.clone(),
                constraint: TypeFunctionResultConstraint::Kind(kind),
                source_anchor: span_anchor(span, "type function call"),
            })
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
            Ok(TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor: Box::new(app.constructor),
                data_kind: Box::new(app.data_kind),
                args: app
                    .args
                    .into_iter()
                    .map(|arg| type_function_result_from_canonical(arg, span))
                    .collect::<Result<Vec<_>, _>>()?,
                kind: app.kind.clone(),
                constraint: TypeFunctionResultConstraint::Kind(app.kind),
                source_anchor: span_anchor(span, "promoted data constructor"),
            })
        }
        CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
            format!(
                "constructor-variable application '{}' cannot be lowered to a type-function result until TASK-907 tracks constructor variables",
                app.constructor.name
            ),
            span,
        )),
    }
}

fn canonical_type_expr_head_name(expr: &CanonicalTypeExpr) -> String {
    match expr {
        CanonicalTypeExpr::Primitive(name) => format!("primitive '{name}'"),
        CanonicalTypeExpr::Var(name) => format!("type variable '{name}'"),
        CanonicalTypeExpr::NominalApp { visible_name, .. } => {
            format!("ordinary type '{visible_name}'")
        }
        CanonicalTypeExpr::Projection { member, .. } => {
            format!("associated projection '{}'", member.name)
        }
        CanonicalTypeExpr::ComputationHeadApp { head, .. } => {
            format!("type function '{}'", head.name)
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
            format!("promoted data constructor '{}'", app.constructor.name)
        }
        CanonicalTypeExpr::ConstructorVariableApp(app) => {
            format!(
                "constructor-variable application '{}'",
                app.constructor.name
            )
        }
    }
}

fn type_function_result_expr_head_name(expr: &TypeFunctionResultExpr) -> String {
    match expr {
        TypeFunctionResultExpr::Primitive { name, .. } => format!("primitive '{name}'"),
        TypeFunctionResultExpr::Var { name, .. } => format!("type variable '{name}'"),
        TypeFunctionResultExpr::NominalApp { visible_name, .. } => {
            format!("ordinary type '{visible_name}'")
        }
        TypeFunctionResultExpr::DomainConstructorApp { constructor, .. } => {
            format!("sealed-domain constructor '{}'", constructor.name)
        }
        TypeFunctionResultExpr::PromotedDataConstructorApp { constructor, .. } => {
            format!("promoted data constructor '{}'", constructor.name)
        }
        TypeFunctionResultExpr::Projection { member, .. } => {
            format!("associated projection '{}'", member.name)
        }
        TypeFunctionResultExpr::ComputationHeadApp { head, .. } => {
            format!("type function '{}'", head.name)
        }
    }
}

fn associated_family_result_from_canonical(
    canonical: CanonicalTypeExpr,
    span: Span,
) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
    match canonical {
        CanonicalTypeExpr::Primitive(name) => Ok(AssociatedFamilyResultExpr::Primitive {
            name: name.clone(),
            kind: Kind::Type,
            constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
            source_anchor: span_anchor(span, format!("primitive type {name}")),
        }),
        CanonicalTypeExpr::Var(name) => Ok(AssociatedFamilyResultExpr::Var {
            name: name.clone(),
            kind: Kind::Type,
            constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
            source_anchor: span_anchor(span, format!("type variable {name}")),
        }),
        CanonicalTypeExpr::NominalApp {
            origin,
            visible_name,
            args,
            kind,
        } => Ok(AssociatedFamilyResultExpr::NominalApp {
            origin,
            visible_name: visible_name.clone(),
            args: args
                .into_iter()
                .map(|arg| associated_family_result_from_canonical(arg, span))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            constraint: AssociatedFamilyResultConstraint::Kind(kind),
            source_anchor: span_anchor(span, format!("nominal type {visible_name}")),
        }),
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args,
            kind,
            rigidity,
        } => Ok(AssociatedFamilyResultExpr::Projection {
            interface,
            member,
            args: args
                .into_iter()
                .map(|arg| associated_family_result_from_canonical(arg, span))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            constraint: AssociatedFamilyResultConstraint::Kind(kind),
            rigidity,
            source_anchor: span_anchor(span, "associated projection"),
        }),
        CanonicalTypeExpr::ComputationHeadApp { head, args, kind } => {
            Ok(AssociatedFamilyResultExpr::ComputationHeadApp {
                head,
                args: args
                    .into_iter()
                    .map(|arg| associated_family_result_from_canonical(arg, span))
                    .collect::<Result<Vec<_>, _>>()?,
                kind: kind.clone(),
                constraint: AssociatedFamilyResultConstraint::Kind(kind),
                source_anchor: span_anchor(span, "type function call"),
            })
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => Err(TypeEnvError::InvalidDefinition(
            format!(
                "promoted data constructor '{}' cannot be used as an associated-family result; promoted data constructors are not representable in associated-family result expressions",
                app.constructor.name
            ),
            span,
        )),
        CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
            format!(
                "constructor-variable application '{}' cannot be used as an associated-family result until TASK-907 tracks constructor variables and TASK-908 defines higher-kinded interface evidence",
                app.constructor.name
            ),
            span,
        )),
    }
}

fn associated_family_result_from_normal(
    normal: NormalTypeExpr,
    source_anchor: SourceAnchor,
) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
    let span = anchor_span(&source_anchor);
    match normal {
        NormalTypeExpr::Primitive(name) => Ok(AssociatedFamilyResultExpr::Primitive {
            name,
            kind: Kind::Type,
            constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
            source_anchor,
        }),
        NormalTypeExpr::Var(name) => Ok(AssociatedFamilyResultExpr::Var {
            name,
            kind: Kind::Type,
            constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
            source_anchor,
        }),
        NormalTypeExpr::NominalApp {
            origin,
            visible_name,
            args,
            kind,
        } => Ok(AssociatedFamilyResultExpr::NominalApp {
            origin,
            visible_name,
            args: args
                .into_iter()
                .map(|arg| associated_family_result_from_normal(arg, source_anchor.clone()))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            constraint: AssociatedFamilyResultConstraint::Kind(kind),
            source_anchor,
        }),
        NormalTypeExpr::DomainConstructorApp {
            constructor,
            domain,
            args,
            kind,
        } => Ok(AssociatedFamilyResultExpr::DomainConstructorApp {
            constructor,
            domain: domain.clone(),
            args: args
                .into_iter()
                .map(|arg| associated_family_result_from_normal(arg, source_anchor.clone()))
                .collect::<Result<Vec<_>, _>>()?,
            kind,
            constraint: AssociatedFamilyResultConstraint::Domain(domain),
            source_anchor,
        }),
        NormalTypeExpr::NeutralComputationApp {
            head, args, kind, ..
        } => Ok(AssociatedFamilyResultExpr::ComputationHeadApp {
            head,
            args: args
                .into_iter()
                .map(|arg| associated_family_result_from_normal(arg, source_anchor.clone()))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            constraint: AssociatedFamilyResultConstraint::Kind(kind),
            source_anchor,
        }),
        NormalTypeExpr::Projection {
            interface,
            member,
            args,
            kind,
            rigidity,
            ..
        } => Ok(AssociatedFamilyResultExpr::Projection {
            interface,
            member,
            args: args
                .into_iter()
                .map(|arg| associated_family_result_from_normal(arg, source_anchor.clone()))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            constraint: AssociatedFamilyResultConstraint::Kind(kind),
            rigidity,
            source_anchor,
        }),
        NormalTypeExpr::ConstructorVariableApp { constructor, .. } => {
            Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor-variable application '{}' cannot be published as an associated-family result until TASK-907 tracks constructor variables",
                    constructor.name
                ),
                span,
            ))
        }
        NormalTypeExpr::PromotedDataConstructorApp { constructor, .. } => {
            Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' cannot be used as an associated-family result; promoted data constructors are not representable in associated-family result expressions",
                    constructor.name
                ),
                span,
            ))
        }
    }
}

fn associated_family_selection_blocker_to_normal_reason(
    blocker: AssociatedFamilySelectionBlocker,
) -> NormalFormBlockReason {
    match blocker {
        AssociatedFamilySelectionBlocker::NoApplicableScheme => {
            NormalFormBlockReason::MissingAssociatedEvidence
        }
        AssociatedFamilySelectionBlocker::AbstractScrutinee => {
            NormalFormBlockReason::AbstractScrutinee
        }
        AssociatedFamilySelectionBlocker::NeutralScrutinee => {
            NormalFormBlockReason::NeutralScrutinee
        }
        AssociatedFamilySelectionBlocker::RigidProjection => NormalFormBlockReason::RigidProjection,
        AssociatedFamilySelectionBlocker::Ambiguous => {
            NormalFormBlockReason::AmbiguousAssociatedFamilySelection
        }
    }
}

fn matches_associated_family_result_constraint(
    canonical: &CanonicalTypeExpr,
    constraint: &AssociatedFamilyResultConstraint,
) -> bool {
    match (canonical, constraint) {
        (CanonicalTypeExpr::Primitive(name), AssociatedFamilyResultConstraint::Kind(kind)) => {
            name == "Type" && kind == &Kind::Type
        }
        (CanonicalTypeExpr::Var(name), AssociatedFamilyResultConstraint::Domain(domain)) => {
            name == &domain.name
        }
        _ => false,
    }
}

fn canonical_expr_for_associated_family_constraint(
    constraint: &AssociatedFamilyResultConstraint,
) -> CanonicalTypeExpr {
    match constraint {
        AssociatedFamilyResultConstraint::Kind(Kind::Type) => {
            CanonicalTypeExpr::Primitive("Type".to_string())
        }
        AssociatedFamilyResultConstraint::Kind(kind) => {
            CanonicalTypeExpr::Primitive(format!("{kind:?}"))
        }
        AssociatedFamilyResultConstraint::Domain(domain) => {
            CanonicalTypeExpr::Var(domain.name.clone())
        }
    }
}

fn associated_family_result_expr_to_canonical(
    expr: &AssociatedFamilyResultExpr,
) -> Result<CanonicalTypeExpr, TypeEnvError> {
    match expr {
        AssociatedFamilyResultExpr::Primitive { name, .. } => {
            Ok(CanonicalTypeExpr::Primitive(name.clone()))
        }
        AssociatedFamilyResultExpr::Var { name, .. } => Ok(CanonicalTypeExpr::Var(name.clone())),
        AssociatedFamilyResultExpr::NominalApp {
            origin,
            visible_name,
            args,
            kind,
            ..
        } => Ok(CanonicalTypeExpr::NominalApp {
            origin: origin.clone(),
            visible_name: visible_name.clone(),
            args: args
                .iter()
                .map(associated_family_result_expr_to_canonical)
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
        }),
        AssociatedFamilyResultExpr::Projection {
            interface,
            member,
            args,
            kind,
            rigidity,
            ..
        } => Ok(CanonicalTypeExpr::Projection {
            interface: interface.clone(),
            member: member.clone(),
            args: args
                .iter()
                .map(associated_family_result_expr_to_canonical)
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
            rigidity: *rigidity,
        }),
        AssociatedFamilyResultExpr::ComputationHeadApp {
            head, args, kind, ..
        } => Ok(CanonicalTypeExpr::ComputationHeadApp {
            head: head.clone(),
            args: args
                .iter()
                .map(associated_family_result_expr_to_canonical)
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
        }),
        AssociatedFamilyResultExpr::AssociatedFamilyProjection {
            head,
            source_anchor,
            ..
        } => Err(TypeEnvError::InvalidDefinition(
            format!(
                "associated-family summary dependency closure cannot losslessly represent nested associated-family projection argument '{}::{}'",
                head.interface.name, head.member.name
            ),
            anchor_span(source_anchor),
        )),
        AssociatedFamilyResultExpr::DomainConstructorApp {
            constructor,
            source_anchor,
            ..
        } => Err(TypeEnvError::InvalidDefinition(
            format!(
                "associated-family summary dependency closure cannot losslessly represent nested domain-constructor argument '{}'",
                constructor.name
            ),
            anchor_span(source_anchor),
        )),
    }
}

fn hidden_imported_associated_family_heads(
    summaries: &[ModuleSemanticSummary],
) -> HashSet<AssociatedFamilyHeadId> {
    let mut hidden = HashSet::new();
    let visible_heads = summaries
        .iter()
        .flat_map(|summary| summary.exported_associated_families.iter())
        .filter(|family| !is_dependency_metadata_name(&family.visible_name))
        .map(|family| family.head.clone())
        .collect::<HashSet<_>>();
    for summary in summaries {
        for family in &summary.exported_associated_families {
            if is_dependency_metadata_name(&family.visible_name)
                && !visible_heads.contains(&family.head)
            {
                hidden.insert(family.head.clone());
            }
            for dependency in &family.dependency_closure.associated_families {
                if !dependency.source_visible && !visible_heads.contains(&dependency.family) {
                    hidden.insert(dependency.family.clone());
                }
            }
        }
    }
    hidden
}

fn is_dependency_metadata_name(visible_name: &str) -> bool {
    visible_name.starts_with("$ash_dependency$")
}

fn dependency_metadata_name(visible_name: &str) -> String {
    const DEPENDENCY_METADATA_PREFIX: &str = "$ash_dependency$";
    if visible_name.starts_with(DEPENDENCY_METADATA_PREFIX) {
        visible_name.to_string()
    } else {
        format!("{DEPENDENCY_METADATA_PREFIX}{visible_name}")
    }
}

fn associated_family_result_constraint_label(
    constraint: &AssociatedFamilyResultConstraint,
) -> String {
    match constraint {
        AssociatedFamilyResultConstraint::Kind(kind) => format!("kind {kind:?}"),
        AssociatedFamilyResultConstraint::Domain(domain) => {
            format!("sealed domain '{}'", domain.name)
        }
    }
}

#[allow(dead_code)]
fn resolve_associated_types_for_interface(
    ty: &mut Type,
    interface: &str,
    interface_type_params: &[TypeVar],
) {
    match ty {
        Type::Associated {
            interface: iface,
            base,
            ..
        } => match (iface.is_empty(), base.as_ref()) {
            (true, Type::Var(v)) if interface_type_params.contains(v) => {
                *iface = interface.to_string();
            }
            _ => {}
        },
        Type::Constructor { args, .. } => {
            for arg in args {
                resolve_associated_types_for_interface(arg, interface, interface_type_params);
            }
        }
        Type::List(inner) => {
            resolve_associated_types_for_interface(inner, interface, interface_type_params);
        }
        Type::Record(fields) => {
            for (_, field_ty) in fields {
                resolve_associated_types_for_interface(field_ty, interface, interface_type_params);
            }
        }
        Type::Fn(params, ret) => {
            for param in params {
                resolve_associated_types_for_interface(param, interface, interface_type_params);
            }
            resolve_associated_types_for_interface(ret, interface, interface_type_params);
        }
        Type::Fun(params, ret, _) => {
            for param in params {
                resolve_associated_types_for_interface(param, interface, interface_type_params);
            }
            resolve_associated_types_for_interface(ret, interface, interface_type_params);
        }
        _ => {}
    }
}

fn unresolved_associated_projection_name(ty: &Type) -> Option<&str> {
    match ty {
        Type::Associated { base, .. } => unresolved_associated_projection_name(base),
        Type::Constructor { args, .. } => {
            args.iter().find_map(unresolved_associated_projection_name)
        }
        Type::List(inner) => unresolved_associated_projection_name(inner),
        Type::Record(fields) => fields
            .iter()
            .find_map(|(_, field_ty)| unresolved_associated_projection_name(field_ty)),
        Type::Fn(params, ret) => params
            .iter()
            .find_map(unresolved_associated_projection_name)
            .or_else(|| unresolved_associated_projection_name(ret)),
        Type::Fun(params, ret, _) => params
            .iter()
            .find_map(unresolved_associated_projection_name)
            .or_else(|| unresolved_associated_projection_name(ret)),
        _ => None,
    }
}

fn is_closed_world_nominal_impl_target(ty: &Type) -> bool {
    match ty {
        Type::Int
        | Type::String
        | Type::Bool
        | Type::Float
        | Type::Null
        | Type::Time
        | Type::Ref
        | Type::Instance { .. }
        | Type::InstanceAddr { .. }
        | Type::ControlLink { .. } => true,
        Type::List(_)
        | Type::Record(_)
        | Type::Cap { .. }
        | Type::Fun(_, _, _)
        | Type::Fn(_, _) => false,
        Type::Var(_) => false,
        Type::Constructor { args, .. } => args.iter().all(is_closed_world_nominal_impl_target),
        Type::ConstructorVariableApp { .. } => false,
        Type::Associated { .. } => false,
    }
}

fn interface_param_kind(param: &InterfaceTypeParam) -> Kind {
    param
        .kind
        .as_ref()
        .map(|annotation| annotation.kind.clone())
        .unwrap_or(Kind::Type)
}

fn interface_param_kinds(params: &[InterfaceTypeParam]) -> Vec<Kind> {
    params.iter().map(interface_param_kind).collect()
}

fn render_type_constructor_head(head: &TypeConstructorHeadId) -> String {
    match head {
        TypeConstructorHeadId::Nominal { visible_name, .. } => visible_name.clone(),
        TypeConstructorHeadId::Computation(head) => head.name.clone(),
        _ => "<unsupported-type-constructor-head>".to_string(),
    }
}

fn render_type_constructor_expr(expr: &TypeConstructorExpr) -> String {
    match expr {
        TypeConstructorExpr::ProperType(ty) => format!("{ty:?}"),
        TypeConstructorExpr::ConstructorHead(head) => render_type_constructor_head(head),
        TypeConstructorExpr::PartialApplication(app) => render_type_constructor_head(&app.head),
        _ => "<unsupported-type-constructor-expr>".to_string(),
    }
}

fn type_contains_constructor_variable_app(ty: &Type) -> bool {
    match ty {
        Type::ConstructorVariableApp { .. } => true,
        Type::List(inner) => type_contains_constructor_variable_app(inner),
        Type::Record(fields) => fields
            .iter()
            .any(|(_, ty)| type_contains_constructor_variable_app(ty)),
        Type::Fun(params, ret, _) | Type::Fn(params, ret) => {
            params.iter().any(type_contains_constructor_variable_app)
                || type_contains_constructor_variable_app(ret)
        }
        Type::Constructor { args, .. } => args.iter().any(type_contains_constructor_variable_app),
        Type::Associated { base, .. } => type_contains_constructor_variable_app(base),
        Type::Int
        | Type::String
        | Type::Bool
        | Type::Float
        | Type::Null
        | Type::Time
        | Type::Ref
        | Type::Var(_)
        | Type::Cap { .. }
        | Type::Instance { .. }
        | Type::InstanceAddr { .. }
        | Type::ControlLink { .. } => false,
    }
}

fn apply_constructor_evidence_arg(
    arg: &InterfaceEvidenceArg,
    applied_args: &[Type],
    param_mapping: &HashMap<String, TypeVar>,
) -> Option<Type> {
    match arg {
        InterfaceEvidenceArg::Constructor(expr) => match expr.as_ref() {
            TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::Nominal {
                visible_name,
                ..
            }) => Some(Type::Constructor {
                name: QualifiedName::root(visible_name.clone()),
                args: applied_args.to_vec(),
                kind: Kind::Type,
            }),
            TypeConstructorExpr::PartialApplication(app) => {
                let TypeConstructorHeadId::Nominal { visible_name, .. } = &app.head else {
                    return None;
                };
                let mut applied = applied_args.iter();
                let mut args = Vec::with_capacity(app.args.len());
                for arg in &app.args {
                    match arg {
                        PartialTypeArg::Hole(_) => args.push(applied.next()?.clone()),
                        PartialTypeArg::Applied(canonical) => {
                            args.push(canonical_type_expr_to_type(canonical, param_mapping)?);
                        }
                        _ => return None,
                    }
                }
                if applied.next().is_some() {
                    return None;
                }
                Some(Type::Constructor {
                    name: QualifiedName::root(visible_name.clone()),
                    args,
                    kind: Kind::Type,
                })
            }
            _ => None,
        },
        _ => None,
    }
}

fn canonical_type_expr_to_type(
    expr: &CanonicalTypeExpr,
    param_mapping: &HashMap<String, TypeVar>,
) -> Option<Type> {
    match expr {
        CanonicalTypeExpr::Primitive(name) => match name.as_str() {
            "Int" => Some(Type::Int),
            "String" => Some(Type::String),
            "Bool" => Some(Type::Bool),
            "Float" => Some(Type::Float),
            "Null" | "Unit" => Some(Type::Null),
            "Time" => Some(Type::Time),
            "Ref" => Some(Type::Ref),
            _ => None,
        },
        CanonicalTypeExpr::Var(name) => param_mapping.get(name).copied().map(Type::Var),
        CanonicalTypeExpr::NominalApp {
            visible_name,
            args,
            kind,
            ..
        } => {
            let args = args
                .iter()
                .map(|arg| canonical_type_expr_to_type(arg, param_mapping))
                .collect::<Option<Vec<_>>>()?;
            Some(Type::Constructor {
                name: QualifiedName::root(visible_name.clone()),
                args,
                kind: kind.clone(),
            })
        }
        CanonicalTypeExpr::ConstructorVariableApp(app) => {
            let args = app
                .args
                .iter()
                .map(|arg| canonical_type_expr_to_type(arg, param_mapping))
                .collect::<Option<Vec<_>>>()?;
            Some(Type::ConstructorVariableApp {
                constructor: app.constructor.name.clone(),
                args,
                kind: app.kind.clone(),
            })
        }
        _ => None,
    }
}

fn substitute_constructor_variable_apps(
    ty: &Type,
    constructor_args: &HashMap<String, InterfaceEvidenceArg>,
    param_mapping: &HashMap<String, TypeVar>,
) -> Type {
    match ty {
        Type::ConstructorVariableApp {
            constructor,
            args,
            kind,
        } => {
            let args = args
                .iter()
                .map(|arg| {
                    substitute_constructor_variable_apps(arg, constructor_args, param_mapping)
                })
                .collect::<Vec<_>>();
            constructor_args
                .get(constructor)
                .and_then(|arg| apply_constructor_evidence_arg(arg, &args, param_mapping))
                .unwrap_or_else(|| Type::ConstructorVariableApp {
                    constructor: constructor.clone(),
                    args,
                    kind: kind.clone(),
                })
        }
        Type::List(inner) => Type::List(Box::new(substitute_constructor_variable_apps(
            inner,
            constructor_args,
            param_mapping,
        ))),
        Type::Record(fields) => Type::Record(
            fields
                .iter()
                .map(|(name, ty)| {
                    (
                        name.clone(),
                        substitute_constructor_variable_apps(ty, constructor_args, param_mapping),
                    )
                })
                .collect(),
        ),
        Type::Fun(params, ret, effect) => Type::Fun(
            params
                .iter()
                .map(|ty| substitute_constructor_variable_apps(ty, constructor_args, param_mapping))
                .collect(),
            Box::new(substitute_constructor_variable_apps(
                ret,
                constructor_args,
                param_mapping,
            )),
            *effect,
        ),
        Type::Fn(params, ret) => Type::Fn(
            params
                .iter()
                .map(|ty| substitute_constructor_variable_apps(ty, constructor_args, param_mapping))
                .collect(),
            Box::new(substitute_constructor_variable_apps(
                ret,
                constructor_args,
                param_mapping,
            )),
        ),
        Type::Constructor { name, args, kind } => Type::Constructor {
            name: name.clone(),
            args: args
                .iter()
                .map(|ty| substitute_constructor_variable_apps(ty, constructor_args, param_mapping))
                .collect(),
            kind: kind.clone(),
        },
        Type::Associated {
            interface,
            base,
            name,
        } => Type::Associated {
            interface: interface.clone(),
            base: Box::new(substitute_constructor_variable_apps(
                base,
                constructor_args,
                param_mapping,
            )),
            name: name.clone(),
        },
        other => other.clone(),
    }
}

fn render_interface_evidence_arg(arg: &InterfaceEvidenceArg) -> String {
    match arg {
        InterfaceEvidenceArg::Proper(ty) => ty.to_string(),
        InterfaceEvidenceArg::Constructor(expr) => render_type_constructor_expr(expr),
    }
}

fn render_interface_evidence_key(interface: &str, args: &[InterfaceEvidenceArg]) -> String {
    let args = args
        .iter()
        .map(render_interface_evidence_arg)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{interface}<{args}>")
}

fn interface_evidence_args_match(
    scheme_args: &[InterfaceEvidenceArg],
    requested_args: &[InterfaceEvidenceArg],
    allow_generic_match: bool,
) -> bool {
    scheme_args.len() == requested_args.len()
        && scheme_args
            .iter()
            .zip(requested_args)
            .all(|(scheme, requested)| {
                interface_evidence_arg_matches(scheme, requested, allow_generic_match)
            })
}

fn interface_evidence_arg_matches(
    scheme_arg: &InterfaceEvidenceArg,
    requested_arg: &InterfaceEvidenceArg,
    allow_generic_match: bool,
) -> bool {
    if !allow_generic_match {
        return scheme_arg == requested_arg;
    }

    match (scheme_arg, requested_arg) {
        (InterfaceEvidenceArg::Proper(scheme), InterfaceEvidenceArg::Proper(requested)) => {
            unify(scheme, requested).is_ok()
        }
        (
            InterfaceEvidenceArg::Constructor(scheme),
            InterfaceEvidenceArg::Constructor(requested),
        ) => type_constructor_expr_matches_pattern(scheme, requested),
        _ => false,
    }
}

fn type_constructor_expr_matches_pattern(
    pattern: &TypeConstructorExpr,
    requested: &TypeConstructorExpr,
) -> bool {
    let mut bindings = HashMap::new();
    type_constructor_expr_matches_pattern_inner(pattern, requested, &mut bindings)
}

fn type_constructor_expr_matches_pattern_inner(
    pattern: &TypeConstructorExpr,
    requested: &TypeConstructorExpr,
    bindings: &mut HashMap<String, CanonicalTypeExpr>,
) -> bool {
    match (pattern, requested) {
        (TypeConstructorExpr::ProperType(pattern), TypeConstructorExpr::ProperType(requested)) => {
            canonical_type_expr_matches_pattern(pattern, requested, bindings)
        }
        (
            TypeConstructorExpr::ConstructorHead(pattern),
            TypeConstructorExpr::ConstructorHead(requested),
        ) => type_constructor_heads_match(pattern, requested),
        (
            TypeConstructorExpr::PartialApplication(pattern),
            TypeConstructorExpr::PartialApplication(requested),
        ) => {
            type_constructor_heads_match(&pattern.head, &requested.head)
                && pattern.args.len() == requested.args.len()
                && pattern
                    .args
                    .iter()
                    .zip(&requested.args)
                    .all(|(pattern, requested)| {
                        partial_type_arg_matches_pattern(pattern, requested, bindings)
                    })
        }
        _ => false,
    }
}

fn type_constructor_heads_match(
    pattern: &TypeConstructorHeadId,
    requested: &TypeConstructorHeadId,
) -> bool {
    match (pattern, requested) {
        (
            TypeConstructorHeadId::Nominal {
                visible_name: pattern,
                ..
            },
            TypeConstructorHeadId::Nominal {
                visible_name: requested,
                ..
            },
        ) => pattern == requested,
        (
            TypeConstructorHeadId::Computation(pattern),
            TypeConstructorHeadId::Computation(requested),
        ) => pattern == requested,
        _ => false,
    }
}

fn partial_type_arg_matches_pattern(
    pattern: &PartialTypeArg,
    requested: &PartialTypeArg,
    bindings: &mut HashMap<String, CanonicalTypeExpr>,
) -> bool {
    match (pattern, requested) {
        (PartialTypeArg::Hole(_), PartialTypeArg::Hole(_)) => true,
        (PartialTypeArg::Applied(pattern), PartialTypeArg::Applied(requested)) => {
            canonical_type_expr_matches_pattern(pattern, requested, bindings)
        }
        _ => false,
    }
}

fn canonical_type_expr_matches_pattern(
    pattern: &CanonicalTypeExpr,
    requested: &CanonicalTypeExpr,
    bindings: &mut HashMap<String, CanonicalTypeExpr>,
) -> bool {
    match pattern {
        CanonicalTypeExpr::Var(name) => match bindings.get(name) {
            Some(bound) => bound == requested,
            None => {
                bindings.insert(name.clone(), requested.clone());
                true
            }
        },
        CanonicalTypeExpr::Primitive(pattern) => {
            matches!(requested, CanonicalTypeExpr::Primitive(requested) if pattern == requested)
        }
        CanonicalTypeExpr::NominalApp {
            visible_name: pattern_name,
            args: pattern_args,
            ..
        } => match requested {
            CanonicalTypeExpr::NominalApp {
                visible_name: requested_name,
                args: requested_args,
                ..
            } => {
                pattern_name == requested_name
                    && pattern_args.len() == requested_args.len()
                    && pattern_args
                        .iter()
                        .zip(requested_args)
                        .all(|(pattern, requested)| {
                            canonical_type_expr_matches_pattern(pattern, requested, bindings)
                        })
            }
            _ => false,
        },
        CanonicalTypeExpr::ConstructorVariableApp(pattern) => match requested {
            CanonicalTypeExpr::ConstructorVariableApp(requested) => {
                pattern.constructor.name == requested.constructor.name
                    && pattern.args.len() == requested.args.len()
                    && pattern
                        .args
                        .iter()
                        .zip(&requested.args)
                        .all(|(pattern, requested)| {
                            canonical_type_expr_matches_pattern(pattern, requested, bindings)
                        })
            }
            _ => false,
        },
        _ => pattern == requested,
    }
}

fn interface_evidence_arg_as_legacy_type(arg: &InterfaceEvidenceArg) -> Type {
    match arg {
        InterfaceEvidenceArg::Proper(ty) => ty.clone(),
        InterfaceEvidenceArg::Constructor(expr) => match expr.as_ref() {
            TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::Nominal {
                visible_name,
                ..
            }) => Type::Constructor {
                name: QualifiedName::root(visible_name.clone()),
                args: Vec::new(),
                kind: Kind::n_ary(1),
            },
            other => Type::Constructor {
                name: QualifiedName::root(render_type_constructor_expr(other)),
                args: Vec::new(),
                kind: Kind::n_ary(1),
            },
        },
    }
}

impl TypeInfo {
    /// Get the name of the type
    pub fn name(&self) -> &str {
        match self {
            TypeInfo::Enum { name, .. } => name,
            TypeInfo::Struct { name, .. } => name,
        }
    }

    /// Get the type parameters
    pub fn params(&self) -> &[TypeVar] {
        match self {
            TypeInfo::Enum { params, .. } => params,
            TypeInfo::Struct { params, .. } => params,
        }
    }

    /// Look up a variant by name (only for enums)
    pub fn lookup_variant(&self, variant_name: &str) -> Option<(VariantIndex, &VariantInfo)> {
        match self {
            TypeInfo::Enum { variants, .. } => variants
                .iter()
                .enumerate()
                .find(|(_, v)| v.name == variant_name),
            TypeInfo::Struct { .. } => None,
        }
    }
}

/// Convert an AST TypeDef to internal TypeInfo
fn convert_variant_fields(
    variant: &VariantDef,
    param_mapping: &HashMap<String, TypeVar>,
    type_env: &TypeEnv,
) -> Result<Vec<(FieldName, Type)>, TypeError> {
    match &variant.payload {
        VariantPayload::Unit => Ok(vec![]),
        VariantPayload::Record(fields) => fields
            .iter()
            .map(|(fname, ftype)| {
                type_expr_to_type(ftype, param_mapping, type_env).map(|ty| (fname.clone(), ty))
            })
            .collect(),
        VariantPayload::Tuple(items) => items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                type_expr_to_type(item, param_mapping, type_env)
                    .map(|ty| (tuple_field_name(index), ty))
            })
            .collect(),
    }
}

fn convert_variant_payload_shape(payload: &VariantPayload) -> VariantPayloadShape {
    match payload {
        VariantPayload::Unit => VariantPayloadShape::Unit,
        VariantPayload::Record(_) => VariantPayloadShape::Record,
        VariantPayload::Tuple(_) => VariantPayloadShape::Tuple,
    }
}

fn convert_type_def(type_def: &TypeDef, type_env: &TypeEnv) -> Result<TypeInfo, TypeError> {
    // Create mapping from param names to fresh type variables
    let param_mapping: HashMap<String, TypeVar> = type_def
        .params
        .iter()
        .map(|param| (param.clone(), TypeVar::fresh()))
        .collect();

    let params: Vec<TypeVar> = type_def
        .params
        .iter()
        .map(|p| param_mapping.get(p).copied().unwrap_or_else(TypeVar::fresh))
        .collect();

    match &type_def.body {
        TypeBody::Enum(variants) => {
            let converted_variants: Result<Vec<_>, _> = variants
                .iter()
                .map(|v| {
                    convert_variant_fields(v, &param_mapping, type_env).map(|fields| VariantInfo {
                        name: v.name.clone(),
                        fields,
                        payload_shape: convert_variant_payload_shape(&v.payload),
                    })
                })
                .collect();

            Ok(TypeInfo::Enum {
                name: type_def.name.clone(),
                params,
                variants: converted_variants?,
            })
        }
        TypeBody::Struct(fields) => {
            let converted_fields: Result<Vec<_>, _> = fields
                .iter()
                .map(|(fname, ftype)| {
                    type_expr_to_type(ftype, &param_mapping, type_env).map(|ty| (fname.clone(), ty))
                })
                .collect();

            Ok(TypeInfo::Struct {
                name: type_def.name.clone(),
                params,
                fields: converted_fields?,
            })
        }
        TypeBody::Alias(target_expr) => {
            // Expand alias to underlying type immediately
            let target_type = type_expr_to_type(target_expr, &param_mapping, type_env)?;
            // Store as a struct with the target type as a special field
            Ok(TypeInfo::Struct {
                name: type_def.name.clone(),
                params,
                fields: vec![("__alias_target".to_string(), target_type)],
            })
        }
    }
}

/// Non-denotable compiler-known parameter classes accepted by workflow intrinsics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowIntrinsicParameterClass {
    Requirement,
    OpenPostcondition,
}

impl WorkflowIntrinsicParameterClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requirement => "Requirement",
            Self::OpenPostcondition => "OpenPostcondition",
        }
    }
}

/// Compiler-known workflow intrinsic operation identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowIntrinsicKind {
    Requires,
    Ensures,
}

/// Compiler-known workflow intrinsic descriptor with typed opaque parameter metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowIntrinsic {
    pub kind: WorkflowIntrinsicKind,
    pub qualified_name: &'static str,
    pub parameter_class: WorkflowIntrinsicParameterClass,
    pub result_type: crate::types::Type,
}

impl WorkflowIntrinsic {
    #[must_use]
    pub fn requires(result_type: crate::types::Type) -> Self {
        Self {
            kind: WorkflowIntrinsicKind::Requires,
            qualified_name: "workflow::requires",
            parameter_class: WorkflowIntrinsicParameterClass::Requirement,
            result_type,
        }
    }

    #[must_use]
    pub fn ensures(result_type: crate::types::Type) -> Self {
        Self {
            kind: WorkflowIntrinsicKind::Ensures,
            qualified_name: "workflow::ensures",
            parameter_class: WorkflowIntrinsicParameterClass::OpenPostcondition,
            result_type,
        }
    }

    #[must_use]
    pub const fn parameter_class(&self) -> WorkflowIntrinsicParameterClass {
        self.parameter_class
    }

    #[must_use]
    pub const fn result_type(&self) -> &crate::types::Type {
        &self.result_type
    }
}

/// Public manifest role for a visible computation-tower entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicTowerManifestKind {
    /// A source-visible `Monad<K>` construction algebra.
    Monad,
    /// A source-visible opaque process-handle surface, not a constructor API.
    ProcessHandle,
    /// A source-visible library/domain example that participates in tower tests.
    DomainExample,
}

/// Authority source for a public tower operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicTowerOperationAuthority {
    /// The operation is requested by public, nameable Ash algebra.
    VisibleAlgebra,
    /// Reserved for detecting regressions where runtime/compiler magic becomes
    /// an independent construction authority.
    HiddenSemanticRoot,
}

/// Role played by a public tower operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicTowerOperationRole {
    Return,
    Bind,
    Then,
    ExplicitLift,
    Process,
    WorkflowContract,
    DomainConstructor,
    DomainBind,
}

/// Runtime/compiler implementation class for a visible tower operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicTowerIntrinsicKind {
    /// Ordinary stdlib/library implementation.
    LibrarySurface,
    /// Runtime intrinsic backing a public stdlib operation.
    RuntimeIntrinsic,
    /// Compiler-prelude evidence retained during migration but shaped like an
    /// ordinary selected operation.
    CompilerPreludeEvidence,
    /// Public data constructor used as a domain return operation.
    DataConstructor,
}

/// Mapping from a public operation to the intrinsic or library implementation
/// that backs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicTowerIntrinsicMapping {
    pub kind: PublicTowerIntrinsicKind,
    pub visible_operation: &'static str,
    pub implementation: &'static str,
}

/// Public algebra entry for a tower carrier or canonical domain example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicTowerAlgebra {
    pub name: &'static str,
    pub kind: PublicTowerManifestKind,
    pub nameable: bool,
    pub typeable: bool,
    pub user_constructible: bool,
    pub note: &'static str,
}

/// Public operation entry in the computation-tower manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicTowerOperation {
    pub name: &'static str,
    pub algebra: &'static str,
    pub role: PublicTowerOperationRole,
    pub authority: PublicTowerOperationAuthority,
    pub nameable: bool,
    pub typeable: bool,
    pub intrinsic: PublicTowerIntrinsicMapping,
}

/// Public computation-tower manifest used by the type environment and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicTowerManifest {
    algebras: &'static [PublicTowerAlgebra],
    operations: &'static [PublicTowerOperation],
}

impl PublicTowerManifest {
    /// Return all public algebra entries in this manifest.
    #[must_use]
    pub const fn algebras(&self) -> &'static [PublicTowerAlgebra] {
        self.algebras
    }

    /// Return all public operation entries in this manifest.
    #[must_use]
    pub const fn operations(&self) -> &'static [PublicTowerOperation] {
        self.operations
    }

    /// Look up a public algebra entry by manifest name.
    #[must_use]
    pub fn algebra(&self, name: &str) -> Option<&'static PublicTowerAlgebra> {
        self.algebras.iter().find(|entry| entry.name == name)
    }

    /// Look up a public operation by fully-qualified source-visible name.
    #[must_use]
    pub fn operation(&self, name: &str) -> Option<&'static PublicTowerOperation> {
        self.operations.iter().find(|entry| entry.name == name)
    }
}

const PUBLIC_TOWER_ALGEBRAS: &[PublicTowerAlgebra] = &[
    PublicTowerAlgebra {
        name: "Act",
        kind: PublicTowerManifestKind::Monad,
        nameable: true,
        typeable: true,
        user_constructible: true,
        note: "effectful computation algebra; ActEnv remains runtime-owned",
    },
    PublicTowerAlgebra {
        name: "Proc",
        kind: PublicTowerManifestKind::Monad,
        nameable: true,
        typeable: true,
        user_constructible: true,
        note: "process-capable computation algebra; process identity remains runtime-owned",
    },
    PublicTowerAlgebra {
        name: "Workflow",
        kind: PublicTowerManifestKind::Monad,
        nameable: true,
        typeable: true,
        user_constructible: true,
        note: "workflow algebra is currently exposed through TypeEnv/prelude metadata",
    },
    PublicTowerAlgebra {
        name: "Result<_, E>",
        kind: PublicTowerManifestKind::Monad,
        nameable: true,
        typeable: true,
        user_constructible: true,
        note: "canonical partial-constructor domain algebra using Ok and result::and_then",
    },
    PublicTowerAlgebra {
        name: "Option",
        kind: PublicTowerManifestKind::DomainExample,
        nameable: true,
        typeable: true,
        user_constructible: true,
        note: "canonical user/library monad example for later selected evidence lowering",
    },
    PublicTowerAlgebra {
        name: "P",
        kind: PublicTowerManifestKind::ProcessHandle,
        nameable: true,
        typeable: true,
        user_constructible: false,
        note: "opaque process handle returned by Proc operations",
    },
];

const fn intrinsic(
    kind: PublicTowerIntrinsicKind,
    visible_operation: &'static str,
    implementation: &'static str,
) -> PublicTowerIntrinsicMapping {
    PublicTowerIntrinsicMapping {
        kind,
        visible_operation,
        implementation,
    }
}

const PUBLIC_TOWER_OPERATIONS: &[PublicTowerOperation] = &[
    PublicTowerOperation {
        name: "act::unit",
        algebra: "Act",
        role: PublicTowerOperationRole::Return,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::CompilerPreludeEvidence,
            "act::unit",
            "act::__unit",
        ),
    },
    PublicTowerOperation {
        name: "act::bind",
        algebra: "Act",
        role: PublicTowerOperationRole::Bind,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::CompilerPreludeEvidence,
            "act::bind",
            "act::__bind",
        ),
    },
    PublicTowerOperation {
        name: "proc::unit",
        algebra: "Proc",
        role: PublicTowerOperationRole::Return,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "proc::unit",
            "proc::unit",
        ),
    },
    PublicTowerOperation {
        name: "proc::bind",
        algebra: "Proc",
        role: PublicTowerOperationRole::Bind,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "proc::bind",
            "proc::bind",
        ),
    },
    PublicTowerOperation {
        name: "proc::from_act",
        algebra: "Proc",
        role: PublicTowerOperationRole::ExplicitLift,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "proc::from_act",
            "proc::from_act",
        ),
    },
    PublicTowerOperation {
        name: "proc::par",
        algebra: "Proc",
        role: PublicTowerOperationRole::Process,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "proc::par",
            "proc::par",
        ),
    },
    PublicTowerOperation {
        name: "proc::await",
        algebra: "Proc",
        role: PublicTowerOperationRole::Process,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "proc::await",
            "proc::await",
        ),
    },
    PublicTowerOperation {
        name: "workflow::unit",
        algebra: "Workflow",
        role: PublicTowerOperationRole::Return,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "workflow::unit",
            "workflow::unit",
        ),
    },
    PublicTowerOperation {
        name: "workflow::bind",
        algebra: "Workflow",
        role: PublicTowerOperationRole::Bind,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "workflow::bind",
            "workflow::bind",
        ),
    },
    PublicTowerOperation {
        name: "workflow::from_proc",
        algebra: "Workflow",
        role: PublicTowerOperationRole::ExplicitLift,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "workflow::from_proc",
            "workflow::from_proc",
        ),
    },
    PublicTowerOperation {
        name: "workflow::from_act",
        algebra: "Workflow",
        role: PublicTowerOperationRole::ExplicitLift,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::RuntimeIntrinsic,
            "workflow::from_act",
            "workflow::from_act",
        ),
    },
    PublicTowerOperation {
        name: "workflow::requires",
        algebra: "Workflow",
        role: PublicTowerOperationRole::WorkflowContract,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::CompilerPreludeEvidence,
            "workflow::requires",
            "workflow::requires",
        ),
    },
    PublicTowerOperation {
        name: "workflow::ensures",
        algebra: "Workflow",
        role: PublicTowerOperationRole::WorkflowContract,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::CompilerPreludeEvidence,
            "workflow::ensures",
            "workflow::ensures",
        ),
    },
    PublicTowerOperation {
        name: "Ok",
        algebra: "Result<_, E>",
        role: PublicTowerOperationRole::DomainConstructor,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(PublicTowerIntrinsicKind::DataConstructor, "Ok", "Ok"),
    },
    PublicTowerOperation {
        name: "result::and_then",
        algebra: "Result<_, E>",
        role: PublicTowerOperationRole::DomainBind,
        authority: PublicTowerOperationAuthority::VisibleAlgebra,
        nameable: true,
        typeable: true,
        intrinsic: intrinsic(
            PublicTowerIntrinsicKind::LibrarySurface,
            "result::and_then",
            "result::and_then",
        ),
    },
];

static PUBLIC_TOWER_MANIFEST: PublicTowerManifest = PublicTowerManifest {
    algebras: PUBLIC_TOWER_ALGEBRAS,
    operations: PUBLIC_TOWER_OPERATIONS,
};

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
    /// Compiler-known workflow intrinsics whose parameters are not source-denotable types.
    workflow_intrinsics: HashMap<String, WorkflowIntrinsic>,
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
    /// Workflow effect context for the three-vertex boundary (SPEC-031 §4.8).
    ///
    /// `Some(effect)` means we are type-checking inside a workflow body at the
    /// given effect level; closures (`Expr::FnDef`) are therefore typed as
    /// `Type::Fun(params, ret, effect)` rather than the pure `Type::Fn(params, ret)`.
    /// `None` means we are in a pure-fn or module-level context.
    workflow_effect: Option<ash_core::Effect>,
    /// True when type-checking a capability implementation operation body.
    ///
    /// Implementation bodies intentionally receive a stripped environment so
    /// they cannot use ambient variables, functions, capability symbols, or
    /// provider-style authority. This flag closes expression-level intrinsic
    /// escape hatches such as `invoke(...)` that bypass ordinary environment
    /// lookup.
    capability_implementation_body: bool,
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

impl TypeEnv {
    fn convert_interface_method(
        &self,
        method: &InterfaceMethodSig,
        param_mapping: &HashMap<String, TypeVar>,
        ordered_param_names: &[String],
        interface_name: &str,
    ) -> Result<(String, InterfaceMethodInfo), TypeEnvError> {
        // Allow multi-parameter interface methods for associated-type support (TASK-567)
        let mut method_env = self.clone();
        for name in ordered_param_names {
            method_env
                .type_var_interface_bounds
                .entry(param_mapping[name])
                .or_default()
                .insert(interface_name.to_string());
        }

        let params: Vec<Type> = method
            .params
            .iter()
            .map(|ty| surface_type_to_type(ty, param_mapping, &method_env))
            .collect::<Result<Vec<_>, _>>()?;

        let return_type = surface_type_to_type(&method.return_type, param_mapping, &method_env)?;

        let type_params: Vec<TypeVar> = ordered_param_names
            .iter()
            .map(|name| param_mapping[name])
            .collect();

        Ok((
            method.name.to_string(),
            InterfaceMethodInfo {
                type_params,
                params,
                return_type,
            },
        ))
    }

    /// Create a new empty type environment
    #[must_use]
    pub fn new() -> Self {
        Self {
            ast_types: HashMap::with_capacity(10),
            type_info: HashMap::with_capacity(10),
            constructors: HashMap::with_capacity(10),
            transparent_aliases: HashSet::with_capacity(4),
            type_declaration_states: HashMap::with_capacity(10),
            type_alias_identities: HashMap::with_capacity(10),
            canonical_type_names: HashMap::with_capacity(10),
            interface_identity_aliases: HashMap::with_capacity(4),
            interface_identity_alias_is_imported: HashMap::with_capacity(4),
            canonical_interface_names: HashMap::with_capacity(4),
            local_interface_arities: HashMap::with_capacity(4),
            known_interface_identities: HashSet::with_capacity(4),
            associated_member_identity_aliases: HashMap::with_capacity(4),
            associated_member_identity_alias_is_imported: HashMap::with_capacity(4),
            known_associated_member_identities: HashSet::with_capacity(4),
            interfaces: HashMap::with_capacity(4),
            capability_interfaces: HashMap::with_capacity(4),
            resource_types: HashMap::with_capacity(4),
            capability_implementations: HashMap::with_capacity(4),
            capability_bindings: HashMap::with_capacity(4),
            impls: Vec::new(),
            proposition_assumptions: Vec::new(),
            proposition_obligations: Vec::new(),
            proposition_predicate_aliases: HashMap::with_capacity(4),
            proposition_predicates: HashMap::with_capacity(4),
            type_var_interface_bounds: HashMap::with_capacity(4),
            type_parameter_kinds: HashMap::with_capacity(4),
            variables: HashMap::with_capacity(10),
            workflow_intrinsics: HashMap::with_capacity(2),
            public_workflow_summaries: HashMap::with_capacity(2),
            fn_contracts: HashMap::with_capacity(10),
            capability_symbols: HashSet::with_capacity(8),
            parent: None,
            providers: HashSet::new(),
            sealed_domain_identities: HashSet::new(),
            sealed_domain_aliases: HashMap::new(),
            sealed_domain_summaries: HashMap::new(),
            promoted_data_kind_identities: HashSet::new(),
            promoted_data_kind_aliases: HashMap::new(),
            promoted_data_kind_summaries: HashMap::new(),
            promoted_constructor_summaries: HashMap::new(),
            promoted_constructor_kinds: HashMap::new(),
            local_type_function_heads: HashMap::new(),
            local_type_functions: HashMap::new(),
            current_module_identity: None,
            associated_family_declarations: HashMap::new(),
            associated_family_name_index: HashMap::new(),
            associated_family_schemes: HashMap::new(),
            workflow_effect: None,
            capability_implementation_body: false,
        }
    }

    /// Return the workflow effect level currently in scope, if any.
    ///
    /// `Some(effect)` ⟹ we are inside a workflow body; closures get `Type::Fun`.
    /// `None`         ⟹ pure-fn or module-level context; closures get `Type::Fn`.
    #[must_use]
    pub fn workflow_effect(&self) -> Option<ash_core::Effect> {
        self.workflow_effect
    }

    /// Return the public computation-tower manifest for alpha tower algebra.
    #[must_use]
    pub fn public_tower_manifest(&self) -> &'static PublicTowerManifest {
        &PUBLIC_TOWER_MANIFEST
    }

    /// Set the module identity used for source-local semantic declarations.
    pub fn set_current_module_identity(&mut self, module: ModuleIdentity) {
        self.current_module_identity = Some(module);
    }

    /// Return the module identity used for source-local semantic declarations.
    #[must_use]
    pub fn current_module_identity(&self) -> Option<&ModuleIdentity> {
        self.current_module_identity.as_ref()
    }

    /// Set the source-local module identity only when the environment does not already have one.
    pub fn ensure_current_module_identity(&mut self, module: ModuleIdentity) {
        self.current_module_identity.get_or_insert(module);
    }

    fn ensure_local_interface_identity(
        &mut self,
        interface_name: &str,
        module: &ModuleIdentity,
    ) -> InterfaceIdentityId {
        if let Some(existing) = self.interface_identity_aliases.get(interface_name) {
            return existing.clone();
        }

        let identity = InterfaceIdentityId::new(module.clone(), interface_name.to_string());
        self.known_interface_identities.insert(identity.clone());
        self.canonical_interface_names
            .insert(identity.clone(), interface_name.to_string());
        self.interface_identity_aliases
            .insert(interface_name.to_string(), identity.clone());
        self.interface_identity_alias_is_imported
            .insert(interface_name.to_string(), false);
        identity
    }

    fn ensure_local_associated_member_identity(
        &mut self,
        interface_name: &str,
        interface: &InterfaceIdentityId,
        member_name: &str,
    ) -> AssociatedMemberIdentityId {
        let alias_key = (interface_name.to_string(), member_name.to_string());
        if let Some(existing) = self.associated_member_identity_aliases.get(&alias_key) {
            return existing.clone();
        }

        let identity = AssociatedMemberIdentityId::associated_type(
            interface.clone(),
            member_name.to_string(),
            vec![interface_name.to_string(), member_name.to_string()],
        );
        self.known_associated_member_identities
            .insert(identity.clone());
        self.associated_member_identity_aliases
            .insert(alias_key.clone(), identity.clone());
        self.associated_member_identity_alias_is_imported
            .insert(alias_key, false);
        identity
    }

    fn sealed_domain_constraint_from_surface(
        &self,
        ty: &SurfaceType,
        span: Span,
    ) -> Result<SealedDomainId, TypeEnvError> {
        let SurfaceType::Name(name) = ty else {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<declaration>".to_string(),
                reason: format!(
                    "expected sealed domain name, found {}",
                    surface_projection_base_spelling(ty)
                ),
                span,
            });
        };
        self.lookup_sealed_domain(name.as_ref())
            .map(|domain| domain.id.clone())
            .ok_or_else(|| TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<declaration>".to_string(),
                reason: format!("unknown sealed result domain '{name}'"),
                span,
            })
    }

    fn associated_family_result_constraint_from_surface(
        &self,
        ty: &SurfaceType,
        span: Span,
    ) -> Result<AssociatedFamilyResultConstraint, TypeEnvError> {
        if matches!(ty, SurfaceType::Name(name) if name.as_ref() == "Type") {
            return Ok(AssociatedFamilyResultConstraint::Kind(Kind::Type));
        }
        self.sealed_domain_constraint_from_surface(ty, span)
            .map(AssociatedFamilyResultConstraint::Domain)
    }

    fn optional_param_domain_constraint(
        &self,
        ty: Option<&SurfaceType>,
        span: Span,
    ) -> Result<Option<SealedDomainId>, TypeEnvError> {
        ty.map(|ty| self.sealed_domain_constraint_from_surface(ty, span))
            .transpose()
    }

    fn associated_family_declarations_for_interface(
        &self,
        interface_name: &str,
    ) -> Vec<&AssociatedFamilyDeclarationInfo> {
        self.associated_family_name_index
            .iter()
            .filter_map(|((candidate_interface, _), head)| {
                (candidate_interface == interface_name)
                    .then(|| self.associated_family_declarations.get(head))
                    .flatten()
            })
            .collect()
    }

    /// Look up sealed associated-family declaration metadata by visible interface/member names.
    #[must_use]
    pub fn lookup_associated_family_declaration(
        &self,
        interface_name: &str,
        family_name: &str,
    ) -> Option<&AssociatedFamilyDeclarationInfo> {
        let head = self
            .associated_family_name_index
            .get(&(interface_name.to_string(), family_name.to_string()))?;
        self.associated_family_declarations.get(head)
    }

    /// Return coherence-checked associated-family schemes for a canonical head.
    #[must_use]
    pub fn associated_family_schemes(
        &self,
        head: &AssociatedFamilyHeadId,
    ) -> Option<&Vec<RegisteredAssociatedFamilyScheme>> {
        self.associated_family_schemes.get(head)
    }

    /// Look up sealed associated-family declaration metadata by canonical head.
    #[must_use]
    pub fn lookup_associated_family_declaration_by_head(
        &self,
        head: &AssociatedFamilyHeadId,
    ) -> Option<&AssociatedFamilyDeclarationInfo> {
        self.associated_family_declarations.get(head)
    }

    /// Reduce one local associated-family projection from already-normalized
    /// normalizer arguments.
    ///
    /// This TASK-866/TASK-867 API consults validated local or imported family
    /// declarations and schemes that are normalizer-available in this `TypeEnv`.
    #[must_use]
    pub fn reduce_local_associated_family_projection_from_normal_args(
        &self,
        head: &AssociatedFamilyHeadId,
        interface_args: &[NormalTypeExpr],
    ) -> LocalAssociatedFamilyProjectionLookup<'_> {
        let Some(_declaration) = self.lookup_associated_family_declaration_by_head(head) else {
            let reason = if self.associated_member_identity_known(&head.member) {
                NormalFormBlockReason::AssociatedFamilyNotSealed
            } else {
                NormalFormBlockReason::MissingAssociatedEvidence
            };
            return LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason,
            };
        };

        let Some(schemes) = self.associated_family_schemes.get(head) else {
            return LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason: NormalFormBlockReason::AssociatedFamilyLocalUnavailable,
            };
        };

        let mut selected = Vec::new();
        let mut blocker = None;
        for registered in schemes {
            for equation in &registered.scheme.equations {
                if equation.interface_arg_patterns.len() != interface_args.len() {
                    continue;
                }
                let mut bindings = BTreeMap::new();
                match Self::match_associated_family_normal_pattern_spine(
                    &equation.interface_arg_patterns,
                    interface_args,
                    &mut bindings,
                ) {
                    Ok(()) => selected.push(SelectedNormalizedAssociatedFamilyScheme {
                        family_head: head.clone(),
                        registered,
                        equation,
                        scheme_param_bindings: bindings,
                    }),
                    Err(AssociatedFamilyMatchFailure::Blocked(reason)) => blocker = Some(reason),
                    Err(AssociatedFamilyMatchFailure::NoMatch) => {}
                }
            }
        }

        match selected.len() {
            1 => {
                let selected = selected.remove(0);
                let result = Self::substitute_associated_family_result_expr_from_normal_bindings(
                    &selected.equation.result,
                    &selected.scheme_param_bindings,
                );
                LocalAssociatedFamilyProjectionLookup::Reduced(Box::new(
                    LocalAssociatedFamilyReduction { selected, result },
                ))
            }
            n if n > 1 => LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason: NormalFormBlockReason::AmbiguousAssociatedFamilySelection,
            },
            _ => LocalAssociatedFamilyProjectionLookup::Blocked {
                family: Box::new(head.clone()),
                reason: blocker.map_or(
                    NormalFormBlockReason::MissingAssociatedEvidence,
                    associated_family_selection_blocker_to_normal_reason,
                ),
            },
        }
    }

    /// Select a unique associated-family scheme by one-way structural matching.
    #[must_use]
    pub fn select_associated_family_scheme(
        &self,
        head: &AssociatedFamilyHeadId,
        interface_args: &[CanonicalTypeExpr],
    ) -> AssociatedFamilySelection<'_> {
        let Some(schemes) = self.associated_family_schemes.get(head) else {
            return AssociatedFamilySelection::NoMatch {
                family: head.clone(),
            };
        };
        let mut selected = Vec::new();
        let mut blocker = None;
        for registered in schemes {
            for equation in &registered.scheme.equations {
                if equation.interface_arg_patterns.len() != interface_args.len() {
                    continue;
                }
                let mut bindings = BTreeMap::new();
                match Self::match_associated_family_pattern_spine(
                    &equation.interface_arg_patterns,
                    interface_args,
                    &mut bindings,
                ) {
                    Ok(()) => selected.push(SelectedAssociatedFamilyScheme {
                        family_head: head.clone(),
                        registered,
                        equation,
                        scheme_param_bindings: bindings,
                    }),
                    Err(AssociatedFamilyMatchFailure::Blocked(reason)) => blocker = Some(reason),
                    Err(AssociatedFamilyMatchFailure::NoMatch) => {}
                }
            }
        }
        match selected.len() {
            1 => AssociatedFamilySelection::Selected(selected.remove(0)),
            n if n > 1 => AssociatedFamilySelection::Ambiguous {
                family: head.clone(),
                candidate_count: n,
            },
            _ => blocker.map_or_else(
                || AssociatedFamilySelection::NoMatch {
                    family: head.clone(),
                },
                |reason| AssociatedFamilySelection::Blocked {
                    family: head.clone(),
                    reason,
                },
            ),
        }
    }

    /// Reduce a projection once when a unique associated-family scheme applies.
    pub fn reduce_associated_family_projection_once(
        &self,
        head: &AssociatedFamilyHeadId,
        interface_args: &[CanonicalTypeExpr],
    ) -> Result<AssociatedFamilyReduction<'_>, TypeEnvError> {
        match self.select_associated_family_scheme(head, interface_args) {
            AssociatedFamilySelection::Selected(selected) => {
                let result = Self::substitute_associated_family_result_expr(
                    &selected.equation.result,
                    &selected.scheme_param_bindings,
                );
                Ok(AssociatedFamilyReduction { selected, result })
            }
            AssociatedFamilySelection::Ambiguous {
                candidate_count, ..
            } => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "ambiguous associated-family selection for '{}::{}' with {candidate_count} candidates",
                    head.interface.name, head.member.name
                ),
                Span::default(),
            )),
            AssociatedFamilySelection::Blocked { reason, .. } => {
                Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family selection for '{}::{}' is blocked by {reason:?}",
                        head.interface.name, head.member.name
                    ),
                    Span::default(),
                ))
            }
            AssociatedFamilySelection::NoMatch { .. } => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "no associated-family scheme matches '{}::{}'",
                    head.interface.name, head.member.name
                ),
                Span::default(),
            )),
        }
    }

    fn match_associated_family_pattern_spine(
        patterns: &[AssociatedFamilyPattern],
        args: &[CanonicalTypeExpr],
        bindings: &mut BTreeMap<String, CanonicalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        for (pattern, arg) in patterns.iter().zip(args.iter()) {
            Self::match_associated_family_pattern(pattern, arg, bindings)?;
        }
        Ok(())
    }

    fn match_associated_family_pattern(
        pattern: &AssociatedFamilyPattern,
        arg: &CanonicalTypeExpr,
        bindings: &mut BTreeMap<String, CanonicalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match pattern {
            AssociatedFamilyPattern::Var { name, .. } => {
                Self::ensure_associated_family_arg_is_capturable(arg)?;
                match bindings.get(name) {
                    Some(existing) if existing == arg => Ok(()),
                    Some(_) => Err(AssociatedFamilyMatchFailure::NoMatch),
                    None => {
                        bindings.insert(name.clone(), arg.clone());
                        Ok(())
                    }
                }
            }
            AssociatedFamilyPattern::Wildcard { .. } => {
                Self::ensure_associated_family_arg_is_capturable(arg)
            }
            AssociatedFamilyPattern::Primitive { name, .. } => match arg {
                CanonicalTypeExpr::Primitive(arg_name) if name == arg_name => Ok(()),
                CanonicalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                CanonicalTypeExpr::ComputationHeadApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                CanonicalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::NominalApp {
                origin,
                visible_name,
                args: pattern_args,
                ..
            } => match arg {
                CanonicalTypeExpr::NominalApp {
                    origin: arg_origin,
                    visible_name: arg_name,
                    args: arg_args,
                    ..
                } if origin == arg_origin
                    && visible_name == arg_name
                    && pattern_args.len() == arg_args.len() =>
                {
                    Self::match_associated_family_pattern_spine(pattern_args, arg_args, bindings)
                }
                CanonicalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                CanonicalTypeExpr::ComputationHeadApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                CanonicalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::DomainConstructor { .. } => match arg {
                CanonicalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                CanonicalTypeExpr::ComputationHeadApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                CanonicalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
        }
    }

    fn ensure_associated_family_arg_is_capturable(
        arg: &CanonicalTypeExpr,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match arg {
            CanonicalTypeExpr::ComputationHeadApp { .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::NeutralScrutinee,
                ))
            }
            CanonicalTypeExpr::Projection { rigidity, .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                    ProjectionRigidity::Rigid => AssociatedFamilySelectionBlocker::RigidProjection,
                    ProjectionRigidity::Neutral => {
                        AssociatedFamilySelectionBlocker::NeutralScrutinee
                    }
                }))
            }
            CanonicalTypeExpr::PromotedDataConstructorApp { .. } => Err(
                AssociatedFamilyMatchFailure::Blocked(AssociatedFamilySelectionBlocker::Ambiguous),
            ),
            _ => Ok(()),
        }
    }

    fn match_associated_family_normal_pattern_spine(
        patterns: &[AssociatedFamilyPattern],
        args: &[NormalTypeExpr],
        bindings: &mut BTreeMap<String, NormalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        for (pattern, arg) in patterns.iter().zip(args.iter()) {
            Self::match_associated_family_normal_pattern(pattern, arg, bindings)?;
        }
        Ok(())
    }

    fn match_associated_family_normal_pattern(
        pattern: &AssociatedFamilyPattern,
        arg: &NormalTypeExpr,
        bindings: &mut BTreeMap<String, NormalTypeExpr>,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match pattern {
            AssociatedFamilyPattern::Var { name, .. } => {
                Self::ensure_associated_family_normal_arg_is_capturable(arg)?;
                match bindings.get(name) {
                    Some(existing) if existing == arg => Ok(()),
                    Some(_) => Err(AssociatedFamilyMatchFailure::NoMatch),
                    None => {
                        bindings.insert(name.clone(), arg.clone());
                        Ok(())
                    }
                }
            }
            AssociatedFamilyPattern::Wildcard { .. } => {
                Self::ensure_associated_family_normal_arg_is_capturable(arg)
            }
            AssociatedFamilyPattern::Primitive { name, .. } => match arg {
                NormalTypeExpr::Primitive(arg_name) if name == arg_name => Ok(()),
                NormalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                NormalTypeExpr::NeutralComputationApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                NormalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::NominalApp {
                origin,
                visible_name,
                args: pattern_args,
                ..
            } => match arg {
                NormalTypeExpr::NominalApp {
                    origin: arg_origin,
                    visible_name: arg_name,
                    args: arg_args,
                    ..
                } if origin == arg_origin
                    && visible_name == arg_name
                    && pattern_args.len() == arg_args.len() =>
                {
                    Self::match_associated_family_normal_pattern_spine(
                        pattern_args,
                        arg_args,
                        bindings,
                    )
                }
                NormalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                NormalTypeExpr::NeutralComputationApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                NormalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
            AssociatedFamilyPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                ..
            } => match arg {
                NormalTypeExpr::DomainConstructorApp {
                    constructor: arg_constructor,
                    domain: arg_domain,
                    args: arg_args,
                    ..
                } if constructor.as_ref() == arg_constructor
                    && domain.as_ref() == arg_domain
                    && fields.len() == arg_args.len() =>
                {
                    Self::match_associated_family_normal_pattern_spine(fields, arg_args, bindings)
                }
                NormalTypeExpr::Var(_) => Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::AbstractScrutinee,
                )),
                NormalTypeExpr::NeutralComputationApp { .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(
                        AssociatedFamilySelectionBlocker::NeutralScrutinee,
                    ))
                }
                NormalTypeExpr::Projection { rigidity, .. } => {
                    Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                        ProjectionRigidity::Rigid => {
                            AssociatedFamilySelectionBlocker::RigidProjection
                        }
                        ProjectionRigidity::Neutral => {
                            AssociatedFamilySelectionBlocker::NeutralScrutinee
                        }
                    }))
                }
                _ => Err(AssociatedFamilyMatchFailure::NoMatch),
            },
        }
    }

    fn ensure_associated_family_normal_arg_is_capturable(
        arg: &NormalTypeExpr,
    ) -> Result<(), AssociatedFamilyMatchFailure> {
        match arg {
            NormalTypeExpr::NeutralComputationApp { .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(
                    AssociatedFamilySelectionBlocker::NeutralScrutinee,
                ))
            }
            NormalTypeExpr::Projection { rigidity, .. } => {
                Err(AssociatedFamilyMatchFailure::Blocked(match rigidity {
                    ProjectionRigidity::Rigid => AssociatedFamilySelectionBlocker::RigidProjection,
                    ProjectionRigidity::Neutral => {
                        AssociatedFamilySelectionBlocker::NeutralScrutinee
                    }
                }))
            }
            NormalTypeExpr::PromotedDataConstructorApp { .. } => Err(
                AssociatedFamilyMatchFailure::Blocked(AssociatedFamilySelectionBlocker::Ambiguous),
            ),
            _ => Ok(()),
        }
    }

    fn substitute_associated_family_result_expr_from_normal_bindings(
        result: &AssociatedFamilyResultExpr,
        bindings: &BTreeMap<String, NormalTypeExpr>,
    ) -> AssociatedFamilyResultExpr {
        match result {
            AssociatedFamilyResultExpr::Var {
                name,
                source_anchor,
                ..
            } => bindings
                .get(name)
                .cloned()
                .and_then(|normal| {
                    associated_family_result_from_normal(normal, source_anchor.clone()).ok()
                })
                .unwrap_or_else(|| result.clone()),
            AssociatedFamilyResultExpr::Primitive { .. } => result.clone(),
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::NominalApp {
                origin: origin.clone(),
                visible_name: visible_name.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor: constructor.clone(),
                domain: domain.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                constraint,
                source_anchor,
                ..
            } => {
                let interface_args = interface_args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&interface_args);
                AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                    head: head.clone(),
                    interface_args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::Projection {
                interface,
                member,
                args,
                kind,
                constraint,
                source_anchor,
                ..
            } => {
                let args = args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&args);
                AssociatedFamilyResultExpr::Projection {
                    interface: interface.clone(),
                    member: member.clone(),
                    args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::ComputationHeadApp {
                head,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        Self::substitute_associated_family_result_expr_from_normal_bindings(
                            arg, bindings,
                        )
                    })
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
        }
    }

    fn substitute_associated_family_result_expr(
        result: &AssociatedFamilyResultExpr,
        bindings: &BTreeMap<String, CanonicalTypeExpr>,
    ) -> AssociatedFamilyResultExpr {
        match result {
            AssociatedFamilyResultExpr::Var {
                name,
                source_anchor,
                ..
            } => bindings
                .get(name)
                .cloned()
                .and_then(|canonical| {
                    associated_family_result_from_canonical(canonical, Span::default()).ok()
                })
                .unwrap_or_else(|| AssociatedFamilyResultExpr::Var {
                    name: name.clone(),
                    kind: Kind::Type,
                    constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    source_anchor: source_anchor.clone(),
                }),
            AssociatedFamilyResultExpr::Primitive { .. } => result.clone(),
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::NominalApp {
                origin: origin.clone(),
                visible_name: visible_name.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor: constructor.clone(),
                domain: domain.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                constraint,
                rigidity: _,
                source_anchor,
            } => {
                let interface_args = interface_args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&interface_args);
                AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                    head: head.clone(),
                    interface_args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::Projection {
                interface,
                member,
                args,
                kind,
                constraint,
                rigidity: _,
                source_anchor,
            } => {
                let args = args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect::<Vec<_>>();
                let rigidity = projection_rigidity_for_associated_family_args(&args);
                AssociatedFamilyResultExpr::Projection {
                    interface: interface.clone(),
                    member: member.clone(),
                    args,
                    kind: kind.clone(),
                    constraint: constraint.clone(),
                    rigidity,
                    source_anchor: source_anchor.clone(),
                }
            }
            AssociatedFamilyResultExpr::ComputationHeadApp {
                head,
                args,
                kind,
                constraint,
                source_anchor,
            } => AssociatedFamilyResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::substitute_associated_family_result_expr(arg, bindings))
                    .collect(),
                kind: kind.clone(),
                constraint: constraint.clone(),
                source_anchor: source_anchor.clone(),
            },
        }
    }

    fn associated_family_result_expr_constraint(
        expr: &AssociatedFamilyResultExpr,
    ) -> &AssociatedFamilyResultConstraint {
        match expr {
            AssociatedFamilyResultExpr::Primitive { constraint, .. }
            | AssociatedFamilyResultExpr::Var { constraint, .. }
            | AssociatedFamilyResultExpr::NominalApp { constraint, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { constraint, .. }
            | AssociatedFamilyResultExpr::AssociatedFamilyProjection { constraint, .. }
            | AssociatedFamilyResultExpr::Projection { constraint, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { constraint, .. } => constraint,
        }
    }

    fn associated_family_expr_conforms_to_constraint(
        expr: &AssociatedFamilyResultExpr,
        expected: &AssociatedFamilyResultConstraint,
    ) -> bool {
        match expected {
            AssociatedFamilyResultConstraint::Kind(expected_kind) => {
                matches!(
                    Self::associated_family_result_expr_constraint(expr),
                    AssociatedFamilyResultConstraint::Kind(actual_kind) if actual_kind == expected_kind
                ) || matches!(
                    Self::associated_family_result_expr_constraint(expr),
                    AssociatedFamilyResultConstraint::Domain(_) if expected_kind == &Kind::Type
                )
            }
            AssociatedFamilyResultConstraint::Domain(expected_domain) => match expr {
                AssociatedFamilyResultExpr::DomainConstructorApp {
                    domain, constraint, ..
                } => {
                    domain == expected_domain
                        && matches!(constraint, AssociatedFamilyResultConstraint::Domain(actual) if actual == expected_domain)
                }
                other => matches!(
                    Self::associated_family_result_expr_constraint(other),
                    AssociatedFamilyResultConstraint::Domain(actual) if actual == expected_domain
                ),
            },
        }
    }

    fn lower_associated_family_result_expr(
        &self,
        ty: &SurfaceType,
        expected_constraint: &AssociatedFamilyResultConstraint,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        let Some(expected_domain) = (match expected_constraint {
            AssociatedFamilyResultConstraint::Domain(domain) => Some(domain),
            AssociatedFamilyResultConstraint::Kind(_) => None,
        }) else {
            return self.lower_associated_family_unconstrained_result_expr(
                ty,
                var_constraints,
                span,
            );
        };
        match ty {
            SurfaceType::AssociatedFamilyProjection {
                interface,
                args,
                member,
                span: projection_span,
            } => self.lower_associated_family_projection_result_expr(
                interface,
                args,
                member,
                expected_constraint,
                var_constraints,
                *projection_span,
            ),
            SurfaceType::Name(name) => {
                if let Some((domain, constructor)) =
                    self.find_domain_constructor_cloned(expected_domain, name.as_ref())
                {
                    return self.lower_associated_family_domain_constructor_result(
                        &domain,
                        &constructor,
                        &[],
                        var_constraints,
                        span,
                    );
                }
                if let Some((domain, _)) = self.find_any_domain_constructor(name.as_ref()) {
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl binding>".to_string(),
                        reason: format!(
                            "marker constructor '{}' belongs to sealed domain '{}', not '{}'",
                            name, domain.exported_name, expected_domain.name
                        ),
                        span,
                    });
                }
                Ok(AssociatedFamilyResultExpr::Var {
                    name: name.to_string(),
                    kind: Kind::Type,
                    constraint: var_constraints
                        .get(name.as_ref())
                        .cloned()
                        .unwrap_or(AssociatedFamilyResultConstraint::Kind(Kind::Type)),
                    source_anchor: span_anchor(span, format!("associated family result {name}")),
                })
            }
            SurfaceType::Constructor { name, args } => {
                if let Some((domain, constructor)) =
                    self.find_domain_constructor_cloned(expected_domain, name.as_ref())
                {
                    return self.lower_associated_family_domain_constructor_result(
                        &domain,
                        &constructor,
                        args,
                        var_constraints,
                        span,
                    );
                }
                if let Some((domain, _)) = self.find_any_domain_constructor(name.as_ref()) {
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl binding>".to_string(),
                        reason: format!(
                            "marker constructor '{}' belongs to sealed domain '{}', not '{}'",
                            name, domain.exported_name, expected_domain.name
                        ),
                        span,
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                    .and_then(|canonical| associated_family_result_from_canonical(canonical, span))
            }
            _ => self
                .lower_surface_type_to_canonical(ty)
                .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                .and_then(|canonical| associated_family_result_from_canonical(canonical, span)),
        }
    }

    fn find_domain_constructor_cloned(
        &self,
        domain_id: &SealedDomainId,
        constructor_name: &str,
    ) -> Option<(SealedDomainSummary, DomainConstructorSummary)> {
        self.find_domain_constructor(domain_id, constructor_name)
            .map(|(domain, constructor)| (domain.clone(), constructor.clone()))
    }

    fn lower_associated_family_domain_constructor_result(
        &self,
        domain: &SealedDomainSummary,
        constructor: &DomainConstructorSummary,
        args: &[SurfaceType],
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        if constructor.fields.len() != args.len() {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<impl binding>".to_string(),
                reason: format!(
                    "marker constructor '{}' expects {} type arguments, found {}",
                    constructor.exported_name,
                    constructor.fields.len(),
                    args.len()
                ),
                span,
            });
        }
        let args = constructor
            .fields
            .iter()
            .zip(args.iter())
            .map(|(field, arg)| {
                if let Some(field_domain) = &field.domain_constraint {
                    self.lower_associated_family_result_expr(
                        arg,
                        &AssociatedFamilyResultConstraint::Domain(field_domain.clone()),
                        var_constraints,
                        span,
                    )
                } else {
                    self.lower_associated_family_unconstrained_result_expr(
                        arg,
                        var_constraints,
                        span,
                    )
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AssociatedFamilyResultExpr::DomainConstructorApp {
            constructor: constructor.id.clone(),
            domain: domain.id.clone(),
            args,
            kind: Kind::Type,
            constraint: AssociatedFamilyResultConstraint::Domain(domain.id.clone()),
            source_anchor: span_anchor(
                span,
                format!("associated family result {}", constructor.exported_name),
            ),
        })
    }

    fn lower_associated_family_unconstrained_result_expr(
        &self,
        ty: &SurfaceType,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        match ty {
            SurfaceType::AssociatedFamilyProjection {
                interface,
                args,
                member,
                span: projection_span,
            } => self.lower_associated_family_projection_result_expr(
                interface,
                args,
                member,
                &AssociatedFamilyResultConstraint::Kind(Kind::Type),
                var_constraints,
                *projection_span,
            ),
            SurfaceType::Name(name) => {
                if let Some(constraint) = var_constraints.get(name.as_ref()) {
                    Ok(AssociatedFamilyResultExpr::Var {
                        name: name.to_string(),
                        kind: Kind::Type,
                        constraint: constraint.clone(),
                        source_anchor: span_anchor(
                            span,
                            format!("associated family result {name}"),
                        ),
                    })
                } else {
                    self.lower_surface_type_to_canonical(ty)
                        .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                        .and_then(|canonical| {
                            associated_family_result_from_canonical(canonical, span)
                        })
                }
            }
            _ => self
                .lower_surface_type_to_canonical(ty)
                .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))
                .and_then(|canonical| associated_family_result_from_canonical(canonical, span)),
        }
    }

    fn associated_family_constraint_for_domain(
        domain: Option<&SealedDomainId>,
    ) -> AssociatedFamilyResultConstraint {
        domain.map_or(
            AssociatedFamilyResultConstraint::Kind(Kind::Type),
            |domain| AssociatedFamilyResultConstraint::Domain(domain.clone()),
        )
    }

    fn lower_associated_family_pattern(
        &self,
        ty: &SurfaceType,
        expected_domain: Option<&SealedDomainId>,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyPattern, TypeEnvError> {
        let constraint = Self::associated_family_constraint_for_domain(expected_domain);
        match ty {
            SurfaceType::Name(name) => {
                if let Some(domain_id) = expected_domain {
                    if let Some((domain, constructor)) =
                        self.find_domain_constructor_cloned(domain_id, name.as_ref())
                    {
                        return self.lower_associated_family_domain_constructor_pattern(
                            &domain,
                            &constructor,
                            &[],
                            var_constraints,
                            span,
                        );
                    }
                    if let Some((domain, _)) = self.find_any_domain_constructor(name.as_ref()) {
                        return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                            family: "<impl head>".to_string(),
                            reason: format!(
                                "marker constructor '{}' belongs to sealed domain '{}', not '{}'",
                                name, domain.exported_name, domain_id.name
                            ),
                            span,
                        });
                    }
                }
                if let Some(var_constraint) = var_constraints.get(name.as_ref()) {
                    return Ok(AssociatedFamilyPattern::Var {
                        name: name.to_string(),
                        constraint: var_constraint.clone(),
                        source_anchor: span_anchor(
                            span,
                            format!("associated family pattern {name}"),
                        ),
                    });
                }
                let canonical = self
                    .lower_surface_type_to_canonical(ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))?;
                Self::associated_family_pattern_from_canonical(
                    canonical,
                    &constraint,
                    var_constraints,
                    span,
                )
            }
            SurfaceType::Constructor { name, args } => {
                if let Some(domain_id) = expected_domain {
                    if let Some((domain, constructor)) =
                        self.find_domain_constructor_cloned(domain_id, name.as_ref())
                    {
                        return self.lower_associated_family_domain_constructor_pattern(
                            &domain,
                            &constructor,
                            args,
                            var_constraints,
                            span,
                        );
                    }
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl head>".to_string(),
                        reason: format!(
                            "unknown marker constructor '{}' for sealed domain '{}'",
                            name, domain_id.name
                        ),
                        span,
                    });
                }
                let canonical = self
                    .lower_surface_type_to_canonical(ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))?;
                Self::associated_family_pattern_from_canonical(
                    canonical,
                    &constraint,
                    var_constraints,
                    span,
                )
            }
            SurfaceType::List(item) => {
                if expected_domain.is_some() {
                    return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                        family: "<impl head>".to_string(),
                        reason: "list pattern requires an unconstrained Type interface parameter"
                            .to_string(),
                        span,
                    });
                }
                let list_ty = SurfaceType::Constructor {
                    name: "List".into(),
                    args: vec![item.as_ref().clone()],
                };
                let canonical = self
                    .lower_surface_type_to_canonical(&list_ty)
                    .map_err(|err| TypeEnvError::InvalidDefinition(format!("{err}"), span))?;
                Self::associated_family_pattern_from_canonical(
                    canonical,
                    &constraint,
                    var_constraints,
                    span,
                )
            }
            _ => Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<impl head>".to_string(),
                reason: format!(
                    "unsupported associated-family impl-head pattern '{}'",
                    surface_projection_base_spelling(ty)
                ),
                span,
            }),
        }
    }

    fn associated_family_pattern_from_canonical(
        canonical: CanonicalTypeExpr,
        constraint: &AssociatedFamilyResultConstraint,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyPattern, TypeEnvError> {
        match canonical {
            CanonicalTypeExpr::Primitive(name) => Ok(AssociatedFamilyPattern::Primitive {
                name: name.clone(),
                constraint: constraint.clone(),
                source_anchor: span_anchor(
                    span,
                    format!("associated family primitive pattern {name}"),
                ),
            }),
            CanonicalTypeExpr::Var(name) => Ok(AssociatedFamilyPattern::Var {
                name: name.clone(),
                constraint: var_constraints
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_else(|| constraint.clone()),
                source_anchor: span_anchor(span, format!("associated family pattern {name}")),
            }),
            CanonicalTypeExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind: _,
            } => Ok(AssociatedFamilyPattern::NominalApp {
                origin,
                visible_name: visible_name.clone(),
                args: args
                    .into_iter()
                    .map(|arg| {
                        Self::associated_family_pattern_from_canonical(
                            arg,
                            constraint,
                            var_constraints,
                            span,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                constraint: constraint.clone(),
                source_anchor: span_anchor(
                    span,
                    format!("associated family pattern {visible_name}"),
                ),
            }),
            CanonicalTypeExpr::Projection { .. }
            | CanonicalTypeExpr::ComputationHeadApp { .. }
            | CanonicalTypeExpr::PromotedDataConstructorApp(_) => {
                Ok(AssociatedFamilyPattern::Wildcard {
                    constraint: constraint.clone(),
                    source_anchor: span_anchor(span, "associated family unsupported pattern"),
                })
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor-variable application '{}' cannot be lowered to an associated-family pattern until TASK-907 tracks constructor variables",
                    app.constructor.name
                ),
                span,
            )),
        }
    }

    fn lower_associated_family_domain_constructor_pattern(
        &self,
        domain: &SealedDomainSummary,
        constructor: &DomainConstructorSummary,
        args: &[SurfaceType],
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyPattern, TypeEnvError> {
        if constructor.fields.len() != args.len() {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: "<impl head>".to_string(),
                reason: format!(
                    "marker constructor '{}' expects {} type arguments, found {}",
                    constructor.exported_name,
                    constructor.fields.len(),
                    args.len()
                ),
                span,
            });
        }
        let fields = constructor
            .fields
            .iter()
            .zip(args.iter())
            .map(|(field, arg)| {
                self.lower_associated_family_pattern(
                    arg,
                    field.domain_constraint.as_ref(),
                    var_constraints,
                    span,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AssociatedFamilyPattern::DomainConstructor {
            constructor: Box::new(constructor.id.clone()),
            domain: Box::new(domain.id.clone()),
            fields,
            constraint: AssociatedFamilyResultConstraint::Domain(domain.id.clone()),
            source_anchor: span_anchor(
                span,
                format!("associated family pattern {}", constructor.exported_name),
            ),
        })
    }

    fn associated_family_pattern_spines_overlap(
        left: &[AssociatedFamilyPattern],
        right: &[AssociatedFamilyPattern],
    ) -> bool {
        left.len() == right.len()
            && left
                .iter()
                .zip(right.iter())
                .all(|(left, right)| Self::associated_family_patterns_overlap(left, right))
    }

    fn associated_family_patterns_overlap(
        left: &AssociatedFamilyPattern,
        right: &AssociatedFamilyPattern,
    ) -> bool {
        match (left, right) {
            (
                AssociatedFamilyPattern::DomainConstructor {
                    constructor: left_constructor,
                    domain: left_domain,
                    fields: left_fields,
                    ..
                },
                AssociatedFamilyPattern::DomainConstructor {
                    constructor: right_constructor,
                    domain: right_domain,
                    fields: right_fields,
                    ..
                },
            ) => {
                left_constructor == right_constructor
                    && left_domain == right_domain
                    && Self::associated_family_pattern_spines_overlap(left_fields, right_fields)
            }
            (
                AssociatedFamilyPattern::NominalApp {
                    origin: left_origin,
                    visible_name: left_name,
                    args: left_args,
                    ..
                },
                AssociatedFamilyPattern::NominalApp {
                    origin: right_origin,
                    visible_name: right_name,
                    args: right_args,
                    ..
                },
            ) => {
                left_origin == right_origin
                    && left_name == right_name
                    && Self::associated_family_pattern_spines_overlap(left_args, right_args)
            }
            (
                AssociatedFamilyPattern::Primitive {
                    name: left_name, ..
                },
                AssociatedFamilyPattern::Primitive {
                    name: right_name, ..
                },
            ) => left_name == right_name,
            (AssociatedFamilyPattern::Primitive { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::Primitive { .. })
            | (
                AssociatedFamilyPattern::Primitive { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::Primitive { .. },
            ) => true,
            (AssociatedFamilyPattern::Primitive { .. }, _)
            | (_, AssociatedFamilyPattern::Primitive { .. }) => false,
            (
                AssociatedFamilyPattern::DomainConstructor { .. },
                AssociatedFamilyPattern::NominalApp { .. },
            )
            | (
                AssociatedFamilyPattern::NominalApp { .. },
                AssociatedFamilyPattern::DomainConstructor { .. },
            ) => false,
            (
                AssociatedFamilyPattern::DomainConstructor { .. },
                AssociatedFamilyPattern::Var { .. },
            )
            | (
                AssociatedFamilyPattern::Var { .. },
                AssociatedFamilyPattern::DomainConstructor { .. },
            )
            | (
                AssociatedFamilyPattern::DomainConstructor { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::DomainConstructor { .. },
            )
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::Wildcard { .. })
            | (AssociatedFamilyPattern::Wildcard { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::NominalApp { .. }, AssociatedFamilyPattern::Var { .. })
            | (AssociatedFamilyPattern::Var { .. }, AssociatedFamilyPattern::NominalApp { .. })
            | (
                AssociatedFamilyPattern::NominalApp { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::NominalApp { .. },
            )
            | (
                AssociatedFamilyPattern::Wildcard { .. },
                AssociatedFamilyPattern::Wildcard { .. },
            ) => true,
        }
    }

    /// Enter a workflow context at the given effect level.
    ///
    /// All `Expr::FnDef` nodes type-checked in this environment (or any child
    /// derived from it via `extend()`) will be assigned `Type::Fun(…, effect)`
    /// instead of the pure `Type::Fn(…)`.
    pub fn set_workflow_effect(&mut self, effect: ash_core::Effect) {
        self.workflow_effect = Some(effect);
    }

    /// Create a new type environment with builtin types registered
    #[must_use]
    pub fn with_builtin_types() -> Self {
        let mut env = Self::new();
        env.add_builtin_types();
        env
    }

    /// Pre-declare a type name by inserting a placeholder into `ast_types`.
    /// This allows `resolve_type` to find the name during sibling type registration.
    /// The placeholder will be upgraded by a subsequent `register_type` call.
    pub fn declare_type_name(&mut self, name: &str) {
        let placeholder = TypeDef {
            name: name.to_owned(),
            params: vec![],
            body: TypeBody::Struct(vec![]), // minimal placeholder: empty struct
            visibility: ash_core::ast::Visibility::Public,
            builtin: false,
        };
        self.ast_types.entry(name.to_owned()).or_insert(placeholder);
        self.type_declaration_states
            .entry(name.to_owned())
            .or_insert(TypeDeclarationState::Placeholder);
    }

    fn is_placeholder_name(&self, name: &str) -> bool {
        matches!(
            self.type_declaration_states.get(name),
            Some(TypeDeclarationState::Placeholder)
        )
    }

    fn is_identity_only_name(&self, name: &str) -> bool {
        matches!(
            self.type_declaration_states.get(name),
            Some(TypeDeclarationState::IdentityOnly)
        )
    }

    /// Register a type definition without exposing its constructors or
    /// representation symbols.
    pub fn register_type_identity(&mut self, def: &TypeDef) -> Result<(), TypeEnvError> {
        let type_name = def.name.clone();

        if self.ast_types.contains_key(&type_name) {
            // Allow upgrading an explicit placeholder, or replacing an
            // identity-only summary declaration with the same imported fallback
            // definition.
            if !self.is_placeholder_name(&type_name) && !self.is_identity_only_name(&type_name) {
                return Err(TypeEnvError::DuplicateType(type_name, Span::default()));
            }
            // Placeholder/identity-only entry will be replaced below.
        }

        // Convert to internal TypeInfo for type checking
        let type_info = convert_type_def(def, self).map_err(|e| {
            TypeEnvError::InvalidDefinition(format!("type '{}': {e}", def.name), Span::default())
        })?;

        self.ast_types.insert(type_name.clone(), def.clone());
        self.type_info.insert(type_name, type_info);
        self.type_declaration_states
            .insert(def.name.clone(), TypeDeclarationState::Full);
        Ok(())
    }

    /// Expose constructors/representation for a previously-registered type.
    pub fn expose_type_representation(&mut self, name: &str) -> Result<(), TypeEnvError> {
        let Some(type_info) = self.type_info.get(name).cloned() else {
            return Err(TypeEnvError::TypeNotFound(
                name.to_string(),
                Span::default(),
            ));
        };

        match type_info {
            TypeInfo::Enum { variants, .. } => {
                for (index, variant) in variants.iter().enumerate() {
                    self.constructors
                        .insert(variant.name.clone(), (name.to_string(), index));
                }
            }
            TypeInfo::Struct { fields, .. } if matches!(fields.as_slice(), [(field_name, _)] if field_name == "__alias_target") =>
            {
                self.transparent_aliases.insert(name.to_string());
            }
            TypeInfo::Struct { .. } => {}
        }

        Ok(())
    }

    #[must_use]
    pub fn transparent_alias_target(&self, name: &QualifiedName, args: &[Type]) -> Option<Type> {
        if !self.transparent_aliases.contains(name.name.as_str()) {
            return None;
        }

        match self.unfold_constructor(name, args).ok()? {
            UnfoldedBody::Struct(fields) => match fields.as_slice() {
                [(field_name, target)] if field_name == "__alias_target" => Some(target.clone()),
                _ => None,
            },
            UnfoldedBody::Enum(_) => None,
        }
    }

    /// Register a type definition and its constructors from AST TypeDef
    pub fn register_type(&mut self, def: &TypeDef) -> Result<(), TypeEnvError> {
        self.register_type_identity(def)?;
        self.type_alias_identities
            .entry(def.name.clone())
            .or_insert_with(|| fallback_canonical_type_decl_id(&def.name));
        if let Some(identity) = self.type_alias_identities.get(&def.name).cloned() {
            self.canonical_type_names
                .entry(identity)
                .or_insert_with(|| def.name.clone());
        }
        self.expose_type_representation(&def.name)
    }

    fn existing_summary_contract_conflicts(
        &self,
        visible_name: &str,
        existing: &TypeDef,
        summary: &TypeDeclSummary,
    ) -> bool {
        if existing.params != summary.params || existing.visibility != summary.visibility {
            return true;
        }

        match self.type_declaration_states.get(visible_name) {
            Some(TypeDeclarationState::Full) => match &summary.representation {
                TypeRepresentationSummary::Exposed(body) => existing.body != *body,
                TypeRepresentationSummary::Opaque { builtin: true } => !existing.builtin,
                TypeRepresentationSummary::Opaque { builtin: false } => true,
            },
            Some(TypeDeclarationState::IdentityOnly) => false,
            Some(TypeDeclarationState::Placeholder) | None => false,
        }
    }

    fn declare_summary_type_identity(
        &mut self,
        summary: &TypeDeclSummary,
    ) -> Result<(), TypeEnvError> {
        let visible_name = summary.exported_name.clone();
        let conflicting_existing_summary = self
            .canonical_type_names
            .get(&summary.id)
            .cloned()
            .is_some_and(|existing_visible_name| {
                existing_visible_name != visible_name
                    && self
                        .ast_types
                        .get(&existing_visible_name)
                        .is_some_and(|existing| {
                            self.existing_summary_contract_conflicts(
                                &existing_visible_name,
                                existing,
                                summary,
                            )
                        })
            });
        if conflicting_existing_summary {
            return Err(TypeEnvError::InvalidDefinition(
                conflicting_summary_contract_diagnostic(&visible_name),
                Span::default(),
            ));
        }
        let fallback_compatible_builtin_identity = self
            .type_alias_identities
            .get(&visible_name)
            .is_some_and(|existing| existing == &fallback_canonical_type_decl_id(&visible_name))
            && self.ast_types.get(&visible_name).is_some_and(|existing| {
                (is_builtin_prelude_ordinary_type_compatibility_name(&visible_name)
                    && !self.existing_summary_contract_conflicts(&visible_name, existing, summary))
                    || (existing.builtin
                        && matches!(
                            summary.representation,
                            TypeRepresentationSummary::Opaque { .. }
                        ))
            });
        match self.type_alias_identities.get(&visible_name) {
            Some(existing) if existing != &summary.id && !fallback_compatible_builtin_identity => {
                return Err(TypeEnvError::InvalidDefinition(
                    duplicate_summary_identity_diagnostic(&visible_name, existing, summary),
                    Span::default(),
                ));
            }
            _ => {}
        }
        if let Some(existing) = self.ast_types.get(&visible_name) {
            let existing_identity = self.type_alias_identities.get(&visible_name);
            if !self.is_placeholder_name(&visible_name) && existing_identity != Some(&summary.id) {
                if fallback_compatible_builtin_identity {
                    self.type_alias_identities
                        .insert(visible_name.clone(), summary.id.clone());
                    self.canonical_type_names
                        .entry(summary.id.clone())
                        .or_insert(visible_name);
                    return Ok(());
                }
                if matches!(
                    (&summary.representation, existing.builtin),
                    (TypeRepresentationSummary::Opaque { builtin: true }, true)
                ) {
                    self.type_alias_identities
                        .insert(visible_name.clone(), summary.id.clone());
                    self.canonical_type_names
                        .entry(summary.id.clone())
                        .or_insert(visible_name);
                    return Ok(());
                }
                if (existing_identity.is_none()
                    || existing_identity == Some(&fallback_canonical_type_decl_id(&visible_name)))
                    && is_builtin_prelude_ordinary_type_compatibility_name(&visible_name)
                    && !self.existing_summary_contract_conflicts(&visible_name, existing, summary)
                {
                    self.type_alias_identities
                        .insert(visible_name.clone(), summary.id.clone());
                    self.canonical_type_names
                        .entry(summary.id.clone())
                        .or_insert(visible_name);
                    return Ok(());
                }
                if let Some(existing_identity) = existing_identity {
                    return Err(TypeEnvError::InvalidDefinition(
                        duplicate_summary_identity_diagnostic(
                            &visible_name,
                            existing_identity,
                            summary,
                        ),
                        Span::default(),
                    ));
                }
                return Err(TypeEnvError::DuplicateType(visible_name, Span::default()));
            }
            if existing_identity == Some(&summary.id)
                && self.existing_summary_contract_conflicts(&visible_name, existing, summary)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    conflicting_summary_contract_diagnostic(&visible_name),
                    Span::default(),
                ));
            }
        }

        let identity_def = TypeDef {
            name: visible_name.clone(),
            params: summary.params.clone(),
            body: TypeBody::Struct(vec![]),
            visibility: summary.visibility,
            builtin: matches!(
                summary.representation,
                TypeRepresentationSummary::Opaque { builtin: true }
            ),
        };
        self.ast_types.insert(visible_name.clone(), identity_def);
        let type_info = TypeInfo::Struct {
            name: visible_name.clone(),
            params: summary.params.iter().map(|_| TypeVar::fresh()).collect(),
            fields: vec![],
        };
        self.type_info.insert(visible_name.clone(), type_info);
        self.type_declaration_states
            .insert(visible_name.clone(), TypeDeclarationState::IdentityOnly);
        self.type_alias_identities
            .insert(visible_name.clone(), summary.id.clone());
        self.canonical_type_names
            .entry(summary.id.clone())
            .or_insert(visible_name);
        Ok(())
    }

    fn expose_summary_type_representation(
        &mut self,
        ty: &TypeDeclSummary,
        constructors: &[ConstructorSummary],
    ) -> Result<(), TypeEnvError> {
        let visible_name = ty.exported_name.as_str();
        let Some(type_info) = self.type_info.get(visible_name).cloned() else {
            return Err(TypeEnvError::TypeNotFound(
                visible_name.to_string(),
                Span::default(),
            ));
        };

        match type_info {
            TypeInfo::Enum { variants, .. } => {
                let matching_constructors = constructors
                    .iter()
                    .filter(|constructor| constructor.parent == ty.id)
                    .collect::<Vec<_>>();
                if !matching_constructors.is_empty() {
                    for constructor in &matching_constructors {
                        let Some((index, _)) = variants
                            .iter()
                            .enumerate()
                            .find(|(_, variant)| variant.name == constructor.id.name)
                        else {
                            return Err(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor summary '{}' does not match any exposed variant on type '{}'",
                                    constructor.exported_name, visible_name
                                ),
                                Span::default(),
                            ));
                        };
                        match self.constructors.get(&constructor.exported_name) {
                            Some((existing_type, existing_index))
                                if existing_type != visible_name || *existing_index != index =>
                            {
                                return Err(TypeEnvError::InvalidDefinition(
                                    format!(
                                        "duplicate exported constructor summary '{}' conflicts with an existing constructor binding",
                                        constructor.exported_name
                                    ),
                                    Span::default(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                for constructor in matching_constructors {
                    let Some((index, _)) = variants
                        .iter()
                        .enumerate()
                        .find(|(_, variant)| variant.name == constructor.id.name)
                    else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "constructor summary '{}' does not match any exposed variant on type '{}'",
                                constructor.exported_name, visible_name
                            ),
                            Span::default(),
                        ));
                    };
                    match self.constructors.get(&constructor.exported_name) {
                        Some((existing_type, existing_index))
                            if existing_type != visible_name || *existing_index != index =>
                        {
                            return Err(TypeEnvError::InvalidDefinition(
                                format!(
                                    "duplicate exported constructor summary '{}' conflicts with an existing constructor binding",
                                    constructor.exported_name
                                ),
                                Span::default(),
                            ));
                        }
                        _ => {}
                    }
                    self.constructors.insert(
                        constructor.exported_name.clone(),
                        (visible_name.to_string(), index),
                    );
                }
            }
            TypeInfo::Struct { fields, .. } if matches!(fields.as_slice(), [(field_name, _)] if field_name == "__alias_target") =>
            {
                self.transparent_aliases.insert(visible_name.to_string());
            }
            TypeInfo::Struct { .. } => {
                if constructors
                    .iter()
                    .any(|constructor| constructor.parent == ty.id)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "constructor summaries for '{}' require an exposed enum body",
                            visible_name
                        ),
                        Span::default(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Register all visible ordinary type identities from a module semantic summary first,
    /// then validate/expose public representations in a second pass.
    pub fn register_module_semantic_summary(
        &mut self,
        summary: &ModuleSemanticSummary,
    ) -> Result<(), TypeEnvError> {
        self.register_module_semantic_summaries(std::slice::from_ref(summary))
    }

    /// Batch-register imported semantic summaries.
    ///
    /// The batch path declares all imported identities and public computation
    /// heads before equation revalidation so cross-summary public reductions and
    /// dependency-closure helper heads are normalizer-available atomically.
    pub fn register_module_semantic_summaries(
        &mut self,
        summaries: &[ModuleSemanticSummary],
    ) -> Result<(), TypeEnvError> {
        for summary in summaries {
            summary
                .validate_summary_version_contract()
                .map_err(summary_version_contract_error)?;
        }

        let mut staged = self.clone();
        for summary in summaries {
            validate_summary_visibility_and_duplicates(summary)?;
        }
        for summary in summaries {
            for ty in &summary.exported_types {
                staged.declare_summary_type_identity(ty)?;
            }
            for interface in &summary.interface_identities {
                staged.register_interface_identity_summary_imported(interface)?;
            }
            for member in &summary.associated_member_identities {
                staged.register_associated_member_identity_summary_imported(member)?;
            }
            for domain in &summary.exported_sealed_domains {
                staged.declare_sealed_domain_identity(domain)?;
            }
            for data_kind in &summary.exported_promoted_data_kinds {
                staged.declare_promoted_data_kind_identity(data_kind)?;
            }
        }
        for summary in summaries {
            for type_fn in &summary.exported_type_functions {
                staged.declare_imported_type_function_summary(type_fn)?;
            }
        }
        let hidden_associated_family_heads = hidden_imported_associated_family_heads(summaries);
        for summary in summaries {
            for family in &summary.exported_associated_families {
                staged.declare_imported_associated_family_summary(
                    family,
                    !hidden_associated_family_heads.contains(&family.head),
                )?;
            }
            for predicate in &summary.exported_proposition_predicates {
                staged.register_proposition_predicate_summary(predicate)?;
            }
        }
        for summary in summaries {
            staged.register_module_semantic_summary_representations_and_domains(summary)?;
        }
        for summary in summaries {
            for data_kind in &summary.exported_promoted_data_kinds {
                staged.validate_and_register_promoted_data_kind(data_kind)?;
            }
        }
        for summary in summaries {
            for type_fn in &summary.exported_type_functions {
                staged.validate_imported_type_function_summary(type_fn)?;
            }
        }
        for summary in summaries {
            for family in &summary.exported_associated_families {
                staged.validate_and_register_imported_associated_family_summary(family)?;
            }
            for fact in &summary.exported_proposition_facts {
                staged.validate_and_register_imported_proposition_fact(summary, fact)?;
            }
        }
        *self = staged;
        Ok(())
    }

    /// Batch-register imported semantic summaries and atomically discharge all
    /// required proposition facts they introduce.
    pub fn register_module_semantic_summaries_and_discharge_required_propositions(
        &mut self,
        summaries: &[ModuleSemanticSummary],
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        let mut staged = self.clone();
        staged.register_module_semantic_summaries(summaries)?;
        let outcomes = staged.discharge_required_proposition_obligations()?;
        *self = staged;
        Ok(outcomes)
    }

    fn validate_and_register_imported_proposition_fact(
        &mut self,
        summary: &ModuleSemanticSummary,
        fact: &PropositionFactSummary,
    ) -> Result<(), TypeEnvError> {
        let predicate_dependencies = self.validate_public_proposition_dependencies(
            "imported proposition summary fact",
            &fact.proposition,
            anchor_span(&fact.source_anchor),
        )?;
        for dependency in &fact.predicate_dependencies {
            let Some(info) = self.proposition_predicate_by_id(dependency) else {
                return Err(TypeEnvError::UnknownPropositionPredicate {
                    name: dependency.name.to_string(),
                    span: anchor_span(&fact.source_anchor),
                });
            };
            if info.summary.visibility != ash_core::ast::Visibility::Public {
                return Err(private_proposition_dependency_error(
                    "imported proposition summary fact",
                    "proposition predicate",
                    info.summary.exported_name.as_ref(),
                    anchor_span(&fact.source_anchor),
                ));
            }
        }
        for dependency in &predicate_dependencies {
            if !fact.predicate_dependencies.contains(dependency) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "imported proposition summary fact omits predicate dependency '{}' from dependency metadata",
                        dependency.name
                    ),
                    anchor_span(&fact.source_anchor),
                ));
            }
        }

        let outcome = Some(
            self.solve_proposition(&fact.proposition, Some(fact.source_anchor.clone()))
                .map_err(proposition_revalidation_error)?,
        );
        self.push_proposition_fact(
            fact.role,
            fact.proposition.clone(),
            fact.source_anchor.clone(),
            PropositionCheckingSite::new(
                0x8790_0000u64 + self.proposition_obligations.len() as u64,
                PropositionCheckingSiteKind::Synthetic,
                Some(format!(
                    "imported proposition fact from {}",
                    summary.module.path.join("::")
                )),
            ),
            outcome,
        );
        Ok(())
    }

    fn register_module_semantic_summary_representations_and_domains(
        &mut self,
        summary: &ModuleSemanticSummary,
    ) -> Result<(), TypeEnvError> {
        for ty in &summary.exported_types {
            if ty.representation_exposure != RepresentationExposure::Exposed {
                continue;
            }
            let TypeRepresentationSummary::Exposed(body) = &ty.representation else {
                continue;
            };
            let def = TypeDef {
                name: ty.exported_name.clone(),
                params: ty.params.clone(),
                body: body.clone(),
                visibility: ty.visibility,
                builtin: false,
            };
            let type_info = convert_type_def(&def, self).map_err(|e| {
                TypeEnvError::InvalidDefinition(
                    format!("type '{}': {e}", def.name),
                    Span::default(),
                )
            })?;
            self.ast_types.insert(def.name.clone(), def.clone());
            self.type_info.insert(def.name.clone(), type_info);
            self.type_declaration_states
                .insert(def.name.clone(), TypeDeclarationState::Full);
            self.expose_summary_type_representation(ty, &summary.exported_constructors)?;
        }

        for domain in &summary.exported_sealed_domains {
            self.validate_and_register_sealed_domain(domain)?;
        }
        Ok(())
    }

    fn declare_imported_type_function_summary(
        &mut self,
        summary: &TypeFunctionSummary,
    ) -> Result<(), TypeEnvError> {
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-public type-function summary '{}' is not valid public metadata",
                    summary.exported_name
                ),
                Span::default(),
            ));
        }
        if let Some(existing) = self.local_type_functions.get(&summary.head) {
            let incoming = imported_type_function_def(summary);
            if existing != &incoming {
                return Err(TypeEnvError::ImportOrderConflict {
                    family: "type-function summary".to_string(),
                    name: summary.exported_name.clone(),
                    span: summary.equations.first().map_or_else(
                        || anchor_span(&summary.source_anchors.definition),
                        |equation| anchor_span(&equation.source_anchor),
                    ),
                });
            }
            return Ok(());
        }
        self.local_type_functions
            .insert(summary.head.clone(), imported_type_function_def(summary));
        Ok(())
    }

    fn validate_imported_type_function_summary(
        &self,
        summary: &TypeFunctionSummary,
    ) -> Result<(), TypeEnvError> {
        if summary.export_mode != TypeFunctionExportMode::TransparentEquations {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{}' has unsupported export mode",
                    summary.exported_name
                ),
                Span::default(),
            ));
        }
        if summary.revalidation_metadata.spec_version != SummaryVersion::SPEC062_TYPE_COMPUTATION_V3
            || !summary.revalidation_metadata.structural_recursion_checked
            || !summary.revalidation_metadata.kind_and_domain_checked
            || !summary.revalidation_metadata.coverage_and_overlap_checked
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{}' lacks required SPEC-062 revalidation metadata",
                    summary.exported_name
                ),
                Span::default(),
            ));
        }
        let def = self
            .local_type_functions
            .get(&summary.head)
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' head was not declared before validation",
                        summary.exported_name
                    ),
                    Span::default(),
                )
            })?;
        for param in &def.params {
            if param.kind != Kind::Type {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' parameter '{}' has non-Type kind",
                        def.name, param.name
                    ),
                    Span::default(),
                ));
            }
            self.validate_imported_type_function_signature_type(&def.name, &param.ty, "parameter")?;
            if let Some(domain) = &param.domain_constraint
                && self.lookup_sealed_domain_by_id(domain).is_none()
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' parameter '{}' references unknown sealed domain '{}'",
                        def.name, param.name, domain.name
                    ),
                    Span::default(),
                ));
            }
        }
        if def.return_kind != Kind::Type {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{}' return has non-Type kind",
                    def.name
                ),
                Span::default(),
            ));
        }
        self.validate_imported_type_function_signature_type(&def.name, &def.return_type, "return")?;
        for equation in &def.equations {
            if equation.head != def.head || equation.patterns.len() != def.params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type-function summary '{}' equation arity or head mismatch",
                        def.name
                    ),
                    Span::default(),
                ));
            }
            let mut vars = HashMap::new();
            for (pattern, param) in equation.patterns.iter().zip(&def.params) {
                self.validate_imported_type_function_pattern(
                    pattern,
                    &constraint_for_param(param),
                    &mut vars,
                )?;
            }
            let actual = self.validate_imported_type_function_result(&equation.result, &vars)?;
            self.validate_imported_result_constraint_value(
                &actual,
                &def.result_constraint,
                Span::default(),
            )?;
        }
        self.validate_type_function_pattern_coverage(
            &def.name,
            &def.params,
            &def.equations,
            Span::default(),
        )?;
        self.validate_type_function_structural_recursion(
            &def.name,
            &def.head,
            &def.params,
            def.decreases.as_deref(),
            &def.equations,
            Span::default(),
        )?;
        self.validate_public_type_function_export_closure(def, Span::default())
    }

    fn associated_family_result_constraint_from_summary(
        &self,
        family: &AssociatedFamilySummary,
    ) -> Result<AssociatedFamilyResultConstraint, TypeEnvError> {
        match &family.result_domain {
            CanonicalTypeExpr::Primitive(name) if name == "Type" => {
                Ok(AssociatedFamilyResultConstraint::Kind(Kind::Type))
            }
            CanonicalTypeExpr::Var(name) => self
                .sealed_domain_summaries
                .values()
                .find(|domain| domain.id.name == *name || domain.exported_name == *name)
                .map(|domain| AssociatedFamilyResultConstraint::Domain(domain.id.clone()))
                .ok_or_else(|| TypeEnvError::WrongAssociatedFamilyResultDomain {
                    family: family.visible_name.clone(),
                    reason: format!("unknown associated-family result domain '{name}'"),
                    span: anchor_span(&family.source_anchor),
                }),
            other => Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.visible_name.clone(),
                reason: format!("unsupported associated-family result domain {other:?}"),
                span: anchor_span(&family.source_anchor),
            }),
        }
    }

    fn declare_imported_associated_family_summary(
        &mut self,
        family: &AssociatedFamilySummary,
        source_visible: bool,
    ) -> Result<(), TypeEnvError> {
        if family.head.interface != family.interface_identity
            || family.head.member != family.member_identity
            || family.member_identity.interface != family.interface_identity
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has inconsistent interface/member identities",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        let first_scheme = family.schemes.first().ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' must contain at least one scheme",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            )
        })?;
        let result_domain = self.associated_family_result_constraint_from_summary(family)?;
        let interface_params = first_scheme
            .params
            .iter()
            .map(|param| AssociatedFamilyInterfaceParamInfo {
                name: param.name.clone(),
                domain_constraint: param.domain_constraint.clone(),
            })
            .collect::<Vec<_>>();

        self.known_interface_identities
            .insert(family.interface_identity.clone());
        self.canonical_interface_names.insert(
            family.interface_identity.clone(),
            family.interface_identity.name.to_string(),
        );
        self.local_interface_arities
            .entry(family.interface_identity.clone())
            .or_insert(interface_params.len());
        self.known_associated_member_identities
            .insert(family.member_identity.clone());

        let declaration = AssociatedFamilyDeclarationInfo {
            defining_module: family.interface_identity.module.clone(),
            result_domain,
            decreases: family
                .revalidation_metadata
                .decreases
                .first()
                .map(|decreases| decreases.parameter.clone()),
            interface_params,
            head: family.head.clone(),
        };
        if let Some(existing) = self.associated_family_declarations.get(&family.head) {
            if existing != &declaration {
                return Err(TypeEnvError::ImportOrderConflict {
                    family: "associated-family summary".to_string(),
                    name: family.visible_name.clone(),
                    span: anchor_span(&family.source_anchor),
                });
            }
        } else {
            self.associated_family_declarations
                .insert(family.head.clone(), declaration);
        }

        if source_visible {
            self.interface_identity_aliases.insert(
                family.interface_identity.name.to_string(),
                family.interface_identity.clone(),
            );
            self.interface_identity_alias_is_imported
                .insert(family.interface_identity.name.to_string(), true);
            self.associated_member_identity_aliases.insert(
                (
                    family.interface_identity.name.to_string(),
                    family.visible_name.clone(),
                ),
                family.member_identity.clone(),
            );
            self.associated_member_identity_alias_is_imported.insert(
                (
                    family.interface_identity.name.to_string(),
                    family.visible_name.clone(),
                ),
                true,
            );
            self.associated_family_name_index.insert(
                (
                    family.interface_identity.name.to_string(),
                    family.visible_name.clone(),
                ),
                family.head.clone(),
            );
        }
        Ok(())
    }

    fn validate_and_register_imported_associated_family_summary(
        &mut self,
        family: &AssociatedFamilySummary,
    ) -> Result<(), TypeEnvError> {
        if family.export_mode != AssociatedFamilyExportMode::TransparentEquations
            || family.revalidation_metadata.spec_version
                != SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4
            || !family.revalidation_metadata.kind_and_domain_checked
            || !family.revalidation_metadata.coverage_and_overlap_checked
            || !family.revalidation_metadata.coherence_checked
            || !family.revalidation_metadata.recursion_checked
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' lacks required SPEC-063 revalidation metadata",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        if family.result_kind != Kind::Type {
            return Err(TypeEnvError::WrongAssociatedFamilyResultKind {
                family: family.visible_name.clone(),
                expected: format!("{:?}", Kind::Type),
                found: format!("{:?}", family.result_kind),
                span: anchor_span(&family.source_anchor),
            });
        }
        if !family
            .dependency_closure
            .closure_metadata
            .public_closure_checked
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has unchecked dependency closure",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        let metadata = &family.dependency_closure.closure_metadata;
        if metadata.public_ordinary_type_count != family.dependency_closure.ordinary_types.len()
            || metadata.public_sealed_domain_count != family.dependency_closure.sealed_domains.len()
            || metadata.public_domain_constructor_count
                != family.dependency_closure.domain_constructors.len()
            || metadata.public_type_function_count != family.dependency_closure.type_functions.len()
            || metadata.public_projection_count
                != family.dependency_closure.associated_projections.len()
            || metadata.public_associated_family_count
                != family.dependency_closure.associated_families.len() + 1
            || metadata.helper_family_count
                != family
                    .dependency_closure
                    .associated_families
                    .iter()
                    .filter(|dependency| !dependency.source_visible)
                    .count()
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has inconsistent dependency closure metadata counts",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        if !family
            .revalidation_metadata
            .decreases
            .iter()
            .all(|decreases| {
                decreases.structural_recursion_checked
                    && family.schemes.first().is_some_and(|scheme| {
                        scheme
                            .params
                            .get(decreases.parameter_index)
                            .is_some_and(|param| {
                                param.name == decreases.parameter
                                    && param.domain_constraint.as_ref() == Some(&decreases.domain)
                            })
                    })
            })
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' has malformed decreases revalidation metadata",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        self.validate_imported_associated_family_dependency_closure(family)?;
        self.validate_imported_associated_family_dependency_closure_complete(family)?;
        let declaration = self
            .associated_family_declarations
            .get(&family.head)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' was not declared before validation",
                        family.visible_name
                    ),
                    anchor_span(&family.source_anchor),
                )
            })?;
        if !matches_associated_family_result_constraint(
            &family.result_domain,
            &declaration.result_domain,
        ) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.visible_name.clone(),
                reason: "summary result-domain annotation does not match the declaration"
                    .to_string(),
                span: anchor_span(&family.source_anchor),
            });
        }
        if family.schemes.is_empty() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' must contain at least one scheme",
                    family.visible_name
                ),
                anchor_span(&family.source_anchor),
            ));
        }
        for scheme in &family.schemes {
            self.validate_and_insert_imported_associated_family_scheme(
                family,
                &declaration,
                scheme.clone(),
            )?;
        }
        Ok(())
    }

    fn validate_imported_associated_family_dependency_closure(
        &self,
        family: &AssociatedFamilySummary,
    ) -> Result<(), TypeEnvError> {
        for ty in &family.dependency_closure.ordinary_types {
            if !self.canonical_type_names.contains_key(ty) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown ordinary type dependency '{}'",
                        family.visible_name, ty.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for domain in &family.dependency_closure.sealed_domains {
            if self.lookup_sealed_domain_by_id(domain).is_none() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown sealed-domain dependency '{}'",
                        family.visible_name, domain.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for constructor in &family.dependency_closure.domain_constructors {
            let domain = self.lookup_sealed_domain_by_id(&constructor.domain).ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown constructor domain '{}'",
                        family.visible_name, constructor.domain.name
                    ),
                    anchor_span(&family.source_anchor),
                )
            })?;
            if !domain
                .constructors
                .iter()
                .any(|candidate| candidate.id == *constructor)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown domain constructor '{}'",
                        family.visible_name, constructor.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for head in &family.dependency_closure.type_functions {
            if !self.local_type_functions.contains_key(head) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown type-function dependency '{}'",
                        family.visible_name, head.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        for projection in &family.dependency_closure.associated_projections {
            if !self
                .known_interface_identities
                .contains(&projection.head.interface)
                || !self
                    .known_associated_member_identities
                    .contains(&projection.head.member)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown associated projection dependency '{}::{}'",
                        family.visible_name,
                        projection.head.interface.name,
                        projection.head.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
            if let Some(declaration) = self.associated_family_declarations.get(&projection.head) {
                let expected = declaration.interface_params.len();
                let found = projection.interface_args.len();
                if found != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' associated projection dependency '{}::{}' has {} interface argument(s), expected {}",
                            family.visible_name,
                            projection.head.interface.name,
                            projection.head.member.name,
                            found,
                            expected
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
            }
        }
        for dependency in &family.dependency_closure.associated_families {
            if !self
                .associated_family_declarations
                .contains_key(&dependency.family)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' references unknown associated-family dependency '{}::{}'",
                        family.visible_name,
                        dependency.family.interface.name,
                        dependency.family.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
            if dependency.normalizer_available
                && !self
                    .associated_family_declarations
                    .contains_key(&dependency.family)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' lacks normalizer-available dependency '{}::{}'",
                        family.visible_name,
                        dependency.family.interface.name,
                        dependency.family.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }
        Ok(())
    }

    fn validate_imported_associated_family_dependency_closure_complete(
        &self,
        family: &AssociatedFamilySummary,
    ) -> Result<(), TypeEnvError> {
        let mut required = PublicAssociatedFamilyClosure::default();
        self.collect_public_canonical_type_closure_for_associated_family(
            &family.result_domain,
            &mut required,
        );
        for scheme in &family.schemes {
            self.collect_public_associated_family_scheme_closure(scheme, &mut required)?;
        }
        for projection in &family.dependency_closure.associated_projections {
            for arg in &projection.interface_args {
                self.collect_public_canonical_type_closure_for_associated_family(
                    arg,
                    &mut required,
                );
            }
        }
        required.associated_families.remove(&family.head);

        let ordinary_types = family
            .dependency_closure
            .ordinary_types
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for ty in required.ordinary_types {
            if !ordinary_types.contains(&ty) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits ordinary type '{}'",
                        family.visible_name, ty.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let sealed_domains = family
            .dependency_closure
            .sealed_domains
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for domain in required.sealed_domains {
            if !sealed_domains.contains(&domain) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits sealed domain '{}'",
                        family.visible_name, domain.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let domain_constructors = family
            .dependency_closure
            .domain_constructors
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for constructor in required.domain_constructors {
            if !domain_constructors.contains(&constructor) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits domain constructor '{}'",
                        family.visible_name, constructor.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let type_functions = family
            .dependency_closure
            .type_functions
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for head in required.type_functions {
            if !type_functions.contains(&head) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits type function '{}'",
                        family.visible_name, head.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let associated_projections = family
            .dependency_closure
            .associated_projections
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        for projection in required.projections {
            if !associated_projections.contains(&projection) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits associated projection '{}::{}' with complete argument spine",
                        family.visible_name,
                        projection.head.interface.name,
                        projection.head.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        let associated_families = family
            .dependency_closure
            .associated_families
            .iter()
            .map(|dependency| dependency.family.clone())
            .collect::<HashSet<_>>();
        for head in required.associated_families {
            if !associated_families.contains(&head) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' dependency closure omits associated family '{}::{}'",
                        family.visible_name, head.interface.name, head.member.name
                    ),
                    anchor_span(&family.source_anchor),
                ));
            }
        }

        Ok(())
    }

    fn validate_and_insert_imported_associated_family_scheme(
        &mut self,
        family: &AssociatedFamilySummary,
        declaration: &AssociatedFamilyDeclarationInfo,
        scheme: AssociatedFamilyScheme,
    ) -> Result<(), TypeEnvError> {
        if scheme.head != family.head {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' scheme head does not match summary head",
                    family.visible_name
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }
        if scheme.result_kind != Kind::Type {
            return Err(TypeEnvError::WrongAssociatedFamilyResultKind {
                family: family.visible_name.clone(),
                expected: format!("{:?}", Kind::Type),
                found: format!("{:?}", scheme.result_kind),
                span: anchor_span(&scheme.source_anchor),
            });
        }
        if !matches_associated_family_result_constraint(
            &scheme.result_domain,
            &declaration.result_domain,
        ) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.visible_name.clone(),
                reason: "scheme result-domain annotation does not match the associated family declaration"
                    .to_string(),
                span: anchor_span(&scheme.source_anchor),
            });
        }
        if scheme.params.len() != declaration.interface_params.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' scheme arity mismatch: expected {}, found {}",
                    family.visible_name,
                    declaration.interface_params.len(),
                    scheme.params.len()
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }
        for (param, expected) in scheme.params.iter().zip(&declaration.interface_params) {
            if param.kind != Kind::Type
                || param.name != expected.name
                || param.domain_constraint != expected.domain_constraint
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' scheme parameter '{}' does not match declaration",
                        family.visible_name, param.name
                    ),
                    anchor_span(&param.source_anchor),
                ));
            }
        }
        if scheme.equations.is_empty() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family summary '{}' scheme must contain at least one equation",
                    family.visible_name
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }
        for equation in &scheme.equations {
            if equation.head != scheme.head
                || equation.interface_arg_patterns.len() != declaration.interface_params.len()
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated-family summary '{}' equation head or arity mismatch",
                        family.visible_name
                    ),
                    anchor_span(&equation.source_anchor),
                ));
            }
            let mut vars = HashMap::new();
            for (pattern, param) in equation
                .interface_arg_patterns
                .iter()
                .zip(&declaration.interface_params)
            {
                self.validate_imported_associated_family_pattern(
                    family,
                    pattern,
                    &param.domain_constraint,
                    &mut vars,
                )?;
            }
            self.validate_imported_associated_family_result_expr(family, &equation.result, &vars)?;
            if !Self::associated_family_expr_conforms_to_constraint(
                &equation.result,
                &declaration.result_domain,
            ) {
                return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                    family: family.visible_name.clone(),
                    reason: format!(
                        "RHS does not conform to associated family result constraint {}",
                        associated_family_result_constraint_label(&declaration.result_domain)
                    ),
                    span: anchor_span(&equation.source_anchor),
                });
            }
        }
        for (index, left) in scheme.equations.iter().enumerate() {
            for right in scheme.equations.iter().skip(index + 1) {
                if Self::associated_family_pattern_spines_overlap(
                    &left.interface_arg_patterns,
                    &right.interface_arg_patterns,
                ) {
                    return Err(TypeEnvError::OverlappingAssociatedFamilyScheme {
                        family: family.visible_name.clone(),
                        span: anchor_span(&right.source_anchor),
                    });
                }
            }
        }
        if let Some(existing_schemes) = self.associated_family_schemes.get(&scheme.head) {
            if existing_schemes
                .iter()
                .any(|existing| existing.scheme == scheme)
            {
                return Ok(());
            }
            for existing in existing_schemes {
                for existing_equation in &existing.scheme.equations {
                    for new_equation in &scheme.equations {
                        if Self::associated_family_pattern_spines_overlap(
                            &existing_equation.interface_arg_patterns,
                            &new_equation.interface_arg_patterns,
                        ) {
                            return Err(TypeEnvError::OverlappingAssociatedFamilyScheme {
                                family: family.visible_name.clone(),
                                span: anchor_span(&new_equation.source_anchor),
                            });
                        }
                    }
                }
            }
        }
        self.associated_family_schemes
            .entry(scheme.head.clone())
            .or_default()
            .push(RegisteredAssociatedFamilyScheme {
                defining_module: declaration.defining_module.clone(),
                scheme,
            });
        Ok(())
    }

    fn validate_imported_associated_family_pattern(
        &self,
        family: &AssociatedFamilySummary,
        pattern: &AssociatedFamilyPattern,
        expected_domain: &Option<SealedDomainId>,
        vars: &mut HashMap<String, AssociatedFamilyResultConstraint>,
    ) -> Result<(), TypeEnvError> {
        match pattern {
            AssociatedFamilyPattern::Var {
                name, constraint, ..
            } => {
                let expected = expected_domain.clone().map_or(
                    AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    AssociatedFamilyResultConstraint::Domain,
                );
                if constraint != &expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' pattern variable '{}' has invalid constraint",
                            family.visible_name, name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                if vars.insert(name.clone(), expected).is_some() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' has non-linear pattern variable '{}'",
                            family.visible_name, name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                Ok(())
            }
            AssociatedFamilyPattern::Wildcard { constraint, .. } => {
                let expected = expected_domain.clone().map_or(
                    AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    AssociatedFamilyResultConstraint::Domain,
                );
                if constraint == &expected {
                    Ok(())
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' wildcard pattern has invalid constraint",
                            family.visible_name
                        ),
                        anchor_span(&family.source_anchor),
                    ))
                }
            }
            AssociatedFamilyPattern::Primitive { .. }
            | AssociatedFamilyPattern::NominalApp { .. } => Ok(()),
            AssociatedFamilyPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                ..
            } => {
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' pattern references unknown sealed domain '{}'",
                            family.visible_name, domain.name
                        ),
                        anchor_span(&family.source_anchor),
                    )
                })?;
                if !domain_summary
                    .constructors
                    .iter()
                    .any(|candidate| candidate.id == **constructor)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' pattern references unknown constructor '{}'",
                            family.visible_name, constructor.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for field in fields {
                    self.validate_imported_associated_family_pattern(family, field, &None, vars)?;
                }
                Ok(())
            }
        }
    }

    fn validate_imported_associated_family_result_expr(
        &self,
        family: &AssociatedFamilySummary,
        expr: &AssociatedFamilyResultExpr,
        vars: &HashMap<String, AssociatedFamilyResultConstraint>,
    ) -> Result<(), TypeEnvError> {
        match expr {
            AssociatedFamilyResultExpr::Primitive { kind, .. }
            | AssociatedFamilyResultExpr::Var { kind, .. }
            | AssociatedFamilyResultExpr::NominalApp { kind, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { kind, .. }
            | AssociatedFamilyResultExpr::AssociatedFamilyProjection { kind, .. }
            | AssociatedFamilyResultExpr::Projection { kind, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { kind, .. } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result expression has non-Type kind",
                            family.visible_name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
            }
        }
        match expr {
            AssociatedFamilyResultExpr::Var { name, .. } => {
                if vars.contains_key(name) || name == "Type" {
                    Ok(())
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unbound variable '{}'",
                            family.visible_name, name
                        ),
                        anchor_span(&family.source_anchor),
                    ))
                }
            }
            AssociatedFamilyResultExpr::Primitive { .. } => Ok(()),
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                ..
            } => {
                if !self.canonical_type_names.contains_key(origin) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown ordinary type '{}'",
                            family.visible_name, visible_name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                ..
            } => {
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown sealed domain '{}'",
                            family.visible_name, domain.name
                        ),
                        anchor_span(&family.source_anchor),
                    )
                })?;
                if !domain_summary
                    .constructors
                    .iter()
                    .any(|candidate| candidate.id == *constructor)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown constructor '{}'",
                            family.visible_name, constructor.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                if !self.associated_family_declarations.contains_key(head) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown associated family '{}::{}'",
                            family.visible_name, head.interface.name, head.member.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in interface_args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                if !self.known_interface_identities.contains(interface)
                    || !self.known_associated_member_identities.contains(member)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown projection '{}::{}'",
                            family.visible_name, interface.name, member.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::ComputationHeadApp { head, args, .. } => {
                if !self.local_type_functions.contains_key(head) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "associated-family summary '{}' result references unknown type function '{}'",
                            family.visible_name, head.name
                        ),
                        anchor_span(&family.source_anchor),
                    ));
                }
                for arg in args {
                    self.validate_imported_associated_family_result_expr(family, arg, vars)?;
                }
                Ok(())
            }
        }
    }

    fn validate_imported_type_function_signature_type(
        &self,
        owner: &str,
        ty: &CanonicalTypeExpr,
        position: &str,
    ) -> Result<(), TypeEnvError> {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} nominal '{visible_name}' has non-Type kind"
                        ),
                        Span::default(),
                    ));
                }
                match self.type_alias_identities.get(visible_name) {
                    Some(registered) if registered == origin => {}
                    Some(registered) => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function summary '{owner}' {position} nominal '{visible_name}' has identity mismatch: expected {:?}, found {:?}",
                                origin, registered
                            ),
                            Span::default(),
                        ));
                    }
                    None => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function summary '{owner}' {position} references unknown ordinary type '{visible_name}'"
                            ),
                            Span::default(),
                        ));
                    }
                }
                if !self.canonical_type_names.contains_key(origin) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unregistered ordinary type identity {:?}",
                            origin
                        ),
                        Span::default(),
                    ));
                }
                let expected_arity = self
                    .type_info
                    .get(visible_name)
                    .map(TypeInfo::type_arg_count)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function summary '{owner}' {position} references ordinary type '{visible_name}' without arity metadata"
                            ),
                            Span::default(),
                        )
                    })?;
                if expected_arity != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} nominal '{visible_name}' arity mismatch: expected {}, found {}",
                            expected_arity,
                            args.len()
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                kind,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} projection '{}::{}' has non-Type kind",
                            interface.name, member.name
                        ),
                        Span::default(),
                    ));
                }
                if !self.known_interface_identities.contains(interface) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unknown projection interface '{}'",
                            interface.name
                        ),
                        Span::default(),
                    ));
                }
                if !self.known_associated_member_identities.contains(member)
                    || member.interface != *interface
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unknown projection member '{}::{}'",
                            interface.name, member.name
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    Span::default(),
                )?;
                for (index, arg) in app.args.iter().enumerate() {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                    if let Some(expected_kind) = self
                        .promoted_constructor_kind(&app.constructor)
                        .and_then(|kinding| kinding.field_data_kind_constraints.get(index))
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(
                            arg,
                            expected_kind,
                            Span::default(),
                        )?;
                    }
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type-function summary '{owner}' {position} contains constructor-variable application '{}', which is unsupported until TASK-907",
                    app.constructor.name
                ),
                Span::default(),
            )),
            CanonicalTypeExpr::ComputationHeadApp { head, args, kind } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} computation head '{}' has non-Type kind",
                            head.name
                        ),
                        Span::default(),
                    ));
                }
                let callee = self.local_type_functions.get(head).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} references unknown type function '{}'",
                            head.name
                        ),
                        Span::default(),
                    )
                })?;
                if callee.params.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function summary '{owner}' {position} computation head '{}' arity mismatch: expected {}, found {}",
                            head.name,
                            callee.params.len(),
                            args.len()
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_signature_type(owner, arg, position)?;
                }
                Ok(())
            }
        }
    }

    fn validate_imported_type_function_pattern(
        &self,
        pattern: &TypeFunctionPattern,
        expected: &TypeFunctionPatternConstraint,
        vars: &mut HashMap<String, TypeFunctionResultConstraint>,
    ) -> Result<(), TypeEnvError> {
        match pattern {
            TypeFunctionPattern::Var {
                name, constraint, ..
            } => {
                if constraint != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("type-function pattern variable '{name}' has invalid constraint"),
                        Span::default(),
                    ));
                }
                if vars
                    .insert(name.clone(), result_constraint_from_pattern(constraint))
                    .is_some()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("non-linear type-function pattern variable '{name}'"),
                        Span::default(),
                    ));
                }
                Ok(())
            }
            TypeFunctionPattern::Wildcard { constraint, .. } => {
                if constraint != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function wildcard pattern has invalid constraint".to_string(),
                        Span::default(),
                    ));
                }
                Ok(())
            }
            TypeFunctionPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                constraint,
                ..
            } => {
                if constraint != expected {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function constructor pattern '{}' has invalid constraint",
                            constructor.name
                        ),
                        Span::default(),
                    ));
                }
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function pattern references unknown sealed domain '{}'",
                            domain.name
                        ),
                        Span::default(),
                    )
                })?;
                let constructor_summary = domain_summary
                    .constructors
                    .iter()
                    .find(|candidate| candidate.id == **constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function pattern references unknown constructor '{}'",
                                constructor.name
                            ),
                            Span::default(),
                        )
                    })?;
                if constructor_summary.fields.len() != fields.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function constructor pattern '{}' field arity mismatch",
                            constructor.name
                        ),
                        Span::default(),
                    ));
                }
                for (field_pattern, field) in fields.iter().zip(&constructor_summary.fields) {
                    let field_constraint = field.domain_constraint.clone().map_or_else(
                        || TypeFunctionPatternConstraint::Kind(field.kind.clone()),
                        TypeFunctionPatternConstraint::Domain,
                    );
                    self.validate_imported_type_function_pattern(
                        field_pattern,
                        &field_constraint,
                        vars,
                    )?;
                }
                Ok(())
            }
        }
    }

    fn validate_imported_type_function_result(
        &self,
        expr: &TypeFunctionResultExpr,
        vars: &HashMap<String, TypeFunctionResultConstraint>,
    ) -> Result<TypeFunctionResultConstraint, TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::Primitive { kind, .. } => {
                if kind == &Kind::Type {
                    Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        "type-function result expression has non-Type kind".to_string(),
                        Span::default(),
                    ))
                }
            }
            TypeFunctionResultExpr::Var { name, kind, .. } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("type-function result variable '{name}' has non-Type kind"),
                        Span::default(),
                    ));
                }
                vars.get(name).cloned().ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!("unbound type-function result variable '{name}'"),
                        Span::default(),
                    )
                })
            }
            TypeFunctionResultExpr::NominalApp {
                origin,
                visible_name,
                args,
                kind,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function nominal result expression has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                match self.type_alias_identities.get(visible_name) {
                    Some(registered) if registered == origin => {}
                    Some(registered) => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result nominal '{}' has identity mismatch: expected {:?}, found {:?}",
                                visible_name, origin, registered
                            ),
                            Span::default(),
                        ));
                    }
                    None => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result references unknown ordinary type '{}'",
                                visible_name
                            ),
                            Span::default(),
                        ));
                    }
                }
                if !self.canonical_type_names.contains_key(origin) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unregistered ordinary type identity {:?}",
                            origin
                        ),
                        Span::default(),
                    ));
                }
                let expected_arity = self
                    .type_info
                    .get(visible_name)
                    .map(TypeInfo::type_arg_count)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result references ordinary type '{}' without arity metadata",
                                visible_name
                            ),
                            Span::default(),
                        )
                    })?;
                if expected_arity != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result nominal '{}' arity mismatch: expected {}, found {}",
                            visible_name,
                            expected_arity,
                            args.len()
                        ),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_result(arg, vars)?;
                }
                Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
            }
            TypeFunctionResultExpr::Projection {
                interface,
                member,
                args,
                kind,
                constraint,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function projection result expression has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                if !self.known_interface_identities.contains(interface) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function projection result references unknown interface identity {:?}",
                            interface
                        ),
                        Span::default(),
                    ));
                }
                if !self.known_associated_member_identities.contains(member) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function projection result references unknown associated member identity {:?}",
                            member
                        ),
                        Span::default(),
                    ));
                }
                if member.interface != *interface {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function projection result member {:?} does not belong to interface {:?}",
                            member, interface
                        ),
                        Span::default(),
                    ));
                }
                if !matches!(constraint, TypeFunctionResultConstraint::Kind(Kind::Type)) {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function projection result cannot forge a sealed-domain constraint"
                            .to_string(),
                        Span::default(),
                    ));
                }
                for arg in args {
                    self.validate_imported_type_function_result(arg, vars)?;
                }
                Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
            }
            TypeFunctionResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
                ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function domain-constructor result has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                let domain_summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unknown sealed domain '{}'",
                            domain.name
                        ),
                        Span::default(),
                    )
                })?;
                let constructor_summary = domain_summary
                    .constructors
                    .iter()
                    .find(|candidate| candidate.id == *constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "type-function result references unknown constructor '{}'",
                                constructor.name
                            ),
                            Span::default(),
                        )
                    })?;
                if constructor_summary.fields.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result constructor '{}' field arity mismatch",
                            constructor.name
                        ),
                        Span::default(),
                    ));
                }
                for (arg, field) in args.iter().zip(&constructor_summary.fields) {
                    let actual = self.validate_imported_type_function_result(arg, vars)?;
                    let expected = field.domain_constraint.clone().map_or_else(
                        || TypeFunctionResultConstraint::Kind(field.kind.clone()),
                        TypeFunctionResultConstraint::Domain,
                    );
                    self.validate_imported_result_constraint_value(
                        &actual,
                        &expected,
                        Span::default(),
                    )?;
                }
                Ok(TypeFunctionResultConstraint::Domain(domain.clone()))
            }
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                kind,
                constraint,
                ..
            } => {
                self.validate_registered_promoted_constructor_app(
                    constructor,
                    data_kind,
                    args.len(),
                    kind,
                    Span::default(),
                )?;
                if !matches!(constraint, TypeFunctionResultConstraint::Kind(Kind::Type)) {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function promoted constructor result cannot forge a sealed-domain constraint"
                            .to_string(),
                        Span::default(),
                    ));
                }
                let kinding = self.promoted_constructor_kind(constructor).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unknown promoted data constructor '{}'",
                            constructor.name
                        ),
                        Span::default(),
                    )
                })?;
                for (index, arg) in args.iter().enumerate() {
                    let actual = self.validate_imported_type_function_result(arg, vars)?;
                    self.validate_imported_result_constraint_value(
                        &actual,
                        &TypeFunctionResultConstraint::Kind(Kind::Type),
                        Span::default(),
                    )?;
                    if let Some(expected_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_promoted_result_arg_data_kind(arg, expected_kind)?;
                    }
                }
                Ok(TypeFunctionResultConstraint::Kind(Kind::Type))
            }
            TypeFunctionResultExpr::ComputationHeadApp {
                head, args, kind, ..
            } => {
                if kind != &Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        "type-function computation result has non-Type kind".to_string(),
                        Span::default(),
                    ));
                }
                let callee = self.local_type_functions.get(head).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result references unknown computation head '{}'",
                            head.name
                        ),
                        Span::default(),
                    )
                })?;
                if callee.params.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "type-function result computation '{}' arity mismatch",
                            head.name
                        ),
                        Span::default(),
                    ));
                }
                for (arg, param) in args.iter().zip(&callee.params) {
                    let actual = self.validate_imported_type_function_result(arg, vars)?;
                    let expected = param.domain_constraint.clone().map_or_else(
                        || TypeFunctionResultConstraint::Kind(param.kind.clone()),
                        TypeFunctionResultConstraint::Domain,
                    );
                    self.validate_imported_result_constraint_value(
                        &actual,
                        &expected,
                        Span::default(),
                    )?;
                }
                Ok(callee.result_constraint.clone())
            }
        }
    }

    fn validate_imported_result_constraint_value(
        &self,
        actual: &TypeFunctionResultConstraint,
        expected: &TypeFunctionResultConstraint,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match (expected, actual) {
            (
                TypeFunctionResultConstraint::Domain(expected_domain),
                TypeFunctionResultConstraint::Domain(actual_domain),
            ) if expected_domain == actual_domain => Ok(()),
            (TypeFunctionResultConstraint::Domain(expected_domain), found) => {
                Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "result domain mismatch: expected sealed domain '{}', found {:?}",
                        expected_domain.name, found
                    ),
                    span,
                ))
            }
            (TypeFunctionResultConstraint::Kind(_), _) => Ok(()),
        }
    }

    fn validate_registered_promoted_constructor_app(
        &self,
        constructor: &PromotedConstructorId,
        data_kind: &PromotedDataKindId,
        arg_count: usize,
        kind: &Kind,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        if kind != &Kind::Type {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' has non-Type kind",
                    constructor.name
                ),
                span,
            ));
        }
        let Some(kind_summary) = self.lookup_promoted_data_kind_by_id(data_kind) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' references unknown promoted data kind '{}'",
                    constructor.name, data_kind.name
                ),
                span,
            ));
        };
        if !kind_summary
            .constructors
            .iter()
            .any(|candidate| candidate.id == *constructor)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' is not registered in promoted data kind '{}'",
                    constructor.name, data_kind.name
                ),
                span,
            ));
        }
        let Some(kinding) = self.promoted_constructor_kind(constructor) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' has no validated kinding metadata",
                    constructor.name
                ),
                span,
            ));
        };
        if &kinding.result_data_kind != data_kind {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' result data kind mismatch: expected '{}', found '{}'",
                    constructor.name, kinding.result_data_kind.name, data_kind.name
                ),
                span,
            ));
        }
        if kinding.field_data_kind_constraints.len() != arg_count {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data constructor '{}' arity mismatch: expected {}, found {}",
                    constructor.name,
                    kinding.field_data_kind_constraints.len(),
                    arg_count
                ),
                span,
            ));
        }
        Ok(())
    }

    fn validate_canonical_promoted_data_kind(
        &self,
        expr: &CanonicalTypeExpr,
        expected_kind: &PromotedDataKindId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                if &app.data_kind != expected_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "promoted data constructor '{}' has data kind '{}', expected '{}'",
                            app.constructor.name, app.data_kind.name, expected_kind.name
                        ),
                        span,
                    ));
                }
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    span,
                )?;
                let kinding = self
                    .promoted_constructor_kind(&app.constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "promoted data constructor '{}' has no validated kinding metadata",
                                app.constructor.name
                            ),
                            span,
                        )
                    })?;
                for (index, arg) in app.args.iter().enumerate() {
                    if let Some(field_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(arg, field_kind, span)?;
                    }
                }
                Ok(())
            }
            other => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind constrained field expected value of promoted data kind '{}', found {}",
                    expected_kind.name,
                    canonical_type_expr_head_name(other)
                ),
                span,
            )),
        }
    }

    fn validate_promoted_result_arg_data_kind(
        &self,
        expr: &TypeFunctionResultExpr,
        expected_kind: &PromotedDataKindId,
    ) -> Result<(), TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                kind,
                ..
            } => {
                if data_kind.as_ref() != expected_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "promoted data constructor '{}' has data kind '{}', expected '{}'",
                            constructor.name, data_kind.name, expected_kind.name
                        ),
                        Span::default(),
                    ));
                }
                self.validate_registered_promoted_constructor_app(
                    constructor,
                    data_kind,
                    args.len(),
                    kind,
                    Span::default(),
                )?;
                let kinding = self.promoted_constructor_kind(constructor).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "promoted data constructor '{}' has no validated kinding metadata",
                            constructor.name
                        ),
                        Span::default(),
                    )
                })?;
                for (index, arg) in args.iter().enumerate() {
                    if let Some(field_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_promoted_result_arg_data_kind(arg, field_kind)?;
                    }
                }
                Ok(())
            }
            other => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind constrained field expected value of promoted data kind '{}', found {}",
                    expected_kind.name,
                    type_function_result_expr_head_name(other)
                ),
                Span::default(),
            )),
        }
    }

    // ------------------------------------------------------------------
    // Sealed-domain registration helpers
    // ------------------------------------------------------------------

    /// First pass: declare a sealed-domain identity and visible alias.
    ///
    /// Checks that the domain identity is not already registered under a
    /// different visible name, and that the visible name does not collide
    /// with ordinary types or other sealed domains.
    fn declare_sealed_domain_identity(
        &mut self,
        domain: &SealedDomainSummary,
    ) -> Result<(), TypeEnvError> {
        let visible_name = domain.exported_name.as_str();

        // Check for collision with ordinary types.
        if self.ast_types.contains_key(visible_name)
            || self.type_alias_identities.contains_key(visible_name)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "sealed domain name '{}' collides with an existing ordinary type",
                    visible_name
                ),
                Span::default(),
            ));
        }

        // Check for collision with other sealed domains (different identity, same name).
        if let Some(existing) = self.sealed_domain_aliases.get(visible_name)
            && existing != &domain.id
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "duplicate sealed domain alias '{}': existing {:?}, new {:?}",
                    visible_name, existing, domain.id
                ),
                Span::default(),
            ));
        }

        // Check that the identity is not already registered under a different name.
        if self.sealed_domain_identities.contains(&domain.id)
            && let Some(alias) = self.sealed_domain_aliases.iter().find_map(|(k, v)| {
                if v == &domain.id {
                    Some(k.as_str())
                } else {
                    None
                }
            })
            && alias != visible_name
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "sealed domain identity already registered under alias '{}'",
                    alias
                ),
                Span::default(),
            ));
        }

        self.sealed_domain_identities.insert(domain.id.clone());
        self.sealed_domain_aliases
            .insert(visible_name.to_string(), domain.id.clone());

        Ok(())
    }

    /// Second pass: validate structural constraints and store the full domain summary.
    ///
    /// Validates:
    /// - Field domain references resolve to known domains
    /// - At most one `StructuralSelfDomain` field per constructor
    /// - Constructor id domain matches enclosing domain
    fn validate_and_register_sealed_domain(
        &mut self,
        domain: &SealedDomainSummary,
    ) -> Result<(), TypeEnvError> {
        for constructor in &domain.constructors {
            // Constructor id must reference the enclosing domain.
            if constructor.id.domain != domain.id {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "constructor '{}' in domain '{}' references a different domain",
                        constructor.exported_name, domain.exported_name
                    ),
                    Span::default(),
                ));
            }
            if constructor.id.name != constructor.exported_name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "constructor '{}' in domain '{}' has id name '{}' that does not match exported name",
                        constructor.exported_name, domain.exported_name, constructor.id.name
                    ),
                    Span::default(),
                ));
            }

            // At most one StructuralSelfDomain field per constructor.
            let structural_count = constructor
                .fields
                .iter()
                .filter(|f| f.structural_status == StructuralFieldStatus::StructuralSelfDomain)
                .count();
            if structural_count > 1 {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "constructor '{}' in domain '{}' has {} structural self-domain fields; at most one is permitted",
                        constructor.exported_name, domain.exported_name, structural_count
                    ),
                    Span::default(),
                ));
            }

            // Validate field kinds, structural status, and domain references.
            for field in &constructor.fields {
                if field.kind != Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in constructor '{}' has non-Type kind",
                            field.name, constructor.exported_name
                        ),
                        Span::default(),
                    ));
                }
                let expected_status = if field.domain_constraint.as_ref() == Some(&domain.id) {
                    StructuralFieldStatus::StructuralSelfDomain
                } else {
                    StructuralFieldStatus::NonStructural
                };
                if field.structural_status != expected_status {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in constructor '{}' has structural status {:?}; expected {:?}",
                            field.name,
                            constructor.exported_name,
                            field.structural_status,
                            expected_status
                        ),
                        Span::default(),
                    ));
                }
                if let Some(ref constraint) = field.domain_constraint {
                    // The constraint must be the enclosing domain (self-reference) or
                    // a domain already declared in this environment.
                    if constraint != &domain.id
                        && !self.sealed_domain_identities.contains(constraint)
                    {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "field '{}' in constructor '{}' references unknown sealed domain",
                                field.name, constructor.exported_name
                            ),
                            Span::default(),
                        ));
                    }
                }
            }
        }

        // Store the full domain summary.
        self.sealed_domain_summaries
            .insert(domain.id.clone(), domain.clone());

        Ok(())
    }

    /// Look up a sealed domain by its visible exported name.
    #[must_use]
    pub fn lookup_sealed_domain(&self, name: &str) -> Option<&SealedDomainSummary> {
        let id = self.sealed_domain_aliases.get(name)?;
        self.sealed_domain_summaries.get(id)
    }

    /// First pass: declare a promoted data-kind identity and visible alias.
    fn declare_promoted_data_kind_identity(
        &mut self,
        data_kind: &PromotedDataKindSummary,
    ) -> Result<(), TypeEnvError> {
        let visible_name = data_kind.exported_name.as_str();
        let hidden_dependency_metadata = is_dependency_metadata_name(visible_name);
        if !hidden_dependency_metadata
            && let Some(existing) = self.promoted_data_kind_aliases.get(visible_name)
            && existing != &data_kind.id
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "duplicate promoted data-kind alias '{}': existing {:?}, new {:?}",
                    visible_name, existing, data_kind.id
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }
        if !hidden_dependency_metadata
            && self.promoted_data_kind_identities.contains(&data_kind.id)
            && let Some(alias) = self
                .promoted_data_kind_aliases
                .iter()
                .find_map(|(alias, id)| {
                    if id == &data_kind.id {
                        Some(alias.as_str())
                    } else {
                        None
                    }
                })
            && alias != visible_name
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind identity already registered under alias '{}'",
                    alias
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }

        self.promoted_data_kind_identities
            .insert(data_kind.id.clone());
        if !hidden_dependency_metadata {
            self.promoted_data_kind_aliases
                .insert(visible_name.to_string(), data_kind.id.clone());
        }
        Ok(())
    }

    /// Second pass: validate source-ADT, source-constructor, field-domain, and kinding metadata.
    fn validate_and_register_promoted_data_kind(
        &mut self,
        data_kind: &PromotedDataKindSummary,
    ) -> Result<(), TypeEnvError> {
        let source_visible_name = self
            .canonical_type_names
            .get(&data_kind.source_type)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted data-kind '{}' references unknown source ADT '{}'",
                        data_kind.exported_name, data_kind.source_type.name
                    ),
                    anchor_span(&data_kind.source_anchor),
                )
            })?;
        let source_variants = match self.type_info.get(&source_visible_name).cloned() {
            Some(TypeInfo::Enum { variants, .. }) => variants,
            _ => {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted data-kind '{}' source ADT '{}' is not an exposed enum",
                        data_kind.exported_name, data_kind.source_type.name
                    ),
                    anchor_span(&data_kind.source_anchor),
                ));
            }
        };

        if data_kind.constructors.len() != source_variants.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "promoted data-kind '{}' has {} constructor(s) but source ADT '{}' has {}",
                    data_kind.exported_name,
                    data_kind.constructors.len(),
                    data_kind.source_type.name,
                    source_variants.len()
                ),
                anchor_span(&data_kind.source_anchor),
            ));
        }

        for (index, constructor) in data_kind.constructors.iter().enumerate() {
            if constructor.source_constructor.parent != data_kind.source_type {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "source constructor for promoted constructor '{}' does not belong to source ADT '{}'",
                        constructor.exported_name, data_kind.source_type.name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }

            let source_variant = &source_variants[index];
            if constructor.source_constructor.name != source_variant.name {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor '{}' at index {} does not match source constructor '{}'",
                        constructor.exported_name, index, source_variant.name
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            let actual_payload_kind = match &source_variant.payload_shape {
                VariantPayloadShape::Unit => ConstructorPayloadKind::Unit,
                VariantPayloadShape::Record => ConstructorPayloadKind::Record,
                VariantPayloadShape::Tuple => ConstructorPayloadKind::Tuple,
            };
            if actual_payload_kind != constructor.source_constructor.payload_kind {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor '{}' source payload kind {:?} conflicts with exposed source ADT payload kind {:?}",
                        constructor.exported_name,
                        constructor.source_constructor.payload_kind,
                        actual_payload_kind
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }
            if constructor.fields.len() != source_variant.fields.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "promoted constructor '{}' has {} promoted field(s) but source constructor '{}' has {} field(s)",
                        constructor.exported_name,
                        constructor.fields.len(),
                        constructor.source_constructor.name,
                        source_variant.fields.len()
                    ),
                    anchor_span(&constructor.source_anchor),
                ));
            }

            let mut field_constraints = Vec::with_capacity(constructor.fields.len());
            for (index, field) in constructor.fields.iter().enumerate() {
                let (source_field_name, source_field_ty) = &source_variant.fields[index];
                if source_variant.payload_shape == VariantPayloadShape::Record
                    && field.name.as_str() != source_field_name.as_str()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' does not match source field '{}'",
                            field.name, constructor.exported_name, source_field_name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                if field.kind != Kind::Type {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' has non-Type kind",
                            field.name, constructor.exported_name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                let Some(field_data_kind) = field.data_kind_constraint.clone() else {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' lacks promoted data-kind constraint",
                            field.name, constructor.exported_name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                };
                if !self
                    .promoted_data_kind_identities
                    .contains(&field_data_kind)
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' references unknown promoted data kind '{}'",
                            field.name, constructor.exported_name, field_data_kind.name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                let expected_source_name = self
                    .canonical_type_names
                    .get(&field_data_kind.source_type)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "field '{}' in promoted constructor '{}' references promoted data kind '{}' with unknown source ADT '{}'",
                                field.name,
                                constructor.exported_name,
                                field_data_kind.name,
                                field_data_kind.source_type.name
                            ),
                            anchor_span(&field.source_anchor),
                        )
                    })?;
                let source_field_matches_promoted_kind = matches!(
                    source_field_ty,
                    Type::Constructor { name, args, kind }
                        if args.is_empty() && kind.is_type() && name.name == *expected_source_name
                );
                if !source_field_matches_promoted_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "field '{}' in promoted constructor '{}' expects source field type for promoted data kind '{}'",
                            field.name, constructor.exported_name, field_data_kind.name
                        ),
                        anchor_span(&field.source_anchor),
                    ));
                }
                field_constraints.push(Some(field_data_kind));
            }

            self.promoted_constructor_kinds.insert(
                constructor.id.clone(),
                PromotedConstructorKindInfo {
                    kind: Kind::n_ary(constructor.fields.len()),
                    result_data_kind: data_kind.id.clone(),
                    field_data_kind_constraints: field_constraints,
                },
            );
            self.promoted_constructor_summaries
                .insert(constructor.id.clone(), constructor.clone());
        }

        let should_store_data_kind = self
            .promoted_data_kind_summaries
            .get(&data_kind.id)
            .is_none_or(|existing| {
                is_dependency_metadata_name(&existing.exported_name)
                    || !is_dependency_metadata_name(&data_kind.exported_name)
            });
        if should_store_data_kind {
            self.promoted_data_kind_summaries
                .insert(data_kind.id.clone(), data_kind.clone());
        }
        Ok(())
    }

    /// Look up a promoted data kind by its visible exported name.
    #[must_use]
    pub fn lookup_promoted_data_kind(&self, name: &str) -> Option<&PromotedDataKindSummary> {
        let id = self.promoted_data_kind_aliases.get(name)?;
        self.promoted_data_kind_summaries.get(id)
    }

    /// Look up a promoted data kind by canonical identity.
    #[must_use]
    pub fn lookup_promoted_data_kind_by_id(
        &self,
        id: &PromotedDataKindId,
    ) -> Option<&PromotedDataKindSummary> {
        self.promoted_data_kind_summaries.get(id)
    }

    /// Look up a promoted data constructor by canonical identity.
    #[must_use]
    pub fn lookup_promoted_constructor_by_id(
        &self,
        id: &PromotedConstructorId,
    ) -> Option<&PromotedConstructorSummary> {
        self.promoted_constructor_summaries.get(id)
    }

    /// Return checked kind/domain metadata for a promoted data constructor.
    #[must_use]
    pub fn promoted_constructor_kind(
        &self,
        id: &PromotedConstructorId,
    ) -> Option<&PromotedConstructorKindInfo> {
        self.promoted_constructor_kinds.get(id)
    }

    /// Register a source-ordered batch of module-local type functions.
    ///
    /// TASK-834 deliberately performs only minimal honest lowering/registration:
    /// the current head is provisional during its own lowering, earlier published
    /// heads are visible, later same-module heads are rejected, and the checked
    /// carrier is published only after lowering succeeds. Deeper SPEC-E validation
    /// (coverage, overlap, and recursion proof obligations) remains owned by
    /// TASK-836/837.
    pub fn register_local_type_functions(
        &mut self,
        module: &ModuleIdentity,
        defs: &[SurfaceTypeFnDef],
    ) -> Result<(), TypeEnvError> {
        let mut staged = self.clone();
        staged.register_local_type_functions_inner(module, defs)?;
        *self = staged;
        Ok(())
    }

    /// Register a local sealed-domain summary for source declarations in the current module.
    ///
    /// Unlike `register_module_semantic_summary`, this does not require public visibility because
    /// it models same-module domains before export filtering. Public export validation rejects any
    /// `pub type fn` whose checked equations depend on private domains or marker constructors.
    pub fn register_local_sealed_domain_summary(
        &mut self,
        domain: &SealedDomainSummary,
    ) -> Result<(), TypeEnvError> {
        let mut staged = self.clone();
        staged.declare_sealed_domain_identity(domain)?;
        staged.validate_and_register_sealed_domain(domain)?;
        *self = staged;
        Ok(())
    }

    /// Look up a source-visible type function by local or imported name.
    #[must_use]
    pub fn lookup_local_type_function(&self, name: &str) -> Option<&TypeFunctionDef> {
        let head = self.local_type_function_heads.get(name)?;
        self.local_type_functions.get(head)
    }

    /// Make an imported public type-function summary source-visible under `name`.
    ///
    /// Import loaders call this only for explicitly selected or glob-imported
    /// public heads. Dependency-closure helper heads remain normalizer-available
    /// by canonical identity but are not inserted here.
    pub fn expose_imported_type_function_name(
        &mut self,
        name: impl Into<String>,
        head: TypeComputationHeadId,
    ) -> Result<(), TypeEnvError> {
        let name = name.into();
        if !self.local_type_functions.contains_key(&head) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "cannot expose imported type function '{}' before registering summary head '{}::{}'",
                    name,
                    head.module.path.join("::"),
                    head.name
                ),
                Span::default(),
            ));
        }
        if let Some(existing) = self.local_type_function_heads.get(&name) {
            if existing == &head {
                return Ok(());
            }
            return Err(TypeEnvError::ImportOrderConflict {
                family: "type-function visible name".to_string(),
                name,
                span: Span::default(),
            });
        }
        self.local_type_function_heads.insert(name, head);
        Ok(())
    }

    /// Look up a published computation head by canonical identity.
    ///
    /// This unified normalizer lookup covers checked local declarations,
    /// atomically imported public summaries, and any future TypeEnv-owned
    /// computation-head sources. Imported heads are deliberately not inserted into
    /// `local_type_function_heads`, so they remain unavailable to local-name
    /// source lookup unless a later import/re-export path makes them visible.
    #[must_use]
    pub(crate) fn lookup_type_function_by_head(
        &self,
        head: &TypeComputationHeadId,
    ) -> Option<&TypeFunctionDef> {
        self.local_type_functions.get(head)
    }

    /// Iterate source-visible local and imported type-function names.
    ///
    /// Imported dependency-closure helper heads are intentionally omitted unless
    /// the import loader explicitly exposes them through selected/glob syntax.
    pub fn local_type_function_names(&self) -> impl Iterator<Item = &str> {
        self.local_type_function_heads.keys().map(String::as_str)
    }

    /// Lower checked, transparent, export-closed public local type functions into
    /// SPEC-062 public computation summaries.
    ///
    /// This only exports already-validated public source definitions for the
    /// requested defining module. It deliberately does not register imported
    /// normalizer facts or expose private/local-only type functions.
    pub fn export_public_type_function_summaries(
        &self,
        module: &ModuleIdentity,
    ) -> Result<Vec<TypeFunctionSummary>, TypeEnvError> {
        let mut defs = self
            .local_type_functions
            .values()
            .filter(|def| {
                def.visibility == ash_core::ast::Visibility::Public && def.head.module == *module
            })
            .collect::<Vec<_>>();
        defs.sort_by(|left, right| {
            let left_start = left
                .source_anchors
                .definition
                .span
                .map_or(usize::MAX, |s| s.start);
            let right_start = right
                .source_anchors
                .definition
                .span
                .map_or(usize::MAX, |s| s.start);
            left_start
                .cmp(&right_start)
                .then_with(|| left.name.cmp(&right.name))
        });

        defs.into_iter()
            .map(|def| self.lower_public_type_function_summary(def))
            .collect()
    }

    /// Lower checked public associated-family schemes for the requested module into
    /// SPEC-063 public associated-family summaries.
    pub fn export_public_associated_family_summaries(
        &self,
        module: &ModuleIdentity,
    ) -> Result<Vec<AssociatedFamilySummary>, TypeEnvError> {
        let mut declarations = self
            .associated_family_declarations
            .values()
            .filter(|declaration| declaration.defining_module == *module)
            .collect::<Vec<_>>();
        declarations.sort_by(|left, right| {
            left.head
                .interface
                .name
                .cmp(&right.head.interface.name)
                .then_with(|| left.head.member.name.cmp(&right.head.member.name))
        });

        let mut exportable_heads = HashMap::new();
        for declaration in &declarations {
            let interface_name = declaration.head.interface.name.to_string();
            let is_public_interface = self
                .interfaces
                .get(&interface_name)
                .is_some_and(|info| matches!(info.visibility, ash_core::ast::Visibility::Public));
            if !is_public_interface {
                continue;
            }
            let schemes = self
                .associated_family_schemes
                .get(&declaration.head)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|registered| registered.defining_module == *module)
                .map(|registered| registered.scheme)
                .collect::<Vec<_>>();
            if schemes.is_empty() {
                continue;
            }
            exportable_heads.insert(declaration.head.clone(), true);

            let mut closure = PublicAssociatedFamilyClosure::default();
            self.collect_public_associated_family_constraint_closure(
                &declaration.result_domain,
                &mut closure,
            );
            for scheme in &schemes {
                self.collect_public_associated_family_scheme_closure(scheme, &mut closure)?;
            }
            for dependency in closure.associated_families {
                if dependency == declaration.head {
                    continue;
                }
                if self
                    .associated_family_declarations
                    .get(&dependency)
                    .is_some_and(|dependency_declaration| {
                        dependency_declaration.defining_module == *module
                            && !self
                                .interfaces
                                .get(dependency.interface.name.as_str())
                                .is_some_and(|info| {
                                    matches!(info.visibility, ash_core::ast::Visibility::Public)
                                })
                    })
                {
                    exportable_heads.entry(dependency).or_insert(false);
                }
            }
        }

        let mut summaries = Vec::new();
        for declaration in declarations {
            let Some(source_visible) = exportable_heads.get(&declaration.head).copied() else {
                continue;
            };
            let schemes = self
                .associated_family_schemes
                .get(&declaration.head)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|registered| registered.defining_module == *module)
                .map(|registered| registered.scheme)
                .collect::<Vec<_>>();
            if schemes.is_empty() {
                continue;
            }
            let mut closure = PublicAssociatedFamilyClosure::default();
            self.collect_public_associated_family_constraint_closure(
                &declaration.result_domain,
                &mut closure,
            );
            for scheme in &schemes {
                self.collect_public_associated_family_scheme_closure(scheme, &mut closure)?;
            }
            self.validate_public_associated_family_export_closure(&closure, &declaration.head)?;
            let associated_family_refs =
                closure.associated_family_summary_refs(&declaration.head, module, self);
            let helper_family_count = associated_family_refs
                .iter()
                .filter(|reference| !reference.source_visible)
                .count();
            let public_associated_family_count = associated_family_refs.len() + 1;
            let decreases = declaration
                .decreases
                .as_ref()
                .and_then(|param| {
                    declaration
                        .interface_params
                        .iter()
                        .position(|candidate| candidate.name == *param)
                        .map(|index| (param, index))
                })
                .and_then(|(param, index)| {
                    declaration
                        .interface_params
                        .get(index)
                        .and_then(|param_info| param_info.domain_constraint.clone())
                        .map(|domain| ValidatedDecreasesSummary {
                            parameter: param.clone(),
                            parameter_index: index,
                            domain,
                            structural_recursion_checked: true,
                            source_anchor: SourceAnchor::new(
                                SourceOrigin::Synthetic {
                                    reason: "associated family decreases export".to_string(),
                                },
                                None,
                                format!("associated family decreases {param}"),
                            ),
                        })
                })
                .into_iter()
                .collect::<Vec<_>>();
            summaries.push(AssociatedFamilySummary {
                head: declaration.head.clone(),
                interface_identity: declaration.head.interface.clone(),
                member_identity: declaration.head.member.clone(),
                visible_name: if source_visible {
                    declaration.head.member.name.to_string()
                } else {
                    dependency_metadata_name(&declaration.head.member.name)
                },
                result_domain: canonical_expr_for_associated_family_constraint(
                    &declaration.result_domain,
                ),
                result_kind: Kind::Type,
                export_mode: AssociatedFamilyExportMode::TransparentEquations,
                schemes,
                dependency_closure: ash_core::semantic_summary::AssociatedFamilyDependencyClosure {
                    ordinary_types: closure.ordinary_types.iter().cloned().collect(),
                    sealed_domains: closure.sealed_domains.iter().cloned().collect(),
                    domain_constructors: closure.domain_constructors.iter().cloned().collect(),
                    type_functions: closure.type_functions.iter().cloned().collect(),
                    associated_projections: closure.projections.iter().cloned().collect(),
                    associated_families: associated_family_refs,
                    type_function_summaries: Vec::new(),
                    closure_metadata: AssociatedFamilyClosureMetadata {
                        public_closure_checked: true,
                        public_ordinary_type_count: closure.ordinary_types.len(),
                        public_sealed_domain_count: closure.sealed_domains.len(),
                        public_domain_constructor_count: closure.domain_constructors.len(),
                        public_type_function_count: closure.type_functions.len(),
                        public_associated_family_count,
                        public_projection_count: closure.projections.len(),
                        helper_family_count,
                    },
                },
                source_anchor: SourceAnchor::new(
                    SourceOrigin::Synthetic {
                        reason: "associated family summary export".to_string(),
                    },
                    None,
                    format!("associated family summary {}", declaration.head.member.name),
                ),
                revalidation_metadata: AssociatedFamilyRevalidationMetadata {
                    spec_version: SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
                    kind_and_domain_checked: true,
                    coverage_and_overlap_checked: true,
                    coherence_checked: true,
                    recursion_checked: true,
                    decreases,
                },
            });
        }
        Ok(summaries)
    }

    fn validate_public_associated_family_export_closure(
        &self,
        closure: &PublicAssociatedFamilyClosure,
        head: &AssociatedFamilyHeadId,
    ) -> Result<(), TypeEnvError> {
        for ty in &closure.ordinary_types {
            if ty.module == head.interface.module
                && self
                    .ast_types
                    .get(ty.name.as_str())
                    .is_some_and(|def| !matches!(def.visibility, ash_core::ast::Visibility::Public))
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "public associated family '{}::{}' references private ordinary type '{}'",
                        head.interface.name, head.member.name, ty.name
                    ),
                    Span::default(),
                ));
            }
        }
        for domain in &closure.sealed_domains {
            if domain.module == head.interface.module
                && self
                    .sealed_domain_summaries
                    .get(domain)
                    .is_some_and(|summary| {
                        !matches!(summary.visibility, ash_core::ast::Visibility::Public)
                    })
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "public associated family '{}::{}' references private sealed domain '{}'",
                        head.interface.name, head.member.name, domain.name
                    ),
                    Span::default(),
                ));
            }
        }
        Ok(())
    }

    fn collect_public_associated_family_constraint_closure(
        &self,
        constraint: &AssociatedFamilyResultConstraint,
        closure: &mut PublicAssociatedFamilyClosure,
    ) {
        match constraint {
            AssociatedFamilyResultConstraint::Kind(_) => {}
            AssociatedFamilyResultConstraint::Domain(domain) => {
                closure.sealed_domains.insert(domain.clone());
            }
        }
    }

    fn collect_public_canonical_type_closure_for_associated_family(
        &self,
        ty: &CanonicalTypeExpr,
        closure: &mut PublicAssociatedFamilyClosure,
    ) {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => {}
            CanonicalTypeExpr::NominalApp { origin, args, .. } => {
                closure.ordinary_types.insert(origin.clone());
                for arg in args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                kind,
                rigidity,
            } => {
                closure.projections.insert(AssociatedFamilyProjection {
                    head: AssociatedFamilyHeadId {
                        interface: interface.clone(),
                        member: member.clone(),
                    },
                    interface_args: args.clone(),
                    kind: kind.clone(),
                    rigidity: *rigidity,
                    mode: AssociatedFamilyProjectionMode::NeutralBlockedOrUnavailable,
                });
                for arg in args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
                closure.type_functions.insert(head.clone());
                for arg in args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                for arg in &app.args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => {
                for arg in &app.args {
                    self.collect_public_canonical_type_closure_for_associated_family(arg, closure);
                }
            }
        }
    }

    fn collect_public_associated_family_scheme_closure(
        &self,
        scheme: &AssociatedFamilyScheme,
        closure: &mut PublicAssociatedFamilyClosure,
    ) -> Result<(), TypeEnvError> {
        self.collect_public_canonical_type_closure_for_associated_family(
            &scheme.result_domain,
            closure,
        );
        for param in &scheme.params {
            self.collect_public_canonical_type_closure_for_associated_family(&param.ty, closure);
            if let Some(domain) = &param.domain_constraint {
                closure.sealed_domains.insert(domain.clone());
            }
        }
        for equation in &scheme.equations {
            for pattern in &equation.interface_arg_patterns {
                self.collect_public_associated_family_pattern_closure(pattern, closure);
            }
            self.collect_public_associated_family_result_closure(&equation.result, closure)?;
        }
        Ok(())
    }

    fn collect_public_associated_family_pattern_closure(
        &self,
        pattern: &AssociatedFamilyPattern,
        closure: &mut PublicAssociatedFamilyClosure,
    ) {
        match pattern {
            AssociatedFamilyPattern::Var { constraint, .. }
            | AssociatedFamilyPattern::Wildcard { constraint, .. } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
            }
            AssociatedFamilyPattern::Primitive { constraint, .. } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
            }
            AssociatedFamilyPattern::NominalApp {
                origin,
                args,
                constraint,
                ..
            } => {
                closure.ordinary_types.insert(origin.clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_associated_family_pattern_closure(arg, closure);
                }
            }
            AssociatedFamilyPattern::DomainConstructor {
                domain,
                constructor,
                fields,
                constraint,
                ..
            } => {
                closure.sealed_domains.insert((**domain).clone());
                closure.domain_constructors.insert((**constructor).clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for field in fields {
                    self.collect_public_associated_family_pattern_closure(field, closure);
                }
            }
        }
    }

    fn collect_public_associated_family_head_closure(
        &self,
        head: &AssociatedFamilyHeadId,
        closure: &mut PublicAssociatedFamilyClosure,
    ) -> Result<(), TypeEnvError> {
        if !closure.associated_families.insert(head.clone()) {
            return Ok(());
        }
        let schemes = self
            .associated_family_schemes
            .get(head)
            .cloned()
            .unwrap_or_default();
        for registered in schemes {
            self.collect_public_associated_family_scheme_closure(&registered.scheme, closure)?;
        }
        Ok(())
    }

    fn collect_public_associated_family_result_closure(
        &self,
        expr: &AssociatedFamilyResultExpr,
        closure: &mut PublicAssociatedFamilyClosure,
    ) -> Result<(), TypeEnvError> {
        match expr {
            AssociatedFamilyResultExpr::Primitive { constraint, .. }
            | AssociatedFamilyResultExpr::Var { constraint, .. } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
            }
            AssociatedFamilyResultExpr::NominalApp {
                origin,
                args,
                constraint,
                ..
            } => {
                closure.ordinary_types.insert(origin.clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
            AssociatedFamilyResultExpr::DomainConstructorApp {
                domain,
                constructor,
                args,
                constraint,
                ..
            } => {
                closure.sealed_domains.insert(domain.clone());
                closure.domain_constructors.insert(constructor.clone());
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
            AssociatedFamilyResultExpr::Projection {
                args, constraint, ..
            }
            | AssociatedFamilyResultExpr::ComputationHeadApp {
                args, constraint, ..
            } => {
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                if let AssociatedFamilyResultExpr::ComputationHeadApp { head, .. } = expr {
                    closure.type_functions.insert(head.clone());
                }
                for arg in args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                rigidity,
                constraint,
                ..
            } => {
                self.collect_public_associated_family_head_closure(head, closure)?;
                self.collect_public_associated_family_constraint_closure(constraint, closure);
                let canonical_args = interface_args
                    .iter()
                    .map(associated_family_result_expr_to_canonical)
                    .collect::<Result<Vec<_>, _>>()?;
                closure.projections.insert(AssociatedFamilyProjection {
                    head: head.clone(),
                    interface_args: canonical_args,
                    kind: kind.clone(),
                    rigidity: *rigidity,
                    mode: AssociatedFamilyProjectionMode::ReducibleSealedFamilyHead,
                });
                for arg in interface_args {
                    self.collect_public_associated_family_result_closure(arg, closure)?;
                }
            }
        }
        Ok(())
    }

    fn lower_public_type_function_summary(
        &self,
        def: &TypeFunctionDef,
    ) -> Result<TypeFunctionSummary, TypeEnvError> {
        self.validate_public_type_function_export_closure(def, Span::default())?;

        let mut closure = PublicTypeFunctionClosure::default();
        self.collect_public_type_function_def_closure(def, &mut closure);

        Ok(TypeFunctionSummary {
            exported_name: def.name.clone(),
            head: def.head.clone(),
            visibility: def.visibility,
            params: def
                .params
                .iter()
                .map(|param| TypeFunctionParamSummary {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                    kind: param.kind.clone(),
                    domain_constraint: param.domain_constraint.clone(),
                    source_anchor: param.source_anchor.clone(),
                })
                .collect(),
            return_type: def.return_type.clone(),
            return_kind: def.return_kind.clone(),
            result_constraint: def.result_constraint.clone(),
            export_mode: TypeFunctionExportMode::TransparentEquations,
            source_anchors: def.source_anchors.clone(),
            equations: def.equations.clone(),
            dependency_summary_refs: closure.dependency_summary_refs(),
            closure_metadata: TypeFunctionClosureMetadata {
                public_closure_checked: true,
                public_ordinary_type_count: closure.ordinary_types.len(),
                public_sealed_domain_count: closure.sealed_domains.len(),
                public_type_function_count: closure.type_functions.len(),
                public_projection_count: closure.projections.len(),
            },
            revalidation_metadata: TypeFunctionRevalidationMetadata {
                spec_version: SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
                structural_recursion_checked: true,
                kind_and_domain_checked: true,
                coverage_and_overlap_checked: true,
                decreases_param: def.decreases.clone(),
            },
        })
    }

    fn collect_public_type_function_def_closure(
        &self,
        def: &TypeFunctionDef,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        if !closure.type_functions.insert(def.head.clone()) {
            return;
        }
        for param in &def.params {
            if let Some(domain) = &param.domain_constraint {
                closure.sealed_domains.insert(domain.clone());
            }
            self.collect_public_canonical_type_closure(&param.ty, closure);
        }
        if let TypeFunctionResultConstraint::Domain(domain) = &def.result_constraint {
            closure.sealed_domains.insert(domain.clone());
        }
        self.collect_public_canonical_type_closure(&def.return_type, closure);
        for equation in &def.equations {
            for pattern in &equation.patterns {
                self.collect_public_pattern_closure(pattern, closure);
            }
            self.collect_public_result_closure(&equation.result, closure);
        }
    }

    fn collect_public_type_function_head_closure(
        &self,
        head: &TypeComputationHeadId,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match self.local_type_functions.get(head) {
            Some(def) if def.visibility == ash_core::ast::Visibility::Public => {
                self.collect_public_type_function_def_closure(def, closure);
            }
            _ => {
                closure.type_functions.insert(head.clone());
            }
        }
    }

    fn collect_public_canonical_type_closure(
        &self,
        ty: &CanonicalTypeExpr,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => {}
            CanonicalTypeExpr::NominalApp { origin, args, .. } => {
                closure.ordinary_types.insert(origin.clone());
                for arg in args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                closure
                    .projections
                    .insert((interface.clone(), member.clone()));
                for arg in args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
                self.collect_public_type_function_head_closure(head, closure);
                for arg in args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                closure.promoted_data_kinds.insert(app.data_kind.clone());
                closure
                    .promoted_constructors
                    .insert(app.constructor.clone());
                closure
                    .ordinary_types
                    .insert(app.data_kind.source_type.clone());
                for arg in &app.args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => {
                for arg in &app.args {
                    self.collect_public_canonical_type_closure(arg, closure);
                }
            }
        }
    }

    fn collect_public_pattern_closure(
        &self,
        pattern: &TypeFunctionPattern,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match pattern {
            TypeFunctionPattern::DomainConstructor { domain, fields, .. } => {
                closure.sealed_domains.insert((**domain).clone());
                for field in fields {
                    self.collect_public_pattern_closure(field, closure);
                }
            }
            TypeFunctionPattern::Var { constraint, .. }
            | TypeFunctionPattern::Wildcard { constraint, .. } => {
                if let TypeFunctionPatternConstraint::Domain(domain) = constraint {
                    closure.sealed_domains.insert(domain.clone());
                }
            }
        }
    }

    fn collect_public_result_closure(
        &self,
        expr: &TypeFunctionResultExpr,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        match expr {
            TypeFunctionResultExpr::Primitive { constraint, .. }
            | TypeFunctionResultExpr::Var { constraint, .. } => {
                Self::collect_result_constraint_closure(constraint, closure);
            }
            TypeFunctionResultExpr::NominalApp {
                origin,
                args,
                constraint,
                ..
            } => {
                closure.ordinary_types.insert(origin.clone());
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::DomainConstructorApp {
                domain,
                args,
                constraint,
                ..
            } => {
                closure.sealed_domains.insert(domain.clone());
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                constraint,
                ..
            } => {
                closure.promoted_data_kinds.insert((**data_kind).clone());
                closure
                    .promoted_constructors
                    .insert((**constructor).clone());
                closure.ordinary_types.insert(data_kind.source_type.clone());
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::Projection {
                interface,
                member,
                args,
                constraint,
                ..
            } => {
                closure
                    .projections
                    .insert((interface.clone(), member.clone()));
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
            TypeFunctionResultExpr::ComputationHeadApp {
                head,
                args,
                constraint,
                ..
            } => {
                self.collect_public_type_function_head_closure(head, closure);
                Self::collect_result_constraint_closure(constraint, closure);
                for arg in args {
                    self.collect_public_result_closure(arg, closure);
                }
            }
        }
    }

    fn collect_result_constraint_closure(
        constraint: &TypeFunctionResultConstraint,
        closure: &mut PublicTypeFunctionClosure,
    ) {
        if let TypeFunctionResultConstraint::Domain(domain) = constraint {
            closure.sealed_domains.insert(domain.clone());
        }
    }

    fn register_local_type_functions_inner(
        &mut self,
        module: &ModuleIdentity,
        defs: &[SurfaceTypeFnDef],
    ) -> Result<(), TypeEnvError> {
        let mut seen_in_batch = HashSet::new();
        for def in defs {
            let name = def.name.to_string();
            if self.local_type_function_heads.contains_key(&name)
                || !seen_in_batch.insert(name.clone())
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("duplicate type function '{name}'"),
                    def.span,
                ));
            }
        }

        for (index, def) in defs.iter().enumerate() {
            let later_names: HashSet<String> = defs
                .iter()
                .skip(index + 1)
                .map(|later| later.name.to_string())
                .collect();
            let lowered = self.lower_local_type_function(module, def, &later_names)?;
            let obligation_start = self.proposition_obligations.len();
            self.local_type_function_heads
                .insert(lowered.name.clone(), lowered.head.clone());
            self.local_type_functions
                .insert(lowered.head.clone(), lowered);
            if let Some(tail) = &def.proposition_tail {
                self.add_proposition_obligations_from_tail(
                    tail,
                    SourceOrigin::Synthetic {
                        reason: format!(
                            "type function proposition checking point {}::{}",
                            module.path.join("::"),
                            def.name
                        ),
                    },
                    PropositionCheckingSite::new(
                        0x8800_0000u64 + index as u64,
                        PropositionCheckingSiteKind::ExplicitRequirement,
                        Some(format!("type fn {} proposition tail", def.name)),
                    ),
                )
                .map_err(proposition_revalidation_error)?;
                self.discharge_required_proposition_obligations_from(obligation_start)?;
            }
        }
        Ok(())
    }

    fn lower_local_type_function(
        &self,
        module: &ModuleIdentity,
        def: &SurfaceTypeFnDef,
        later_names: &HashSet<String>,
    ) -> Result<TypeFunctionDef, TypeEnvError> {
        let head = TypeComputationHeadId::new(module.clone(), def.name.to_string());
        let params = def
            .params
            .iter()
            .map(|param| {
                let (ty, constraint) = self.lower_type_fn_signature_type(&param.ty)?;
                Ok(TypeFunctionParam {
                    name: param.name.to_string(),
                    ty,
                    kind: Kind::Type,
                    domain_constraint: constraint,
                    source_anchor: span_anchor(param.span, format!("type fn param {}", param.name)),
                })
            })
            .collect::<Result<Vec<_>, TypeEnvError>>()?;
        if !params.iter().any(|param| param.domain_constraint.is_some()) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type function '{}' has no sealed-domain scrutinee in its parameter list",
                    def.name
                ),
                def.header_span,
            ));
        }
        let (return_type, result_domain) = self.lower_type_fn_signature_type(&def.return_type)?;
        let result_constraint = match result_domain.clone() {
            Some(domain) => TypeFunctionResultConstraint::Domain(domain),
            None => TypeFunctionResultConstraint::Kind(Kind::Type),
        };

        let mut equations = Vec::with_capacity(def.equations.len());
        for (ordinal, equation) in def.equations.iter().enumerate() {
            if equation.head.as_ref() != def.name.as_ref() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "case head '{}' does not match type function '{}'",
                        equation.head, def.name
                    ),
                    equation.head_span,
                ));
            }
            if equation.patterns.len() != params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type function '{}' equation arity mismatch: expected {}, found {}",
                        def.name,
                        params.len(),
                        equation.patterns.len()
                    ),
                    equation.span,
                ));
            }
            let mut pattern_vars = HashMap::new();
            let patterns = equation
                .patterns
                .iter()
                .zip(&params)
                .map(|(pattern, param)| {
                    let constraint = constraint_for_param(param);
                    self.lower_type_function_pattern(pattern, &constraint, &mut pattern_vars)
                })
                .collect::<Result<Vec<_>, TypeEnvError>>()?;
            let result_context = TypeFunctionResultLoweringContext {
                pattern_vars: &pattern_vars,
                current_head: Some((&def.name, &head, &params, &result_constraint)),
                later_names,
            };
            let result = self.lower_type_function_result_expr(
                &equation.result,
                result_domain.as_ref(),
                &result_context,
                equation.result_span,
            )?;
            self.validate_type_function_result_constraint(
                &result,
                &result_constraint,
                equation.result_span,
            )?;
            equations.push(TypeFunctionEquation {
                head: head.clone(),
                ordinal,
                patterns,
                result,
                source_anchor: span_anchor(equation.span, format!("type fn equation {ordinal}")),
                case_head_anchor: span_anchor(
                    equation.head_span,
                    format!("case head {}", equation.head),
                ),
            });
        }

        self.validate_type_function_pattern_coverage(
            def.name.as_ref(),
            &params,
            &equations,
            def.header_span,
        )?;

        self.validate_type_function_structural_recursion(
            def.name.as_ref(),
            &head,
            &params,
            def.decreases
                .as_ref()
                .map(|decreases| decreases.param.as_ref()),
            &equations,
            def.header_span,
        )?;

        let lowered = TypeFunctionDef {
            visibility: core_visibility_from_surface(&def.visibility),
            head,
            name: def.name.to_string(),
            params,
            return_type,
            return_kind: Kind::Type,
            result_constraint,
            decreases: def
                .decreases
                .as_ref()
                .map(|decreases| decreases.param.to_string()),
            source_anchors: TypeFunctionSourceAnchors {
                definition: span_anchor(def.header_span, format!("type fn {}", def.name)),
                decreases: def.decreases.as_ref().map(|decreases| {
                    span_anchor(decreases.span, format!("decreases {}", decreases.param))
                }),
            },
            equations,
        };
        if lowered.visibility == ash_core::ast::Visibility::Public {
            self.validate_public_type_function_export_closure(&lowered, def.span)?;
        }
        Ok(lowered)
    }

    fn validate_public_type_function_export_closure(
        &self,
        def: &TypeFunctionDef,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        for equation in &def.equations {
            for pattern in &equation.patterns {
                self.validate_public_type_function_pattern_export_closure(def, pattern, span)?;
            }
            self.validate_public_type_function_result_export_closure(def, &equation.result, span)?;
        }
        for param in &def.params {
            if let Some(domain) = &param.domain_constraint {
                self.ensure_public_type_function_domain_dependency(def, domain, span)?;
            }
            self.validate_public_canonical_type_dependency(def, &param.ty, span)?;
        }
        if let TypeFunctionResultConstraint::Domain(domain) = &def.result_constraint {
            self.ensure_public_type_function_domain_dependency(def, domain, span)?;
        }
        self.validate_public_canonical_type_dependency(def, &def.return_type, span)
    }

    fn validate_public_type_function_pattern_export_closure(
        &self,
        def: &TypeFunctionDef,
        pattern: &TypeFunctionPattern,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match pattern {
            TypeFunctionPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                ..
            } => {
                self.ensure_public_type_function_constructor_dependency(def, constructor, span)?;
                self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                for field in fields {
                    self.validate_public_type_function_pattern_export_closure(def, field, span)?;
                }
                Ok(())
            }
            TypeFunctionPattern::Var { constraint, .. }
            | TypeFunctionPattern::Wildcard { constraint, .. } => {
                if let TypeFunctionPatternConstraint::Domain(domain) = constraint {
                    self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                }
                Ok(())
            }
        }
    }

    fn validate_public_type_function_result_export_closure(
        &self,
        def: &TypeFunctionDef,
        expr: &TypeFunctionResultExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::Primitive { .. } => Ok(()),
            TypeFunctionResultExpr::Var { constraint, .. } => {
                if let TypeFunctionResultConstraint::Domain(domain) = constraint {
                    self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::NominalApp {
                visible_name, args, ..
            } => {
                self.ensure_public_type_function_ordinary_type_dependency(def, visible_name, span)?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::DomainConstructorApp {
                constructor,
                domain,
                args,
                ..
            } => {
                self.ensure_public_type_function_constructor_dependency(def, constructor, span)?;
                self.ensure_public_type_function_domain_dependency(def, domain, span)?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::PromotedDataConstructorApp {
                constructor,
                data_kind,
                args,
                kind,
                ..
            } => {
                self.validate_registered_promoted_constructor_app(
                    constructor,
                    data_kind,
                    args.len(),
                    kind,
                    span,
                )?;
                self.ensure_public_type_function_promoted_constructor_dependency(
                    def,
                    constructor,
                    data_kind,
                    span,
                )?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                self.ensure_public_type_function_projection_dependency(
                    def, interface, member, span,
                )?;
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
                if head != &def.head {
                    let Some(callee) = self.local_type_functions.get(head) else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "public type function '{}' export closure cannot resolve type function dependency '{}'",
                                def.name, head.name
                            ),
                            span,
                        ));
                    };
                    if callee.visibility != ash_core::ast::Visibility::Public {
                        return Err(TypeEnvError::PrivateDependencyExportFailure {
                            public_item: def.name.clone(),
                            dependency: callee.name.clone(),
                            dependency_kind: "type function".to_string(),
                            span,
                        });
                    }
                }
                for arg in args {
                    self.validate_public_type_function_result_export_closure(def, arg, span)?;
                }
                Ok(())
            }
        }
    }

    fn validate_public_canonical_type_dependency(
        &self,
        def: &TypeFunctionDef,
        ty: &CanonicalTypeExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match ty {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp {
                visible_name, args, ..
            } => {
                self.ensure_public_type_function_ordinary_type_dependency(def, visible_name, span)?;
                for arg in args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                self.ensure_public_type_function_projection_dependency(
                    def, interface, member, span,
                )?;
                for arg in args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::ComputationHeadApp { args, .. } => {
                for arg in args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    span,
                )?;
                self.ensure_public_type_function_promoted_constructor_dependency(
                    def,
                    &app.constructor,
                    &app.data_kind,
                    span,
                )?;
                for arg in &app.args {
                    self.validate_public_canonical_type_dependency(def, arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' cannot export constructor-variable application '{}' until TASK-907 tracks constructor variables",
                    def.name, app.constructor.name
                ),
                span,
            )),
        }
    }

    fn ensure_public_type_function_domain_dependency(
        &self,
        def: &TypeFunctionDef,
        domain: &SealedDomainId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let Some(summary) = self.lookup_sealed_domain_by_id(domain) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve sealed domain '{}'",
                    def.name, domain.name
                ),
                span,
            ));
        };
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: summary.exported_name.clone(),
                dependency_kind: "sealed domain".to_string(),
                span: if anchor_span(&summary.anchor) == Span::default() {
                    span
                } else {
                    anchor_span(&summary.anchor)
                },
            });
        }
        Ok(())
    }

    fn ensure_public_type_function_constructor_dependency(
        &self,
        def: &TypeFunctionDef,
        constructor: &DomainConstructorId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let Some(domain) = self.lookup_sealed_domain_by_id(&constructor.domain) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve marker constructor '{}'",
                    def.name, constructor.name
                ),
                span,
            ));
        };
        if domain.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: constructor.name.clone(),
                dependency_kind: "marker constructor".to_string(),
                span,
            });
        }
        Ok(())
    }

    fn ensure_public_type_function_promoted_data_kind_dependency(
        &self,
        def: &TypeFunctionDef,
        data_kind: &PromotedDataKindId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let Some(summary) = self.lookup_promoted_data_kind_by_id(data_kind) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve promoted data kind '{}'",
                    def.name, data_kind.name
                ),
                span,
            ));
        };
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: summary.exported_name.clone(),
                dependency_kind: "promoted data kind".to_string(),
                span: if anchor_span(&summary.source_anchor) == Span::default() {
                    span
                } else {
                    anchor_span(&summary.source_anchor)
                },
            });
        }
        self.ensure_public_type_function_ordinary_type_dependency(
            def,
            &data_kind.source_type.name,
            span,
        )?;
        Ok(())
    }

    fn ensure_public_type_function_promoted_constructor_dependency(
        &self,
        def: &TypeFunctionDef,
        constructor: &PromotedConstructorId,
        data_kind: &PromotedDataKindId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        self.ensure_public_type_function_promoted_data_kind_dependency(def, data_kind, span)?;
        let Some(summary) = self.promoted_constructor_summaries.get(constructor) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve promoted data constructor '{}'",
                    def.name, constructor.name
                ),
                span,
            ));
        };
        if constructor.kind != *data_kind {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure references promoted constructor '{}' from promoted data kind '{}', not '{}'",
                    def.name, constructor.name, constructor.kind.name, data_kind.name
                ),
                span,
            ));
        }
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: summary.exported_name.clone(),
                dependency_kind: "promoted data constructor".to_string(),
                span: if anchor_span(&summary.source_anchor) == Span::default() {
                    span
                } else {
                    anchor_span(&summary.source_anchor)
                },
            });
        }
        Ok(())
    }

    fn ensure_public_type_function_projection_dependency(
        &self,
        def: &TypeFunctionDef,
        interface: &InterfaceIdentityId,
        member: &AssociatedMemberIdentityId,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        if !self.known_interface_identities.contains(interface) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve projection interface '{}'",
                    def.name, interface.name
                ),
                span,
            ));
        }
        if !self.known_associated_member_identities.contains(member)
            || member.interface != *interface
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' export closure cannot resolve projection member '{}::{}'",
                    def.name, interface.name, member.name
                ),
                span,
            ));
        }
        if let Some(info) = self.interfaces.get(interface.name.as_str())
            && info.visibility != ash_core::ast::Visibility::Public
        {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: format!("{}::{}", interface.name, member.name),
                dependency_kind: "projection".to_string(),
                span,
            });
        }
        Ok(())
    }

    fn ensure_public_type_function_ordinary_type_dependency(
        &self,
        def: &TypeFunctionDef,
        visible_name: &str,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        if let Some(type_def) = self.ast_types.get(visible_name)
            && type_def.visibility != ash_core::ast::Visibility::Public
        {
            return Err(TypeEnvError::PrivateDependencyExportFailure {
                public_item: def.name.clone(),
                dependency: visible_name.to_string(),
                dependency_kind: "ordinary type".to_string(),
                span,
            });
        }
        Ok(())
    }

    fn lower_type_fn_signature_type(
        &self,
        ty: &SurfaceType,
    ) -> Result<(CanonicalTypeExpr, Option<SealedDomainId>), TypeEnvError> {
        if let SurfaceType::Name(name) = ty {
            if name.as_ref() == "Type" {
                return Ok((CanonicalTypeExpr::Var("Type".to_string()), None));
            }
            if let Some(domain) = self.lookup_sealed_domain(name.as_ref()) {
                return Ok((
                    CanonicalTypeExpr::Var(domain.exported_name.clone()),
                    Some(domain.id.clone()),
                ));
            }
        }
        let canonical = self.lower_surface_type_to_canonical(ty).map_err(|err| {
            let spelling =
                surface_type_name(ty).unwrap_or_else(|| surface_projection_base_spelling(ty));
            TypeEnvError::InvalidDefinition(
                format!("unresolved type in type-function signature '{spelling}': {err}"),
                Span::default(),
            )
        })?;
        if matches!(canonical, CanonicalTypeExpr::Var(_)) {
            let spelling =
                surface_type_name(ty).unwrap_or_else(|| surface_projection_base_spelling(ty));
            return Err(TypeEnvError::InvalidDefinition(
                format!("unresolved type in type-function signature '{spelling}'"),
                Span::default(),
            ));
        }
        Ok((canonical, None))
    }

    fn validate_type_function_pattern_coverage(
        &self,
        name: &str,
        params: &[TypeFunctionParam],
        equations: &[TypeFunctionEquation],
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let sealed_positions = params
            .iter()
            .enumerate()
            .filter_map(|(index, param)| {
                param
                    .domain_constraint
                    .clone()
                    .map(|domain| (index, domain))
            })
            .collect::<Vec<_>>();
        if sealed_positions.is_empty() {
            return Ok(());
        }

        let spaces = sealed_positions
            .iter()
            .map(|(param_index, domain)| {
                self.coverage_space_for_domain(
                    domain,
                    equations
                        .iter()
                        .filter_map(|equation| equation.patterns.get(*param_index)),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let universe = Self::coverage_tuple_universe(&spaces);
        let mut covered = HashSet::new();
        let mut covered_by_default = HashSet::new();

        for equation in equations {
            let row_patterns = sealed_positions
                .iter()
                .map(|(index, _)| &equation.patterns[*index])
                .collect::<Vec<_>>();
            let row_space = universe
                .iter()
                .filter(|tuple| {
                    tuple.iter().zip(&row_patterns).all(|(value, pattern)| {
                        Self::coverage_value_matches_pattern(value, pattern)
                    })
                })
                .cloned()
                .collect::<HashSet<_>>();
            let residual = row_space
                .difference(&covered)
                .cloned()
                .collect::<HashSet<_>>();
            let has_default = row_patterns
                .iter()
                .any(|pattern| Self::pattern_has_domain_default(pattern));
            let is_all_default = row_patterns
                .iter()
                .all(|pattern| Self::pattern_is_all_domain_default(pattern));
            if residual.is_empty() {
                let message = if has_default && is_all_default {
                    format!(
                        "empty residual default in type function '{name}' equation {}",
                        equation.ordinal
                    )
                } else if row_space
                    .iter()
                    .any(|value| covered_by_default.contains(value))
                {
                    format!(
                        "unreachable type function equation {} in '{name}' after earlier default",
                        equation.ordinal
                    )
                } else {
                    format!(
                        "overlapping type function equation {} in '{name}'",
                        equation.ordinal
                    )
                };
                return Err(TypeEnvError::InvalidDefinition(message, span));
            }
            if has_default {
                covered_by_default.extend(residual.iter().cloned());
            }
            covered.extend(residual);
        }

        if covered.len() != universe.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "non-exhaustive type function '{name}': uncovered closed constructor tuple(s)"
                ),
                span,
            ));
        }
        Ok(())
    }

    fn validate_type_function_structural_recursion(
        &self,
        name: &str,
        head: &TypeComputationHeadId,
        params: &[TypeFunctionParam],
        decreases: Option<&str>,
        equations: &[TypeFunctionEquation],
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let recursive = equations
            .iter()
            .any(|equation| Self::result_contains_computation_head(&equation.result, head));

        let Some(decreases) = decreases else {
            if recursive {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("missing decreases clause for recursive type function '{name}'"),
                    span,
                ));
            }
            return Ok(());
        };

        let Some(decreasing_index) = params.iter().position(|param| param.name == decreases) else {
            return Err(TypeEnvError::InvalidDefinition(
                format!("unknown decreases parameter '{decreases}' in type function '{name}'"),
                span,
            ));
        };

        let Some(decreasing_domain) = params[decreasing_index].domain_constraint.as_ref() else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in type function '{name}': parameter is not a sealed domain"
                ),
                span,
            ));
        };

        if !self.domain_has_structural_subcomponent_metadata(decreasing_domain)? {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in type function '{name}': sealed domain has no structural subcomponent metadata"
                ),
                span,
            ));
        }

        for equation in equations {
            let allowed_subcomponents = equation
                .patterns
                .get(decreasing_index)
                .map(|pattern| self.direct_structural_subcomponent_vars(pattern))
                .transpose()?
                .unwrap_or_default();
            self.validate_recursive_calls_in_result(
                name,
                head,
                decreasing_index,
                &allowed_subcomponents,
                &equation.result,
                span,
            )?;
        }

        Ok(())
    }

    fn domain_has_structural_subcomponent_metadata(
        &self,
        domain: &SealedDomainId,
    ) -> Result<bool, TypeEnvError> {
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "unknown sealed domain '{}' in decreases clause",
                    domain.name
                ),
                Span::default(),
            )
        })?;
        Ok(summary.constructors.iter().any(|constructor| {
            constructor
                .fields
                .iter()
                .any(|field| field.structural_status == StructuralFieldStatus::StructuralSelfDomain)
        }))
    }

    fn direct_structural_subcomponent_vars(
        &self,
        pattern: &TypeFunctionPattern,
    ) -> Result<HashSet<String>, TypeEnvError> {
        let TypeFunctionPattern::DomainConstructor {
            constructor,
            domain,
            fields,
            ..
        } = pattern
        else {
            return Ok(HashSet::new());
        };
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "unknown sealed domain '{}' in recursion matrix",
                    domain.name
                ),
                Span::default(),
            )
        })?;
        let Some(constructor_summary) = summary
            .constructors
            .iter()
            .find(|candidate| candidate.id == **constructor)
        else {
            return Ok(HashSet::new());
        };

        let mut vars = HashSet::new();
        for (field_pattern, field) in fields.iter().zip(&constructor_summary.fields) {
            if field.structural_status != StructuralFieldStatus::StructuralSelfDomain {
                continue;
            }
            if let TypeFunctionPattern::Var { name, .. } = field_pattern {
                vars.insert(name.clone());
            }
        }
        Ok(vars)
    }

    fn validate_recursive_calls_in_result(
        &self,
        function_name: &str,
        self_head: &TypeComputationHeadId,
        decreasing_index: usize,
        allowed_subcomponents: &HashSet<String>,
        expr: &TypeFunctionResultExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            TypeFunctionResultExpr::Primitive { .. } | TypeFunctionResultExpr::Var { .. } => Ok(()),
            TypeFunctionResultExpr::NominalApp { args, .. }
            | TypeFunctionResultExpr::DomainConstructorApp { args, .. }
            | TypeFunctionResultExpr::PromotedDataConstructorApp { args, .. }
            | TypeFunctionResultExpr::Projection { args, .. } => {
                for arg in args {
                    self.validate_recursive_calls_in_result(
                        function_name,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
                for arg in args {
                    self.validate_recursive_calls_in_result(
                        function_name,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                if head == self_head {
                    let Some(decreasing_arg) = args.get(decreasing_index) else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in type function '{function_name}': missing decreasing argument"
                            ),
                            span,
                        ));
                    };
                    match decreasing_arg {
                        TypeFunctionResultExpr::Var { name, .. }
                            if allowed_subcomponents.contains(name) =>
                        {
                            Ok(())
                        }
                        _ => Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in type function '{function_name}': decreasing argument must be a direct structural subcomponent"
                            ),
                            span,
                        )),
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    fn result_contains_computation_head(
        expr: &TypeFunctionResultExpr,
        needle: &TypeComputationHeadId,
    ) -> bool {
        match expr {
            TypeFunctionResultExpr::Primitive { .. } | TypeFunctionResultExpr::Var { .. } => false,
            TypeFunctionResultExpr::NominalApp { args, .. }
            | TypeFunctionResultExpr::DomainConstructorApp { args, .. }
            | TypeFunctionResultExpr::PromotedDataConstructorApp { args, .. }
            | TypeFunctionResultExpr::Projection { args, .. } => args
                .iter()
                .any(|arg| Self::result_contains_computation_head(arg, needle)),
            TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
                head == needle
                    || args
                        .iter()
                        .any(|arg| Self::result_contains_computation_head(arg, needle))
            }
        }
    }

    fn coverage_space_for_domain<'a>(
        &self,
        domain: &SealedDomainId,
        patterns: impl Iterator<Item = &'a TypeFunctionPattern>,
    ) -> Result<TypeFunctionCoverageSpace, TypeEnvError> {
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!("unknown sealed domain '{}' in coverage matrix", domain.name),
                Span::default(),
            )
        })?;
        let mut inspected: HashMap<(DomainConstructorId, usize), Vec<&TypeFunctionPattern>> =
            HashMap::new();
        for pattern in patterns {
            self.collect_coverage_inspections(pattern, &mut inspected)?;
        }
        let mut alts = Vec::with_capacity(summary.constructors.len());
        for constructor in &summary.constructors {
            let mut fields = Vec::with_capacity(constructor.fields.len());
            for (field_index, field) in constructor.fields.iter().enumerate() {
                if let Some(nested_patterns) = inspected.get(&(constructor.id.clone(), field_index))
                {
                    let nested_domain = field.domain_constraint.clone().ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "nested constructor pattern under '{}' field '{}' requires a sealed-domain field",
                                constructor.exported_name, field.name
                            ),
                            Span::default(),
                        )
                    })?;
                    fields.push(Some(self.coverage_space_for_domain(
                        &nested_domain,
                        nested_patterns.iter().copied(),
                    )?));
                } else {
                    fields.push(None);
                }
            }
            alts.push(TypeFunctionCoverageAlt {
                constructor: constructor.id.clone(),
                fields,
            });
        }
        Ok(TypeFunctionCoverageSpace {
            domain: domain.clone(),
            alts,
        })
    }

    fn collect_coverage_inspections<'a>(
        &self,
        pattern: &'a TypeFunctionPattern,
        inspected: &mut HashMap<(DomainConstructorId, usize), Vec<&'a TypeFunctionPattern>>,
    ) -> Result<(), TypeEnvError> {
        let TypeFunctionPattern::DomainConstructor {
            constructor,
            domain,
            fields,
            ..
        } = pattern
        else {
            return Ok(());
        };
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!("unknown sealed domain '{}' in coverage matrix", domain.name),
                Span::default(),
            )
        })?;
        let Some(constructor_summary) = summary
            .constructors
            .iter()
            .find(|candidate| candidate.id == **constructor)
        else {
            return Ok(());
        };
        for (field_index, field_pattern) in fields.iter().enumerate() {
            if matches!(field_pattern, TypeFunctionPattern::DomainConstructor { .. }) {
                inspected
                    .entry(((**constructor).clone(), field_index))
                    .or_default()
                    .push(field_pattern);
                let Some(field) = constructor_summary.fields.get(field_index) else {
                    continue;
                };
                if field.domain_constraint.is_none() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "nested constructor pattern under '{}' field '{}' requires a sealed-domain field",
                            constructor_summary.exported_name, field.name
                        ),
                        Span::default(),
                    ));
                }
                self.collect_coverage_inspections(field_pattern, inspected)?;
            }
        }
        Ok(())
    }

    fn coverage_tuple_universe(
        spaces: &[TypeFunctionCoverageSpace],
    ) -> HashSet<Vec<TypeFunctionCoverageValue>> {
        let mut tuples = vec![Vec::new()];
        for values in spaces.iter().map(Self::coverage_values_for_space) {
            let mut next = Vec::new();
            for prefix in &tuples {
                for value in &values {
                    let mut tuple = prefix.clone();
                    tuple.push(value.clone());
                    next.push(tuple);
                }
            }
            tuples = next;
        }
        tuples.into_iter().collect()
    }

    fn coverage_values_for_space(
        space: &TypeFunctionCoverageSpace,
    ) -> Vec<TypeFunctionCoverageValue> {
        let _ = &space.domain;
        let mut values = Vec::new();
        for alt in &space.alts {
            let mut field_values = vec![Vec::new()];
            for field_space in &alt.fields {
                if let Some(field_space) = field_space {
                    let nested_values = Self::coverage_values_for_space(field_space);
                    let mut next = Vec::new();
                    for prefix in &field_values {
                        for nested in &nested_values {
                            let mut fields = prefix.clone();
                            fields.push(Some(nested.clone()));
                            next.push(fields);
                        }
                    }
                    field_values = next;
                } else {
                    for prefix in &mut field_values {
                        prefix.push(None);
                    }
                }
            }
            values.extend(
                field_values
                    .into_iter()
                    .map(|fields| TypeFunctionCoverageValue {
                        constructor: alt.constructor.clone(),
                        fields,
                    }),
            );
        }
        values
    }

    fn coverage_value_matches_pattern(
        value: &TypeFunctionCoverageValue,
        pattern: &TypeFunctionPattern,
    ) -> bool {
        match pattern {
            TypeFunctionPattern::Wildcard { .. } | TypeFunctionPattern::Var { .. } => true,
            TypeFunctionPattern::DomainConstructor {
                constructor,
                fields,
                ..
            } => {
                constructor.as_ref() == &value.constructor
                    && fields.iter().enumerate().all(|(index, field_pattern)| {
                        match value.fields.get(index).and_then(Option::as_ref) {
                            Some(nested) => {
                                Self::coverage_value_matches_pattern(nested, field_pattern)
                            }
                            None => !matches!(
                                field_pattern,
                                TypeFunctionPattern::DomainConstructor { .. }
                            ),
                        }
                    })
            }
        }
    }

    fn pattern_has_domain_default(pattern: &TypeFunctionPattern) -> bool {
        match pattern {
            TypeFunctionPattern::Wildcard { constraint, .. }
            | TypeFunctionPattern::Var { constraint, .. } => {
                matches!(constraint, TypeFunctionPatternConstraint::Domain(_))
            }
            TypeFunctionPattern::DomainConstructor { fields, .. } => {
                fields.iter().any(Self::pattern_has_domain_default)
            }
        }
    }

    fn pattern_is_all_domain_default(pattern: &TypeFunctionPattern) -> bool {
        matches!(
            pattern,
            TypeFunctionPattern::Wildcard {
                constraint: TypeFunctionPatternConstraint::Domain(_),
                ..
            } | TypeFunctionPattern::Var {
                constraint: TypeFunctionPatternConstraint::Domain(_),
                ..
            }
        )
    }

    fn lower_type_function_pattern(
        &self,
        pattern: &SurfaceTypePattern,
        constraint: &TypeFunctionPatternConstraint,
        pattern_vars: &mut HashMap<String, TypeFunctionPatternConstraint>,
    ) -> Result<TypeFunctionPattern, TypeEnvError> {
        match pattern {
            SurfaceTypePattern::Wildcard { span } => Ok(TypeFunctionPattern::Wildcard {
                constraint: constraint.clone(),
                source_anchor: span_anchor(*span, "wildcard type pattern"),
            }),
            SurfaceTypePattern::Var { name, span } => {
                if let TypeFunctionPatternConstraint::Domain(domain_id) = constraint
                    && let Some((domain, constructor)) =
                        self.find_domain_constructor(domain_id, name.as_ref())
                {
                    return self.lower_domain_constructor_pattern(
                        constructor,
                        domain,
                        &[],
                        *span,
                        pattern_vars,
                    );
                }
                let name = name.to_string();
                if pattern_vars
                    .insert(name.clone(), constraint.clone())
                    .is_some()
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("repeated type pattern variable '{name}'"),
                        *span,
                    ));
                }
                Ok(TypeFunctionPattern::Var {
                    name,
                    constraint: constraint.clone(),
                    source_anchor: span_anchor(*span, "type pattern variable"),
                })
            }
            SurfaceTypePattern::Constructor { name, args, span } => {
                let TypeFunctionPatternConstraint::Domain(domain_id) = constraint else {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "constructor pattern '{}' requires a sealed-domain position",
                            name
                        ),
                        *span,
                    ));
                };
                let Some((domain, constructor)) =
                    self.find_domain_constructor(domain_id, name.as_ref())
                else {
                    if let Some((other_domain, _)) = self.find_any_domain_constructor(name.as_ref())
                    {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "marker constructor '{}' belongs to sealed domain '{}', not expected sealed domain '{}'",
                                name, other_domain.exported_name, domain_id.name
                            ),
                            *span,
                        ));
                    }
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "unknown marker constructor '{}' for sealed-domain pattern",
                            name
                        ),
                        *span,
                    ));
                };
                if self.visible_type_head_exists(name.as_ref())
                    || self.local_type_function_heads.contains_key(name.as_ref())
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "ambiguous marker constructor '{}' also resolves as a type-level head",
                            name
                        ),
                        *span,
                    ));
                }
                self.lower_domain_constructor_pattern(
                    constructor,
                    domain,
                    args,
                    *span,
                    pattern_vars,
                )
            }
        }
    }

    fn lower_domain_constructor_pattern(
        &self,
        constructor: &DomainConstructorSummary,
        domain: &SealedDomainSummary,
        args: &[SurfaceTypePattern],
        span: Span,
        pattern_vars: &mut HashMap<String, TypeFunctionPatternConstraint>,
    ) -> Result<TypeFunctionPattern, TypeEnvError> {
        if constructor.fields.len() != args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "marker constructor '{}' pattern arity mismatch: expected {}, found {}",
                    constructor.exported_name,
                    constructor.fields.len(),
                    args.len()
                ),
                span,
            ));
        }
        let fields = args
            .iter()
            .zip(&constructor.fields)
            .map(|(arg, field)| {
                let constraint = field
                    .domain_constraint
                    .clone()
                    .map(TypeFunctionPatternConstraint::Domain)
                    .unwrap_or_else(|| TypeFunctionPatternConstraint::Kind(field.kind.clone()));
                self.lower_type_function_pattern(arg, &constraint, pattern_vars)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TypeFunctionPattern::DomainConstructor {
            constructor: Box::new(constructor.id.clone()),
            domain: Box::new(domain.id.clone()),
            fields,
            constraint: TypeFunctionPatternConstraint::Domain(domain.id.clone()),
            source_anchor: span_anchor(
                span,
                format!("marker constructor pattern {}", constructor.exported_name),
            ),
        })
    }

    fn lower_type_function_result_expr(
        &self,
        ty: &SurfaceType,
        expected_domain: Option<&SealedDomainId>,
        context: &TypeFunctionResultLoweringContext<'_>,
        span: Span,
    ) -> Result<TypeFunctionResultExpr, TypeEnvError> {
        match ty {
            SurfaceType::Name(name) => self.lower_type_function_result_head(
                name.as_ref(),
                &[],
                expected_domain,
                context,
                span,
            ),
            SurfaceType::Constructor { name, args } => self.lower_type_function_result_head(
                name.as_ref(),
                args,
                expected_domain,
                context,
                span,
            ),
            other => self
                .lower_surface_type_to_canonical(other)
                .and_then(|canonical| {
                    type_function_result_from_canonical(canonical, span)
                        .map_err(|err| TypeError::TypeEnv(Box::new(err)))
                })
                .map_err(|err| {
                    TypeEnvError::InvalidDefinition(format!("result kind mismatch: {err}"), span)
                }),
        }
    }

    fn lower_type_function_result_head(
        &self,
        name: &str,
        args: &[SurfaceType],
        expected_domain: Option<&SealedDomainId>,
        context: &TypeFunctionResultLoweringContext<'_>,
        span: Span,
    ) -> Result<TypeFunctionResultExpr, TypeEnvError> {
        if args.is_empty() && context.pattern_vars.contains_key(name) {
            let constraint = context
                .pattern_vars
                .get(name)
                .expect("checked contains_key");
            return Ok(TypeFunctionResultExpr::Var {
                name: name.to_string(),
                kind: Kind::Type,
                constraint: result_constraint_from_pattern(constraint),
                source_anchor: span_anchor(span, format!("type pattern variable {name}")),
            });
        }
        if let Some(domain_id) = expected_domain {
            if let Some((domain, constructor)) = self.find_domain_constructor(domain_id, name) {
                let current_head_has_same_name = context
                    .current_head
                    .as_ref()
                    .is_some_and(|(self_name, _, _, _)| name == *self_name);
                if self.visible_type_head_exists(name)
                    || self.local_type_function_heads.contains_key(name)
                    || current_head_has_same_name
                {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "ambiguous marker constructor '{name}' also resolves as a type-level head"
                        ),
                        span,
                    ));
                }
                if constructor.fields.len() != args.len() {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "marker constructor '{}' result arity mismatch: expected {}, found {}",
                            constructor.exported_name,
                            constructor.fields.len(),
                            args.len()
                        ),
                        span,
                    ));
                }
                let mut lowered_args = Vec::with_capacity(args.len());
                for (index, (arg, field)) in args.iter().zip(&constructor.fields).enumerate() {
                    let lowered = self.lower_type_function_result_expr(
                        arg,
                        field.domain_constraint.as_ref(),
                        context,
                        span,
                    )?;
                    if let Some(expected_domain) = &field.domain_constraint {
                        match self.result_expr_constraint(&lowered) {
                            TypeFunctionResultConstraint::Domain(actual)
                                if actual == *expected_domain => {}
                            found => {
                                return Err(TypeEnvError::InvalidDefinition(
                                    format!(
                                        "result constructor field {index} domain mismatch: expected sealed domain '{}', found {:?}",
                                        expected_domain.name, found
                                    ),
                                    span,
                                ));
                            }
                        }
                    }
                    lowered_args.push(lowered);
                }
                return Ok(TypeFunctionResultExpr::DomainConstructorApp {
                    constructor: constructor.id.clone(),
                    domain: domain.id.clone(),
                    args: lowered_args,
                    kind: Kind::Type,
                    constraint: TypeFunctionResultConstraint::Domain(domain.id.clone()),
                    source_anchor: span_anchor(span, format!("marker constructor result {name}")),
                });
            }
            if let Some((other_domain, _)) = self.find_any_domain_constructor(name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "marker constructor '{name}' belongs to sealed domain '{}', not expected sealed domain '{}'",
                        other_domain.exported_name, domain_id.name
                    ),
                    span,
                ));
            }
        }
        if let Some((_, head, params, result_constraint)) = context
            .current_head
            .filter(|(self_name, _, _, _)| name == *self_name)
        {
            if self.visible_type_head_exists(name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("ambiguous type-function/type head '{name}'"),
                    span,
                ));
            }
            if params.len() != args.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type function '{name}' application arity mismatch: expected {}, found {}",
                        params.len(),
                        args.len()
                    ),
                    span,
                ));
            }
            let lowered_args = args
                .iter()
                .zip(params)
                .map(|(arg, param)| {
                    self.lower_type_function_result_expr(
                        arg,
                        param.domain_constraint.as_ref(),
                        context,
                        span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            self.validate_type_function_application_args(name, &lowered_args, params, span)?;
            return Ok(TypeFunctionResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: lowered_args,
                kind: Kind::Type,
                constraint: result_constraint.clone(),
                source_anchor: span_anchor(span, format!("type function call {name}")),
            });
        }
        if let Some(head) = self.local_type_function_heads.get(name) {
            if self.visible_type_head_exists(name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("ambiguous type-function/type head '{name}'"),
                    span,
                ));
            }
            let callee = self.local_type_functions.get(head).ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!("unresolved type function or type head '{name}'"),
                    span,
                )
            })?;
            if callee.params.len() != args.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "type function '{name}' application arity mismatch: expected {}, found {}",
                        callee.params.len(),
                        args.len()
                    ),
                    span,
                ));
            }
            let lowered_args = args
                .iter()
                .zip(&callee.params)
                .map(|(arg, param)| {
                    self.lower_type_function_result_expr(
                        arg,
                        param.domain_constraint.as_ref(),
                        context,
                        span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            self.validate_type_function_application_args(
                name,
                &lowered_args,
                &callee.params,
                span,
            )?;
            return Ok(TypeFunctionResultExpr::ComputationHeadApp {
                head: head.clone(),
                args: lowered_args,
                kind: Kind::Type,
                constraint: callee.result_constraint.clone(),
                source_anchor: span_anchor(span, format!("type function call {name}")),
            });
        }
        if context.later_names.contains(name) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "forward reference to later type function '{name}' is unsupported in SPEC-E"
                ),
                span,
            ));
        }
        if args.is_empty()
            && matches!(
                name,
                "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref"
            )
        {
            return Ok(TypeFunctionResultExpr::Primitive {
                name: name.to_string(),
                kind: Kind::Type,
                constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                source_anchor: span_anchor(span, format!("primitive type {name}")),
            });
        }
        if args.is_empty() && name.chars().next().is_some_and(char::is_lowercase) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("unknown RHS type variable '{name}'"),
                span,
            ));
        }
        let surface = if args.is_empty() {
            SurfaceType::Name(Box::from(name))
        } else {
            SurfaceType::Constructor {
                name: Box::from(name),
                args: args.to_vec(),
            }
        };
        self.lower_surface_type_to_canonical(&surface)
            .and_then(|canonical| {
                type_function_result_from_canonical(canonical, span)
                    .map_err(|err| TypeError::TypeEnv(Box::new(err)))
            })
            .map_err(|_| {
                let prefix =
                    if name.chars().next().is_some_and(char::is_uppercase) && args.is_empty() {
                        "result kind mismatch: "
                    } else {
                        ""
                    };
                TypeEnvError::InvalidDefinition(
                    format!("{prefix}unresolved type function or type head '{name}'"),
                    span,
                )
            })
    }

    fn visible_type_head_exists(&self, name: &str) -> bool {
        self.ast_types.contains_key(name) || self.type_alias_identities.contains_key(name)
    }

    fn result_expr_constraint(
        &self,
        expr: &TypeFunctionResultExpr,
    ) -> TypeFunctionResultConstraint {
        match expr {
            TypeFunctionResultExpr::Primitive { constraint, .. }
            | TypeFunctionResultExpr::Var { constraint, .. }
            | TypeFunctionResultExpr::NominalApp { constraint, .. }
            | TypeFunctionResultExpr::DomainConstructorApp { constraint, .. }
            | TypeFunctionResultExpr::PromotedDataConstructorApp { constraint, .. }
            | TypeFunctionResultExpr::Projection { constraint, .. }
            | TypeFunctionResultExpr::ComputationHeadApp { constraint, .. } => constraint.clone(),
        }
    }

    fn validate_type_function_result_constraint(
        &self,
        expr: &TypeFunctionResultExpr,
        expected: &TypeFunctionResultConstraint,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        let actual = self.result_expr_constraint(expr);
        match (expected, actual) {
            (
                TypeFunctionResultConstraint::Domain(expected_domain),
                TypeFunctionResultConstraint::Domain(actual_domain),
            ) if expected_domain == &actual_domain => Ok(()),
            (TypeFunctionResultConstraint::Domain(expected_domain), found) => {
                Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "result domain mismatch: expected sealed domain '{}', found {:?}",
                        expected_domain.name, found
                    ),
                    span,
                ))
            }
            (TypeFunctionResultConstraint::Kind(_), _) => Ok(()),
        }
    }

    fn validate_type_function_application_args(
        &self,
        name: &str,
        args: &[TypeFunctionResultExpr],
        params: &[TypeFunctionParam],
        span: Span,
    ) -> Result<(), TypeEnvError> {
        for (index, (arg, param)) in args.iter().zip(params).enumerate() {
            if let Some(expected_domain) = &param.domain_constraint {
                match self.result_expr_constraint(arg) {
                    TypeFunctionResultConstraint::Domain(actual) if actual == *expected_domain => {}
                    found => {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "type function '{name}' argument {index} domain mismatch: expected sealed domain '{}', found {:?}",
                                expected_domain.name, found
                            ),
                            span,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn find_domain_constructor(
        &self,
        domain_id: &SealedDomainId,
        constructor_name: &str,
    ) -> Option<(&SealedDomainSummary, &DomainConstructorSummary)> {
        let domain = self.lookup_sealed_domain_by_id(domain_id)?;
        let constructor = domain
            .constructors
            .iter()
            .find(|constructor| constructor.exported_name == constructor_name)?;
        Some((domain, constructor))
    }

    fn find_any_domain_constructor(
        &self,
        constructor_name: &str,
    ) -> Option<(&SealedDomainSummary, &DomainConstructorSummary)> {
        self.sealed_domain_summaries.values().find_map(|domain| {
            domain
                .constructors
                .iter()
                .find(|constructor| constructor.exported_name == constructor_name)
                .map(|constructor| (domain, constructor))
        })
    }

    /// Look up a sealed domain by its canonical identity.
    #[must_use]
    pub fn lookup_sealed_domain_by_id(&self, id: &SealedDomainId) -> Option<&SealedDomainSummary> {
        self.sealed_domain_summaries.get(id)
    }

    /// Iterate over all visible sealed-domain exported names.
    pub fn sealed_domain_names(&self) -> impl Iterator<Item = &str> {
        self.sealed_domain_aliases.keys().map(String::as_str)
    }

    /// Register an interface identity summary in the canonical Phase 110 registry.
    pub fn register_interface_identity_summary(
        &mut self,
        summary: &InterfaceIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_interface_identity_summary_with_provenance(summary, false)
    }

    fn register_interface_identity_summary_imported(
        &mut self,
        summary: &InterfaceIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_interface_identity_summary_with_provenance(summary, true)
    }

    fn register_interface_identity_summary_with_provenance(
        &mut self,
        summary: &InterfaceIdentitySummary,
        imported: bool,
    ) -> Result<(), TypeEnvError> {
        self.known_interface_identities.insert(summary.id.clone());
        self.canonical_interface_names
            .insert(summary.id.clone(), summary.name.to_string());

        let visible_name = summary.name.as_str();
        if let Some(existing) = self.interface_identity_aliases.get(visible_name)
            && existing != &summary.id
        {
            let existing_is_imported = self
                .interface_identity_alias_is_imported
                .get(visible_name)
                .copied()
                .unwrap_or(false);
            if imported || !existing_is_imported {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "conflicting visible interface alias '{}': {:?} vs {:?}",
                        summary.name, existing, summary.id
                    ),
                    Span::default(),
                ));
            }
        }

        self.interface_identity_aliases
            .insert(summary.name.to_string(), summary.id.clone());
        self.interface_identity_alias_is_imported
            .insert(summary.name.to_string(), imported);
        if !imported {
            let Some(interface) = self.interfaces.get(summary.name.as_str()) else {
                return Ok(());
            };
            self.local_interface_arities
                .insert(summary.id.clone(), interface.type_params.len());
        }
        Ok(())
    }

    /// Register an associated-member identity summary in the canonical Phase 110 registry.
    pub fn register_associated_member_identity_summary(
        &mut self,
        summary: &AssociatedMemberIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_associated_member_identity_summary_with_provenance(summary, false)
    }

    fn register_associated_member_identity_summary_imported(
        &mut self,
        summary: &AssociatedMemberIdentitySummary,
    ) -> Result<(), TypeEnvError> {
        self.register_associated_member_identity_summary_with_provenance(summary, true)
    }

    fn register_associated_member_identity_summary_with_provenance(
        &mut self,
        summary: &AssociatedMemberIdentitySummary,
        imported: bool,
    ) -> Result<(), TypeEnvError> {
        self.known_associated_member_identities
            .insert(summary.id.clone());
        let alias_key = (
            summary.id.interface.name.to_string(),
            summary.name.to_string(),
        );
        if let Some(existing) = self.associated_member_identity_aliases.get(&alias_key)
            && existing != &summary.id
        {
            let existing_is_imported = self
                .associated_member_identity_alias_is_imported
                .get(&alias_key)
                .copied()
                .unwrap_or(false);
            if imported || !existing_is_imported {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "conflicting visible associated-member alias '{}::{}': {:?} vs {:?}",
                        alias_key.0, alias_key.1, existing, summary.id
                    ),
                    Span::default(),
                ));
            }
        }
        self.associated_member_identity_aliases
            .insert(alias_key.clone(), summary.id.clone());
        self.associated_member_identity_alias_is_imported
            .insert(alias_key, imported);
        Ok(())
    }

    fn lower_associated_projection_to_canonical(
        &self,
        base: &CanonicalTypeExpr,
        member_name: &str,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        let projection_spelling = format!(
            "{}::{}",
            canonical_projection_base_spelling(base),
            member_name
        );
        let (base_name, projection_args, rigidity) = match base {
            CanonicalTypeExpr::Var(name) => (
                name.clone(),
                vec![CanonicalTypeExpr::Var(name.clone())],
                ProjectionRigidity::Neutral,
            ),
            CanonicalTypeExpr::NominalApp {
                visible_name, args, ..
            } => (
                visible_name.clone(),
                args.clone(),
                projection_rigidity_for_canonical_args(args),
            ),
            CanonicalTypeExpr::Projection { .. } => {
                return Err(TypeError::ConstructorNameMismatch {
                    expected: "supported associated projection base (nested projection bases are unsupported)"
                        .to_string(),
                    found: format!("nested projection base {projection_spelling}"),
                    span: Span::default(),
                });
            }
            _ => {
                return Err(TypeError::ConstructorNameMismatch {
                    expected:
                        "supported associated projection base (type variable or nominal application)"
                            .to_string(),
                    found: format!("unsupported projection base {projection_spelling}"),
                    span: Span::default(),
                });
            }
        };

        let interface = self
            .interface_identity_for_name(&base_name)
            .cloned()
            .or_else(|| {
                self.interfaces.iter().find_map(|(iface_name, iface_info)| {
                    iface_info
                        .associated_types
                        .contains(&member_name.to_string())
                        .then(|| self.interface_identity_for_name(iface_name).cloned())
                        .flatten()
                })
            })
            .or_else(|| {
                let mut matches = self
                    .known_associated_member_identities
                    .iter()
                    .filter(|id| id.name == member_name)
                    .map(|id| id.interface.clone());
                let first = matches.next()?;
                matches.all(|candidate| candidate == first).then_some(first)
            })
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: "registered associated projection".to_string(),
                found: format!("{base_name}::{member_name}"),
                span: Span::default(),
            })?;

        let member = self
            .associated_member_identity_for_interface_member(&interface.name, member_name)
            .cloned()
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: format!("registered member on interface {}", interface.name),
                found: projection_spelling.clone(),
                span: Span::default(),
            })?;

        let expected_arity = self
            .local_interface_arities
            .get(&interface)
            .copied()
            .unwrap_or(projection_args.len());
        if expected_arity != projection_args.len() {
            return Err(TypeError::ConstructorArityMismatch {
                name: format!("{} for projection {}", interface.name, projection_spelling),
                expected_arity,
                found_arity: projection_args.len(),
                span: Span::default(),
            });
        }

        let rigidity = if self
            .lookup_associated_family_declaration(&interface.name, member_name)
            .is_some()
        {
            rigidity
        } else if matches!(base, CanonicalTypeExpr::NominalApp { .. }) {
            ProjectionRigidity::Rigid
        } else {
            rigidity
        };

        Ok(CanonicalTypeExpr::Projection {
            interface,
            member,
            args: projection_args,
            kind: Kind::Type,
            rigidity,
        })
    }

    fn lower_explicit_associated_family_projection_to_canonical(
        &self,
        interface_name: &str,
        args: &[SurfaceType],
        member_name: &str,
        span: Span,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        let declaration = self
            .lookup_associated_family_declaration(interface_name, member_name)
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: "registered sealed associated-family projection".to_string(),
                found: format!("<{interface_name}<...>>::{member_name}"),
                span,
            })?;

        if declaration.interface_params.len() != args.len() {
            return Err(TypeError::ConstructorArityMismatch {
                name: format!("{}::{}", interface_name, member_name),
                expected_arity: declaration.interface_params.len(),
                found_arity: args.len(),
                span,
            });
        }

        let lowered_args = args
            .iter()
            .map(|arg| self.lower_surface_type_to_canonical(arg))
            .collect::<Result<Vec<_>, _>>()?;
        let rigidity = projection_rigidity_for_canonical_args(&lowered_args);

        Ok(CanonicalTypeExpr::Projection {
            interface: declaration.head.interface.clone(),
            member: declaration.head.member.clone(),
            args: lowered_args,
            kind: Kind::Type,
            rigidity,
        })
    }

    fn lower_associated_family_projection_result_expr(
        &self,
        interface_name: &str,
        args: &[SurfaceType],
        member_name: &str,
        expected_constraint: &AssociatedFamilyResultConstraint,
        var_constraints: &HashMap<String, AssociatedFamilyResultConstraint>,
        span: Span,
    ) -> Result<AssociatedFamilyResultExpr, TypeEnvError> {
        let declaration = self
            .lookup_associated_family_declaration(interface_name, member_name)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!("unknown sealed associated-family projection '<{interface_name}<...>>::{member_name}'"),
                    span,
                )
            })?;

        if declaration.interface_params.len() != args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated-family projection '{}::{}' expects {} interface arguments, found {}",
                    interface_name,
                    member_name,
                    declaration.interface_params.len(),
                    args.len()
                ),
                span,
            ));
        }

        let interface_args = args
            .iter()
            .zip(declaration.interface_params.iter())
            .map(|(arg, param)| {
                let constraint =
                    Self::associated_family_constraint_for_domain(param.domain_constraint.as_ref());
                self.lower_associated_family_result_expr(arg, &constraint, var_constraints, span)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = AssociatedFamilyResultExpr::AssociatedFamilyProjection {
            head: declaration.head.clone(),
            interface_args,
            kind: Kind::Type,
            constraint: declaration.result_domain.clone(),
            rigidity: projection_rigidity_for_associated_family_args(&[]),
            source_anchor: span_anchor(
                span,
                format!("associated family projection {interface_name}::{member_name}"),
            ),
        };
        let result = match result {
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                kind,
                constraint,
                source_anchor,
                ..
            } => {
                let rigidity = projection_rigidity_for_associated_family_args(&interface_args);
                AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                    head,
                    interface_args,
                    kind,
                    constraint,
                    rigidity,
                    source_anchor,
                }
            }
            _ => unreachable!("constructed as associated family projection"),
        };
        if !Self::associated_family_expr_conforms_to_constraint(&result, expected_constraint) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: format!("{}::{}", interface_name, member_name),
                reason: format!(
                    "projection result constraint '{}' does not conform to expected '{}'",
                    associated_family_result_constraint_label(&declaration.result_domain),
                    associated_family_result_constraint_label(expected_constraint)
                ),
                span,
            });
        }
        Ok(result)
    }

    fn canonical_type_identity_for_visible_name(
        &self,
        visible_name: &str,
    ) -> Result<TypeDeclId, TypeError> {
        self.type_identity_for_name(visible_name)
            .cloned()
            .ok_or_else(|| TypeError::ConstructorNameMismatch {
                expected: "registered canonical type identity".to_string(),
                found: visible_name.to_string(),
                span: Span::default(),
            })
    }

    /// Lower a core `TypeExpr` into the Phase 110 canonical type-expression substrate.
    pub fn lower_core_type_expr_to_canonical(
        &self,
        expr: &TypeExpr,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        match expr {
            TypeExpr::Named(name) => match name.as_str() {
                "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" => {
                    Ok(CanonicalTypeExpr::Primitive(name.clone()))
                }
                _ => {
                    if let Some(kind) = self.type_parameter_kind(name) {
                        if kind.is_type() {
                            Ok(CanonicalTypeExpr::Var(name.clone()))
                        } else {
                            Err(TypeError::from(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor variable '{name}' has kind {kind}; expected a fully applied proper type"
                                ),
                                Span::default(),
                            )))
                        }
                    } else {
                        match self.resolve_type(name) {
                            Ok((qualified, _)) => {
                                self.check_type_constructor_arity(&qualified, 0)?;
                                Ok(CanonicalTypeExpr::NominalApp {
                                    origin: self.canonical_type_identity_for_visible_name(name)?,
                                    visible_name: name.clone(),
                                    args: vec![],
                                    kind: Kind::Type,
                                })
                            }
                            Err(TypeError::UnboundVariable(_, _)) => {
                                Ok(CanonicalTypeExpr::Var(name.clone()))
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
            },
            TypeExpr::Constructor { name, args } => {
                if let Some(kind) = self.type_parameter_kind(name) {
                    if kind.is_type() {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "proper type variable '{name}' of kind * cannot be applied as a constructor"
                            ),
                            Span::default(),
                        )));
                    }
                    let expected_arity = kind.arity();
                    if args.len() != expected_arity {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "wrong arity for constructor variable '{name}': expected {expected_arity}, found {}",
                                args.len()
                            ),
                            Span::default(),
                        )));
                    }
                    let lowered_args = args
                        .iter()
                        .map(|arg| self.lower_core_type_expr_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(CanonicalTypeExpr::ConstructorVariableApp(Box::new(
                        ConstructorVariableApp::new(
                            ConstructorVariableRef::new(name.clone(), kind.clone(), None),
                            lowered_args,
                            Kind::Type,
                            None,
                        ),
                    )));
                }
                let (qualified, _) = self.resolve_type(name)?;
                self.check_type_constructor_arity(&qualified, args.len())?;
                Ok(CanonicalTypeExpr::NominalApp {
                    origin: self.canonical_type_identity_for_visible_name(name)?,
                    visible_name: name.clone(),
                    args: args
                        .iter()
                        .map(|arg| self.lower_core_type_expr_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?,
                    kind: Kind::Type,
                })
            }
            TypeExpr::Tuple(items) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Tuple({})", items.len()),
                span: Span::default(),
            }),
            TypeExpr::Record(fields) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Record({})", fields.len()),
                span: Span::default(),
            }),
            TypeExpr::Associated { base, name } => {
                if matches!(base.as_ref(), TypeExpr::Associated { .. }) {
                    return Err(TypeError::ConstructorNameMismatch {
                        expected: "supported associated projection base (nested projection bases are unsupported)"
                            .to_string(),
                        found: format!("nested projection base {base:?}"),
                        span: Span::default(),
                    });
                }
                if matches!(base.as_ref(), TypeExpr::Tuple(_) | TypeExpr::Record(_)) {
                    let found = match base.as_ref() {
                        TypeExpr::Tuple(items) => {
                            format!("unsupported projection base Tuple({})", items.len())
                        }
                        TypeExpr::Record(fields) => {
                            format!("unsupported projection base Record({})", fields.len())
                        }
                        _ => unreachable!("guarded by matches!"),
                    };
                    return Err(TypeError::ConstructorNameMismatch {
                        expected: "supported associated projection base (type variable or nominal application)"
                            .to_string(),
                        found,
                        span: Span::default(),
                    });
                }
                let lowered_base = self.lower_core_type_expr_to_canonical(base)?;
                self.lower_associated_projection_to_canonical(&lowered_base, name)
            }
        }
    }

    /// Lower a surface proposition tail into canonical proposition carriers without solving.
    pub fn register_proposition_predicate_decl(
        &mut self,
        decl: &PropositionPredicateDecl,
    ) -> Result<PropositionPredicateId, TypeError> {
        reject_constructor_kinded_proposition_params(
            &decl.params,
            "proposition predicate parameter",
            "TASK-908",
        )
        .map_err(TypeError::from)?;

        let module = self
            .current_module_identity
            .clone()
            .unwrap_or_else(synthetic_proposition_module_identity);
        let origin = proposition_module_source_origin(&module);
        let id = PropositionPredicateId::new(module, decl.name.to_string());
        let params = decl
            .params
            .iter()
            .map(|param| {
                let ty = self.lower_surface_type_to_canonical(&param.domain)?;
                Ok(PropositionPredicateParamSummary {
                    name: param.name.to_string(),
                    ty,
                    kind: Kind::Type,
                    source_anchor: proposition_source_anchor(
                        origin.clone(),
                        param.span,
                        format!("proposition predicate parameter {}", param.name),
                    ),
                })
            })
            .collect::<Result<Vec<_>, TypeError>>()?;
        let summary = PropositionPredicateSummary {
            id: id.clone(),
            exported_name: decl.name.to_string(),
            visibility: core_visibility_from_surface(&decl.visibility),
            params,
            source_anchor: proposition_source_anchor(
                origin,
                decl.span,
                format!("proposition predicate {}", decl.name),
            ),
        };
        self.register_proposition_predicate_summary_with_solver_kind(
            &summary,
            PropositionPredicateSolverKind::DeferredUnsupported,
        )?;
        Ok(id)
    }

    pub fn register_proposition_predicate_summary(
        &mut self,
        summary: &PropositionPredicateSummary,
    ) -> Result<(), TypeEnvError> {
        self.register_proposition_predicate_summary_with_solver_kind(
            summary,
            PropositionPredicateSolverKind::DeferredUnsupported,
        )
    }

    pub fn register_builtin_proposition_predicate_summary(
        &mut self,
        summary: &PropositionPredicateSummary,
    ) -> Result<(), TypeEnvError> {
        self.register_proposition_predicate_summary_with_solver_kind(
            summary,
            PropositionPredicateSolverKind::CompilerBuiltinSatisfied,
        )
    }

    fn register_proposition_predicate_summary_with_solver_kind(
        &mut self,
        summary: &PropositionPredicateSummary,
        solver_kind: PropositionPredicateSolverKind,
    ) -> Result<(), TypeEnvError> {
        self.validate_public_proposition_predicate_summary_dependencies(summary)?;
        let visible_name = summary.exported_name.to_string();
        if let Some(existing) = self.proposition_predicate_aliases.get(&visible_name)
            && existing != &summary.id
        {
            return Err(TypeEnvError::ImportOrderConflict {
                family: "proposition predicate visible name".to_string(),
                name: visible_name,
                span: anchor_span(&summary.source_anchor),
            });
        }
        if let Some(existing) = self.proposition_predicates.get(&summary.id) {
            if existing.summary != *summary || existing.solver_kind != solver_kind {
                return Err(TypeEnvError::ImportOrderConflict {
                    family: "proposition predicate summary".to_string(),
                    name: summary.exported_name.to_string(),
                    span: anchor_span(&summary.source_anchor),
                });
            }
            return Ok(());
        }
        self.proposition_predicate_aliases
            .insert(visible_name, summary.id.clone());
        self.proposition_predicates.insert(
            summary.id.clone(),
            PropositionPredicateInfo {
                summary: summary.clone(),
                solver_kind,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn lookup_proposition_predicate(&self, name: &str) -> Option<&PropositionPredicateInfo> {
        let id = self.proposition_predicate_aliases.get(name)?;
        self.proposition_predicates.get(id)
    }

    #[must_use]
    pub fn proposition_predicate_by_id(
        &self,
        id: &PropositionPredicateId,
    ) -> Option<&PropositionPredicateInfo> {
        self.proposition_predicates.get(id)
    }

    /// Lower a surface proposition tail into canonical proposition carriers without solving.
    pub fn lower_proposition_tail(
        &self,
        tail: &PropositionTail,
        source_origin: SourceOrigin,
    ) -> Result<Vec<LoweredPropositionClause>, TypeError> {
        tail.clauses
            .iter()
            .map(|clause| self.lower_proposition_clause(clause, source_origin.clone()))
            .collect()
    }

    /// Add required proposition obligations generated by a specific checking site.
    pub fn add_proposition_obligations_from_tail(
        &mut self,
        tail: &PropositionTail,
        source_origin: SourceOrigin,
        owner_site: PropositionCheckingSite,
    ) -> Result<(), TypeError> {
        let lowered = self.lower_proposition_tail(tail, source_origin)?;
        for clause in lowered {
            self.push_proposition_fact(
                PropositionFactRole::Requirement,
                clause.proposition,
                clause.source_anchor,
                owner_site.clone(),
                clause.outcome,
            );
        }
        Ok(())
    }

    /// Add assumed proposition facts generated by a specific checking site.
    pub fn add_proposition_assumptions_from_tail(
        &mut self,
        tail: &PropositionTail,
        source_origin: SourceOrigin,
        owner_site: PropositionCheckingSite,
    ) -> Result<(), TypeError> {
        let lowered = self.lower_proposition_tail(tail, source_origin)?;
        for clause in lowered {
            self.push_proposition_fact(
                PropositionFactRole::Assumption,
                clause.proposition,
                clause.source_anchor,
                owner_site.clone(),
                clause.outcome,
            );
        }
        Ok(())
    }

    /// Proposition assumptions available as inputs to later solvers.
    #[must_use]
    pub fn proposition_assumptions(&self) -> &[PropositionFactRecord] {
        &self.proposition_assumptions
    }

    /// Required proposition obligations that later task-owned solvers must discharge.
    #[must_use]
    pub fn proposition_obligations(&self) -> &[PropositionFactRecord] {
        &self.proposition_obligations
    }

    /// Export public proposition requirements through the SPEC-064/V5 summary carrier.
    pub fn export_public_proposition_fact_summaries(
        &self,
        module: &ModuleIdentity,
    ) -> Result<Vec<PropositionFactSummary>, TypeEnvError> {
        let public_item = format!(
            "module '{}' public proposition requirement",
            module.path.join("::")
        );
        let mut facts = Vec::new();
        for record in &self.proposition_obligations {
            let predicate_dependencies = self.validate_public_proposition_dependencies(
                &public_item,
                &record.proposition,
                anchor_span(&record.source_anchor),
            )?;
            let outcome = match &record.outcome {
                Some(outcome) => Some(outcome.clone()),
                None => Some(
                    self.solve_proposition(&record.proposition, Some(record.source_anchor.clone()))
                        .map_err(proposition_revalidation_error)?,
                ),
            };
            facts.push(PropositionFactSummary {
                proposition: record.proposition.clone(),
                role: record.role,
                source_anchor: record.source_anchor.clone(),
                predicate_dependencies,
                dependency_summary_refs: Vec::new(),
                outcome,
            });
        }
        Ok(facts)
    }

    fn validate_public_proposition_dependencies(
        &self,
        public_item: &str,
        proposition: &TypeProposition,
        span: Span,
    ) -> Result<Vec<PropositionPredicateId>, TypeEnvError> {
        let mut predicate_dependencies = Vec::new();
        match proposition {
            TypeProposition::Equality(equality) => {
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &equality.lhs,
                    span,
                )?;
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &equality.rhs,
                    span,
                )?;
            }
            TypeProposition::Disequality(disequality) => {
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &disequality.lhs,
                    span,
                )?;
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &disequality.rhs,
                    span,
                )?;
            }
            TypeProposition::InterfaceBound(bound) => {
                if !self.public_interface_dependency_known(&bound.interface) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "interface",
                        &bound.interface.name,
                        span,
                    ));
                }
                self.validate_public_proposition_term_dependencies(
                    public_item,
                    &bound.subject,
                    span,
                )?;
                for arg in &bound.interface_args {
                    self.validate_public_proposition_term_dependencies(public_item, arg, span)?;
                }
            }
            TypeProposition::NamedPredicate(named) => {
                let Some(info) = self.proposition_predicate_by_id(&named.predicate) else {
                    return Err(TypeEnvError::UnknownPropositionPredicate {
                        name: named.predicate.name.to_string(),
                        span,
                    });
                };
                if info.summary.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "proposition predicate",
                        &info.summary.exported_name,
                        span,
                    ));
                }
                if info.summary.params.len() != named.args.len() {
                    return Err(TypeEnvError::PropositionPredicateArityMismatch {
                        name: info.summary.exported_name.to_string(),
                        expected: info.summary.params.len(),
                        actual: named.args.len(),
                        span,
                    });
                }
                predicate_dependencies.push(named.predicate.clone());
                for arg in &named.args {
                    self.validate_public_proposition_term_dependencies(public_item, arg, span)?;
                }
            }
        }
        predicate_dependencies.sort_by(|left, right| {
            left.module
                .path
                .cmp(&right.module.path)
                .then_with(|| left.name.cmp(&right.name))
        });
        predicate_dependencies.dedup();
        Ok(predicate_dependencies)
    }

    fn validate_public_proposition_predicate_summary_dependencies(
        &self,
        summary: &PropositionPredicateSummary,
    ) -> Result<(), TypeEnvError> {
        if summary.visibility != ash_core::ast::Visibility::Public {
            return Ok(());
        }
        let public_item = format!("public proposition predicate '{}'", summary.exported_name);
        for param in &summary.params {
            self.validate_public_canonical_proposition_dependencies(
                &public_item,
                &param.ty,
                anchor_span(&param.source_anchor),
            )?;
        }
        Ok(())
    }

    fn validate_public_proposition_term_dependencies(
        &self,
        public_item: &str,
        term: &TypePropositionTerm,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match term {
            TypePropositionTerm::Canonical(expr) => {
                self.validate_public_canonical_proposition_dependencies(public_item, expr, span)
            }
            TypePropositionTerm::DomainConstructorApp {
                constructor,
                domain,
                args,
                ..
            } => {
                let Some(summary) = self.lookup_sealed_domain_by_id(domain) else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "sealed domain",
                        &domain.name,
                        span,
                    ));
                };
                if summary.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "sealed domain",
                        &summary.exported_name,
                        span,
                    ));
                }
                if !summary
                    .constructors
                    .iter()
                    .any(|candidate| candidate.id == *constructor)
                {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "domain constructor",
                        &constructor.name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_proposition_term_dependencies(public_item, arg, span)?;
                }
                Ok(())
            }
        }
    }

    fn public_interface_dependency_known(&self, interface: &InterfaceIdentityId) -> bool {
        if !self.known_interface_identities.contains(interface) {
            return false;
        }
        self.interfaces
            .get(interface.name.as_str())
            .is_none_or(|info| info.visibility == ash_core::ast::Visibility::Public)
    }

    fn public_associated_member_dependency_known(
        &self,
        member: &AssociatedMemberIdentityId,
    ) -> bool {
        self.known_associated_member_identities.contains(member)
            && self.public_interface_dependency_known(&member.interface)
    }

    fn validate_public_canonical_proposition_dependencies(
        &self,
        public_item: &str,
        expr: &CanonicalTypeExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp { origin, args, .. } => {
                let Some(visible_name) = self.canonical_type_names.get(origin) else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "ordinary type",
                        &origin.name,
                        span,
                    ));
                };
                if !self.ast_types.get(visible_name).is_some_and(|ty| {
                    ty.visibility == ash_core::ast::Visibility::Public || ty.builtin
                }) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "ordinary type",
                        visible_name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            CanonicalTypeExpr::Projection {
                interface,
                member,
                args,
                ..
            } => {
                if !self.public_interface_dependency_known(interface) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "interface",
                        &interface.name,
                        span,
                    ));
                }
                if !self.public_associated_member_dependency_known(member) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "associated member",
                        &member.name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
                let Some(def) = self.local_type_functions.get(head) else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "type function",
                        &head.name,
                        span,
                    ));
                };
                if def.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "type function",
                        &head.name,
                        span,
                    ));
                }
                for arg in args {
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                let Some(kind_summary) = self.lookup_promoted_data_kind_by_id(&app.data_kind)
                else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data kind",
                        &app.data_kind.name,
                        span,
                    ));
                };
                if kind_summary.visibility != ash_core::ast::Visibility::Public {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data kind",
                        &kind_summary.exported_name,
                        span,
                    ));
                }
                let Some(source_visible_name) =
                    self.canonical_type_names.get(&kind_summary.source_type)
                else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted source ADT",
                        &kind_summary.source_type.name,
                        span,
                    ));
                };
                if !self.ast_types.get(source_visible_name).is_some_and(|ty| {
                    ty.visibility == ash_core::ast::Visibility::Public || ty.builtin
                }) {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted source ADT",
                        source_visible_name,
                        span,
                    ));
                }
                let Some(constructor_summary) =
                    self.lookup_promoted_constructor_by_id(&app.constructor)
                else {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data constructor",
                        &app.constructor.name,
                        span,
                    ));
                };
                if constructor_summary.visibility != ash_core::ast::Visibility::Public
                    || constructor_summary.id.kind != kind_summary.id
                {
                    return Err(private_proposition_dependency_error(
                        public_item,
                        "promoted data constructor",
                        &constructor_summary.exported_name,
                        span,
                    ));
                }
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    span,
                )?;
                let kinding = self
                    .promoted_constructor_kind(&app.constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "promoted data constructor '{}' has no validated kinding metadata",
                                app.constructor.name
                            ),
                            span,
                        )
                    })?;
                for (index, arg) in app.args.iter().enumerate() {
                    self.validate_public_canonical_proposition_dependencies(
                        public_item,
                        arg,
                        span,
                    )?;
                    if let Some(expected_kind) = kinding
                        .field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(arg, expected_kind, span)?;
                    }
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public proposition '{public_item}' cannot export constructor-variable application '{}' until TASK-908 defines higher-kinded evidence summaries",
                    app.constructor.name
                ),
                span,
            )),
        }
    }

    /// Add one required proposition obligation that has already been lowered to core carriers.
    pub fn add_proposition_obligation(
        &mut self,
        proposition: TypeProposition,
        source_anchor: SourceAnchor,
        owner_site: PropositionCheckingSite,
    ) {
        self.push_proposition_fact(
            PropositionFactRole::Requirement,
            proposition,
            source_anchor,
            owner_site,
            None,
        );
    }

    /// Solve one proposition using the conservative SPEC-064 equality/disequality layer.
    pub fn solve_proposition(
        &self,
        proposition: &TypeProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        match proposition {
            TypeProposition::Equality(equality) => {
                self.solve_equality_proposition(proposition, equality, source_anchor)
            }
            TypeProposition::Disequality(disequality) => {
                self.solve_disequality_proposition(proposition, disequality, source_anchor)
            }
            TypeProposition::InterfaceBound(bound) => {
                Ok(self.solve_interface_bound_proposition(proposition, bound, source_anchor))
            }
            TypeProposition::NamedPredicate(named) => {
                self.solve_named_predicate_proposition(proposition, named, source_anchor)
            }
        }
    }

    fn solve_interface_bound_proposition(
        &self,
        proposition: &TypeProposition,
        bound: &InterfaceBoundProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> PropositionOutcome {
        let exact_evidence = self.proposition_assumptions.iter().find_map(|record| {
            if !matches!(
                record.role,
                PropositionFactRole::Assumption | PropositionFactRole::Evidence
            ) {
                return None;
            }
            if !matches!(
                &record.proposition,
                TypeProposition::InterfaceBound(assumed) if assumed == bound
            ) {
                return None;
            }
            match record.owner_site.kind {
                PropositionCheckingSiteKind::ConcreteImpl => {
                    Some((record, PropositionEvidenceRule::ConcreteImplEvidence))
                }
                PropositionCheckingSiteKind::TypeVariableInterfaceBound
                | PropositionCheckingSiteKind::ImplWhereBound => {
                    Some((record, PropositionEvidenceRule::InScopeInterfaceBound))
                }
                PropositionCheckingSiteKind::ExplicitRequirement
                | PropositionCheckingSiteKind::Synthetic => None,
            }
        });

        let Some((record, rule)) = exact_evidence else {
            return proposition_deferral(
                proposition,
                PropositionDeferredKind::MissingInterfaceEvidence,
                source_anchor,
                true,
            );
        };

        proposition_satisfaction(
            proposition,
            None,
            rule,
            source_anchor.or_else(|| Some(record.source_anchor.clone())),
        )
    }

    fn solve_named_predicate_proposition(
        &self,
        proposition: &TypeProposition,
        named: &NamedPredicateProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        let Some(info) = self.proposition_predicate_by_id(&named.predicate) else {
            return Err(TypeEnvError::UnknownPropositionPredicate {
                name: named.predicate.name.to_string(),
                span: source_anchor
                    .as_ref()
                    .map_or_else(Span::default, anchor_span),
            }
            .into());
        };

        if info.summary.params.len() != named.args.len() {
            return Err(TypeEnvError::PropositionPredicateArityMismatch {
                name: info.summary.exported_name.to_string(),
                expected: info.summary.params.len(),
                actual: named.args.len(),
                span: source_anchor
                    .as_ref()
                    .map_or_else(Span::default, anchor_span),
            }
            .into());
        }

        match info.solver_kind {
            PropositionPredicateSolverKind::CompilerBuiltinSatisfied => {
                Ok(proposition_satisfaction(
                    proposition,
                    None,
                    PropositionEvidenceRule::NamedPredicateAssumption,
                    source_anchor,
                ))
            }
            PropositionPredicateSolverKind::DeferredUnsupported => Ok(proposition_deferral(
                proposition,
                PropositionDeferredKind::UnsupportedNamedPredicate,
                source_anchor,
                true,
            )),
        }
    }

    /// Solve all stored proposition obligations, updating each fact record with its outcome.
    pub fn solve_proposition_obligations(&mut self) -> Result<Vec<PropositionOutcome>, TypeError> {
        let pending = self
            .proposition_obligations
            .iter()
            .enumerate()
            .map(|(index, record)| {
                (
                    index,
                    record.proposition.clone(),
                    record.source_anchor.clone(),
                )
            })
            .collect::<Vec<_>>();

        let mut outcomes = Vec::with_capacity(pending.len());
        for (index, proposition, source_anchor) in pending {
            let outcome = self.solve_proposition(&proposition, Some(source_anchor))?;
            if let Some(record) = self.proposition_obligations.get_mut(index) {
                record.outcome = Some(outcome.clone());
            }
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    /// Solve and require discharge of every stored proposition obligation.
    ///
    /// Plain solving may conservatively return deferred outcomes. Checking points
    /// that require proofs call this stricter path so refuted or deferred
    /// propositions become ordinary type-environment errors without invoking
    /// inversion or meta-solving.
    pub fn discharge_required_proposition_obligations(
        &mut self,
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        self.discharge_required_proposition_obligations_from(0)
    }

    pub(crate) fn discharge_required_proposition_obligations_since(
        &mut self,
        start_index: usize,
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        self.discharge_required_proposition_obligations_from(start_index)
    }

    fn discharge_required_proposition_obligations_from(
        &mut self,
        start_index: usize,
    ) -> Result<Vec<PropositionOutcome>, TypeEnvError> {
        let pending = self
            .proposition_obligations
            .iter()
            .enumerate()
            .skip(start_index)
            .map(|(index, record)| {
                (
                    index,
                    record.proposition.clone(),
                    record.source_anchor.clone(),
                    record.owner_site.clone(),
                )
            })
            .collect::<Vec<_>>();

        let mut checked = Vec::with_capacity(pending.len());
        for (index, proposition, source_anchor, owner_site) in pending {
            let outcome = self
                .solve_proposition(&proposition, Some(source_anchor.clone()))
                .map_err(proposition_revalidation_error)?;
            match &outcome {
                PropositionOutcome::Satisfied(_) => checked.push((index, outcome)),
                PropositionOutcome::Refuted(_) | PropositionOutcome::Deferred(_) => {
                    return Err(required_proposition_discharge_error(
                        &owner_site,
                        &source_anchor,
                        &outcome,
                    ));
                }
            }
        }
        let mut outcomes = Vec::with_capacity(checked.len());
        for (index, outcome) in checked {
            if let Some(record) = self.proposition_obligations.get_mut(index) {
                record.outcome = Some(outcome.clone());
            }
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    fn solve_equality_proposition(
        &self,
        proposition: &TypeProposition,
        equality: &TypeEqualityProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        let span = source_anchor.as_ref().map(anchor_span).unwrap_or_default();
        self.validate_proposition_term_promoted_operands(&equality.lhs, span)?;
        self.validate_proposition_term_promoted_operands(&equality.rhs, span)?;
        let (result, lhs_norm, rhs_norm) =
            self.compare_proposition_terms(&equality.lhs, &equality.rhs)?;
        let normalized_terms = Some(proposition_comparison_terms(
            lhs_norm.clone(),
            rhs_norm.clone(),
        ));

        Ok(match result {
            DefinitionalEqualityResult::Equal => proposition_satisfaction(
                proposition,
                normalized_terms,
                PropositionEvidenceRule::DefinitionalEquality,
                source_anchor,
            ),
            DefinitionalEqualityResult::NotEqual { .. } => proposition_refutation(
                proposition,
                normalized_terms,
                PropositionRefutationReason::DefinitionalEquality,
                source_anchor,
            ),
            DefinitionalEqualityResult::BlockedByNeutrality {
                neutral_subterms, ..
            } => {
                let kind = if neutral_subterms.is_empty() {
                    proposition_deferred_kind_from_blocked_normals(&lhs_norm, &rhs_norm)
                } else {
                    proposition_deferred_kind_from_blockers(&neutral_subterms)
                };
                proposition_deferral(proposition, kind, source_anchor, true)
            }
        })
    }

    fn solve_disequality_proposition(
        &self,
        proposition: &TypeProposition,
        disequality: &TypeDisequalityProposition,
        source_anchor: Option<SourceAnchor>,
    ) -> Result<PropositionOutcome, TypeError> {
        let span = source_anchor.as_ref().map(anchor_span).unwrap_or_default();
        self.validate_proposition_term_promoted_operands(&disequality.lhs, span)?;
        self.validate_proposition_term_promoted_operands(&disequality.rhs, span)?;
        let normalizer = Normalizer::new(self);
        let lhs_norm = self.normalize_proposition_term(&normalizer, &disequality.lhs)?;
        let rhs_norm = self.normalize_proposition_term(&normalizer, &disequality.rhs)?;
        let comparison = normalizer.definitional_equality_normal_forms(&lhs_norm, &rhs_norm);
        let normalized_terms = Some(proposition_comparison_terms(
            lhs_norm.clone(),
            rhs_norm.clone(),
        ));

        if matches!(comparison, DefinitionalEqualityResult::Equal) {
            return Ok(proposition_refutation(
                proposition,
                normalized_terms,
                PropositionRefutationReason::DefinitionalEquality,
                source_anchor,
            ));
        }

        if sealed_domain_constructor_heads_are_disjoint(&lhs_norm, &rhs_norm) {
            return Ok(proposition_satisfaction(
                proposition,
                normalized_terms,
                PropositionEvidenceRule::SealedDomainConstructorDisjointness,
                source_anchor,
            ));
        }

        let kind = match comparison {
            DefinitionalEqualityResult::BlockedByNeutrality {
                neutral_subterms, ..
            } if !neutral_subterms.is_empty() => {
                proposition_deferred_kind_from_blockers(&neutral_subterms)
            }
            _ if proposition_normal_form_is_open_or_blocked(&lhs_norm)
                || proposition_normal_form_is_open_or_blocked(&rhs_norm) =>
            {
                proposition_deferred_kind_from_blocked_normals(&lhs_norm, &rhs_norm)
            }
            _ => PropositionDeferredKind::UnsupportedProofSearch,
        };

        Ok(proposition_deferral(proposition, kind, source_anchor, true))
    }

    fn validate_proposition_term_promoted_operands(
        &self,
        term: &TypePropositionTerm,
        span: Span,
    ) -> Result<(), TypeError> {
        match term {
            TypePropositionTerm::Canonical(expr) => {
                self.validate_canonical_proposition_promoted_operands(expr, span)
            }
            TypePropositionTerm::DomainConstructorApp { args, .. } => {
                for arg in args {
                    self.validate_proposition_term_promoted_operands(arg, span)?;
                }
                Ok(())
            }
        }
    }

    fn validate_canonical_proposition_promoted_operands(
        &self,
        expr: &CanonicalTypeExpr,
        span: Span,
    ) -> Result<(), TypeError> {
        match expr {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => Ok(()),
            CanonicalTypeExpr::NominalApp { args, .. }
            | CanonicalTypeExpr::Projection { args, .. }
            | CanonicalTypeExpr::ComputationHeadApp { args, .. } => {
                for arg in args {
                    self.validate_canonical_proposition_promoted_operands(arg, span)?;
                }
                Ok(())
            }
            CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
                self.validate_registered_promoted_constructor_app(
                    &app.constructor,
                    &app.data_kind,
                    app.args.len(),
                    &app.kind,
                    span,
                )?;
                let field_data_kind_constraints = self
                    .promoted_constructor_kind(&app.constructor)
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "promoted data constructor '{}' has no validated kinding metadata",
                                app.constructor.name
                            ),
                            span,
                        )
                    })?
                    .field_data_kind_constraints
                    .clone();
                for (index, arg) in app.args.iter().enumerate() {
                    self.validate_canonical_proposition_promoted_operands(arg, span)?;
                    if let Some(expected_kind) = field_data_kind_constraints
                        .get(index)
                        .and_then(|constraint| constraint.as_ref())
                    {
                        self.validate_canonical_promoted_data_kind(arg, expected_kind, span)?;
                    }
                }
                Ok(())
            }
            CanonicalTypeExpr::ConstructorVariableApp(app) => {
                for arg in &app.args {
                    self.validate_canonical_proposition_promoted_operands(arg, span)?;
                }
                Ok(())
            }
        }
    }

    fn compare_proposition_terms(
        &self,
        lhs: &TypePropositionTerm,
        rhs: &TypePropositionTerm,
    ) -> Result<(DefinitionalEqualityResult, NormalTypeExpr, NormalTypeExpr), TypeError> {
        let normalizer = Normalizer::new(self);
        match (lhs, rhs) {
            (TypePropositionTerm::Canonical(lhs), TypePropositionTerm::Canonical(rhs)) => {
                let result = normalizer
                    .definitional_equality(lhs, rhs)
                    .map_err(proposition_normalization_error)?;
                let lhs_norm = match &result {
                    DefinitionalEqualityResult::Equal => {
                        normalizer
                            .normalize(lhs)
                            .map_err(proposition_normalization_error)?
                            .normal
                    }
                    DefinitionalEqualityResult::NotEqual { lhs_norm, .. }
                    | DefinitionalEqualityResult::BlockedByNeutrality { lhs_norm, .. } => {
                        lhs_norm.clone()
                    }
                };
                let rhs_norm = match &result {
                    DefinitionalEqualityResult::Equal => {
                        normalizer
                            .normalize(rhs)
                            .map_err(proposition_normalization_error)?
                            .normal
                    }
                    DefinitionalEqualityResult::NotEqual { rhs_norm, .. }
                    | DefinitionalEqualityResult::BlockedByNeutrality { rhs_norm, .. } => {
                        rhs_norm.clone()
                    }
                };
                Ok((result, lhs_norm, rhs_norm))
            }
            _ => {
                let lhs_norm = self.normalize_proposition_term(&normalizer, lhs)?;
                let rhs_norm = self.normalize_proposition_term(&normalizer, rhs)?;
                let result = normalizer.definitional_equality_normal_forms(&lhs_norm, &rhs_norm);
                Ok((result, lhs_norm, rhs_norm))
            }
        }
    }

    fn normalize_proposition_term(
        &self,
        normalizer: &Normalizer<'_>,
        term: &TypePropositionTerm,
    ) -> Result<NormalTypeExpr, TypeError> {
        match term {
            TypePropositionTerm::Canonical(expr) => normalizer
                .normalize(expr)
                .map(|outcome| outcome.normal)
                .map_err(proposition_normalization_error),
            TypePropositionTerm::DomainConstructorApp {
                constructor,
                domain,
                args,
                kind,
            } => {
                let args = args
                    .iter()
                    .map(|arg| self.normalize_proposition_term(normalizer, arg))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(NormalTypeExpr::DomainConstructorApp {
                    constructor: constructor.clone(),
                    domain: domain.clone(),
                    args,
                    kind: kind.clone(),
                })
            }
        }
    }

    fn lower_proposition_clause(
        &self,
        clause: &PropositionClause,
        source_origin: SourceOrigin,
    ) -> Result<LoweredPropositionClause, TypeError> {
        let source_anchor =
            proposition_source_anchor(source_origin, clause.span, "source proposition clause");
        let (proposition, outcome) = match &clause.kind {
            PropositionClauseKind::Equality { lhs, rhs, .. } => {
                let lhs = self.lower_surface_type_term(lhs)?;
                let rhs = self.lower_surface_type_term(rhs)?;
                (
                    TypeProposition::Equality(TypeEqualityProposition { lhs, rhs }),
                    None,
                )
            }
            PropositionClauseKind::Disequality { lhs, rhs, .. } => {
                let lhs = self.lower_surface_type_term(lhs)?;
                let rhs = self.lower_surface_type_term(rhs)?;
                (
                    TypeProposition::Disequality(TypeDisequalityProposition { lhs, rhs }),
                    None,
                )
            }
            PropositionClauseKind::InterfaceBound {
                subject, interface, ..
            } => {
                let subject = self.lower_surface_type_term(subject)?;
                let (interface_name, interface_args) =
                    self.interface_clause_name_and_args(interface)?;
                let interface_id = self
                    .interface_identity_for_name(&interface_name)
                    .cloned()
                    .ok_or_else(|| {
                        TypeEnvError::MissingInterface(interface_name.clone(), clause.span)
                    })?;
                let interface_args = interface_args
                    .iter()
                    .map(|arg| self.lower_surface_type_term(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                (
                    TypeProposition::InterfaceBound(InterfaceBoundProposition {
                        subject,
                        interface: interface_id,
                        interface_args,
                    }),
                    None,
                )
            }
            PropositionClauseKind::NamedPredicate {
                name,
                name_span,
                args,
            } => {
                let predicate_info = self
                    .lookup_proposition_predicate(name.as_ref())
                    .ok_or_else(|| {
                        TypeError::from(TypeEnvError::UnknownPropositionPredicate {
                            name: name.to_string(),
                            span: *name_span,
                        })
                    })?;
                if predicate_info.summary.params.len() != args.len() {
                    return Err(TypeEnvError::PropositionPredicateArityMismatch {
                        name: name.to_string(),
                        expected: predicate_info.summary.params.len(),
                        actual: args.len(),
                        span: clause.span,
                    }
                    .into());
                }
                let args = args
                    .iter()
                    .map(|arg| self.lower_surface_type_term(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let proposition = TypeProposition::NamedPredicate(NamedPredicateProposition {
                    predicate: predicate_info.summary.id.clone(),
                    args,
                });
                let outcome = match predicate_info.solver_kind {
                    PropositionPredicateSolverKind::CompilerBuiltinSatisfied => None,
                    PropositionPredicateSolverKind::DeferredUnsupported => {
                        Some(proposition_deferral(
                            &proposition,
                            PropositionDeferredKind::UnsupportedNamedPredicate,
                            Some(source_anchor.clone()),
                            true,
                        ))
                    }
                };
                (proposition, outcome)
            }
        };
        Ok(LoweredPropositionClause {
            proposition,
            source_anchor,
            outcome,
        })
    }

    fn lower_surface_type_term(&self, ty: &SurfaceType) -> Result<TypePropositionTerm, TypeError> {
        match ty {
            SurfaceType::Name(name) => {
                if let Some((domain, constructor)) = self.find_any_domain_constructor(name.as_ref())
                {
                    if !constructor.fields.is_empty() {
                        return Err(TypeError::ConstructorNameMismatch {
                            expected: format!(
                                "{} type arguments for sealed-domain constructor {}",
                                constructor.fields.len(),
                                constructor.exported_name
                            ),
                            found: "0".to_string(),
                            span: Span::default(),
                        });
                    }
                    return Ok(TypePropositionTerm::DomainConstructorApp {
                        constructor: constructor.id.clone(),
                        domain: domain.id.clone(),
                        args: Vec::new(),
                        kind: Kind::Type,
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(proposition_term_from_canonical)
            }
            SurfaceType::Constructor { name, args } => {
                if let Some((domain, constructor)) = self.find_any_domain_constructor(name.as_ref())
                {
                    let domain = domain.clone();
                    let constructor = constructor.clone();
                    if constructor.fields.len() != args.len() {
                        return Err(TypeError::ConstructorNameMismatch {
                            expected: format!(
                                "{} type arguments for sealed-domain constructor {}",
                                constructor.fields.len(),
                                constructor.exported_name
                            ),
                            found: args.len().to_string(),
                            span: Span::default(),
                        });
                    }
                    let args = args
                        .iter()
                        .map(|arg| self.lower_surface_type_term(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(TypePropositionTerm::DomainConstructorApp {
                        constructor: constructor.id,
                        domain: domain.id,
                        args,
                        kind: Kind::Type,
                    })
                } else if let Some(head) = self.local_type_function_heads.get(name.as_ref()) {
                    let args = args
                        .iter()
                        .map(|arg| self.lower_surface_type_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(TypePropositionTerm::Canonical(
                        CanonicalTypeExpr::ComputationHeadApp {
                            head: head.clone(),
                            args,
                            kind: Kind::Type,
                        },
                    ))
                } else {
                    self.lower_surface_type_to_canonical(ty)
                        .map(proposition_term_from_canonical)
                }
            }
            _ => self
                .lower_surface_type_to_canonical(ty)
                .map(proposition_term_from_canonical),
        }
    }

    fn interface_clause_name_and_args<'a>(
        &self,
        interface: &'a SurfaceType,
    ) -> Result<(String, &'a [SurfaceType]), TypeError> {
        match interface {
            SurfaceType::Name(name) => Ok((name.to_string(), &[])),
            SurfaceType::Constructor { name, args } => Ok((name.to_string(), args.as_slice())),
            other => Err(TypeError::ConstructorNameMismatch {
                expected: "interface name or interface type application".to_string(),
                found: surface_projection_base_spelling(other),
                span: Span::default(),
            }),
        }
    }

    fn push_proposition_fact(
        &mut self,
        role: PropositionFactRole,
        proposition: TypeProposition,
        source_anchor: SourceAnchor,
        owner_site: PropositionCheckingSite,
        outcome: Option<PropositionOutcome>,
    ) {
        let record = PropositionFactRecord {
            proposition,
            source_anchor,
            owner_site,
            role,
            outcome,
        };
        let facts = match role {
            PropositionFactRole::Requirement => &mut self.proposition_obligations,
            PropositionFactRole::Assumption | PropositionFactRole::Evidence => {
                &mut self.proposition_assumptions
            }
        };
        if !facts.iter().any(|existing| existing == &record) {
            facts.push(record);
        }
    }

    fn record_type_var_interface_bound_assumption(
        &mut self,
        var: TypeVar,
        interface: &str,
        source_anchor: SourceAnchor,
        owner_site: PropositionCheckingSite,
    ) {
        let Some(interface_id) = self.interface_identity_for_name(interface).cloned() else {
            return;
        };
        let proposition = TypeProposition::InterfaceBound(InterfaceBoundProposition {
            subject: type_var_proposition_term(var),
            interface: interface_id,
            interface_args: Vec::new(),
        });
        self.push_proposition_fact(
            PropositionFactRole::Assumption,
            proposition,
            source_anchor,
            owner_site,
            None,
        );
    }

    fn record_concrete_impl_interface_assumption(
        &mut self,
        interface: &str,
        lowered_type_args: &[Type],
        source_anchor: SourceAnchor,
    ) {
        let Some(interface_id) = self.interface_identity_for_name(interface).cloned() else {
            return;
        };
        let Some((subject, interface_args)) = lowered_type_args.split_first() else {
            return;
        };
        let Some(subject) = self
            .lower_type_to_canonical_for_equality(subject)
            .map(proposition_term_from_canonical)
        else {
            return;
        };
        let Some(interface_args) = interface_args
            .iter()
            .map(|arg| {
                self.lower_type_to_canonical_for_equality(arg)
                    .map(proposition_term_from_canonical)
            })
            .collect::<Option<Vec<_>>>()
        else {
            return;
        };
        let proposition = TypeProposition::InterfaceBound(InterfaceBoundProposition {
            subject,
            interface: interface_id,
            interface_args,
        });
        self.push_proposition_fact(
            PropositionFactRole::Assumption,
            proposition,
            source_anchor,
            PropositionCheckingSite::new(
                0x8753_0000u64 + self.impls.len() as u64,
                PropositionCheckingSiteKind::ConcreteImpl,
                Some(format!("concrete impl for interface {interface}")),
            ),
            None,
        );
    }

    /// Lower a surface `Type` into the Phase 110 canonical type-expression substrate.
    pub fn lower_surface_type_to_canonical(
        &self,
        ty: &SurfaceType,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        match ty {
            SurfaceType::Hole { span } => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: "type hole _".to_string(),
                span: *span,
            }),
            SurfaceType::Name(name) => match name.as_ref() {
                "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref" => {
                    Ok(CanonicalTypeExpr::Primitive(name.to_string()))
                }
                _ => {
                    if let Some(kind) = self.type_parameter_kind(name.as_ref()) {
                        if kind.is_type() {
                            Ok(CanonicalTypeExpr::Var(name.to_string()))
                        } else {
                            Err(TypeError::from(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor variable '{}' has kind {}; expected a fully applied proper type",
                                    name, kind
                                ),
                                Span::default(),
                            )))
                        }
                    } else {
                        match self.resolve_type(name.as_ref()) {
                            Ok((qualified, _)) => {
                                self.check_type_constructor_arity(&qualified, 0)?;
                                Ok(CanonicalTypeExpr::NominalApp {
                                    origin: self
                                        .canonical_type_identity_for_visible_name(name.as_ref())?,
                                    visible_name: name.to_string(),
                                    args: vec![],
                                    kind: Kind::Type,
                                })
                            }
                            Err(TypeError::UnboundVariable(_, _)) => {
                                Ok(CanonicalTypeExpr::Var(name.to_string()))
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
            },
            SurfaceType::Constructor { name, args } => {
                if let Some(kind) = self.type_parameter_kind(name.as_ref()) {
                    if kind.is_type() {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "proper type variable '{}' of kind * cannot be applied as a constructor",
                                name
                            ),
                            Span::default(),
                        )));
                    }
                    let expected_arity = kind.arity();
                    if args.len() != expected_arity {
                        return Err(TypeError::from(TypeEnvError::InvalidDefinition(
                            format!(
                                "wrong arity for constructor variable '{}': expected {}, found {}",
                                name,
                                expected_arity,
                                args.len()
                            ),
                            Span::default(),
                        )));
                    }
                    let lowered_args = args
                        .iter()
                        .map(|arg| self.lower_surface_type_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(CanonicalTypeExpr::ConstructorVariableApp(Box::new(
                        ConstructorVariableApp::new(
                            ConstructorVariableRef::new(name.to_string(), kind.clone(), None),
                            lowered_args,
                            Kind::Type,
                            None,
                        ),
                    )));
                }
                let (qualified, _) =
                    self.resolve_type(name.as_ref()).map_err(|err| match err {
                        TypeError::UnboundVariable(_, span) => {
                            TypeError::from(TypeEnvError::InvalidDefinition(
                                format!(
                                    "constructor-variable application '{}<...>' cannot be lowered until TASK-907 tracks constructor variables",
                                    name
                                ),
                                span,
                            ))
                        }
                        err => err,
                    })?;
                self.check_type_constructor_arity(&qualified, args.len())?;
                Ok(CanonicalTypeExpr::NominalApp {
                    origin: self.canonical_type_identity_for_visible_name(name.as_ref())?,
                    visible_name: name.to_string(),
                    args: args
                        .iter()
                        .map(|arg| self.lower_surface_type_to_canonical(arg))
                        .collect::<Result<Vec<_>, _>>()?,
                    kind: Kind::Type,
                })
            }
            SurfaceType::Associated { base, name } => {
                if let SurfaceType::Constructor {
                    name: interface,
                    args,
                } = base.as_ref()
                    && self
                        .lookup_associated_family_declaration(interface, name)
                        .is_some()
                {
                    return self.lower_explicit_associated_family_projection_to_canonical(
                        interface,
                        args,
                        name,
                        Span::default(),
                    );
                }
                if matches!(base.as_ref(), SurfaceType::Associated { .. }) {
                    return Err(TypeError::ConstructorNameMismatch {
                        expected: "supported associated projection base (nested projection bases are unsupported)"
                            .to_string(),
                        found: format!("nested projection base {base:?}"),
                        span: Span::default(),
                    });
                }
                if matches!(
                    base.as_ref(),
                    SurfaceType::Hole { .. }
                        | SurfaceType::Tuple(_)
                        | SurfaceType::Record(_)
                        | SurfaceType::List(_)
                        | SurfaceType::Capability(_)
                        | SurfaceType::Fn(_, _)
                ) {
                    let found = match base.as_ref() {
                        SurfaceType::Tuple(items) => {
                            format!("unsupported projection base Tuple({})", items.len())
                        }
                        SurfaceType::Record(fields) => {
                            format!("unsupported projection base Record({})", fields.len())
                        }
                        SurfaceType::List(_) => "unsupported projection base List".to_string(),
                        SurfaceType::Capability(name) => {
                            format!("unsupported projection base Capability({name})")
                        }
                        SurfaceType::Fn(_, _) => "unsupported projection base Fn".to_string(),
                        SurfaceType::Hole { .. } => {
                            "unsupported projection base type hole _".to_string()
                        }
                        _ => unreachable!("guarded by matches!"),
                    };
                    return Err(TypeError::ConstructorNameMismatch {
                        expected: "supported associated projection base (type variable or nominal application)"
                            .to_string(),
                        found,
                        span: Span::default(),
                    });
                }
                let lowered_base = self.lower_surface_type_to_canonical(base)?;
                self.lower_associated_projection_to_canonical(&lowered_base, name)
            }
            SurfaceType::Tuple(items) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Tuple({})", items.len()),
                span: Span::default(),
            }),
            SurfaceType::Record(fields) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Record({})", fields.len()),
                span: Span::default(),
            }),
            SurfaceType::List(_) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: "List".to_string(),
                span: Span::default(),
            }),
            SurfaceType::Capability(name) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: format!("Capability({name})"),
                span: Span::default(),
            }),
            SurfaceType::Fn(_, _) => Err(TypeError::ConstructorNameMismatch {
                expected: "nominal or associated type expression supported by TASK-798 lowering"
                    .to_string(),
                found: "Fn".to_string(),
                span: Span::default(),
            }),
            SurfaceType::AssociatedFamilyProjection {
                interface,
                args,
                member,
                span,
            } => self.lower_explicit_associated_family_projection_to_canonical(
                interface, args, member, *span,
            ),
        }
    }

    /// Elaborate an audited explicit do-target type into a core constructor
    /// expression, preserving exactly one source hole as a partial application.
    ///
    /// This is the TASK-901 semantic substrate only: it validates kind/arity and
    /// hole placement for MVP partial target shapes without selecting Monad
    /// evidence or integrating with do-target resolution.
    pub fn elaborate_do_target_constructor_expr(
        &self,
        ty: &SurfaceType,
    ) -> Result<TypeConstructorExpr, PartialConstructorElaborationError> {
        self.elaborate_partial_type_constructor(ty, true)
    }

    /// Elaborate a surface type/constructor expression into the core
    /// `TypeConstructorExpr` carrier used by partial-constructor consumers.
    pub fn elaborate_partial_type_constructor(
        &self,
        ty: &SurfaceType,
        require_partial_target: bool,
    ) -> Result<TypeConstructorExpr, PartialConstructorElaborationError> {
        match ty {
            SurfaceType::Name(name) => {
                let constructor = name.to_string();
                let arity = self
                    .type_constructor_arity_for_visible_name(name.as_ref())
                    .ok_or_else(|| PartialConstructorElaborationError::UnknownConstructor {
                        constructor: constructor.clone(),
                        span: Span::default(),
                    })?;
                if require_partial_target {
                    if arity > 1 {
                        return Err(
                            PartialConstructorElaborationError::BareHigherArityConstructor {
                                constructor: constructor.clone(),
                                arity,
                                hint: bare_constructor_hole_hint(&constructor, arity),
                                span: Span::default(),
                            },
                        );
                    }
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor,
                        span: Span::default(),
                    });
                }
                if arity == 0 {
                    return self
                        .lower_surface_type_to_canonical(ty)
                        .map(TypeConstructorExpr::ProperType)
                        .map_err(|err| {
                            PartialConstructorElaborationError::ArgumentLoweringFailed {
                                constructor,
                                reason: err.to_string(),
                                span: Span::default(),
                            }
                        });
                }
                let origin = self
                    .canonical_type_identity_for_visible_name(name.as_ref())
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: constructor.clone(),
                            reason: err.to_string(),
                            span: Span::default(),
                        },
                    )?;
                Ok(TypeConstructorExpr::ConstructorHead(
                    TypeConstructorHeadId::nominal(origin, constructor),
                ))
            }
            SurfaceType::Constructor { name, args } => self.elaborate_constructor_application(
                name.as_ref(),
                args,
                require_partial_target,
                Span::default(),
            ),
            SurfaceType::AssociatedFamilyProjection { span, .. } => {
                if surface_type_contains_hole(ty) {
                    return Err(PartialConstructorElaborationError::NoInversionBoundary {
                        context: "associated-family projection output".to_string(),
                        span: *span,
                    });
                }
                if require_partial_target {
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor: "associated-family projection".to_string(),
                        span: *span,
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(TypeConstructorExpr::ProperType)
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: "associated-family projection".to_string(),
                            reason: err.to_string(),
                            span: *span,
                        },
                    )
            }
            SurfaceType::Associated { base, name } => {
                if surface_type_contains_hole(base) {
                    return Err(PartialConstructorElaborationError::NoInversionBoundary {
                        context: format!("associated projection `{name}`"),
                        span: Span::default(),
                    });
                }
                if require_partial_target {
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor: format!("associated projection `{name}`"),
                        span: Span::default(),
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(TypeConstructorExpr::ProperType)
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: name.to_string(),
                            reason: err.to_string(),
                            span: Span::default(),
                        },
                    )
            }
            SurfaceType::Hole { span } => Err(
                PartialConstructorElaborationError::UnsupportedHolePosition {
                    reason: "bare `_` has no constructor head or expected value slot".to_string(),
                    span: *span,
                },
            ),
            SurfaceType::List(_)
            | SurfaceType::Tuple(_)
            | SurfaceType::Record(_)
            | SurfaceType::Capability(_)
            | SurfaceType::Fn(_, _) => {
                if surface_type_contains_hole(ty) {
                    return Err(
                        PartialConstructorElaborationError::UnsupportedHolePosition {
                            reason:
                                "holes are enabled only in explicit constructor argument spines"
                                    .to_string(),
                            span: Span::default(),
                        },
                    );
                }
                if require_partial_target {
                    return Err(PartialConstructorElaborationError::MissingHole {
                        constructor: "proper type expression".to_string(),
                        span: Span::default(),
                    });
                }
                self.lower_surface_type_to_canonical(ty)
                    .map(TypeConstructorExpr::ProperType)
                    .map_err(
                        |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: "proper type expression".to_string(),
                            reason: err.to_string(),
                            span: Span::default(),
                        },
                    )
            }
        }
    }

    fn elaborate_constructor_application(
        &self,
        constructor: &str,
        args: &[SurfaceType],
        require_partial_target: bool,
        span: Span,
    ) -> Result<TypeConstructorExpr, PartialConstructorElaborationError> {
        let Some(expected_arity) = self.type_constructor_arity_for_visible_name(constructor) else {
            return Err(PartialConstructorElaborationError::UnknownConstructor {
                constructor: constructor.to_string(),
                span,
            });
        };
        if args.len() != expected_arity {
            return Err(PartialConstructorElaborationError::WrongArity {
                constructor: constructor.to_string(),
                expected_arity,
                found_arity: args.len(),
                span,
            });
        }

        let hole_count = args.iter().map(surface_type_hole_count).sum::<usize>();
        if require_partial_target && hole_count == 0 {
            return Err(PartialConstructorElaborationError::MissingHole {
                constructor: constructor.to_string(),
                span,
            });
        }
        if hole_count > 1 {
            return Err(PartialConstructorElaborationError::MultipleHoles {
                constructor: constructor.to_string(),
                count: hole_count,
                span,
            });
        }

        let origin = self
            .canonical_type_identity_for_visible_name(constructor)
            .map_err(
                |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                    constructor: constructor.to_string(),
                    reason: err.to_string(),
                    span,
                },
            )?;
        if hole_count == 0 {
            return self
                .lower_surface_type_to_canonical(&SurfaceType::Constructor {
                    name: constructor.into(),
                    args: args.to_vec(),
                })
                .map(TypeConstructorExpr::ProperType)
                .map_err(
                    |err| PartialConstructorElaborationError::ArgumentLoweringFailed {
                        constructor: constructor.to_string(),
                        reason: err.to_string(),
                        span,
                    },
                );
        }

        let mut partial_args = Vec::with_capacity(args.len());
        let mut hole_metadata = Vec::with_capacity(1);
        for arg in args {
            match arg {
                SurfaceType::Hole { span: hole_span } => {
                    let id = TypeHoleId::new(hole_metadata.len() as u64);
                    partial_args.push(PartialTypeArg::Hole(id));
                    hole_metadata.push(TypeHoleMetadata::new(
                        id,
                        span_anchor(*hole_span, "type hole"),
                        Some(Kind::Type),
                        TypeHoleAmbiguity::ExpectedValueSlot,
                    ));
                }
                SurfaceType::AssociatedFamilyProjection { span, .. } => {
                    if surface_type_contains_hole(arg) {
                        return Err(PartialConstructorElaborationError::NoInversionBoundary {
                            context: "associated-family projection output".to_string(),
                            span: *span,
                        });
                    }
                    partial_args.push(PartialTypeArg::Applied(Box::new(
                        self.lower_surface_type_to_canonical(arg).map_err(|err| {
                            PartialConstructorElaborationError::ArgumentLoweringFailed {
                                constructor: constructor.to_string(),
                                reason: err.to_string(),
                                span: *span,
                            }
                        })?,
                    )));
                }
                SurfaceType::Associated { .. } if surface_type_contains_hole(arg) => {
                    return Err(PartialConstructorElaborationError::NoInversionBoundary {
                        context: "associated projection".to_string(),
                        span,
                    });
                }
                other if surface_type_contains_hole(other) => {
                    return Err(
                        PartialConstructorElaborationError::UnsupportedHolePosition {
                            reason: "nested holes are not enabled for MVP partial targets"
                                .to_string(),
                            span,
                        },
                    );
                }
                other => partial_args.push(PartialTypeArg::Applied(Box::new(
                    self.lower_surface_type_to_canonical(other).map_err(|err| {
                        PartialConstructorElaborationError::ArgumentLoweringFailed {
                            constructor: constructor.to_string(),
                            reason: err.to_string(),
                            span,
                        }
                    })?,
                ))),
            }
        }

        Ok(TypeConstructorExpr::PartialApplication(
            PartialTypeConstructorApp::new_with_hole_metadata(
                TypeConstructorHeadId::nominal(origin, constructor.to_string()),
                partial_args,
                Kind::n_ary(hole_count),
                hole_metadata,
                Some(span_anchor(
                    span,
                    format!("partial application {constructor}"),
                )),
            ),
        ))
    }

    fn type_constructor_arity_for_visible_name(&self, name: &str) -> Option<usize> {
        match name {
            "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" | "()" => {
                Some(0)
            }
            _ => self
                .type_info
                .get(name)
                .map(TypeInfo::type_arg_count)
                .or_else(|| self.ast_types.get(name).map(|def| def.params.len())),
        }
    }

    #[must_use]
    pub fn type_identity_for_name(&self, name: &str) -> Option<&TypeDeclId> {
        self.type_alias_identities.get(name)
    }

    #[must_use]
    pub fn interface_identity_for_name(&self, name: &str) -> Option<&InterfaceIdentityId> {
        self.interface_identity_aliases.get(name)
    }

    #[must_use]
    pub fn associated_member_identity_for_interface_member(
        &self,
        interface_name: &str,
        member_name: &str,
    ) -> Option<&AssociatedMemberIdentityId> {
        self.associated_member_identity_aliases
            .get(&(interface_name.to_string(), member_name.to_string()))
    }

    #[must_use]
    pub fn interface_identity_known(&self, id: &InterfaceIdentityId) -> bool {
        self.known_interface_identities.contains(id)
    }

    #[must_use]
    pub fn associated_member_identity_known(&self, id: &AssociatedMemberIdentityId) -> bool {
        self.known_associated_member_identities.contains(id)
    }

    #[must_use]
    pub fn canonical_type_name(&self, id: &TypeDeclId) -> Option<&String> {
        self.canonical_type_names.get(id)
    }

    fn canonical_constructor_name_for_equality(&self, name: &QualifiedName) -> QualifiedName {
        if !name.is_root() {
            return name.clone();
        }

        self.type_alias_identities
            .get(name.name.as_str())
            .and_then(|id| self.canonical_type_names.get(id))
            .map(|canonical| QualifiedName::root(canonical.clone()))
            .unwrap_or_else(|| name.clone())
    }

    fn associated_member_identity_for_visible_interface_member(
        &self,
        interface_name: &str,
        member_name: &str,
    ) -> Option<&AssociatedMemberIdentityId> {
        if let Some(member) =
            self.associated_member_identity_for_interface_member(interface_name, member_name)
        {
            return Some(member);
        }

        let interface_id = self.interface_identity_for_name(interface_name)?;
        self.associated_member_identity_aliases
            .iter()
            .find_map(|((_, visible_member), member)| {
                (visible_member == member_name && &member.interface == interface_id)
                    .then_some(member)
            })
    }

    fn canonical_associated_projection_for_equality(
        &self,
        interface_name: &str,
        member_name: &str,
    ) -> Option<(String, String)> {
        let interface_id = self.interface_identity_for_name(interface_name)?;
        let member_id = self
            .associated_member_identity_for_visible_interface_member(interface_name, member_name)?;

        if &member_id.interface != interface_id {
            return None;
        }

        let canonical_interface = self
            .canonical_interface_names
            .get(interface_id)
            .cloned()
            .unwrap_or_else(|| interface_name.to_string());

        Some((canonical_interface, member_id.name.clone()))
    }

    /// Returns the canonical target of a transparent nominal alias application
    /// when all alias arguments are representable in the current type API.
    ///
    /// This helper is intentionally narrow for the Phase 112 normalizer: it only
    /// peels already-registered transparent aliases at normalizer inputs and does
    /// not force associated projections or install new equality forcing points.
    #[must_use]
    pub fn transparent_alias_canonical_target(
        &self,
        origin: &TypeDeclId,
        visible_name: &str,
        args: &[CanonicalTypeExpr],
    ) -> Option<CanonicalTypeExpr> {
        let registered_origin = self
            .type_identity_for_name(visible_name)
            .cloned()
            .unwrap_or_else(|| fallback_canonical_type_decl_id(visible_name));
        if registered_origin != *origin {
            return None;
        }
        let mut bridge = AliasCanonicalVarBridge::default();
        let type_args: Vec<_> = args
            .iter()
            .map(|arg| bridge.placeholder_for_arg(arg))
            .collect();
        let target =
            self.transparent_alias_target(&QualifiedName::root(visible_name), &type_args)?;
        self.type_to_canonical_expr_for_alias(&target, &bridge)
            .map(|target| self.canonical_expr_with_registered_origin(target))
    }

    fn canonical_expr_with_registered_origin(&self, expr: CanonicalTypeExpr) -> CanonicalTypeExpr {
        match expr {
            CanonicalTypeExpr::NominalApp {
                visible_name,
                args,
                kind,
                origin,
            } => CanonicalTypeExpr::NominalApp {
                origin: self
                    .type_identity_for_name(&visible_name)
                    .cloned()
                    .unwrap_or(origin),
                visible_name,
                args,
                kind,
            },
            other => other,
        }
    }

    fn type_to_canonical_expr_for_alias(
        &self,
        ty: &Type,
        bridge: &AliasCanonicalVarBridge,
    ) -> Option<CanonicalTypeExpr> {
        match ty {
            Type::Int => Some(CanonicalTypeExpr::Primitive("Int".to_string())),
            Type::String => Some(CanonicalTypeExpr::Primitive("String".to_string())),
            Type::Bool => Some(CanonicalTypeExpr::Primitive("Bool".to_string())),
            Type::Float => Some(CanonicalTypeExpr::Primitive("Float".to_string())),
            Type::Null => Some(CanonicalTypeExpr::Primitive("Null".to_string())),
            Type::Time => Some(CanonicalTypeExpr::Primitive("Time".to_string())),
            Type::Ref => Some(CanonicalTypeExpr::Primitive("Ref".to_string())),
            Type::Var(var) => bridge
                .args
                .get(var)
                .cloned()
                .or_else(|| Some(CanonicalTypeExpr::Var(format!("T{}", var.0)))),
            Type::Constructor { name, args, kind } if name.is_root() => {
                let args = args
                    .iter()
                    .map(|arg| self.type_to_canonical_expr_for_alias(arg, bridge))
                    .collect::<Option<_>>()?;
                Some(CanonicalTypeExpr::NominalApp {
                    origin: self
                        .type_identity_for_name(&name.name)
                        .cloned()
                        .unwrap_or_else(|| fallback_canonical_type_decl_id(&name.name)),
                    visible_name: name.name.clone(),
                    args,
                    kind: kind.clone(),
                })
            }
            Type::Associated {
                interface,
                base,
                name,
            } => {
                let base = self.type_to_canonical_expr_for_alias(base, bridge)?;
                self.lower_associated_projection_to_canonical(&base, name)
                    .ok()
                    .map(|projection| match projection {
                        CanonicalTypeExpr::Projection {
                            interface: projection_interface,
                            member,
                            args,
                            kind,
                            rigidity,
                        } if projection_interface.name == *interface => {
                            CanonicalTypeExpr::Projection {
                                interface: projection_interface,
                                member,
                                args,
                                kind,
                                rigidity,
                            }
                        }
                        other => other,
                    })
            }
            Type::List(_)
            | Type::Record(_)
            | Type::Cap { .. }
            | Type::Fun(_, _, _)
            | Type::Fn(_, _)
            | Type::ConstructorVariableApp { .. }
            | Type::Instance { .. }
            | Type::InstanceAddr { .. }
            | Type::ControlLink { .. }
            | Type::Constructor { .. } => None,
        }
    }

    /// Recursively peel registered transparent aliases inside a type without
    /// changing current equality/unification boundaries. This helper is for
    /// later boundary adoption tasks; callers that want existing nominal
    /// equality behavior should continue using `canonicalize_type_for_equality`.
    #[must_use]
    pub fn canonicalize_transparent_aliases(&self, ty: &Type) -> Type {
        match ty {
            Type::Constructor { name, args, kind } => {
                let canonical_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.canonicalize_transparent_aliases(arg))
                    .collect();

                if let Some(target) = self.transparent_alias_target(name, &canonical_args) {
                    self.canonicalize_transparent_aliases(&target)
                } else {
                    Type::Constructor {
                        name: name.clone(),
                        args: canonical_args,
                        kind: kind.clone(),
                    }
                }
            }
            Type::ConstructorVariableApp {
                constructor,
                args,
                kind,
            } => Type::ConstructorVariableApp {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.canonicalize_transparent_aliases(arg))
                    .collect(),
                kind: kind.clone(),
            },
            Type::List(inner) => Type::List(Box::new(self.canonicalize_transparent_aliases(inner))),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.canonicalize_transparent_aliases(ty)))
                    .collect(),
            ),
            Type::Fn(params, ret) => Type::Fn(
                params
                    .iter()
                    .map(|param| self.canonicalize_transparent_aliases(param))
                    .collect(),
                Box::new(self.canonicalize_transparent_aliases(ret)),
            ),
            Type::Fun(params, ret, effect) => Type::Fun(
                params
                    .iter()
                    .map(|param| self.canonicalize_transparent_aliases(param))
                    .collect(),
                Box::new(self.canonicalize_transparent_aliases(ret)),
                *effect,
            ),
            Type::Associated {
                interface,
                base,
                name,
            } => Type::Associated {
                interface: interface.clone(),
                base: Box::new(self.canonicalize_transparent_aliases(base)),
                name: name.clone(),
            },
            other => other.clone(),
        }
    }

    #[must_use]
    pub fn render_type_for_diagnostics(&self, ty: &Type) -> String {
        ty.to_string()
    }

    #[must_use]
    pub fn canonicalize_type_for_equality(&self, ty: &Type) -> Type {
        match ty {
            Type::Constructor { name, args, kind } => {
                let canonical_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.canonicalize_type_for_equality(arg))
                    .collect();

                if let Some(target) = self.transparent_alias_target(name, &canonical_args) {
                    self.canonicalize_type_for_equality(&target)
                } else {
                    Type::Constructor {
                        name: self.canonical_constructor_name_for_equality(name),
                        args: canonical_args,
                        kind: kind.clone(),
                    }
                }
            }
            Type::ConstructorVariableApp {
                constructor,
                args,
                kind,
            } => Type::ConstructorVariableApp {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.canonicalize_type_for_equality(arg))
                    .collect(),
                kind: kind.clone(),
            },
            Type::List(inner) => Type::List(Box::new(self.canonicalize_type_for_equality(inner))),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.canonicalize_type_for_equality(ty)))
                    .collect(),
            ),
            Type::Fn(params, ret) => Type::Fn(
                params
                    .iter()
                    .map(|param| self.canonicalize_type_for_equality(param))
                    .collect(),
                Box::new(self.canonicalize_type_for_equality(ret)),
            ),
            Type::Fun(params, ret, effect) => Type::Fun(
                params
                    .iter()
                    .map(|param| self.canonicalize_type_for_equality(param))
                    .collect(),
                Box::new(self.canonicalize_type_for_equality(ret)),
                *effect,
            ),
            Type::Associated {
                interface,
                base,
                name,
            } => {
                let (canonical_interface, canonical_name) = self
                    .canonical_associated_projection_for_equality(interface, name)
                    .unwrap_or_else(|| (interface.clone(), name.clone()));

                Type::Associated {
                    interface: canonical_interface,
                    base: Box::new(self.canonicalize_type_for_equality(base)),
                    name: canonical_name,
                }
            }
            other => other.clone(),
        }
    }

    /// Canonicalize a scrutinee type for pattern typing and exhaustiveness.
    ///
    /// Unlike equality canonicalization, this API only succeeds when the result
    /// is a concrete ordinary enum ADT with a known constructor universe.
    #[must_use]
    pub fn canonicalize_type_for_pattern(&self, ty: &Type) -> PatternCanonicalization {
        let source_type = ty.clone();
        let candidate = match self.pattern_canonical_candidate_type(ty) {
            Ok(candidate) => candidate,
            Err(reason) => {
                return PatternCanonicalization::Blocked {
                    source_type,
                    reason,
                };
            }
        };

        let Type::Constructor { name, args, kind } = candidate else {
            return PatternCanonicalization::Blocked {
                source_type,
                reason: PatternCanonicalizationBlockedReason::NonAdt,
            };
        };

        if !name.is_root() {
            return PatternCanonicalization::Blocked {
                source_type,
                reason: PatternCanonicalizationBlockedReason::UnknownType { name },
            };
        }

        let canonical_name = self.canonical_constructor_name_for_equality(&name);
        let canonical_type = Type::Constructor {
            name: canonical_name.clone(),
            args: args.clone(),
            kind,
        };

        if args.iter().any(Self::pattern_type_contains_unresolved_var) {
            return PatternCanonicalization::Blocked {
                source_type,
                reason: PatternCanonicalizationBlockedReason::NonConcreteTypeArgument,
            };
        }

        match self.pattern_constructors_for_adt(&canonical_name, &args) {
            Ok(constructors) => PatternCanonicalization::Matchable(PatternCanonicalType {
                source_type,
                canonical_type,
                canonical_name,
                canonical_type_args: args,
                constructors,
            }),
            Err(reason) => PatternCanonicalization::Blocked {
                source_type,
                reason,
            },
        }
    }

    fn pattern_canonical_candidate_type(
        &self,
        ty: &Type,
    ) -> Result<Type, PatternCanonicalizationBlockedReason> {
        match ty {
            Type::Associated {
                interface, name, ..
            } => self
                .pattern_normalize_associated_projection(ty)
                .map_err(
                    |()| PatternCanonicalizationBlockedReason::RigidAssociatedProjection {
                        interface: interface.clone(),
                        member: name.clone(),
                    },
                ),
            Type::Var(_) => Err(PatternCanonicalizationBlockedReason::TypeVariable),
            Type::ConstructorVariableApp { constructor, .. } => Err(
                PatternCanonicalizationBlockedReason::ConstructorVariableApplication {
                    constructor: constructor.clone(),
                },
            ),
            _ => Ok(self.canonicalize_type_for_equality(ty)),
        }
    }

    fn pattern_type_contains_unresolved_var(ty: &Type) -> bool {
        match ty {
            Type::Var(_) => true,
            Type::List(inner) => Self::pattern_type_contains_unresolved_var(inner),
            Type::Record(fields) => fields
                .iter()
                .any(|(_, field_ty)| Self::pattern_type_contains_unresolved_var(field_ty)),
            Type::Fn(params, ret) => {
                params
                    .iter()
                    .any(Self::pattern_type_contains_unresolved_var)
                    || Self::pattern_type_contains_unresolved_var(ret)
            }
            Type::Fun(params, ret, _) => {
                params
                    .iter()
                    .any(Self::pattern_type_contains_unresolved_var)
                    || Self::pattern_type_contains_unresolved_var(ret)
            }
            Type::Constructor { args, .. } | Type::ConstructorVariableApp { args, .. } => {
                args.iter().any(Self::pattern_type_contains_unresolved_var)
            }
            Type::Associated { base, .. } => Self::pattern_type_contains_unresolved_var(base),
            Type::Int
            | Type::String
            | Type::Bool
            | Type::Float
            | Type::Null
            | Type::Time
            | Type::Ref
            | Type::Cap { .. }
            | Type::Instance { .. }
            | Type::InstanceAddr { .. }
            | Type::ControlLink { .. } => false,
        }
    }

    fn pattern_normalize_associated_projection(&self, ty: &Type) -> Result<Type, ()> {
        let canonical = self.type_to_canonical_expr_for_equality(ty).ok_or(())?;
        let outcome = Normalizer::new(self)
            .normalize(&canonical)
            .map_err(|_| ())?;
        self.normal_type_to_pattern_type(&outcome.normal).ok_or(())
    }

    fn normal_type_to_pattern_type(&self, normal: &NormalTypeExpr) -> Option<Type> {
        match normal {
            NormalTypeExpr::Primitive(name) => primitive_pattern_type(name),
            NormalTypeExpr::NominalApp {
                visible_name,
                args,
                kind,
                ..
            } => {
                let args = args
                    .iter()
                    .map(|arg| self.normal_type_to_pattern_type(arg))
                    .collect::<Option<Vec<_>>>()?;
                let name = self.canonical_constructor_name_for_equality(&QualifiedName::root(
                    visible_name.clone(),
                ));
                Some(Type::Constructor {
                    name,
                    args,
                    kind: kind.clone(),
                })
            }
            NormalTypeExpr::Var(_)
            | NormalTypeExpr::ConstructorVariableApp { .. }
            | NormalTypeExpr::NeutralComputationApp { .. }
            | NormalTypeExpr::Projection { .. }
            | NormalTypeExpr::DomainConstructorApp { .. }
            | NormalTypeExpr::PromotedDataConstructorApp { .. } => None,
        }
    }

    fn pattern_constructors_for_adt(
        &self,
        name: &QualifiedName,
        args: &[Type],
    ) -> Result<Vec<PatternCanonicalConstructor>, PatternCanonicalizationBlockedReason> {
        let unfolded = self.unfold_constructor(name, args).map_err(|_| {
            PatternCanonicalizationBlockedReason::UnknownType { name: name.clone() }
        })?;

        let UnfoldedBody::Enum(variants) = unfolded else {
            return Err(PatternCanonicalizationBlockedReason::NonAdt);
        };

        let mut constructors = Vec::with_capacity(variants.len());
        for (variant_index, variant) in variants.into_iter().enumerate() {
            match self.constructors.get(&variant.name) {
                Some((constructor_type, constructor_index))
                    if constructor_type == &name.name && *constructor_index == variant_index => {}
                _ => {
                    return Err(
                        PatternCanonicalizationBlockedReason::UnknownConstructorUniverse {
                            name: name.clone(),
                        },
                    );
                }
            }

            constructors.push(PatternCanonicalConstructor {
                name: variant.name,
                variant_index,
                fields: variant.fields,
                payload_shape: variant.payload_shape,
            });
        }

        Ok(constructors)
    }

    /// Unify types using TypeEnv's canonical imported-summary identity map.
    pub fn unify_types(&self, left: &Type, right: &Type) -> Result<Substitution, UnifyError> {
        if self
            .definitionally_equal_types_when_canonicalizable(left, right)
            .is_some_and(|equal| equal)
        {
            return Ok(Substitution::new());
        }

        unify(
            &self.canonicalize_type_for_equality(left),
            &self.canonicalize_type_for_equality(right),
        )
    }

    #[must_use]
    pub fn types_equivalent_for_equality(&self, left: &Type, right: &Type) -> bool {
        self.definitionally_equal_types_when_canonicalizable(left, right)
            .unwrap_or_else(|| self.unify_types(left, right).is_ok())
    }

    /// TASK-826 guarded TypeEnv forcing-point helper.
    ///
    /// This wrapper consumes the TASK-817 matrix only at the central TypeEnv
    /// equality boundary: if both current `Type` values can be represented in the
    /// Phase 110 canonical IR, compare their normal forms through the SPEC-060
    /// normalizer/definitional-equality API. Unsupported legacy shapes and
    /// inference-meta solving remain owned by the fallback `Type` unifier.
    #[must_use]
    fn definitionally_equal_types_when_canonicalizable(
        &self,
        left: &Type,
        right: &Type,
    ) -> Option<bool> {
        let left = self.canonicalize_type_for_equality(left);
        let right = self.canonicalize_type_for_equality(right);
        let left = self.type_to_canonical_expr_for_equality(&left)?;
        let right = self.type_to_canonical_expr_for_equality(&right)?;
        let evidence = Normalizer::new(self)
            .definitional_equality(&left, &right)
            .ok()?;
        Some(matches!(evidence, DefinitionalEqualityResult::Equal))
    }

    #[must_use]
    pub fn lower_type_to_canonical_for_equality(&self, ty: &Type) -> Option<CanonicalTypeExpr> {
        let ty = self.canonicalize_type_for_equality(ty);
        self.type_to_canonical_expr_for_equality(&ty)
    }

    fn type_to_canonical_expr_for_equality(&self, ty: &Type) -> Option<CanonicalTypeExpr> {
        match ty {
            Type::Int => Some(CanonicalTypeExpr::Primitive("Int".to_string())),
            Type::String => Some(CanonicalTypeExpr::Primitive("String".to_string())),
            Type::Bool => Some(CanonicalTypeExpr::Primitive("Bool".to_string())),
            Type::Float => Some(CanonicalTypeExpr::Primitive("Float".to_string())),
            Type::Null => Some(CanonicalTypeExpr::Primitive("Null".to_string())),
            Type::Time => Some(CanonicalTypeExpr::Primitive("Time".to_string())),
            Type::Ref => Some(CanonicalTypeExpr::Primitive("Ref".to_string())),
            Type::Var(_) => None,
            Type::Constructor { name, args, kind } if name.is_root() => {
                let args = args
                    .iter()
                    .map(|arg| self.type_to_canonical_expr_for_equality(arg))
                    .collect::<Option<_>>()?;
                let canonical_name = self.canonical_constructor_name_for_equality(name);
                Some(CanonicalTypeExpr::NominalApp {
                    origin: self
                        .type_identity_for_name(&canonical_name.name)
                        .cloned()
                        .unwrap_or_else(|| fallback_canonical_type_decl_id(&canonical_name.name)),
                    visible_name: canonical_name.name,
                    args,
                    kind: kind.clone(),
                })
            }
            Type::ConstructorVariableApp {
                constructor,
                args,
                kind,
            } => {
                let args: Vec<CanonicalTypeExpr> = args
                    .iter()
                    .map(|arg| self.type_to_canonical_expr_for_equality(arg))
                    .collect::<Option<_>>()?;
                let constructor_kind = self
                    .type_parameter_kind(constructor)
                    .cloned()
                    .unwrap_or_else(|| Kind::n_ary(args.len()));
                Some(CanonicalTypeExpr::ConstructorVariableApp(Box::new(
                    ConstructorVariableApp::new(
                        ConstructorVariableRef::new(constructor.clone(), constructor_kind, None),
                        args,
                        kind.clone(),
                        None,
                    ),
                )))
            }
            Type::Associated {
                interface,
                base,
                name,
            } => {
                let (canonical_interface, canonical_name) = self
                    .canonical_associated_projection_for_equality(interface, name)
                    .unwrap_or_else(|| (interface.clone(), name.clone()));
                if let Type::Var(var) = base.as_ref() {
                    if canonical_interface.is_empty() {
                        return None;
                    }
                    let interface_id = self
                        .interface_identity_for_name(&canonical_interface)?
                        .clone();
                    let member = self
                        .associated_member_identity_for_interface_member(
                            &canonical_interface,
                            &canonical_name,
                        )?
                        .clone();
                    return Some(CanonicalTypeExpr::Projection {
                        interface: interface_id,
                        member,
                        args: vec![CanonicalTypeExpr::Var(format!("_t{}", var.0))],
                        kind: Kind::Type,
                        rigidity: ProjectionRigidity::Rigid,
                    });
                }
                let base = self.type_to_canonical_expr_for_equality(base)?;
                self.lower_associated_projection_to_canonical(&base, &canonical_name)
                    .ok()
                    .map(|projection| match projection {
                        CanonicalTypeExpr::Projection {
                            interface,
                            member,
                            args,
                            kind,
                            rigidity,
                        } if interface.name == canonical_interface => {
                            let canonical_interface_id = self
                                .interface_identity_for_name(&canonical_interface)
                                .cloned()
                                .unwrap_or(interface);
                            CanonicalTypeExpr::Projection {
                                interface: canonical_interface_id,
                                member,
                                args,
                                kind,
                                rigidity,
                            }
                        }
                        other => other,
                    })
            }
            Type::List(_)
            | Type::Record(_)
            | Type::Cap { .. }
            | Type::Fun(_, _, _)
            | Type::Fn(_, _)
            | Type::Instance { .. }
            | Type::InstanceAddr { .. }
            | Type::ControlLink { .. }
            | Type::Constructor { .. } => None,
        }
    }

    /// Register an interface declaration.
    pub fn register_interface(&mut self, def: &InterfaceDef) -> Result<(), TypeEnvError> {
        let interface_name = def.name.to_string();
        if self.interfaces.contains_key(&interface_name) {
            return Err(TypeEnvError::DuplicateInterface(
                interface_name,
                Span::default(),
            ));
        }
        let has_sealed_family = def
            .associated_types
            .iter()
            .any(|associated| matches!(associated.kind, AssociatedTypeKind::SealedFamily { .. }));
        let owner_module = if has_sealed_family {
            Some(self.current_module_identity.clone().ok_or_else(|| {
                TypeEnvError::AssociatedFamilyModuleOwnerViolation {
                    family: def
                        .associated_types
                        .iter()
                        .find(|associated| {
                            matches!(associated.kind, AssociatedTypeKind::SealedFamily { .. })
                        })
                        .map(|associated| associated.name.to_string())
                        .unwrap_or_else(|| "<unknown>".to_string()),
                    reason: "missing current module identity while registering sealed family declaration"
                        .to_string(),
                    span: def.span,
                }
            })?)
        } else {
            None
        };

        let interface_param_domains = def
            .type_params
            .iter()
            .map(|param| {
                self.optional_param_domain_constraint(param.domain.as_ref(), param.span)
                    .map(|domain_constraint| AssociatedFamilyInterfaceParamInfo {
                        name: param.name.to_string(),
                        domain_constraint,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut seen_associated_names: HashMap<String, bool> = HashMap::new();
        for associated in &def.associated_types {
            let is_family = matches!(associated.kind, AssociatedTypeKind::SealedFamily { .. });
            if let Some(previous_was_family) =
                seen_associated_names.insert(associated.name.to_string(), is_family)
            {
                if previous_was_family || is_family {
                    return Err(TypeEnvError::DuplicateAssociatedFamilyHead {
                        interface: interface_name.clone(),
                        family: associated.name.to_string(),
                        span: associated.span,
                    });
                }
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate associated type '{}' in interface '{}'",
                        associated.name, interface_name
                    ),
                    associated.span,
                ));
            }
        }

        if owner_module.is_some() {
            for associated in &def.associated_types {
                let AssociatedTypeKind::SealedFamily {
                    result_domain,
                    decreases,
                    ..
                } = &associated.kind
                else {
                    continue;
                };
                let family_name = associated.name.to_string();
                self.associated_family_result_constraint_from_surface(
                    result_domain,
                    associated.span,
                )
                .map_err(|err| match err {
                    TypeEnvError::WrongAssociatedFamilyResultDomain { reason, span, .. } => {
                        TypeEnvError::WrongAssociatedFamilyResultDomain {
                            family: family_name.clone(),
                            reason,
                            span,
                        }
                    }
                    other => other,
                })?;
                if let Some(decreases) = decreases {
                    let Some(param) = interface_param_domains
                        .iter()
                        .find(|param| param.name == decreases.param.as_ref())
                    else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' is not an interface parameter for associated family '{}::{}'",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    };
                    if param.domain_constraint.is_none() {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' for associated family '{}::{}' must have a sealed-domain constraint",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    }
                }
            }
        }

        let param_mapping: HashMap<String, TypeVar> = def
            .type_params
            .iter()
            .map(|param| (param.to_string(), TypeVar::fresh()))
            .collect();

        let ordered_param_names: Vec<String> =
            def.type_params.iter().map(ToString::to_string).collect();
        let type_param_kinds = interface_param_kinds(&def.type_params);
        let interface_type_params = def
            .type_params
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let associated_types = def
            .associated_types
            .iter()
            .map(|a| a.name.to_string())
            .collect::<Vec<_>>();

        // Make the interface's own arity visible while converting method
        // signatures. Existing interface syntax uses the interface name as the
        // nominal head in method parameters (for example `Pair<A, B>`), which
        // may coexist with a zero-arity ordinary carrier type named `Pair`.
        self.interfaces.insert(
            interface_name.clone(),
            InterfaceInfo {
                name: interface_name.clone(),
                visibility: core_visibility_from_surface(&def.visibility),
                type_params: interface_type_params.clone(),
                type_param_kinds: type_param_kinds.clone(),
                associated_types: associated_types.clone(),
                methods: HashMap::new(),
            },
        );

        let mut method_env = self.clone();
        for (name, kind) in interface_type_params.iter().zip(type_param_kinds.iter()) {
            method_env.register_type_parameter_kind(name, kind.clone())?;
        }

        let methods = match def
            .methods
            .iter()
            .map(|method| {
                method_env.convert_interface_method(
                    method,
                    &param_mapping,
                    &ordered_param_names,
                    &interface_name,
                )
            })
            .collect::<Result<HashMap<_, _>, _>>()
        {
            Ok(methods) => methods,
            Err(error) => {
                self.interfaces.remove(&interface_name);
                return Err(error);
            }
        };

        self.interfaces.insert(
            interface_name.clone(),
            InterfaceInfo {
                name: interface_name.clone(),
                visibility: core_visibility_from_surface(&def.visibility),
                type_params: interface_type_params.clone(),
                type_param_kinds: type_param_kinds.clone(),
                associated_types: associated_types.clone(),
                methods: methods.clone(),
            },
        );
        if let Some(current_module) = self.current_module_identity.clone() {
            let interface_id =
                self.ensure_local_interface_identity(&interface_name, &current_module);
            self.local_interface_arities
                .insert(interface_id.clone(), def.type_params.len());
            for associated in &def.associated_types {
                self.ensure_local_associated_member_identity(
                    &interface_name,
                    &interface_id,
                    associated.name.as_ref(),
                );
            }
        }
        if let Some(owner_module) = owner_module {
            let interface_id = self.ensure_local_interface_identity(&interface_name, &owner_module);
            self.local_interface_arities
                .insert(interface_id.clone(), def.type_params.len());
            for associated in &def.associated_types {
                let AssociatedTypeKind::SealedFamily {
                    result_domain,
                    decreases,
                    ..
                } = &associated.kind
                else {
                    continue;
                };
                let family_name = associated.name.to_string();
                let result_domain = self
                    .associated_family_result_constraint_from_surface(
                        result_domain,
                        associated.span,
                    )
                    .map_err(|err| match err {
                        TypeEnvError::WrongAssociatedFamilyResultDomain {
                            reason, span, ..
                        } => TypeEnvError::WrongAssociatedFamilyResultDomain {
                            family: family_name.clone(),
                            reason,
                            span,
                        },
                        other => other,
                    })?;
                if let Some(decreases) = decreases {
                    let Some(param) = interface_param_domains
                        .iter()
                        .find(|param| param.name == decreases.param.as_ref())
                    else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' is not an interface parameter for associated family '{}::{}'",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    };
                    if param.domain_constraint.is_none() {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "decreases parameter '{}' for associated family '{}::{}' must have a sealed-domain constraint",
                                decreases.param, interface_name, family_name
                            ),
                            decreases.span,
                        ));
                    }
                }
                let member = self.ensure_local_associated_member_identity(
                    &interface_name,
                    &interface_id,
                    &family_name,
                );
                let head = AssociatedFamilyHeadId {
                    interface: interface_id.clone(),
                    member,
                };
                if self.associated_family_declarations.contains_key(&head) {
                    return Err(TypeEnvError::DuplicateAssociatedFamilyHead {
                        interface: interface_name.clone(),
                        family: family_name,
                        span: associated.span,
                    });
                }
                let declaration = AssociatedFamilyDeclarationInfo {
                    defining_module: owner_module.clone(),
                    result_domain,
                    decreases: decreases
                        .as_ref()
                        .map(|decreases| decreases.param.to_string()),
                    interface_params: interface_param_domains.clone(),
                    head: head.clone(),
                };
                self.associated_family_name_index.insert(
                    (interface_name.clone(), associated.name.to_string()),
                    head.clone(),
                );
                self.associated_family_declarations
                    .insert(head, declaration);
            }
        }
        if let Some(interface_id) = self.interface_identity_for_name(&interface_name).cloned() {
            let imported = self
                .interface_identity_alias_is_imported
                .get(&interface_name)
                .copied()
                .unwrap_or(false);
            if !imported {
                self.local_interface_arities
                    .insert(interface_id, def.type_params.len());
            }
        }
        if interface_name == "Monad" {
            self.register_compiler_prelude_tower_monad_evidence()?;
        }
        Ok(())
    }

    fn register_compiler_prelude_tower_monad_evidence(&mut self) -> Result<(), TypeEnvError> {
        let interface =
            self.interfaces.get("Monad").cloned().ok_or_else(|| {
                TypeEnvError::MissingInterface("Monad".to_string(), Span::default())
            })?;
        let expected_methods = ["unit", "bind"];
        if !expected_methods
            .iter()
            .all(|method| interface.methods.contains_key(*method))
        {
            return Ok(());
        }

        for carrier in ["Act", "Proc", "Workflow"] {
            if !self.has_type(carrier) {
                continue;
            }
            let surface_args = [SurfaceType::Name(carrier.into())];
            let head_args = self.lower_interface_evidence_args(
                "Monad",
                &interface,
                &surface_args,
                &HashMap::new(),
            )?;
            if self.impls.iter().any(|scheme| {
                scheme.interface == "Monad"
                    && interface_evidence_args_match(&scheme.head_args, &head_args, false)
            }) {
                continue;
            }
            let lowered_type_args: Vec<Type> = head_args
                .iter()
                .map(interface_evidence_arg_as_legacy_type)
                .collect();
            self.impls.push(ImplScheme {
                interface: "Monad".to_string(),
                type_params: Vec::new(),
                head: Type::Constructor {
                    name: QualifiedName::root("Monad"),
                    args: lowered_type_args,
                    kind: Kind::Type,
                },
                head_args,
                where_bounds: Vec::new(),
                associated_type_bindings: HashMap::new(),
                methods: Vec::new(),
            });
        }

        Ok(())
    }

    fn validate_associated_family_scheme_totality(
        &self,
        family: &str,
        declaration: &AssociatedFamilyDeclarationInfo,
        scheme: &AssociatedFamilyScheme,
        require_coverage: bool,
    ) -> Result<(), TypeEnvError> {
        let scheme_param_names = scheme
            .params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<HashSet<_>>();
        let recursive = scheme.equations.iter().any(|equation| {
            Self::associated_family_result_contains_head_with_scheme_param_arg(
                &equation.result,
                &scheme.head,
                &scheme_param_names,
            )
        });
        if !recursive {
            if require_coverage && declaration.decreases.is_some() {
                self.validate_associated_family_pattern_coverage(family, scheme)?;
            }
            return Ok(());
        }

        if scheme.equations.iter().any(|equation| {
            Self::associated_family_result_contains_other_family(&equation.result, &scheme.head)
        }) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("mutual recursion in associated family '{family}' is unsupported"),
                anchor_span(&scheme.source_anchor),
            ));
        }

        let Some(decreases) = declaration.decreases.as_deref() else {
            return Err(TypeEnvError::InvalidDefinition(
                format!("missing decreases clause for recursive associated family '{family}'"),
                anchor_span(&scheme.source_anchor),
            ));
        };

        let Some(decreasing_index) = scheme
            .params
            .iter()
            .position(|param| param.name == decreases)
        else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "unknown decreases parameter '{decreases}' in associated family '{family}'"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        };
        let Some(decreasing_domain) = scheme.params[decreasing_index].domain_constraint.as_ref()
        else {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in associated family '{family}': parameter is not a sealed domain"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        };
        if !self.domain_has_structural_subcomponent_metadata(decreasing_domain)? {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid decreases parameter '{decreases}' in associated family '{family}': sealed domain has no structural subcomponent metadata"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }

        if require_coverage {
            self.validate_associated_family_pattern_coverage(family, scheme)?;
        }

        for equation in &scheme.equations {
            let allowed = equation
                .interface_arg_patterns
                .get(decreasing_index)
                .map(|pattern| self.direct_associated_family_structural_subcomponent_vars(pattern))
                .transpose()?
                .unwrap_or_default();
            self.validate_recursive_associated_family_calls(
                family,
                &scheme.head,
                decreasing_index,
                &allowed,
                &equation.result,
                anchor_span(&equation.source_anchor),
            )?;
        }
        Ok(())
    }

    fn validate_associated_family_pattern_coverage(
        &self,
        family: &str,
        scheme: &AssociatedFamilyScheme,
    ) -> Result<(), TypeEnvError> {
        let params = scheme
            .params
            .iter()
            .map(|param| TypeFunctionParam {
                name: param.name.clone(),
                ty: param.ty.clone(),
                kind: param.kind.clone(),
                domain_constraint: param.domain_constraint.clone(),
                source_anchor: param.source_anchor.clone(),
            })
            .collect::<Vec<_>>();
        let pseudo_head = TypeComputationHeadId::new(scheme.head.interface.module.clone(), family);
        let equations = scheme
            .equations
            .iter()
            .map(|equation| {
                let patterns = equation
                    .interface_arg_patterns
                    .iter()
                    .map(Self::associated_family_pattern_to_type_function_pattern)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeFunctionEquation {
                    head: pseudo_head.clone(),
                    ordinal: equation.ordinal,
                    patterns,
                    result: TypeFunctionResultExpr::Var {
                        name: "__task865_result".to_string(),
                        kind: Kind::Type,
                        constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                        source_anchor: equation.source_anchor.clone(),
                    },
                    source_anchor: equation.source_anchor.clone(),
                    case_head_anchor: equation.case_head_anchor.clone(),
                })
            })
            .collect::<Result<Vec<_>, TypeEnvError>>()?;
        self.validate_type_function_pattern_coverage(
            family,
            &params,
            &equations,
            anchor_span(&scheme.source_anchor),
        )
    }

    fn associated_family_pattern_to_type_function_pattern(
        pattern: &AssociatedFamilyPattern,
    ) -> Result<TypeFunctionPattern, TypeEnvError> {
        match pattern {
            AssociatedFamilyPattern::DomainConstructor {
                constructor,
                domain,
                fields,
                constraint,
                source_anchor,
            } => {
                let fields = fields
                    .iter()
                    .map(Self::associated_family_pattern_to_type_function_pattern)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeFunctionPattern::DomainConstructor {
                    constructor: constructor.clone(),
                    domain: domain.clone(),
                    fields,
                    constraint: associated_family_constraint_to_type_function_pattern(constraint),
                    source_anchor: source_anchor.clone(),
                })
            }
            AssociatedFamilyPattern::Var {
                name,
                constraint,
                source_anchor,
            } => Ok(TypeFunctionPattern::Var {
                name: name.clone(),
                constraint: associated_family_constraint_to_type_function_pattern(constraint),
                source_anchor: source_anchor.clone(),
            }),
            AssociatedFamilyPattern::Wildcard {
                constraint,
                source_anchor,
            } => Ok(TypeFunctionPattern::Wildcard {
                constraint: associated_family_constraint_to_type_function_pattern(constraint),
                source_anchor: source_anchor.clone(),
            }),
            AssociatedFamilyPattern::NominalApp {
                visible_name,
                source_anchor,
                ..
            }
            | AssociatedFamilyPattern::Primitive {
                name: visible_name,
                source_anchor,
                ..
            } => Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated family coverage pattern '{visible_name}' is not a sealed-domain pattern"
                ),
                anchor_span(source_anchor),
            )),
        }
    }

    fn direct_associated_family_structural_subcomponent_vars(
        &self,
        pattern: &AssociatedFamilyPattern,
    ) -> Result<HashSet<String>, TypeEnvError> {
        let AssociatedFamilyPattern::DomainConstructor {
            constructor,
            domain,
            fields,
            ..
        } = pattern
        else {
            return Ok(HashSet::new());
        };
        let summary = self.lookup_sealed_domain_by_id(domain).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "unknown sealed domain '{}' in associated-family recursion matrix",
                    domain.name
                ),
                Span::default(),
            )
        })?;
        let Some(constructor_summary) = summary
            .constructors
            .iter()
            .find(|candidate| candidate.id == **constructor)
        else {
            return Ok(HashSet::new());
        };
        let mut vars = HashSet::new();
        for (field_pattern, field) in fields.iter().zip(&constructor_summary.fields) {
            if field.structural_status != StructuralFieldStatus::StructuralSelfDomain {
                continue;
            }
            if let AssociatedFamilyPattern::Var { name, .. } = field_pattern {
                vars.insert(name.clone());
            }
        }
        Ok(vars)
    }

    fn validate_recursive_associated_family_calls(
        &self,
        family: &str,
        self_head: &AssociatedFamilyHeadId,
        decreasing_index: usize,
        allowed_subcomponents: &HashSet<String>,
        expr: &AssociatedFamilyResultExpr,
        span: Span,
    ) -> Result<(), TypeEnvError> {
        match expr {
            AssociatedFamilyResultExpr::Primitive { .. }
            | AssociatedFamilyResultExpr::Var { .. } => Ok(()),
            AssociatedFamilyResultExpr::NominalApp { args, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { args, .. }
            | AssociatedFamilyResultExpr::Projection { args, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { args, .. } => {
                for arg in args {
                    self.validate_recursive_associated_family_calls(
                        family,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                Ok(())
            }
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                for arg in interface_args {
                    self.validate_recursive_associated_family_calls(
                        family,
                        self_head,
                        decreasing_index,
                        allowed_subcomponents,
                        arg,
                        span,
                    )?;
                }
                if head == self_head {
                    let Some(decreasing_arg) = interface_args.get(decreasing_index) else {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in associated family '{family}': missing decreasing argument"
                            ),
                            span,
                        ));
                    };
                    match decreasing_arg {
                        AssociatedFamilyResultExpr::Var { name, .. }
                            if allowed_subcomponents.contains(name) =>
                        {
                            Ok(())
                        }
                        _ => Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "non-decreasing recursive call in associated family '{family}': decreasing argument must be a direct structural subcomponent"
                            ),
                            span,
                        )),
                    }
                } else {
                    Err(TypeEnvError::InvalidDefinition(
                        format!("mutual recursion in associated family '{family}' is unsupported"),
                        span,
                    ))
                }
            }
        }
    }

    fn associated_family_result_contains_head_with_scheme_param_arg(
        expr: &AssociatedFamilyResultExpr,
        needle: &AssociatedFamilyHeadId,
        scheme_param_names: &HashSet<&str>,
    ) -> bool {
        match expr {
            AssociatedFamilyResultExpr::Primitive { .. } | AssociatedFamilyResultExpr::Var { .. } => false,
            AssociatedFamilyResultExpr::NominalApp { args, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { args, .. }
            | AssociatedFamilyResultExpr::Projection { args, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { args, .. } => args.iter().any(|arg| {
                Self::associated_family_result_contains_head_with_scheme_param_arg(
                    arg,
                    needle,
                    scheme_param_names,
                )
            }),
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                (head == needle
                    && interface_args.iter().any(|arg| {
                        matches!(arg, AssociatedFamilyResultExpr::Var { name, .. } if scheme_param_names.contains(name.as_str()))
                    }))
                    || interface_args.iter().any(|arg| {
                        Self::associated_family_result_contains_head_with_scheme_param_arg(
                            arg,
                            needle,
                            scheme_param_names,
                        )
                    })
            }
        }
    }

    fn associated_family_result_contains_other_family(
        expr: &AssociatedFamilyResultExpr,
        self_head: &AssociatedFamilyHeadId,
    ) -> bool {
        match expr {
            AssociatedFamilyResultExpr::Primitive { .. }
            | AssociatedFamilyResultExpr::Var { .. } => false,
            AssociatedFamilyResultExpr::NominalApp { args, .. }
            | AssociatedFamilyResultExpr::DomainConstructorApp { args, .. }
            | AssociatedFamilyResultExpr::Projection { args, .. }
            | AssociatedFamilyResultExpr::ComputationHeadApp { args, .. } => args
                .iter()
                .any(|arg| Self::associated_family_result_contains_other_family(arg, self_head)),
            AssociatedFamilyResultExpr::AssociatedFamilyProjection {
                head,
                interface_args,
                ..
            } => {
                head != self_head
                    || interface_args.iter().any(|arg| {
                        Self::associated_family_result_contains_other_family(arg, self_head)
                    })
            }
        }
    }

    /// Register a coherence-checked associated-family scheme for a sealed family head.
    pub fn register_associated_family_scheme(
        &mut self,
        scheme: AssociatedFamilyScheme,
        defining_module: ModuleIdentity,
    ) -> Result<(), TypeEnvError> {
        self.register_associated_family_scheme_with_totality(scheme, defining_module, true)
    }

    fn register_associated_family_scheme_with_totality(
        &mut self,
        scheme: AssociatedFamilyScheme,
        defining_module: ModuleIdentity,
        require_totality: bool,
    ) -> Result<(), TypeEnvError> {
        let declaration = self
            .associated_family_declarations
            .get(&scheme.head)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    "associated family scheme references an undeclared sealed family head"
                        .to_string(),
                    Span::default(),
                )
            })?;
        let family = declaration.head.member.name.to_string();

        if declaration.defining_module != defining_module {
            return Err(TypeEnvError::UnauthorizedAssociatedFamilyExtension {
                family,
                owner_module: declaration.defining_module,
                attempted_module: defining_module,
                span: anchor_span(&scheme.source_anchor),
            });
        }

        if scheme.result_kind != Kind::Type {
            return Err(TypeEnvError::WrongAssociatedFamilyResultKind {
                family,
                expected: format!("{:?}", Kind::Type),
                found: format!("{:?}", scheme.result_kind),
                span: anchor_span(&scheme.source_anchor),
            });
        }

        if scheme.equations.is_empty() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "associated family scheme for '{family}' must contain at least one equation"
                ),
                anchor_span(&scheme.source_anchor),
            ));
        }

        if !matches_associated_family_result_constraint(
            &scheme.result_domain,
            &declaration.result_domain,
        ) {
            return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                family: family.clone(),
                reason: "scheme result-domain annotation does not match the associated family declaration"
                    .to_string(),
                span: anchor_span(&scheme.source_anchor),
            });
        }

        for equation in &scheme.equations {
            if equation.head != scheme.head {
                return Err(TypeEnvError::InvalidDefinition(
                    "associated family scheme equation head does not match scheme head".to_string(),
                    anchor_span(&equation.source_anchor),
                ));
            }
            if equation.interface_arg_patterns.len() != declaration.interface_params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "associated family scheme for '{family}' expects {} interface argument patterns, found {}",
                        declaration.interface_params.len(),
                        equation.interface_arg_patterns.len()
                    ),
                    anchor_span(&equation.source_anchor),
                ));
            }
        }

        self.validate_associated_family_scheme_totality(
            &family,
            &declaration,
            &scheme,
            require_totality,
        )?;

        for equation in &scheme.equations {
            if !Self::associated_family_expr_conforms_to_constraint(
                &equation.result,
                &declaration.result_domain,
            ) {
                return Err(TypeEnvError::WrongAssociatedFamilyResultDomain {
                    family: family.clone(),
                    reason: format!(
                        "RHS does not conform to associated family result constraint {}",
                        associated_family_result_constraint_label(&declaration.result_domain)
                    ),
                    span: anchor_span(&equation.source_anchor),
                });
            }
        }

        if let Some(existing_schemes) = self.associated_family_schemes.get(&scheme.head) {
            for existing in existing_schemes {
                for existing_equation in &existing.scheme.equations {
                    for new_equation in &scheme.equations {
                        if Self::associated_family_pattern_spines_overlap(
                            &existing_equation.interface_arg_patterns,
                            &new_equation.interface_arg_patterns,
                        ) {
                            return Err(TypeEnvError::OverlappingAssociatedFamilyScheme {
                                family: family.clone(),
                                span: anchor_span(&new_equation.source_anchor),
                            });
                        }
                    }
                }
            }
        }

        self.associated_family_schemes
            .entry(scheme.head.clone())
            .or_default()
            .push(RegisteredAssociatedFamilyScheme {
                defining_module,
                scheme,
            });
        Ok(())
    }

    fn convert_capability_operation(
        &self,
        operation: &CapabilityOperationSig,
    ) -> Result<(String, CapabilityOperationInfo), TypeEnvError> {
        let param_names = operation
            .params
            .iter()
            .map(|param| param.name.to_string())
            .collect();
        let param_mapping = HashMap::new();
        let params = operation
            .params
            .iter()
            .map(|param| surface_type_to_type(&param.ty, &param_mapping, self))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = surface_type_to_type(&operation.return_type, &param_mapping, self)?;

        Ok((
            operation.name.to_string(),
            CapabilityOperationInfo {
                mode: operation.mode,
                param_names,
                params,
                return_type,
            },
        ))
    }

    /// Register a resource type declaration.
    pub fn register_resource_type(&mut self, def: &ResourceTypeDef) -> Result<(), TypeEnvError> {
        let resource_name = def.name.to_string();
        if self.resource_types.contains_key(&resource_name) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("resource type '{resource_name}' is already defined"),
                def.span,
            ));
        }

        let mut field_names = HashSet::with_capacity(def.fields.len());
        for field in &def.fields {
            let field_name = field.name.to_string();
            if !field_names.insert(field_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "resource type '{resource_name}' defines duplicate field '{field_name}'"
                    ),
                    field.span,
                ));
            }
        }

        let param_mapping = HashMap::new();
        let fields = def
            .fields
            .iter()
            .map(|field| {
                surface_type_to_type(&field.ty, &param_mapping, self)
                    .map(|ty| (field.name.to_string(), ty))
                    .map_err(|error| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "resource type '{resource_name}' field '{}' has invalid ordinary type: {error}",
                                field.name
                            ),
                            field.span,
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.resource_types.insert(
            resource_name.clone(),
            ResourceTypeInfo {
                name: resource_name,
                fields,
            },
        );
        Ok(())
    }

    /// Check if a resource type is registered.
    pub fn has_resource_type(&self, name: &str) -> bool {
        self.resource_types.contains_key(name)
    }

    /// Look up a registered resource type.
    pub fn lookup_resource_type(&self, name: &str) -> Option<&ResourceTypeInfo> {
        self.resource_types.get(name)
    }

    /// Register a capability interface declaration.
    pub fn register_capability_interface(
        &mut self,
        def: &CapabilityInterfaceDef,
    ) -> Result<(), TypeEnvError> {
        let interface_name = def.name.to_string();
        if self.capability_interfaces.contains_key(&interface_name) {
            return Err(TypeEnvError::InvalidDefinition(
                format!("capability interface '{interface_name}' is already defined"),
                def.span,
            ));
        }

        let mut operations = HashMap::with_capacity(def.operations.len());
        let mut operation_names = HashSet::with_capacity(def.operations.len());
        for operation in &def.operations {
            let operation_name = operation.name.to_string();
            if !operation_names.insert(operation_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability interface '{interface_name}' defines duplicate operation '{operation_name}'"
                    ),
                    operation.span,
                ));
            }
        }

        for operation in &def.operations {
            let (operation_name, operation_info) = self.convert_capability_operation(operation)?;
            operations.insert(operation_name, operation_info);
        }

        self.capability_interfaces.insert(
            interface_name.clone(),
            CapabilityInterfaceInfo {
                name: interface_name,
                operations,
            },
        );

        Ok(())
    }

    /// True if this environment is currently type-checking a capability implementation body.
    #[must_use]
    pub fn is_capability_implementation_body(&self) -> bool {
        self.capability_implementation_body
    }

    /// Register a capability implementation recipe and validate conformance to its interface.
    pub fn register_capability_implementation(
        &mut self,
        def: &CapabilityImplementationDef,
    ) -> Result<(), TypeEnvError> {
        let implementation_name = def.name.to_string();
        if self
            .capability_implementations
            .contains_key(&implementation_name)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!("capability implementation '{implementation_name}' is already defined"),
                def.span,
            ));
        }

        let interface_name = def.interface.to_string();
        let interface = self
            .capability_interfaces
            .get(&interface_name)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' targets unknown capability interface '{interface_name}'"
                    ),
                    def.span,
                )
            })?;

        let mut operation_names = HashSet::with_capacity(def.operations.len());
        for operation in &def.operations {
            let operation_name = operation.name.to_string();
            if !operation_names.insert(operation_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines duplicate operation '{operation_name}'"
                    ),
                    operation.span,
                ));
            }
        }

        for operation_name in interface.operations.keys() {
            if !operation_names.contains(operation_name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' is missing required operation '{operation_name}' for interface '{interface_name}'"
                    ),
                    def.span,
                ));
            }
        }

        for operation_name in &operation_names {
            if !interface.operations.contains_key(operation_name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines extra operation '{operation_name}' not present in interface '{interface_name}'"
                    ),
                    def.span,
                ));
            }
        }

        let dependencies = def
            .dependencies
            .iter()
            .map(|dependency| self.convert_capability_implementation_dependency(dependency))
            .collect::<Result<Vec<_>, _>>()?;
        let mut dependency_names = HashSet::with_capacity(dependencies.len());
        for dependency in &dependencies {
            if !dependency_names.insert(dependency.name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines duplicate dependency '{}'",
                        dependency.name
                    ),
                    def.span,
                ));
            }
        }

        let mut operations = HashMap::with_capacity(def.operations.len());
        for operation in &def.operations {
            let operation_name = operation.name.to_string();
            let expected = interface.operations.get(&operation_name).ok_or_else(|| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation '{implementation_name}' defines extra operation '{operation_name}' not present in interface '{interface_name}'"
                    ),
                    operation.span,
                )
            })?;
            let operation_info = self.convert_capability_implementation_operation(operation)?;

            if operation_info.mode != expected.mode {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation '{implementation_name}::{operation_name}' mode mismatch: expected {:?}, found {:?}",
                        expected.mode, operation_info.mode
                    ),
                    operation.span,
                ));
            }

            if operation_info.params.len() != expected.params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation '{implementation_name}::{operation_name}' arity mismatch: expected {} parameters, found {}",
                        expected.params.len(),
                        operation_info.params.len()
                    ),
                    operation.span,
                ));
            }

            for (index, (expected_param, actual_param)) in expected
                .params
                .iter()
                .zip(operation_info.params.iter())
                .enumerate()
            {
                if !self.types_equivalent_for_equality(expected_param, actual_param) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "capability implementation operation '{implementation_name}::{operation_name}' parameter {index} type mismatch: expected {expected_param}, found {actual_param}"
                        ),
                        operation.span,
                    ));
                }
            }

            if !self
                .types_equivalent_for_equality(&operation_info.return_type, &expected.return_type)
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation '{implementation_name}::{operation_name}' return type mismatch: expected {}, found {}",
                        expected.return_type, operation_info.return_type
                    ),
                    operation.span,
                ));
            }

            for param_name in &operation_info.param_names {
                if dependency_names.contains(param_name) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "capability implementation operation '{implementation_name}::{operation_name}' parameter '{param_name}' collides with a declared dependency name"
                        ),
                        operation.span,
                    ));
                }
            }

            self.validate_capability_implementation_operation_body(
                &implementation_name,
                operation,
                &operation_info,
                &dependencies,
            )?;

            operations.insert(operation_name, operation_info);
        }

        let authority_provenance = classify_authority_provenance(&dependencies);
        let authority_sources = implementation_authority_sources(&dependencies);

        self.capability_implementations.insert(
            implementation_name.clone(),
            CapabilityImplementationInfo {
                name: implementation_name,
                interface: interface_name,
                dependencies,
                operations,
                authority_provenance,
                authority_sources,
            },
        );

        Ok(())
    }

    fn convert_capability_implementation_dependency(
        &self,
        dependency: &CapabilityImplementationDependency,
    ) -> Result<CapabilityImplementationDependencyInfo, TypeEnvError> {
        let name = dependency.name.to_string();
        let target_name = surface_type_name(&dependency.ty).ok_or_else(|| {
            TypeEnvError::InvalidDefinition(
                format!(
                    "{:?} dependency '{name}' must name a single target type or interface",
                    dependency.kind
                ),
                dependency.span,
            )
        })?;

        match dependency.kind {
            CapabilityImplementationDependencyKind::Resource => {
                if !self.has_resource_type(&target_name) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "resource dependency '{name}' references unknown resource type '{target_name}'"
                        ),
                        dependency.span,
                    ));
                }
                Ok(CapabilityImplementationDependencyInfo {
                    kind: dependency.kind,
                    name,
                    ty: Type::Constructor {
                        name: QualifiedName::root(target_name.clone()),
                        args: vec![],
                        kind: Kind::Type,
                    },
                    target_name: Some(target_name),
                })
            }
            CapabilityImplementationDependencyKind::Capability => {
                if !self.has_capability_interface(&target_name) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "capability dependency '{name}' references unknown capability interface '{target_name}'"
                        ),
                        dependency.span,
                    ));
                }
                Ok(CapabilityImplementationDependencyInfo {
                    kind: dependency.kind,
                    name,
                    ty: Type::Cap {
                        name: Box::from(target_name.as_str()),
                        effect: ash_core::Effect::Operational,
                    },
                    target_name: Some(target_name),
                })
            }
            CapabilityImplementationDependencyKind::Config => {
                let param_mapping = HashMap::new();
                let ty = surface_type_to_type(&dependency.ty, &param_mapping, self)?;
                Ok(CapabilityImplementationDependencyInfo {
                    kind: dependency.kind,
                    name,
                    ty,
                    target_name: None,
                })
            }
        }
    }

    fn convert_capability_implementation_operation(
        &self,
        operation: &CapabilityImplementationOperation,
    ) -> Result<CapabilityImplementationOperationInfo, TypeEnvError> {
        let param_mapping = HashMap::new();
        let param_names = operation
            .params
            .iter()
            .map(|param| param.name.to_string())
            .collect();
        let params = operation
            .params
            .iter()
            .map(|param| surface_type_to_type(&param.ty, &param_mapping, self))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = surface_type_to_type(&operation.return_type, &param_mapping, self)?;
        Ok(CapabilityImplementationOperationInfo {
            mode: operation.mode,
            param_names,
            params,
            return_type,
        })
    }

    fn validate_capability_implementation_operation_body(
        &self,
        implementation_name: &str,
        operation: &CapabilityImplementationOperation,
        operation_info: &CapabilityImplementationOperationInfo,
        dependencies: &[CapabilityImplementationDependencyInfo],
    ) -> Result<(), TypeEnvError> {
        let mut body_env = self.capability_implementation_body_env(operation_info.mode);
        for dependency in dependencies {
            if !matches!(
                dependency.kind,
                CapabilityImplementationDependencyKind::Config
            ) {
                continue;
            }
            body_env.bind_variable(&dependency.name, dependency.ty.clone());
        }
        for (param_name, param_type) in operation_info
            .param_names
            .iter()
            .zip(operation_info.params.iter())
        {
            body_env.bind_variable(param_name, param_type.clone());
        }

        let body_result = crate::check_expr::check_expr(&body_env, &operation.body);
        if !body_result.is_ok() {
            let reason = body_result
                .errors
                .into_iter()
                .next()
                .map(|error| error.to_string())
                .unwrap_or_else(|| {
                    format!(
                        "failed to typecheck body for capability implementation operation '{}::{}'",
                        implementation_name, operation.name
                    )
                });
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "invalid capability implementation operation body for '{}::{}': {}",
                    implementation_name, operation.name, reason
                ),
                operation.span,
            ));
        }

        let actual_return_ty = body_result.substitution.apply(&body_result.ty);
        self.unify_types(&operation_info.return_type, &actual_return_ty)
            .map(|_| ())
            .map_err(|_| {
                TypeEnvError::InvalidDefinition(
                    format!(
                        "capability implementation operation body '{}::{}' must return {}, found {}",
                        implementation_name,
                        operation.name,
                        operation_info.return_type,
                        actual_return_ty
                    ),
                    operation.span,
                )
            })
    }

    fn capability_implementation_body_env(&self, mode: CapabilityOperationMode) -> Self {
        let mut body_env = Self {
            ast_types: self.ast_types.clone(),
            type_info: self.type_info.clone(),
            constructors: self.constructors.clone(),
            transparent_aliases: self.transparent_aliases.clone(),
            type_declaration_states: self.type_declaration_states.clone(),
            type_alias_identities: self.type_alias_identities.clone(),
            canonical_type_names: self.canonical_type_names.clone(),
            interface_identity_aliases: self.interface_identity_aliases.clone(),
            interface_identity_alias_is_imported: self.interface_identity_alias_is_imported.clone(),
            canonical_interface_names: self.canonical_interface_names.clone(),
            local_interface_arities: self.local_interface_arities.clone(),
            known_interface_identities: self.known_interface_identities.clone(),
            associated_member_identity_aliases: self.associated_member_identity_aliases.clone(),
            associated_member_identity_alias_is_imported: self
                .associated_member_identity_alias_is_imported
                .clone(),
            known_associated_member_identities: self.known_associated_member_identities.clone(),
            interfaces: self.interfaces.clone(),
            capability_interfaces: self.capability_interfaces.clone(),
            resource_types: self.resource_types.clone(),
            capability_implementations: self.capability_implementations.clone(),
            capability_bindings: HashMap::new(),
            impls: self.impls.clone(),
            proposition_assumptions: self.proposition_assumptions.clone(),
            proposition_obligations: self.proposition_obligations.clone(),
            proposition_predicate_aliases: self.proposition_predicate_aliases.clone(),
            proposition_predicates: self.proposition_predicates.clone(),
            type_var_interface_bounds: self.type_var_interface_bounds.clone(),
            type_parameter_kinds: self.type_parameter_kinds.clone(),
            variables: HashMap::with_capacity(10),
            workflow_intrinsics: self.workflow_intrinsics.clone(),
            public_workflow_summaries: HashMap::new(),
            fn_contracts: HashMap::new(),
            capability_symbols: HashSet::new(),
            parent: None,
            providers: self.providers.clone(),
            sealed_domain_identities: self.sealed_domain_identities.clone(),
            sealed_domain_aliases: self.sealed_domain_aliases.clone(),
            sealed_domain_summaries: self.sealed_domain_summaries.clone(),
            promoted_data_kind_identities: self.promoted_data_kind_identities.clone(),
            promoted_data_kind_aliases: self.promoted_data_kind_aliases.clone(),
            promoted_data_kind_summaries: self.promoted_data_kind_summaries.clone(),
            promoted_constructor_summaries: self.promoted_constructor_summaries.clone(),
            promoted_constructor_kinds: self.promoted_constructor_kinds.clone(),
            local_type_function_heads: self.local_type_function_heads.clone(),
            local_type_functions: self.local_type_functions.clone(),
            current_module_identity: self.current_module_identity.clone(),
            associated_family_declarations: self.associated_family_declarations.clone(),
            associated_family_name_index: self.associated_family_name_index.clone(),
            associated_family_schemes: self.associated_family_schemes.clone(),
            workflow_effect: None,
            capability_implementation_body: true,
        };
        let effect = match mode {
            CapabilityOperationMode::Observe => ash_core::Effect::Epistemic,
            CapabilityOperationMode::Execute => ash_core::Effect::Operational,
        };
        body_env.set_workflow_effect(effect);
        body_env
    }

    fn type_constructor_expr_kind(&self, expr: &TypeConstructorExpr) -> Option<Kind> {
        match expr {
            TypeConstructorExpr::ProperType(_) => Some(Kind::Type),
            TypeConstructorExpr::ConstructorHead(head) => match head {
                TypeConstructorHeadId::Nominal { visible_name, .. } => self
                    .type_constructor_arity_for_visible_name(visible_name)
                    .map(Kind::n_ary),
                TypeConstructorHeadId::Computation(_) => None,
                _ => None,
            },
            TypeConstructorExpr::PartialApplication(app) => Some(app.result_kind.clone()),
            _ => None,
        }
    }

    fn lower_interface_evidence_args(
        &self,
        interface_name: &str,
        interface: &InterfaceInfo,
        args: &[SurfaceType],
        param_mapping: &HashMap<String, TypeVar>,
    ) -> Result<Vec<InterfaceEvidenceArg>, TypeEnvError> {
        interface
            .type_param_kinds
            .iter()
            .zip(args.iter())
            .map(|(expected_kind, arg)| {
                if expected_kind.is_type() {
                    return surface_type_to_type(arg, param_mapping, self)
                        .map(InterfaceEvidenceArg::Proper);
                }

                let expr = match arg {
                    SurfaceType::Name(name) => {
                        let constructor = name.to_string();
                        let arity = self
                            .type_constructor_arity_for_visible_name(name.as_ref())
                            .ok_or_else(|| {
                                TypeEnvError::InvalidDefinition(
                                    format!(
                                        "unknown constructor evidence argument '{constructor}' for interface '{interface_name}'"
                                    ),
                                    Span::default(),
                                )
                            })?;
                        if arity == 0 {
                            self.lower_surface_type_to_canonical(arg)
                                .map(TypeConstructorExpr::ProperType)
                                .map_err(|err| {
                                    TypeEnvError::InvalidDefinition(
                                        format!(
                                            "invalid constructor evidence argument for interface '{interface_name}': {err}"
                                        ),
                                        Span::default(),
                                    )
                                })?
                        } else {
                            let origin = self
                                .type_identity_for_name(name.as_ref())
                                .cloned()
                                .unwrap_or_else(|| fallback_canonical_type_decl_id(name.as_ref()));
                            TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(
                                origin,
                                constructor,
                            ))
                        }
                    }
                    _ => self.elaborate_partial_type_constructor(arg, false).map_err(|err| {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "invalid constructor evidence argument for interface '{interface_name}': {err}"
                            ),
                            Span::default(),
                        )
                    })?,
                };
                let found_kind = self.type_constructor_expr_kind(&expr).ok_or_else(|| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "unsupported constructor evidence argument '{}' for interface '{interface_name}'",
                            render_type_constructor_expr(&expr)
                        ),
                        Span::default(),
                    )
                })?;
                if &found_kind != expected_kind {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "interface '{interface_name}' evidence argument '{}' has kind {found_kind}, expected {expected_kind}",
                            render_type_constructor_expr(&expr)
                        ),
                        Span::default(),
                    ));
                }

                Ok(InterfaceEvidenceArg::Constructor(Box::new(expr)))
            })
            .collect()
    }

    /// Register a closed-world interface impl.
    pub fn register_impl(&mut self, def: &ImplDef) -> Result<(), TypeEnvError> {
        let interface_name = def.interface.to_string();
        let interface = self
            .interfaces
            .get(&interface_name)
            .cloned()
            .ok_or_else(|| {
                TypeEnvError::MissingInterface(interface_name.clone(), Span::default())
            })?;

        if interface.type_params.len() != def.type_args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' expects {} type parameters, but impl provides {}",
                    interface_name,
                    interface.type_params.len(),
                    def.type_args.len()
                ),
                Span::default(),
            ));
        }
        reject_constructor_kinded_interface_params(&def.type_params, "impl parameter", "TASK-908")?;

        let param_mapping: HashMap<String, TypeVar> = def
            .type_params
            .iter()
            .map(|param| (param.to_string(), TypeVar::fresh()))
            .collect();

        let head_args = self.lower_interface_evidence_args(
            &interface_name,
            &interface,
            &def.type_args,
            &param_mapping,
        )?;

        let lowered_type_args: Vec<Type> = head_args
            .iter()
            .map(interface_evidence_arg_as_legacy_type)
            .collect();

        if def.type_params.is_empty()
            && !lowered_type_args
                .iter()
                .all(is_closed_world_nominal_impl_target)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!("impl for interface '{interface_name}' must target concrete nominal types"),
                Span::default(),
            ));
        }

        let impl_head = Type::Constructor {
            name: QualifiedName::root(interface_name.clone()),
            args: lowered_type_args.clone(),
            kind: Kind::Type,
        };

        // Overlap check
        for scheme in self.impls.iter().filter(|s| s.interface == interface_name) {
            if self.unify_types(&scheme.head, &impl_head).is_ok() {
                if scheme.type_params.is_empty() && def.type_params.is_empty() {
                    if head_args
                        .iter()
                        .any(|arg| matches!(arg, InterfaceEvidenceArg::Constructor(_)))
                    {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "duplicate overlapping impl for evidence {}",
                                render_interface_evidence_key(&interface_name, &head_args)
                            ),
                            Span::default(),
                        ));
                    }
                    return Err(TypeEnvError::DuplicateImpl {
                        interface: interface_name,
                        ty: impl_head.to_string(),
                        span: Span::default(),
                    });
                }
                return Err(TypeEnvError::OverlappingImpls {
                    interface: interface_name,
                    span: Span::default(),
                });
            }
        }

        let where_bounds: Vec<WhereBound> = def
            .where_bounds
            .iter()
            .map(|wb| {
                let type_var = param_mapping
                    .get(wb.param.as_ref())
                    .copied()
                    .ok_or_else(|| {
                        TypeEnvError::InvalidDefinition(
                            format!("unknown type parameter '{}' in where bound", wb.param),
                            Span::default(),
                        )
                    })?;
                let bound_interface = wb.bound.to_string();
                if !self.has_interface(&bound_interface) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!("unknown interface '{}' in where bound", bound_interface),
                        Span::default(),
                    ));
                }
                Ok(WhereBound {
                    type_var,
                    interface: bound_interface,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut impl_binding_env = self.clone();
        for bound in &where_bounds {
            impl_binding_env
                .type_var_interface_bounds
                .entry(bound.type_var)
                .or_default()
                .insert(bound.interface.clone());
        }

        let family_declarations = self
            .associated_family_declarations_for_interface(&interface_name)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let family_names = family_declarations
            .iter()
            .map(|decl| decl.head.member.name.to_string())
            .collect::<HashSet<_>>();
        let ordinary_associated_names = interface
            .associated_types
            .iter()
            .filter(|name| !family_names.contains(name.as_str()))
            .cloned()
            .collect::<HashSet<_>>();
        let mut family_var_constraints = HashMap::new();
        for param in &def.type_params {
            if let Some(domain) =
                self.optional_param_domain_constraint(param.domain.as_ref(), param.span)?
            {
                family_var_constraints.insert(
                    param.name.to_string(),
                    AssociatedFamilyResultConstraint::Domain(domain),
                );
            }
        }
        for family in &family_declarations {
            for (arg, param) in def.type_args.iter().zip(family.interface_params.iter()) {
                if let (SurfaceType::Name(name), Some(domain)) =
                    (arg, param.domain_constraint.as_ref())
                {
                    family_var_constraints
                        .entry(name.to_string())
                        .or_insert_with(|| {
                            AssociatedFamilyResultConstraint::Domain(domain.clone())
                        });
                }
            }
        }
        let impl_family_module = if family_declarations.is_empty() {
            None
        } else {
            Some(self.current_module_identity.clone().ok_or_else(|| {
                TypeEnvError::AssociatedFamilyModuleOwnerViolation {
                    family: family_declarations
                        .first()
                        .map(|family| family.head.member.name.to_string())
                        .unwrap_or_else(|| "<unknown>".to_string()),
                    reason: "missing current module identity while registering sealed family impl"
                        .to_string(),
                    span: def.span,
                }
            })?)
        };

        for binding in &def.associated_type_bindings {
            let binding_name = binding.name.to_string();
            if !interface.associated_types.contains(&binding_name) {
                return Err(
                    if family_declarations.is_empty() || !ordinary_associated_names.is_empty() {
                        TypeEnvError::InvalidDefinition(
                            format!(
                                "extraneous associated type binding '{binding_name}' in impl for interface '{interface_name}'"
                            ),
                            binding.span,
                        )
                    } else {
                        TypeEnvError::ExtraAssociatedFamilyBinding {
                            interface: interface_name.clone(),
                            family: binding_name,
                            span: binding.span,
                        }
                    },
                );
            }
        }

        let mut staged_family_schemes = Vec::new();
        for family in &family_declarations {
            let family_name = family.head.member.name.to_string();
            let Some(binding) = def
                .associated_type_bindings
                .iter()
                .find(|binding| binding.name.as_ref() == family_name)
            else {
                return Err(TypeEnvError::MissingAssociatedFamilyBinding {
                    interface: interface_name.clone(),
                    family: family_name,
                    span: def.span,
                });
            };
            let result = self
                .lower_associated_family_result_expr(
                    &binding.ty,
                    &family.result_domain,
                    &family_var_constraints,
                    binding.span,
                )
                .map_err(|err| match err {
                    TypeEnvError::WrongAssociatedFamilyResultDomain { reason, span, .. } => {
                        TypeEnvError::WrongAssociatedFamilyResultDomain {
                            family: family_name.clone(),
                            reason,
                            span,
                        }
                    }
                    other => other,
                })?;
            let params = family
                .interface_params
                .iter()
                .map(|param| AssociatedFamilySchemeParam {
                    name: param.name.clone(),
                    ty: CanonicalTypeExpr::Var(param.name.clone()),
                    kind: Kind::Type,
                    domain_constraint: param.domain_constraint.clone(),
                    source_anchor: span_anchor(
                        binding.span,
                        format!("associated family param {}", param.name),
                    ),
                })
                .collect::<Vec<_>>();
            let interface_arg_patterns = def
                .type_args
                .iter()
                .zip(family.interface_params.iter())
                .map(|(arg, param)| {
                    self.lower_associated_family_pattern(
                        arg,
                        param.domain_constraint.as_ref(),
                        &family_var_constraints,
                        binding.span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let scheme = AssociatedFamilyScheme {
                head: family.head.clone(),
                params,
                result_domain: canonical_expr_for_associated_family_constraint(
                    &family.result_domain,
                ),
                result_kind: Kind::Type,
                equations: vec![AssociatedFamilyEquation {
                    head: family.head.clone(),
                    ordinal: 0,
                    interface_arg_patterns,
                    result,
                    decreases: None,
                    source_anchor: span_anchor(binding.span, "associated family equation"),
                    case_head_anchor: span_anchor(binding.span, "associated family case head"),
                }],
                source_anchor: span_anchor(binding.span, "associated family scheme"),
            };
            let defining_module = impl_family_module
                .clone()
                .expect("family declarations require module context");
            staged_family_schemes.push((scheme, defining_module));
        }

        let associated_type_bindings: HashMap<String, Type> = def
            .associated_type_bindings
            .iter()
            .filter(|binding| !family_names.contains(binding.name.as_ref()))
            .map(|binding| {
                let ty = surface_type_to_type(&binding.ty, &param_mapping, &impl_binding_env)?;
                if let Some(name) = unresolved_associated_projection_name(&ty) {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "unresolved associated type '{name}' in impl associated type binding '{}' for interface '{interface_name}'",
                            binding.name
                        ),
                        Span::default(),
                    ));
                }
                Ok((binding.name.to_string(), ty))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        for assoc_name in &interface.associated_types {
            if family_names.contains(assoc_name) {
                continue;
            }
            if !associated_type_bindings.contains_key(assoc_name) {
                return Err(TypeEnvError::MissingAssociatedType {
                    interface: interface_name.clone(),
                    name: assoc_name.clone(),
                    span: Span::default(),
                });
            }
        }
        for bound_name in associated_type_bindings.keys() {
            if !interface.associated_types.contains(bound_name) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "extraneous associated type binding '{bound_name}' in impl for interface '{interface_name}'"
                    ),
                    Span::default(),
                ));
            }
        }

        let temp_scheme = ImplScheme {
            interface: interface.name.clone(),
            type_params: param_mapping.values().copied().collect(),
            head: impl_head.clone(),
            head_args: head_args.clone(),
            where_bounds: where_bounds.clone(),
            associated_type_bindings: associated_type_bindings.clone(),
            methods: vec![],
        };
        let constructor_arg_mapping = interface
            .type_params
            .iter()
            .cloned()
            .zip(head_args.iter().cloned())
            .collect::<HashMap<_, _>>();

        let mut method_names = HashSet::new();
        let mut method_infos = Vec::new();
        for method in &def.methods {
            let method_name = method.name.to_string();
            let Some(method_info) = interface.methods.get(&method_name) else {
                return Err(TypeEnvError::MissingInterfaceMethod {
                    interface: interface.name.clone(),
                    method: method_name,
                    span: Span::default(),
                });
            };

            if !method_names.insert(method_name.clone()) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "duplicate method '{method_name}' in impl for interface '{}'",
                        interface.name
                    ),
                    Span::default(),
                ));
            }

            if method_info.params.len() != method.params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "impl method '{}::{}' signature expects {} parameters, found {}",
                        interface.name,
                        method_name,
                        method_info.params.len(),
                        method.params.len()
                    ),
                    Span::default(),
                ));
            }

            let mut subst = Substitution::new();
            for (tv, concrete_arg) in method_info.type_params.iter().zip(lowered_type_args.iter()) {
                subst.insert(*tv, concrete_arg.clone());
            }

            let mut method_env = self.clone();
            for (param_name, param_type) in method.params.iter().zip(method_info.params.iter()) {
                let param_ty = substitute_constructor_variable_apps(
                    &subst.apply(param_type),
                    &constructor_arg_mapping,
                    &param_mapping,
                );
                method_env.bind_variable(param_name.as_ref(), param_ty);
            }

            let expected_return_ty = substitute_constructor_variable_apps(
                &subst.apply(&method_info.return_type),
                &constructor_arg_mapping,
                &param_mapping,
            );
            let expected_return_ty =
                self.normalize_associated_types(&expected_return_ty, &temp_scheme, &subst)?;

            let body_result = crate::check_expr::check_expr(&method_env, &method.body);
            if !body_result.is_ok() {
                let reason = body_result
                    .errors
                    .into_iter()
                    .next()
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| {
                        format!(
                            "failed to typecheck body for impl method '{}::{}'",
                            interface.name, method_name
                        )
                    });

                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "invalid impl method body for '{}::{}': {}",
                        interface.name, method_name, reason
                    ),
                    Span::default(),
                ));
            }

            let actual_return_ty = body_result.substitution.apply(&body_result.ty);
            self.unify_types(&expected_return_ty, &actual_return_ty)
                .map_err(|_| {
                    TypeEnvError::InvalidDefinition(
                        format!(
                            "impl method '{}::{}' must return {}, found {}",
                            interface.name, method_name, expected_return_ty, actual_return_ty
                        ),
                        Span::default(),
                    )
                })?;

            let core_body = ash_parser::lower_expr(&method.body).map_err(|e| {
                TypeEnvError::InvalidDefinition(format!("lowering error: {e}"), Span::default())
            })?;

            method_infos.push(ImplMethodInfo {
                name: method_name,
                param_names: method
                    .params
                    .iter()
                    .map(|param| param.to_string())
                    .collect(),
                type_params: method_info.type_params.clone(),
                params: method_info
                    .params
                    .iter()
                    .map(|t| {
                        substitute_constructor_variable_apps(
                            &subst.apply(t),
                            &constructor_arg_mapping,
                            &param_mapping,
                        )
                    })
                    .collect(),
                return_type: expected_return_ty,
                body: core_body,
            });
        }

        for required_method in interface.methods.keys() {
            if !method_names.contains(required_method) {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "impl for interface '{}' is missing method '{required_method}'",
                        interface.name
                    ),
                    Span::default(),
                ));
            }
        }

        let previous_family_schemes = self.associated_family_schemes.clone();
        for (scheme, defining_module) in staged_family_schemes {
            if let Err(error) =
                self.register_associated_family_scheme_with_totality(scheme, defining_module, false)
            {
                self.associated_family_schemes = previous_family_schemes;
                return Err(error);
            }
        }

        for (bound, source_bound) in where_bounds.iter().zip(def.where_bounds.iter()) {
            self.record_type_var_interface_bound_assumption(
                bound.type_var,
                &bound.interface,
                proposition_source_anchor(
                    SourceOrigin::Synthetic {
                        reason: "impl where-bound proposition assumption".to_string(),
                    },
                    source_bound.span,
                    format!(
                        "impl where-bound type variable {} satisfies interface {}",
                        bound.type_var.0, bound.interface
                    ),
                ),
                PropositionCheckingSite::new(
                    0x8752_0000u64 + u64::from(bound.type_var.0),
                    PropositionCheckingSiteKind::ImplWhereBound,
                    Some(format!(
                        "impl where type_var_{}: {}",
                        bound.type_var.0, bound.interface
                    )),
                ),
            );
        }

        if def.type_params.is_empty() {
            self.record_concrete_impl_interface_assumption(
                &interface.name,
                &lowered_type_args,
                proposition_source_anchor(
                    SourceOrigin::Synthetic {
                        reason: "concrete impl proposition assumption".to_string(),
                    },
                    def.span,
                    format!("concrete impl evidence for interface {}", interface.name),
                ),
            );
        }

        self.impls.push(ImplScheme {
            interface: interface.name,
            type_params: param_mapping.values().copied().collect(),
            head: impl_head,
            head_args,
            where_bounds,
            associated_type_bindings,
            methods: method_infos,
        });

        Ok(())
    }

    /// Look up a constructor by name
    ///
    /// Returns `Some((type_name, variant_index))` if found, `None` otherwise
    pub fn lookup_constructor(&self, name: &str) -> Option<(TypeName, VariantIndex)> {
        self.constructors.get(name).cloned()
    }

    /// Look up a type definition by name (AST version)
    pub fn lookup_type(&self, name: &str) -> Option<&TypeDef> {
        self.ast_types.get(name)
    }

    /// Iterate over AST type definitions visible in this environment.
    pub fn ast_type_defs(&self) -> impl Iterator<Item = (&TypeName, &TypeDef)> {
        self.ast_types.iter()
    }

    /// Look up internal type info by name
    pub fn lookup_type_info(&self, name: &str) -> Option<&TypeInfo> {
        self.type_info.get(name)
    }

    #[cfg(test)]
    pub(crate) fn remove_type_info_for_test(&mut self, name: &str) {
        self.type_info.remove(name);
    }

    /// Get the variant definition for a constructor
    pub fn get_variant(
        &self,
        constructor_name: &str,
    ) -> Option<(&TypeInfo, VariantIndex, &VariantInfo)> {
        let (type_name, variant_index) = self.lookup_constructor(constructor_name)?;
        let type_info = self.type_info.get(&type_name)?;

        if let TypeInfo::Enum { variants, .. } = type_info {
            variants
                .get(variant_index)
                .map(|v| (type_info, variant_index, v))
        } else {
            None
        }
    }

    /// Add builtin types (Option, Result, and List)
    pub fn add_builtin_types(&mut self) {
        self.add_option_type();
        self.add_result_type();
        self.add_list_type();
        self.add_record_type();
        self.add_act_type();
        self.add_proc_type();
        self.add_workflow_type();
        self.add_process_handle_type();
        self.add_act_builtin_values();
        self.add_proc_builtin_values();
        self.add_workflow_builtin_values();
        self.add_result_builtin_values();
        self.add_builtin_capability_symbols();
    }

    fn add_builtin_capability_symbols(&mut self) {
        for capability in ["Args", "Dir", "Fs", "Meta", "Stdio"] {
            self.register_capability_symbol(capability);
        }
    }

    /// Add the Option<T> type
    fn add_option_type(&mut self) {
        // Option<T> = Some { value: T } | None
        let option_type = TypeDef {
            name: "Option".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Some".to_string(),
                    fields: vec![("value".to_string(), TypeExpr::Named("T".to_string()))],
                    payload: VariantPayload::Record(vec![(
                        "value".to_string(),
                        TypeExpr::Named("T".to_string()),
                    )]),
                },
                VariantDef {
                    name: "None".to_string(),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                },
            ]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: false,
        };

        self.register_type_identity(&option_type)
            .expect("Failed to register Option type");
        self.expose_type_representation("Option")
            .expect("Failed to expose Option constructors");
    }

    /// Add the Result<T, E> type
    fn add_result_type(&mut self) {
        // Result<T, E> = Ok { value: T } | Err { error: E }
        let result_type = TypeDef {
            name: "Result".to_string(),
            params: vec!["T".to_string(), "E".to_string()],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Ok".to_string(),
                    fields: vec![("value".to_string(), TypeExpr::Named("T".to_string()))],
                    payload: VariantPayload::Record(vec![(
                        "value".to_string(),
                        TypeExpr::Named("T".to_string()),
                    )]),
                },
                VariantDef {
                    name: "Err".to_string(),
                    fields: vec![("error".to_string(), TypeExpr::Named("E".to_string()))],
                    payload: VariantPayload::Record(vec![(
                        "error".to_string(),
                        TypeExpr::Named("E".to_string()),
                    )]),
                },
            ]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: false,
        };

        self.register_type(&result_type)
            .expect("Failed to register Result type");
    }

    /// Add the List<T> type
    fn add_list_type(&mut self) {
        // List<T> is a generic builtin type represented as a struct with a type parameter
        let list_type = TypeDef {
            name: "List".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]), // opaque builtin; no fields needed for type checking
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&list_type)
            .expect("Failed to register List type");
    }

    /// Add the Record type
    fn add_record_type(&mut self) {
        let record_type = TypeDef {
            name: "Record".to_string(),
            params: vec![],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type_identity(&record_type)
            .expect("Failed to register Record type");
        self.expose_type_representation("Record")
            .expect("Failed to expose Record representation");
    }

    /// Add the Act<T> type
    fn add_act_type(&mut self) {
        let act_type = TypeDef {
            name: "Act".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&act_type)
            .expect("Failed to register Act type");
    }

    /// Add the Proc<T> type.
    fn add_proc_type(&mut self) {
        let proc_type = TypeDef {
            name: "Proc".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&proc_type)
            .expect("Failed to register Proc type");
    }

    /// Add the public Workflow<T> type.
    fn add_workflow_type(&mut self) {
        let workflow_type = TypeDef {
            name: "Workflow".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&workflow_type)
            .expect("Failed to register Workflow type");
    }

    /// Add the opaque P<T> process handle type.
    fn add_process_handle_type(&mut self) {
        let process_handle_type = TypeDef {
            name: "P".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type(&process_handle_type)
            .expect("Failed to register P type");
    }

    /// Add the qualified act module builtin value signatures.
    fn add_act_builtin_values(&mut self) {
        let a = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let b = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let act_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let act_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };

        self.bind_variable(
            "act::unit",
            crate::types::Type::Fn(vec![a.clone()], Box::new(act_a.clone())),
        );
        self.bind_variable(
            "act::bind",
            crate::types::Type::Fn(
                vec![
                    act_a.clone(),
                    crate::types::Type::Fn(vec![a], Box::new(act_b.clone())),
                ],
                Box::new(act_b.clone()),
            ),
        );
        self.bind_variable(
            "act::then",
            crate::types::Type::Fn(vec![act_a.clone(), act_b.clone()], Box::new(act_b)),
        );
        self.bind_variable(
            "act::guard",
            crate::types::Type::Fn(
                vec![crate::types::Type::String, act_a.clone()],
                Box::new(act_a),
            ),
        );
        self.bind_variable(
            "act::policy_check",
            crate::types::Type::Fn(
                vec![crate::types::Type::String],
                Box::new(crate::types::Type::Bool),
            ),
        );
    }

    /// Add the qualified proc module builtin value signatures.
    fn add_proc_builtin_values(&mut self) {
        let a = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let b = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let act_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let proc_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let proc_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };
        let handle_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("P"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let handle_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("P"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };
        let proc_null = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![crate::types::Type::Null],
            kind: crate::Kind::Type,
        };
        let proc_pair_handles = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![crate::types::Type::Record(vec![
                ("_0".into(), handle_a.clone()),
                ("_1".into(), handle_b.clone()),
            ])],
            kind: crate::Kind::Type,
        };
        let proc_pair_ab = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![crate::types::Type::Record(vec![
                ("_0".into(), a.clone()),
                ("_1".into(), b.clone()),
            ])],
            kind: crate::Kind::Type,
        };
        let list_a = crate::types::Type::List(Box::new(a.clone()));
        let list_handle_a = crate::types::Type::List(Box::new(handle_a.clone()));
        let proc_list_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![list_a.clone()],
            kind: crate::Kind::Type,
        };
        let list_handle_b = crate::types::Type::List(Box::new(handle_b.clone()));
        let proc_list_handle_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![list_handle_b],
            kind: crate::Kind::Type,
        };

        self.bind_variable(
            "proc::unit",
            crate::types::Type::Fn(vec![a.clone()], Box::new(proc_a.clone())),
        );
        self.bind_variable(
            "proc::from_act",
            crate::types::Type::Fn(vec![act_a], Box::new(proc_a.clone())),
        );
        self.bind_variable(
            "proc::bind",
            crate::types::Type::Fn(
                vec![
                    proc_a.clone(),
                    crate::types::Type::Fn(vec![a.clone()], Box::new(proc_b.clone())),
                ],
                Box::new(proc_b.clone()),
            ),
        );
        self.bind_variable(
            "proc::then",
            crate::types::Type::Fn(
                vec![proc_a.clone(), proc_b.clone()],
                Box::new(proc_b.clone()),
            ),
        );
        self.bind_variable(
            "proc::await",
            crate::types::Type::Fn(vec![handle_a.clone()], Box::new(proc_a.clone())),
        );
        self.bind_variable(
            "proc::yield",
            crate::types::Type::Fn(vec![], Box::new(proc_null)),
        );
        self.bind_variable(
            "proc::par",
            crate::types::Type::Fn(
                vec![proc_a.clone(), proc_b.clone()],
                Box::new(proc_pair_handles),
            ),
        );
        self.bind_variable(
            "proc::scatter",
            crate::types::Type::Fn(
                vec![list_a, crate::types::Type::Fn(vec![a], Box::new(proc_b))],
                Box::new(proc_list_handle_b),
            ),
        );
        self.bind_variable(
            "proc::join",
            crate::types::Type::Fn(vec![handle_a, handle_b], Box::new(proc_pair_ab)),
        );
        self.bind_variable(
            "proc::gather",
            crate::types::Type::Fn(vec![list_handle_a], Box::new(proc_list_a)),
        );
    }

    /// Add the qualified workflow module builtin value signatures.
    fn add_workflow_builtin_values(&mut self) {
        let a = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let b = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let workflow_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Workflow"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let workflow_b = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Workflow"),
            args: vec![b.clone()],
            kind: crate::Kind::Type,
        };
        let proc_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Proc"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        let act_a = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![a.clone()],
            kind: crate::Kind::Type,
        };
        self.bind_variable(
            "workflow::unit",
            crate::types::Type::Fn(vec![a.clone()], Box::new(workflow_a.clone())),
        );
        self.bind_variable(
            "workflow::bind",
            crate::types::Type::Fn(
                vec![
                    workflow_a.clone(),
                    crate::types::Type::Fn(vec![a], Box::new(workflow_b.clone())),
                ],
                Box::new(workflow_b.clone()),
            ),
        );
        self.bind_variable(
            "workflow::then",
            crate::types::Type::Fn(
                vec![workflow_a.clone(), workflow_b.clone()],
                Box::new(workflow_b),
            ),
        );
        self.bind_variable(
            "workflow::from_proc",
            crate::types::Type::Fn(vec![proc_a], Box::new(workflow_a.clone())),
        );
        self.bind_variable(
            "workflow::from_act",
            crate::types::Type::Fn(vec![act_a], Box::new(workflow_a)),
        );
        let workflow_unit = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Workflow"),
            args: vec![crate::types::Type::Null],
            kind: crate::Kind::Type,
        };
        self.workflow_intrinsics.insert(
            "workflow::requires".to_string(),
            WorkflowIntrinsic::requires(workflow_unit.clone()),
        );
        self.workflow_intrinsics.insert(
            "workflow::ensures".to_string(),
            WorkflowIntrinsic::ensures(workflow_unit),
        );
    }

    /// Add the qualified result module helper signatures used by the public tower manifest.
    fn add_result_builtin_values(&mut self) {
        let t = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let e = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let u = crate::types::Type::Var(crate::types::TypeVar::fresh());
        let result_t_e = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Result"),
            args: vec![t.clone(), e.clone()],
            kind: crate::Kind::Type,
        };
        let result_u_e = crate::types::Type::Constructor {
            name: crate::QualifiedName::root("Result"),
            args: vec![u.clone(), e],
            kind: crate::Kind::Type,
        };

        self.bind_variable(
            "result::and_then",
            crate::types::Type::Fn(
                vec![
                    result_t_e,
                    crate::types::Type::Fn(vec![t], Box::new(result_u_e.clone())),
                ],
                Box::new(result_u_e),
            ),
        );
    }

    /// Check if a type is registered
    pub fn has_type(&self, name: &str) -> bool {
        self.ast_types.contains_key(name)
    }

    /// Check if a type is registered with a full (non-placeholder) definition.
    /// Returns `false` for unregistered names and for placeholder entries.
    pub fn has_full_type(&self, name: &str) -> bool {
        match self.ast_types.get(name) {
            None => false,
            Some(_) => matches!(
                self.type_declaration_states.get(name),
                Some(TypeDeclarationState::Full)
            ),
        }
    }

    /// Check if a constructor is registered
    pub fn has_constructor(&self, name: &str) -> bool {
        self.constructors.contains_key(name)
    }

    /// Bind a variable to a type in this environment
    pub fn bind_variable(&mut self, name: &str, ty: crate::types::Type) {
        self.variables.insert(name.to_string(), ty);
    }

    /// Look up a compiler-known workflow intrinsic.
    pub fn lookup_workflow_intrinsic(&self, name: &str) -> Option<WorkflowIntrinsic> {
        self.workflow_intrinsics.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.lookup_workflow_intrinsic(name))
        })
    }

    /// Bind a public Workflow summary imported from module metadata.
    pub fn bind_public_workflow_summary(
        &mut self,
        name: &str,
        summary: ash_core::workflow_carrier::PublicWorkflowSummary,
    ) {
        self.public_workflow_summaries
            .insert(name.to_string(), summary);
    }

    /// Look up a public Workflow summary by local or qualified binding name.
    pub fn lookup_public_workflow_summary(
        &self,
        name: &str,
    ) -> Option<ash_core::workflow_carrier::PublicWorkflowSummary> {
        self.public_workflow_summaries
            .get(name)
            .cloned()
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.lookup_public_workflow_summary(name))
            })
    }

    /// Return the names of all registered unit constructors.
    pub fn unit_constructor_names(&self) -> impl Iterator<Item = String> + '_ {
        self.constructors.iter().filter_map(|(name, _)| {
            self.get_variant(name).and_then(|(_, _, variant)| {
                (variant.payload_shape == VariantPayloadShape::Unit).then(|| name.clone())
            })
        })
    }

    /// Return the names of all bound variables (used for name resolution of imported callables).
    pub fn variable_names(&self) -> impl Iterator<Item = String> + '_ {
        self.variables.keys().cloned()
    }

    /// Store the lowered contract boundary for a pure function.
    pub fn bind_fn_contract(&mut self, name: &str, contract: StoredFnContract) {
        self.fn_contracts.insert(name.to_string(), contract);
    }

    /// Record that a workflow type variable satisfies an interface bound.
    pub fn bind_type_var_interface_bound(&mut self, var: TypeVar, interface: &str) {
        let inserted = self
            .type_var_interface_bounds
            .entry(var)
            .or_default()
            .insert(interface.to_string());
        if inserted {
            self.record_type_var_interface_bound_assumption(
                var,
                interface,
                synthetic_proposition_source_anchor(format!(
                    "type variable {} satisfies interface {interface}",
                    var.0
                )),
                PropositionCheckingSite::new(
                    0x8751_0000u64 + u64::from(var.0),
                    PropositionCheckingSiteKind::TypeVariableInterfaceBound,
                    Some(format!("type_var_{}: {interface}", var.0)),
                ),
            );
        }
    }

    /// Register the kind of a source-visible type parameter in this TypeEnv.
    pub fn register_type_parameter_kind(
        &mut self,
        name: impl Into<String>,
        kind: Kind,
    ) -> Result<(), TypeEnvError> {
        let name = name.into();
        if let Some(existing) = self.type_parameter_kinds.get(&name)
            && existing != &kind
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "type parameter '{name}' already has kind {existing}, cannot also register kind {kind}"
                ),
                Span::default(),
            ));
        }
        self.type_parameter_kinds.insert(name, kind);
        Ok(())
    }

    /// Look up the kind of a source-visible type parameter.
    #[must_use]
    pub fn type_parameter_kind(&self, name: &str) -> Option<&Kind> {
        self.type_parameter_kinds.get(name).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.type_parameter_kind(name))
        })
    }

    /// Look up a variable's type in this environment
    ///
    /// Searches current scope first, then parent scopes
    pub fn lookup_variable(&self, name: &str) -> Option<crate::types::Type> {
        if let Some(ty) = self.variables.get(name) {
            return Some(ty.clone());
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup_variable(name);
        }
        None
    }

    /// Look up a lowered pure-function contract boundary.
    pub fn lookup_fn_contract(&self, name: &str) -> Option<StoredFnContract> {
        if let Some(contract) = self.fn_contracts.get(name) {
            return Some(contract.clone());
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup_fn_contract(name);
        }
        None
    }

    /// Snapshot all lowered pure-function contract boundaries in scope.
    pub fn function_contracts(&self) -> HashMap<String, StoredFnContract> {
        let mut contracts = self
            .parent
            .as_ref()
            .map_or_else(HashMap::new, |parent| parent.function_contracts());
        contracts.extend(self.fn_contracts.clone());
        contracts
    }

    /// Resolve a function call target.
    ///
    /// Qualified calls must resolve to the exact qualified binding; they must not silently
    /// fall back to an unrelated unqualified function with the same base name.
    pub fn lookup_call_target(
        &self,
        module: Option<&str>,
        name: &str,
    ) -> Option<crate::types::Type> {
        match module {
            Some(module) => self.lookup_variable(&format!("{module}::{name}")),
            None => self.lookup_variable(name),
        }
    }

    pub fn register_capability_symbol(&mut self, name: impl Into<String>) {
        self.capability_symbols.insert(name.into());
    }

    pub fn has_capability_symbol(&self, name: &str) -> bool {
        self.capability_symbols.contains(name)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.has_capability_symbol(name))
    }

    /// Create a new child environment with this as parent
    ///
    /// Used for block scoping - variables bound in the child
    /// are not visible in the parent. The workflow effect context is inherited
    /// so that closures nested inside a workflow body still get `Type::Fun`.
    #[must_use]
    pub fn extend(&self) -> Self {
        Self {
            ast_types: self.ast_types.clone(),
            type_info: self.type_info.clone(),
            constructors: self.constructors.clone(),
            transparent_aliases: self.transparent_aliases.clone(),
            type_declaration_states: self.type_declaration_states.clone(),
            type_alias_identities: self.type_alias_identities.clone(),
            canonical_type_names: self.canonical_type_names.clone(),
            interface_identity_aliases: self.interface_identity_aliases.clone(),
            interface_identity_alias_is_imported: self.interface_identity_alias_is_imported.clone(),
            canonical_interface_names: self.canonical_interface_names.clone(),
            local_interface_arities: self.local_interface_arities.clone(),
            known_interface_identities: self.known_interface_identities.clone(),
            associated_member_identity_aliases: self.associated_member_identity_aliases.clone(),
            associated_member_identity_alias_is_imported: self
                .associated_member_identity_alias_is_imported
                .clone(),
            known_associated_member_identities: self.known_associated_member_identities.clone(),
            interfaces: self.interfaces.clone(),
            capability_interfaces: self.capability_interfaces.clone(),
            resource_types: self.resource_types.clone(),
            capability_implementations: self.capability_implementations.clone(),
            capability_bindings: self.capability_bindings.clone(),
            impls: self.impls.clone(),
            proposition_assumptions: self.proposition_assumptions.clone(),
            proposition_obligations: self.proposition_obligations.clone(),
            proposition_predicate_aliases: self.proposition_predicate_aliases.clone(),
            proposition_predicates: self.proposition_predicates.clone(),
            type_var_interface_bounds: self.type_var_interface_bounds.clone(),
            type_parameter_kinds: self.type_parameter_kinds.clone(),
            variables: HashMap::with_capacity(10),
            workflow_intrinsics: self.workflow_intrinsics.clone(),
            public_workflow_summaries: self.public_workflow_summaries.clone(),
            fn_contracts: self.fn_contracts.clone(),
            capability_symbols: self.capability_symbols.clone(),
            parent: Some(Box::new(self.clone())),
            providers: self.providers.clone(),
            sealed_domain_identities: self.sealed_domain_identities.clone(),
            sealed_domain_aliases: self.sealed_domain_aliases.clone(),
            sealed_domain_summaries: self.sealed_domain_summaries.clone(),
            promoted_data_kind_identities: self.promoted_data_kind_identities.clone(),
            promoted_data_kind_aliases: self.promoted_data_kind_aliases.clone(),
            promoted_data_kind_summaries: self.promoted_data_kind_summaries.clone(),
            promoted_constructor_summaries: self.promoted_constructor_summaries.clone(),
            promoted_constructor_kinds: self.promoted_constructor_kinds.clone(),
            local_type_function_heads: self.local_type_function_heads.clone(),
            local_type_functions: self.local_type_functions.clone(),
            current_module_identity: self.current_module_identity.clone(),
            associated_family_declarations: self.associated_family_declarations.clone(),
            associated_family_name_index: self.associated_family_name_index.clone(),
            associated_family_schemes: self.associated_family_schemes.clone(),
            workflow_effect: self.workflow_effect,
            capability_implementation_body: self.capability_implementation_body,
        }
    }

    /// Check if an interface is registered.
    pub fn has_interface(&self, name: &str) -> bool {
        self.interfaces.contains_key(name)
    }

    /// Look up a registered interface.
    pub fn lookup_interface(&self, name: &str) -> Option<&InterfaceInfo> {
        self.interfaces.get(name)
    }

    /// Resolve explicit interface evidence by matching the registered impl head spine.
    pub fn resolve_interface_evidence(
        &self,
        interface: &str,
        args: &[SurfaceType],
    ) -> Result<&ImplScheme, TypeEnvError> {
        let interface_info = self.interfaces.get(interface).ok_or_else(|| {
            TypeEnvError::MissingInterface(interface.to_string(), Span::default())
        })?;
        if interface_info.type_params.len() != args.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' expects {} type parameters, but evidence lookup provides {}",
                    interface,
                    interface_info.type_params.len(),
                    args.len()
                ),
                Span::default(),
            ));
        }

        let evidence_args =
            self.lower_interface_evidence_args(interface, interface_info, args, &HashMap::new())?;
        let mut matches = self.impls.iter().filter(|scheme| {
            scheme.interface == interface
                && interface_evidence_args_match(
                    &scheme.head_args,
                    &evidence_args,
                    !scheme.type_params.is_empty(),
                )
        });
        let first = matches.next().ok_or_else(|| TypeEnvError::MissingImpl {
            interface: interface.to_string(),
            ty: render_interface_evidence_key(interface, &evidence_args),
            span: Span::default(),
        })?;
        if matches.next().is_some() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "ambiguous evidence for {}",
                    render_interface_evidence_key(interface, &evidence_args)
                ),
                Span::default(),
            ));
        }
        Ok(first)
    }

    /// Check if a capability interface is registered.
    pub fn has_capability_interface(&self, name: &str) -> bool {
        self.capability_interfaces.contains_key(name)
    }

    /// Look up a registered capability interface.
    pub fn lookup_capability_interface(&self, name: &str) -> Option<&CapabilityInterfaceInfo> {
        self.capability_interfaces.get(name)
    }

    /// Look up a registered capability operation signature.
    pub fn lookup_capability_operation(
        &self,
        interface: &str,
        operation: &str,
    ) -> Option<&CapabilityOperationInfo> {
        self.capability_interfaces
            .get(interface)
            .and_then(|info| info.operations.get(operation))
    }

    /// Check if a capability implementation is registered.
    pub fn has_capability_implementation(&self, name: &str) -> bool {
        self.capability_implementations.contains_key(name)
    }

    /// Look up a registered capability implementation.
    pub fn lookup_capability_implementation(
        &self,
        name: &str,
    ) -> Option<&CapabilityImplementationInfo> {
        self.capability_implementations.get(name)
    }

    /// Register a workflow-admitted capability binding for operation-call resolution.
    pub fn register_capability_binding(&mut self, binding: CapabilityBindingInfo) {
        self.capability_bindings
            .insert(binding.name.clone(), binding);
    }

    /// Look up a workflow-admitted capability binding by local binding name.
    pub fn lookup_capability_binding(&self, name: &str) -> Option<&CapabilityBindingInfo> {
        self.capability_bindings
            .get(name)
            .or_else(|| self.parent.as_ref()?.lookup_capability_binding(name))
    }

    /// Check whether a workflow-admitted capability binding exists.
    pub fn has_capability_binding(&self, name: &str) -> bool {
        self.lookup_capability_binding(name).is_some()
    }

    /// Return local workflow-admitted capability binding names.
    pub fn capability_binding_names(&self) -> Vec<String> {
        self.capability_bindings.keys().cloned().collect()
    }

    /// Return all registered impl schemes.
    pub fn impl_schemes(&self) -> &[ImplScheme] {
        &self.impls
    }

    fn type_var_has_interface_bound(&self, var: TypeVar, interface: &str) -> bool {
        self.type_var_interface_bounds
            .get(&var)
            .is_some_and(|bounds| bounds.contains(interface))
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.type_var_has_interface_bound(var, interface))
    }

    pub fn normalize_associated_types(
        &self,
        ty: &Type,
        scheme: &ImplScheme,
        subst: &Substitution,
    ) -> Result<Type, TypeEnvError> {
        match ty {
            Type::Associated {
                interface,
                base: _,
                name,
            } => {
                if scheme.interface != *interface {
                    return Err(TypeEnvError::MismatchedProjectionInterface {
                        expected: scheme.interface.clone(),
                        found: interface.clone(),
                        span: Span::default(),
                    });
                }
                let binding = scheme.associated_type_bindings.get(name).ok_or_else(|| {
                    TypeEnvError::MissingAssociatedType {
                        interface: interface.clone(),
                        name: name.clone(),
                        span: Span::default(),
                    }
                })?;
                let normalized = subst.apply(binding);
                self.normalize_associated_types(&normalized, scheme, subst)
            }
            Type::Constructor { name, args, kind } => {
                let norm_args = args
                    .iter()
                    .map(|a| self.normalize_associated_types(a, scheme, subst))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Constructor {
                    name: name.clone(),
                    args: norm_args,
                    kind: kind.clone(),
                })
            }
            Type::Fun(params, ret, effect) => {
                let norm_params = params
                    .iter()
                    .map(|p| self.normalize_associated_types(p, scheme, subst))
                    .collect::<Result<Vec<_>, _>>()?;
                let norm_ret = self.normalize_associated_types(ret, scheme, subst)?;
                Ok(Type::Fun(norm_params, Box::new(norm_ret), *effect))
            }
            Type::Fn(params, ret) => {
                let norm_params = params
                    .iter()
                    .map(|p| self.normalize_associated_types(p, scheme, subst))
                    .collect::<Result<Vec<_>, _>>()?;
                let norm_ret = self.normalize_associated_types(ret, scheme, subst)?;
                Ok(Type::Fn(norm_params, Box::new(norm_ret)))
            }
            Type::List(inner) => Ok(Type::List(Box::new(
                self.normalize_associated_types(inner, scheme, subst)?,
            ))),
            Type::Record(fields) => {
                let norm_fields = fields
                    .iter()
                    .map(|(n, t)| {
                        Ok((
                            n.clone(),
                            self.normalize_associated_types(t, scheme, subst)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Record(norm_fields))
            }
            other => Ok(other.clone()),
        }
    }

    /// Resolve a canonical `Interface::method(value)` call.
    pub fn resolve_interface_method_call(
        &self,
        interface: &str,
        method: &str,
        arg_types: &[Type],
    ) -> Result<Type, TypeEnvError> {
        let (selected, scheme) = self.select_impl_scheme(interface, method, arg_types)?;
        let method_info = scheme
            .methods
            .iter()
            .find(|m| m.name == method)
            .ok_or_else(|| TypeEnvError::MissingInterfaceMethod {
                interface: interface.to_string(),
                method: method.to_string(),
                span: Span::default(),
            })?;
        let raw_return = selected.substitution.apply(&method_info.return_type);
        self.normalize_associated_types(&raw_return, scheme, &selected.substitution)
    }

    pub fn select_impl_scheme(
        &self,
        interface: &str,
        method: &str,
        arg_types: &[Type],
    ) -> Result<(SelectedScheme, &ImplScheme), TypeEnvError> {
        let interface_info = self.interfaces.get(interface).ok_or_else(|| {
            TypeEnvError::MissingInterface(interface.to_string(), Span::default())
        })?;

        let method_info = interface_info.methods.get(method).ok_or_else(|| {
            TypeEnvError::MissingInterfaceMethod {
                interface: interface.to_string(),
                method: method.to_string(),
                span: Span::default(),
            }
        })?;

        if method_info.params.len() != arg_types.len() {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface method '{}::{}' expects {} arguments, found {}",
                    interface,
                    method,
                    method_info.params.len(),
                    arg_types.len()
                ),
                Span::default(),
            ));
        }

        if method_info
            .params
            .iter()
            .any(type_contains_constructor_variable_app)
        {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' type parameters could not be fully determined from arguments; evidence lookup does not invert constructor-variable applications",
                    interface
                ),
                Span::default(),
            ));
        }

        let mut subst = Substitution::new();
        for (expected, actual) in method_info.params.iter().zip(arg_types.iter()) {
            let sub = self
                .unify_types(&subst.apply(expected), actual)
                .map_err(|e| TypeEnvError::InvalidDefinition(format!("{e}"), Span::default()))?;
            subst = subst.compose(&sub);
        }

        let head_args: Vec<Type> = method_info
            .type_params
            .iter()
            .map(|tp| subst.apply(&Type::Var(*tp)))
            .collect();

        if head_args.iter().any(|t| {
            if let Type::Var(var) = t {
                !self.type_var_has_interface_bound(*var, interface)
            } else {
                false
            }
        }) {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "interface '{}' type parameters could not be fully determined from arguments",
                    interface
                ),
                Span::default(),
            ));
        }

        let target_head = Type::Constructor {
            name: QualifiedName::root(interface.to_string()),
            args: head_args,
            kind: Kind::Type,
        };

        let (selected, scheme) = self.find_matching_impl_scheme(interface, &target_head, 0)?;

        if !scheme.methods.iter().any(|m| m.name == method) {
            return Err(TypeEnvError::MissingInterfaceMethod {
                interface: interface.to_string(),
                method: method.to_string(),
                span: Span::default(),
            });
        }

        Ok((selected, scheme))
    }

    fn find_matching_impl_scheme(
        &self,
        interface: &str,
        target_head: &Type,
        depth: usize,
    ) -> Result<(SelectedScheme, &ImplScheme), TypeEnvError> {
        if depth > 32 {
            return Err(TypeEnvError::RecursiveBound {
                message: "depth limit".into(),
                span: Span::default(),
            });
        }
        for scheme in self.impls.iter().filter(|s| s.interface == interface) {
            if let Ok(scheme_subst) = self.unify_types(&scheme.head, target_head) {
                let mut bounds_ok = true;
                for bound in &scheme.where_bounds {
                    let bounded_ty = scheme_subst.apply(&Type::Var(bound.type_var));
                    let bound_head = Type::Constructor {
                        name: QualifiedName::root(bound.interface.clone()),
                        args: vec![bounded_ty],
                        kind: Kind::Type,
                    };
                    match self.find_matching_impl_scheme(&bound.interface, &bound_head, depth + 1) {
                        Ok(_) => {}
                        Err(TypeEnvError::RecursiveBound { .. }) => {
                            return Err(TypeEnvError::RecursiveBound {
                                message: "depth limit".into(),
                                span: Span::default(),
                            });
                        }
                        Err(_) => {
                            bounds_ok = false;
                            break;
                        }
                    }
                }
                if bounds_ok {
                    return Ok((
                        SelectedScheme {
                            substitution: scheme_subst,
                        },
                        scheme,
                    ));
                }
            }
        }
        Err(TypeEnvError::MissingImpl {
            interface: interface.to_string(),
            ty: target_head.to_string(),
            span: Span::default(),
        })
    }

    /// Resolve a type name to its qualified form and info
    pub fn resolve_type(
        &self,
        name: &str,
    ) -> Result<(QualifiedName, Option<&TypeInfo>), TypeError> {
        // Try as primitive first
        match name {
            "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" | "()" => {
                return Ok((
                    QualifiedName::root(if name == "Unit" { "Null" } else { name }),
                    None,
                ));
            }
            _ => {}
        }

        // Try local types. Identity-only summaries deliberately resolve as
        // names with known arity but without unfoldable representation.
        if self.type_info.contains_key(name) {
            if self.is_identity_only_name(name) {
                return Ok((QualifiedName::root(name), None));
            }
            return Ok((QualifiedName::root(name), self.type_info.get(name)));
        }

        // Try AST types for types not yet converted
        if self.ast_types.contains_key(name) {
            return Ok((QualifiedName::root(name), None));
        }

        Err(TypeError::UnboundVariable(
            name.to_string(),
            Span::default(),
        ))
    }

    /// Check the number of type arguments supplied to a known builtin process type constructor.
    pub fn check_type_constructor_arity(
        &self,
        name: &QualifiedName,
        found_arity: usize,
    ) -> Result<(), TypeError> {
        if !name.is_root() {
            return Ok(());
        }

        match self.interfaces.get(&name.name) {
            Some(interface) if found_arity > 0 => {
                let expected_arity = interface.type_params.len();
                if expected_arity != found_arity {
                    return Err(TypeError::ConstructorArityMismatch {
                        name: name.display(),
                        expected_arity,
                        found_arity,
                        span: Span::default(),
                    });
                }
                return Ok(());
            }
            _ => {}
        }

        let Some(type_def) = self.ast_types.get(&name.name) else {
            return Ok(());
        };

        if self.is_placeholder_name(&name.name) {
            return Ok(());
        }

        let expected_arity = self
            .type_info
            .get(&name.name)
            .map(TypeInfo::type_arg_count)
            .unwrap_or_else(|| type_def.params.len());

        if expected_arity != found_arity {
            return Err(TypeError::ConstructorArityMismatch {
                name: name.display(),
                expected_arity,
                found_arity,
                span: Span::default(),
            });
        }

        Ok(())
    }

    /// Unfold a constructor to its definition with type arguments substituted
    pub fn unfold_constructor(
        &self,
        name: &QualifiedName,
        args: &[Type],
    ) -> Result<UnfoldedBody, TypeError> {
        let (_, type_info) = self.resolve_type(&name.name)?;

        let type_info =
            type_info.ok_or_else(|| TypeError::NotAConstructor(name.display(), Span::default()))?;

        match type_info {
            TypeInfo::Enum {
                params, variants, ..
            } => {
                if params.len() != args.len() {
                    return Err(TypeError::ConstructorArityMismatch {
                        name: name.display(),
                        expected_arity: params.len(),
                        found_arity: args.len(),
                        span: Span::default(),
                    });
                }

                // Create substitution from param vars to args
                let subst = params.iter().copied().zip(args.iter().cloned()).fold(
                    Substitution::new(),
                    |mut acc, (var, ty)| {
                        acc.insert(var, ty);
                        acc
                    },
                );

                // Apply substitution to variants
                let unfolded_variants: Vec<_> = variants
                    .iter()
                    .map(|v| VariantInfo {
                        name: v.name.clone(),
                        fields: v
                            .fields
                            .iter()
                            .map(|(n, t)| (n.clone(), subst.apply(t)))
                            .collect(),
                        payload_shape: v.payload_shape.clone(),
                    })
                    .collect();

                Ok(UnfoldedBody::Enum(unfolded_variants))
            }
            TypeInfo::Struct { params, fields, .. } => {
                if params.len() != args.len() {
                    return Err(TypeError::ConstructorArityMismatch {
                        name: name.display(),
                        expected_arity: params.len(),
                        found_arity: args.len(),
                        span: Span::default(),
                    });
                }

                // Create substitution from param vars to args
                let subst = params.iter().copied().zip(args.iter().cloned()).fold(
                    Substitution::new(),
                    |mut acc, (var, ty)| {
                        acc.insert(var, ty);
                        acc
                    },
                );

                // Apply substitution to fields
                let unfolded_fields: Vec<_> = fields
                    .iter()
                    .map(|(n, t)| (n.clone(), subst.apply(t)))
                    .collect();

                Ok(UnfoldedBody::Struct(unfolded_fields))
            }
        }
    }

    // ============================================================
    // Capability Provider Methods
    // ============================================================

    /// Register a capability provider.
    ///
    /// # Arguments
    /// * `name` - The provider name (e.g., "io", "http", "db")
    pub fn register_provider(&mut self, name: impl Into<String>) {
        self.providers.insert(name.into());
    }

    /// Check if a provider is registered.
    ///
    /// # Arguments
    /// * `name` - The provider name to check
    ///
    /// # Returns
    /// * `true` - If the provider is registered or if checking is not strict
    /// * `false` - If the provider is not registered (only in strict mode)
    pub fn has_provider(&self, name: &str) -> bool {
        // For now, accept any provider to maintain backward compatibility
        // TODO: Add strict mode that only accepts registered providers
        self.providers.is_empty() || self.providers.contains(name)
    }

    /// Get all registered providers.
    pub fn providers(&self) -> &HashSet<String> {
        &self.providers
    }
}

/// Unfolded type body with substituted type arguments
#[derive(Debug, Clone, PartialEq)]
pub enum UnfoldedBody {
    /// Enum with variants
    Enum(Vec<VariantInfo>),
    /// Struct with fields
    Struct(Vec<(FieldName, Type)>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, Visibility};
    use ash_core::semantic_summary::ConstructorId;
    use ash_core::type_ir::PromotedConstructorApp;

    // ============================================================
    // TypeInfo Tests
    // ============================================================

    #[test]
    fn test_type_info_name() {
        let enum_def = TypeInfo::Enum {
            name: "Option".to_string(),
            params: vec![],
            variants: vec![],
        };
        assert_eq!(enum_def.name(), "Option");

        let struct_def = TypeInfo::Struct {
            name: "Point".to_string(),
            params: vec![],
            fields: vec![],
        };
        assert_eq!(struct_def.name(), "Point");
    }

    #[test]
    fn test_type_info_lookup_variant() {
        let enum_def = TypeInfo::Enum {
            name: "Option".to_string(),
            params: vec![],
            variants: vec![
                VariantInfo {
                    name: "Some".to_string(),
                    fields: vec![("value".to_string(), Type::Int)],
                    payload_shape: VariantPayloadShape::Record,
                },
                VariantInfo {
                    name: "None".to_string(),
                    fields: vec![],
                    payload_shape: VariantPayloadShape::Unit,
                },
            ],
        };

        let (idx, variant) = enum_def.lookup_variant("Some").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(variant.name, "Some");

        let (idx, variant) = enum_def.lookup_variant("None").unwrap();
        assert_eq!(idx, 1);
        assert_eq!(variant.name, "None");

        assert!(enum_def.lookup_variant("Unknown").is_none());
    }

    #[test]
    fn test_struct_info_lookup_variant_returns_none() {
        let struct_def = TypeInfo::Struct {
            name: "Point".to_string(),
            params: vec![],
            fields: vec![("x".to_string(), Type::Int)],
        };
        assert!(struct_def.lookup_variant("x").is_none());
    }

    // ============================================================
    // TypeEnv Tests
    // ============================================================

    #[test]
    fn test_type_env_new() {
        let env = TypeEnv::new();
        assert!(!env.has_type("Option"));
        assert!(!env.has_constructor("Some"));
    }

    #[test]
    fn test_type_env_with_builtin_types() {
        let env = TypeEnv::with_builtin_types();

        // Check Option type exists
        assert!(env.has_type("Option"));
        assert!(env.has_constructor("Some"));
        assert!(env.has_constructor("None"));

        // Check Result type exists
        assert!(env.has_type("Result"));
        assert!(env.has_constructor("Ok"));
        assert!(env.has_constructor("Err"));

        // Check visible Act algebra exists while the hidden runtime ActEnv does not.
        assert!(!env.has_type("ActEnv"));
        assert!(env.has_type("Act"));
    }

    #[test]
    fn test_lookup_constructor() {
        let env = TypeEnv::with_builtin_types();

        let (type_name, variant_idx) = env.lookup_constructor("Some").unwrap();
        assert_eq!(type_name, "Option");
        assert_eq!(variant_idx, 0);

        let (type_name, variant_idx) = env.lookup_constructor("None").unwrap();
        assert_eq!(type_name, "Option");
        assert_eq!(variant_idx, 1);

        let (type_name, variant_idx) = env.lookup_constructor("Ok").unwrap();
        assert_eq!(type_name, "Result");
        assert_eq!(variant_idx, 0);

        let (type_name, variant_idx) = env.lookup_constructor("Err").unwrap();
        assert_eq!(type_name, "Result");
        assert_eq!(variant_idx, 1);

        assert!(env.lookup_constructor("Unknown").is_none());
    }

    #[test]
    fn test_lookup_type() {
        let env = TypeEnv::with_builtin_types();

        let type_def = env.lookup_type("Option").unwrap();
        assert_eq!(type_def.name, "Option");
        assert_eq!(type_def.params.len(), 1);

        let type_def = env.lookup_type("Result").unwrap();
        assert_eq!(type_def.name, "Result");
        assert_eq!(type_def.params.len(), 2);

        assert!(env.lookup_type("Unknown").is_none());
    }

    #[test]
    fn test_get_variant() {
        let env = TypeEnv::with_builtin_types();

        let (type_info, variant_idx, variant) = env.get_variant("Some").unwrap();
        assert_eq!(type_info.name(), "Option");
        assert_eq!(variant_idx, 0);
        assert_eq!(variant.name, "Some");
        assert_eq!(variant.fields.len(), 1);
        assert_eq!(variant.fields[0].0, "value");

        let (_, _, variant) = env.get_variant("None").unwrap();
        assert_eq!(variant.name, "None");
        assert!(variant.fields.is_empty());

        assert!(env.get_variant("Unknown").is_none());
    }

    #[test]
    fn test_register_custom_type() {
        let mut env = TypeEnv::new();

        let status_type = TypeDef {
            name: "Status".to_string(),
            params: vec![],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Pending".to_string(),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                },
                VariantDef {
                    name: "Complete".to_string(),
                    fields: vec![("result".to_string(), TypeExpr::Named("Int".to_string()))],
                    payload: VariantPayload::Record(vec![(
                        "result".to_string(),
                        TypeExpr::Named("Int".to_string()),
                    )]),
                },
            ]),
            visibility: Visibility::Public,
            builtin: false,
        };

        env.register_type(&status_type).unwrap();

        assert!(env.has_type("Status"));
        assert!(env.has_constructor("Pending"));
        assert!(env.has_constructor("Complete"));

        let (type_name, idx) = env.lookup_constructor("Pending").unwrap();
        assert_eq!(type_name, "Status");
        assert_eq!(idx, 0);

        let (type_name, idx) = env.lookup_constructor("Complete").unwrap();
        assert_eq!(type_name, "Status");
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_register_type_identity_keeps_constructors_hidden() {
        let mut env = TypeEnv::new();

        let hidden_type = TypeDef {
            name: "Hidden".to_string(),
            params: vec!["A".to_string()],
            body: TypeBody::Enum(vec![VariantDef {
                name: "Hidden".to_string(),
                fields: vec![("value".to_string(), TypeExpr::Named("A".to_string()))],
                payload: VariantPayload::Record(vec![(
                    "value".to_string(),
                    TypeExpr::Named("A".to_string()),
                )]),
            }]),
            visibility: Visibility::Private,
            builtin: false,
        };

        env.register_type_identity(&hidden_type).unwrap();

        let type_def = env
            .lookup_type("Hidden")
            .expect("type identity should register");
        assert_eq!(type_def.params.len(), 1);
        assert!(
            env.lookup_constructor("Hidden").is_none(),
            "identity-only registration should not expose constructors"
        );
    }

    #[test]
    fn test_expose_type_representation_registers_constructors_after_identity() {
        let mut env = TypeEnv::new();

        let hidden_type = TypeDef {
            name: "Hidden".to_string(),
            params: vec![],
            body: TypeBody::Enum(vec![VariantDef {
                name: "Reveal".to_string(),
                fields: vec![("value".to_string(), TypeExpr::Named("Int".to_string()))],
                payload: VariantPayload::Record(vec![(
                    "value".to_string(),
                    TypeExpr::Named("Int".to_string()),
                )]),
            }]),
            visibility: Visibility::Private,
            builtin: false,
        };

        env.register_type_identity(&hidden_type).unwrap();
        assert!(env.lookup_constructor("Reveal").is_none());

        env.expose_type_representation("Hidden").unwrap();

        let (type_name, variant_idx) = env
            .lookup_constructor("Reveal")
            .expect("constructor should become visible after representation exposure");
        assert_eq!(type_name, "Hidden");
        assert_eq!(variant_idx, 0);
    }

    #[test]
    fn test_option_type_structure() {
        let env = TypeEnv::with_builtin_types();

        // Check AST type definition
        let type_def = env.lookup_type("Option").unwrap();
        assert_eq!(type_def.name, "Option");
        assert_eq!(type_def.params.len(), 1);

        // Check internal type info
        let type_info = env.lookup_type_info("Option").unwrap();
        match type_info {
            TypeInfo::Enum {
                name,
                params,
                variants,
            } => {
                assert_eq!(name, "Option");
                assert_eq!(params.len(), 1);
                assert_eq!(variants.len(), 2);

                // Some variant
                assert_eq!(variants[0].name, "Some");
                assert_eq!(variants[0].fields.len(), 1);
                assert_eq!(variants[0].fields[0].0, "value");
                // Should be a type variable
                assert!(matches!(variants[0].fields[0].1, Type::Var(_)));

                // None variant
                assert_eq!(variants[1].name, "None");
                assert!(variants[1].fields.is_empty());
            }
            _ => panic!("Option should be an enum"),
        }
    }

    #[test]
    fn test_result_type_structure() {
        let env = TypeEnv::with_builtin_types();

        // Check AST type definition
        let ast_type_def = env.lookup_type("Result").unwrap();
        assert_eq!(ast_type_def.name, "Result");
        assert_eq!(ast_type_def.params.len(), 2);

        // Check internal type info
        let type_info = env.lookup_type_info("Result").unwrap();
        match type_info {
            TypeInfo::Enum {
                name,
                params,
                variants,
            } => {
                assert_eq!(name, "Result");
                assert_eq!(params.len(), 2);
                assert_eq!(variants.len(), 2);

                // Ok variant
                assert_eq!(variants[0].name, "Ok");
                assert_eq!(variants[0].fields.len(), 1);
                assert_eq!(variants[0].fields[0].0, "value");

                // Err variant
                assert_eq!(variants[1].name, "Err");
                assert_eq!(variants[1].fields.len(), 1);
                assert_eq!(variants[1].fields[0].0, "error");
            }
            _ => panic!("Result should be an enum"),
        }
    }

    #[test]
    fn type_expr_constructor_converts_properly() {
        use crate::kind::Kind;

        let env = TypeEnv::with_builtin_types();

        // Option<Int> should become Constructor { name: "Option", args: [Int] }
        let type_expr = TypeExpr::Constructor {
            name: "Option".to_string(),
            args: vec![TypeExpr::Named("Int".to_string())],
        };

        let ty = type_expr_to_type(&type_expr, &HashMap::new(), &env).unwrap();

        match ty {
            Type::Constructor { name, args, kind } => {
                assert_eq!(name.display(), "Option");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Type::Int);
                assert_eq!(kind, Kind::Type);
            }
            _ => panic!("Expected Type::Constructor, got {:?}", ty),
        }
    }

    #[test]
    fn task689d_act_env_type_expr_is_not_source_denotable() {
        let env = TypeEnv::with_builtin_types();
        let type_expr = TypeExpr::Constructor {
            name: "Fn".to_string(),
            args: vec![
                TypeExpr::Named("ActEnv".to_string()),
                TypeExpr::Tuple(vec![
                    TypeExpr::Named("ActEnv".to_string()),
                    TypeExpr::Named("Int".to_string()),
                ]),
            ],
        };

        let err = type_expr_to_type(&type_expr, &HashMap::new(), &env)
            .expect_err("ActEnv is runtime-owned and not source-denotable");
        assert!(err.to_string().contains("ActEnv"), "{err}");
    }

    fn task896_module_identity() -> ModuleIdentity {
        ModuleIdentity::new(
            Some(CrateId(896)),
            ModuleId(118),
            vec!["typeenv".into(), "task896".into()],
            ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
                reason: "task-896-type-function-promoted-closure".into(),
            },
        )
    }

    fn task896_source_anchor(label: &str) -> SourceAnchor {
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: "task-896-type-function-promoted-closure".into(),
            },
            None,
            label,
        )
    }

    fn task896_promoted_nat_summary_for_typeenv(
        visibility: Visibility,
    ) -> (
        ModuleSemanticSummary,
        PromotedDataKindId,
        PromotedConstructorId,
    ) {
        let module = task896_module_identity();
        let source_type = TypeDeclId::ordinary(module.clone(), "Nat");
        let source_constructor =
            ConstructorId::variant(source_type.clone(), "Z", ConstructorPayloadKind::Unit);
        let kind = PromotedDataKindId::new(module.clone(), source_type.clone(), "NatKind");
        let constructor = PromotedConstructorId::new(kind.clone(), source_constructor.clone(), "Z");
        let summary = ModuleSemanticSummary::new(module.clone())
            .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
            .with_exported_type(TypeDeclSummary::new(
                source_type.clone(),
                "Nat",
                Visibility::Public,
                RepresentationExposure::Exposed,
                TypeRepresentationSummary::Exposed(TypeBody::Enum(vec![VariantDef {
                    name: "Z".into(),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                }])),
                task896_source_anchor("Nat"),
            ))
            .with_exported_constructor(ConstructorSummary::new(
                source_constructor.clone(),
                source_type.clone(),
                "Z",
                ConstructorPayloadKind::Unit,
                Visibility::Public,
                task896_source_anchor("Z"),
            ))
            .with_exported_promoted_data_kind(
                PromotedDataKindSummary::new(
                    kind.clone(),
                    "NatKind",
                    visibility,
                    source_type,
                    task896_source_anchor("NatKind"),
                )
                .with_constructor(PromotedConstructorSummary::new(
                    constructor.clone(),
                    "Z",
                    source_constructor,
                    vec![],
                    visibility,
                    task896_source_anchor("promoted Z"),
                )),
            );
        (summary, kind, constructor)
    }

    fn task896_type_function_def_returning_promoted_z(
        module: &ModuleIdentity,
        kind: &PromotedDataKindId,
        constructor: &PromotedConstructorId,
    ) -> TypeFunctionDef {
        let head = TypeComputationHeadId::new(module.clone(), "ZeroNat");
        TypeFunctionDef {
            visibility: Visibility::Public,
            head: head.clone(),
            name: "ZeroNat".into(),
            params: vec![],
            return_type: CanonicalTypeExpr::Primitive("Type".into()),
            return_kind: Kind::Type,
            result_constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
            decreases: None,
            source_anchors: TypeFunctionSourceAnchors {
                definition: task896_source_anchor("type fn ZeroNat"),
                decreases: None,
            },
            equations: vec![TypeFunctionEquation {
                head,
                ordinal: 0,
                patterns: vec![],
                result: TypeFunctionResultExpr::PromotedDataConstructorApp {
                    constructor: Box::new(constructor.clone()),
                    data_kind: Box::new(kind.clone()),
                    args: vec![],
                    kind: Kind::Type,
                    constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
                    source_anchor: task896_source_anchor("ZeroNat rhs"),
                },
                source_anchor: task896_source_anchor("case ZeroNat = Z"),
                case_head_anchor: task896_source_anchor("ZeroNat case head"),
            }],
        }
    }

    #[test]
    fn task896_public_type_function_summary_records_promoted_data_kind_dependency() {
        let (summary, kind, constructor) =
            task896_promoted_nat_summary_for_typeenv(Visibility::Public);
        let mut env = TypeEnv::new();
        env.register_module_semantic_summary(&summary)
            .expect("public promoted kind imports");
        let def =
            task896_type_function_def_returning_promoted_z(&summary.module, &kind, &constructor);

        let exported = env
            .lower_public_type_function_summary(&def)
            .expect("public promoted constructor dependency is export-closed");

        assert!(exported.dependency_summary_refs.iter().any(|dependency| {
            dependency.summary_ref.module == summary.module
                && dependency.summary_ref.version == SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
        }));
    }

    #[test]
    fn task896_public_type_function_export_rejects_private_promoted_data_kind_dependency() {
        let (summary, kind, constructor) =
            task896_promoted_nat_summary_for_typeenv(Visibility::Public);
        let mut env = TypeEnv::new();
        env.register_module_semantic_summary(&summary)
            .expect("public promoted kind imports before privacy mutation");
        env.promoted_data_kind_summaries
            .get_mut(&kind)
            .expect("registered kind")
            .visibility = Visibility::Private;
        let def =
            task896_type_function_def_returning_promoted_z(&summary.module, &kind, &constructor);

        let err = env
            .lower_public_type_function_summary(&def)
            .expect_err("public type function must not leak private promoted data kind");
        let msg = err.to_string();
        assert!(
            msg.contains("private")
                && msg.contains("promoted data kind")
                && msg.contains("NatKind"),
            "unexpected diagnostic: {msg}"
        );
    }

    #[test]
    fn task896_public_type_function_export_rejects_private_promoted_constructor_dependency() {
        let (summary, kind, constructor) =
            task896_promoted_nat_summary_for_typeenv(Visibility::Public);
        let mut env = TypeEnv::new();
        env.register_module_semantic_summary(&summary)
            .expect("public promoted kind imports before constructor privacy mutation");
        env.promoted_constructor_summaries
            .get_mut(&constructor)
            .expect("registered promoted constructor")
            .visibility = Visibility::Private;
        let def =
            task896_type_function_def_returning_promoted_z(&summary.module, &kind, &constructor);

        let err = env
            .lower_public_type_function_summary(&def)
            .expect_err("public type function must not leak private promoted constructor");
        let msg = err.to_string();
        assert!(
            msg.contains("private")
                && msg.contains("promoted data constructor")
                && msg.contains("Z"),
            "unexpected diagnostic: {msg}"
        );
    }

    #[test]
    fn task896_associated_family_result_conversion_rejects_promoted_constructor_without_panic() {
        let (summary, kind, constructor) =
            task896_promoted_nat_summary_for_typeenv(Visibility::Public);
        let promoted =
            CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(PromotedConstructorApp {
                constructor,
                data_kind: kind,
                args: vec![],
                kind: Kind::Type,
            }));

        let err = associated_family_result_from_canonical(promoted, Span::new(118, 119, 1, 1))
            .expect_err("promoted constructors are not associated-family result carriers");
        let msg = err.to_string();
        assert!(
            msg.contains("promoted data constructor")
                && msg.contains("associated-family result")
                && msg.contains("Z"),
            "unexpected diagnostic for {:?}: {msg}",
            summary.module
        );
    }

    #[test]
    fn unfold_option_int() {
        let env = TypeEnv::with_builtin_types();

        // Unfold Option<Int>
        let unfolded = env
            .unfold_constructor(&QualifiedName::root("Option"), &[Type::Int])
            .unwrap();

        // Should get: Some { value: Int } | None
        match unfolded {
            UnfoldedBody::Enum(variants) => {
                assert_eq!(variants.len(), 2);

                // Check Some variant
                let some = &variants[0];
                assert_eq!(some.name, "Some");
                assert_eq!(some.fields.len(), 1);
                assert_eq!(some.fields[0].0, "value");
                assert_eq!(some.fields[0].1, Type::Int);

                // Check None variant
                let none = &variants[1];
                assert_eq!(none.name, "None");
                assert!(none.fields.is_empty());
            }
            _ => panic!("Expected enum body, got {:?}", unfolded),
        }
    }

    #[test]
    fn unfold_result_int_string() {
        let env = TypeEnv::with_builtin_types();

        // Unfold Result<Int, String>
        let unfolded = env
            .unfold_constructor(&QualifiedName::root("Result"), &[Type::Int, Type::String])
            .unwrap();

        // Should get: Ok { value: Int } | Err { error: String }
        match unfolded {
            UnfoldedBody::Enum(variants) => {
                assert_eq!(variants.len(), 2);

                // Check Ok variant
                let ok = &variants[0];
                assert_eq!(ok.name, "Ok");
                assert_eq!(ok.fields.len(), 1);
                assert_eq!(ok.fields[0].0, "value");
                assert_eq!(ok.fields[0].1, Type::Int);

                // Check Err variant
                let err = &variants[1];
                assert_eq!(err.name, "Err");
                assert_eq!(err.fields.len(), 1);
                assert_eq!(err.fields[0].0, "error");
                assert_eq!(err.fields[0].1, Type::String);
            }
            _ => panic!("Expected enum body, got {:?}", unfolded),
        }
    }

    #[test]
    fn unfold_constructor_wrong_arity() {
        let env = TypeEnv::with_builtin_types();

        // Option expects 1 type argument, but we provide 2
        let result =
            env.unfold_constructor(&QualifiedName::root("Option"), &[Type::Int, Type::String]);

        assert!(matches!(
            result,
            Err(TypeError::ConstructorArityMismatch {
                name,
                expected_arity: 1,
                found_arity: 2,
                ..
            }) if name == "Option"
        ));
    }
}
