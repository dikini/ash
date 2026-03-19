//! Type system for Ash
//!
//! Defines the type representation, type variables, substitutions,
//! and unification algorithm for the Ash type checker.

use ash_core::{Effect, Value};
use std::collections::HashMap;
use thiserror::Error;

/// Types in the Ash type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type
    Int,
    /// String type
    String,
    /// Boolean type
    Bool,
    /// Null type
    Null,
    /// Time type
    Time,
    /// Reference type
    Ref,
    /// List of elements
    List(Box<Type>),
    /// Record with named fields
    Record(Vec<(Box<str>, Type)>),
    /// Capability with name and effect
    Cap { name: Box<str>, effect: Effect },
    /// Function type: arguments, return type, effect
    Fun(Vec<Type>, Box<Type>, Effect),
    /// Type variable
    Var(TypeVar),

    /// Instance type (composite of addr + control link)
    Instance { workflow_type: Box<str> },

    /// Opaque instance address
    InstanceAddr { workflow_type: Box<str> },

    /// Control link (affine - must be used exactly once)
    ControlLink { workflow_type: Box<str> },
}

/// Type variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

impl TypeVar {
    /// Generate a fresh type variable with a unique ID
    pub fn fresh() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        TypeVar(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Type substitution: maps type variables to types
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Substitution {
    mappings: HashMap<TypeVar, Type>,
}

impl Substitution {
    /// Create a new empty substitution
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Create a substitution from an iterator of (TypeVar, Type) pairs
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::types::{Substitution, Type, TypeVar};
    ///
    /// let pairs = vec![
    ///     (TypeVar(0), &Type::Int),
    ///     (TypeVar(1), &Type::String),
    /// ];
    /// let subst = Substitution::from_pairs(pairs.into_iter());
    ///
    /// assert_eq!(subst.apply(&Type::Var(TypeVar(0))), Type::Int);
    /// assert_eq!(subst.apply(&Type::Var(TypeVar(1))), Type::String);
    /// ```
    pub fn from_pairs<'a>(pairs: impl Iterator<Item = (TypeVar, &'a Type)>) -> Self {
        let mut mappings = HashMap::new();
        for (var, ty) in pairs {
            mappings.insert(var, ty.clone());
        }
        Self { mappings }
    }

    /// Insert a mapping from a type variable to a type
    pub fn insert(&mut self, var: TypeVar, ty: Type) {
        self.mappings.insert(var, ty);
    }

    /// Look up a type variable in the substitution
    pub fn get(&self, var: TypeVar) -> Option<&Type> {
        self.mappings.get(&var)
    }

    /// Apply this substitution to a type, recursively substituting type variables
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(var) => {
                // If this variable has a substitution, apply it recursively
                match self.get(*var) {
                    Some(substituted) => self.apply(substituted),
                    None => Type::Var(*var),
                }
            }
            Type::List(elem) => Type::List(Box::new(self.apply(elem))),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.apply(ty)))
                    .collect(),
            ),
            Type::Cap { name, effect } => Type::Cap {
                name: name.clone(),
                effect: *effect,
            },
            Type::Fun(args, ret, effect) => Type::Fun(
                args.iter().map(|a| self.apply(a)).collect(),
                Box::new(self.apply(ret)),
                *effect,
            ),
            // Primitives have no variables to substitute
            _ => ty.clone(),
        }
    }

    /// Compose two substitutions: self ∘ other
    /// The result applies `other` first, then `self`
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = Substitution::new();

        // First, add all mappings from other (with self applied to the values)
        for (var, ty) in &other.mappings {
            result.insert(*var, self.apply(ty));
        }

        // Then, add mappings from self that don't have keys in other
        for (var, ty) in &self.mappings {
            if !other.mappings.contains_key(var) {
                result.insert(*var, ty.clone());
            }
        }

        result
    }
}

/// Schema validation error
#[derive(Debug, Clone, Error, PartialEq)]
pub enum SchemaError {
    /// Type mismatch
    #[error("type mismatch: expected {expected}, got {actual}")]
    Mismatch { expected: String, actual: String },

    /// Missing field in record
    #[error("missing field '{field}' in record")]
    MissingField { field: String },

