---
status: drafting
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: []
tags: [alignment, surface, ir, semantics, interpreter, consolidation]
---

# MCE-007: Full Layer Alignment

## Problem Statement

All layers of Ash must be consistent: surface syntax → IR → big-step semantics → small-step semantics → interpreter.

This exploration consolidates the accepted big-step alignment from MCE-004 together with the later small-step and interpreter alignment work into a complete, verified stack.

## Scope

- **In scope:**
  - All five layers: Surface, IR, Big-step, Small-step, Interpreter
  - Cross-layer consistency checks
  - Resolution of conflicts between layers
  - Documentation of the complete pipeline

- **Out of scope:**
  - New feature design
  - Optimization
  - Non-semantic concerns (formatting, etc.)

- **Related but separate:**
  - MCE-004: Big-step alignment (accepted prerequisite; surface → IR → big-step contract settled)
  - MCE-005: Small-step semantics (prerequisite)
  - MCE-006: Small-step ↔ IR execution (prerequisite)

## The Five Layers

```
┌─────────────────────────────────────┐
│  Layer 1: Surface Syntax            │  User-facing Ash language
│  Example: `let x = 1 in x + 2`      │
└──────────────┬──────────────────────┘
               │ lowering
               ▼
┌─────────────────────────────────────┐
│  Layer 2: Intermediate (IR)         │  Canonical representation
│  Example: `Let("x", Lit(1), ...)   │
└──────────────┬──────────────────────┘
               │ semantics
               ▼
┌─────────────────────────────────────┐
│  Layer 3: Big-Step Semantics        │  "Evaluates to" relation
│  Example: `e ⇓ v`                   │
└──────────────┬──────────────────────┘
               │ refinement
               ▼
┌─────────────────────────────────────┐
│  Layer 4: Small-Step Semantics      │  Single-step transitions
│  Example: `e → e'`                  │
└──────────────┬──────────────────────┘
               │ implementation
               ▼
┌─────────────────────────────────────┐
│  Layer 5: Interpreter               │  Executable implementation
│  Rust code in ash-core              │
└─────────────────────────────────────┘
```

## Alignment Verification

For each construct, we need:

1. **Surface → IR:** Lowering function defined
2. **IR → Big-step:** Semantic rule defined
3. **Big-step → Small-step:** Correspondence argument
4. **Small-step → Interpreter:** Implementation matches

### Verification Matrix

| Construct | Surface→IR | IR→Big | Big→Small | Small→Interp | Status |
|-----------|------------|--------|-----------|--------------|--------|
| Let | ✅ | ✅ | ❓ | ❓ | Partial |
| If | ✅ | ✅ | ❓ | ❓ | Partial |
| Match | ⚠️ | ✅ | ❓ | ❓ | Partial |
| Par | ✅ | ✅ | ❓ | ❓ | Partial |
| Call | ✅ | ✅ | ❓ | ❓ | Partial |
| Spawn | ✅ | ✅ | ❓ | ❓ | Partial |
| Act | ✅ | ✅ | ❓ | ❓ | Partial |
| Observe | ✅ | ✅ | ❓ | ❓ | Partial |

## Remaining Full-Stack Gaps

The canonical surface → IR → big-step questions that were previously tracked under MCE-004 are now resolved and should be treated as fixed inputs to this exploration:

- `Workflow::Seq` remains primitive.
- `Expr::Match` remains primitive, and `if let` lowers to `Expr::Match`.
- `Par` effect aggregation is defined in big-step semantics via branch-effect join plus helper-backed concurrent aggregation.
- Spawn completion seals the child workflow's authoritative terminal state in `CompletionPayload`.

The remaining open work for MCE-007 is therefore cross-layer work beyond MCE-004's scope.

### Gap 1: Big-step ↔ small-step correspondence

- Show that the future small-step configuration semantics refine the accepted big-step rules.
- Make concurrency, interleaving, and fairness choices explicit enough to compare with big-step `Par` and spawn behavior.

### Gap 2: Small-step ↔ interpreter correspondence

- Document how the executable interpreter/runtime realizes the later small-step model.
- Verify that runtime scheduling, control-link handling, and completion/reporting behavior match the semantic contracts.

### Gap 3: Ongoing drift prevention

- Define how future language or runtime changes should be checked against the full five-layer stack.
- Decide whether any of those checks should become automated.

## Deliverables

1. **Alignment Document:** For each construct, the complete pipeline
2. **Correspondence Proofs/Arguments:** Why layers are consistent
3. **Gap Analysis:** What's missing or inconsistent
4. **Remediation Plan:** How to fix gaps

## Open Questions

1. Do we need formal proofs or strong arguments?
2. What level of detail for correspondence arguments?
3. Should alignment be checked automatically (e.g., via property tests)?
4. How do we handle evolution — how to keep layers aligned over time?

## Dependencies

This exploration depends on:
- MCE-002 (IR forms known)
- MCE-004 (Big-step aligned)
- MCE-005 (Small-step defined)
- MCE-006 (Execution aligned)

**Recommendation:** Don't start this until prerequisites are at least `candidate` status.

## Related Explorations

- All other MCE-* explorations feed into this

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need complete alignment view |

## Next Steps

- [ ] Wait for prerequisite explorations to mature
- [ ] Create verification matrix for all constructs
- [ ] Document correspondence arguments
- [ ] Identify critical gaps for remediation
