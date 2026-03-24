//! Pattern type checking for Ash (TASK-128)
//!
//! Provides type checking for patterns in match expressions, including
//! variable binding extraction and type compatibility checking.

use crate::solver::TypeError;
use crate::types::{Type, TypeVar};
use ash_core::ast::TypeBody;
use ash_parser::surface::{Literal, Pattern};
use std::collections::HashMap;

pub use ash_core::ast::{TypeDef, VariantDef};

/// Bindings from pattern variables to their types
pub type Bindings = HashMap<String, Type>;

/// Type environment for pattern checking
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Variable types in scope
    vars: HashMap<String, Type>,
    /// Type definitions (for variant checking)
    type_defs: HashMap<String, TypeDef>,
}

impl TypeEnv {
    /// Create a new empty type environment
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            type_defs: HashMap::new(),
        }
    }

    /// Add a variable binding to the environment
    pub fn bind_var(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    /// Look up a variable's type
    pub fn lookup_var(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    /// Add a type definition
    pub fn add_type_def(&mut self, name: String, def: TypeDef) {
        self.type_defs.insert(name, def);
    }

    /// Look up a type definition
    pub fn lookup_type_def(&self, name: &str) -> Option<&TypeDef> {
        self.type_defs.get(name)
    }

    fn lookup_variant(
        &self,
        variant_name: &str,
        field_patterns: Option<&[(Box<str>, Pattern)]>,
    ) -> Result<Option<&VariantDef>, TypeError> {
        let named_matches: Vec<&VariantDef> = self
            .type_defs
            .values()
            .filter_map(|type_def| match &type_def.body {
                TypeBody::Enum(variants) => Some(variants.iter()),
                _ => None,
            })
            .flatten()
            .filter(|variant| variant.name == variant_name)
            .collect();

        match named_matches.as_slice() {
            [] => Ok(None),
            [variant] => Ok(Some(*variant)),
            _ => {
                let requested_fields: Option<Vec<&str>> = field_patterns.map(|patterns| {
                    patterns
                        .iter()
                        .map(|(field_name, _)| field_name.as_ref())
                        .collect()
                });
                let mut disambiguated = named_matches.into_iter().filter(|variant| {
                    requested_fields.as_ref().is_some_and(|requested_fields| {
                        requested_fields.iter().all(|requested| {
                            variant.fields.iter().any(|(name, _)| name == requested)
                        })
                    })
                });

                let first = disambiguated.next();
                let second = disambiguated.next();

                match (first, second) {
                    (Some(variant), None) => Ok(Some(variant)),
                    _ => Err(TypeError::InvalidPattern {
                        message: format!("ambiguous variant: {variant_name}"),
                    }),
                }
            }
        }
    }
}

/// Type check a pattern against an expected type
///
/// Returns the bindings from pattern variables to their types.
///
/// # Arguments
/// * `env` - The type environment
/// * `pattern` - The pattern to check
/// * `expected` - The expected type the pattern should match
///
/// # Returns
/// * `Ok(Bindings)` - Variable bindings from the pattern
/// * `Err(TypeError)` - Type error if pattern doesn't match
///
/// # Examples
///
/// ```
/// use ash_typeck::check_pattern::{check_pattern, TypeEnv};
/// use ash_typeck::types::Type;
/// use ash_parser::surface::Pattern;
///
/// let env = TypeEnv::new();
/// let pattern = Pattern::Variable("x".into());
/// let expected = Type::Int;
///
/// let bindings = check_pattern(&env, &pattern, &expected).unwrap();
/// assert_eq!(bindings.get("x"), Some(&Type::Int));
/// ```
pub fn check_pattern(
    env: &TypeEnv,
    pattern: &Pattern,
    expected: &Type,
) -> Result<Bindings, TypeError> {
    let mut bindings = Bindings::new();
    check_pattern_inner(env, pattern, expected, &mut bindings)?;
    Ok(bindings)
}

/// Inner recursive pattern checking function
fn check_pattern_inner(
    env: &TypeEnv,
    pattern: &Pattern,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match pattern {
        // Wildcard matches anything, no bindings
        Pattern::Wildcard => Ok(()),

        // Variable binds to the expected type
        Pattern::Variable(name) => {
            bindings.insert(name.to_string(), expected.clone());
            Ok(())
        }

        // Literal must match the literal type
        Pattern::Literal(lit) => {
            let lit_type = literal_to_type(lit);
            if types_compatible(expected, &lit_type) {
                Ok(())
            } else {
                Err(TypeError::PatternMismatch {
                    expected: expected.clone(),
                    actual: lit_type,
                })
            }
        }

        // Variant pattern: check variant exists and field patterns match
        Pattern::Variant { name, fields } => {
            check_variant_pattern(env, name, fields.as_deref(), expected, bindings)
        }

        // Tuple pattern: check element count and types
        Pattern::Tuple(patterns) => check_tuple_pattern(env, patterns, expected, bindings),

        // Record pattern: check field names and types
        Pattern::Record(field_patterns) => {
            check_record_pattern(env, field_patterns, expected, bindings)
        }

        // List pattern: check element patterns
        Pattern::List { elements, rest } => check_list_pattern(
            env,
            elements,
            rest.as_ref().map(|v| v.as_ref()),
            expected,
            bindings,
        ),
    }
}

/// Convert a literal to its type
fn literal_to_type(lit: &Literal) -> Type {
    match lit {
        Literal::Int(_) => Type::Int,
        Literal::Float(_) => Type::Var(TypeVar::fresh()), // Float not in core types
        Literal::String(_) => Type::String,
        Literal::Bool(_) => Type::Bool,
        Literal::Null => Type::Null,
        Literal::List(_) => Type::List(Box::new(Type::Var(TypeVar::fresh()))),
    }
}

/// Check if two types are compatible
fn types_compatible(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        // Same types are compatible
        (t1, t2) if t1 == t2 => true,
        // Type variables are compatible with anything
        (Type::Var(_), _) => true,
        (_, Type::Var(_)) => true,
        // Lists are compatible if elements are
        (Type::List(e1), Type::List(a1)) => types_compatible(e1, a1),
        // Otherwise not compatible
        _ => false,
    }
}

