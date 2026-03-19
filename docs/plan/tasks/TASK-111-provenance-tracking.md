# TASK-111: Provenance Tracking for All Capabilities

## Status: ✅ Complete

## Description

Implement provenance tracking for observe, receive, set, and send operations.

## Specification Reference

- SPEC-017: Capability Integration - Section 5 Provenance

## Requirements

### Functional Requirements

1. Track all capability operations in provenance
2. Record input values (with sensitivity handling)
3. Record output values
4. Include policy decisions
5. Include effect classification

### Property Requirements

```rust
// Input recorded
execute_observe(..., &mut provenance).await;
assert!(provenance.has_event(CapabilityEventType::Observed));

// Output recorded
execute_set(..., &mut provenance).await;
assert!(provenance.has_event(CapabilityEventType::Set));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_observe_provenance() {
    let mut provenance = ProvenanceTracker::new();
    
    execute_observe(&observe, ctx, &behaviour_ctx, &mut provenance).await;
    
    let events = provenance.events();
    assert!(events.iter().any(|e| e.event_type == Observed));
}

#[tokio::test]
async fn test_set_provenance() {
    let mut provenance = ProvenanceTracker::new();
    
    execute_set(&set, ctx, &behaviour_ctx, &mut provenance).await;
    
    let events = provenance.events();
    assert!(events.iter().any(|e| e.event_type == Set));
}
```

### Step 2: Verify RED

Expected: FAIL - provenance not integrated

### Step 3: Implement (Green)

```rust
pub async fn execute_observe(
    observe: &Observe,
    ctx: Context,
    behaviour_ctx: &BehaviourContext,
    provenance: &mut ProvenanceTracker,
) -> ExecResult<Context> {
    -- Record intent
    provenance.record(Intent::Observe {
        capability: observe.capability.clone(),
        channel: observe.channel.clone(),
    });
    
    -- Execute
    let result = provider.sample(...).await;
    
    -- Record outcome
    if let Ok(ref value) = result {
        provenance.record(ProvenanceEvent {
            event_type: CapabilityEventType::Observed,
            direction: Direction::Input,
            capability: observe.capability.clone(),
            channel: observe.channel.clone(),
            value: Some(value.clone()),
            effect: Effect::Epistemic,
            ...
        });
    }
    
    result
}

-- Similar for set, receive, send
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: provenance tracking for all capabilities"
```

## Completion Checklist

- [ ] Observe provenance
- [ ] Receive provenance
- [ ] Set provenance
- [ ] Send provenance
- [ ] Policy decisions recorded
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

6 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

None
