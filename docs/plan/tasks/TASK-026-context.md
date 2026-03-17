# TASK-026: Runtime Context and State

## Status: 🟢 Complete

## Description

Implement the runtime context and state management for the Ash interpreter, including variable bindings, capability providers, and execution environment.

## Specification Reference

- SPEC-004: Operational Semantics - Section 2. Semantic Domains
- SHARO_CORE_LANGUAGE.md - Section 9. Execution Engine

## Requirements

### Runtime Context

```rust
/// Runtime execution context
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// Variable environment (immutable, structural sharing)
    pub env: Environment,
    
    /// Available capabilities
    pub capabilities: Arc<CapabilityRegistry>,
    
    /// Active policies
    pub policies: Arc<PolicyRegistry>,
    
    /// Current effect level
    pub effect_level: Effect,
    
    /// Active obligations
    pub obligations: Vec<Obligation>,
    
    /// Provenance tracking
    pub provenance: Provenance,
    
    /// Configuration
    pub config: RuntimeConfig,
}

/// Variable environment with structural sharing
#[derive(Debug, Clone, Default)]
pub struct Environment {
    bindings: HashMap<Box<str>, Value>,
    parent: Option<Arc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_parent(parent: Arc<Environment>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }
    
    /// Get a variable's value
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.get(name))
        })
    }
    
    /// Bind a variable (returns new environment)
    pub fn bind(&self, name: impl Into<Box<str>>, value: Value) -> Self {
        let mut new = self.clone();
        new.bindings.insert(name.into(), value);
        new
    }
    
    /// Bind multiple variables (returns new environment)
    pub fn bind_many(&self, bindings: impl IntoIterator<Item = (Box<str>, Value)>) -> Self {
        let mut new = self.clone();
        new.bindings.extend(bindings);
        new
    }
    
    /// Create a child scope
    pub fn enter_scope(self: &Arc<Self>) -> Environment {
        Environment::with_parent(self.clone())
    }
}

impl RuntimeContext {
    pub fn new(
        capabilities: Arc<CapabilityRegistry>,
        policies: Arc<PolicyRegistry>,
    ) -> Self {
        Self {
            env: Environment::new(),
            capabilities,
            policies,
            effect_level: Effect::Epistemic,
            obligations: Vec::new(),
            provenance: Provenance::root(),
            config: RuntimeConfig::default(),
        }
    }
    
    /// Fork context for parallel execution
    pub fn fork(&self) -> Self {
        Self {
            env: self.env.clone(),
            capabilities: self.capabilities.clone(),
            policies: self.policies.clone(),
            effect_level: self.effect_level,
            obligations: self.obligations.clone(),
            provenance: self.provenance.fork(),
            config: self.config.clone(),
        }
    }
    
    /// Merge contexts after parallel execution
    pub fn merge(&self, others: &[Self]) -> Self {
        // Merge environments (taking union of bindings)
        let mut merged_env = self.env.clone();
        for other in others {
            for (name, value) in &other.env.bindings {
                merged_env.bindings.insert(name.clone(), value.clone());
            }
        }
        
        // Effect is max of all
        let max_effect = others.iter()
            .map(|o| o.effect_level)
            .fold(self.effect_level, Effect::join);
        
        // Merge provenance
        let merged_provenance = others.iter()
            .fold(self.provenance.clone(), |acc, o| acc.merge(&o.provenance));
        
        Self {
            env: merged_env,
            capabilities: self.capabilities.clone(),
            policies: self.policies.clone(),
            effect_level: max_effect,
            obligations: self.obligations.clone(), // Obligations must be satisfied by all
            provenance: merged_provenance,
            config: self.config.clone(),
        }
    }
    
    /// Update effect level
    pub fn with_effect(&self, effect: Effect) -> Self {
        let mut new = self.clone();
        new.effect_level = new.effect_level.join(effect);
        new
    }
    
    /// Add obligation
    pub fn with_obligation(&self, obligation: Obligation) -> Self {
        let mut new = self.clone();
        new.obligations.push(obligation);
        new
    }
    
    /// Lookup capability
    pub fn get_capability(&self, name: &str) -> Option<Arc<dyn CapabilityProvider>> {
        self.capabilities.get(name)
    }
    
    /// Evaluate policy
    pub fn evaluate_policy(&self, name: &str, input: &Value) -> Decision {
        self.policies.evaluate(name, input, &self.env)
    }
}
```

### Runtime Configuration

