# TASK-1986: Canonical Core Reconciliation and Promotion

**Status:** Planned
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1985

## Description

Reconcile the candidate grammar/type/effect/lowering/Core/CPS/runtime/conformance sources and
promote one compact, coherent canonical core.

## Requirements

- Assign one canonical owner per subject listed in PLAN-202.
- Resolve workflow-first versus target function/Core/CPS authority explicitly.
- Preserve unique historical rationale through typed links or archive manifests.
- Update conformance cases and handoff contracts with each promoted rule.
- Generate the default human and agent read paths from the manifest.

## TDD Steps

1. Add failing ownership/read-path fixtures for the known authority conflicts.
2. Reconcile one vertical slice at a time: grammar, type/effect, lowering, Core/CPS, runtime,
   observable/conformance.
3. Run executable examples or conformance fixtures for each promoted slice.
4. Regenerate indexes/packs and run documentation gates.

## Completion Checklist

- [ ] The canonical core covers all eight PLAN-202 subjects.
- [ ] No unresolved conflict is presented as canonical.
- [ ] Default agent paths exclude historical/research claims.
- [ ] Handoff and conformance artifacts cite stable rule identities.
