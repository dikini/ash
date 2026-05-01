# TASK-787: TypeEnv Two-Pass Registration from Semantic Summaries

## Status: ✅ Complete

## References

- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [PLAN-105](../PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-020](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md)

## Dependencies

TASK-783, TASK-785, and TASK-786.

## Objective

Consume semantic summaries in TypeEnv with two-pass declaration, validation, and representation exposure.

## Requirements

1. Declare all visible type names and canonical identities before validating bodies, using canonical identity-aware keys or alias-to-identity bindings rather than relying only on string names.
2. Validate type bodies and expose representations/constructors according to visibility; distinguish identity-only imports from full public representation exposure.
3. Preserve sibling forward-reference behavior from SPEC-030.
4. Handle placeholder upgrade conflicts explicitly with a tagged declaration/placeholder state so real empty structs and opaque summaries are not mistaken for placeholders.
5. Register imported public type identities before imported signatures are checked, and register local ordinary type definitions before interfaces/resources/functions that may mention them. The implementation should explicitly reconcile live `TypeEnv` APIs such as `register_type_identity`, `expose_type_representation`, `declare_type_name`, and `register_type` with the new summary path.
6. Register ordinary type identities before binding or checking imported Phase 108 `PublicWorkflowSummary` users, including `TypeEnv::bind_public_workflow_summary`, `TypeEnv::lookup_public_workflow_summary`, and imported `do:Workflow` / `[...]: Workflow` composition checks.
7. Add tests for imported public types, private leakage, duplicates, sibling/self/generic references, malformed generic arity where current ordinary generic metadata is sufficient to validate it, and workflow-summary preservation when imported Workflow summaries mention ordinary public type identities.

## Verification

- [x] Sibling/self/generic type references register independent of order.
- [x] Imported public type identities are available before imported callables are checked.
- [x] Placeholder behavior is explicit and tested.

## Implementation Notes

- Follow TDD for any code changes.
- Update CHANGELOG.md when the implementation task is completed.
- Run focused tests for the changed crate and broader regressions requested by PLAN-105.
