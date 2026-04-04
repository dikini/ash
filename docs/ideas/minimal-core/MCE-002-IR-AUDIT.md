---
status: accepted
created: 2026-03-30
last-revised: 2026-04-03
related-plan-tasks: [TASK-370]
tags: [ir, minimal-core, forms, elimination]
archived: true
archive-note: Promoted to TASK-370. See docs/plan/tasks/TASK-370-ir-core-forms-audit.md for active work.
---

# MCE-002: IR Core Forms Audit

## Problem Statement

During initial Ash exploration, we may have added IR forms that are unnecessary or expressible in terms of simpler constructs. This exploration inventories all current IR forms and identifies candidates for elimination or consolidation.

Goal: A minimal but sufficient IR for the execution environment.

## Scope

- **In scope:**
  - All current IR forms in `ash-core`
  - Expressibility analysis (can form X be rewritten to forms Y+Z?)
  - Cost/benefit of elimination

- **Out of scope:**
  - Surface syntax features (handled separately)
  - Optimization passes
  - Backend-specific forms

- **Related but separate:**
  - MCE-004: Big-step semantics alignment
  - MCE-007: Full layer alignment

## Current Understanding

### What we know

- IR is the canonical representation after lowering from surface syntax
- Current forms include: Let, If, Match, Call, Spawn, Par, Seq, Act, Observe, Return, etc.
- Some forms may overlap (Par vs async Spawn)
- Some forms may be sugar (under investigation in TASK-370)

### What we're uncertain about

- Complete inventory of current IR forms
- Which forms are actually primitive vs derived
- Performance implications of elimination
- Semantics preservation under rewriting

## Candidate Forms for Elimination

| Form | Current Status | Candidate For | Expressible As | Confidence |
|------|----------------|---------------|----------------|------------|
| `Seq` | Body construct | **Keep** | Primitive sequencing (no valid rewrite to `Let`) | High |
| `Par` | Body construct | Keep (primitive) | — | High |
| `Spawn` | Operation | Keep (primitive) | — | High |
| `Call` | Operation | Keep (primitive) | — | High |
| `Observe` | Effect | Review | `Act` with pure capability? | Low |
| `Let` | Binding | Keep (primitive) | — | High |
| `Expr::Match` | Control flow | Review | Nested `If` + destructuring? | Medium |
| `If` | Control flow | Keep (primitive) | — | High |

## Analysis

### Seq Elimination Status

**Status: REJECTED** — `Seq` is a primitive form.

The hypothesis that `Seq(a, b)` could rewrite to `Let { pattern: "_", expr: a, continuation: b }`
is **invalid** because `Workflow::Seq` composes two `Workflow`s while `Workflow::Let` expects an `Expr`.

**Conclusion (from TASK-370):** `Seq` cannot be eliminated. It is a primitive sequencing construct
required for composing workflows where the first component is not an expression.
### Match as If+Destructuring

```
-- Current: match e { pat1 => e1, pat2 => e2 }
Match(e, [(pat1, e1), (pat2, e2)])

-- Potential: if is_pat1(e) then e1[extracted] else if ...
-- Requires: is_* predicates and extractors for each pattern
```

**Pros:** Fewer forms
**Cons:** Verbose, pattern matching is fundamental to ergonomics

## Minimal Core Proposal

Essential forms for minimal execution:

1. **Values:** Literal, Variable
2. **Binding:** Let
3. **Control:** If, Call, Return
4. **Concurrency:** Par, Spawn
5. **Effects:** Act (with capability)
6. **Observation:** Observe (may merge with Act)

Elimination candidates (analysis ongoing in TASK-370):
- ~~`Seq` → Open question (no valid rewrite identified)~~ **RESOLVED:** `Seq` is primitive, kept as essential sequencing form
- `Expr::Match` → Potentially expressible as `If` + primitive destructuring (TBD)

## Open Questions

1. What is the current complete list of IR forms?
2. Which forms does the interpreter actually implement natively?
3. Can we formally define "primitive" vs "derived"?
4. What is the cost of Match elimination on code size?
5. Do we need explicit forms for obligation discharge, or is it implicit in Act/Return?

## Related Explorations

- MCE-003: Functions vs capabilities (affects Call form)
- MCE-004: Big-step semantics alignment
- MCE-005: Small-step semantics (forms must support transition rules)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Initial inventory needed |

## Task Reference

This exploration has been promoted to a formal task:
- **[TASK-370: IR Core Forms Audit](../../plan/tasks/TASK-370-ir-core-forms-audit.md)**

## Next Steps

- [x] Inventory all current IR forms from codebase (30 Workflow + 13 Expr forms identified)
- [ ] Document semantics of each form in detail (TASK-370)
- [ ] Prototype eliminations in test cases (TASK-370)
- [ ] Measure impact on example programs (TASK-370)
