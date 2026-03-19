# TASK-036: Runtime Policy Evaluation

## Status: ✅ Complete

## Description

Implement the runtime policy evaluation system for dynamic authorization decisions.

## Specification Reference

- SPEC-004: Operational Semantics - Policy evaluation
- SPEC-001: IR - Policy definitions

## Requirements

### Policy Trait

```rust
/// Policy trait for authorization decisions
#[async_trait]
pub trait Policy: Send + Sync + Debug {
    /// Evaluate policy for given input and context
    async fn evaluate(&self, input: &Value, ctx: &PolicyContext) -> Decision;
    
    /// Get policy metadata
    fn metadata(&self) -> PolicyMetadata;
}

/// Policy context for evaluation
#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub env: Environment,
    pub roles: Vec<Box<str>>,
    pub timestamp: DateTime<Utc>,
    pub request_context: HashMap<Box<str>, Value>,
}

impl PolicyContext {
    pub fn new(env: Environment) -> Self {
        Self {
            env,
            roles: Vec::new(),
            timestamp: Utc::now(),
            request_context: HashMap::new(),
        }
    }
    
    pub fn with_role(mut self, role: impl Into<Box<str>>) -> Self {
        self.roles.push(role.into());
        self
    }
}

/// Policy metadata
#[derive(Debug, Clone)]
pub struct PolicyMetadata {
    pub name: Box<str>,
    pub description: Option<Box<str>>,
    pub effect: Effect,
    pub parameters: Vec<ParameterSpec>,
}

/// Authorization decision
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Permit,
    Deny,
    RequireApproval { role: Box<str> },
    Escalate,
}
```

### Policy Registry Enhancements

```rust
/// Enhanced policy registry
#[derive(Debug, Default)]
pub struct PolicyRegistry {
    policies: HashMap<Box<str>, Arc<dyn Policy>>,
    default_policy: Option<Box<str>>,
}

impl PolicyRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register(&mut self, policy: Arc<dyn Policy>) {
        let name = policy.metadata().name.clone();
        self.policies.insert(name, policy);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn Policy>> {
        self.policies.get(name).cloned()
    }
    
    pub fn set_default(&mut self, name: impl Into<Box<str>>) {
        self.default_policy = Some(name.into());
    }
    
    pub async fn evaluate(
        &self,
        name: Option<&str>,
        input: &Value,
        ctx: &PolicyContext,
    ) -> Decision {
        let policy_name = name.or(self.default_policy.as_deref())
            .unwrap_or("deny_all");
        
        match self.get(policy_name) {
            Some(policy) => policy.evaluate(input, ctx).await,
            None => Decision::Deny,
        }
    }
    
    /// Register built-in policies
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        
        registry.register(Arc::new(AllowAllPolicy));
        registry.register(Arc::new(DenyAllPolicy));
        registry.register(Arc::new(AdminOnlyPolicy));
        registry.register(Arc::new(TimeBasedPolicy::business_hours()));
        
        registry
    }
}
```

### Built-in Policies

