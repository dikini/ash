use super::*;

/// Default fuel budget for Stage-3 proof totality checking.
pub const DEFAULT_PROOF_FUEL: usize = 1000;

/// Result status for the Stage-3 proof totality checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofTotalityStatus {
    /// The proof body was traversed within the configured fuel budget.
    Checked,
    /// The proof could not be checked conclusively in this slice.
    Untested(ProofTotalityUntestedReason),
}

/// Non-error reasons a proof remains untested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofTotalityUntestedReason {
    /// The proof-body normalization/traversal fuel budget was exhausted.
    FuelExhausted,
}

/// Structured outcome for proof totality checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofTotalityResult {
    /// Whether the checker completed or intentionally left the proof untested.
    pub status: ProofTotalityStatus,
    /// Configured fuel limit for this proof check.
    pub fuel_limit: usize,
    /// Remaining fuel after traversing the proof body.
    pub fuel_remaining: usize,
}

/// Typechecker-owned erased proof token for Stage-3 proof irrelevance.
///
/// The carrier preserves the proposition being proved while intentionally
/// discarding proof declaration identity, proof body, and witness identity. This
/// is a local/static typechecker artifact only; runtime proof escape prevention
/// remains a follow-on task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasedProof {
    /// Proposition retained as the equality boundary for erased proofs.
    pub proposition: TypeProposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TypeFunctionCoverageValue {
    pub(super) constructor: ash_core::semantic_summary::DomainConstructorId,
    pub(super) fields: Vec<Option<TypeFunctionCoverageValue>>,
}

#[derive(Debug, Clone)]
pub(super) struct TypeFunctionCoverageAlt {
    pub(super) constructor: ash_core::semantic_summary::DomainConstructorId,
    pub(super) fields: Vec<Option<TypeFunctionCoverageSpace>>,
}

