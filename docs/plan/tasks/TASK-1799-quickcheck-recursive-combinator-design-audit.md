# TASK-1799: Re-audit recursive QuickCheck combinator design against live language features

## Status: ✅ Complete

## Description

Decide whether `recursive` and `recursive_with` can now be implemented as ordinary Ash combinators, or whether the API should be re-scoped to an explicit depth/threaded helper that preserves the target semantics without self-referential values.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ TASK-1795 readiness audit complete

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1511 recursive combinators | PLAN-151/TASK-1511 | Self-referential values and closure/language limits | Closure/module-helper visibility is now present; self-referential values remain absent but are avoidable | Preserve SPEC-087 public API; use fail-closed guard until the private size-descending helper design is admitted by parser/type-metadata support | Final-surface QuickCheck recursive combinator fixtures plus fail-closed execution regression |

## Requirements

### Functional Requirements

1. Read current `std/src/test/quickcheck` combinator modules and runner expectations.
2. Check live support for recursive values, closure capture, let destructors, imported type identities, list operations, and pure function bodies.
3. Record the chosen API and proof obligations before implementation.
4. Patch TASK-1800 with exact target source and tests.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Inspect stdlib and tests

Read `std/src/test/quickcheck/*.ash`, parser/typechecker tests, and TASK-1511 completion notes.

### Step 2: Evaluate API options

Compare self-referential strategy values against explicit depth-parameter helpers.

### Step 3: Write decision table

Record the accepted implementation shape, rejected options, and required final-surface tests.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
checklist:
  - [x] Decision table exists
  - [x] TASK-1800 scope patched to chosen API
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

Do not implement a recursive combinator by adding a hidden Rust fallback. The point is ordinary-Ash strategy semantics.

## Completion Evidence

- Decision table: [PHASE-176 QuickCheck recursive combinator audit](../../audit/PHASE-176-quickcheck-recursive-combinator-audit.md).
- Chosen API: keep SPEC-087 `recursive` / `recursive_with` public surface; keep the private size-descending helper recursion as the future ordinary-Ash design instead of self-referential values or a Rust fallback.
- Phase 176 implementation note: TASK-1800 landed the public API/config and a fail-closed execution guard because the private helper body still needs parser/type-metadata substrate support.
- Focused verification: `cargo run -q -p ash-cli -- check std/src/test/quickcheck/combinator.ash` and `cargo test -p ash-engine --test phase151_quickcheck_stdlib` passed during the audit/remediation cycle.
