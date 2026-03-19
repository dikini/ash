//! Pattern type checking for Ash (TASK-128)
//!
//! Provides type checking for patterns in match expressions, including
//! variable binding extraction and type compatibility checking.

use crate::solver::TypeError;
use crate::types::{Type, TypeVar};
use ash_parser::surface::{Literal, Pattern};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Field name used to tag variant types in record representation
const VARIANT_TAG: &str = "__variant";

/// Bindings from pattern variables to their types
pub type Bindings = HashMap<String, Type>;

/// Static empty type environment for use in recursive pattern checking.
/// This avoids creating new empty environments on each recursive call.
static EMPTY_ENV: OnceLock<TypeEnv> = OnceLock::new();

/// Get a reference to the static empty type environment.
fn empty_env() -> &'static TypeEnv {
    EMPTY_ENV.get_or_init(TypeEnv::new)
}

/// Type environment for pattern checking
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Variable types in scope
    vars: HashMap<String, Type>,
    /// Type definitions (for variant checking)
    type_defs: HashMap<String, TypeDef>,
}

/// Type definition for sum types (enums with variants)
#[derive(Debug, Clone)]
pub struct TypeDef {
    /// Name of the type
    pub name: String,
    /// Variants of the sum type
    pub variants: Vec<VariantDef>,
}

/// Variant definition
#[derive(Debug, Clone)]
pub struct VariantDef {
    /// Name of the variant
    pub name: String,
    /// Fields of the variant (name, type pairs)
    pub fields: Vec<(String, Type)>,
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
    _env: &TypeEnv,
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
            check_variant_pattern(name, fields.as_deref(), expected, bindings)
        }

        // Tuple pattern: check element count and types
        Pattern::Tuple(patterns) => check_tuple_pattern(patterns, expected, bindings),

        // Record pattern: check field names and types
        Pattern::Record(field_patterns) => check_record_pattern(field_patterns, expected, bindings),

        // List pattern: check element patterns
        Pattern::List { elements, rest } => check_list_pattern(
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
    variant_name: &str,
    field_patterns: Option<&[(Box<str>, Pattern)]>,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    // For now, we handle variant patterns by checking against Record types
    // In a full implementation, this would look up the type definition
    // and verify the variant exists with correct field types
    match expected {
        Type::Record(fields) => {
            // Check if this looks like a variant representation
            // (first field is the variant name)
            if let Some((name_field, _)) = fields.first()
                && name_field.as_ref() == VARIANT_TAG
            {
                // This is a variant type representation
                // Check field patterns against the record fields
                if let Some(field_pats) = field_patterns {
                    for (field_name, field_pattern) in field_pats {
                        let field_type = fields
                            .iter()
                            .find(|(n, _)| n.as_ref() == field_name.as_ref())
                            .map(|(_, t)| t)
                            .ok_or(TypeError::InvalidPattern {
                                message: format!("unknown field: {field_name}"),
                            })?;
                        check_pattern_inner(empty_env(), field_pattern, field_type, bindings)?;
                    }
                }
                return Ok(());
            }
            Err(TypeError::PatternMismatch {
                expected: expected.clone(),
                actual: Type::Record(vec![(Box::from(VARIANT_TAG), Type::String)]),
            })
        }
        Type::Var(_) => {
            // Type variable - accept any variant pattern
            // Bind any field patterns
            if let Some(field_pats) = field_patterns {
                for (field_name, field_pattern) in field_pats {
                    let _ = field_name;
                    let fresh_type = Type::Var(TypeVar::fresh());
                    check_pattern_inner(empty_env(), field_pattern, &fresh_type, bindings)?;
                }
            }
            Ok(())
        }
        _ => Err(TypeError::UnknownVariant(variant_name.to_string())),
    }
}

/// Check a tuple pattern against a type
fn check_tuple_pattern(
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
                check_pattern_inner(empty_env(), pattern, field_type, bindings)?;
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh types for each element
            for pattern in patterns {
                let fresh_type = Type::Var(TypeVar::fresh());
                check_pattern_inner(empty_env(), pattern, &fresh_type, bindings)?;
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
                check_pattern_inner(empty_env(), field_pattern, field_type, bindings)?;
            }
            Ok(())
        }
        Type::Var(_) => {
            // Type variable - create fresh types for each field
            for (field_name, field_pattern) in field_patterns {
                let _ = field_name;
                let fresh_type = Type::Var(TypeVar::fresh());
                check_pattern_inner(empty_env(), field_pattern, &fresh_type, bindings)?;
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
    elements: &[Pattern],
    rest: Option<&str>,
    expected: &Type,
    bindings: &mut Bindings,
) -> Result<(), TypeError> {
    match expected {
        Type::List(elem_type) => {
            for element in elements {
                check_pattern_inner(empty_env(), element, elem_type, bindings)?;
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
                check_pattern_inner(empty_env(), element, &elem_type, bindings)?;
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
        let env = TypeEnv::new();
        // Some { value: x } pattern against Option<Int> representation
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        // Option<Int> represented as record with variant info
        let option_type = Type::Record(vec![
            (Box::from(VARIANT_TAG), Type::String),
            (Box::from("value"), Type::Int),
        ]);

        let bindings = check_pattern(&env, &pattern, &option_type).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_variant_pattern_none() {
        let env = TypeEnv::new();
        // None pattern
        let pattern = Pattern::Variant {
            name: "None".into(),
            fields: None,
        };

        // Option<T> with None variant represented
        let option_type = Type::Record(vec![(Box::from(VARIANT_TAG), Type::String)]);

        let bindings = check_pattern(&env, &pattern, &option_type).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_variant_pattern_type_var() {
        let env = TypeEnv::new();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        // Against a type variable - should accept and bind field
        let type_var = Type::Var(TypeVar::fresh());
        let bindings = check_pattern(&env, &pattern, &type_var).unwrap();
        assert!(bindings.contains_key("x"));
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
        // Test: `Some { value: x }` against Option<Int> binds x: Int
        let env = TypeEnv::new();
        let pattern = Pattern::Variant {
            name: "Some".into(),
            fields: Some(vec![("value".into(), Pattern::Variable("x".into()))]),
        };

        let option_type = Type::Record(vec![
            (Box::from("__variant"), Type::String),
            (Box::from("value"), Type::Int),
        ]);

        let bindings = check_pattern(&env, &pattern, &option_type).unwrap();
        assert_eq!(bindings.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_none_against_option_no_bindings() {
        // Test: `None` against Option<T> (no bindings)
        let env = TypeEnv::new();
        let pattern = Pattern::Variant {
            name: "None".into(),
            fields: None,
        };

        let option_type = Type::Record(vec![(Box::from(VARIANT_TAG), Type::String)]);

        let bindings = check_pattern(&env, &pattern, &option_type).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_nested_pattern() {
        // Test: Some { value: (a, b) } - nested tuple in variant
        let env = TypeEnv::new();
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

        let option_type = Type::Record(vec![
            (Box::from(VARIANT_TAG), Type::String),
            (
                Box::from("value"),
                Type::Record(vec![
                    (Box::from("0"), Type::Int),
                    (Box::from("1"), Type::String),
                ]),
            ),
        ]);

        let bindings = check_pattern(&env, &pattern, &option_type).unwrap();
        assert_eq!(bindings.get("a"), Some(&Type::Int));
        assert_eq!(bindings.get("b"), Some(&Type::String));
    }
}
