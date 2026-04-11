# TASK-501: Pure Functions Prerequisites and Parser Model

## Status: ✅ Passed

## Description

Freeze the prerequisite spec/docs work for PLAN-023 and reconcile the top-level parser model around
`ModuleFile` as the authoritative file parse result, with `Program` reserved for entry-point
loading/validation.

## Specification Reference

- [PLAN-023: Pure Functions Phase](../PLAN-023-PURE-FUNCTIONS-PHASE.md)
- [DESIGN-020: Pure Functions and the Three-Vertex Model](../../design/DESIGN-020-PURE-FUNCTIONS-THREE-VERTEX-MODEL.md)
- [SPEC-002: Surface Language](../../spec/SPEC-002-SURFACE.md)
- [SPEC-009: Modules](../../spec/SPEC-009-MODULES.md)
- [SPEC-027: Pure Functions](../../spec/SPEC-027-PURE-FUNCTIONS.md)

## Requirements

1. Reconcile the file/root split so ordinary source files parse as `ModuleFile`.
2. Reserve `Program` for executable entry-point loading/validation only.
3. Ensure the active specs consistently describe top-level definitions, module declarations, and an
   optional workflow in the file-level grammar.
4. Freeze the prerequisite spec changes needed before fn implementation starts.

## Likely Files

- Modify: `docs/plan/PLAN-023-PURE-FUNCTIONS-PHASE.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-009-MODULES.md`
- Modify: `docs/spec/SPEC-027-PURE-FUNCTIONS.md`

## TDD Steps

### Red
- Identify all active docs that still describe the old `Program { definitions, workflow }`-only root.

### Green
- Update the active docs so parser/model assumptions are consistent before implementation work lands.

## Completion Checklist

- [ ] `ModuleFile` vs `Program` split frozen in active docs
- [ ] top-level file grammar aligned across PLAN-023 / SPEC-002 / SPEC-009
- [ ] prerequisite spec wording updated consistently
- [ ] CHANGELOG.md updated if phase work is landed together
