# TASK-1848: Audit operation authority boundaries

## Description

Audit target specs and implementation boundaries for operation identity, row requirements, and authority discharge.

## Requirements

- Identify current operation row parsing/typechecking/lowering support.
- Identify current admission checks for operation, resource, role, policy, evidence, and failure rows.
- Record gaps where diagnostics or names imply rows grant authority or collapse discharge families.

## Completion criteria

- [x] Audit evidence names affected source/test/spec files.
- [x] Audit distinguishes implemented substrate from remaining gaps.
- [x] Audit records the bounded implementation decision.

## Evidence

- Audited parser/typechecker/lowering/admission seams in SPEC-096b, SPEC-098c, SPEC-099b, SPEC-100, `crates/ash-engine/src/row_admission.rs`, Phase 179 row admission tests, and existing impl-qualified operation identity tests. Phase 183 implements admission-side discharge classification and diagnostics; full handler execution and row-polymorphic inference remain out of scope.

## Depends on

- TASK-1847.
