//! Runtime context for variable bindings
//!
//! Provides nested scope management for the interpreter.

use ash_core::{Name, Value};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// Runtime execution context with variable bindings and obligation tracking
///
/// Contexts form a hierarchy - lookups traverse from child to parent.
/// Bindings are immutable once set (functional style).
/// Obligations use interior mutability for linear discharge semantics.
#[derive(Debug, Clone, Default)]
pub struct Context {
    bindings: HashMap<Name, Value>,
    parent: Option<Box<Context>>,
    /// Active obligations that must be discharged (interior mutability for &self discharge)
    obligations: RefCell<HashSet<Name>>,
    /// Optional role context for authority and obligation tracking
    role_context: Option<crate::role_context::RoleContext>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
            obligations: RefCell::new(HashSet::new()),
            role_context: None,
        }
    }

    /// Add an obligation to the context
    pub fn add_obligation(&self, obligation: Name) {
        self.obligations.borrow_mut().insert(obligation);
    }

    /// Check if an obligation exists and discharge it (remove it)
    /// Returns true if the obligation was found and discharged
    pub fn discharge_obligation(&self, obligation: &str) -> bool {
        self.obligations.borrow_mut().remove(obligation)
    }

    /// Check if an obligation exists (without discharging)
    pub fn has_obligation(&self, obligation: &str) -> bool {
        self.obligations.borrow().contains(obligation)
    }

    /// Look up a variable by name
    ///
    /// Searches current scope, then parent scopes.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }

    /// Bind a variable in the current scope
    ///
    /// Returns the previous value if the name was already bound.
    pub fn set(&mut self, name: Name, value: Value) -> Option<Value> {
        self.bindings.insert(name, value)
    }

    /// Set multiple bindings at once
    pub fn set_many(&mut self, bindings: HashMap<Name, Value>) {
        self.bindings.extend(bindings);
    }

    /// Create a child context that inherits from this one
    ///
    /// Lookups in the child will fall through to parent,
    /// but bindings in the child don't affect the parent.
    pub fn extend(&self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(self.clone())),
            obligations: RefCell::new(HashSet::new()),
            role_context: self.role_context.clone(),
        }
    }

    /// Create a child context with initial bindings
    pub fn with_bindings(bindings: HashMap<Name, Value>) -> Self {
        Self {
            bindings,
            parent: None,
            obligations: RefCell::new(HashSet::new()),
            role_context: None,
        }
    }

    /// Set the role context for this context
    pub fn with_role_context(mut self, role_context: crate::role_context::RoleContext) -> Self {
        self.role_context = Some(role_context);
        self
    }

    /// Get a reference to the role context if set
    pub fn role_context(&self) -> Option<&crate::role_context::RoleContext> {
        self.role_context.as_ref()
    }

    /// Check if all role obligations have been discharged
    ///
    /// Returns true if there is no role context or if all obligations are discharged.
    /// Returns false if there are pending obligations.
    pub fn role_obligations_complete(&self) -> bool {
        self.role_context
            .as_ref()
            .map(|rc| rc.all_discharged())
            .unwrap_or(true)
    }

    /// Get pending role obligations
    ///
    /// Returns empty vector if there is no role context.
    pub fn pending_role_obligations(&self) -> Vec<Name> {
        self.role_context
            .as_ref()
            .map(|rc| rc.pending_obligations())
            .unwrap_or_default()
    }

    /// Get all bindings in this context (excluding parent)
    pub fn local_bindings(&self) -> &HashMap<Name, Value> {
        &self.bindings
    }

    /// Check if a name is bound in this context or any parent
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new_is_empty() {
        let ctx = Context::new();
        assert!(ctx.get("x").is_none());
    }

    #[test]
    fn test_context_set_and_get() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(42));

        assert_eq!(ctx.get("x"), Some(&Value::Int(42)));
        assert_eq!(ctx.get("y"), None);
    }

    #[test]
    fn test_context_parent_lookup() {
        let mut parent = Context::new();
        parent.set("x".to_string(), Value::Int(1));
        parent.set("y".to_string(), Value::Int(2));

        let mut child = parent.extend();
        child.set("y".to_string(), Value::Int(20)); // Shadow y

        // Child sees its own binding for y
        assert_eq!(child.get("y"), Some(&Value::Int(20)));
        // Child sees parent's binding for x
        assert_eq!(child.get("x"), Some(&Value::Int(1)));
        // Neither has z
        assert_eq!(child.get("z"), None);
    }

    #[test]
    fn test_context_parent_unchanged() {
        let mut parent = Context::new();
        parent.set("x".to_string(), Value::Int(1));

        let mut child = parent.extend();
        child.set("x".to_string(), Value::Int(99));

        // Parent is unchanged
        assert_eq!(parent.get("x"), Some(&Value::Int(1)));
    }

    #[test]
    fn test_context_set_many() {
        let mut ctx = Context::new();
        let mut bindings = HashMap::new();
        bindings.insert("a".to_string(), Value::Int(1));
        bindings.insert("b".to_string(), Value::Int(2));

        ctx.set_many(bindings);

        assert_eq!(ctx.get("a"), Some(&Value::Int(1)));
        assert_eq!(ctx.get("b"), Some(&Value::Int(2)));
    }

    #[test]
    fn test_context_with_bindings() {
        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), Value::String("hello".to_string()));

        let ctx = Context::with_bindings(bindings);

        assert_eq!(ctx.get("x"), Some(&Value::String("hello".to_string())));
    }

    #[test]
    fn test_context_contains() {
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Null);

        assert!(ctx.contains("x"));
        assert!(!ctx.contains("y"));
    }

    #[test]
    fn test_context_nested_extend() {
        let mut grandparent = Context::new();
        grandparent.set("a".to_string(), Value::Int(1));

        let mut parent = grandparent.extend();
        parent.set("b".to_string(), Value::Int(2));

        let mut child = parent.extend();
        child.set("c".to_string(), Value::Int(3));

        assert_eq!(child.get("a"), Some(&Value::Int(1)));
        assert_eq!(child.get("b"), Some(&Value::Int(2)));
        assert_eq!(child.get("c"), Some(&Value::Int(3)));
    }
}
