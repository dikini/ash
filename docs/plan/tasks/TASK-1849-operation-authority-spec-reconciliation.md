# TASK-1849: Reconcile operation authority specs

## Description

Update target specs and indexes so the operation authority model is stated consistently.

## Requirements

- Specs state operations are interface methods.
- Specs state operation identity is impl/type-qualified (`PosixFs::read`, `F::read`).
- Specs state rows require operations and do not grant authority.
- Specs state providers/handlers discharge operation requirements, while resource, role, policy, evidence, and failure rows use separate discharge rules.
- Update orientation indexes when routing changes.

## Completion criteria

- [x] Specs do not imply operation rows grant authority.
- [x] Specs and indexes route operation authority work through current target docs.
- [x] Docs gate passes.

## Evidence

- Updated SPEC-096b, SPEC-098c, SPEC-099b, SPEC-100, SPEC-INDEX, and NOTE-INDEX to route operation authority work through Phase 183 and distinguish operation/resource/role/policy/evidence/failure discharge families.

## Depends on

- TASK-1848.
