# TASK-108: Effect Tracking for All Capabilities

## Status: 🔴 Not Started

## Description

Implement effect tracking for observe, receive, set, and send operations.

## Specification Reference

- SPEC-017: Capability Integration - Section 2 Effects System

## Requirements

### Functional Requirements

1. Classify all capability operations by effect
2. Track effect in workflow type
3. Compute join of all operation effects
4. Enforce effect constraints

### Property Requirements

```rust
// Pure workflow
let effect = workflow.effect();
assert_eq!(effect, Effect::Epistemic);

// Effectful workflow
let effect = workflow_with_set.effect();
assert_eq!(effect, Effect::Operational);
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_observe_effect() {
    let workflow = parse("observe sensor:temp as t");
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_set_effect() {
    let workflow = parse("set hvac:target = 72");
    assert_eq!(workflow.effect(), Effect::Operational);
}

#[test]
fn test_mixed_effect() {
    let workflow = parse("observe sensor:temp; set hvac:target = 72");
    assert_eq!(workflow.effect(), Effect::Operational);
}
```

### Step 2: Verify RED

Expected: FAIL - effect tracking not implemented

### Step 3: Implement (Green)

```rust
impl Workflow {
    pub fn effect(&self) -> Effect {
        match self {
            Workflow::Observe { .. } => Effect::Epistemic,
            Workflow::Receive { .. } => Effect::Epistemic,
            Workflow::Set { .. } => Effect::Operational,
            Workflow::Send { .. } => Effect::Operational,
            Workflow::Seq { first, second } => {
                first.effect().join(second.effect())
            }
            -- ... other variants
        }
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: effect tracking for all capabilities"
```

## Completion Checklist

- [ ] Effect classification for all ops
- [ ] Workflow effect computation
- [ ] Effect join implementation
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

4 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-109 (Obligation checking)
