//! Type environment for tracking type definitions and constructor mappings
//!
//! Provides `TypeEnv` for managing type definitions and looking up constructors.

#![allow(clippy::result_large_err)]

use crate::error::TypeEnvError;
use crate::solver::TypeError;
use crate::types::{Substitution, Type, TypeVar, unify};
use crate::{Kind, QualifiedName};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload};
use ash_core::workflow_contract::{Contract as WorkflowContract, RuntimePostconditionContract};
use ash_parser::surface::{
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, ImplDef, InterfaceDef,
    InterfaceMethodSig, Type as SurfaceType,
};
use ash_parser::token::Span;
use std::collections::{HashMap, HashSet};

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
                let ret = arg_types
                    .pop()
                    .expect("Fn constructor type should include a return type");
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
            let base_ty = type_expr_to_type(base, param_mapping, type_env)?;
            Ok(Type::Associated {
                interface: String::new(), // unresolved at this level
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
    fn type_arg_count(&self) -> usize {
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

            // Try to resolve the interface from type variable bounds
            let interface = if let Type::Var(v) = &base_ty {
                if let Some(bounds) = type_env.type_var_interface_bounds.get(v) {
                    let mut candidates = Vec::new();
                    for bound_iface in bounds {
                        if let Some(iface_info) = type_env.interfaces.get(bound_iface)
                            && iface_info.associated_types.contains(&name.to_string())
                        {
                            candidates.push(bound_iface.clone());
                        }
                    }
                    if candidates.len() == 1 {
                        candidates.into_iter().next().unwrap()
                    } else if candidates.len() > 1 {
                        return Err(TypeEnvError::AmbiguousAssociatedType {
                            name: name.to_string(),
                            span: Span::default(),
                        });
                    } else {
                        String::new() // unresolved for now
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            Ok(Type::Associated {
                interface,
                base: Box::new(base_ty),
                name: name.to_string(),
            })
        }
    }
}

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
        } => {
            if iface.is_empty()
                && let Type::Var(v) = base.as_ref()
                && interface_type_params.contains(v)
            {
                *iface = interface.to_string();
            }
        }
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
    /// Registered interfaces by name.
    pub(crate) interfaces: HashMap<String, InterfaceInfo>,
    /// Registered capability interfaces by name.
    capability_interfaces: HashMap<String, CapabilityInterfaceInfo>,
    /// Registered closed-world impls.
    impls: Vec<ImplScheme>,
    /// Interface bounds attached to workflow type variables.
    pub(crate) type_var_interface_bounds: HashMap<TypeVar, HashSet<String>>,
    /// Variable bindings: variable name -> type
    variables: HashMap<String, crate::types::Type>,
    /// Lowered pure-function contracts kept at the type/runtime boundary.
    fn_contracts: HashMap<String, StoredFnContract>,
    /// Capability symbols known to be capability targets, not pure functions.
    capability_symbols: HashSet<String>,
    /// Parent environment for nested scopes (None for root)
    parent: Option<Box<TypeEnv>>,
    /// Registered capability providers (e.g., "io", "http", "db")
    providers: HashSet<String>,
    /// Workflow effect context for the three-vertex boundary (SPEC-031 §4.8).
    ///
    /// `Some(effect)` means we are type-checking inside a workflow body at the
    /// given effect level; closures (`Expr::FnDef`) are therefore typed as
    /// `Type::Fun(params, ret, effect)` rather than the pure `Type::Fn(params, ret)`.
    /// `None` means we are in a pure-fn or module-level context.
    workflow_effect: Option<ash_core::Effect>,
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
        let mut params: Vec<Type> = method
            .params
            .iter()
            .map(|ty| surface_type_to_type(ty, param_mapping, self))
            .collect::<Result<Vec<_>, _>>()?;

        for param in &mut params {
            resolve_associated_types_for_interface(
                param,
                interface_name,
                &ordered_param_names
                    .iter()
                    .map(|n| param_mapping[n])
                    .collect::<Vec<_>>(),
            );
        }

        let mut return_type = surface_type_to_type(&method.return_type, param_mapping, self)?;
        resolve_associated_types_for_interface(
            &mut return_type,
            interface_name,
            &ordered_param_names
                .iter()
                .map(|n| param_mapping[n])
                .collect::<Vec<_>>(),
        );

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
            interfaces: HashMap::with_capacity(4),
            capability_interfaces: HashMap::with_capacity(4),
            impls: Vec::new(),
            type_var_interface_bounds: HashMap::with_capacity(4),
            variables: HashMap::with_capacity(10),
            fn_contracts: HashMap::with_capacity(10),
            capability_symbols: HashSet::with_capacity(8),
            parent: None,
            providers: HashSet::new(),
            workflow_effect: None,
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
    }

    /// Check whether a `TypeDef` is a placeholder inserted by `declare_type_name`.
    ///
    /// Placeholders are identified by having no type parameters and an empty struct body.
    fn is_placeholder(def: &TypeDef) -> bool {
        def.params.is_empty() && matches!(&def.body, TypeBody::Struct(fields) if fields.is_empty())
    }

    /// Register a type definition without exposing its constructors or
    /// representation symbols.
    pub fn register_type_identity(&mut self, def: &TypeDef) -> Result<(), TypeEnvError> {
        let type_name = def.name.clone();

        if let Some(existing) = self.ast_types.get(&type_name) {
            // Allow upgrading a placeholder (empty struct with same name and no params)
            if !Self::is_placeholder(existing) {
                return Err(TypeEnvError::DuplicateType(type_name, Span::default()));
            }
            // Placeholder will be replaced below
        }

        // Convert to internal TypeInfo for type checking
        let type_info = convert_type_def(def, self).map_err(|e| {
            TypeEnvError::InvalidDefinition(format!("type '{}': {e}", def.name), Span::default())
        })?;

        self.ast_types.insert(type_name.clone(), def.clone());
        self.type_info.insert(type_name, type_info);
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

        let methods = def
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
            .collect::<Result<HashMap<_, _>, _>>()?;

        self.interfaces.insert(
            interface_name.clone(),
            InterfaceInfo {
                name: interface_name,
                type_params: def.type_params.iter().map(ToString::to_string).collect(),
                associated_types: def
                    .associated_types
                    .iter()
                    .map(|a| a.name.to_string())
                    .collect(),
                methods,
            },
        );
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
            if crate::types::unify(&scheme.head, &impl_head).is_ok() {
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
                Ok(WhereBound {
                    type_var,
                    interface: wb.bound.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let associated_type_bindings: HashMap<String, Type> = def
            .associated_type_bindings
            .iter()
            .map(|binding| {
                let ty = surface_type_to_type(&binding.ty, &param_mapping, self)?;
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
            unify(&expected_return_ty, &actual_return_ty).map_err(|_| {
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
        self.add_process_handle_type();
        self.add_proc_builtin_values();
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

    /// Check if a type is registered
    pub fn has_type(&self, name: &str) -> bool {
        self.ast_types.contains_key(name)
    }

    /// Check if a type is registered with a full (non-placeholder) definition.
    /// Returns `false` for unregistered names and for placeholder entries.
    pub fn has_full_type(&self, name: &str) -> bool {
        match self.ast_types.get(name) {
            None => false,
            Some(existing) => !Self::is_placeholder(existing),
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
            interfaces: self.interfaces.clone(),
            capability_interfaces: self.capability_interfaces.clone(),
            impls: self.impls.clone(),
            type_var_interface_bounds: self.type_var_interface_bounds.clone(),
            variables: HashMap::with_capacity(10),
            fn_contracts: self.fn_contracts.clone(),
            capability_symbols: self.capability_symbols.clone(),
            parent: Some(Box::new(self.clone())),
            providers: self.providers.clone(),
            workflow_effect: self.workflow_effect,
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
            let sub = unify(&subst.apply(expected), actual)
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
            if let Ok(scheme_subst) = crate::types::unify(&scheme.head, target_head) {
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

        // Try local types
        if let Some(info) = self.type_info.get(name) {
            return Ok((QualifiedName::root(name), Some(info)));
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
        let Some(type_def) = name
            .is_root()
            .then(|| self.ast_types.get(&name.name))
            .flatten()
            .filter(|type_def| type_def.builtin && matches!(name.name.as_str(), "Proc" | "P"))
        else {
            return Ok(());
        };

        if Self::is_placeholder(type_def) {
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
