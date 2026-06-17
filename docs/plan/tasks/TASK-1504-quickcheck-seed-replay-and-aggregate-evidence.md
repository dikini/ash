# TASK-1504: QuickCheck seed, replay, and aggregate evidence history

## Status: ✅ Complete

## Description

Implement random-by-default seed policy, external replay override, source-seed linting, exact source case budgets, run records, compatible aggregate pass history, sticky errors, nondeterminism detection, and active finding flags.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1503: QuickCheck runner generation and shrink semantics (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Default seed is random and recorded.
2. CLI/replay/external seed overrides source seed.
3. Source `seed` is allowed but lints/warns.
4. Source `cases N` is exact; CLI/project cases only fill in when source is silent.
5. Store individual run records with exact seed/cases/version/identity/outcome.
6. Aggregate compatible runs across different case counts while preserving buckets.
7. Keep compatible errors/counterexamples active until identity changes.
8. Detect same-seed divergent outcomes as nondeterminism/error.

### Property Requirements

- Positive aggregate requires no compatible counterexamples, no compatible errors, and no nondeterminism.
- Later passes do not clear sticky compatible errors.
- Counterexample and error findings coexist.
- Broad identity changes invalidate active findings without deleting historical audit records if retained.

## TDD Steps

### Step 1: RED evidence schema tests

Add JSON/unit tests for run records, seed sources, case precedence, aggregate rollups, sticky errors, coexistence, and invalidation.

### Step 2: GREEN seed/replay policy

Implement random seed default and external replay override.

### Step 3: GREEN aggregate evidence

Implement compatible aggregate computation and active finding flags.

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

- Evidence-history substrate for TASK-1505 docs and TASK-1506 closeout.
- Replay semantics for counterexamples.

## Notes

Default execution adds another run to the aggregate; it does not skip because prior compatible runs passed.

## Implementation Evidence

- Implemented in Phase 151 worktree `feat/phase-151-quickcheck-v1`.
- Focused verification: `cargo test -p ash-cli quickcheck -- --nocapture`.
- Broad scoped gates: `cargo check -p ash-cli`; `cargo clippy -p ash-cli --all-targets -- -D warnings`; `cargo fmt --check`; `git diff --check`.
