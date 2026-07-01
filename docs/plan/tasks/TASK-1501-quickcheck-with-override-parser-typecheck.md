# TASK-1501: Property-test proof evidence with source-visible strategy overrides

## Status: ✅ Complete

## Description

Make `by test property` (and accepted synonym `quickcheck`) first-class proof evidence by extending the parser, AST, and runner schema so that strategy overrides are source-visible `Strategy<T>` expressions, not metadata strings. `property` and `quickcheck` are synonymous surface vocabulary; only one AST representation (`ProofBody::ByTestProperty`) should exist, extended with an optional `strategies` payload.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- ✅ TASK-1497: Live syntax and seam audit
- ✅ TASK-1498: QuickCheck stdlib module split and prelude
- ✅ TASK-1499: GenContext, RNG, and Strategy value core
- ✅ TASK-1500: Arbitrary evidence resolution without hidden bridges
- ✅ TASK-1510: Parser support for `fn` expressions in multi-field struct literals

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Parse `by test property` and `by test quickcheck` as synonymous proof evidence modes into a single AST node.
2. Parse optional `with { x <- strategy_expr, y <- strategy_expr }` strategy override block.
3. Extend `ProofBody::ByTestProperty` with `strategies: Vec<PropertyStrategyBinding>`.
4. Extend `LawTestEvidence::Property` with structured strategy descriptors.
5. Validate: unknown binding → error; duplicate binding → error; strategy type mismatch → error/deferred; unsupported strategy expression → deferred with honest repro.
6. Allow partial overrides: missing bindings fall back to in-scope `Arbitrary<T>` evidence.
7. Preserve source spelling (`property` vs `quickcheck`) for diagnostics only; no semantic branching.

### Property Requirements

- Synonymous surface forms (`property`, `quickcheck`) produce identical AST and identical runner metadata.
- Partial override fallback uses ordinary in-scope `Arbitrary<T>` evidence.
- Wrong type / unsupported expression / missing binding diagnostics fail closed and do not run the property.
- Strategy expressions survive into runner metadata without string formatting loss.

## TDD Steps

### Step 1: RED — parser fixtures

Add parser tests for:
- `by test property` (no `with`)
- `by test quickcheck` (no `with`)
- `by test property with { x <- expr }`
- `by test quickcheck with { x <- expr }`
- `by test property with { x <- expr, y <- expr }` (trailing comma)
- duplicate binding in `with` block
- unknown binding in `with` block
- strategy expression with multi-field struct literal (TASK-1510 pattern)

### Step 2: GREEN — parser + AST + schema

- Extend `ProofBody::ByTestProperty` with `strategies` payload.
- Add `PropertyStrategyBinding` surface struct.
- Extend `LawTestEvidence::Property` with strategy descriptors.
- Update `extract_laws` / `law_test_evidence_from_proof_body` to preserve bindings.
- Update `format_expr` to support `Constructor` and `FnDef` so strategy expressions survive into metadata.

### Step 3: GREEN — runner override resolution

- Wire `law_property_results` to consume `Property` strategy descriptors.
- Map explicit strategy expressions to generated domains (initially via transitional path resolver).
- Validate unknown/duplicate bindings against law params.

### Step 4: REFACTOR — bridge removal

- Add negative leakage test: metadata-only `@test strategy` still works but is not the independent semantic authority.
- Document transitional bridge status in CHANGELOG.

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
  - cargo test -p ash-parser --test task_1501_property_test_override_parsing -- --nocapture
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

- `property` and `quickcheck` are synonyms; do not add a separate `ByTestQuickCheck` AST branch.
- Do not put `cases`, `seed`, or shrink limits inside the `with` block; run config remains in backend options / metadata.
- The transitional bridge may accept path-like strategy expressions first (e.g. `test::quickcheck::int::positive`) before full ordinary `Strategy<T>` evaluation lands.
