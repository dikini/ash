# TASK-098: Typed Registry Integration

## Status: 🔴 Not Started

## Description

Update registries to store and expose type schemas for validation.

## Specification Reference

- SPEC-015: Typed Providers - Section 4.2 Registry Integration

## Requirements

### Functional Requirements

1. `BehaviourRegistry` stores `TypedBehaviourProvider` (not raw providers)
2. Schema lookup by capability:channel
3. `StreamRegistry` similarly updated
4. Registration requires type schema
5. Backward compatibility (optional untyped for migration)

### Property Requirements

```rust
let mut registry = BehaviourRegistry::new();
registry.register(typed_provider)?;

assert_eq!(registry.get_schema("sensor", "temp"), Some(&expected_schema));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_registry_stores_schema() {
    let mut registry = BehaviourRegistry::new();
    let provider = TypedBehaviourProvider::new(
        mock_provider,
        Type::Record(vec![("x".into(), Type::Int)]),
    );
    
    registry.register(provider);
    
    let schema = registry.get_schema("sensor", "temp");
    assert!(schema.is_some());
}

#[test]
fn test_registry_lookup() {
    let mut registry = BehaviourRegistry::new();
    registry.register(typed_provider);
    
    let found = registry.get("sensor", "temp");
    assert!(found.is_some());
}
```

### Step 2: Verify RED

Expected: FAIL - registry doesn't have schema storage

### Step 3: Implement (Green)

```rust
pub struct BehaviourRegistry {
    providers: HashMap<(Name, Name), TypedBehaviourProvider>,
}

impl BehaviourRegistry {
    pub fn register(&mut self, provider: TypedBehaviourProvider) {
        let key = (provider.capability_name().into(), provider.channel_name().into());
        self.providers.insert(key, provider);
    }
    
    pub fn get(&self, cap: &str, channel: &str) -> Option<&TypedBehaviourProvider> {
        self.providers.get(&(cap.into(), channel.into()))
    }
    
    pub fn get_schema(&self, cap: &str, channel: &str) -> Option<&Type> {
        self.get(cap, channel).map(|p| p.schema())
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: typed registry with schema storage"
```

## Completion Checklist

- [ ] Registry stores typed providers
- [ ] Schema lookup method
- [ ] Stream registry updated
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-096 (Typed provider wrapper)

## Blocked By

- TASK-096

## Blocks

- TASK-099 (Runtime validation)
