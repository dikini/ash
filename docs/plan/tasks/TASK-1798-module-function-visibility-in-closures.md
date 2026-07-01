# TASK-1798: Fix module-level pure function visibility inside closures

## Status: ✅ Complete

## Description

Make module-level pure functions visible to closures through the same typed/module identity path used by ordinary function calls, without treating module functions as captured capability authority.

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
| TASK-1580 | PLAN-158 | Needed power-tower lifting / pure-vs-Act distinction | Unknown until audit | Re-evaluate after target redesign | Closure lookup positive and effect-leakage negative tests |

## Requirements

### Functional Requirements

1. Add a failing test where a closure passed through a workflow calls a module-level pure helper.
2. Implement lookup/capture through typed callable identity or module environment lookup, not by blindly copying all module symbols into every closure.
3. Reject or preserve existing diagnostics for effectful/capability names that are not pure callables.
4. Document the exact runtime/typechecker responsibility boundary.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Write RED tests

Add positive and negative tests in the affected engine/interpreter/typeck test suites.

### Step 2: Trace closure creation and application

Use LSP/search to find closure environment construction and module callable lookup paths before patching.

### Step 3: Implement the minimal lookup bridge

Prefer a typed module callable environment over broad lexical capture.

### Step 4: Verify effect leakage

Add negative tests showing capability/provider names are not captured as ordinary pure functions.

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
  - cargo fmt --check
  - cargo test -p ash-interp
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
checklist:
  - [x] Positive closure/module-function test passes
  - [x] Negative effect/capability leakage tests pass
  - [x] Broad gates pass
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

If effect-row substrate is still not present in code, split this task instead of landing an authority-widening workaround.

Completion evidence: `crates/ash-engine/tests/task_1798_closure_module_function_visibility.rs` covers local sibling helper visibility, imported public callable closure access to private same-module helpers, and non-leakage of the private helper into caller runtime bindings. Runtime implementation uses shared module callable `EnvFrame`s and hidden imported module runtime callables rather than copying all module symbols into caller top-level bindings. Verification passed with `cargo fmt --check`, `cargo test -p ash-engine --test task_1798_closure_module_function_visibility -- --nocapture`, `cargo test -p ash-interp -p ash-engine -p ash-typeck --all-targets`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `git diff --check`.
