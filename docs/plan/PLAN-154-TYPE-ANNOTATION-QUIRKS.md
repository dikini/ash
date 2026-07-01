# PLAN-154: Fix Type Annotation Quirks in fn Expressions with Imported Types

**Status:** ✅ Complete
**Spec:** [SPEC-090: Type Annotation Quirks](../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)
**Amends:** [PLAN-151](PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) (TASK-1511), [PLAN-152](PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)
**Builds on:** [ASSESSMENT-002](../assessments/ASSESSMENT-002-TYPE-ANNOTATION-QUIRKS.md)
**Task range:** TASK-1540 through TASK-1544

## Goal

Fix the type system limitation where imported types cannot be used in local type definitions, `fn` return type annotations, and record field types. This unblocks modular type design, smart constructors, and cross-module type composition.

## Core Design

Two-pass type processing:
```
Pass 1: Import Resolution — Parse `use` statements, register imported types in TypeEnv
Pass 2: Type Definition Processing — Process local `type` definitions with imported types available
Pass 3: Expression Type Checking — Typecheck expressions with full type environment
```

## Non-Goals

- No changes to runtime value representation
- No changes to builtin dispatch
- No new syntax (uses existing `use` and `type` syntax)

## Decision Gates

| Gate | Decision | Owner task |
|---|---|---|
| D1 | Parser collects imports before type definitions | TASK-1540 |
| D2 | TypeEnv registers imported types early | TASK-1541 |
| D3 | Type name resolution checks imported types | TASK-1542 |
| D4 | Diagnostics for type inference leakage | TASK-1543 |
| D5 | Closeout with verification | TASK-1544 |

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1540](tasks/TASK-1540-parser-import-first-pass.md) | Modify parser to collect imports before type definitions | ✅ Complete |
| [TASK-1541](tasks/TASK-1541-typeenv-imported-type-registration.md) | Modify TypeEnv to register imported types before local types | ✅ Complete |
| [TASK-1542](tasks/TASK-1542-type-name-resolution-imported.md) | Update type name resolution to check imported types | ✅ Complete |
| [TASK-1543](tasks/TASK-1543-type-inference-leakage-diagnostics.md) | Add diagnostics for type inference leakage | ✅ Complete |
| [TASK-1544](tasks/TASK-1544-phase-154-closeout.md) | Close out Phase 154 with verification and documentation | ✅ Complete |

## Implementation Order

1. TASK-1540: Parser changes (foundation)
2. TASK-1541: TypeEnv changes (depends on parser)
3. TASK-1542: Resolution changes (depends on TypeEnv)
4. TASK-1543: Diagnostics (depends on resolution)
5. TASK-1544: Closeout

## Verification Strategy

Every task must include:
- Focused Rust tests for the changed component
- Integration tests for cross-module type usage
- `cargo fmt --check`, `cargo test`, `cargo clippy` gates
- `git diff --check`

## Closeout Criteria

- All TASK-1540 through TASK-1543 tasks complete
- SPEC-090, PLAN-154, and PLAN-INDEX agree on scope/status
- No regressions in existing type tests
- CHANGELOG.md records the fix
- Phase 151 tasks updated with new dependencies

## Notes

This phase unblocks Phase 151's TASK-1511 by enabling:
- `Strategy<T>` to reference imported `GenContext`
- `fn` return annotations with imported types like `Strategy<Int>`
- Smart constructors for opaque types

The implementation landed in the engine/module-loader semantic-summary boundary: imported type names are registered before local type validation, callable-signature private types become opaque public identities, and constructor misuse remains rejected without changing runtime value representation.
