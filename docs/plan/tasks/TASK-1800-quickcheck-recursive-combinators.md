# TASK-1800: Implement or explicitly re-scope recursive QuickCheck combinators

## Status: ✅ Complete / Explicitly Re-scoped

## Description

Implement or explicitly re-scope the remaining QuickCheck recursive combinator surface according to TASK-1799. Phase 176 landed the SPEC-087 public names and config shape, plus final-surface import/check coverage, but re-scoped execution to a visible fail-closed private helper because the ordinary-Ash size-descending implementation exposed current parser/type-metadata limitations around fn-body `match`/`panic` helper shapes.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ TASK-1799 design audit complete

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1511 recursive combinators | PLAN-151/TASK-1511 | Self-referential values and closure/language limits | Closure/module-helper visibility is now present; self-referential values remain absent and size-descending ordinary-Ash helpers hit current parser/type-metadata limits | Re-scope: land SPEC-087 public names/config and fail-closed execution guard; keep real bounded generation deferred to parser/type-metadata follow-up | Final-surface import/check fixtures plus visible `recursive_deferred` guard |

## Requirements

### Functional Requirements

1. Add final-surface tests for the SPEC-087 recursive strategy API imports/checking:
   - `recursive(base, expand)`
   - `recursive_with(base, expand, config)`
   - `recursive_config(base_weight, expand_weight, size_step)`
2. Land the public API in ordinary Ash stdlib source without a hidden Rust fallback and without a self-referential value binding; if bounded generation cannot be parsed/checked honestly, route execution through a visible fail-closed private helper.
3. Align `RecursiveConfig` to SPEC-087 fields: `base_weight`, `expand_weight`, and `size_step`.
4. Keep bounded generation semantics deferred with an explicit blocker until the parser/type-metadata path accepts the required fn-body helper shapes.
5. Preserve conservative shrink semantics unless a domain-preserving recursive shrinker is explicitly supplied.
6. Update docs and TASK-1511 notes to remove stale closure-visibility blocker language, distinguish the future size-descending ordinary-Ash implementation design from the landed fail-closed Phase 176 guard, and avoid claiming bounded generation is implemented.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Write RED final-surface fixtures

Add engine/stdlib tests that fail before implementation and prove final-surface import/check visibility plus fail-closed recursive execution. Move base-only zero-size generation, positive-size expansion, and invalid config execution tests to the parser/type-metadata follow-up that implements bounded generation.

### Step 2: Implement ordinary Ash source

Patch `std/src/test/quickcheck/combinator.ash` with the ordinary-Ash public API and a private fail-closed guard, avoiding hidden Rust generator fallbacks and avoiding self-referential values.

### Step 3: Verify bounded behavior

Verify the execution guard fails closed with a blocker-named diagnostic. Keep size descent, base/expand weights, invalid config, and deterministic context resizing/splitting behavior deferred until bounded generation is implementable.

### Step 4: Update docs/status

Patch TASK-1511 or related docs to avoid stale blocker text.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-engine
  - cargo test -p ash-cli
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - git diff --check
checklist:
  - [x] Final-surface recursive combinator import/check tests pass
  - [x] No hidden Rust fallback introduced
  - [x] Execution boundary is fail-closed via private ordinary-Ash helper and focused regression
  - [x] Docs/task status match re-scoped API
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

If this task keeps a deferral, it must add a durable negative/blocked test or explicit guard so the boundary is visible.

## TASK-1799 decision

TASK-1799 selected the SPEC-087 public API and the future private size-descending helper design. Phase 176 landed the public API/config plus fail-closed execution guard; do not introduce self-referential values or a Rust fallback. See `../../audit/PHASE-176-quickcheck-recursive-combinator-audit.md`.


## Completion Evidence

Phase 176 landed the final-surface names in `std/src/test/quickcheck/combinator.ash` and exported them from `std/src/test/quickcheck/mod.ash`:

- `recursive`
- `recursive_with`
- `recursive_config`
- `default_recursive_config`
- `RecursiveConfig { base_weight, expand_weight, size_step }`

Execution intentionally calls private `recursive_deferred`, which reaches an unresolved blocker-named function instead of pretending bounded recursive generation is implemented. The attempted ordinary-Ash size-descending helper implementation reproduced parser/type-metadata failures for fn-body helper shapes (`match` over field-access scrutinees and direct `panic` in anonymous fn bodies).

Focused validation:

```text
cargo run -q -p ash-cli -- check std/src/test/quickcheck/combinator.ash
[OK] std/src/test/quickcheck/combinator.ash: OK (module file: 2 type(s), 14 fn(s))

cargo test -p ash-engine --test phase151_quickcheck_stdlib
4 passed
```
