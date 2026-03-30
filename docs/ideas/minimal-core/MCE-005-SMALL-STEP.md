---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [semantics, small-step, transitions, reduction]
---

# MCE-005: Small-Step Semantics

## Problem Statement

Big-step semantics describes final results but doesn't model intermediate states, concurrency interleaving, or step-by-step execution. Small-step semantics defines single reduction steps, enabling reasoning about:

- Concurrent execution interleaving
- Step-level debugging
- Fairness and progress
- Alignment with actual interpreter execution

This exploration develops the small-step semantics for Ash.

## Scope

- **In scope:**
  - Transition relation definition (e.g → e')
  - Configuration structure (expression + state)
  - Reduction rules for each IR form
  - Evaluation contexts / congruence rules

- **Out of scope:**
  - Abstract machine implementation
  - Optimized reduction strategies
  - Distributed execution model

- **Related but separate:**
  - MCE-004: Big-step alignment (may inform small-step)
  - MCE-006: Small-step ↔ IR execution alignment

## Current Understanding

### What we know

- Small-step is finer-grained than big-step
- Each step transforms configuration to configuration
- Concurrency requires interleaving semantics
- Evaluation contexts (or congruence rules) handle sub-term reduction

### What we're uncertain about

- What exactly is in the configuration? (just expression? expression + store? + capability set?)
- How are obligations tracked across steps?
- How do we handle blocking (e.g., waiting for async result)?
- Congruence rules vs explicit contexts

## Configuration Design

Candidate configuration structures:

### Option 1: Minimal
```
C ::= e  (just the expression)
```

**Pros:** Simple
**Cons:** Can't track obligation state, effects, or store

### Option 2: With Obligation State
```
C ::= ⟨e, O⟩
  where O is current obligation set
```

**Pros:** Can check obligation discharge
**Cons:** Still lacks store/environment

### Option 3: Full (Recommended)
```
C ::= ⟨e, ρ, O, S⟩
  where e is expression
        ρ is environment (variable bindings)
        O is obligation set
        S is store (mutable state, if any)
```

**Pros:** Comprehensive
**Cons:** More complex

## Transition Categories

| Category | Description | Example |
|----------|-------------|---------|
| **Computation** | Actual reduction | `(λx.e) v → e[v/x]` |
| **Congruence** | Reduce sub-term | `e1 + e2 → e1' + e2` if `e1 → e1'` |
| **Communication** | Async message | `spawn(w)` creates new thread |
| **Synchronization** | Wait for result | `await h` when handle complete |

## Key Rules to Define

### Sequential Rules
- Let: `let x = v in e → e[v/x]`
- If-True: `if true then e1 else e2 → e1`
- If-False: `if false then e1 else e2 → e2`

### Concurrency Rules
- Par-Left: `par { e1 | e2 } → par { e1' | e2 }` if `e1 → e1'`
- Par-Right: `par { e1 | e2 } → par { e1 | e2' }` if `e2 → e2'`
- Par-Done: `par { v1 | v2 } → (v1, v2)`

### Async Rules
- Spawn: `spawn(w) → h` (returns handle, creates new thread)
- Await-Progress: `await h` when h not ready → stuck (or retry)
- Await-Complete: `await h` when h = v → v

### Capability Rules
- Act: `act c.m(args) → effect + return value`
- Observe: `observe c.m(args) → pure value`

## Open Questions

1. Do we need a labeled transition system (LTS) for action traces?
2. How do we model fairness in par/spawn interleaving?
3. Should small-step model stack frames explicitly?
4. How does suspension/yield appear in small-step?
5. Do we need different semantics for different schedulers?

## Relationship to Big-Step

Ideally: `e →* v` in small-step ⟺ `e ⇓ v` in big-step

This needs to be proven (or at least argued) once both are defined.

## Related Explorations

- MCE-004: Big-step alignment (consistency check)
- MCE-006: Small-step ↔ IR execution
- MCE-007: Full layer alignment

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need small-step for execution alignment |

## Next Steps

- [ ] Choose configuration structure
- [ ] Define evaluation contexts or congruence rules
- [ ] Draft rules for all IR primitives
- [ ] Check consistency with big-step