#[derive(Debug, Clone)]
pub(super) struct TypeFunctionCoverageSpace {
    pub(super) domain: SealedDomainId,
    pub(super) alts: Vec<TypeFunctionCoverageAlt>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PublicTypeFunctionClosure {
    pub(super) ordinary_types: HashSet<TypeDeclId>,
    pub(super) sealed_domains: HashSet<SealedDomainId>,
    pub(super) promoted_data_kinds: HashSet<PromotedDataKindId>,
    pub(super) promoted_constructors: HashSet<PromotedConstructorId>,
    pub(super) type_functions: HashSet<TypeComputationHeadId>,
    pub(super) projections: HashSet<(InterfaceIdentityId, AssociatedMemberIdentityId)>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PublicAssociatedFamilyClosure {
    pub(super) ordinary_types: HashSet<TypeDeclId>,
    pub(super) sealed_domains: HashSet<SealedDomainId>,
    pub(super) domain_constructors: HashSet<DomainConstructorId>,
    pub(super) type_functions: HashSet<TypeComputationHeadId>,
    pub(super) projections: HashSet<AssociatedFamilyProjection>,
    pub(super) associated_families: HashSet<AssociatedFamilyHeadId>,
}

impl PublicAssociatedFamilyClosure {
    pub(super) fn associated_family_summary_refs(
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
    pub(super) fn dependency_summary_refs(&self) -> Vec<TypeFunctionDependencySummaryRef> {
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

pub(super) fn push_dependency_summary_ref(
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
pub(super) enum TypeDeclarationState {
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
                "Prop" => Ok(Type::Constructor {
                    name: QualifiedName::root("Prop"),
                    args: vec![],
                    kind: Kind::Prop,
                }),
                "Null" | "Unit" => Ok(Type::Null),
                "Time" => Ok(Type::Time),
                "Ref" => Ok(Type::Ref),
                _ => {
                    // User-defined type with no args - look it up
                    let (qualified, _) = type_env.resolve_type(name)?;
                    type_env.check_type_constructor_arity(&qualified, 0)?;
                    if let Some(target) = type_env.transparent_alias_target(&qualified, &[]) {
                        Ok(target)
                    } else {
                        Ok(Type::Constructor {
                            name: qualified,
                            args: vec![],
                            kind: Kind::Type,
                        })
                    }
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

                let arg_types = arg_types?;
                if let Some(target) = type_env.transparent_alias_target(&qualified, &arg_types) {
                    Ok(target)
                } else {
                    Ok(Type::Constructor {
                        name: qualified,
                        args: arg_types,
                        kind: Kind::Type,
                    })
                }
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

pub(super) fn primitive_pattern_type(name: &str) -> Option<Type> {
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
    /// Implicit method-level type variables appearing in the method signature.
    pub method_type_params: Vec<TypeVar>,
    /// Canonical single-argument parameter types.
    pub params: Vec<Type>,
    /// Declared return type.
    pub return_type: Type,
}

/// Interface-owned required evidence constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceEvidenceConstraintInfo {
    /// Interface parameter named to the left of `:`.
    pub subject_param: String,
    /// Position of the subject parameter in the constrained interface head.
    pub subject_param_index: usize,
    /// Required evidence interface named to the right of `:`.
    pub required_interface: String,
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
    /// Interface-owned required evidence constraints.
    pub evidence_constraints: Vec<InterfaceEvidenceConstraintInfo>,
    /// Law names declared by the interface.
    pub law_names: Vec<String>,
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
    pub method_type_params: Vec<TypeVar>,
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
pub(super) enum PropositionPredicateSolverKind {
    /// Ordinary source/imported predicates are opaque in TASK-878 and must defer.
    DeferredUnsupported,
    /// Compiler-owned builtin predicate explicitly registered in this TypeEnv.
    CompilerBuiltinSatisfied,
}

/// TypeEnv-owned named proposition predicate metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropositionPredicateInfo {
    pub summary: PropositionPredicateSummary,
    pub(super) solver_kind: PropositionPredicateSolverKind,
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
pub(super) enum AssociatedFamilyMatchFailure {
    NoMatch,
    Blocked(AssociatedFamilySelectionBlocker),
}

#[derive(Debug, Clone)]
pub struct SelectedScheme {
    pub substitution: Substitution,
}

#[derive(Debug, Default)]
pub(super) struct AliasCanonicalVarBridge {
    pub(super) next_var: u32,
    pub(super) args: HashMap<TypeVar, CanonicalTypeExpr>,
}

impl AliasCanonicalVarBridge {
    pub(super) fn placeholder_for_arg(&mut self, expr: &CanonicalTypeExpr) -> Type {
        let var = TypeVar(0x8230_0000u32.wrapping_add(self.next_var));
        self.next_var = self.next_var.wrapping_add(1);
        self.args.insert(var, expr.clone());
        Type::Var(var)
    }
}

pub(super) fn fallback_canonical_type_decl_id(name: &str) -> TypeDeclId {
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

pub(super) fn resolve_associated_interface_from_type_var_bounds(
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

pub(super) fn lower_explicit_associated_family_projection_to_type(
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

pub(super) fn lower_core_explicit_associated_family_projection_to_type(
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

pub(super) fn constructor_variable_application_to_type(
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

pub(super) fn surface_type_to_type(
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
                "Prop" => Ok(Type::Constructor {
                    name: QualifiedName::root("Prop"),
                    args: vec![],
                    kind: Kind::Prop,
                }),
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

pub(super) fn surface_type_name(ty: &SurfaceType) -> Option<String> {
    match ty {
        SurfaceType::Name(name) => Some(name.to_string()),
        SurfaceType::Capability(name) => Some(name.to_string()),
        _ => None,
    }
}

pub(super) fn is_primitive_surface_type_name(name: &str) -> bool {
    matches!(
        name,
        "Int" | "String" | "Bool" | "Float" | "Null" | "Unit" | "Time" | "Ref" | "()"
    )
}

pub(super) fn collect_implicit_interface_method_type_params(
    ty: &SurfaceType,
    param_mapping: &HashMap<String, TypeVar>,
    type_env: &TypeEnv,
    implicit_params: &mut BTreeMap<String, TypeVar>,
) {
    match ty {
        SurfaceType::Name(name) => {
            let name = name.as_ref();
            if !param_mapping.contains_key(name)
                && !is_primitive_surface_type_name(name)
                && type_env.type_parameter_kind(name).is_none()
                && type_env.resolve_type(name).is_err()
            {
                implicit_params
                    .entry(name.to_string())
                    .or_insert_with(TypeVar::fresh);
            }
        }
        SurfaceType::Constructor { name, args } => {
            let name = name.as_ref();
            if param_mapping.contains_key(name)
                || type_env.type_parameter_kind(name).is_some()
                || name == "List"
                || type_env.resolve_type(name).is_ok()
            {
                for arg in args {
                    collect_implicit_interface_method_type_params(
                        arg,
                        param_mapping,
                        type_env,
                        implicit_params,
                    );
                }
            }
        }
        SurfaceType::List(item) => collect_implicit_interface_method_type_params(
            item,
            param_mapping,
            type_env,
            implicit_params,
        ),
        SurfaceType::Tuple(items) => {
            for item in items {
                collect_implicit_interface_method_type_params(
                    item,
                    param_mapping,
                    type_env,
                    implicit_params,
                );
            }
        }
        SurfaceType::Record(fields) => {
            for (_, field_ty) in fields {
                collect_implicit_interface_method_type_params(
                    field_ty,
                    param_mapping,
                    type_env,
                    implicit_params,
                );
            }
        }
        SurfaceType::Fn(params, ret) => {
            for param in params {
                collect_implicit_interface_method_type_params(
                    param,
                    param_mapping,
                    type_env,
                    implicit_params,
                );
            }
            collect_implicit_interface_method_type_params(
                ret,
                param_mapping,
                type_env,
                implicit_params,
            );
        }
        SurfaceType::Associated { base, .. } => collect_implicit_interface_method_type_params(
            base,
            param_mapping,
            type_env,
            implicit_params,
        ),
        SurfaceType::AssociatedFamilyProjection { args, .. } => {
            for arg in args {
                collect_implicit_interface_method_type_params(
                    arg,
                    param_mapping,
                    type_env,
                    implicit_params,
                );
            }
        }
        SurfaceType::Hole { .. } | SurfaceType::Capability(_) => {}
    }
}

pub(super) fn bind_constructor_variable_for_method_call(
    constructor: &str,
    applied_arity: usize,
    actual_constructor: Type,
    constructor_bindings: &mut HashMap<String, Type>,
) -> Result<(), TypeEnvError> {
    if let Some(existing) = constructor_bindings.get(constructor) {
        if existing != &actual_constructor {
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "constructor variable '{constructor}' was inferred as both {existing} and {actual_constructor}"
                ),
                Span::default(),
            ));
        }
        return Ok(());
    }

    let mut constructor_ty = actual_constructor;
    if let Type::Constructor { kind, .. } = &mut constructor_ty {
        *kind = Kind::n_ary(applied_arity);
    }
    constructor_bindings.insert(constructor.to_string(), constructor_ty);
    Ok(())
}

pub(super) fn match_interface_method_call_pattern(
    type_env: &TypeEnv,
    expected: &Type,
    actual: &Type,
    substitution: &mut Substitution,
    constructor_bindings: &mut HashMap<String, Type>,
) -> Result<(), TypeEnvError> {
    match substitution.apply(expected) {
        Type::ConstructorVariableApp {
            constructor, args, ..
        } => {
            let actual_args = match actual {
                Type::Constructor {
                    name,
                    args: actual_args,
                    ..
                } => {
                    if actual_args.len() < args.len() {
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "constructor-variable application '{constructor}<...>' expected at least {} arguments, found {}",
                                args.len(),
                                actual_args.len()
                            ),
                            Span::default(),
                        ));
                    }
                    bind_constructor_variable_for_method_call(
                        &constructor,
                        args.len(),
                        Type::Constructor {
                            name: name.clone(),
                            args: actual_args[args.len()..].to_vec(),
                            kind: Kind::n_ary(args.len()),
                        },
                        constructor_bindings,
                    )?;
                    actual_args[..args.len()].to_vec()
                }
                Type::List(item) if args.len() == 1 => {
                    bind_constructor_variable_for_method_call(
                        &constructor,
                        1,
                        Type::Constructor {
                            name: QualifiedName::root("List"),
                            args: Vec::new(),
                            kind: Kind::n_ary(1),
                        },
                        constructor_bindings,
                    )?;
                    vec![item.as_ref().clone()]
                }
                _ => {
                    return Err(TypeEnvError::InvalidDefinition(
                        format!(
                            "constructor-variable application '{constructor}<...>' cannot match argument type {actual}"
                        ),
                        Span::default(),
                    ));
                }
            };

            if args.len() != actual_args.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "constructor-variable application '{constructor}<...>' expected {} arguments, found {}",
                        args.len(),
                        actual_args.len()
                    ),
                    Span::default(),
                ));
            }
            for (expected_arg, actual_arg) in args.iter().zip(actual_args.iter()) {
                match_interface_method_call_pattern(
                    type_env,
                    expected_arg,
                    actual_arg,
                    substitution,
                    constructor_bindings,
                )?;
            }
            Ok(())
        }
        Type::Fn(expected_params, expected_ret) => {
            let Type::Fn(actual_params, actual_ret) = actual else {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("expected function type, found {actual}"),
                    Span::default(),
                ));
            };
            if expected_params.len() != actual_params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "function arity mismatch: expected {}, found {}",
                        expected_params.len(),
                        actual_params.len()
                    ),
                    Span::default(),
                ));
            }
            for (expected_param, actual_param) in expected_params.iter().zip(actual_params.iter()) {
                match_interface_method_call_pattern(
                    type_env,
                    expected_param,
                    actual_param,
                    substitution,
                    constructor_bindings,
                )?;
            }
            match_interface_method_call_pattern(
                type_env,
                &expected_ret,
                actual_ret,
                substitution,
                constructor_bindings,
            )
        }
        Type::Fun(expected_params, expected_ret, expected_effect) => {
            let Type::Fun(actual_params, actual_ret, actual_effect) = actual else {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("expected effectful function type, found {actual}"),
                    Span::default(),
                ));
            };
            if expected_effect != *actual_effect || expected_params.len() != actual_params.len() {
                return Err(TypeEnvError::InvalidDefinition(
                    "effectful function type mismatch".to_string(),
                    Span::default(),
                ));
            }
            for (expected_param, actual_param) in expected_params.iter().zip(actual_params.iter()) {
                match_interface_method_call_pattern(
                    type_env,
                    expected_param,
                    actual_param,
                    substitution,
                    constructor_bindings,
                )?;
            }
            match_interface_method_call_pattern(
                type_env,
                &expected_ret,
                actual_ret,
                substitution,
                constructor_bindings,
            )
        }
        Type::List(expected_item) => {
            let Type::List(actual_item) = actual else {
                return Err(TypeEnvError::InvalidDefinition(
                    format!("expected list type, found {actual}"),
                    Span::default(),
                ));
            };
            match_interface_method_call_pattern(
                type_env,
                &expected_item,
                actual_item,
                substitution,
                constructor_bindings,
            )
        }
        expected => {
            let sub = type_env
                .unify_types(&expected, actual)
                .map_err(|e| TypeEnvError::InvalidDefinition(format!("{e}"), Span::default()))?;
            *substitution = substitution.compose(&sub);
            Ok(())
        }
    }
}

