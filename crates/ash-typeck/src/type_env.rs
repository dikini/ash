//! Type environment for tracking type definitions and constructor mappings
//!
//! Provides `TypeEnv` for managing type definitions and looking up constructors.

use crate::types::{Type, TypeVar};
use std::collections::HashMap;

/// Type name (e.g., "Option", "Result")
pub type TypeName = String;

/// Field name in a variant
pub type FieldName = String;

/// Index of a variant within an enum type
pub type VariantIndex = usize;

/// Definition of a variant constructor
#[derive(Debug, Clone, PartialEq)]
pub struct VariantDef {
    /// Name of the variant (e.g., "Some", "None")
    pub name: String,
    /// Fields of the variant: (field_name, field_type)
    pub fields: Vec<(FieldName, Type)>,
}

/// Definition of a type (enum or struct)
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDef {
    /// Enum type with multiple variants
    Enum {
        /// Name of the type
        name: TypeName,
        /// Type parameters (for generic types)
        params: Vec<TypeVar>,
        /// Variants of the enum
        variants: Vec<VariantDef>,
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

impl TypeDef {
    /// Get the name of the type
    pub fn name(&self) -> &str {
        match self {
            TypeDef::Enum { name, .. } => name,
            TypeDef::Struct { name, .. } => name,
        }
    }

    /// Get the type parameters
    pub fn params(&self) -> &[TypeVar] {
        match self {
            TypeDef::Enum { params, .. } => params,
            TypeDef::Struct { params, .. } => params,
        }
    }

    /// Look up a variant by name (only for enums)
    pub fn lookup_variant(&self, variant_name: &str) -> Option<(VariantIndex, &VariantDef)> {
        match self {
            TypeDef::Enum { variants, .. } => variants
                .iter()
                .enumerate()
                .find(|(_, v)| v.name == variant_name),
            TypeDef::Struct { .. } => None,
        }
    }
}

/// Type environment for tracking type definitions and constructor mappings
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Type definitions by name
    types: HashMap<TypeName, TypeDef>,
    /// Constructor mappings: constructor name -> (type name, variant index)
    constructors: HashMap<String, (TypeName, VariantIndex)>,
}

impl TypeEnv {
    /// Create a new empty type environment
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            constructors: HashMap::new(),
        }
    }

    /// Create a new type environment with builtin types registered
    pub fn with_builtin_types() -> Self {
        let mut env = Self::new();
        env.add_builtin_types();
        env
    }

    /// Register a type definition and its constructors
    pub fn register_type(&mut self, def: TypeDef) {
        let type_name = def.name().to_string();

        // Register constructors for enum variants
        if let TypeDef::Enum { variants, .. } = &def {
            for (index, variant) in variants.iter().enumerate() {
                self.constructors
                    .insert(variant.name.clone(), (type_name.clone(), index));
            }
        }

        self.types.insert(type_name, def);
    }

    /// Look up a constructor by name
    ///
    /// Returns `Some((type_name, variant_index))` if found, `None` otherwise
    pub fn lookup_constructor(&self, name: &str) -> Option<(TypeName, VariantIndex)> {
        self.constructors.get(name).cloned()
    }

    /// Look up a type definition by name
    pub fn lookup_type(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
    }

    /// Get the variant definition for a constructor
    pub fn get_variant(
        &self,
        constructor_name: &str,
    ) -> Option<(&TypeDef, VariantIndex, &VariantDef)> {
        let (type_name, variant_index) = self.lookup_constructor(constructor_name)?;
        let type_def = self.types.get(&type_name)?;

        if let TypeDef::Enum { variants, .. } = type_def {
            variants
                .get(variant_index)
                .map(|v| (type_def, variant_index, v))
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
        let t_var = TypeVar::fresh();

        let option_type = TypeDef::Enum {
            name: "Option".to_string(),
            params: vec![t_var],
            variants: vec![
                VariantDef {
                    name: "Some".to_string(),
                    fields: vec![("value".to_string(), Type::Var(t_var))],
                },
                VariantDef {
                    name: "None".to_string(),
                    fields: vec![],
                },
            ],
        };

        self.register_type(option_type);
    }

    /// Add the Result<T, E> type
    fn add_result_type(&mut self) {
        // Result<T, E> = Ok { value: T } | Err { error: E }
        let t_var = TypeVar::fresh();
        let e_var = TypeVar::fresh();

        let result_type = TypeDef::Enum {
            name: "Result".to_string(),
            params: vec![t_var, e_var],
            variants: vec![
                VariantDef {
                    name: "Ok".to_string(),
                    fields: vec![("value".to_string(), Type::Var(t_var))],
                },
                VariantDef {
                    name: "Err".to_string(),
                    fields: vec![("error".to_string(), Type::Var(e_var))],
                },
            ],
        };

        self.register_type(result_type);
    }

    /// Check if a type is registered
    pub fn has_type(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Check if a constructor is registered
    pub fn has_constructor(&self, name: &str) -> bool {
        self.constructors.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // TypeDef Tests
    // ============================================================

    #[test]
    fn test_type_def_name() {
        let enum_def = TypeDef::Enum {
            name: "Option".to_string(),
            params: vec![],
            variants: vec![],
        };
        assert_eq!(enum_def.name(), "Option");

        let struct_def = TypeDef::Struct {
            name: "Point".to_string(),
            params: vec![],
            fields: vec![],
        };
        assert_eq!(struct_def.name(), "Point");
    }

    #[test]
    fn test_type_def_lookup_variant() {
        let enum_def = TypeDef::Enum {
            name: "Option".to_string(),
            params: vec![],
            variants: vec![
                VariantDef {
                    name: "Some".to_string(),
                    fields: vec![("value".to_string(), Type::Int)],
                },
                VariantDef {
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
    fn test_struct_def_lookup_variant_returns_none() {
        let struct_def = TypeDef::Struct {
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
        assert_eq!(type_def.name(), "Option");
        assert_eq!(type_def.params().len(), 1);

        let type_def = env.lookup_type("Result").unwrap();
        assert_eq!(type_def.name(), "Result");
        assert_eq!(type_def.params().len(), 2);

        assert!(env.lookup_type("Unknown").is_none());
    }

    #[test]
    fn test_get_variant() {
        let env = TypeEnv::with_builtin_types();

        let (type_def, variant_idx, variant) = env.get_variant("Some").unwrap();
        assert_eq!(type_def.name(), "Option");
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

        let status_type = TypeDef::Enum {
            name: "Status".to_string(),
            params: vec![],
            variants: vec![
                VariantDef {
                    name: "Pending".to_string(),
                    fields: vec![],
                },
                VariantDef {
                    name: "Complete".to_string(),
                    fields: vec![("result".to_string(), Type::Int)],
                },
            ],
        };

        env.register_type(status_type);

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

        let type_def = env.lookup_type("Option").unwrap();
        match type_def {
            TypeDef::Enum {
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

        let type_def = env.lookup_type("Result").unwrap();
        match type_def {
            TypeDef::Enum {
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
}
