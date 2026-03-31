# TASK-S57-6: Update SPEC-003/SPEC-022 with Entry Workflow Typing Contract

## Status: ✅ Complete

## Description

Update SPEC-003 and SPEC-022 with the normative typing contract for entry workflows: `main` workflow with signature `Result<(), RuntimeError>` and capability-only parameters.

## Background

Per architectural review, TASK-364 assumes:

- Entry workflow named `main`
- Return type exactly `Result<(), RuntimeError>`
- Only capability parameters

This typing contract is now grounded normatively in SPEC-022, with a supporting cross-reference in SPEC-003.

## Resolved Design

- The canonical entry-workflow rule lives in SPEC-022 as a specialized workflow-typing judgment.
- SPEC-003 adds a short ownership/cross-reference note rather than duplicating the full rule.
- The designated entry workflow identifier must be exactly `main`.
- The designated entry workflow return type must be exactly `Result<(), RuntimeError>`.
- The entry workflow may declare zero or more parameters, but every parameter type must be a
  usage-site capability type of the form `cap X`.
- Entry-workflow effects remain inferred from the workflow body; S57-6 adds no special effect
  annotation rule.
- Typing failures include wrong entry name, wrong return type, and any non-capability parameter.

## Requirements

Update SPEC-003 and SPEC-022 with:

1. **Entry workflow identifier**: Named `main`
2. **Return type constraint**: Must be `Result<(), RuntimeError>`
3. **Parameter constraints**: Only capability types allowed
4. **Typechecking rule**: How entry workflow type is verified

## Acceptance Criteria

- [x] Entry workflow name constraint specified
- [x] Return type constraint specified (exactly `Result<(), RuntimeError>`)
- [x] Parameter constraint specified (capabilities only)
- [x] Typechecking judgment/rule defined
- [x] Error cases specified (wrong name, wrong return, non-cap params)
- [x] 57B TASK-364 can implement against this spec

## Related

- SPEC-003: Type system
- SPEC-022: Workflow typing
- SPEC-005: CLI (may reference this contract)
- MCE-001: Entry point design
- TASK-364: Main verification (now unblocked on spec grounds)
- TYPES-001: RuntimeError syntax (related)

## Est. Hours: 2-3

## Blocking

- TASK-366: CLI error messages (needs to reference spec for errors)

## Completion Summary

SPEC-022 now contains the canonical typing rule for the designated entry workflow, requiring the
identifier `main`, the exact return type `Result<(), RuntimeError>`, and zero or more parameters
whose types are all usage-site capability forms `cap X`. SPEC-003 now cross-references that
specialized judgment without duplicating it.

This resolves the remaining entry-signature ambiguity for TASK-364 and keeps runtime capability
availability checks outside pure typing.
