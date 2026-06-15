# TASK-1501: QuickCheck with-block override parser and typechecker support

## Status: 📝 Planned

## Description

Implement parser/typechecker support for `by test quickcheck with { ... }` where override RHSs are pure `Strategy<T>` expressions, accepting both explicit `strategy` and inferred forms.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1500: Arbitrary evidence resolution without hidden bridges (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Parse `x <- strategy expr` and `x <- expr` in QuickCheck backend-local override blocks.
2. Keep `with` blocks parameter/domain-only; run config remains in backend options.
3. Reject unknown/duplicate bindings, wrong strategy type, and impure RHSs.
4. Allow partial overrides with fallback to `Arbitrary<T>` evidence.

### Property Requirements

- Explicit and inferred strategy forms elaborate to the same internal override representation.
- Partial override fallback uses ordinary evidence.
- Wrong type/impure/missing binding diagnostics fail closed and do not run the property.

## TDD Steps

### Step 1: RED parser/type fixtures

Add fixtures for explicit, inferred, partial, duplicate, unknown, wrong-type, and impure overrides.

### Step 2: GREEN parser and typed elaboration

Wire surface parsing, typed override resolution, and diagnostics.

### Step 3: Runner handoff test

Assert typed strategy expressions reach runner resolution without metadata strings.

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

- Final override syntax substrate for TASK-1503 and docs in TASK-1505.

## Notes

Do not put `cases`, `seed`, or shrink limits inside the `with` block.
