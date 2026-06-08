# TASK-1046: Migrate stdlib `Monoid` to `where A: Semigroup` and reconcile examples/reference wording

## Status: ✅ Complete

## Description

Migrate stdlib `Monoid` to `where A: Semigroup` and reconcile examples/reference wording.

## Specification Reference

- [SPEC-080](../../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [PLAN-130](../PLAN-130-INTERFACE-EVIDENCE-CONSTRAINTS.md)

## Task Type

Stdlib/Docs

## Dependencies

- Depends on TASK-1039 audit gate completion and exact verification-command replacement.
- Depends on TASK-1040 through TASK-1043 completion.

## Requirements

1. Preserve SPEC-080's core rule: constrained evidence entails required evidence, but Ash does not automatically derive implementations.
2. Use “requires”, “entails”, “evidence constraint”, or “required evidence”; do not use object-hierarchy wording in user-facing docs or diagnostics.
3. Keep interface-level constraints distinct from generic impl `where` constraints.
4. Add final `std::algebra` import-path tests, not fixture-only local interface tests.
5. Prove missing required evidence fails and reverse entailment is not inferred.
6. Avoid broadening generalized proposition syntax, proof search, implementation derivation, or overlap/specialization semantics.

## File Targets

- Stdlib algebra source: `std/src/algebra/`
- Engine/module final-path tests: `crates/ash-engine/tests/`
- Type checker focused tests: `crates/ash-typeck/tests/`
- Reference/docs/status as needed: `reference/`, `docs/plan/PLAN-INDEX.md`, `CHANGELOG.md`

## TDD / Execution Steps

1. Re-read SPEC-080 and PLAN-130 before editing code.
2. Add RED final-path tests for this exact algebra relation.
3. Update only the relevant stdlib interface and tests.
4. Verify the type checker rejects missing required evidence and accepts the stdlib evidence chain.
5. Update reference/status wording only after focused tests pass.

## Dispatch

```yaml
agent: hermes
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - RUSTC_WRAPPER= cargo check --workspace
checklist:
  - [x] Focused final-path tests are non-zero and pass
  - [x] Missing required evidence has a negative test
  - [x] Reverse entailment has a negative test or is covered by TASK-1043
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Notes

This task must not create a blanket generic impl to encode the relation. The interface declaration itself owns the required evidence.

## Completion notes

- Updated `std/src/algebra/monoid.ash` so `Monoid<A>` declares `where A: Semigroup` through the final stdlib source.
- Added `crates/ash-engine/tests/task_1046_stdlib_monoid_constraint.rs` to prove the stdlib surface preserves the constraint, final `string.ash`/`list.ash` Monoid implementations discharge it via Semigroup evidence, and missing `Semigroup<String>` evidence rejects a local `Monoid<String>` impl.
- Reverse entailment remains covered by TASK-1043's directional evidence tests; this task does not add derivation, blanket impls, or proof search.
- Verification: `cargo fmt --check`; `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1046_stdlib_monoid_constraint -- --list` (3 tests); `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1046_stdlib_monoid_constraint -- --nocapture` (3 passed); `RUSTC_WRAPPER= cargo check --workspace`; `RUSTC_WRAPPER= cargo clippy -p ash-engine --test task_1046_stdlib_monoid_constraint -- -D warnings`; `git diff --check`.
