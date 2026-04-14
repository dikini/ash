//! Environment frame for closure capture
//!
//! Provides shared, immutable environment snapshots used by closure values.
//! Forms a parent chain via `Arc` for O(1) capture without flattening.

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
            BindingSlot::Late(cell) => cell.lock().unwrap().clone(),
        }
    }

    /// Fill a late-binding slot with a value.
    pub fn set_late(&self, value: Value) {
        if let BindingSlot::Late(cell) = self {
            *cell.lock().unwrap() = Some(value);
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