pub(super) fn core_projection_base_spelling(base: &TypeExpr) -> String {
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

pub(super) fn surface_projection_base_spelling(base: &SurfaceType) -> String {
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

pub(super) fn source_span_from_parser_span(span: Span) -> ash_core::ast::Span {
    ash_core::ast::Span {
        start: span.start,
        end: span.end,
    }
}

pub(super) fn proposition_source_anchor(
    origin: SourceOrigin,
    span: Span,
    label: impl Into<String>,
) -> SourceAnchor {
    SourceAnchor::new(origin, Some(source_span_from_parser_span(span)), label)
}

pub(super) fn proposition_module_source_origin(module: &ModuleIdentity) -> SourceOrigin {
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

pub(super) fn synthetic_proposition_source_anchor(label: impl Into<String>) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "typeenv proposition environment".to_string(),
        },
        None,
        label,
    )
}

pub(super) fn type_var_proposition_term(var: TypeVar) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Var(format!("type_var_{}", var.0)))
}

pub(super) fn interface_constraint_subject_term(
    subject: &TypePropositionTerm,
    interface_args: &[TypePropositionTerm],
    subject_param_index: usize,
) -> Option<TypePropositionTerm> {
    if subject_param_index == 0 {
        Some(subject.clone())
    } else {
        interface_args.get(subject_param_index - 1).cloned()
    }
}