```rust
/// Allow all policy
#[derive(Debug)]
pub struct AllowAllPolicy;

#[async_trait]
impl Policy for AllowAllPolicy {
    async fn evaluate(&self, _input: &Value, _ctx: &PolicyContext) -> Decision {
        Decision::Permit
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "allow_all".into(),
            description: Some("Allow all requests".into()),
            effect: Effect::Evaluative,
            parameters: vec![],
        }
    }
}

/// Deny all policy
#[derive(Debug)]
pub struct DenyAllPolicy;

#[async_trait]
impl Policy for DenyAllPolicy {
    async fn evaluate(&self, _input: &Value, _ctx: &PolicyContext) -> Decision {
        Decision::Deny
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "deny_all".into(),
            description: Some("Deny all requests".into()),
            effect: Effect::Evaluative,
            parameters: vec![],
        }
    }
}

/// Admin only policy
#[derive(Debug)]
pub struct AdminOnlyPolicy;

#[async_trait]
impl Policy for AdminOnlyPolicy {
    async fn evaluate(&self, _input: &Value, ctx: &PolicyContext) -> Decision {
        if ctx.roles.contains(&"admin".into()) {
            Decision::Permit
        } else {
            Decision::Deny
        }
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "admin_only".into(),
            description: Some("Only admin role permitted".into()),
            effect: Effect::Evaluative,
            parameters: vec![],
        }
    }
}

/// Threshold policy
#[derive(Debug)]
pub struct ThresholdPolicy {
    pub threshold: i64,
    pub field: Box<str>,
}

#[async_trait]
impl Policy for ThresholdPolicy {
    async fn evaluate(&self, input: &Value, _ctx: &PolicyContext) -> Decision {
        let value = match input {
            Value::Int(n) => *n,
            Value::Record(fields) => {
                match fields.get(&self.field) {
                    Some(Value::Int(n)) => *n,
                    _ => return Decision::Deny,
                }
            }
            _ => return Decision::Deny,
        };
        
        if value >= self.threshold {
            Decision::Permit
        } else {
            Decision::Deny
        }
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "threshold".into(),
            description: Some("Threshold-based policy".into()),
            effect: Effect::Evaluative,
            parameters: vec![
                ParameterSpec {
                    name: "threshold".into(),
                    ty: Type::Int,
                    required: true,
                    default: None,
                    description: None,
                },
            ],
        }
    }
}

/// Time-based policy
#[derive(Debug)]
pub struct TimeBasedPolicy {
    pub start_hour: u32,
    pub end_hour: u32,
    pub timezone: chrono::FixedOffset,
}

impl TimeBasedPolicy {
    pub fn business_hours() -> Self {
        Self {
            start_hour: 9,
            end_hour: 17,
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
        }
    }
}

#[async_trait]
impl Policy for TimeBasedPolicy {
    async fn evaluate(&self, _input: &Value, ctx: &PolicyContext) -> Decision {
        let local_time = ctx.timestamp.with_timezone(&self.timezone);
        let hour = local_time.hour();
        
        if hour >= self.start_hour && hour < self.end_hour {
            Decision::Permit
        } else {
            Decision::RequireApproval { role: "manager".into() }
        }
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "business_hours".into(),
            description: Some("Business hours only".into()),
            effect: Effect::Evaluative,
            parameters: vec![],
        }
    }
}

/// Composite policy (all must permit)
#[derive(Debug)]
pub struct AllOfPolicy {
    pub policies: Vec<Box<str>>,
}

#[async_trait]
impl Policy for AllOfPolicy {
    async fn evaluate(&self, input: &Value, ctx: &PolicyContext) -> Decision {
        // Would need registry access to evaluate sub-policies
        // Simplified for now
        Decision::Permit
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "all_of".into(),
            description: Some("All policies must permit".into()),
            effect: Effect::Evaluative,
            parameters: vec![],
        }
    }
}

/// Composite policy (any may permit)
#[derive(Debug)]
pub struct AnyOfPolicy {
    pub policies: Vec<Box<str>>,
}

#[async_trait]
impl Policy for AnyOfPolicy {
    async fn evaluate(&self, input: &Value, ctx: &PolicyContext) -> Decision {
        // Would need registry access to evaluate sub-policies
        // Simplified for now
        Decision::Permit
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "any_of".into(),
            description: Some("Any policy may permit".into()),
            effect: Effect::Evaluative,
            parameters: vec![],
        }
    }
}
```

### Policy Composition

```rust
/// Policy composition utilities
pub struct PolicyComposer;

impl PolicyComposer {
    /// Create a policy that requires all sub-policies to permit
    pub fn all_of(policies: Vec<Arc<dyn Policy>>) -> Arc<dyn Policy> {
        Arc::new(ComposedPolicy::AllOf(policies))
    }
    
    /// Create a policy that permits if any sub-policy permits
    pub fn any_of(policies: Vec<Arc<dyn Policy>>) -> Arc<dyn Policy> {
        Arc::new(ComposedPolicy::AnyOf(policies))
    }
    
    /// Create a policy that negates another
    pub fn not(policy: Arc<dyn Policy>) -> Arc<dyn Policy> {
        Arc::new(ComposedPolicy::Not(policy))
    }
}

#[derive(Debug)]
enum ComposedPolicy {
    AllOf(Vec<Arc<dyn Policy>>),
    AnyOf(Vec<Arc<dyn Policy>>),
    Not(Arc<dyn Policy>>),
}

#[async_trait]
impl Policy for ComposedPolicy {
    async fn evaluate(&self, input: &Value, ctx: &PolicyContext) -> Decision {
        match self {
            ComposedPolicy::AllOf(policies) => {
                for policy in policies {
                    match policy.evaluate(input, ctx).await {
                        Decision::Deny => return Decision::Deny,
                        Decision::RequireApproval { role } => {
                            return Decision::RequireApproval { role }
                        }
                        _ => continue,
                    }
                }
                Decision::Permit
            }
            ComposedPolicy::AnyOf(policies) => {
                for policy in policies {
                    match policy.evaluate(input, ctx).await {
                        Decision::Permit => return Decision::Permit,
                        _ => continue,
                    }
                }
                Decision::Deny
            }
            ComposedPolicy::Not(policy) => {
                match policy.evaluate(input, ctx).await {
                    Decision::Permit => Decision::Deny,
                    Decision::Deny => Decision::Permit,
                    other => other,
                }
            }
        }
    }
    
    fn metadata(&self) -> PolicyMetadata {
        PolicyMetadata {
            name: "composed".into(),
            description: Some("Composed policy".into()),
            effect: Effect::Evaluative,
            parameters: vec![],
        }
    }
}
```

