# TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics

## Status: ✅ Complete

## Description

Complete the workflow-form rule inventory for MCE-005 and record the canonical concurrency/blocking semantics that later formal rule writing must preserve. This is documentation/spec-planning work only. It does not implement a runtime or interpreter.

This task closes the rule-shape questions that remained after the MCE-005 scope/configuration contract was frozen:

- which canonical `SPEC-001` workflow forms the small-step corpus must cover;
- which families are structural, branching, effectful, modal, receive-oriented, or concurrent;
- which semantic boundaries remain atomic in v1;
- how `Par` interleaving, helper-backed aggregation, and blocked receive behavior are framed without inventing new user syntax.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-394: Small-Step Semantics Scope and Configuration Contract](TASK-394-small-step-semantics-scope-and-configuration-contract.md)

## Dependencies

- ✅ [TASK-394: Small-Step Semantics Scope and Configuration Contract](TASK-394-small-step-semantics-scope-and-configuration-contract.md)
- ✅ [TASK-393: Big-Step Semantics Alignment (MCE-004)](TASK-393-big-step-semantics-alignment.md)
- ✅ [TASK-350: Revise SPEC-004 to Complete Big-Step Core Semantics](TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md)

## Requirements

### Functional Requirements

1. Revise `MCE-005` so it includes an explicit rule inventory over the canonical `SPEC-001` workflow forms.
2. Record canonical rule families for at least:
   - terminal/structural flow,
   - binding/branching forms,
   - capability/policy/obligation forms,
   - modal/fallback forms,
   - receive/concurrency forms.
3. Make explicit that v1 small-step semantics remain workflow-first and keep pure expressions/patterns atomic via the existing `SPEC-004` subjudgments/helpers.
4. Record the canonical concurrency stance for `Par`:
   - branch-local progress is interleaved,
   - terminal aggregation remains helper-backed,
   - helper-owned nondeterminism/determinism boundaries from `SPEC-004` are preserved.
5. Record the canonical blocking stance for `Receive`:
   - blocking `Receive` with no selectable arm is blocked/suspended,
   - non-blocking miss is not modeled as stuckness.
6. Record explicitly what is not a canonical workflow rule family in v1, especially user-visible `await` or invented surface forms.
7. Keep the scope to docs/spec planning and semantic design only.

### Non-Functional Requirements

1. Use canonical `SPEC-001` / `SPEC-004` vocabulary.
2. Do not design a concrete scheduler, queue layout, or abstract machine.
3. Keep all references repo-relative.
4. Mark complete only if the updated corpus is coherent with MCE-005, MCE-006, and MCE-007.

## Completion Notes

- Expanded `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md` from a backbone-only closure note into a mature planning/design artifact with an explicit canonical workflow rule inventory.
- Recorded the v1 atomic boundaries explicitly: expressions, patterns, receive selection, parallel aggregation, provenance/obligation helpers, and spawn/completion helper contracts remain atomic.
- Recorded the canonical concurrency stance for `Par` as interleaving plus helper-backed terminal aggregation, preserving `SPEC-004` helper contracts and determinism boundaries.
- Recorded `Receive` blocking as blocked/suspended rather than stuck.
- Explicitly fenced non-canonical drift items out of the rule inventory, including user-visible `await`.

## Completion Checklist

- [x] `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md` updated with a canonical workflow rule inventory
- [x] v1 atomic boundaries recorded explicitly
- [x] `Par` concurrency/interleaving stance recorded coherently with `SPEC-004`
- [x] `Receive` blocking contract recorded as blocked/suspended rather than stuck
- [x] non-canonical surface/runtime drift items fenced out
- [x] downstream references to MCE-006 / MCE-007 kept coherent

## Files

- Modify: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Reference: `docs/spec/SPEC-001-IR.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Reference: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`

## Non-goals

- No Rust runtime/interpreter changes
- No new canonical workflow syntax
- No fairness proof or scheduler implementation design
