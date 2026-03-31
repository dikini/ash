# TASK-S57-6 Entry Workflow Typing Design

## Context

TASK-S57-6 closes the remaining specification gap for program entry typing. Downstream runtime verification work already assumes an entry workflow contract for `main`, but that contract is not yet stated normatively in the typing specs.

## Decision

- The normative entry-workflow rule should live primarily in `SPEC-022`, because it is a workflow-validity judgment rather than a general value-type constructor rule.
- `SPEC-003` should add a short cross-reference that entry-workflow validation is a specialized workflow-typing judgment and that pure typing does not perform runtime availability checks.
- The only valid entry workflow name is `main`.
- The entry workflow return type must be exactly `Result<(), RuntimeError>`.
- The entry workflow parameter list may be empty, or may contain one or more parameters, but every parameter type must be a usage-site capability type of the form `cap X`.
- Entry-workflow effects remain inferred from the workflow body; S57-6 does not add a special effect annotation rule.
- Wrong name, wrong return type, and any non-capability parameter are typing failures of the entry-workflow contract.

## Rationale

This keeps ordinary type formation in `SPEC-003` and places the special workflow-level contract where workflows are validated in `SPEC-022`. It also aligns with `SPEC-017`, which already defines `cap X` as the canonical usage-site capability type syntax, and it gives TASK-364 a single normative contract to verify against.

## Scope

This design updates:

- `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- `docs/spec/SPEC-022-WORKFLOW-TYPING.md`
- `docs/plan/tasks/TASK-S57-6-spec-003-022-entry-typing.md`
- `docs/plan/tasks/TASK-364-main-verification.md`
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`

It does not define the concrete `RuntimeError` ADT or CLI diagnostics wording beyond the typing-layer error categories needed to unblock follow-on tasks.
