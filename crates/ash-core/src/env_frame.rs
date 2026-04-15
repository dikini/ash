//! Environment frame for closure capture
//!
//! Provides shared, immutable environment snapshots used by closure values.
//! Forms a parent chain via `Arc` for O(1) capture without flattening.
//!
//! # Usage
//!
//! [`EnvFrame`] is the runtime representation of a captured lexical scope.
//! When a closure (`Value::Closure`) is created, the current variable bindings
//! are snapshot into an `EnvFrame` tree.  Lookup walks the parent chain.
//!
//! [`BindingSlot`] supports two modes:
//! - `Bound`: normal immutable value binding
//! - `Late`: a fill-later slot used for recursive closures — created empty
//!   via [`BindingSlot::new_late()`], then filled via [`BindingSlot::set_late()`]
//!   after the closure is constructed.
//!
//! Both types are `Send + Sync` (verified by compile-time assertions in `lib.rs`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::Value;

/// A slot in the environment. Supports late binding for recursive closures.
#[derive(Debug, Clone)]
pub enum BindingSlot {
    /// Normal immutable binding.
    Bound(Value),
    /// Late binding for recursive let. Filled after closure construction.
    Late(Arc<Mutex<Option<Value>>>),
}

impl BindingSlot {
    /// Create a new late-binding slot.
    pub fn new_late() -> Self {
        Self::Late(Arc::new(Mutex::new(None)))
    }

    /// Resolve the slot to a value, if bound.
    pub fn resolve(&self) -> Option<Value> {
        match self {
            BindingSlot::Bound(v) => Some(v.clone()),
            BindingSlot::Late(cell) => cell.lock().unwrap_or_else(|e| e.into_inner()).clone(),
        }
    }

    /// Fill a late-binding slot with a value.
    pub fn set_late(&self, value: Value) {
        if let BindingSlot::Late(cell) = self {
            *cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(value);
        }
    }
}

/// Shared environment frame for closure capture.
///
/// Forms a parent chain via `Arc` -- O(1) capture, no flattening.
#[derive(Debug, Clone)]
pub struct EnvFrame {
    bindings: HashMap<String, BindingSlot>,
    parent: Option<Arc<EnvFrame>>,
}

impl EnvFrame {
    /// Create a new empty environment frame.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new frame with the given parent.
    pub fn with_parent(parent: Arc<EnvFrame>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Insert an immutable binding.
    pub fn insert(&mut self, name: String, value: Value) {
        self.bindings.insert(name, BindingSlot::Bound(value));
    }

    /// Insert a late-binding slot and return a handle to fill it later.
    pub fn insert_late(&mut self, name: String) -> BindingSlot {
        let slot = BindingSlot::new_late();
        self.bindings.insert(name, slot.clone());
        slot
    }

    /// Look up a binding by name, walking up the parent chain.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(slot) = self.bindings.get(name) {
            return slot.resolve();
        }
        if let Some(parent) = &self.parent {
            return parent.get(name);
        }
        None
    }

    /// Iterate over all local bindings (name, value) in this frame.
    /// Only returns `Bound` slots, not `Late` slots that are still unfilled.
    pub fn iter_bindings(&self) -> impl Iterator<Item = (String, Value)> + '_ {
        self.bindings
            .iter()
            .filter_map(|(name, slot)| slot.resolve().map(|v| (name.clone(), v)))
    }

    /// Get the parent frame, if any.
    pub fn parent(&self) -> Option<&Arc<EnvFrame>> {
        self.parent.as_ref()
    }
}

/// Closures compare by identity (reference equality on captured EnvFrame).
/// Two non-empty frames always compare as not-equal regardless of content.
/// Two empty frames compare as equal only if they share the same parent Arc
/// or both have no parent.
impl PartialEq for EnvFrame {
    fn eq(&self, other: &Self) -> bool {
        self.bindings.is_empty()
            && other.bindings.is_empty()
            && match (&self.parent, &other.parent) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            }
    }
}

