# Role Contract Simplification Design

## Goal

Align the canonical Ash role model with the forward-looking design in [todo-examples/definitions/roles.md](../../todo-examples/definitions/roles.md): a role is a governance carrier with authority and obligations, while workflow/process supervision remains a separate runtime concern.

## Context

The current repository still reflects an older mixed model:

- [SPEC-002](../spec/SPEC-002-SURFACE.md) defines `role_def` with `supervises`.
- Parser and core data structures still carry a legacy `supervises` field.
- Runtime approval handling already works mostly with flat named roles rather than true role hierarchy.
- Several examples and older materials still present supervision as part of the role contract.

This creates unnecessary ambiguity because the word `supervision` already has a separate runtime meaning around workflow control and lifecycle management.

## Design Decision

### 1. Canonical role meaning

The canonical role contract is:

- `name`
- `authority`
- `obligations`

Roles do not encode hierarchy or supervision.

### 2. Approval remains role-directed but flat

Policies may still name approval roles directly, for example `require_approval(role: admin)`, but this is a direct policy reference rather than an inheritance rule derived from role hierarchy.

### 3. Workflow supervision remains separate

Workflow/process supervision is still a runtime concern involving control authority, liveness, and recovery. It is not modeled as part of the role definition.

### 4. Implementation convergence target

The implementation target is not to invent richer runtime role hierarchy. It is to:

- remove legacy `supervises` residue from canonical surface/core role models,
- support source role definitions end-to-end in parser/lowering,
- keep runtime approval routing role-directed but flat,
- align examples and tests with the simplified contract.

## Spec Impact

This design updates the canonical specs so that:

- [SPEC-002](../spec/SPEC-002-SURFACE.md) removes `supervises` from `role_def`.
- [SPEC-001](../spec/SPEC-001-IR.md) defines the canonical core role shape without supervision.
- [SPEC-017](../spec/SPEC-017-CAPABILITY-INTEGRATION.md) and [SPEC-018](../spec/SPEC-018-CAPABILITY-MATRIX.md) explicitly treat approval-role references as flat named roles.

## Non-goals

This design does not:

- define a new organizational hierarchy feature,
- redesign obligations or policies beyond the role-adjacent clarifications above,
- add yield/resume or workflow-supervision semantics,
- require a richer runtime role object than the current implementation actually needs.

## Acceptance Criteria

This design is successful when:

1. Canonical specs no longer define role supervision.
2. Implementation plans treat `supervises` as legacy residue to remove, not as intended future behavior.
3. Approval-by-role remains available without implying hierarchy.
4. The next implementation tasks are scoped, ordered, and small enough to execute incrementally.
