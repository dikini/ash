---
status: drafting
created: 2026-03-30
last-revised: 2026-03-30
related-plan-tasks: []
tags: [ir, minimal-core, forms, elimination]
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
- Some forms may be sugar (Seq as nested Let)

### What we're uncertain about

- Complete inventory of current IR forms
- Which forms are actually primitive vs derived
- Performance implications of elimination
- Semantics preservation under rewriting

## Candidate Forms for Elimination

| Form | Current Status | Candidate For | Expressible As | Confidence |
|------|----------------|---------------|----------------|------------|
| `Seq` | Body construct | Elimination | Nested `Let` | Medium |
| `Par` | Body construct | Keep (primitive) | — | High |
| `Spawn` | Operation | Keep (primitive) | — | High |
| `Call` | Operation | Keep (primitive) | — | High |
| `Observe` | Effect | Review | `Act` with pure capability? | Low |
| `Let` | Binding | Keep (primitive) | — | High |
| `Match` | Control flow | Review | Nested `If` + destructuring? | Medium |
| `If` | Control flow | Keep (primitive) | — | High |

## Analysis

### Seq as Nested Let

```
-- Current: seq(e1, e2)
Seq(e1, e2)

-- Potential: let _ = e1 in e2
Let("_", e1, e2)
```

**Pros:** One fewer form
**Cons:** Loses explicit sequencing intent, may affect effect ordering

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

Eliminated/sugar:
- Seq → Let
- Match → If + primitive destructuring (TBD)

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

## Next Steps

- [ ] Inventory all current IR forms from codebase
- [ ] Document semantics of each form
- [ ] Prototype eliminations in test cases
- [ ] Measure impact on example programs
