# TASK-790: Diagnostics, Negative Tests, and Non-Interference Coverage

## Status: 📝 Planned

## References

- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [PLAN-105](../PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-020](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md)

## Dependencies

TASK-786, TASK-787, TASK-788, and TASK-789.

## Objective

Harden user-visible diagnostics and prove SPEC-A does not change unrelated language behavior.

## Requirements

1. Add duplicate identity diagnostics.
2. Add missing summary diagnostics.
3. Add private type leak diagnostics.
4. Add constructor visibility diagnostics, alias import diagnostics, missing `pub use` target diagnostics, constructor-only import diagnostics, and private type leak-through-public-signature diagnostics.
5. Add snippet fallback diagnostics.
6. Add negative tests for deferred `type fn` and `sealed type domain` syntax.
7. Run non-regression coverage for ADTs, imports, interfaces, associated types, workflows, capabilities/resources, do, and comprehensions. At minimum, run affected crate tests plus `cargo test --all` when feasible; document any unrelated failures with exact commands.
8. Include Phase 108 TASK-777 workflow summary import/export regressions in the required non-interference set. Tests must prove ordinary-type summary work does not clear `InlineCallable.workflow_summary`, `Workflow.imported_workflow_summaries`, TypeEnv public workflow-summary bindings, or imported-summary origins.
9. Preserve Phase 108 reference-only example boundaries: supported `do:Workflow` and `[...]: Workflow` examples should remain executable, while reference-only algebra/lift source-file examples must keep their explicit reference-only expectations until their separate typed-elaboration follow-up exists.

## Verification

- [ ] Diagnostics include module/source context.
- [ ] Deferred features remain unsupported.
- [ ] Existing unrelated behavior is preserved or explicitly documented with exact commands and failures, if any.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
