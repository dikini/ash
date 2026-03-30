---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [small-step, ir, execution, alignment, interpreter]
---

# MCE-006: Align Small-Step Semantics with IR Execution

## Problem Statement

Small-step semantics must correspond to actual IR execution. This exploration ensures the theoretical small-step rules can be efficiently and correctly implemented by the interpreter.

Gap to close: Theory (small-step) ↔ Practice (interpreter execution)

## Scope

- **In scope:**
  - Mapping small-step transitions to interpreter operations
  - Identifying implementation challenges in small-step rules
  - Ensuring small-step configurations match runtime structures
  - Defining the abstract machine implied by small-step

- **Out of scope:**
  - Optimized JIT compilation
  - Backend code generation
  - Distributed execution

- **Related but separate:**
  - MCE-005: Small-step semantics development
  - MCE-007: Full layer alignment
  - MCE-009: Test workflows (validation)

## Current Understanding

### What we know

- Interpreter currently implements big-step-like evaluation
- IR is the interpreter's input format
- Runtime has: stack, capability registry, obligation tracker
- Async execution uses some form of thread/task scheduling

### What we're uncertain about

- Does current execution match intended small-step semantics?
- What is the gap between theory and implementation?
- How are configurations represented at runtime?
- Is the interpreter already doing small-step under a different name?

## Alignment Questions

| Small-Step Concept | IR/Runtime Concept | Aligned? |
|-------------------|-------------------|----------|
| Configuration ⟨e, ρ, O, S⟩ | Stack frame + obligation set? | ❓ |
| Transition e → e' | Single evaluator step? | ❓ |
| Congruence rules | Recursive evaluation? | ❓ |
| Par interleaving | Thread scheduler? | ❓ |
| Spawn new thread | Task creation? | ❓ |
| Handle h | Task ID / Future? | ❓ |

## Potential Gaps

### Gap 1: Configuration Mismatch

Small-step may define configurations that don't match runtime structures.

**Example:** Small-step has explicit obligation set `O`, but runtime may track obligations differently.

**Resolution:** Either adjust small-step or document the mapping.

### Gap 2: Atomicity

Small-step may assume atomic steps that aren't atomic in implementation.

**Example:** `act` may involve multiple runtime operations (capability lookup, permission check, effect recording).

**Resolution:** Define small-step "atomic" as "observable at language level" not "single CPU instruction."

### Gap 3: Nondeterminism

Small-step Par rules may allow interleavings that implementation restricts.

**Example:** Small-step allows any interleaving; implementation may use cooperative scheduling.

**Resolution:** Document scheduling guarantees as implementation-defined.

## Implementation Strategy

### Option 1: Direct Implementation

Implement small-step rules directly as interpreter algorithm.

**Pros:** Faithful to semantics
**Cons:** May be inefficient (many small allocations, context switches)

### Option 2: Abstract Machine

Define an abstract machine (CEK, SECD, etc.) that corresponds to small-step.

**Pros:** Better performance characteristics
**Cons:** Extra translation layer

### Option 3: Current Interpreter + Validation

Keep current interpreter, validate against small-step.

**Pros:** Minimal change
**Cons:** May hide semantic bugs

## Proposed Approach

1. Define small-step semantics (MCE-005)
2. Define abstract machine configuration matching small-step
3. Prove/show correspondence: small-step ↔ abstract machine
4. Implement abstract machine (or show current interpreter is one)

## Open Questions

1. Is the current interpreter stack-based or continuation-based?
2. How does the runtime represent suspended workflows?
3. What is the exact capability call protocol at runtime?
4. How are effects logged/aggregated during execution?

## Related Explorations

- MCE-005: Small-step semantics
- MCE-008: Runtime cleanup (runtime structures)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need to close theory-practice gap |

## Next Steps

- [ ] Document current interpreter execution model
- [ ] Compare with small-step configuration
- [ ] Identify specific mismatches
- [ ] Design abstract machine or adaptation
