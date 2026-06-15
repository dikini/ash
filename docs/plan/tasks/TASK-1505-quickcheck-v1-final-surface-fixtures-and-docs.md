# TASK-1505: QuickCheck v1 final-surface fixtures and docs

## Status: 📝 Planned

## Description

Add user-facing no-Cargo fixtures and documentation for ordinary strategies, explicit evidence imports, overrides, RNG/replay, recursive/weighted generators, shrinking, and aggregate evidence history.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1504: QuickCheck seed, replay, and aggregate evidence history (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Add final-surface `.ash` examples for default `Arbitrary`, alternate `Int` strategies, sorted lists, safe recursive expressions, and override blocks.
2. Add docs explaining seed policy, cases policy, shrink failure classes, and aggregate evidence history.
3. Ensure examples use canonical submodule paths in reference sections and alpha aliases only in tutorials.
4. Run through `$ASH_UNDER_TEST test ...`, not cargo-run-only internal fixtures.

### Property Requirements

- Docs do not show hidden runner registries or source seeds as normal style.
- Examples compile/parse/run in no-Cargo user-facing path.
- Recursive examples demonstrate bounded recursion and explicit shrinkers.

## TDD Steps

### Step 1: RED docs/fixture examples

Write examples before or alongside implementation verification; they should fail if the final surface is not wired.

### Step 2: GREEN docs and fixtures

Update reference/tools docs and cookbook surfaces.

### Step 3: No-Cargo verification

Run `$ASH_UNDER_TEST test ...` fixtures and record commands/output in the task file.

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

- User-facing proof that SPEC-087 is not only a Rust/internal bridge.
- Documentation consumed by closeout.

## Notes

If an example uses illustrative future syntax, label it explicitly; do not mix it with runnable fixtures.