    /// Field type mismatch
    #[error("field '{field}' type mismatch: expected {expected}, got {actual}")]
    FieldMismatch {
        field: String,
        expected: String,
        actual: String,
    },
}

/// Unification error
#[derive(Debug, Clone, thiserror::Error)]
pub enum UnifyError {
    /// Types cannot be unified
    #[error("Cannot unify {0:?} with {1:?}")]
    Mismatch(Type, Type),
    /// Infinite type detected (occurs check failed)
    #[error("Infinite type {0:?} in {1:?}")]
    InfiniteType(TypeVar, Type),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::String => write!(f, "String"),
            Type::Bool => write!(f, "Bool"),
            Type::Null => write!(f, "Null"),
            Type::Time => write!(f, "Time"),
            Type::Ref => write!(f, "Ref"),
            Type::List(elem) => write!(f, "List<{}>", elem),
            Type::Record(fields) => {
                write!(f, "{{")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, "}}")
            }
            Type::Cap { name, effect } => write!(f, "Cap<{}: {:?}>", name, effect),
            Type::Fun(args, ret, effect) => {
                write!(f, "fn(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> {} [{:?}]", ret, effect)
            }
            Type::Var(v) => write!(f, "Var<{}>", v.0),
            Type::Instance { workflow_type } => write!(f, "Instance<{}>", workflow_type),
            Type::InstanceAddr { workflow_type } => write!(f, "InstanceAddr<{}>", workflow_type),
            Type::ControlLink { workflow_type } => write!(f, "ControlLink<{}>", workflow_type),
        }
    }
}

impl Type {
    /// Check if a value matches this type schema
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (Type::Int, Value::Int(_)) => true,
            (Type::String, Value::String(_)) => true,
            (Type::Bool, Value::Bool(_)) => true,
            (Type::Null, Value::Null) => true,
            (Type::Time, Value::Time(_)) => true,
            (Type::Ref, Value::Ref(_)) => true,
            (Type::Cap { name, .. }, Value::Cap(v)) => name.as_ref() == v.as_str(),
            // Type variables match anything during inference
            (Type::Var(_), _) => true,

            (Type::List(elem_type), Value::List(items)) => {
                items.iter().all(|item| elem_type.matches(item))
            }

            (Type::Record(fields), Value::Record(record)) => fields.iter().all(|(name, ty)| {
                record
                    .get(name.as_ref())
                    .map(|val| ty.matches(val))
                    .unwrap_or(false)
            }),

            _ => false,
        }
    }

    /// Detailed validation with error message
    pub fn validate(&self, value: &Value) -> Result<(), SchemaError> {
        if self.matches(value) {
            Ok(())
        } else {
            Err(SchemaError::Mismatch {
                expected: self.to_string(),
                actual: value.to_string(),
            })
        }
    }
}

