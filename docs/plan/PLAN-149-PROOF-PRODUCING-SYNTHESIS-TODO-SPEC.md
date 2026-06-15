# PLAN-149: Proof-Producing Synthesis Todo Spec

**Status:** ⏸️ Deferred / To-Spec
**Spec:** [SPEC-085: Proof-Producing Synthesis Todo Spec](../spec/SPEC-085-PROOF-PRODUCING-SYNTHESIS-TODO-SPEC.md)
**Depends on:** [PLAN-145: Law Test Evidence Substrate](PLAN-145-LAW-TEST-EVIDENCE-SUBSTRATE.md)
**Task range:** TASK-1482 through TASK-1484
**Estimated effort:** 6h

## Overview

Record proof-producing synthesis as a separate non-test evidence family and defer implementation until symbolic/proof artifact trust boundaries are designed.

## Goals

- [ ] Document future proof-producing synthesis as deferred.
- [ ] Separate empirical `by test` evidence from symbolic/solver proof evidence.
- [ ] Define criteria for a later implementation-grade spec.

## Non-Goals

- implementation of symbolic execution
- solver integration
- proof checker
- proof-producing synthesis runtime
- changes to `ash test` beyond documentation

## Orchestrator Guidance

- Treat this as a documentation/deferred-spec packet, not an implementation phase.
- Load `ash-language-feature-spec-writing`, `software-planning`, and `verification-before-completion`.
- Do not add parser syntax, solver integrations, proof checkers, or runner behavior in this phase.
- Keep future syntax examples explicitly illustrative and non-normative.
- Update `CHANGELOG.md`, `PLAN-INDEX.md`, and relevant `reference/tools/test.md` caveats in the closeout task.

## Task Plan

| Task | Title | Estimate | Status |
|---|---|---:|---|
| [TASK-1482](tasks/TASK-1482-proof-producing-synthesis-landscape.md) | Document proof-producing synthesis landscape | 2h | ⏸️ Deferred / To-Spec |
| [TASK-1483](tasks/TASK-1483-proof-evidence-family-boundary.md) | Define future proof evidence family boundary | 2h | ⏸️ Deferred / To-Spec |
| [TASK-1484](tasks/TASK-1484-proof-producing-synthesis-deferred-closeout.md) | Close deferred todo-spec packet | 2h | ⏸️ Deferred / To-Spec |

## Decision Gates

- D1: documentation only; do not implement proof-producing synthesis.
- D2: future syntax examples remain illustrative and non-normative unless a later spec promotes them.
- D3: closeout leaves explicit to-spec checklist, not implementation tasks.

## Verification Strategy

Each deferred documentation task must include:

1. Scoped Markdown link/trailing-whitespace checks.
2. `git diff --check`.
3. A review that future syntax examples are labeled illustrative and not currently supported.
4. Confirmation that no Rust/source implementation files changed for this deferred packet.

The closeout task owns PLAN-INDEX, CHANGELOG, and reference caveat reconciliation.
