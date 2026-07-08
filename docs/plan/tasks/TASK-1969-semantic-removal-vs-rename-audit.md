# TASK-1969: Semantic Removal Vs Rename Audit

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Audit Phase 201 cleanup work for cases where deprecated functionality was only renamed to target
vocabulary instead of removed, refactored into a real target Ash mechanism, or justified as a
target implementation detail.

This task exists because Phase 201's objective is code and documentation cleanup, not vocabulary
normalization. Passing stale-token gates is insufficient when old workflow/tower/capability
mechanisms remain alive under names such as entry, application, computation, callable, registry,
bridge, adapter, shim, fallback, or compatibility.

## Requirements

- Base the audit on Phase 200's audit/removal model:
  [TASK-1952](TASK-1952-legacy-deprecated-form-audit.md),
  [TASK-1958](TASK-1958-old-syntax-removal-demotion.md), and
  [PLAN-200](../PLAN-200-TOOLING-AND-MIGRATION-POLISH.md).
- Review current Phase 201 plan/tasks/evidence, especially TASK-1961 through TASK-1968 and
  `docs/plan/audits/AUDIT-201-deprecated-functionality-removal.md`.
- Create or extend an audit artifact with a dedicated semantic-removal section. Preferred path:
  `docs/plan/audits/AUDIT-201-semantic-removal-vs-rename.md` unless the implementation chooses to
  integrate the tables directly into AUDIT-201.
- For every Phase 201 cleanup slice, classify whether it:
  - deleted stale behavior,
  - refactored behavior into a target Ash primitive,
  - renamed stale code while preserving behavior,
  - kept target-justified implementation detail,
  - or requires a follow-up deletion/refactor plan.
- Audit high-risk mechanisms where target Ash should be ordinary functions with effect rows rather
  than separate entry/workflow systems:
  - runtime callable registries and `Workflow::Call` / `Stmt::Call` execution,
  - child workflow/spawn registries and instance carriers,
  - workflow/application ids, reports, admission carriers, and retained provenance,
  - TCIR/AMIR workflow artifact carriers,
  - parser/lowering workflow form carriers,
  - compatibility shims, fallback paths, bridges, adapters, and generated test scaffolds,
  - docs/reference pages that explain old mechanisms as current.
- For every retained mechanism, cite the target spec or plan that proves it is target Ash.
- For every unproven retained mechanism, assign TASK-1970 ownership and required deletion/refactor
  proof.

## Required Audit Tables

The audit artifact must include these tables:

| Table | Required columns |
|-------|------------------|
| Phase 201 Slice Review | slice/task, changed files, claimed cleanup, removed behavior proof, rename-only risk, decision |
| Retained Mechanism Inventory | mechanism, current names, old semantic origin, target replacement, keep/delete/refactor decision, owner |
| Cosmetic Rename Suspects | old name, new name, behavior preserved, why target justification is weak, required proof |
| Target Function Unification Risks | surface/runtime path, separate entry/workflow behavior, expected function/effect-row replacement, test gap |
| Documentation Staleness Risks | doc/reference path, stale concept, target doc replacement, gate needed |
| Test Adequacy Review | test/gate, what it proves, what it does not prove, required stronger evidence |

## TDD / Evidence Steps

1. Add a failing audit row or gate expectation for at least one known rename-only suspect, such as
   callable workflow/entry registry behavior that may duplicate target function execution.
2. Create the semantic-removal audit artifact and classify the known suspect.
3. Expand the audit across all Phase 201 evidence and high-risk surfaces.
4. Confirm every rename-only or weakly justified mechanism is either:
   - converted to a deletion/refactor owner in TASK-1970, or
   - proven as target-justified with spec and test evidence.
5. Run docs/index verification and record evidence in this task file.

## Completion Checklist

- [x] Semantic-removal audit artifact exists.
- [x] Every Phase 201 cleanup slice is reviewed for behavior deletion vs cosmetic rename.
- [x] All high-risk retained mechanisms are classified.
- [x] Every retained mechanism has a target-spec citation or TASK-1970 owner.
- [x] Tests/gates are reviewed for whether they prove stale functionality is gone, not merely
      renamed.
- [x] Documentation risks are assigned cleanup owners.
- [x] `CHANGELOG.md` records the audit addition.
- [x] Docs/index verification passes.

## Evidence

- Added
  [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md),
  with the required slice review, retained-mechanism inventory, cosmetic rename suspects, target
  function unification risks, documentation staleness risks, and test adequacy review.
- The audit classifies green Phase 201 token gates as necessary but insufficient proof, and assigns
  unproven retained mechanisms to TASK-1970 workstreams.
- Verification recorded in this change:
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  `git diff --check`.