/// Unify two types, returning a substitution that makes them equal
pub fn unify(t1: &Type, t2: &Type) -> Result<Substitution, UnifyError> {
    // Apply any existing substitutions to normalize before unifying
    match (t1, t2) {
        // Same primitives unify with empty substitution
        (Type::Int, Type::Int)
        | (Type::String, Type::String)
        | (Type::Bool, Type::Bool)
        | (Type::Null, Type::Null)
        | (Type::Time, Type::Time)
        | (Type::Ref, Type::Ref) => Ok(Substitution::new()),

        // Type variable unification
        (Type::Var(v), ty) | (ty, Type::Var(v)) => {
            // Occurs check: prevent infinite types
            if occurs_in(*v, ty) {
                return Err(UnifyError::InfiniteType(*v, ty.clone()));
            }
            // Create a substitution mapping the variable to the type
            let mut sub = Substitution::new();
            sub.insert(*v, ty.clone());
            Ok(sub)
        }

        // List unification: elements must unify
        (Type::List(e1), Type::List(e2)) => unify(e1, e2),

        // Record unification: fields must match by name and type
        (Type::Record(f1), Type::Record(f2)) => {
            if f1.len() != f2.len() {
                return Err(UnifyError::Mismatch(t1.clone(), t2.clone()));
            }

            // Sort fields by name to ensure consistent matching
            let mut fields1: Vec<_> = f1.iter().collect();
            let mut fields2: Vec<_> = f2.iter().collect();
            fields1.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));
            fields2.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));

            let mut acc_sub = Substitution::new();

            for ((name1, ty1), (name2, ty2)) in fields1.iter().zip(fields2.iter()) {
                if name1 != name2 {
                    return Err(UnifyError::Mismatch(t1.clone(), t2.clone()));
                }
                // Unify the field types, applying accumulated substitution
                let sub = unify(&acc_sub.apply(ty1), &acc_sub.apply(ty2))?;
                acc_sub = acc_sub.compose(&sub);
            }

            Ok(acc_sub)
        }

        // Capability unification: names and effects must match
        (
            Type::Cap {
                name: n1,
                effect: e1,
            },
            Type::Cap {
                name: n2,
                effect: e2,
            },
        ) => {
            if n1 == n2 && e1 == e2 {
                Ok(Substitution::new())
            } else {
                Err(UnifyError::Mismatch(t1.clone(), t2.clone()))
            }
        }

        // Function unification: args, return, and effect must match
        (Type::Fun(a1, r1, e1), Type::Fun(a2, r2, e2)) => {
            if a1.len() != a2.len() {
                return Err(UnifyError::Mismatch(t1.clone(), t2.clone()));
            }

            // Effects must match exactly
            if e1 != e2 {
                return Err(UnifyError::Mismatch(t1.clone(), t2.clone()));
            }

            let mut acc_sub = Substitution::new();

            // Unify argument types
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                let sub = unify(&acc_sub.apply(arg1), &acc_sub.apply(arg2))?;
                acc_sub = acc_sub.compose(&sub);
            }

            // Unify return type
            let sub = unify(&acc_sub.apply(r1), &acc_sub.apply(r2))?;
            acc_sub = acc_sub.compose(&sub);

            Ok(acc_sub)
        }

        // Different constructors cannot unify
        _ => Err(UnifyError::Mismatch(t1.clone(), t2.clone())),
    }
}

