# TASK-913: Add or select the canonicalization API consumed by pattern typing

## Status: ✅ Complete

## Description

Add or select the canonicalization API consumed by pattern typing

## Specification Reference

- [SPEC-068](../../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
- [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)

## Dependencies

- ✅ SPEC-068: spec packet exists
- ✅ PLAN-117: implementation plan exists
- Depends on TASK-912 audit gate replacing this task's fail-closed verification guard

## Requirements

1. Add or select the canonicalization API consumed by pattern typing.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- `crates/ash-typeck/src/type_env.rs`: add the `TypeEnv` pattern-specific canonicalization API and result type.
- `crates/ash-typeck/src/types.rs`: use existing type carriers only; add no runtime ADT layout changes.
- `crates/ash-typeck/tests/task_913_pattern_canonicalization_api.rs`: create focused API tests for direct ADTs, transparent aliases, selected associated projections, non-concrete ADT type arguments, and typed blocked boundaries for unresolved projections and other non-matchable forms.
- Audit reference: [TASK-912 pattern canonicalization audit gate](../audits/TASK-912-pattern-canonicalization-audit-gate.md).

## TDD / Execution Steps

1. Re-read the referenced SPEC and this task file.
2. Add the smallest failing parser/typechecker/core/engine/doc test that proves this task's boundary.
3. Implement only this task's boundary; do not import later-task semantics.
4. Run the focused command set recorded in this task after the audit gate replaces any failing placeholder.
5. Update this task status, the owning PLAN row, PLAN-INDEX, and CHANGELOG only after verification evidence is fresh.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 16
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_913_pattern_canonicalization_api -- --nocapture
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Focused tests are non-zero and pass
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Verification Evidence

- RED: `cargo test -p ash-typeck --test task_913_pattern_canonicalization_api -- --nocapture` failed before implementation with missing `PatternCanonical*` public reexports and missing `TypeEnv::canonicalize_type_for_pattern`.
- GREEN: `cargo test -p ash-typeck --test task_913_pattern_canonicalization_api -- --nocapture` passed with 10 tests after review remediation.
- Final gate evidence recorded during completion:
  - `cargo fmt --check` passed.
  - `git diff --check` passed.
  - `cargo check --workspace` hit the local `sccache` wrapper sandbox with `Operation not permitted`; `RUSTC_WRAPPER= cargo check --workspace` passed.

## Projection Fixture Status

The API attempts associated-projection normalization through the existing canonical IR normalizer and only succeeds when normalization yields an ordinary ADT. TASK-913 includes a concise selected/reducible projection-to-ADT success fixture using existing sealed associated-family TypeEnv APIs. Rigid/unresolved projections are covered and return typed blocked results.

## Review Remediation

- Added typed blocking for unresolved type variables anywhere inside the canonical ADT type-argument spine via `PatternCanonicalizationBlockedReason::NonConcreteTypeArgument`.
- Added focused coverage for top-level `Type::Var`, nested `Type::Var` inside direct `Result<T, String>`, nested `Type::Var` through transparent `IntResult<T>`, unknown nominal constructors, and constructor-variable applications.
- Added focused coverage for a selected associated projection reducing to `Result<Int, String>` and exposing the canonical `Result` constructor universe.
- Preserved direct concrete ADT and transparent concrete alias success cases.

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Pattern canonicalization is audit-first and must not solve under neutral computation heads.
