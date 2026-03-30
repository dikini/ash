---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [alignment, surface, ir, semantics, interpreter, consolidation]
---

# MCE-007: Full Layer Alignment

## Problem Statement

All layers of Ash must be consistent: surface syntax → IR → big-step semantics → small-step semantics → interpreter.

This exploration consolidates the partial alignments (MCE-004, MCE-006) into a complete, verified stack.

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
  - MCE-004: Big-step alignment (prerequisite)
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
| Match | ⚠️ | ⚠️ | ❓ | ❓ | Needs work |
| Par | ✅ | ⚠️ | ❓ | ❓ | Needs work |
| Call | ✅ | ✅ | ❓ | ❓ | Partial |
| Spawn | ✅ | ⚠️ | ❓ | ❓ | Needs work |
| Act | ✅ | ✅ | ❓ | ❓ | Partial |
| Observe | ✅ | ⚠️ | ❓ | ❓ | Needs work |

## Known Misalignments

### Misalignment 1: Seq elimination

- **Surface:** `seq(e1, e2)`
- **IR:** May be `Seq` or `Let`
- **Semantics:** Depends on IR form
- **Resolution:** Decide on canonical IR form

### Misalignment 2: Match complexity

- **Surface:** Full pattern matching
- **IR:** `Match` form or lowered to If
- **Resolution:** Define canonical lowering or keep Match primitive

### Misalignment 3: Par effect aggregation

- **Big-step:** How are branch effects combined?
- **Small-step:** Interleaving model
- **Interpreter:** Actual scheduling
- **Resolution:** Define and verify consistency

### Misalignment 4: Async obligation isolation

- **Semantics:** Obligations isolated per workflow
- **Runtime:** How is this enforced?
- **Resolution:** Verify runtime tracks per-workflow obligations

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