/// Check if a type variable occurs in a type (occurs check)
pub fn occurs_in(var: TypeVar, ty: &Type) -> bool {
    match ty {
        Type::Var(v) => *v == var,
        Type::List(elem) => occurs_in(var, elem),
        Type::Record(fields) => fields.iter().any(|(_, ty)| occurs_in(var, ty)),
        Type::Fun(args, ret, _) => args.iter().any(|a| occurs_in(var, a)) || occurs_in(var, ret),
        Type::Cap { .. }
        | Type::Int
        | Type::String
        | Type::Bool
        | Type::Null
        | Type::Time
        | Type::Ref
        | Type::Instance { .. }
        | Type::InstanceAddr { .. }
        | Type::ControlLink { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // TypeVar Tests
    // ============================================================

    #[test]
    fn typevar_fresh_generates_unique_vars() {
        let v1 = TypeVar::fresh();
        let v2 = TypeVar::fresh();
        let v3 = TypeVar::fresh();

        assert_ne!(v1, v2, "Fresh type variables should be unique");
        assert_ne!(v2, v3, "Fresh type variables should be unique");
        assert_ne!(v1, v3, "Fresh type variables should be unique");
    }

    #[test]
    fn typevar_fresh_increments_counter() {
        // Reset the counter by creating fresh vars and checking order
        let start = TypeVar::fresh();
        let next = TypeVar::fresh();

        assert_eq!(next.0, start.0 + 1, "Fresh type variables should increment");
    }

    // ============================================================
    // Substitution::apply Tests
    // ============================================================

    #[test]
    fn substitution_apply_primitives() {
        let sub = Substitution::new();

        assert_eq!(sub.apply(&Type::Int), Type::Int);
        assert_eq!(sub.apply(&Type::String), Type::String);
        assert_eq!(sub.apply(&Type::Bool), Type::Bool);
        assert_eq!(sub.apply(&Type::Null), Type::Null);
        assert_eq!(sub.apply(&Type::Time), Type::Time);
        assert_eq!(sub.apply(&Type::Ref), Type::Ref);
    }

    #[test]
    fn substitution_apply_substitutes_vars() {
        let mut sub = Substitution::new();
        let v1 = TypeVar(1);
        sub.insert(v1, Type::Int);

        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
    }

    #[test]
    fn substitution_apply_unbound_var_unchanged() {
        let sub = Substitution::new();
        let v1 = TypeVar(1);

        assert_eq!(sub.apply(&Type::Var(v1)), Type::Var(v1));
    }

    #[test]
    fn substitution_apply_nested_substitution() {
        let mut sub = Substitution::new();
        let v1 = TypeVar(1);
        let v2 = TypeVar(2);
        sub.insert(v1, Type::Var(v2));
        sub.insert(v2, Type::Int);

        // v1 -> v2 -> Int, so applying should resolve to Int
        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
    }

    #[test]
    fn substitution_apply_list() {
        let mut sub = Substitution::new();
        let v1 = TypeVar(1);
        sub.insert(v1, Type::Int);

        let list = Type::List(Box::new(Type::Var(v1)));
        let result = sub.apply(&list);

        assert_eq!(result, Type::List(Box::new(Type::Int)));
    }

    #[test]
    fn substitution_apply_record() {
        let mut sub = Substitution::new();
        let v1 = TypeVar(1);
        sub.insert(v1, Type::String);

        let record = Type::Record(vec![
            (Box::from("name"), Type::Var(v1)),
            (Box::from("age"), Type::Int),
        ]);
        let result = sub.apply(&record);

        assert_eq!(
            result,
            Type::Record(vec![
                (Box::from("name"), Type::String),
                (Box::from("age"), Type::Int),
            ])
        );
    }

    #[test]
    fn substitution_apply_function() {
        let mut sub = Substitution::new();
        let v1 = TypeVar(1);
        sub.insert(v1, Type::Int);

        let func = Type::Fun(
            vec![Type::Var(v1)],
            Box::new(Type::Var(v1)),
            Effect::Epistemic,
        );
        let result = sub.apply(&func);

        assert_eq!(
            result,
            Type::Fun(vec![Type::Int], Box::new(Type::Int), Effect::Epistemic)
        );
    }

    // ============================================================
    // Substitution::compose Tests
    // ============================================================

    #[test]
    fn substitution_compose_combines_mappings() {
        let mut sub1 = Substitution::new();
        let v1 = TypeVar(1);
        sub1.insert(v1, Type::Int);

        let mut sub2 = Substitution::new();
        let v2 = TypeVar(2);
        sub2.insert(v2, Type::String);

        let composed = sub1.compose(&sub2);

        assert_eq!(composed.apply(&Type::Var(v1)), Type::Int);
        assert_eq!(composed.apply(&Type::Var(v2)), Type::String);
    }

    #[test]
    fn substitution_compose_self_applied_to_other() {
        // Test that compose applies self to other's values
        let mut sub1 = Substitution::new();
        let v1 = TypeVar(1);
        let v2 = TypeVar(2);
        sub1.insert(v1, Type::Int);

        let mut sub2 = Substitution::new();
        sub2.insert(v2, Type::Var(v1)); // v2 -> v1

        let composed = sub1.compose(&sub2);

        // v2 -> v1 -> Int, so v2 should resolve to Int
        assert_eq!(composed.apply(&Type::Var(v2)), Type::Int);
    }

    #[test]
    fn substitution_compose_other_overrides_self() {
        // When both have the same key, other takes precedence
        let mut sub1 = Substitution::new();
        let v1 = TypeVar(1);
        sub1.insert(v1, Type::String);

        let mut sub2 = Substitution::new();
        sub2.insert(v1, Type::Int);

        let composed = sub1.compose(&sub2);

        // v1 should map to Int (from sub2)
        assert_eq!(composed.apply(&Type::Var(v1)), Type::Int);
    }

    // ============================================================
    // Unify Primitive Tests
    // ============================================================

    #[test]
    fn unify_same_primitives_succeeds() {
        let primitives = [
            Type::Int,
            Type::String,
            Type::Bool,
            Type::Null,
            Type::Time,
            Type::Ref,
        ];

        for p in &primitives {
            let result = unify(p, p);
            assert!(
                result.is_ok(),
                "Unifying same primitive should succeed: {:?}",
                p
            );
            assert_eq!(result.unwrap(), Substitution::new());
        }
    }

    #[test]
    fn unify_different_primitives_fails() {
        let primitives = [
            Type::Int,
            Type::String,
            Type::Bool,
            Type::Null,
            Type::Time,
            Type::Ref,
        ];

        for i in 0..primitives.len() {
            for j in i + 1..primitives.len() {
                let result = unify(&primitives[i], &primitives[j]);
                assert!(
                    result.is_err(),
                    "Unifying different primitives should fail: {:?} vs {:?}",
                    primitives[i],
                    primitives[j]
                );
            }
        }
    }

    // ============================================================
    // Unify Variable Tests
    // ============================================================

    #[test]
    fn unify_var_with_type_succeeds() {
        let v1 = TypeVar(1);
        let result = unify(&Type::Var(v1), &Type::Int);

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
    }

    #[test]
    fn unify_type_with_var_succeeds() {
        let v1 = TypeVar(1);
        let result = unify(&Type::Int, &Type::Var(v1));

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
    }

    #[test]
    fn unify_var_with_var_succeeds() {
        let v1 = TypeVar(1);
        let v2 = TypeVar(2);
        let result = unify(&Type::Var(v1), &Type::Var(v2));

        assert!(result.is_ok());
        let sub = result.unwrap();
        // Result should map v1 to v2 (or v2 to v1)
        let applied = sub.apply(&Type::Var(v1));
        assert!(
            applied == Type::Var(v1) || applied == Type::Var(v2),
            "Var should unify to something"
        );
    }

    // ============================================================
    // Occurs Check Tests
    // ============================================================

    #[test]
    fn occurs_in_detects_var_in_type() {
        let v1 = TypeVar(1);

        assert!(occurs_in(v1, &Type::Var(v1)));
        assert!(occurs_in(v1, &Type::List(Box::new(Type::Var(v1)))));
        assert!(occurs_in(
            v1,
            &Type::Record(vec![(Box::from("x"), Type::Var(v1))])
        ));
        assert!(occurs_in(
            v1,
            &Type::Fun(vec![Type::Var(v1)], Box::new(Type::Int), Effect::Epistemic)
        ));
    }

    #[test]
    fn occurs_in_not_in_unrelated() {
        let v1 = TypeVar(1);
        let v2 = TypeVar(2);

        assert!(!occurs_in(v1, &Type::Int));
        assert!(!occurs_in(v1, &Type::Var(v2)));
        assert!(!occurs_in(v1, &Type::List(Box::new(Type::Int))));
    }

    #[test]
    fn unify_occurs_check_prevents_infinite_types() {
        let v1 = TypeVar(1);
        let list_var = Type::List(Box::new(Type::Var(v1)));

        let result = unify(&Type::Var(v1), &list_var);

        assert!(result.is_err(), "Unification should fail with occurs check");
        assert!(matches!(
            result.unwrap_err(),
            UnifyError::InfiniteType(_, _)
        ));
    }

    #[test]
    fn unify_occurs_check_in_function() {
        let v1 = TypeVar(1);
        // Trying to unify v1 with a function that takes v1 as argument
        let func = Type::Fun(vec![Type::Var(v1)], Box::new(Type::Int), Effect::Epistemic);

        let result = unify(&Type::Var(v1), &func);

        assert!(result.is_err(), "Unification should fail with occurs check");
    }

    // ============================================================
    // Unify List Tests
    // ============================================================

    #[test]
    fn unify_lists_with_compatible_elements() {
        let list1 = Type::List(Box::new(Type::Int));
        let list2 = Type::List(Box::new(Type::Int));

        let result = unify(&list1, &list2);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Substitution::new());
    }

    #[test]
    fn unify_lists_with_unifiable_elements() {
        let v1 = TypeVar(1);
        let list1 = Type::List(Box::new(Type::Var(v1)));
        let list2 = Type::List(Box::new(Type::Int));

        let result = unify(&list1, &list2);

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
    }

    #[test]
    fn unify_lists_with_incompatible_elements_fails() {
        let list1 = Type::List(Box::new(Type::Int));
        let list2 = Type::List(Box::new(Type::String));

        let result = unify(&list1, &list2);

        assert!(result.is_err());
    }

    // ============================================================
    // Unify Record Tests
    // ============================================================

    #[test]
    fn unify_same_records_succeeds() {
        let record = Type::Record(vec![
            (Box::from("name"), Type::String),
            (Box::from("age"), Type::Int),
        ]);

        let result = unify(&record, &record);
        assert!(result.is_ok());
    }

    #[test]
    fn unify_records_with_unifiable_fields() {
        let v1 = TypeVar(1);
        let record1 = Type::Record(vec![
            (Box::from("name"), Type::Var(v1)),
            (Box::from("age"), Type::Int),
        ]);
        let record2 = Type::Record(vec![
            (Box::from("name"), Type::String),
            (Box::from("age"), Type::Int),
        ]);

        let result = unify(&record1, &record2);

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.apply(&Type::Var(v1)), Type::String);
    }

    #[test]
    fn unify_records_different_field_names_fails() {
        let record1 = Type::Record(vec![(Box::from("x"), Type::Int)]);
        let record2 = Type::Record(vec![(Box::from("y"), Type::Int)]);

        let result = unify(&record1, &record2);

        assert!(result.is_err());
    }

    #[test]
    fn unify_records_different_field_count_fails() {
        let record1 = Type::Record(vec![(Box::from("x"), Type::Int)]);
        let record2 = Type::Record(vec![
            (Box::from("x"), Type::Int),
            (Box::from("y"), Type::String),
        ]);

        let result = unify(&record1, &record2);

        assert!(result.is_err());
    }

    // ============================================================
    // Unify Function Tests
    // ============================================================

    #[test]
    fn unify_same_functions_succeeds() {
        let func = Type::Fun(
            vec![Type::Int, Type::String],
            Box::new(Type::Bool),
            Effect::Epistemic,
        );

        let result = unify(&func, &func);
        assert!(result.is_ok());
    }

    #[test]
    fn unify_functions_with_unifiable_args() {
        let v1 = TypeVar(1);
        let func1 = Type::Fun(
            vec![Type::Var(v1), Type::String],
            Box::new(Type::Bool),
            Effect::Epistemic,
        );
        let func2 = Type::Fun(
            vec![Type::Int, Type::String],
            Box::new(Type::Bool),
            Effect::Epistemic,
        );

        let result = unify(&func1, &func2);

        assert!(result.is_ok());
        let sub = result.unwrap();
        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
    }

    #[test]
    fn unify_functions_different_arg_count_fails() {
        let func1 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Epistemic);
        let func2 = Type::Fun(
            vec![Type::Int, Type::String],
            Box::new(Type::Bool),
            Effect::Epistemic,
        );

        let result = unify(&func1, &func2);

        assert!(result.is_err());
    }

    #[test]
    fn unify_functions_different_effect_fails() {
        let func1 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Epistemic);
        let func2 = Type::Fun(vec![Type::Int], Box::new(Type::Bool), Effect::Operational);

        let result = unify(&func1, &func2);

        assert!(result.is_err());
    }

    // ============================================================
    // Unify Capability Tests
    // ============================================================

    #[test]
    fn unify_same_capabilities_succeeds() {
        let cap = Type::Cap {
            name: Box::from("FileIO"),
            effect: Effect::Operational,
        };

        let result = unify(&cap, &cap);
        assert!(result.is_ok());
    }

    #[test]
    fn unify_capabilities_different_name_fails() {
        let cap1 = Type::Cap {
            name: Box::from("FileIO"),
            effect: Effect::Operational,
        };
        let cap2 = Type::Cap {
            name: Box::from("Network"),
            effect: Effect::Operational,
        };

        let result = unify(&cap1, &cap2);

        assert!(result.is_err());
    }

    #[test]
    fn unify_capabilities_different_effect_fails() {
        let cap1 = Type::Cap {
            name: Box::from("FileIO"),
            effect: Effect::Epistemic,
        };
        let cap2 = Type::Cap {
            name: Box::from("FileIO"),
            effect: Effect::Operational,
        };

        let result = unify(&cap1, &cap2);

        assert!(result.is_err());
    }

    // ============================================================
    // Schema Validation Tests (TASK-097)
    // ============================================================

    #[test]
    fn test_int_matches() {
        use ash_core::Value;

        assert!(Type::Int.matches(&Value::Int(42)));
        assert!(!Type::Int.matches(&Value::String("42".into())));
    }

    #[test]
    fn test_string_matches() {
        use ash_core::Value;

        assert!(Type::String.matches(&Value::String("hello".into())));
        assert!(!Type::String.matches(&Value::Int(42)));
    }

    #[test]
    fn test_list_matches() {
        use ash_core::Value;

        let schema = Type::List(Box::new(Type::Int));
        assert!(schema.matches(&Value::List(vec![Value::Int(1), Value::Int(2)])));
        assert!(!schema.matches(&Value::List(vec![Value::String("a".into())])));
    }

    #[test]
    fn test_record_matches() {
        use ash_core::Value;
        use std::collections::HashMap;

        let schema = Type::Record(vec![
            (Box::from("name"), Type::String),
            (Box::from("age"), Type::Int),
        ]);

        let valid = Value::Record(HashMap::from([
            ("name".to_string(), Value::String("Alice".into())),
            ("age".to_string(), Value::Int(30)),
        ]));
        assert!(schema.matches(&valid));

        let invalid = Value::Record(HashMap::from([
            ("name".to_string(), Value::Int(30)), // Wrong type
            ("age".to_string(), Value::Int(30)),
        ]));
        assert!(!schema.matches(&invalid));
    }

    #[test]
    fn test_nested_record() {
        use ash_core::Value;
        use std::collections::HashMap;

        let schema = Type::Record(vec![(
            Box::from("point"),
            Type::Record(vec![
                (Box::from("x"), Type::Int),
                (Box::from("y"), Type::Int),
            ]),
        )]);

        let value = Value::Record(HashMap::from([(
            "point".to_string(),
            Value::Record(HashMap::from([
                ("x".to_string(), Value::Int(1)),
                ("y".to_string(), Value::Int(2)),
            ])),
        )]));
        assert!(schema.matches(&value));
    }

    #[test]
    fn test_validate_success() {
        use ash_core::Value;

        let result = Type::Int.validate(&Value::Int(42));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_failure() {
        use ash_core::Value;

        let result = Type::Int.validate(&Value::String("hello".into()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SchemaError::Mismatch { .. }));
    }

    #[test]
    fn test_display_type() {
        assert_eq!(Type::Int.to_string(), "Int");
        assert_eq!(Type::String.to_string(), "String");
        assert_eq!(Type::Bool.to_string(), "Bool");
        assert_eq!(Type::Null.to_string(), "Null");
        assert_eq!(Type::Time.to_string(), "Time");
        assert_eq!(Type::Ref.to_string(), "Ref");
        assert_eq!(Type::List(Box::new(Type::Int)).to_string(), "List<Int>");
        assert_eq!(
            Type::Record(vec![
                (Box::from("x"), Type::Int),
                (Box::from("y"), Type::String),
            ])
            .to_string(),
            "{x: Int, y: String}"
        );
    }

    // ============================================================
    // TASK-120: AST Extensions for ADTs - Type System Compilation Tests
    // These tests verify that the new Type variants exist and can be
    // constructed. They will fail to compile until implemented.
    // ============================================================

    #[test]
    fn test_type_instance_exists() {
        // Type::Instance should exist and take a workflow_type: Box<str>
        let _instance = Type::Instance {
            workflow_type: Box::from("MyClass"),
        };

        let _instance_no_args = Type::Instance {
            workflow_type: Box::from("Singleton"),
        };
    }

    #[test]
    fn test_type_instance_addr_exists() {
        // Type::InstanceAddr should exist and take a workflow_type: Box<str>
        let _addr = Type::InstanceAddr {
            workflow_type: Box::from("MyClass"),
        };
    }

    #[test]
    fn test_type_control_link_exists() {
        // Type::ControlLink should exist and take a workflow_type: Box<str>
        let _link = Type::ControlLink {
            workflow_type: Box::from("MyClass"),
        };

        let _link_complex = Type::ControlLink {
            workflow_type: Box::from("ControllerView"),
        };
    }
}
