# TASK-1988: Semantic Implementation and Deprecation Audit

**Status:** Complete
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1986 and Phase 201 reconciliation

## Description

Map canonical semantic rules to Rust symbols, tests, examples, diagnostics, and runtime artifacts;
then produce evidence-led delete/fold/retain/decision tasks for conflicts and orphaned mechanisms.

## Requirements

- Use Rust language-aware symbol/reference/diagnostic tools before broad tracing.
- Reconcile TASK-1971/TASK-1972 and the Phase 201 worktree before assigning overlap.
- Distinguish semantic deletion from rename-only cleanup.
- Require behavior-level parity or absence evidence for every removal.
- Create task files before any resulting implementation work begins.

## TDD Steps

1. Define mapping and classification fixtures.
2. Audit one canonical vertical slice at a time.
3. Verify every public semantic implementation has an owner or private-machinery rationale.
4. Publish the deprecation/removal packet and run docs/index gates.

## Completion Checklist

- [x] Canonical-to-Rust/test mapping is complete for the audited programme vertical slices.
- [x] Conflicts, partial implementations, and unmapped code are explicit.
- [x] Every removal/refactor has an evidence requirement and task owner.
- [x] No Phase 201 evidence or in-flight user change is overwritten.

## Completion evidence

- [TASK-1988 semantic implementation and deprecation packet](../audits/TASK-1988-semantic-implementation-deprecation-packet.md)
  maps all eight TASK-1986 canonical owners across the surface/lowering, Core/CPS, and
  runtime-observable slices to evidence and dispositions without treating implementation as
  canonical authority. Only the Core/CPS slice used exposed rust-analyzer operations; the other
  slices record bounded `rg`/source-tracing evidence because language-aware MCP was unavailable.
- TASK-1971/TASK-1972 were reconciled against the `c9294828` Phase 201 handoff. The packet
  records that neither changed the audited Core/CPS prototype sources.
- New implementation-affecting dispositions are owned by TASK-2000 through TASK-2008. Existing
  TASK-439 is the sole canonical differential-corpus/harness owner; it was augmented rather than
  duplicated.
- Focused Core/CPS behavior evidence passed: 14 Core-to-CPS lowering tests, 17 Core
  handler/affine-checking tests, 5 handler/provider dispatch tests, and 11 CPS multiplicity tests.
- Documentation verification for this packet: `python3 tools/docs/validate_orientation_indexes.py
  --self-test`, `bash scripts/check-docs-gate.sh`, and `git diff --check`.
