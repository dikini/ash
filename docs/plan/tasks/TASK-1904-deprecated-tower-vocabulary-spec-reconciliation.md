# TASK-1904: Deprecated Tower Vocabulary Spec Reconciliation

**Status:** ✅ Complete
**Phase:** [PLAN-195: Process And Concurrency Model](../PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md)

## Description

Reconcile target docs so `Act`, `Proc`, and `Workflow` are deprecated development forms and legacy
reference vocabulary only.

## Requirements

- Update relevant specs and indexes.
- Preserve the distinction between process runtime facts and workflow/normative semantics.
- Remove or fence stale claims that `Act`, `Proc`, or `Workflow` should appear in new surface,
  Core, IR, stdlib, or runtime development.
- Route new process/concurrency work through ambient computations, process row facts, and trace
  evidence.

## TDD Steps

1. Add docs checks or search evidence for stale `Act`/`Proc`/`Workflow` claims.
2. Update spec/index wording.
3. Run orientation and docs gates.

## Completion Checklist

- [x] Specs and notes consistently mark `Act`, `Proc`, and `Workflow` as historical/deprecated.
- [x] No target guidance asks for new `Act`, `Proc`, or `Workflow` surface, Core, IR, stdlib, or
      runtime forms.
- [x] Workflow-specific semantics remain out of scope.
- [x] Orientation indexes are updated if specs or notes change.
