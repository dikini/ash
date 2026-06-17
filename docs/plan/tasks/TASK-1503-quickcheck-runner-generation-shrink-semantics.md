# TASK-1503: QuickCheck runner generation and shrink semantics

## Status: ✅ Complete

## Description

Wire the runner to execute ordinary strategy generation, split per-parameter contexts, stop at first failure, preserve failure classes during shrinking, and report generator/shrinker errors per SPEC-087.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1501: QuickCheck with-block override parser and typechecker support (planned)
- 📝 TASK-1502: QuickCheck combinators, recursion, and weights (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Generate one value per parameter context using deterministic split paths.
2. Stop at the first execution failure and shrink it.
3. Accept shrink candidates only when they reproduce the original execution failure class.
4. Record mismatched/erroring shrink candidates without accepting them.
5. Treat `gen(ctx)` errors as generator errors with repro context and no shrink.
6. Preserve original property failure when shrink errors by default.

### Property Requirements

- Same seed/schedule/source identity reproduces generated bindings.
- Failure-class-preserving shrink never changes property_false into runtime_error as final minimal counterexample.
- Shrink candidate order is preserved exactly and duplicates are not removed.

## TDD Steps

### Step 1: RED runner fixtures

Add fixtures for property_false, runtime_error, timeout if supported, generator_error, shrink_error, duplicate shrink candidates, and mismatched shrink classes.

### Step 2: GREEN runner execution

Wire ordinary strategy execution and shrink loop semantics.

### Step 3: Repro trace tests

Assert traces include per-parameter split paths, contexts, failure class, skipped candidates, and shrink status.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file, coding]
```

## Verification

```
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-cli --test test_command -- --nocapture
  - cargo clippy -p ash-cli --all-targets -- -D warnings
  - git diff --check
checklist:
  - [ ] Focused tests pass and are non-zero
  - [ ] No-Cargo final-surface fixture added where user-facing behavior changed
  - [ ] Negative leakage/fail-closed cases covered where a bridge or error path is touched
  - [ ] CHANGELOG.md updated under [Unreleased]
```

## Dependencies for Next Task

- Runner execution semantics for evidence recording in TASK-1504.
- Final counterexample/shrink behavior for TASK-1505 docs.

## Notes

Setup/evidence errors are not ordinary shrinkable counterexamples.

## Implementation Evidence

- Implemented in Phase 151 worktree `feat/phase-151-quickcheck-v1`.
- Focused verification: `cargo test -p ash-cli quickcheck -- --nocapture`.
- Broad scoped gates: `cargo check -p ash-cli`; `cargo clippy -p ash-cli --all-targets -- -D warnings`; `cargo fmt --check`; `git diff --check`.
