# TASK-469: Update Docs and Examples for Capability Call Sugar and Split Dispatch

## Status: ✅ Complete

## Description

Update active docs, examples, and tutorials to use the new operational call model, including
symbolic capability calls, explicit `provider:action(...)`, and the split provider/action runtime
contract.

## Specification Reference

- [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- [TASK-468](TASK-468-engine-provider-split-dispatch.md)

## Dependencies

- ✅ [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- ✅ [TASK-468](TASK-468-engine-provider-split-dispatch.md)

## Requirements

1. Update tutorials/examples to show the new operational call sugar and explicit qualified form.
2. Remove or revise docs that still imply one overloaded action name for ACT execution.
3. Keep long-term module-facing examples aligned with symbolic capability names such as
   `io::fs_read(...)`.
4. Update planning/index/changelog surfaces as required.
5. Ensure the normative long-term contract is described in `docs/spec/`; `docs/design/` and
   `docs/plan/` may explain or justify the behavior but must not be the only place that defines it.

## Implementation Notes

- Prefer symbolic capability-call examples for library/module-facing user documentation.
- Use explicit `provider:action(...)` where low-level or debugging-oriented examples benefit from
  showing the resolved target directly.
- If any active tutorial or API doc still requires readers to infer behavior from design/plan docs,
  this task is not complete.

## Completed Work

### Files Updated

1. **CHANGELOG.md** - Added Phase 70 completion entry documenting split dispatch and call sugar
2. **docs/plan/PLAN-INDEX.md** - Marked Phase 70 tasks (TASK-463 through TASK-470) as complete
3. **README.md** - Updated language example to show new symbolic and explicit call forms
4. **examples/support_ticket.ash** - Updated to demonstrate new capability call sugar
5. **examples/simple_workflow.ash** - Updated to show symbolic and provider:action calls
6. **tests/workflows/support_ticket.ash** - Updated for consistency with examples
7. **docs/spec/SPEC-022-WORKFLOW-TYPING.md** - Updated example patterns to use new call forms
8. **docs/spec/SPEC-023-PROXY-WORKFLOWS.md** - Updated proxy examples to use new call sugar

### New Surface Syntax Documented

- Symbolic capability calls: `capability(args)`, `capability(args) when guard`
- Explicit provider:action calls: `provider:action(args)`, `provider:action(args) when guard`
- Legacy `act ...` forms remain compatible but lower to same split contract

### Key Points

- Split dispatch: `lookup(provider) -> execute(action_name, args)`
- Core `Workflow::Act` now carries explicit `provider_name` and `action_name` fields
- All user-facing documentation reflects the new operational call model

## Completion Checklist

- [x] tutorial/example surfaces updated
- [x] stale overloaded-name wording removed from active docs
- [x] symbolic and explicit forms both documented
- [x] planning/reporting surfaces updated as needed
- [x] active user-facing docs no longer require `DESIGN-016` / `PLAN-016` to understand the final
      contract
