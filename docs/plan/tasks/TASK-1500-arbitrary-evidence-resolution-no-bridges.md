# TASK-1500: Arbitrary evidence resolution without hidden bridges

## Status: ✅ Complete

## Description

Implement minimal `Arbitrary<A>` default-strategy evidence resolution using ordinary in-scope imports and remove or quarantine Phase 150 hidden fallback registries.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- ✅ TASK-1499: GenContext, RNG, and Strategy value core

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Define `Arbitrary<A>` as `arbitrary() -> Strategy<A>` only.
2. Resolve defaults through ordinary in-scope evidence.
3. Provide stdlib primitive/container evidence through `test::quickcheck::prelude`.
4. Missing imports/evidence fail closed.
5. Remove or quarantine metadata/fallback registries so they are not semantic authority.

### Property Requirements

- With explicit prelude/import, primitive defaults resolve.
- Without import/evidence, the same property errors rather than silently using a runner fallback.
- Multiple alternate strategies for `Int` remain ordinary functions, not `Arbitrary<Int>` variants.

## TDD Steps

### Step 1: RED evidence fixtures

Add positive fixtures with `use test::quickcheck::prelude` and negative fixtures with imports omitted.

### Step 2: GREEN evidence resolver

Wire ordinary evidence lookup and remove/quarantine hidden fallback.

### Step 3: Bridge leakage tests

Assert old metadata/fallback paths cannot make a missing-evidence property pass.

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

- Ordinary default strategy resolution for TASK-1501 and TASK-1503.
- Negative bridge-leakage tests.

## Notes

Bridges tend to become debt. Any retained bridge must be named as compatibility-only and fail negative leakage tests.


## Implementation Evidence

- Implemented in Phase 151 worktree `feat/phase-151-quickcheck-v1`.
- Added explicit source import detection for `test::quickcheck::{Arbitrary}` / `test::quickcheck::prelude` before default `Arbitrary<A>` domains can materialize.
- Explicit `@test strategy` overrides remain a compatibility metadata bridge, but missing default evidence now fails closed instead of silently using a runner fallback.
- RED evidence: `cargo test -p ash-cli --test phase150_quickcheck_metadata quickcheck_v1_final_surface_canonical_paths_and_source_cases_are_no_cargo_visible -- --nocapture` failed on `quickcheck_missing_arbitrary_import_fails_closed` before the resolver change.
- Focused verification: `cargo test -p ash-cli --test phase150_quickcheck_metadata -- --nocapture` (6 passed); `cargo test -p ash-cli parse_quickcheck_arbitrary_evidence_imports -- --nocapture` (1 passed, non-zero unit filter).
- No-Cargo final surface: `$ASH_UNDER_TEST test fixtures/phase151-quickcheck-v1/tests/ash/property/quickcheck_default_arbitrary_bool.ash --format json` passed with explicit import; `$ASH_UNDER_TEST test fixtures/phase151-quickcheck-v1/tests/ash/property/quickcheck_missing_arbitrary_import_fails_closed.ash --format json` failed closed with `missing in-scope Arbitrary<Bool> evidence`.
