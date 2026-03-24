//! Type environment for tracking type definitions and constructor mappings
//!
//! Provides `TypeEnv` for managing type definitions and looking up constructors.

use crate::types::{Type, TypeVar};
use crate::error::TypeEnvError;
use crate::solver::TypeError;
use crate::{Kind, QualifiedName};
use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef};
use std::collections::HashMap;

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
                "Null" => Ok(Type::Null),
                "Time" => Ok(Type::Time),
                "Ref" => Ok(Type::Ref),
                _ => {
                    // User-defined type with no args - look it up
                    let (qualified, _) = type_env.resolve_type(name)?;
                    Ok(Type::Constructor {
                        name: qualified,
                        args: vec![],
                        kind: Kind::Type,
                    })
                }
            }
        }

        TypeExpr::Constructor { name, args } => {
            let (qualified, _) = type_env.resolve_type(name)?;

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
                    let fields: Result<Vec<_>, _> = v
                        .fields
                        .iter()
                        .map(|(fname, ftype)| {
                            type_expr_to_type(ftype, &param_mapping, type_env)
                                .map(|ty| (fname.clone(), ty))
                        })
                        .collect();
                    fields.map(|f| VariantInfo {
                        name: v.name.clone(),
                        fields: f,
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
                    type_expr_to_type(ftype, &param_mapping, type_env)
                        .map(|ty| (fname.clone(), ty))
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
}

impl TypeEnv {
    /// Create a new empty type environment
    #[must_use]
    pub fn new() -> Self {
        Self {
            ast_types: HashMap::with_capacity(10),
            type_info: HashMap::with_capacity(10),
            constructors: HashMap::with_capacity(10),
        }
    }

    /// Create a new type environment with builtin types registered
    #[must_use]
    pub fn with_builtin_types() -> Self {
        let mut env = Self::new();
        env.add_builtin_types();
        env
    }

    /// Register a type definition and its constructors from AST TypeDef
    pub fn register_type(&mut self, def: &TypeDef) -> Result<(), TypeEnvError> {
        let type_name = def.name.clone();

        // Convert to internal TypeInfo for type checking
        let type_info = convert_type_def(def, self)
            .map_err(|e| TypeEnvError::InvalidDefinition(format!("{e}")))?;

        // Register constructors for enum variants
        if let TypeInfo::Enum { variants, .. } = &type_info {
            for (index, variant) in variants.iter().enumerate() {
                self.constructors
                    .insert(variant.name.clone(), (type_name.clone(), index));
            }
        }

        self.ast_types.insert(type_name.clone(), def.clone());
        self.type_info.insert(type_name, type_info);
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

    /// Add builtin types (Option and Result)
    pub fn add_builtin_types(&mut self) {
        self.add_option_type();
        self.add_result_type();
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
                },
                VariantDef {
                    name: "None".to_string(),
                    fields: vec![],
                },
            ]),
            visibility: ash_core::ast::Visibility::Public,
        };

        self.register_type(&option_type)
            .expect("Failed to register Option type");
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
                },
                VariantDef {
                    name: "Err".to_string(),
                    fields: vec![("error".to_string(), TypeExpr::Named("E".to_string()))],
                },
            ]),
            visibility: ash_core::ast::Visibility::Public,
        };

        self.register_type(&result_type)
            .expect("Failed to register Result type");
    }

    /// Check if a type is registered
    pub fn has_type(&self, name: &str) -> bool {
        self.ast_types.contains_key(name)
    }

    /// Check if a constructor is registered
    pub fn has_constructor(&self, name: &str) -> bool {
        self.constructors.contains_key(name)
    }

    /// Resolve a type name to its qualified form and info
    pub fn resolve_type(&self, name: &str) -> Result<(QualifiedName, Option<&TypeInfo>), TypeError> {
        // Try as primitive first
        match name {
            "Int" | "String" | "Bool" | "Null" | "Time" | "Ref" => {
                return Ok((QualifiedName::root(name), None));
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

        Err(TypeError::UnboundVariable(name.to_string()))
    }
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
                },
                VariantInfo {
                    name: "None".to_string(),
                    fields: vec![],
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
                },
                VariantDef {
                    name: "Complete".to_string(),
                    fields: vec![("result".to_string(), TypeExpr::Named("Int".to_string()))],
                },
            ]),
            visibility: Visibility::Public,
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
}