impl Default for EnvFrame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// insert + get basic operations
    #[test]
    fn insert_and_get() {
        let mut frame = EnvFrame::new();
        assert!(frame.get("x").is_none());
        frame.insert("x".into(), Value::Int(42));
        assert_eq!(frame.get("x"), Some(Value::Int(42)));
        // Overwrite
        frame.insert("x".into(), Value::Int(7));
        assert_eq!(frame.get("x"), Some(Value::Int(7)));
    }

    /// Parent chain walking: child can see parent bindings.
    #[test]
    fn parent_chain_walking() {
        let mut parent = EnvFrame::new();
        parent.insert("a".into(), Value::Int(1));
        let parent_arc = Arc::new(parent);

        let mut child = EnvFrame::with_parent(parent_arc);
        child.insert("b".into(), Value::Int(2));

        // Child sees its own binding
        assert_eq!(child.get("b"), Some(Value::Int(2)));
        // Child sees parent binding
        assert_eq!(child.get("a"), Some(Value::Int(1)));
        // Missing key returns None
        assert!(child.get("c").is_none());
    }

    /// iter_bindings() filters out unfilled Late slots.
    #[test]
    fn iter_bindings_filters_unfilled_late() {
        let mut frame = EnvFrame::new();
        frame.insert("bound".into(), Value::Int(10));
        frame.insert_late("late_unfilled".into());

        let bindings: Vec<_> = frame.iter_bindings().collect();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "bound");
        assert_eq!(bindings[0].1, Value::Int(10));
    }

    /// PartialEq identity semantics: two distinct EnvFrames with content
    /// EnvFrame PartialEq uses identity semantics: two non-empty frames
    /// are never equal (even clones with identical content), because closure
    /// capture correctness depends on reference identity, not structural
    /// equality.  This is a design contract, not an implementation artifact.
    #[test]
    fn partial_eq_identity_semantics() {
        // Two empty frames with no parent are equal
        let a = EnvFrame::new();
        let b = EnvFrame::new();
        assert_eq!(a, b);

        // Two empty frames sharing the same parent Arc are equal
        let shared_parent = Arc::new(EnvFrame::new());
        let c = EnvFrame::with_parent(shared_parent.clone());
        let d = EnvFrame::with_parent(shared_parent.clone());
        assert_eq!(c, d);

        // Two empty frames with different parent Arcs are not equal
        let p1 = Arc::new(EnvFrame::new());
        let p2 = Arc::new(EnvFrame::new());
        let e = EnvFrame::with_parent(p1);
        let f = EnvFrame::with_parent(p2);
        assert_ne!(e, f);

        // Non-empty frames are never equal to anything (including themselves)
        let mut g = EnvFrame::new();
        g.insert("x".into(), Value::Int(1));
        assert_ne!(g, EnvFrame::new());
        // Reflexivity violation: non-empty frames use identity-based equality
        assert_ne!(
            g,
            g.clone(),
            "non-empty frame should not equal its own clone (identity-based)"
        );
    }

    /// new_late() / set_late() lifecycle: resolve returns None before fill,
    /// returns Some after fill.
    #[test]
    fn late_lifecycle() {
        let mut frame = EnvFrame::new();
        let slot = frame.insert_late("rec".into());

        // Before fill, resolve returns None
        assert!(slot.resolve().is_none());
        assert!(frame.get("rec").is_none());

        // Fill the late binding
        slot.set_late(Value::Int(99));

        // Now resolve returns Some
        assert_eq!(slot.resolve(), Some(Value::Int(99)));
        assert_eq!(frame.get("rec"), Some(Value::Int(99)));
    }

    /// Late binding shared via clone: two clones of the same Late slot,
    /// set on one, both resolve.
    #[test]
    fn late_shared_via_clone() {
        let slot_a = BindingSlot::new_late();
        let slot_b = slot_a.clone();

        // Neither resolves yet
        assert!(slot_a.resolve().is_none());
        assert!(slot_b.resolve().is_none());

        // Set via one clone
        slot_a.set_late(Value::Int(55));

        // Both resolve
        assert_eq!(slot_a.resolve(), Some(Value::Int(55)));
        assert_eq!(slot_b.resolve(), Some(Value::Int(55)));
    }
}