pub(super) fn proposition_term_from_canonical(expr: CanonicalTypeExpr) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(expr)
}

pub(super) fn proposition_normalization_error(
    error: crate::normalizer::NormalizationError,
) -> TypeError {
    TypeEnvError::InvalidDefinition(
        format!("proposition normalization failed: {error:?}"),
        Span::default(),
    )
    .into()
}

pub(super) fn proposition_revalidation_error(error: TypeError) -> TypeEnvError {
    match error {
        TypeError::TypeEnv(error) => *error,
        other => TypeEnvError::InvalidDefinition(
            format!("proposition fact revalidation failed: {other}"),
            Span::default(),
        ),
    }
}

pub(super) fn constructor_kinded_binder_error(
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

pub(super) fn reject_constructor_kinded_interface_params(
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

pub(super) fn reject_constructor_kinded_proposition_params(
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

pub(super) fn required_proposition_discharge_error(
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

pub(super) fn proposition_help_from_outcome(outcome: &PropositionOutcome) -> String {
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

pub(super) fn proposition_shape_from_outcome(outcome: &PropositionOutcome) -> String {
    match outcome {
        PropositionOutcome::Satisfied(evidence) => format!("{:?}", evidence.proposition),
        PropositionOutcome::Refuted(refutation) => format!("{:?}", refutation.proposition),
        PropositionOutcome::Deferred(reason) => format!("{:?}", reason.proposition),
    }
}

pub(super) fn proposition_found_shape_from_outcome(outcome: &PropositionOutcome) -> String {
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

pub(super) fn proposition_solver_rule_from_outcome(outcome: &PropositionOutcome) -> &'static str {
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

pub(super) fn proposition_diagnostic_kind_from_outcome(
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

pub(super) fn private_proposition_dependency_error(
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

pub(super) fn proposition_comparison_terms(
    lhs: NormalTypeExpr,
    rhs: NormalTypeExpr,
) -> PropositionTypeComparisonEvidence {
    PropositionTypeComparisonEvidence { lhs, rhs }
}

pub(super) fn proposition_satisfaction(
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

pub(super) fn proposition_refutation(
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

pub(super) fn proposition_deferral(
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

pub(super) fn proposition_deferred_kind_from_blocked_normals(
    lhs_norm: &NormalTypeExpr,
    rhs_norm: &NormalTypeExpr,
) -> PropositionDeferredKind {
    let mut blockers = Vec::new();
    collect_proposition_blockers(lhs_norm, &mut blockers);
    collect_proposition_blockers(rhs_norm, &mut blockers);
    proposition_deferred_kind_from_blockers(&blockers)
}

pub(super) fn proposition_deferred_kind_from_blockers(
    blockers: &[NormalTypeExpr],
) -> PropositionDeferredKind {
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

pub(super) fn normal_form_block_reason(normal: &NormalTypeExpr) -> Option<NormalFormBlockReason> {
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

pub(super) fn collect_proposition_blockers(
    normal: &NormalTypeExpr,
    blockers: &mut Vec<NormalTypeExpr>,
) {
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

pub(super) fn proposition_normal_form_is_open_or_blocked(normal: &NormalTypeExpr) -> bool {
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

pub(super) fn sealed_domain_constructor_heads_are_disjoint(
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

pub(super) fn synthetic_proposition_module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(usize::MAX)),
        ModuleId(usize::MAX - 875),
        vec!["typeenv".to_string(), "propositions".to_string()],
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
            reason: "TASK-875 proposition predicate fallback identity".to_string(),
        },
    )
}

pub(super) fn canonical_expr_contains_var(expr: &CanonicalTypeExpr) -> bool {
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

pub(super) fn projection_rigidity_for_canonical_args(
    args: &[CanonicalTypeExpr],
) -> ProjectionRigidity {
    if args.iter().any(canonical_expr_contains_var) {
        ProjectionRigidity::Neutral
    } else {
        ProjectionRigidity::Rigid
    }
}

pub(super) fn associated_family_result_contains_var(expr: &AssociatedFamilyResultExpr) -> bool {
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

pub(super) fn projection_rigidity_for_associated_family_args(
    args: &[AssociatedFamilyResultExpr],
) -> ProjectionRigidity {
    if args.iter().any(associated_family_result_contains_var) {
        ProjectionRigidity::Neutral
    } else {
        ProjectionRigidity::Rigid
    }
}

pub(super) fn canonical_projection_base_spelling(base: &CanonicalTypeExpr) -> String {
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

pub(super) fn provenance_source_kind(
    kind: CapabilityImplementationDependencyKind,
) -> ProvenanceSourceKind {
    match kind {
        CapabilityImplementationDependencyKind::Resource => ProvenanceSourceKind::Resource,
        CapabilityImplementationDependencyKind::Capability => ProvenanceSourceKind::Capability,
        CapabilityImplementationDependencyKind::Config => ProvenanceSourceKind::Config,
    }
}

pub(super) fn classify_authority_provenance(
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

pub(super) fn implementation_authority_sources(
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

pub(super) fn looks_like_unbound_type_var_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

pub(super) fn span_anchor(span: Span, label: impl Into<String>) -> SourceAnchor {
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

pub(super) fn surface_type_contains_hole(ty: &SurfaceType) -> bool {
    surface_type_hole_count(ty) > 0
}

pub(super) fn surface_type_hole_count(ty: &SurfaceType) -> usize {
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

pub(super) fn bare_constructor_hole_hint(constructor: &str, arity: usize) -> String {
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

pub(super) fn core_visibility_from_surface(
    visibility: &SurfaceVisibility,
) -> ash_core::ast::Visibility {
    match visibility {
        SurfaceVisibility::Inherited => ash_core::ast::Visibility::Private,
        SurfaceVisibility::Public => ash_core::ast::Visibility::Public,
        SurfaceVisibility::Crate => ash_core::ast::Visibility::Crate,
        SurfaceVisibility::Super { .. }
        | SurfaceVisibility::Self_
        | SurfaceVisibility::Restricted { .. } => ash_core::ast::Visibility::Private,
    }
}

pub(super) fn constraint_for_param(param: &TypeFunctionParam) -> TypeFunctionPatternConstraint {
    param
        .domain_constraint
        .clone()
        .map(TypeFunctionPatternConstraint::Domain)
        .unwrap_or_else(|| TypeFunctionPatternConstraint::Kind(param.kind.clone()))
}

pub(super) fn associated_family_constraint_to_type_function_pattern(
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

pub(super) type CurrentTypeFunctionHead<'a> = (
    &'a str,
    &'a TypeComputationHeadId,
    &'a [TypeFunctionParam],
    &'a TypeFunctionResultConstraint,
);

pub(super) struct TypeFunctionResultLoweringContext<'a> {
    pub(super) pattern_vars: &'a HashMap<String, TypeFunctionPatternConstraint>,
    pub(super) current_head: Option<CurrentTypeFunctionHead<'a>>,
    pub(super) later_names: &'a HashSet<String>,
}

pub(super) fn result_constraint_from_pattern(
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

pub(super) fn type_function_result_from_canonical(
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

pub(super) fn canonical_type_expr_head_name(expr: &CanonicalTypeExpr) -> String {
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

pub(super) fn type_function_result_expr_head_name(expr: &TypeFunctionResultExpr) -> String {
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

pub(super) fn associated_family_result_from_canonical(
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

pub(super) fn associated_family_result_from_normal(
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

pub(super) fn associated_family_selection_blocker_to_normal_reason(
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

pub(super) fn matches_associated_family_result_constraint(
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

pub(super) fn canonical_expr_for_associated_family_constraint(
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

pub(super) fn associated_family_result_expr_to_canonical(
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

pub(super) fn hidden_imported_associated_family_heads(
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

pub(super) fn is_dependency_metadata_name(visible_name: &str) -> bool {
    visible_name.starts_with("$ash_dependency$")
}

pub(super) fn dependency_metadata_name(visible_name: &str) -> String {
    const DEPENDENCY_METADATA_PREFIX: &str = "$ash_dependency$";
    if visible_name.starts_with(DEPENDENCY_METADATA_PREFIX) {
        visible_name.to_string()
    } else {
        format!("{DEPENDENCY_METADATA_PREFIX}{visible_name}")
    }
}

pub(super) fn associated_family_result_constraint_label(
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
pub(super) fn resolve_associated_types_for_interface(
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

pub(super) fn unresolved_associated_projection_name(ty: &Type) -> Option<&str> {
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

pub(super) fn is_closed_world_nominal_impl_target(ty: &Type) -> bool {
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

pub(super) fn interface_param_kind(param: &InterfaceTypeParam) -> Kind {
    param
        .kind
        .as_ref()
        .map(|annotation| annotation.kind.clone())
        .unwrap_or(Kind::Type)
}

pub(super) fn interface_param_kinds(params: &[InterfaceTypeParam]) -> Vec<Kind> {
    params.iter().map(interface_param_kind).collect()
}

pub(super) fn render_type_constructor_head(head: &TypeConstructorHeadId) -> String {
    match head {
        TypeConstructorHeadId::Nominal { visible_name, .. } => visible_name.clone(),
        TypeConstructorHeadId::Computation(head) => head.name.clone(),
        _ => "<unsupported-type-constructor-head>".to_string(),
    }
}

pub(super) fn render_type_constructor_expr(expr: &TypeConstructorExpr) -> String {
    match expr {
        TypeConstructorExpr::ProperType(ty) => format!("{ty:?}"),
        TypeConstructorExpr::ConstructorHead(head) => render_type_constructor_head(head),
        TypeConstructorExpr::PartialApplication(app) => render_type_constructor_head(&app.head),
        _ => "<unsupported-type-constructor-expr>".to_string(),
    }
}

pub(super) fn type_contains_constructor_variable_app(ty: &Type) -> bool {
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

pub(super) fn type_contains_any_var(ty: &Type, vars: &HashSet<TypeVar>) -> bool {
    match ty {
        Type::Var(var) => vars.contains(var),
        Type::List(inner) => type_contains_any_var(inner, vars),
        Type::Record(fields) => fields.iter().any(|(_, ty)| type_contains_any_var(ty, vars)),
        Type::Fun(params, ret, _) | Type::Fn(params, ret) => {
            params.iter().any(|ty| type_contains_any_var(ty, vars))
                || type_contains_any_var(ret, vars)
        }
        Type::Constructor { args, .. } => args.iter().any(|ty| type_contains_any_var(ty, vars)),
        Type::ConstructorVariableApp { args, .. } => {
            args.iter().any(|ty| type_contains_any_var(ty, vars))
        }
        Type::Associated { base, .. } => type_contains_any_var(base, vars),
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

pub(super) fn constructor_variable_apps_are_payload_anchored(
    ty: &Type,
    method_vars: &HashSet<TypeVar>,
) -> bool {
    match ty {
        Type::ConstructorVariableApp { args, .. } => {
            args.iter()
                .any(|arg| type_contains_any_var(arg, method_vars))
                && args
                    .iter()
                    .all(|arg| constructor_variable_apps_are_payload_anchored(arg, method_vars))
        }
        Type::List(inner) => constructor_variable_apps_are_payload_anchored(inner, method_vars),
        Type::Record(fields) => fields
            .iter()
            .all(|(_, ty)| constructor_variable_apps_are_payload_anchored(ty, method_vars)),
        Type::Fun(params, ret, _) | Type::Fn(params, ret) => {
            params
                .iter()
                .all(|ty| constructor_variable_apps_are_payload_anchored(ty, method_vars))
                && constructor_variable_apps_are_payload_anchored(ret, method_vars)
        }
        Type::Constructor { args, .. } => args
            .iter()
            .all(|ty| constructor_variable_apps_are_payload_anchored(ty, method_vars)),
        Type::Associated { base, .. } => {
            constructor_variable_apps_are_payload_anchored(base, method_vars)
        }
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
        | Type::ControlLink { .. } => true,
    }
}

pub(super) fn apply_constructor_evidence_arg(
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

pub(super) fn canonical_type_expr_to_type(
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

pub(super) fn substitute_constructor_variable_apps(
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

pub(super) fn render_interface_evidence_arg(arg: &InterfaceEvidenceArg) -> String {
    match arg {
        InterfaceEvidenceArg::Proper(ty) => ty.to_string(),
        InterfaceEvidenceArg::Constructor(expr) => render_type_constructor_expr(expr),
    }
}

pub(super) fn render_interface_evidence_key(
    interface: &str,
    args: &[InterfaceEvidenceArg],
) -> String {
    let args = args
        .iter()
        .map(render_interface_evidence_arg)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{interface}<{args}>")
}

pub(super) fn interface_constraint_subject_name(subject: &SurfaceType) -> Option<&str> {
    match subject {
        SurfaceType::Name(name) => Some(name.as_ref()),
        _ => None,
    }
}

pub(super) fn interface_constraint_required_name(required: &SurfaceType) -> Option<&str> {
    match required {
        SurfaceType::Name(name) => Some(name.as_ref()),
        _ => None,
    }
}

pub(super) fn interface_evidence_args_match(
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

pub(super) fn interface_evidence_arg_matches(
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

pub(super) fn type_constructor_expr_matches_pattern(
    pattern: &TypeConstructorExpr,
    requested: &TypeConstructorExpr,
) -> bool {
    let mut bindings = HashMap::new();
    type_constructor_expr_matches_pattern_inner(pattern, requested, &mut bindings)
}

pub(super) fn type_constructor_expr_matches_pattern_inner(
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

pub(super) fn type_constructor_heads_match(
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

pub(super) fn partial_type_arg_matches_pattern(
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

pub(super) fn canonical_type_expr_matches_pattern(
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

pub(super) fn interface_evidence_arg_as_legacy_type(arg: &InterfaceEvidenceArg) -> Type {
    interface_evidence_arg_as_legacy_type_with_params(arg, &HashMap::new())
}

pub(super) fn interface_evidence_arg_as_legacy_type_with_params(
    arg: &InterfaceEvidenceArg,
    param_mapping: &HashMap<String, TypeVar>,
) -> Type {
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
            TypeConstructorExpr::PartialApplication(app) => {
                let name = match &app.head {
                    TypeConstructorHeadId::Nominal { visible_name, .. } => visible_name.clone(),
                    _ => render_type_constructor_expr(expr),
                };
                let args = app
                    .args
                    .iter()
                    .filter_map(|arg| match arg {
                        PartialTypeArg::Applied(canonical) => {
                            canonical_type_expr_to_type(canonical, param_mapping)
                        }
                        PartialTypeArg::Hole(_) => None,
                        _ => None,
                    })
                    .collect();
                Type::Constructor {
                    name: QualifiedName::root(name),
                    args,
                    kind: Kind::n_ary(1),
                }
            }
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
pub(super) fn convert_variant_fields(
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

pub(super) fn convert_variant_payload_shape(payload: &VariantPayload) -> VariantPayloadShape {
    match payload {
        VariantPayload::Unit => VariantPayloadShape::Unit,
        VariantPayload::Record(_) => VariantPayloadShape::Record,
        VariantPayload::Tuple(_) => VariantPayloadShape::Tuple,
    }
}

pub(super) fn convert_type_def(
    type_def: &TypeDef,
    type_env: &TypeEnv,
) -> Result<TypeInfo, TypeError> {
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
                    convert_variant_fields(v, &param_mapping, type_env).and_then(|fields| {
                        if let Some((field_name, _)) =
                            fields.iter().find(|(_, ty)| ty.contains_prop_kind())
                        {
                            return Err(TypeEnvError::InvalidDefinition(
                                format!(
                                    "Prop-typed values cannot escape into runtime enum variant '{}::{}' field '{}'",
                                    type_def.name, v.name, field_name
                                ),
                                Span::default(),
                            )
                            .into());
                        }

                        Ok(VariantInfo {
                            name: v.name.clone(),
                            fields,
                            payload_shape: convert_variant_payload_shape(&v.payload),
                        })
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

            let converted_fields = converted_fields?;
            if let Some((field_name, _)) = converted_fields
                .iter()
                .find(|(_, ty)| ty.contains_prop_kind())
            {
                return Err(TypeEnvError::InvalidDefinition(
                    format!(
                        "Prop-typed values cannot escape into runtime struct field '{}::{}'",
                        type_def.name, field_name
                    ),
                    Span::default(),
                )
                .into());
            }

            Ok(TypeInfo::Struct {
                name: type_def.name.clone(),
                params,
                fields: converted_fields,
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
    pub(super) algebras: &'static [PublicTowerAlgebra],
    pub(super) operations: &'static [PublicTowerOperation],
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

pub(super) const PUBLIC_TOWER_ALGEBRAS: &[PublicTowerAlgebra] = &[
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

pub(super) const fn intrinsic(
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

pub(super) const PUBLIC_TOWER_OPERATIONS: &[PublicTowerOperation] = &[
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

pub(super) static PUBLIC_TOWER_MANIFEST: PublicTowerManifest = PublicTowerManifest {
    algebras: PUBLIC_TOWER_ALGEBRAS,
    operations: PUBLIC_TOWER_OPERATIONS,
};
