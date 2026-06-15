# TASK-1498: QuickCheck stdlib module split and prelude

## Status: 📝 Planned

## Description

Refactor/add the `test::quickcheck` Ash stdlib surface into canonical submodules with a narrow prelude and alpha root convenience re-exports.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1497: Live syntax and seam audit (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Create or refactor canonical modules for context, strategy, arbitrary, int, bool, string, list, combinator, and prelude.
2. Keep source files small and agent-editable.
3. Prelude imports core types/helpers and stdlib evidence, not every domain constructor.
4. Root aliases are marked alpha convenience aliases over canonical submodule APIs.

### Property Requirements

- Canonical submodule imports and root aliases resolve consistently.
- No hidden evidence appears without explicit prelude/imports.

## TDD Steps

### Step 1: RED import fixtures

Add `.ash` fixtures that import canonical submodules and `test::quickcheck::prelude`.

### Step 2: GREEN module split/re-exports

Implement the stdlib file/module structure from SPEC-087.

### Step 3: Negative import checks

Add a fixture proving no hidden evidence appears without explicit imports.

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

- Stable stdlib module skeleton for TASK-1499 and TASK-1500.
- Explicit prelude import surface.

## Notes

Do not make root aliases the canonical reference surface. Use submodule paths in reference docs.