```rust
/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Execution timeout
    pub timeout: Duration,
    
    /// Maximum recursion depth
    pub max_recursion: usize,
    
    /// Enable trace recording
    pub trace_enabled: bool,
    
    /// Strict mode (fail on warnings)
    pub strict: bool,
    
    /// Enable provenance tracking
    pub provenance_enabled: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_recursion: 100,
            trace_enabled: true,
            strict: false,
            provenance_enabled: true,
        }
    }
}
```

### Capability Registry

```rust
/// Registry of capability providers
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    providers: HashMap<Box<str>, Arc<dyn CapabilityProvider>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register(&mut self, name: impl Into<Box<str>>, provider: Arc<dyn CapabilityProvider>) {
        self.providers.insert(name.into(), provider);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn CapabilityProvider>> {
        self.providers.get(name).cloned()
    }
    
    /// Built-in capabilities
    pub fn builtins() -> Self {
        let mut registry = Self::new();
        
        // Register built-in capabilities
        registry.register("print", Arc::new(PrintCapability));
        registry.register("read_file", Arc::new(ReadFileCapability));
        registry.register("http_get", Arc::new(HttpGetCapability));
        
        registry
    }
}

/// Capability provider trait
#[async_trait]
pub trait CapabilityProvider: Send + Sync + Debug {
    /// Get capability metadata
    fn metadata(&self) -> CapabilityMetadata;
    
    /// Execute the capability
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError>;
}

#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    pub name: Box<str>,
    pub effect: Effect,
    pub parameters: Vec<ParameterSpec>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct ParameterSpec {
    pub name: Box<str>,
    pub ty: Type,
    pub required: bool,
}
```

### Policy Registry

```rust
/// Registry of policies
#[derive(Debug, Default)]
pub struct PolicyRegistry {
    policies: HashMap<Box<str>, Arc<dyn Policy>>,
}

impl PolicyRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register(&mut self, name: impl Into<Box<str>>, policy: Arc<dyn Policy>) {
        self.policies.insert(name.into(), policy);
    }
    
    pub fn evaluate(&self, name: &str, input: &Value, env: &Environment) -> Decision {
        match self.policies.get(name) {
            Some(policy) => policy.evaluate(input, env),
            None => Decision::Deny, // Default deny
        }
    }
}

/// Policy trait
pub trait Policy: Send + Sync + Debug {
    fn evaluate(&self, input: &Value, env: &Environment) -> Decision;
}
```

## TDD Steps

### Step 1: Implement Environment

Create `crates/ash-interp/src/context.rs` with Environment struct.

### Step 2: Implement RuntimeContext

Add RuntimeContext with fork/merge.

### Step 3: Implement Capability Registry

Add CapabilityRegistry and CapabilityProvider trait.

### Step 4: Implement Policy Registry

Add PolicyRegistry and Policy trait.

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_binding() {
        let env = Environment::new();
        let env = env.bind("x", Value::Int(42));
        
        assert_eq!(env.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_environment_parent_lookup() {
        let parent = Arc::new(Environment::new().bind("x", Value::Int(1)));
        let child = Environment::with_parent(parent).bind("y", Value::Int(2));
        
        assert_eq!(child.get("x"), Some(&Value::Int(1))); // From parent
        assert_eq!(child.get("y"), Some(&Value::Int(2))); // From child
    }

    #[test]
    fn test_context_fork() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        let forked = ctx.fork();
        
        assert_eq!(ctx.effect_level, forked.effect_level);
    }

    #[test]
    fn test_context_merge() {
        let ctx1 = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        let mut ctx2 = ctx1.fork();
        ctx2.env = ctx2.env.bind("x", Value::Int(42));
        
        let merged = ctx1.merge(&[ctx2]);
        
        assert_eq!(merged.env.get("x"), Some(&Value::Int(42)));
    }
}
```

## Completion Checklist

- [ ] Environment with structural sharing
- [ ] RuntimeContext with fork/merge
- [ ] CapabilityRegistry
- [ ] CapabilityProvider trait
- [ ] PolicyRegistry
- [ ] Policy trait
- [ ] RuntimeConfig
- [ ] Unit tests for context operations
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Sharing**: Is environment sharing efficient?
2. **Isolation**: Are forked contexts properly isolated?
3. **Merge semantics**: Are merge operations correct?

## Estimated Effort

4 hours

## Dependencies

- ash-core: Value, Effect, Provenance types

## Blocked By

- ash-core: Core types

## Blocks

- TASK-027: Expression evaluator (needs context)
- All other interpreter tasks
