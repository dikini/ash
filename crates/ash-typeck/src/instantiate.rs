//! Generic type instantiation
//!
//! Provides functionality to instantiate generic types by substituting
//! type parameters with concrete types.

use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef};
use thiserror::Error;

/// Error during type instantiation
#[derive(Debug, Clone, Error, PartialEq)]
pub enum InstantiateError {
    /// Arity mismatch: expected vs actual number of type arguments
    #[error("arity mismatch: expected {expected} type arguments, got {actual}")]
    ArityMismatch { expected: usize, actual: usize },
}

/// A substitution mapping for type instantiation
/// Maps type parameter names (from TypeDef.params) to concrete TypeExpr values
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InstantiateSubst {
    mappings: std::collections::HashMap<String, TypeExpr>,
}

impl InstantiateSubst {
    /// Create a new empty substitution
    pub fn new() -> Self {
        Self {
            mappings: std::collections::HashMap::new(),
        }
    }

    /// Create a substitution from an iterator of (param_name, type_expr) pairs
    pub fn from_pairs(pairs: impl Iterator<Item = (String, TypeExpr)>) -> Self {
        let mut mappings = std::collections::HashMap::new();
        for (name, expr) in pairs {
            mappings.insert(name, expr);
        }
        Self { mappings }
    }

    /// Insert a mapping from a type parameter name to a type expression
    pub fn insert(&mut self, name: String, expr: TypeExpr) {
        self.mappings.insert(name, expr);
    }

    /// Apply this substitution to a type expression
    pub fn apply_expr(&self, expr: &TypeExpr) -> TypeExpr {
        match expr {
            TypeExpr::Named(name) => {
                // If this name is a type parameter, substitute it
                match self.mappings.get(name) {
                    Some(substituted) => substituted.clone(),
                    None => TypeExpr::Named(name.clone()),
                }
            }
            TypeExpr::Constructor { name, args } => TypeExpr::Constructor {
                name: name.clone(),
                args: args.iter().map(|arg| self.apply_expr(arg)).collect(),
            },
            TypeExpr::Tuple(types) => {
                TypeExpr::Tuple(types.iter().map(|t| self.apply_expr(t)).collect())
            }
            TypeExpr::Record(fields) => TypeExpr::Record(
                fields
                    .iter()
                    .map(|(field_name, ty)| (field_name.clone(), self.apply_expr(ty)))
                    .collect(),
            ),
        }
    }

    /// Apply this substitution to a variant definition
    pub fn apply_variant(&self, variant: &VariantDef) -> VariantDef {
        VariantDef {
            name: variant.name.clone(),
            fields: variant
                .fields
                .iter()
                .map(|(field_name, ty)| (field_name.clone(), self.apply_expr(ty)))
                .collect(),
        }
    }

    /// Apply this substitution to a type body
    pub fn apply_body(&self, body: &TypeBody) -> TypeBody {
        match body {
            TypeBody::Enum(variants) => {
                TypeBody::Enum(variants.iter().map(|v| self.apply_variant(v)).collect())
            }
            TypeBody::Struct(fields) => TypeBody::Struct(
                fields
                    .iter()
                    .map(|(field_name, ty)| (field_name.clone(), self.apply_expr(ty)))
                    .collect(),
            ),
            TypeBody::Alias(expr) => TypeBody::Alias(self.apply_expr(expr)),
        }
    }
}

