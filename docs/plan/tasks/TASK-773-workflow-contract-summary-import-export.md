# TASK-773: Workflow Contract Summary Import/Export

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)

## Objective

Preserve enough workflow type and contract summary metadata across modules for first-class workflow composition.

## Requirements

1. Audit current module export/import carriers for workflow signatures and metadata loss.
2. Extend carriers to preserve workflow parameter types, return type, public `Workflow<A>` type, admission envelope summary, and failure/report/provenance summary.
3. Reject imported workflow values used in first-class composition if required summaries are absent.
4. Add tests covering imported workflow values in `do:Workflow` and `[...]: Workflow`.
5. Do not expose private body internals unnecessarily.

## TDD Steps

1. Write failing module import test where imported workflow is used in `do:Workflow`.
2. Write failing negative test for opaque/missing summary.
3. Implement summary propagation through parser/engine/typechecker carriers.
4. Run module import/export tests.

## Verification

- [ ] Imported workflows preserve `Workflow<A>` type.
- [ ] Imported workflow summaries support coverage checking.
- [ ] Missing summaries produce diagnostics.
- [ ] Private body details are not required in public summaries.
- [ ] CHANGELOG.md updated.
