# TASK-1810: Resolve impl-qualified operation row identities

## Status: ✅ Complete

## Description

Add bounded operation identity resolution for source row items using the target sort/impl model. Phase 177 accepts impl-qualified row identities such as `F::read` and `PosixFs::read` where the live resolver can prove them, and fails closed for ambiguous or interface-qualified identity that would overclaim target behavior.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [NOTE-022: Effects as Interfaces](../../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- [NOTE-025: Effect Identity via Sorts and Impls](../../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)

## Dependencies

- ✅ TASK-1809 surface row parser carriers complete.
- ✅ TASK-1807 audit identifies current interface/impl summary sources.

## Requirements

### Functional Requirements

1. Add an operation-row identity carrier that distinguishes unresolved source path, abstract impl identity (`F::read`), and concrete impl identity (`PosixFs::read`).
2. Reuse existing interface/impl/module summary data where possible instead of creating a separate resolver.
3. Accept impl-qualified identities only when the impl type or type parameter identity is visible and has an operation member from the relevant interface sort.
4. Reject interface-qualified operation row identities when they would collapse multiple impl identities.
5. Preserve unresolved identities as diagnostics, not authority grants.
6. Add positive tests for local impl-qualified rows and generic `F: Fs` rows if the current type parameter infrastructure can prove them.
7. Add negative tests for unknown impl type, unknown operation, interface-qualified row identity, and same operation name across multiple impl identities.

### Property Requirements

- Operation identity resolution must not search runtime handlers or providers.
- Operation identity resolution must not install or grant authority.
- Ambiguous or unsupported identity fails closed before Core lowering.

## TDD Steps

### Step 1: Write failing identity tests

Add parser/engine or typechecker tests according to the seam audit. Include positive `PosixFs::read` and negative `Fs.read`/unknown identity cases.

### Step 2: Verify RED

Run the focused test target and confirm failures are identity-resolution failures.

### Step 3: Implement bounded identity carriers/resolution

Thread resolved operation row identity through the parsed surface or module validation layer without broad workspace graph overreach.

### Step 4: Verify GREEN

Run focused tests plus the affected crate test suite.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - git diff --check
checklist:
  - [x] Impl-qualified operation identity is represented.
  - [x] Interface-qualified operation identity fails closed where ambiguous.
  - [x] Unknown impl/operation diagnostics are precise.
  - [x] Resolution does not install provider authority.
```

## Dependencies for Next Task

This task feeds TASK-1811 and TASK-1814.

## Completion Evidence

- Added `OperationRowIdentityResolution` with concrete, abstract, interface-qualified, unknown-target, and unknown-method outcomes backed by registered typechecker interface/impl evidence.
- Added fail-closed diagnostics E182 through E184 for interface-qualified operation rows, unknown impl targets, and unknown impl operations.
- Added focused TASK-1810 typechecker tests covering `PosixFs::read`, generic `F::read` under `F: Fs`, interface-qualified `Fs::read` across multiple impl identities, unknown impl type, and unknown operation.