/// Instantiate a generic type definition with concrete type arguments
///
/// This function takes a type definition (like `Option<T>`) and a list of
/// concrete type arguments (like `[TypeExpr::Named("Int")]`), and returns
/// the instantiated type body with all type parameters substituted.
///
/// # Arguments
///
/// * `def` - The type definition to instantiate
/// * `args` - The concrete type expressions to substitute for type parameters
///
/// # Returns
///
/// * `Ok(TypeBody)` - The instantiated type body
/// * `Err(InstantiateError::ArityMismatch)` - If the number of arguments doesn't match params
///
/// # Example
///
/// ```
/// use ash_typeck::instantiate::instantiate;
/// use ash_core::ast::{TypeDef, TypeBody, TypeExpr, VariantDef, Visibility};
///
/// // Create a simple type definition: type Option<T> = Some { value: T } | None
/// let type_def = TypeDef {
///     name: "Option".into(),
///     params: vec!["T".to_string()],
///     body: TypeBody::Enum(vec![
///         VariantDef {
///             name: "Some".into(),
///             fields: vec![("value".into(), TypeExpr::Named("T".into()))],
///         },
///         VariantDef {
///             name: "None".into(),
///             fields: vec![],
///         },
///     ]),
///     visibility: Visibility::Public,
/// };
///
/// // Instantiate Option<Int>
/// let result = instantiate(&type_def, &[TypeExpr::Named("Int".into())]);
/// assert!(result.is_ok());
/// ```
pub fn instantiate(def: &TypeDef, args: &[TypeExpr]) -> Result<TypeBody, InstantiateError> {
    // Check arity
    if def.params.len() != args.len() {
        return Err(InstantiateError::ArityMismatch {
            expected: def.params.len(),
            actual: args.len(),
        });
    }

    // Create substitution from type parameters to concrete types
    let pairs = def.params.iter().cloned().zip(args.iter().cloned());
    let substitution = InstantiateSubst::from_pairs(pairs);

    // Apply substitution to the type body
    Ok(substitution.apply_body(&def.body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{TypeBody, TypeExpr, VariantDef, Visibility};

    // ============================================================
    // Helper functions for creating test data
    // ============================================================

    fn make_option_def() -> TypeDef {
        // type Option<T> = Some { value: T } | None
        TypeDef {
            name: "Option".into(),
            params: vec!["T".to_string()],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Some".into(),
                    fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                },
                VariantDef {
                    name: "None".into(),
                    fields: vec![],
                },
            ]),
            visibility: Visibility::Public,
        }
    }

    fn make_result_def() -> TypeDef {
        // type Result<T, E> = Ok { value: T } | Err { error: E }
        TypeDef {
            name: "Result".into(),
            params: vec!["T".to_string(), "E".to_string()],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Ok".into(),
                    fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                },
                VariantDef {
                    name: "Err".into(),
                    fields: vec![("error".into(), TypeExpr::Named("E".into()))],
                },
            ]),
            visibility: Visibility::Public,
        }
    }

    fn make_pair_def() -> TypeDef {
        // type Pair<T, U> = { first: T, second: U }
        TypeDef {
            name: "Pair".into(),
            params: vec!["T".to_string(), "U".to_string()],
            body: TypeBody::Struct(vec![
                ("first".into(), TypeExpr::Named("T".into())),
                ("second".into(), TypeExpr::Named("U".into())),
            ]),
            visibility: Visibility::Public,
        }
    }

    fn make_list_alias_def() -> TypeDef {
        // type IntList = List<Int>
        TypeDef {
            name: "IntList".into(),
            params: vec![],
            body: TypeBody::Alias(TypeExpr::Constructor {
                name: "List".into(),
                args: vec![TypeExpr::Named("Int".into())],
            }),
            visibility: Visibility::Private,
        }
    }

    fn make_identity_def() -> TypeDef {
        // type Identity<T> = T
        TypeDef {
            name: "Identity".into(),
            params: vec!["T".to_string()],
            body: TypeBody::Alias(TypeExpr::Named("T".into())),
            visibility: Visibility::Public,
        }
    }

    // ============================================================
    // Arity Mismatch Tests
    // ============================================================

    #[test]
    fn instantiate_fails_with_arity_mismatch_too_few_args() {
        let option_def = make_option_def();
        // Option expects 1 type argument, but we provide 0
        let result = instantiate(&option_def, &[]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            InstantiateError::ArityMismatch {
                expected: 1,
                actual: 0
            }
        );
    }

    #[test]
    fn instantiate_fails_with_arity_mismatch_too_many_args() {
        let option_def = make_option_def();
        // Option expects 1 type argument, but we provide 2
        let result = instantiate(
            &option_def,
            &[
                TypeExpr::Named("Int".into()),
                TypeExpr::Named("String".into()),
            ],
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            InstantiateError::ArityMismatch {
                expected: 1,
                actual: 2
            }
        );
    }

    #[test]
    fn instantiate_fails_with_arity_mismatch_multi_param() {
        let result_def = make_result_def();
        // Result<T, E> expects 2 type arguments, but we provide 1
        let result = instantiate(&result_def, &[TypeExpr::Named("Int".into())]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            InstantiateError::ArityMismatch {
                expected: 2,
                actual: 1
            }
        );
    }

    // ============================================================
    // Enum Instantiation Tests
    // ============================================================

    #[test]
    fn instantiate_enum_single_param() {
        let option_def = make_option_def();
        // Instantiate Option<Int>
        let result = instantiate(&option_def, &[TypeExpr::Named("Int".into())]);

        assert!(result.is_ok());
        let body = result.unwrap();

        // Check that we got an enum body back with correct structure
        match body {
            TypeBody::Enum(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Some");
                assert_eq!(variants[1].name, "None");

                // Check that the Some variant has the field with substituted type
                assert_eq!(variants[0].fields.len(), 1);
                assert_eq!(variants[0].fields[0].0, "value");
                assert_eq!(variants[0].fields[0].1, TypeExpr::Named("Int".into()));
            }
            _ => panic!("Expected enum body"),
        }
    }

    #[test]
    fn instantiate_enum_multi_param() {
        let result_def = make_result_def();
        // Instantiate Result<String, Error>
        let result = instantiate(
            &result_def,
            &[
                TypeExpr::Named("String".into()),
                TypeExpr::Named("Error".into()),
            ],
        );

        assert!(result.is_ok());
        let body = result.unwrap();

        match body {
            TypeBody::Enum(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Ok");
                assert_eq!(variants[1].name, "Err");

                // Check substituted types
                assert_eq!(variants[0].fields[0].1, TypeExpr::Named("String".into()));
                assert_eq!(variants[1].fields[0].1, TypeExpr::Named("Error".into()));
            }
            _ => panic!("Expected enum body"),
        }
    }

    // ============================================================
    // Struct Instantiation Tests
    // ============================================================

    #[test]
    fn instantiate_struct_multi_param() {
        let pair_def = make_pair_def();
        // Instantiate Pair<Int, String>
        let result = instantiate(
            &pair_def,
            &[
                TypeExpr::Named("Int".into()),
                TypeExpr::Named("String".into()),
            ],
        );

        assert!(result.is_ok());
        let body = result.unwrap();

        match body {
            TypeBody::Struct(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "first");
                assert_eq!(fields[1].0, "second");

                // Check substituted types
                assert_eq!(fields[0].1, TypeExpr::Named("Int".into()));
                assert_eq!(fields[1].1, TypeExpr::Named("String".into()));
            }
            _ => panic!("Expected struct body"),
        }
    }

    // ============================================================
    // Alias Instantiation Tests
    // ============================================================

    #[test]
    fn instantiate_alias_no_params() {
        let list_alias_def = make_list_alias_def();
        // Instantiate IntList (no type parameters)
        let result = instantiate(&list_alias_def, &[]);

        assert!(result.is_ok());
        let body = result.unwrap();

        match body {
            TypeBody::Alias(expr) => match expr {
                TypeExpr::Constructor { name, args } => {
                    assert_eq!(name, "List");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], TypeExpr::Named("Int".into()));
                }
                _ => panic!("Expected constructor"),
            },
            _ => panic!("Expected alias body"),
        }
    }

    #[test]
    fn instantiate_alias_identity() {
        let identity_def = make_identity_def();
        // Instantiate Identity<String> = String
        let result = instantiate(&identity_def, &[TypeExpr::Named("String".into())]);

        assert!(result.is_ok());
        let body = result.unwrap();

        match body {
            TypeBody::Alias(expr) => {
                assert_eq!(expr, TypeExpr::Named("String".into()));
            }
            _ => panic!("Expected alias body"),
        }
    }

    // ============================================================
    // Substitution Application Tests
    // ============================================================

    #[test]
    fn instantiate_applies_substitution_to_constructor_args() {
        // Create a type def with nested constructors: Container<T> = { items: List<T> }
        let container_def = TypeDef {
            name: "Container".into(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![(
                "items".into(),
                TypeExpr::Constructor {
                    name: "List".into(),
                    args: vec![TypeExpr::Named("T".into())],
                },
            )]),
            visibility: Visibility::Public,
        };

        let result = instantiate(&container_def, &[TypeExpr::Named("Int".into())]);
        assert!(result.is_ok());

        match result.unwrap() {
            TypeBody::Struct(fields) => {
                assert_eq!(fields[0].0, "items");
                match &fields[0].1 {
                    TypeExpr::Constructor { name, args } => {
                        assert_eq!(name, "List");
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], TypeExpr::Named("Int".into()));
                    }
                    _ => panic!("Expected constructor"),
                }
            }
            _ => panic!("Expected struct body"),
        }
    }

    #[test]
    fn instantiate_applies_substitution_to_tuple() {
        // Create a type def with tuple: PairWrapper<T> = { pair: (T, T) }
        let pair_wrapper_def = TypeDef {
            name: "PairWrapper".into(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![(
                "pair".into(),
                TypeExpr::Tuple(vec![
                    TypeExpr::Named("T".into()),
                    TypeExpr::Named("T".into()),
                ]),
            )]),
            visibility: Visibility::Public,
        };

        let result = instantiate(&pair_wrapper_def, &[TypeExpr::Named("String".into())]);
        assert!(result.is_ok());

        match result.unwrap() {
            TypeBody::Struct(fields) => match &fields[0].1 {
                TypeExpr::Tuple(types) => {
                    assert_eq!(types.len(), 2);
                    assert_eq!(types[0], TypeExpr::Named("String".into()));
                    assert_eq!(types[1], TypeExpr::Named("String".into()));
                }
                _ => panic!("Expected tuple"),
            },
            _ => panic!("Expected struct body"),
        }
    }

    #[test]
    fn instantiate_applies_substitution_to_record() {
        // Create a type def with record: Boxed<T> = { value: { inner: T } }
        let boxed_def = TypeDef {
            name: "Boxed".into(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![(
                "value".into(),
                TypeExpr::Record(vec![("inner".into(), TypeExpr::Named("T".into()))]),
            )]),
            visibility: Visibility::Public,
        };

        let result = instantiate(&boxed_def, &[TypeExpr::Named("Bool".into())]);
        assert!(result.is_ok());

        match result.unwrap() {
            TypeBody::Struct(fields) => match &fields[0].1 {
                TypeExpr::Record(record_fields) => {
                    assert_eq!(record_fields.len(), 1);
                    assert_eq!(record_fields[0].0, "inner");
                    assert_eq!(record_fields[0].1, TypeExpr::Named("Bool".into()));
                }
                _ => panic!("Expected record"),
            },
            _ => panic!("Expected struct body"),
        }
    }

    #[test]
    fn instantiate_preserves_non_parameter_types() {
        // Type def where some types are not parameters: Mixed<T> = { x: T, y: Int }
        let mixed_def = TypeDef {
            name: "Mixed".into(),
            params: vec!["T".to_string()],
            body: TypeBody::Struct(vec![
                ("x".into(), TypeExpr::Named("T".into())),
                ("y".into(), TypeExpr::Named("Int".into())),
            ]),
            visibility: Visibility::Public,
        };

        let result = instantiate(&mixed_def, &[TypeExpr::Named("String".into())]);
        assert!(result.is_ok());

        match result.unwrap() {
            TypeBody::Struct(fields) => {
                assert_eq!(fields[0].1, TypeExpr::Named("String".into())); // substituted
                assert_eq!(fields[1].1, TypeExpr::Named("Int".into())); // preserved
            }
            _ => panic!("Expected struct body"),
        }
    }

    // ============================================================
    // InstantiateSubst Tests
    // ============================================================

    #[test]
    fn instantiate_subst_apply_expr_named() {
        let mut subst = InstantiateSubst::new();
        subst.insert("T".to_string(), TypeExpr::Named("Int".into()));

        let result = subst.apply_expr(&TypeExpr::Named("T".into()));
        assert_eq!(result, TypeExpr::Named("Int".into()));
    }

    #[test]
    fn instantiate_subst_apply_expr_non_param() {
        let mut subst = InstantiateSubst::new();
        subst.insert("T".to_string(), TypeExpr::Named("Int".into()));

        // Non-parameter types should be preserved
        let result = subst.apply_expr(&TypeExpr::Named("String".into()));
        assert_eq!(result, TypeExpr::Named("String".into()));
    }

    #[test]
    fn instantiate_subst_from_pairs() {
        let pairs = vec![
            ("T".to_string(), TypeExpr::Named("Int".into())),
            ("U".to_string(), TypeExpr::Named("String".into())),
        ];
        let subst = InstantiateSubst::from_pairs(pairs.into_iter());

        assert_eq!(
            subst.apply_expr(&TypeExpr::Named("T".into())),
            TypeExpr::Named("Int".into())
        );
        assert_eq!(
            subst.apply_expr(&TypeExpr::Named("U".into())),
            TypeExpr::Named("String".into())
        );
    }

    // ============================================================
    // InstantiateError Display Tests
    // ============================================================

    #[test]
    fn instantiate_error_display_arity_mismatch() {
        let err = InstantiateError::ArityMismatch {
            expected: 2,
            actual: 1,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("arity mismatch"));
        assert!(msg.contains("expected 2"));
        assert!(msg.contains("got 1"));
    }
}
