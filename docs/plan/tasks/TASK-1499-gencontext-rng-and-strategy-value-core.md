# TASK-1499: GenContext, RNG, and Strategy value core

## Status: 📝 Planned

## Description

Implement the ordinary `Strategy<A>` value shape, helper-first `GenContext`, deterministic split helpers, and the versioned `ash-quickcheck-rng-v1` golden-vector contract.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1497: Live syntax and seam audit (planned)
- 📝 TASK-1498: Stdlib module split and prelude (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Define the target `Strategy<A>` carrier and callable-field execution path or audited equivalent if live callable fields need staged support.
2. Implement `GenContext` helper APIs: size, seed/debug, split, variant, indexed, resize, choose_int, choose_bool.
3. Select concrete `ash-quickcheck-rng-v1` algorithm and add platform-stable golden vectors.
4. Ensure case index is trace metadata, not generator-visible context.

### Property Requirements

- Same root seed, size, and split path produce identical child contexts and choices across runs/platforms.
- Distinct split paths produce stable independent streams.
- `gen(ctx)` produces one candidate, not a batch.

## TDD Steps

### Step 1: RED golden vectors

Add deterministic tests for root seed, split, variant, indexed, resize, and choose helpers.

### Step 2: GREEN context/RNG implementation

Implement helper-first context and RNG/split contract.

### Step 3: Trace tests

Assert traces record RNG algorithm, root/effective seed, split path, and size.

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

- Strategy/context substrate for TASK-1500 through TASK-1503.
- Golden vector artifact for future compatibility checks.

## Notes

Do not expose arbitrary seed arithmetic as the recommended authoring path.
