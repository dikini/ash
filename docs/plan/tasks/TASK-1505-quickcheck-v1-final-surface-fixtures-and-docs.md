# TASK-1505: QuickCheck v1 final-surface fixtures and docs

## Status: ✅ Complete

## Description

Add user-facing no-Cargo fixtures and documentation for ordinary strategies, explicit evidence imports, overrides, RNG/replay, recursive/weighted generators, shrinking, and aggregate evidence history.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- ✅ TASK-1504: QuickCheck seed, replay, and aggregate evidence history

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
  - [x] Focused tests pass and are non-zero
  - [x] No-Cargo final-surface fixture added where user-facing behavior changed
  - [x] Negative leakage/fail-closed cases covered where a bridge or error path is touched
  - [x] CHANGELOG.md updated under [Unreleased]
```

## Dependencies for Next Task

- User-facing proof that SPEC-087 is not only a Rust/internal bridge.
- Documentation consumed by closeout.

## Notes

If an example uses illustrative future syntax, label it explicitly; do not mix it with runnable fixtures.

## Implementation Evidence

- Added final-surface fixtures under `fixtures/phase151-quickcheck-v1/tests/ash/property/` for canonical positive `Int` strategy overrides, canonical sorted-list strategy overrides, default `Arbitrary<Bool>` generation, and source-seed warning plus CLI replay override behavior.
- Updated `reference/tools/test.md` to document the Phase 151 seed/case precedence, canonical strategy paths, shrink/failure-class evidence, aggregate evidence history, and the current deferred boundary for full source-visible recursive/weighted strategy expressions.
- Fixed authored property case precedence so source `@test max_cases` is exact and CLI `--max-cases` only fills in when the source is silent.
- RED/GREEN focused regression:
  - RED: `cargo test -p ash-cli --test phase150_quickcheck_metadata quickcheck_v1_final_surface_canonical_paths_and_source_cases_are_no_cargo_visible -- --nocapture` failed with `left: Number(99) right: 2` before the precedence fix.
  - GREEN: same command passed with `1 passed; 0 failed` after the fix.
- No-Cargo final-surface evidence:
  - `export ASH_UNDER_TEST="$PWD/target/debug/ash"; "$ASH_UNDER_TEST" test fixtures/phase151-quickcheck-v1/tests/ash/property --format json --seed 123 --max-cases 99`
  - Result: `success: true`, `total: 4`, `passed: 4`, `failed: 0`; canonical positive source-cases fixture recorded `requested_cases: 2`, `executed_cases: 2`, `seed: 123`, `seed_source: "cli"`, `rng_algorithm: "ash-quickcheck-rng-v1"`.
