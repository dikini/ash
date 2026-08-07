# TASK-2025: Effect-Row Provider Binding Identity and Sanitization

**Status:** Complete — V8 effect-row summaries now separate immutable provider identity from
visible bindings; the module loader sanitizes named/glob/public-re-export closure transport;
inaccessible dependencies and binding conflicts reject before registration/publication; and the
process-local semantic-summary cache key covers the V8 contract. This remains semantic-summary and
typechecking metadata only, with no provider or handler runtime authority.
**Phase:** TASK-1988 implementation follow-up
**Depends on:** [TASK-2001](TASK-2001-target-grammar-gap-and-spec-conflict-decision.md) bounded
effect-row summary transport, canonical module identities, and the existing module-loader/type
environment import boundary.
**Design:** [Effect-Row Provider Binding Design](../../plans/2026-07-24-effect-row-provider-binding-design.md)
and its [implementation plan](../../plans/2026-07-24-effect-row-provider-binding-implementation-plan.md).

## Description

Replace the current implicit effect-row import convention with an explicit, fail-closed contract:
an immutable provider identity is separate from each visible binding; named imports, glob imports,
and public re-exports all run through one sanitizing dependency closure; inaccessible dependencies
are opaque and non-serialized; conflicts reject deterministically; and summaries/caches are
versioned for the new contract.

## Requirements and invariants

1. A provider identity is rooted in the provider module and declaration, and is unchanged by
   aliases, facades, glob imports, or `pub use`.
2. A visible binding records local/re-exported name and exposure separately from provider identity.
3. Named, glob, and public-use selection share one identity-based sanitizing closure. It transports
   only public/exportable dependencies needed by the selected provider.
4. A private, missing, or otherwise inaccessible dependency is represented only by an opaque
   boundary classification. Its name, path, anchor, signature, row text, and provider identity
   never enter serialized summaries, cache keys/values, or public diagnostics.
5. A visible binding may not denote multiple providers or incompatible closure content. Reject
   before summary publication/cache insertion, independently of import order.
6. Provider-binding data has a new schema version and complete semantic-cache coverage. Legacy,
   incomplete, mismatched, or unknown-version payloads fail closed rather than defaulting into
   usable bindings.
7. Effect rows remain non-granting requirements. No provider selection, installation, admission,
   handler frame, invocation, host operation, timeout/cancellation envelope, trace, monitor, or
   runtime execution is introduced.

## TDD steps

1. Add red core tests for immutable provider identity, alias-visible binding separation, opaque
   dependency serialization, and no-private-data leakage.
2. Add red validation/cache tests for the new summary version, legacy/incomplete rejection, and
   deterministic cache-key coverage.
3. Add red engine tests that named/glob/`pub use` share a sanitizer and preserve provider identity
   through a facade.
4. Add red engine/typechecker tests for opaque inaccessible-dependency rejection and two
   import-order permutations of every binding conflict.
5. Implement the smallest core summary, loader, cache, and registration changes to satisfy those
   tests without changing runtime authority.
6. Run focused tests, full workspace format/test/Clippy, orientation/docs/traceability gates, and
   `git diff --check`; then update completion evidence, PLAN index, and changelog.

## Completion checklist

- [x] Provider identity is immutable and visible bindings are separate data.
- [x] Named/glob/`pub use` use the same sanitizing closure.
- [x] Inaccessible dependencies reject opaquely with no private-data leakage.
- [x] Conflicts reject deterministically before publication/cache insertion.
- [x] New summary/cache schema fails closed for legacy, incomplete, mismatched, and unknown data.
- [x] Effect rows remain non-authority metadata; no runtime/provider behavior is added.
- [x] Focused implementation tests, review, documentation checks, and task/index/changelog
  evidence are complete. Broader workspace verification remains recorded by the owning integration
  workflow rather than claimed by this documentation closure.

## Completion evidence

`ash-core` now owns V8 (`STRUCTURAL_EFFECT_ROW_PROVIDER_BINDINGS_V8`) summary carriers for provider identity,
visible binding/exposure, opaque inaccessible-dependency status, sanitizer-schema evidence, and a
public-closure digest. V1–V6 effect-row payloads, incomplete/incoherent V7/V8 payloads, unsupported
sanitizer schemas, opaque public-boundary closures, and unknown future versions reject fail
closed. The V8 semantic cache key covers provider/binding/closure data while its opaque form does
not expose surrounding private detail.

`ash-engine` routes named imports, globs, and public re-exports through one
`sanitized_effect_row_semantic_summary` path. It preserves a provider's immutable identity across
aliases and facades, selects public closure members by provider identity before retaining provider
declaration order, rejects inaccessible private dependencies before imported-summary transport, and
preflights visible-binding/closure conflicts before publication. `ash-typeck` validates before
registration, stages imported summaries transactionally, accepts equivalent duplicate contracts,
and rejects incompatible contracts without replacing the prior visible binding.

The focused regression suite contains 27 controls:

1. 8 core V8/version/serialization/cache-key controls in
   [`task_2025_summary_versioning_cache.rs`](../../../crates/ash-core/tests/task_2025_summary_versioning_cache.rs);
2. 12 loader sanitizer/alias/facade/private-boundary/conflict controls in
   [`task_2025_effect_row_provider_binding.rs`](../../../crates/ash-engine/tests/task_2025_effect_row_provider_binding.rs);
   and
3. 7 TypeEnv opaque-boundary/transactional-conflict/idempotence controls in
   [`task_2025_effect_row_summary_boundary.rs`](../../../crates/ash-typeck/tests/task_2025_effect_row_summary_boundary.rs).

This evidence does **not** install or select a runtime provider, create an admission or handler
frame, dispatch an operation, perform host I/O, define timeout/cancellation behavior, execute a
handler, or make general effect-row residual semantics available. It is not a new formal-spec
claim: it refines the existing `TYPE-TARGET-ROW-001` import/summary transport mapping, so no new
SPEC index or semantic-traceability graph node is required.