/// Check a variant pattern against a type
fn check_variant_pattern(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    if !matches!(expected, Type::Var(_)) {
        return Err(TypeError::PatternMismatch {
            expected: expected.clone(),
            actual: Type::Var(TypeVar::fresh()),
        });
    }

    if let Some(variant_def) = env.lookup_variant(variant_name, field_patterns)? {
        return check_variant_fields(
            env,
            variant_name,
            field_patterns,
            &variant_def.fields,
            bindings,
        );
    }

    match expected {
        Type::Var(_) => Err(TypeError::UnknownVariant(variant_name.to_string())),
        _ => Err(TypeError::UnknownVariant(variant_name.to_string())),
    }
}

/// Simple local conversion of TypeExpr to Type for pattern checking.
/// This handles primitive types and basic type expressions.
fn simple_type_expr_to_type(expr: &ash_core::ast::TypeExpr) -> Type {
    match expr {
        ash_core::ast::TypeExpr::Named(name) => match name.as_str() {
            "Int" => Type::Int,
            "String" => Type::String,
            "Bool" => Type::Bool,
            "Null" => Type::Null,
            "Time" => Type::Time,
            "Ref" => Type::Ref,
            _ => Type::Var(TypeVar::fresh()),
        },
        ash_core::ast::TypeExpr::Constructor { name, args } => {
            // For constructors like Option<Int>, convert arguments and build constructor
            let arg_types: Vec<Type> = args.iter().map(simple_type_expr_to_type).collect();
            Type::Constructor {
                name: crate::QualifiedName::root(name),
                args: arg_types,
                kind: crate::Kind::Type,
            }
        }
        ash_core::ast::TypeExpr::Tuple(elems) => {
            let field_types: Vec<(Box<str>, Type)> = elems
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    (
                        Box::from(format!("_{}", i).as_str()),
                        simple_type_expr_to_type(t),
                    )
                })
                .collect();
            Type::Record(field_types)
        }
        ash_core::ast::TypeExpr::Record(fields) => {
            let field_types: Vec<(Box<str>, Type)> = fields
                .iter()
                .map(|(n, t)| (Box::from(n.as_str()), simple_type_expr_to_type(t)))
                .collect();
            Type::Record(field_types)
        }
    }
}

fn check_variant_fields(
    env: &TypeEnv,
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    variant_fields: &[(String, ash_core::ast::TypeExpr)],
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match field_patterns {
        None => {
            if variant_fields.is_empty() {
                Ok(())
            } else {
                Err(TypeError::InvalidPattern {
                    message: format!("variant {variant_name} requires fields"),
                })
            }
        }
        Some(field_pats) => {
            for (field_name, field_pattern) in field_pats {
                let field_type = variant_fields
                    .iter()
                    .find(|(name, _)| name == field_name.as_ref())
                    .map(|(_, ty)| simple_type_expr_to_type(ty))
                    .ok_or_else(|| TypeError::InvalidPattern {
                        message: format!("unknown field: {field_name}"),
                    })?;
                check_pattern_inner(env, field_pattern, &field_type, bindings)?;
            }

            Ok(())
        }
    }
}

