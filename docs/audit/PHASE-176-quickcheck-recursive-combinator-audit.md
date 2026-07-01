# Phase 176 QuickCheck Recursive Combinator Design Audit

Date: 2026-07-01

## Scope

This audit resolves TASK-1799: whether the deferred QuickCheck recursive combinators from TASK-1511 can now be implemented as ordinary Ash source, or whether Phase 176 must re-scope them to an explicit bounded helper API.

Inputs inspected:

- `docs/spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md`
- `docs/plan/PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md`
- `docs/plan/tasks/TASK-1511-deferred-combinators-ordinary-ash.md`
- `std/src/test/quickcheck/{context,strategy,combinator,mod}.ash`
- `crates/ash-engine/tests/phase151_quickcheck_stdlib.rs`
- Phase 176 TASK-1798 closure/module-helper visibility implementation.

## Current stdlib state

`std/src/test/quickcheck/combinator.ash` already defines ordinary-Ash strategies and combinators for:

- `map`
- `map_with_shrink`
- `map2`
- `with_shrink`
- `constant`
- `weighted`
- `one_of`
- `one_of_weighted`
- `append_shrink`
- `prepend_shrink`
- `default_recursive_config`
- `recursive_config`

The remaining gap is the strategy-recursion entrypoint itself:

- `recursive(base, expand)`
- `recursive_with(base, expand, config)`

TASK-1511 marked those blocked because a literal implementation would need a self-referential strategy value:

```ash
let self_ref = Strategy { gen: fn(ctx) { expand(self_ref).gen(ctx) }, ... }
```

That exact shape is still not an Ash value the language should support implicitly.

## Live prerequisite check

| Capability | Current status | Evidence / implication |
|---|---|---|
| Ordinary `Strategy<T>` records with closure fields | Available | Existing `map`, `map2`, `constant`, `one_of`, and focused Phase 151 stdlib tests pass. |
| Closure capture of ordinary values | Available | Existing combinators capture `s`, `f`, `strategies`, and `choices`; TASK-1798 also proves local/imported helper visibility from closures. |
| Module-level helper calls from closures | Available after TASK-1798 | Local closures can call sibling module pure helpers; imported public callables can carry hidden same-module private helper runtime dependencies without leaking them to the caller. |
| Recursive module-level pure functions | Available enough for helper recursion | Existing `sum_weights`, `pick_weighted`, and `index_strategies` are recursive ordinary helpers in `combinator.ash`; engine checks the module successfully. |
| `GenContext` size/resize/split helpers | Available as builtin declarations | `context.ash` exposes `size`, `resize`, `split`, `variant`, `indexed`, `choose_int`, and `choose_bool`. |
| Self-referential values | Still absent / intentionally not required | The implementation should not add a hidden Rust fallback or a recursive value feature just for QuickCheck. |

Focused verification run during audit:

```text
cargo run -q -p ash-cli -- check std/src/test/quickcheck/combinator.ash
[OK] std/src/test/quickcheck/combinator.ash: OK (module file: 2 type(s), 12 fn(s))

cargo test -p ash-engine --test phase151_quickcheck_stdlib
3 passed
```

## Decision

Preserve the SPEC-087 public recursive API in ordinary Ash source, but do not overclaim bounded generation until the language substrate can express the helper body honestly.

Phase 176 landed the public surface:

```ash
pub type RecursiveConfig = RecursiveConfig {
    base_weight: Int,
    expand_weight: Int,
    size_step: Int,
};

pub fn default_recursive_config() -> RecursiveConfig { ... }
pub fn recursive_config(base_weight: Int, expand_weight: Int, size_step: Int) -> RecursiveConfig { ... }
pub fn recursive<T>(base: Strategy<T>, expand: (Strategy<T>) -> Strategy<T>) -> Strategy<T> { ... }
pub fn recursive_with<T>(base: Strategy<T>, expand: (Strategy<T>) -> Strategy<T>, config: RecursiveConfig) -> Strategy<T> { ... }
```

The intended future implementation shape remains explicit size descent through a private helper, not a self-referential strategy value. Phase 176 verified that this is the right design direction, but the direct implementation hit current parser/type-metadata limits around fn-body helper shapes. The landed behavior is therefore fail-closed: calling the recursive API enters a private `recursive_deferred` guard whose unresolved blocker name makes execution fail visibly instead of silently pretending bounded recursive generation exists.

## Rejected options

| Option | Decision | Reason |
|---|---|---|
| Add a hidden Rust QuickCheck recursive fallback | Rejected | TASK-1799/1800 explicitly require ordinary Ash strategy semantics. |
| Add self-referential value bindings | Rejected | This is a general language feature, not a QuickCheck-local cleanup; it would require a separate spec and runtime semantics. |
| Keep stale TASK-1511 blocker unchanged | Rejected | TASK-1798 removes the closure/module-helper visibility blocker, and size descent avoids the self-reference blocker. |
| Expose only `recursive_at` / depth-threaded helper publicly | Rejected for now | SPEC-087 already names `recursive` and `recursive_with`; an internal helper can supply the explicit depth/threading without changing the public API. |

## TASK-1800 proof obligations

TASK-1800 should implement and verify the honest Phase 176 slice:

1. `recursive`, `recursive_with`, `recursive_config`, and `default_recursive_config` in `std/src/test/quickcheck/combinator.ash`.
2. `RecursiveConfig` fields aligned to SPEC-087: `base_weight`, `expand_weight`, `size_step`.
3. No hidden Rust generator fallback and no self-referential value binding.
4. A visible fail-closed execution guard for bounded recursive generation until parser/type-metadata support can admit the size-descending helper implementation.
5. Final-surface import/check tests plus a regression proving recursive execution fails closed at the guard.
6. Changelog, PLAN/TASK status, and TASK-1511 stale blocker text reconciliation.

Deferred proof obligations for the follow-up substrate task are invalid-config checks and real bounded behavior (`size <= 0` base-only sampling, positive-size expansion, and smaller-context recursion).

## Audit result

TASK-1799 is complete: the chosen future implementation shape remains ordinary-Ash size-descending helper recursion, not self-referential values and not a Rust fallback. TASK-1800 deliberately landed only the public API/config plus fail-closed execution guard.


## TASK-1800 implementation addendum

During TASK-1800, the intended size-descending ordinary-Ash implementation was attempted directly in `std/src/test/quickcheck/combinator.ash`. The public API and config shape were viable, but the private helper implementation exposed current parser/type-metadata limitations in fn-body helper shapes: `match` scrutinees involving record field access and direct `panic` bodies in anonymous fn expressions failed module-file type-metadata parsing.

Phase 176 therefore re-scoped execution honestly: `recursive` and `recursive_with` are present and importable, `RecursiveConfig` uses the SPEC-087 fields, no Rust fallback or self-referential value was introduced, and runtime execution routes through private `recursive_deferred`, which fails closed by calling an unresolved blocker-named function instead of fabricating a generated value. Real bounded generation remains a follow-up parser/type-metadata substrate task rather than a hidden QuickCheck-specific runtime escape hatch.
