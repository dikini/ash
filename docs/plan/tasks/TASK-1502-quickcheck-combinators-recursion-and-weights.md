# TASK-1502: QuickCheck combinators, recursion, and weights

## Status: 📝 Planned

## Description

Implement namespaced function combinators for strategy composition, weighted choice, projection-based shrinking helpers, explicit shrink wrappers, and bounded recursive generation.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1499: GenContext, RNG, and Strategy value core (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Implement `map`, `map_with_shrink`, `map_project`, `map2`, `map2_with_shrink`, `map2_project`.
2. Implement QuickCheck-local generic `Weighted<A>`, `weighted`, `one_of`, and `one_of_weighted`.
3. Implement `recursive`, `recursive_with`, `recursive_config`, and `default_recursive_config`.
4. Implement `with_shrink`, `append_shrink`, and `prepend_shrink`.
5. Validate invalid weights/configs/empty choices fail closed.

### Property Requirements

- Plain map/map2 use empty/conservative shrinkers.
- Projection helpers reuse source shrinkers through `Option` projectors.
- Recursive generation always descends by `size_step` and uses base at size <= 0.
- Weights are constant at positive sizes and invalid weights are never clamped.

## TDD Steps

### Step 1: RED combinator tests

Add unit and final-surface fixtures for map/project, weighted choices, invalid weights, empty lists, and recursive size descent.

### Step 2: GREEN combinator implementation

Implement namespaced functions under `test::quickcheck::combinator`.

### Step 3: Shrink wrapper tests

Assert replacement, append existing-first, prepend extra-first, and no runner dedup.

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

- Combinator library for realistic recursive ADT examples in TASK-1505.
- Invalid-config/weight semantics for TASK-1503 runner errors.

## Notes

Do not implement hidden provenance-based structural shrinking in this phase.