/// Check a tuple pattern against a type
fn check_tuple_pattern(
    env: &TypeEnv,
    patterns: &[Pattern],
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match expected {
        Type::Record(fields) => {
            // Tuples are represented as records with numeric field names
            if patterns.len() != fields.len() {
                return Err(TypeError::PatternArityMismatch {
                    expected: fields.len(),
                    actual: patterns.len(),
                });
            }

            for (i, pattern) in patterns.iter().enumerate() {
                let field_idx = format!("{i}");
                let field_type = fields
                    .iter()
                    .find(|(n, _)| n.as_ref() == field_idx)
                    .map(|(_, t)| t)
                    .ok_or(TypeError::PatternArityMismatch {
                        expected: fields.len(),
                        actual: patterns.len(),
                    })?;
                check_pattern_inner(env, pattern, field_type, bindings)?;
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh types for each element
            for pattern in patterns {
                let fresh_type = Type::Var(TypeVar::fresh());
                check_pattern_inner(env, pattern, &fresh_type, bindings)?;
            }
            Ok(())
        }
        _ => Err(TypeError::PatternMismatch {
            expected: expected.clone(),
            actual: Type::Record(
                patterns
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (Box::from(format!("{i}")), Type::Var(TypeVar::fresh())))
                    .collect(),
            ),
        }),
    }
}

