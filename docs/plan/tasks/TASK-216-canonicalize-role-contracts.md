# TASK-216: Canonicalize Role Contracts

## Status: ✅ Complete

## Description

Align the canonical spec set with the simplified role contract defined in
`todo-examples/definitions/roles.md`: roles carry authority and obligations, while supervision is
not part of the canonical role model.

## Specification Reference

- SPEC-001: Intermediate Representation (IR)
- SPEC-002: Surface Language
- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Reference / Design Inputs

- `todo-examples/definitions/roles.md`
- `docs/plans/2026-03-23-role-contract-simplification-design.md`

## Requirements

### Functional Requirements

1. Remove `supervises` from canonical role syntax in the surface spec
2. Define the canonical core role shape without role supervision
3. Clarify that approval-by-role remains a direct named-role policy reference
4. Clarify that verification/runtime role references are flat and do not imply hierarchy

## Files

- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Create: `docs/plans/2026-03-23-role-contract-simplification-design.md`
- Create: `docs/plans/2026-03-23-role-convergence-implementation-plan.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] canonical role syntax no longer includes supervision
- [x] canonical core role metadata defined without supervision
- [x] approval-role semantics clarified as flat named-role references
- [x] role-convergence design and implementation plan written
- [x] `CHANGELOG.md` updated

## Non-goals

- No parser implementation changes yet
- No runtime hierarchy feature redesign
- No example cleanup yet