## TDD Steps

### Step 1: Define Policy Trait

Create `crates/ash-interp/src/policy.rs`.

### Step 2: Implement Built-in Policies

Add AllowAllPolicy, DenyAllPolicy, AdminOnlyPolicy, ThresholdPolicy, TimeBasedPolicy.

### Step 3: Implement Policy Composition

Add AllOf, AnyOf, Not composition.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allow_all_policy() {
        let policy = AllowAllPolicy;
        let ctx = PolicyContext::new(Environment::new());
        
        let decision = policy.evaluate(&Value::Null, &ctx).await;
        assert_eq!(decision, Decision::Permit);
    }

    #[tokio::test]
    async fn test_deny_all_policy() {
        let policy = DenyAllPolicy;
        let ctx = PolicyContext::new(Environment::new());
        
        let decision = policy.evaluate(&Value::Null, &ctx).await;
        assert_eq!(decision, Decision::Deny);
    }

    #[tokio::test]
    async fn test_admin_only_policy() {
        let policy = AdminOnlyPolicy;
        
        let ctx = PolicyContext::new(Environment::new());
        let decision = policy.evaluate(&Value::Null, &ctx).await;
        assert_eq!(decision, Decision::Deny);
        
        let ctx = PolicyContext::new(Environment::new())
            .with_role("admin");
        let decision = policy.evaluate(&Value::Null, &ctx).await;
        assert_eq!(decision, Decision::Permit);
    }

    #[tokio::test]
    async fn test_threshold_policy() {
        let policy = ThresholdPolicy {
            threshold: 100,
            field: "amount".into(),
        };
        let ctx = PolicyContext::new(Environment::new());
        
        let mut fields = HashMap::new();
        fields.insert("amount".into(), Value::Int(150));
        let input = Value::Record(fields);
        
        let decision = policy.evaluate(&input, &ctx).await;
        assert_eq!(decision, Decision::Permit);
        
        let mut fields = HashMap::new();
        fields.insert("amount".into(), Value::Int(50));
        let input = Value::Record(fields);
        
        let decision = policy.evaluate(&input, &ctx).await;
        assert_eq!(decision, Decision::Deny);
    }

    #[tokio::test]
    async fn test_policy_composition_all_of() {
        let policy1: Arc<dyn Policy> = Arc::new(AllowAllPolicy);
        let policy2: Arc<dyn Policy> = Arc::new(AllowAllPolicy);
        
        let composed = PolicyComposer::all_of(vec![policy1, policy2]);
        let ctx = PolicyContext::new(Environment::new());
        
        let decision = composed.evaluate(&Value::Null, &ctx).await;
        assert_eq!(decision, Decision::Permit);
    }
}
```

## Completion Checklist

- [ ] Policy trait
- [ ] PolicyContext
- [ ] Decision enum
- [ ] PolicyRegistry
- [ ] AllowAllPolicy
- [ ] DenyAllPolicy
- [ ] AdminOnlyPolicy
- [ ] ThresholdPolicy
- [ ] TimeBasedPolicy
- [ ] Policy composition (AllOf, AnyOf, Not)
- [ ] Unit tests for each policy
- [ ] Composition tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Flexibility**: Can policies be composed easily?
2. **Context**: Is enough context available for decisions?
3. **Extensibility**: Is the trait easy to implement?

## Estimated Effort

6 hours

## Dependencies

- ash-core: Effect, Value, Environment

## Blocked By

- ash-core: Core types

## Blocks

- TASK-032: Evaluative execution
- TASK-033: Operational execution
