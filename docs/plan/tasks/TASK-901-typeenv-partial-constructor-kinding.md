# TASK-901: Elaborate holes and partial applications with kind/arity/ambiguity validation

## Status: 📝 Planned

## Description

Elaborate holes and partial applications with kind/arity/ambiguity validation

## Specification Reference

- [SPEC-066](../../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
- [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)

## Dependencies

- ✅ SPEC-066: spec packet exists
- ✅ PLAN-115: implementation plan exists
- Depends on TASK-898 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-900 completion

## Requirements

1. Elaborate holes and partial applications with kind/arity/ambiguity validation.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Audited files: `crates/ash-typeck/src/types.rs`, `crates/ash-typeck/src/lib.rs`, `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/normalizer.rs`, `crates/ash-core/src/type_ir.rs`
- Focused tests: `crates/ash-typeck/tests/task_901_partial_constructor_kinding.rs`
- Audit evidence: `docs/plan/audits/TASK-898-type-hole-audit-gate.md` rows A4, A5, and A7

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
  - cargo test -p ash-typeck --test task_901_partial_constructor_kinding
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [ ] Focused tests are non-zero and pass
  - [ ] cargo fmt --check passes
  - [ ] git diff --check passes
  - [ ] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Explicit `_` holes are not implicit currying and do not solve by inversion.
