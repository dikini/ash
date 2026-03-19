# TASK-108: Effect Tracking for All Capabilities

## Status: ✅ Complete

## Description

Complete effect tracking for observe, receive, set, and send operations.

**Current State:**
- ✅ Effect tracking implemented for Observe, Set, Send, and other workflows
- ✅ `Receive` workflow variant added to surface AST with `ReceiveMode`, `StreamPattern`, `ReceiveArm`
- ✅ Effect tracking for `Receive` (Epistemic for read-only message reception) per SPEC-017

## Specification Reference

- SPEC-017: Capability Integration - Section 2 Effects System

## Requirements

### Functional Requirements

1. Add `Workflow::Receive` variant to surface AST (integrate `ReceiveExpr`)
2. Classify receive operations as `Epistemic` effect (read-only message reception)
3. Compute join of all operation effects including receive
4. Track effect in workflow type

### Property Requirements

```rust
// Receive workflow effect
let workflow = Workflow::Receive { mode, arms, is_control, span };
assert_eq!(workflow.effect(), Effect::Epistemic);

// Mixed workflow with receive and set
let seq = Workflow::Seq {
    first: Box::new(receive_workflow),
    second: Box::new(set_workflow),
};
assert_eq!(seq.effect(), Effect::Operational);  // join(Epistemic, Operational)
```

## Missing Implementation

### Surface AST Integration

The `ReceiveExpr` type exists in `parse_receive.rs` but is not integrated as a `Workflow` variant:

```rust
// In surface.rs - Workflow enum needs:
Receive {
    /// Receive mode (blocking or non-blocking)
    mode: ReceiveMode,
    /// Receive arms for matching messages
    arms: Vec<ReceiveArm>,
    /// Whether this is a control receive
    is_control: bool,
    /// Source span
    span: Span,
}
```

### Effect Computation

```rust
// In Workflow::effect() method:
Workflow::Receive { arms, .. } => {
    // Receive is Epistemic (read-only observation)
    // Join with effects of all arm bodies
    arms.iter()
        .map(|arm| arm.body.effect())
        .fold(Effect::Epistemic, |a, b| a.join(b))
}
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_receive_effect_is_epistemic() {
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![],
        is_control: false,
        span: Span::new(0, 0, 1, 1),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_receive_with_arms_effect() {
    // Receive with operational body should be Operational
    let arm = ReceiveArm {
        pattern: StreamPattern::Wildcard,
        guard: None,
        body: Workflow::Set { /* ... */ },
        span: Span::new(0, 0, 1, 1),
    };
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![arm],
        is_control: false,
        span: Span::new(0, 0, 1, 1),
    };
    assert_eq!(workflow.effect(), Effect::Operational);
}
```

### Step 2: Verify RED

Expected: FAIL - Receive variant not in Workflow enum

### Step 3: Implement (Green)

1. Add `Receive` variant to `Workflow` enum
2. Move/integrate `ReceiveMode`, `StreamPattern`, `ReceiveArm` from `parse_receive.rs` to `surface.rs`
3. Add effect computation for `Receive` in `Workflow::effect()`

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: add Receive workflow variant with effect tracking (TASK-108)"
```

## Completion Checklist

- [x] `Receive` variant added to `Workflow` enum
- [x] `ReceiveMode`, `StreamPattern`, `ReceiveArm` types added to surface AST
- [x] Effect classification for receive (Epistemic)
- [x] Effect computation joins arm body effects
- [x] Tests pass (7 new tests for receive effect tracking)
- [x] `cargo fmt` clean
- [x] `cargo clippy` clean
- [x] CHANGELOG.md updated

## Estimated Effort

4 hours

## Dependencies

- TASK-090 (parse-receive) - Already complete

## Blocked By

Nothing

## Blocks

- TASK-109 (Obligation checking)
