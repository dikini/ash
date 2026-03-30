---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [semantics, big-step, surface, ir, alignment]
---

# MCE-004: Big-Step Semantics Alignment

## Problem Statement

Surface syntax, IR, and big-step semantics have drifted during initial exploration. This exploration identifies misalignments and defines the corrected relationship between these layers.

Goal: A coherent big-step semantics that accurately describes current (intended) Ash behavior.

## Scope

- **In scope:**
  - Surface syntax → IR lowering
  - IR → big-step semantics rules
  - Identifying inconsistencies or gaps
  - Documenting intended behavior

- **Out of scope:**
  - Small-step semantics (MCE-005)
  - Interpreter implementation details
  - Optimization passes

- **Related but separate:**
  - MCE-002: IR audit (what forms exist)
  - MCE-007: Full layer alignment (includes small-step)

## Current Understanding

### What we know

- Big-step semantics defines "evaluates to" relation
- Surface syntax is user-facing, expressive
- IR is simplified, canonical
- Some surface constructs lower to multiple IR forms
- Obligation tracking happens at IR level

### What we're uncertain about

- Exact surface → IR mapping for all constructs
- Whether big-step rules match current IR structure
- How effects are threaded through big-step rules
- How obligations are discharged in big-step

## Alignment Matrix

| Surface Construct | IR Form(s) | Big-Step Rule | Status |
|-------------------|------------|---------------|--------|
| `let x = e1 in e2` | `Let(x, e1, e2)` | Let rule | ✅ Aligned |
| `if e then e1 else e2` | `If(e, e1, e2)` | If-True, If-False | ✅ Aligned |
| `match e { ... }` | `Match(e, arms)` | Match rules | ⚠️ Needs review |
| `par { e1 | e2 }` | `Par(e1, e2)` | Par rule | ⚠️ Effect aggregation? |
| `seq(e1, e2)` | `Seq(e1, e2)` or `Let` | Seq rule | ❌ Unclear |
| `call(w)` | `Call(w)` | Call-Sync | ✅ Aligned |
| `spawn(w)` | `Spawn(w)` | Call-Async | ⚠️ Obligation semantics? |
| `act c.m()` | `Act(c, m, args)` | Act rule | ✅ Aligned |
| `observe c.m()` | `Observe(c, m, args)` | Observe rule | ⚠️ Effect category? |

## Identified Issues

### Issue 1: Seq vs Let

Current IR may have both `Seq` and `Let`. If `Seq` is just `Let` with ignored binding, big-step should reflect this.

**Options:**
- Keep Seq with its own rule
- Eliminate Seq, use Let in both IR and semantics

### Issue 2: Effect Aggregation in Par

Big-step `Par` rule needs to define how effects from both branches combine.

**Question:** Is the Par result effect the join of branch effects?

### Issue 3: Obligation Discharge in Spawn

Big-step `Spawn` rule needs to clarify: does the spawned workflow discharge obligations independently?

**Current understanding:** Yes, obligations are isolated per workflow instance.

### Issue 4: Match as Primitive

If Match lowers to If+destructuring, big-step semantics should define the lowering or eliminate Match as a primitive rule.

## Proposed Alignment

1. **Surface → IR:** Define explicit lowering for each construct
2. **IR → Big-step:** One rule per primitive IR form
3. **Effect propagation:** Explicit in all rules
4. **Obligation discharge:** Explicit at Call and Return

## Open Questions

1. Should big-step rules reference surface syntax or IR?
2. How are runtime errors modeled in big-step?
3. Do we need a separate "configurations" layer between IR and big-step?
4. How does capability checking interact with big-step?

## Related Explorations

- MCE-002: IR audit (determines what forms need rules)
- MCE-003: Functions vs capabilities (affects Call rule)
- MCE-005: Small-step semantics (may inform big-step cleanup)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Alignment issues identified |

## Next Steps

- [ ] Document current surface → IR lowering
- [ ] Review existing big-step rules against IR
- [ ] Identify specific mismatches
- [ ] Draft corrected big-step semantics
