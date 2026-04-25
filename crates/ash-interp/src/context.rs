//! Runtime context for variable bindings
//!
//! Provides nested scope management for the interpreter.

use ash_core::runtime::{EffectScopeId, FailureEntity, LexicalFrameId, TowerLevel};
use ash_core::{Name, Value};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Runtime execution context with variable bindings and obligation tracking
///
/// Contexts form a hierarchy - lookups traverse from child to parent.
/// Bindings are immutable once set (functional style).
/// Obligations use interior mutability for linear discharge semantics.
#[derive(Debug)]
pub struct Context {
    bindings: HashMap<Name, Value>,
    parent: Option<Box<Context>>,
    /// Active obligations that must be discharged.
    obligations: Arc<Mutex<HashSet<Name>>>,
    /// Optional role context for authority and obligation tracking
    role_context: Option<crate::role_context::RoleContext>,
    /// Hidden runtime policy evaluator available to expression-level bridge primitives.
    policy_evaluator: Option<Arc<crate::policy::PolicyEvaluator>>,
    /// Hidden runtime Act environment available to expression-level Act forcing.
    act_env: Option<Arc<crate::act_env::ActEnv>>,
    /// Process identity metadata for component-wise projected child process contexts.
    process_identity: Option<crate::process_env::ProcessEnvIdentity>,
    /// Hidden runtime state for Proc handle observation.
    runtime_state: Option<Arc<crate::runtime_state::RuntimeState>>,
    /// Current lexical-frame identity for pure failure attribution.
    lexical_frame_id: LexicalFrameId,
    /// Current semantic tower used for operational failure attribution.
    current_tower: TowerLevel,
    /// Current effect scope identity when executing effectful/Act code.
    effect_scope_id: Option<EffectScopeId>,
    /// Pure-context nesting depth for SPEC-031 three-vertex boundary enforcement.
    ///
    /// 0 = not in a pure context.  >0 = inside `pure_depth` layers of pure-fn calls.
    /// When `pure_depth > 0`, `Expr::FnDef` is rejected at runtime (the type checker
    /// is the primary enforcer; this is a defense-in-depth safety net).
    pure_depth: u32,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        let obligations = self
            .obligations
            .lock()
            .expect("context obligations mutex should not be poisoned")
            .clone();
        Self {
            bindings: self.bindings.clone(),
            parent: self.parent.clone(),
            obligations: Arc::new(Mutex::new(obligations)),
            role_context: self.role_context.clone(),
            policy_evaluator: self.policy_evaluator.clone(),
            act_env: self.act_env.clone(),
            process_identity: self.process_identity,
            runtime_state: self.runtime_state.clone(),
            lexical_frame_id: self.lexical_frame_id,
            current_tower: self.current_tower,
            effect_scope_id: self.effect_scope_id,
            pure_depth: self.pure_depth,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
            obligations: Arc::new(Mutex::new(HashSet::new())),
            role_context: None,
            policy_evaluator: None,
            act_env: None,
            process_identity: None,
            runtime_state: None,
            lexical_frame_id: LexicalFrameId::new(),
            current_tower: TowerLevel::Pure,
            effect_scope_id: None,
            pure_depth: 0,
        }
    }

    /// Add an obligation to the context
    pub fn add_obligation(&self, obligation: Name) {
        self.obligations
            .lock()
            .expect("context obligations mutex should not be poisoned")
            .insert(obligation);
    }

    /// Check if an obligation exists and discharge it (remove it)
    /// Returns true if the obligation was found and discharged
    pub fn discharge_obligation(&self, obligation: &str) -> bool {
        self.obligations
            .lock()
            .expect("context obligations mutex should not be poisoned")
            .remove(obligation)
    }

    /// Check if an obligation exists (without discharging)
    pub fn has_obligation(&self, obligation: &str) -> bool {
        self.obligations
            .lock()
            .expect("context obligations mutex should not be poisoned")
            .contains(obligation)
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
            obligations: Arc::new(Mutex::new(HashSet::new())),
            role_context: self.role_context.clone(),
            policy_evaluator: self.policy_evaluator.clone(),
            act_env: self.act_env.clone(),
            process_identity: self.process_identity,
            runtime_state: self.runtime_state.clone(),
            lexical_frame_id: LexicalFrameId::new(),
            current_tower: self.current_tower,
            effect_scope_id: self.effect_scope_id,
            pure_depth: self.pure_depth,
        }
    }

    /// Create a child context with initial bindings
    pub fn with_bindings(bindings: HashMap<Name, Value>) -> Self {
        Self {
            bindings,
            parent: None,
            obligations: Arc::new(Mutex::new(HashSet::new())),
            role_context: None,
            policy_evaluator: None,
            act_env: None,
            process_identity: None,
            runtime_state: None,
            lexical_frame_id: LexicalFrameId::new(),
            current_tower: TowerLevel::Pure,
            effect_scope_id: None,
            pure_depth: 0,
        }
    }

    /// Enter a pure-function context (SPEC-031 three-vertex boundary).
    ///
    /// Returns a child context with `pure_depth` incremented by one.
    /// Inside a pure context, `Expr::FnDef` raises `BoundaryViolation`.
    ///
    /// TODO: Currently only used in tests.  Activate in production by
    /// propagating purity context through closure application in eval.rs
    /// (e.g., when calling a `Value::Closure` that was typed as `Type::Fn`).
    pub fn enter_pure(&self) -> Self {
        let mut child = self.extend();
        child.pure_depth = self.pure_depth + 1;
        child.current_tower = TowerLevel::Pure;
        child.effect_scope_id = None;
        child
    }

    /// Returns `true` when we are inside at least one pure-fn call.
    pub fn is_pure(&self) -> bool {
        self.pure_depth > 0
    }

    /// Set the role context for this context
    pub fn with_role_context(mut self, role_context: crate::role_context::RoleContext) -> Self {
        self.role_context = Some(role_context);
        self
    }

    /// Attach a hidden runtime policy evaluator to this context.
    pub fn with_policy_evaluator(
        mut self,
        policy_evaluator: crate::policy::PolicyEvaluator,
    ) -> Self {
        self.policy_evaluator = Some(Arc::new(policy_evaluator));
        self
    }

    /// Return the hidden runtime policy evaluator if one is present.
    pub fn policy_evaluator(&self) -> Option<Arc<crate::policy::PolicyEvaluator>> {
        self.policy_evaluator.clone()
    }

    /// Attach a shared hidden runtime policy evaluator to this context.
    pub(crate) fn with_policy_evaluator_arc(
        mut self,
        policy_evaluator: Arc<crate::policy::PolicyEvaluator>,
    ) -> Self {
        self.policy_evaluator = Some(policy_evaluator);
        self
    }

    /// Attach a hidden runtime Act environment to this context.
    pub fn with_act_env(mut self, act_env: crate::act_env::ActEnv) -> Self {
        self.act_env = Some(Arc::new(act_env));
        self
    }

    /// Attach a shared hidden runtime Act environment to this context.
    pub(crate) fn with_act_env_arc(mut self, act_env: Arc<crate::act_env::ActEnv>) -> Self {
        self.act_env = Some(act_env);
        self
    }

    /// Return the hidden runtime Act environment if one is present.
    pub fn act_env(&self) -> Option<Arc<crate::act_env::ActEnv>> {
        self.act_env.clone()
    }

    /// Return projected process identity metadata when this context represents a child process.
    pub fn process_identity(&self) -> Option<crate::process_env::ProcessEnvIdentity> {
        self.process_identity
    }

    /// Attach hidden runtime state for Proc handle observation.
    pub fn with_runtime_state(mut self, runtime_state: crate::runtime_state::RuntimeState) -> Self {
        self.runtime_state = Some(Arc::new(runtime_state));
        self
    }

    /// Attach shared hidden runtime state for Proc handle observation.
    pub(crate) fn with_runtime_state_arc(
        mut self,
        runtime_state: Arc<crate::runtime_state::RuntimeState>,
    ) -> Self {
        self.runtime_state = Some(runtime_state);
        self
    }

    /// Return the hidden runtime state if one is present.
    pub fn runtime_state(&self) -> Option<Arc<crate::runtime_state::RuntimeState>> {
        self.runtime_state.clone()
    }

    /// Return the current semantic-tower attribution and identity for `fail`.
    pub(crate) fn current_failure_attribution(&self) -> (TowerLevel, FailureEntity) {
        match self.current_tower {
            TowerLevel::Pure => (
                TowerLevel::Pure,
                FailureEntity::LexicalFrame(self.lexical_frame_id),
            ),
            TowerLevel::Effectful => (
                TowerLevel::Effectful,
                FailureEntity::EffectScope(
                    self.effect_scope_id
                        .expect("effectful context must carry an effect scope id"),
                ),
            ),
            TowerLevel::Proc => (
                TowerLevel::Proc,
                FailureEntity::Process(
                    self.process_identity
                        .expect("proc context must carry process identity metadata")
                        .process_id,
                ),
            ),
            TowerLevel::Workflow => {
                panic!("workflow failure attribution is not threaded through Context yet")
            }
        }
    }

    /// Inherit runtime failure-attribution metadata from a parent execution context.
    pub(crate) fn inherit_runtime_metadata_from(mut self, parent: &Context) -> Self {
        self.process_identity = parent.process_identity;
        self.runtime_state = parent.runtime_state.clone();
        self.current_tower = parent.current_tower;
        self.effect_scope_id = parent.effect_scope_id;
        self
    }

    /// Mark this context as entering a fresh effect scope for Act execution.
    pub(crate) fn with_effect_scope(mut self, effect_scope_id: EffectScopeId) -> Self {
        self.current_tower = TowerLevel::Effectful;
        self.effect_scope_id = Some(effect_scope_id);
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

    /// Return the local pending obligations visible in this context frame.
    pub fn local_pending_obligations(&self) -> BTreeSet<Name> {
        self.obligations
            .lock()
            .expect("context obligations mutex should not be poisoned")
            .iter()
            .cloned()
            .collect()
    }

    /// Replace the local pending-obligation set for this frame.
    pub fn replace_local_obligations(&self, obligations: impl IntoIterator<Item = Name>) {
        let replacement: HashSet<Name> = obligations.into_iter().collect();
        *self
            .obligations
            .lock()
            .expect("context obligations mutex should not be poisoned") = replacement;
    }

    /// Return the cumulative pending obligations visible from this frame through its parent chain.
    pub fn visible_pending_obligations(&self) -> BTreeSet<Name> {
        let mut pending = self
            .parent
            .as_ref()
            .map(|parent| parent.visible_pending_obligations())
            .unwrap_or_default();
        pending.extend(self.local_pending_obligations());
        pending
    }

    /// Get all bindings in this context (excluding parent)
    pub fn local_bindings(&self) -> &HashMap<Name, Value> {
        &self.bindings
    }

    /// Snapshot all visible bindings through the parent chain.
    ///
    /// Parent bindings are inserted first and local bindings override them,
    /// matching lookup semantics without copying parent obligation state.
    pub fn visible_bindings(&self) -> HashMap<Name, Value> {
        let mut bindings = self
            .parent
            .as_ref()
            .map(|parent| parent.visible_bindings())
            .unwrap_or_default();
        bindings.extend(self.bindings.clone());
        bindings
    }

    /// Build a child process context from component-wise projection.
    pub(crate) fn project_process_child(
        &self,
        process_identity: crate::process_env::ProcessEnvIdentity,
        role_context: Option<crate::role_context::RoleContext>,
    ) -> Self {
        Self {
            bindings: self.visible_bindings(),
            parent: None,
            obligations: Arc::new(Mutex::new(HashSet::new())),
            role_context,
            policy_evaluator: self.policy_evaluator.clone(),
            act_env: self.act_env.clone(),
            process_identity: Some(process_identity),
            runtime_state: self.runtime_state.clone(),
            lexical_frame_id: LexicalFrameId::new(),
            current_tower: TowerLevel::Proc,
            effect_scope_id: None,
            pure_depth: self.pure_depth,
        }
    }

    /// Check if a name is bound in this context or any parent
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Snapshot current scope chain as an EnvFrame (shared via Arc).
    ///
    /// Walks up the context chain, building an EnvFrame chain.
    /// Each scope level becomes an EnvFrame with a parent link.
    /// NOTE: obligation and role state is not captured in EnvFrame.
    /// Closure bodies that need obligation/role context will not have it
    /// when invoked through the captured environment.
    pub fn to_env_frame(&self) -> std::sync::Arc<ash_core::env_frame::EnvFrame> {
        use ash_core::env_frame::EnvFrame;
        let parent = self.parent.as_ref().map(|p| p.to_env_frame());
        let mut frame = match parent {
            Some(p) => EnvFrame::with_parent(p),
            None => EnvFrame::new(),
        };
        for (name, value) in &self.bindings {
            frame.insert(name.clone(), value.clone());
        }
        std::sync::Arc::new(frame)
    }

    /// Create a Context from a captured EnvFrame.
    ///
    /// Builds a Context that mirrors the EnvFrame chain by walking
    /// the parent links and populating bindings at each level.
    pub fn from_env_frame(frame: &std::sync::Arc<ash_core::env_frame::EnvFrame>) -> Self {
        fn build(frame: &std::sync::Arc<ash_core::env_frame::EnvFrame>) -> Context {
            let parent = frame.parent().map(|p| Box::new(build(p)));
            let mut ctx = Context::new();
            ctx.parent = parent;
            for (name, value) in frame.iter_bindings() {
                ctx.bindings.insert(name, value);
            }
            ctx
        }
        build(frame)
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

    #[test]
    fn test_context_clone_copies_local_obligations_by_value() {
        let ctx = Context::new();
        ctx.add_obligation("audit".to_string());

        let cloned = ctx.clone();
        assert!(cloned.has_obligation("audit"));

        cloned.discharge_obligation("audit");

        assert!(
            ctx.has_obligation("audit"),
            "cloning Context should preserve today's by-value local obligation semantics"
        );
        assert!(
            !cloned.has_obligation("audit"),
            "discharging on the clone should not mutate the original"
        );
    }

    #[test]
    fn task689d_context_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Context>();
    }
}
