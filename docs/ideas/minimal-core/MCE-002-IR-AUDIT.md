---
status: accepted
created: 2026-03-30
last-revised: 2026-04-03
related-plan-tasks: [TASK-370]
tags: [ir, minimal-core, forms, elimination]
archived: true
archive-note: Promoted to TASK-370. See docs/plan/tasks/TASK-370-ir-core-forms-audit.md for the completed audit and current tracking record.
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

- The intended architecture is for IR to be the canonical representation after lowering from surface syntax, though current repository reality still includes active parser-surface/typechecker representation paths alongside the core AST
- Current forms include: Let, If, Match, Call, Spawn, Par, Seq, Act, Observe, Ret, etc.
- Some forms may overlap (Par vs async Spawn)
- Some forms are now confirmed sugar or primitive by TASK-370 (`Expr::IfLet` is sugar over `Match`; `Workflow::Seq` is primitive)

### What remains uncertain after TASK-370

- The safest migration path for eliminating duplicate carrier types in `workflow_contract.rs`
- Whether `Workflow::CheckObligation` should eventually lower to expression-level checking plus explicit workflow composition
- Which receive representation should survive as canonical (`ast.rs` vs `stream.rs`)
- The eventual cost/benefit of revisiting `Set`/`Send` as specialized capability operations

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

Current TASK-370 direction is conservative rather than aggressively eliminative.

Essential forms for minimal execution:

1. **Values:** Literal, Variable
2. **Binding:** Let
3. **Control:** If, Call, Ret, Match, Seq, Done
4. **Concurrency:** Par, Spawn, Split
5. **Effects:** Act (with capability), Observe, Decide, Orient, Propose
6. **Communication/runtime:** Receive, Send, Yield, ProxyResume, lifecycle control forms

Important TASK-370 conclusions:
- `Seq` remains a primitive sequencing form and should stay explicit in the core workflow set
- `Done` remains the terminal workflow form
- the highest-value simplification target is representation consolidation (`workflow_contract.rs`, `stream.rs`, duplicate `Effect`, and the active parser-surface/typechecker representation path) before deeper form elimination inside `ast.rs`

Elimination candidates (post-TASK-370 status):
- ~~`Seq` → Open question (no valid rewrite identified)~~ **RESOLVED:** `Seq` is primitive, kept as essential sequencing form
- `Expr::Match` → Keep for now; elimination deferred until pattern tests/extractors are more explicit
- `Expr::IfLet` → Confirmed sugar over `Match`
- `workflow_contract.rs` duplicate carriers / duplicate `Effect` / duplicate receive carriers → highest-value consolidation targets

## Remaining Follow-up Questions

1. What migration path best removes duplicate carrier types from `workflow_contract.rs` without breaking current parser/interpreter users of `workflow_contract::TypeExpr`, `Span`, and `Contract`?
2. Should `Workflow::CheckObligation` eventually lower to expression-level checking plus explicit workflow composition, or remain as a distinct workflow form?
3. Should the receive representation be normalized around `ast.rs` or `stream.rs` as the canonical carrier?
4. Are `ModuleItem` and `Definition` both still needed, or is one now effectively legacy/low-impact duplication?
5. What is the lowest-risk path for eventually revisiting `Set`/`Send` as specialized capability operations?

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
- [x] Document semantics of each form in detail (see `MCE-002-IR-AUDIT-REPORT.md`, TASK-370)
- [x] Measure impact on example programs and repository references (see `MCE-002-IR-AUDIT-REPORT.md`, TASK-370)
- [ ] Prototype elimination/consolidation changes in follow-on implementation tasks
