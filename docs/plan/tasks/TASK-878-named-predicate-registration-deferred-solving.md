# TASK-878: Named predicate registration and deferred solving

## Status: ✅ Complete

## Description

Register named proposition predicates and route unsupported predicate solving to explicit deferred outcomes and diagnostics.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-877 completion

## Files / Ownership

- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/parse_type_def.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/error.rs`, `diagnostic.rs`
- Test: `crates/ash-parser/tests/task_878_named_predicate_surface.rs`
- Test: `crates/ash-typeck/tests/task_878_named_predicate_registration.rs`
- Audit rows: H-AUD-PARSE-01, H-AUD-PARSE-03, H-AUD-PARSE-04, H-AUD-TYPECK-06, H-FORCE-06

## TASK-872 Binding Notes

- Parser owns raw `prop` declarations and named-predicate proposition uses; TypeEnv owns predicate identities, visibility, parameter domains, source anchors, and deferred solving outcomes.
- Only explicitly registered compiler-known builtin predicates may be satisfied in this task.
- Unknown predicates and unsupported known predicates must be distinct typed outcomes/diagnostics; do not add arbitrary proof search.

## Requirements

### Functional Requirements

1. Register predicate identities, parameter domains, visibility, and source anchors.
2. Lower named predicate uses to canonical `NamedPredicate` propositions.
3. Satisfy only compiler-known builtin predicates explicitly registered by TypeEnv.
4. Defer arbitrary named predicates with stable deferred-feature diagnostics.
5. Reject unknown predicates with source-span diagnostics.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write parser/typeck tests for predicate declaration and use lowering.

### Step 2

- Write tests for unknown predicate and unsupported predicate solving.

### Step 3

- Implement registry and deferred outcome.

### Step 4

- Verify public/private visibility data is available for TASK-879.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Focused parser/typeck tests pass.
- [x] Unknown vs unsupported-known predicate diagnostics are distinct.
- [x] No arbitrary named predicate proof search is attempted.

## Completion Notes

- Added parser and TypeEnv coverage for explicit `prop` declarations, named-predicate proposition uses, predicate identity registration, parameter-domain/visibility/source-anchor preservation, canonical `NamedPredicate` lowering, and non-interference with `pub use path::item` re-export syntax.
- TypeEnv now distinguishes unknown predicate uses from known-but-unsupported predicates: unknown source/core predicate IDs produce structured `UnknownPropositionPredicate` diagnostics, while registered opaque predicates defer as `UnsupportedNamedPredicate` with no-inversion provenance.
- Compiler-known builtin named predicates are satisfied only when explicitly registered through TypeEnv and when the application arity matches the registered predicate summary; malformed builtin applications fail closed instead of being satisfied.
- Independent review initially found direct/core unknown-ID and builtin-wrong-arity gaps; both were fixed and re-reviewed as PASS.
- Focused verification recorded: `cargo test -p ash-parser --test task_878_named_predicate_surface` (3 passed), `cargo test -p ash-typeck --test task_878_named_predicate_registration` (7 passed), `cargo test -p ash-cli commands::run::tests::test_` (10 passed), `cargo fmt --check`, `git diff --check`, and `cargo check --workspace` all passed.

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
  - git diff --check
  - cargo check --workspace
  - test -f crates/ash-parser/tests/task_878_named_predicate_surface.rs
  - cargo test -p ash-parser --test task_878_named_predicate_surface -- --list | grep -q task_878_
  - cargo test -p ash-parser --test task_878_named_predicate_surface
  - test -f crates/ash-typeck/tests/task_878_named_predicate_registration.rs
  - cargo test -p ash-typeck --test task_878_named_predicate_registration -- --list | grep -q task_878_
  - cargo test -p ash-typeck --test task_878_named_predicate_registration
checklist:
  - "[x] Task requirements are satisfied"
  - "[x] Focused verification is recorded"
  - "[x] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-878 for downstream tasks.