/// Check a record pattern against a type
fn check_record_pattern(
    env: &TypeEnv,
    field_patterns: &[(Box<str>, Pattern)],
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match expected {
        Type::Record(fields) => {
            for (field_name, field_pattern) in field_patterns {
                let field_type = fields
                    .iter()
                    .find(|(n, _)| n.as_ref() == field_name.as_ref())
                    .map(|(_, t)| t)
                    .ok_or_else(|| TypeError::InvalidPattern {
                        message: format!("unknown field: {field_name}"),
                    })?;
                check_pattern_inner(env, field_pattern, field_type, bindings)?;
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh types for each field
            for (field_name, field_pattern) in field_patterns {
                let _ = field_name;
                let fresh_type = Type::Var(TypeVar::fresh());
                check_pattern_inner(env, field_pattern, &fresh_type, bindings)?;
            }
            Ok(())
        }
        _ => Err(TypeError::PatternMismatch {
            expected: expected.clone(),
            actual: Type::Record(
                field_patterns
                    .iter()
                    .map(|(n, _)| (n.clone(), Type::Var(TypeVar::fresh())))
                    .collect(),
            ),
        }),
    }
}

/// Check a list pattern against a type
fn check_list_pattern(
    env: &TypeEnv,
    elements: &[Pattern],
    rest: Option<&str>,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match expected {
        Type::List(elem_type) => {
            for element in elements {
                check_pattern_inner(env, element, elem_type, bindings)?;
            }
            if let Some(rest_name) = rest {
                // Rest binding gets the list type
                bindings.insert(rest_name.to_string(), expected.clone());
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh type for elements
            let elem_type = Type::Var(TypeVar::fresh());
            for element in elements {
                check_pattern_inner(env, element, &elem_type, bindings)?;
            }
            if let Some(rest_name) = rest {
                let list_type = Type::List(Box::new(elem_type));
                bindings.insert(rest_name.to_string(), list_type);
            }
            Ok(())
        }
        _ => Err(TypeError::PatternMismatch {
            expected: expected.clone(),
            actual: Type::List(Box::new(Type::Var(TypeVar::fresh()))),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{TypeBody, TypeExpr, Visibility};

    fn option_env() -> TypeEnv {
        let mut env = TypeEnv::new();
        env.add_type_def(
            "Option".to_string(),
            TypeDef {
                name: "Option".to_string(),
                params: vec![],
                body: TypeBody::Enum(vec![
                    VariantDef {
                        name: "Some".to_string(),
                        fields: vec![("value".to_string(), TypeExpr::Named("Int".to_string()))],
                    },
                    VariantDef {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                ]),
                visibility: Visibility::Public,
            },
        );
        env
    }

    // ============================================================
    // Wildcard Pattern Tests
    // ============================================================

    #[test]
    fn test_wildcard_matches_any_type() {
        let env = TypeEnv::new();
        let pattern = Pattern::Wildcard;

        // Wildcard should match any type with no bindings
        let bindings = check_pattern(&env, &pattern, &Type::Int).unwrap();
        assert!(bindings.is_empty());

        let bindings = check_pattern(&env, &pattern, &Type::String).unwrap();
        assert!(bindings.is_empty());

        let bindings = check_pattern(&env, &pattern, &Type::Bool).unwrap();
        assert!(bindings.is_empty());
    }

    // ============================================================
    // Variable Pattern Tests
    // ============================================================

    #[test]
    fn test_variable_binds_to_expected_type() {
        let env = TypeEnv::new();
        let pattern = Pattern::Variable("x".into());

        let bindings = check_pattern(&env, &pattern, &Type::Int).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_variable_binds_different_types() {
        let env = TypeEnv::new();

        let pattern = Pattern::Variable("s".into());
        let bindings = check_pattern(&env, &pattern, &Type::String).unwrap();
        assert_eq!(bindings.get("s"), Some(&Type::String));

        let pattern = Pattern::Variable("b".into());
        let bindings = check_pattern(&env, &pattern, &Type::Bool).unwrap();
        assert_eq!(bindings.get("b"), Some(&Type::Bool));
    }

    // ============================================================
    // Literal Pattern Tests
    // ============================================================

    #[test]
    fn test_literal_int_matches_int() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::Int(42));

        let bindings = check_pattern(&env, &pattern, &Type::Int).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_literal_int_mismatch_error() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::Int(42));

        let result = check_pattern(&env, &pattern, &Type::String);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::PatternMismatch { .. }
        ));
    }

    #[test]
    fn test_literal_string_matches_string() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::String("hello".into()));

        let bindings = check_pattern(&env, &pattern, &Type::String).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_literal_bool_matches_bool() {
        let env = TypeEnv::new();
        let pattern = Pattern::Literal(Literal::Bool(true));

        let bindings = check_pattern(&env, &pattern, &Type::Bool).unwrap();
        assert!(bindings.is_empty());
    }

    // ============================================================
    // Variant Pattern Tests
    // ============================================================

    #[test]
    fn test_variant_pattern_with_fields() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_variant_pattern_rejects_non_adt_expected_type() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        let result = check_pattern(&env, &pattern, &Type::Int);
        assert!(matches!(result, Err(TypeError::PatternMismatch { .. })));
    }

    #[test]
    fn test_variant_pattern_none() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "None".into(),
            fields: None,
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_variant_pattern_type_var() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        let type_var = Type::Var(TypeVar::fresh());
        let bindings = check_pattern(&env, &pattern, &type_var).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    // ============================================================
    // Tuple Pattern Tests
    // ============================================================

    #[test]
    fn test_tuple_pattern_matches() {
        let env = TypeEnv::new();
        // (a, b) pattern
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable("a".into()),
            Pattern::Variable("b".into()),
        ]);

        // Tuple represented as record with numeric fields
        let tuple_type = Type::Record(vec![
            (Box::from("0"), Type::Int),
            (Box::from("1"), Type::String),
        ]);

        let bindings = check_pattern(&env, &pattern, &tuple_type).unwrap();
        assert_eq!(bindings.get("a"), Some(&Type::Int));
        assert_eq!(bindings.get("b"), Some(&Type::String));
    }

    #[test]
    fn test_tuple_pattern_arity_mismatch() {
        let env = TypeEnv::new();
        // (a, b, c) pattern against 2-element tuple
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable("a".into()),
            Pattern::Variable("b".into()),
            Pattern::Variable("c".into()),
        ]);

        let tuple_type = Type::Record(vec![
            (Box::from("0"), Type::Int),
            (Box::from("1"), Type::String),
        ]);

        let result = check_pattern(&env, &pattern, &tuple_type);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::PatternArityMismatch {
                expected: 2,
                actual: 3
            }
        ));
    }

    #[test]
    fn test_tuple_pattern_type_var() {
        let env = TypeEnv::new();
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable("a".into()),
            Pattern::Variable("b".into()),
        ]);

        // Against a type variable
        let type_var = Type::Var(TypeVar::fresh());
        let bindings = check_pattern(&env, &pattern, &type_var).unwrap();
        assert!(bindings.contains_key("a"));
        assert!(bindings.contains_key("b"));
    }

    // ============================================================
    // Record Pattern Tests
    // ============================================================

    #[test]
    fn test_record_pattern_matches() {
        let env = TypeEnv::new();
        // { name: n, age: a } pattern
        let pattern = Pattern::Record(vec![
            (Box::from("name"), Pattern::Variable("n".into())),
            (Box::from("age"), Pattern::Variable("a".into())),
        ]);

        let record_type = Type::Record(vec![
            (Box::from("name"), Type::String),
            (Box::from("age"), Type::Int),
        ]);

        let bindings = check_pattern(&env, &pattern, &record_type).unwrap();
        assert_eq!(bindings.get("n"), Some(&Type::String));
        assert_eq!(bindings.get("a"), Some(&Type::Int));
    }

    #[test]
    fn test_record_pattern_unknown_field() {
        let env = TypeEnv::new();
        // { unknown: x } pattern
        let pattern = Pattern::Record(vec![(Box::from("unknown"), Pattern::Variable("x".into()))]);

        let record_type = Type::Record(vec![(Box::from("name"), Type::String)]);

        let result = check_pattern(&env, &pattern, &record_type);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::InvalidPattern { .. }
        ));
    }

    // ============================================================
    // List Pattern Tests
    // ============================================================

    #[test]
    fn test_list_pattern_matches() {
        let env = TypeEnv::new();
        // [a, b] pattern
        let pattern = Pattern::List {
            elements: vec![Pattern::Variable("a".into()), Pattern::Variable("b".into())],
            rest: None,
        };

        let list_type = Type::List(Box::new(Type::Int));

        let bindings = check_pattern(&env, &pattern, &list_type).unwrap();
        assert_eq!(bindings.get("a"), Some(&Type::Int));
        assert_eq!(bindings.get("b"), Some(&Type::Int));
    }

    #[test]
    fn test_list_pattern_with_rest() {
        let env = TypeEnv::new();
        // [first, ..rest] pattern
        let pattern = Pattern::List {
            elements: vec![Pattern::Variable("first".into())],
            rest: Some(Box::from("rest")),
        };

        let list_type = Type::List(Box::new(Type::Int));

        let bindings = check_pattern(&env, &pattern, &list_type).unwrap();
        assert_eq!(bindings.get("first"), Some(&Type::Int));
        assert_eq!(bindings.get("rest"), Some(&list_type));
    }

    #[test]
    fn test_list_pattern_mismatch() {
        let env = TypeEnv::new();
        let pattern = Pattern::List {
            elements: vec![Pattern::Variable("a".into())],
            rest: None,
        };

        let result = check_pattern(&env, &pattern, &Type::Int);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TypeError::PatternMismatch { .. }
        ));
    }

    // ============================================================
    // Integration Tests
    // ============================================================

    #[test]
    fn test_some_value_against_option_int() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_none_against_option_no_bindings() {
        let env = option_env();
        let pattern = Pattern::Variant {
            name: "None".into(),
            fields: None,
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_nested_pattern() {
        let mut env = TypeEnv::new();
        env.add_type_def(
            "OptionTuple".to_string(),
            TypeDef {
                name: "OptionTuple".to_string(),
                params: vec![],
                body: TypeBody::Enum(vec![
                    VariantDef {
                        name: "Some".to_string(),
                        fields: vec![(
                            "value".to_string(),
                            TypeExpr::Record(vec![
                                ("0".to_string(), TypeExpr::Named("Int".to_string())),
                                ("1".to_string(), TypeExpr::Named("String".to_string())),
                            ]),
                        )],
                    },
                    VariantDef {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                ]),
                visibility: Visibility::Public,
            },
        );
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![(
                "value".into(),
                Pattern::Tuple(vec![
                    Pattern::Variable("a".into()),
                    Pattern::Variable("b".into()),
                ]),
            )]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("a"), Some(&Type::Int));
        assert_eq!(bindings.get("b"), Some(&Type::String));
    }

    #[test]
    fn test_variant_pattern_uses_field_shape_to_disambiguate_constructors() {
        let mut env = option_env();
        env.add_type_def(
            "Maybe".to_string(),
            TypeDef {
                name: "Maybe".to_string(),
                params: vec![],
                body: TypeBody::Enum(vec![
                    VariantDef {
                        name: "Some".to_string(),
                        fields: vec![("other".to_string(), TypeExpr::Named("Bool".to_string()))],
                    },
                    VariantDef {
                        name: "Never".to_string(),
                        fields: vec![],
                    },
                ]),
                visibility: Visibility::Public,
            },
        );

        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }
}
