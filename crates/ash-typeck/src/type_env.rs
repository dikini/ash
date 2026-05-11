//! Type environment for tracking type definitions and constructor mappings
//!
//! Provides `TypeEnv` for managing type definitions and looking up constructors.

#![allow(clippy::result_large_err)]

use crate::error::TypeEnvError;
use crate::normalizer::{DefinitionalEqualityResult, Normalizer};
use crate::solver::TypeError;
use crate::types::{Substitution, Type, TypeVar, UnifyError, unify};
use crate::{Kind, QualifiedName};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, ConstructorPayloadKind,
    ConstructorSummary, DomainConstructorId, DomainConstructorSummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, RepresentationExposure,
    SealedDomainId, SealedDomainSummary, SourceAnchor, SourceOrigin, StructuralFieldStatus,
    SummaryVersion, TypeDeclId, TypeDeclSummary, TypeRepresentationSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, ProjectionRigidity, TypeComputationHeadId, TypeFunctionDef,
    TypeFunctionEquation, TypeFunctionParam, TypeFunctionPattern, TypeFunctionPatternConstraint,
    TypeFunctionResultConstraint, TypeFunctionResultExpr, TypeFunctionSourceAnchors,
};
use ash_core::workflow_contract::{Contract as WorkflowContract, RuntimePostconditionContract};
use ash_parser::surface::{
    CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationDependencyKind, CapabilityImplementationOperation,
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, ImplDef, InterfaceDef,
    InterfaceMethodSig, ResourceTypeDef, Type as SurfaceType, TypeFnDef as SurfaceTypeFnDef,
    TypePattern as SurfaceTypePattern, Visibility as SurfaceVisibility,
};
use ash_parser::token::Span;
use std::collections::{HashMap, HashSet};

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
            let base_ty = match base.as_ref() {
                TypeExpr::Named(base_name) if !param_mapping.contains_key(base_name) => {
                    match type_env.resolve_type(base_name) {
                        Ok(_) => type_expr_to_type(base, param_mapping, type_env)?,
                        Err(_) if looks_like_unbound_type_var_name(base_name) => {
                            return Err(TypeError::TypeEnv(TypeEnvError::InvalidDefinition(
                                format!("unresolved associated type '{name}'"),
                                Span::default(),
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
    /// Associated types declared by the interface.
    pub associated_types: Vec<String>,
    /// Methods declared by the interface.
    pub methods: HashMap<String, InterfaceMethodInfo>,
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
    pub where_bounds: Vec<WhereBound>,
    pub associated_type_bindings: HashMap<String, Type>,
    pub methods: Vec<ImplMethodInfo>,
}

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

fn surface_type_to_type(
    ty: &SurfaceType,
    param_mapping: &HashMap<String, TypeVar>,
    type_env: &TypeEnv,
) -> Result<Type, TypeEnvError> {
    match ty {
        SurfaceType::Name(name) => {
            if let Some(var) = param_mapping.get(name.as_ref()) {
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
            reason: "TASK-834 type-function lowering".to_string(),
        },
        Some(core_span),
        label,
    )
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
) -> TypeFunctionResultExpr {
    match canonical {
        CanonicalTypeExpr::Primitive(name) => TypeFunctionResultExpr::Primitive {
            name: name.clone(),
            kind: Kind::Type,
            constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
            source_anchor: span_anchor(span, format!("primitive type {name}")),
        },
        CanonicalTypeExpr::Var(name) => TypeFunctionResultExpr::Var {
            name: name.clone(),
            kind: Kind::Type,
            constraint: TypeFunctionResultConstraint::Kind(Kind::Type),
            source_anchor: span_anchor(span, format!("type variable {name}")),
        },
        CanonicalTypeExpr::NominalApp {
            origin,
            visible_name,
            args,
            kind,
        } => TypeFunctionResultExpr::NominalApp {
            origin,
            visible_name: visible_name.clone(),
            args: args
                .into_iter()
                .map(|arg| type_function_result_from_canonical(arg, span))
                .collect(),
            kind: kind.clone(),
            constraint: TypeFunctionResultConstraint::Kind(kind),
            source_anchor: span_anchor(span, format!("nominal type {visible_name}")),
        },
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args,
            kind,
            rigidity,
        } => TypeFunctionResultExpr::Projection {
            interface,
            member,
            args: args
                .into_iter()
                .map(|arg| type_function_result_from_canonical(arg, span))
                .collect(),
            kind: kind.clone(),
            constraint: TypeFunctionResultConstraint::Kind(kind),
            rigidity,
            source_anchor: span_anchor(span, "associated projection"),
        },
        CanonicalTypeExpr::ComputationHeadApp { head, args, kind } => {
            TypeFunctionResultExpr::ComputationHeadApp {
                head,
                args: args
                    .into_iter()
                    .map(|arg| type_function_result_from_canonical(arg, span))
                    .collect(),
                kind: kind.clone(),
                constraint: TypeFunctionResultConstraint::Kind(kind),
                source_anchor: span_anchor(span, "type function call"),
            }
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
        Type::Associated { .. } => false,
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
    /// Interface bounds attached to workflow type variables.
    pub(crate) type_var_interface_bounds: HashMap<TypeVar, HashSet<String>>,
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
    /// Module-local type-function names published after minimal TASK-834 lowering succeeds.
    local_type_function_heads: HashMap<String, TypeComputationHeadId>,
    /// Published checked source-backed type-function carriers keyed by computation head.
    local_type_functions: HashMap<TypeComputationHeadId, TypeFunctionDef>,
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
    if summary.version != SummaryVersion::SPEC057_ORDINARY_TYPE_V1
        && summary.version != SummaryVersion::SPEC059_SEALED_DOMAIN_V2
    {
        return Err(TypeEnvError::InvalidDefinition(
            format!(
                "unsupported module semantic summary version {}; expected {} or {}",
                summary.version.0,
                SummaryVersion::SPEC057_ORDINARY_TYPE_V1.0,
                SummaryVersion::SPEC059_SEALED_DOMAIN_V2.0
            ),
            Span::default(),
        ));
    }

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

    Ok(())
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
            type_var_interface_bounds: HashMap::with_capacity(4),
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
            local_type_function_heads: HashMap::new(),
            local_type_functions: HashMap::new(),
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
        self.type_alias_identities
            .entry(def.name.clone())
            .or_insert_with(|| fallback_canonical_type_decl_id(&def.name));
        if let Some(identity) = self.type_alias_identities.get(&def.name).cloned() {
            self.canonical_type_names
                .entry(identity)
                .or_insert_with(|| def.name.clone());
        }
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
        match self.type_alias_identities.get(&visible_name) {
            Some(existing) if existing != &summary.id => {
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
                if existing_identity.is_none()
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

    fn register_module_semantic_summary_inner(
        &mut self,
        summary: &ModuleSemanticSummary,
    ) -> Result<(), TypeEnvError> {
        validate_summary_visibility_and_duplicates(summary)?;

        for ty in &summary.exported_types {
            self.declare_summary_type_identity(ty)?;
        }

        for interface in &summary.interface_identities {
            self.register_interface_identity_summary_imported(interface)?;
        }

        for member in &summary.associated_member_identities {
            self.register_associated_member_identity_summary_imported(member)?;
        }

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

        // Pass 1: Declare all sealed-domain identities.
        for domain in &summary.exported_sealed_domains {
            self.declare_sealed_domain_identity(domain)?;
        }

        // Pass 2: Validate and store domain metadata.
        for domain in &summary.exported_sealed_domains {
            self.validate_and_register_sealed_domain(domain)?;
        }

        Ok(())
    }

    /// Register all visible ordinary type identities from a module semantic summary first,
    /// then validate/expose public representations in a second pass.
    pub fn register_module_semantic_summary(
        &mut self,
        summary: &ModuleSemanticSummary,
    ) -> Result<(), TypeEnvError> {
        let mut staged = self.clone();
        staged.register_module_semantic_summary_inner(summary)?;
        *self = staged;
        Ok(())
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

    /// Look up a published module-local type function by source name.
    #[must_use]
    pub fn lookup_local_type_function(&self, name: &str) -> Option<&TypeFunctionDef> {
        let head = self.local_type_function_heads.get(name)?;
        self.local_type_functions.get(head)
    }

    /// Look up a published module-local type function by canonical computation head.
    ///
    /// This is intentionally crate-local: TASK-838 lets the normalizer consume
    /// checked source declarations already registered in this `TypeEnv`, without
    /// adding public export/import or cross-module equation transport.
    #[must_use]
    pub(crate) fn lookup_local_type_function_by_head(
        &self,
        head: &TypeComputationHeadId,
    ) -> Option<&TypeFunctionDef> {
        self.local_type_functions.get(head)
    }

    /// Iterate published module-local type-function names. This intentionally
    /// exposes no public summary/equation transport before SPEC-F.
    pub fn local_type_function_names(&self) -> impl Iterator<Item = &str> {
        self.local_type_function_heads.keys().map(String::as_str)
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
            self.local_type_function_heads
                .insert(lowered.name.clone(), lowered.head.clone());
            self.local_type_functions
                .insert(lowered.head.clone(), lowered);
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
                        return Err(TypeEnvError::InvalidDefinition(
                            format!(
                                "public type function '{}' depends on private type function '{}'",
                                def.name, callee.name
                            ),
                            span,
                        ));
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
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' depends on private sealed domain '{}'",
                    def.name, summary.exported_name
                ),
                span,
            ));
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
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' depends on private marker constructor '{}'",
                    def.name, constructor.name
                ),
                span,
            ));
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
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' depends on private projection '{}::{}'",
                    def.name, interface.name, member.name
                ),
                span,
            ));
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
            return Err(TypeEnvError::InvalidDefinition(
                format!(
                    "public type function '{}' depends on private ordinary type '{}'",
                    def.name, visible_name
                ),
                span,
            ));
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
                .map(|canonical| type_function_result_from_canonical(canonical, span))
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
            .map(|canonical| type_function_result_from_canonical(canonical, span))
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
                ProjectionRigidity::Rigid,
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

        Ok(CanonicalTypeExpr::Projection {
            interface,
            member,
            args: projection_args,
            kind: Kind::Type,
            rigidity,
        })
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
                _ => match self.resolve_type(name) {
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
                },
            },
            TypeExpr::Constructor { name, args } => {
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

    /// Lower a surface `Type` into the Phase 110 canonical type-expression substrate.
    pub fn lower_surface_type_to_canonical(
        &self,
        ty: &SurfaceType,
    ) -> Result<CanonicalTypeExpr, TypeError> {
        match ty {
            SurfaceType::Name(name) => match name.as_ref() {
                "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref" => {
                    Ok(CanonicalTypeExpr::Primitive(name.to_string()))
                }
                _ => match self.resolve_type(name.as_ref()) {
                    Ok((qualified, _)) => {
                        self.check_type_constructor_arity(&qualified, 0)?;
                        Ok(CanonicalTypeExpr::NominalApp {
                            origin: self.canonical_type_identity_for_visible_name(name.as_ref())?,
                            visible_name: name.to_string(),
                            args: vec![],
                            kind: Kind::Type,
                        })
                    }
                    Err(TypeError::UnboundVariable(_, _)) => {
                        Ok(CanonicalTypeExpr::Var(name.to_string()))
                    }
                    Err(err) => Err(err),
                },
            },
            SurfaceType::Constructor { name, args } => {
                let (qualified, _) = self.resolve_type(name.as_ref())?;
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
                    SurfaceType::Tuple(_)
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
            Type::Associated {
                interface,
                base,
                name,
            } => {
                let base = self.type_to_canonical_expr_for_equality(base)?;
                let (canonical_interface, canonical_name) = self
                    .canonical_associated_projection_for_equality(interface, name)
                    .unwrap_or_else(|| (interface.clone(), name.clone()));
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

        let param_mapping: HashMap<String, TypeVar> = def
            .type_params
            .iter()
            .map(|param| (param.to_string(), TypeVar::fresh()))
            .collect();

        let ordered_param_names: Vec<String> =
            def.type_params.iter().map(ToString::to_string).collect();
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
                associated_types: associated_types.clone(),
                methods: HashMap::new(),
            },
        );

        let methods = match def
            .methods
            .iter()
            .map(|method| {
                self.convert_interface_method(
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
                associated_types: associated_types.clone(),
                methods: methods.clone(),
            },
        );
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
            type_var_interface_bounds: self.type_var_interface_bounds.clone(),
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
            local_type_function_heads: self.local_type_function_heads.clone(),
            local_type_functions: self.local_type_functions.clone(),
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

        let param_mapping: HashMap<String, TypeVar> = def
            .type_params
            .iter()
            .map(|param| (param.to_string(), TypeVar::fresh()))
            .collect();

        let lowered_type_args: Vec<Type> = def
            .type_args
            .iter()
            .map(|ta| surface_type_to_type(ta, &param_mapping, self))
            .collect::<Result<Vec<_>, _>>()?;

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

        let associated_type_bindings: HashMap<String, Type> = def
            .associated_type_bindings
            .iter()
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
            where_bounds: where_bounds.clone(),
            associated_type_bindings: associated_type_bindings.clone(),
            methods: vec![],
        };

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
                let param_ty = subst.apply(param_type);
                method_env.bind_variable(param_name.as_ref(), param_ty);
            }

            let expected_return_ty = self.normalize_associated_types(
                &subst.apply(&method_info.return_type),
                &temp_scheme,
                &subst,
            )?;

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
                type_params: method_info.type_params.clone(),
                params: method_info.params.iter().map(|t| subst.apply(t)).collect(),
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

        self.impls.push(ImplScheme {
            interface: interface.name,
            type_params: param_mapping.values().copied().collect(),
            head: impl_head,
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
        self.add_act_env_type();
        self.add_act_type();
        self.add_proc_type();
        self.add_workflow_type();
        self.add_process_handle_type();
        self.add_proc_builtin_values();
        self.add_workflow_builtin_values();
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

        self.register_type_identity(&result_type)
            .expect("Failed to register Result type");
        self.expose_type_representation("Result")
            .expect("Failed to expose Result constructors");
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

        self.register_type_identity(&list_type)
            .expect("Failed to register List type");
        self.expose_type_representation("List")
            .expect("Failed to expose List representation");
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

    /// Add the ActEnv type
    fn add_act_env_type(&mut self) {
        let act_env_type = TypeDef {
            name: "ActEnv".to_string(),
            params: vec![],
            body: TypeBody::Struct(vec![]),
            visibility: ash_core::ast::Visibility::Public,
            builtin: true,
        };

        self.register_type_identity(&act_env_type)
            .expect("Failed to register ActEnv type");
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
        let a = crate::types::Type::Var(crate::types::TypeVar(0));
        let b = crate::types::Type::Var(crate::types::TypeVar(1));
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
        self.type_var_interface_bounds
            .entry(var)
            .or_default()
            .insert(interface.to_string());
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
            type_var_interface_bounds: self.type_var_interface_bounds.clone(),
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
            local_type_function_heads: self.local_type_function_heads.clone(),
            local_type_functions: self.local_type_functions.clone(),
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

        // Check runtime-managed Act substrate types exist
        assert!(env.has_type("ActEnv"));
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
    fn task689d_fn_constructor_type_expr_converts_to_function_type() {
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

        let ty = type_expr_to_type(&type_expr, &HashMap::new(), &env).unwrap();
        match ty {
            Type::Fn(params, ret) => {
                assert_eq!(params.len(), 1);
                match &params[0] {
                    Type::Constructor { name, args, .. } => {
                        assert_eq!(name.display(), "ActEnv");
                        assert!(args.is_empty());
                    }
                    other => panic!("expected ActEnv parameter type, got {other:?}"),
                }
                match ret.as_ref() {
                    Type::Record(fields) => {
                        assert_eq!(fields.len(), 2);
                    }
                    other => panic!("expected tuple-lowered return record, got {other:?}"),
                }
            }
            other => panic!("expected Type::Fn, got {other:?}"),
        }
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
