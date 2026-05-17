# SPEC-068: Pattern and Exhaustiveness Canonicalization

**Status:** Implemented MVP
**Date:** 2026-05-14
**Promotes:** [DESIGN-039](../design/DESIGN-039-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
**Origin:** [TASK-890](../plan/tasks/TASK-890-pattern-exhaustiveness-alias-canonicalization-packet.md)
**Builds on:** [SPEC-058](SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-060](SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-063](SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
**Related:** [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-020](SPEC-020-ADT-TYPES.md)
**Plan:** [PLAN-117](../plan/PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
**Implementation Tasks:** [TASK-912](../plan/tasks/TASK-912-pattern-canonicalization-audit-gate.md) through [TASK-917](../plan/tasks/TASK-917-pattern-canonicalization-closeout.md)

## 1. Summary

SPEC-068 defines how transparent alias/projection canonicalization may be consumed by pattern checking and exhaustiveness. It closes the remaining DESIGN-034 gap that Phase 110 deliberately left outside equality-boundary adoption.

The central rule is conservative: pattern checking may normalize transparent aliases and selected reducible projections only through a pattern-specific API. It must not solve under neutral computation heads or reinterpret unrelated same-visible-name constructors as equivalent.

## 2. Baseline

Live substrate:

- `TypeEnv::canonicalize_type_for_equality` and equality forcing points exist from SPEC-058/SPEC-060;
- pattern checking lives in `crates/ash-typeck/src/check_pattern.rs`;
- exhaustiveness lives in `crates/ash-typeck/src/exhaustiveness.rs`;
- constructor resolution and ADT metadata use current type identities;
- Phase 110 intentionally excluded pattern/exhaustiveness from broad canonicalization rollout.

## 3. Scope

In scope:

1. audit of live pattern and exhaustiveness callsites;
2. a pattern-specific canonicalization API or an explicit decision to consume the equality API;
3. alias-equivalent constructor resolution where safe;
4. exhaustiveness over the canonical ADT constructor universe;
5. positive alias/projection tests and negative leakage tests;
6. diagnostics for neutral/stuck/non-matchable canonical forms.

Out of scope:

- changing ADT runtime layout;
- matching on type-level sealed-domain or promoted constructors at runtime;
- inversion of type functions or associated families to discover constructors;
- broad search-and-replace adoption of equality canonicalization;
- GADT/refinement-pattern semantics.

## 4. Pattern Canonicalization API

The audit gate must choose one of two implementation strategies:

1. reuse `TypeEnv::canonicalize_type_for_equality` only after proving all consumers need exactly that behavior;
2. introduce a narrower `canonicalize_type_for_pattern` API that expands transparent aliases/projections but rejects or preserves neutral/stuck forms more explicitly.

The second strategy is preferred unless the audit proves equality and pattern consumers have identical safety needs.

## 5. Constructor Resolution

Rules:

- constructor lookup must resolve against the canonical ADT identity selected for pattern typing;
- source aliases may be transparent for finding the same ADT constructor set;
- unrelated modules exporting constructors with the same visible names remain distinct;
- projections reduce only when the normalizer can produce a concrete ADT identity without inversion;
- neutral computation heads and rigid projections are not matchable ADTs.

## 6. Exhaustiveness

Exhaustiveness uses the same canonical ADT identity and constructor set as pattern typing. A match is exhaustive only when every constructor in that canonical set is covered under the existing pattern algebra.

If the scrutinee type canonicalizes to a blocked/neutral form, exhaustiveness must produce an explicit unsupported/blocked diagnostic instead of guessing a constructor universe.

## 7. Diagnostics

Required diagnostics:

- alias/projection canonicalization blocked by neutrality;
- constructor visible name matches but canonical ADT identity differs;
- exhaustiveness cannot determine constructor universe for neutral type;
- pattern accepted by alias canonicalization with source-name note when helpful;
- non-interference assertion for existing direct ADT matches.

## 8. Acceptance Matrix

Phase 121 implements the MVP pattern/exhaustiveness slice for ordinary runtime ADTs. It does not add GADT/refinement patterns, type-level sealed-domain or promoted-constructor runtime matching, broad equality adoption, ADT runtime layout changes, or inversion under neutral computation heads.

| ID | Case | Expected result | Phase 121 evidence | Scope status |
|----|------|-----------------|--------------------|--------------|
| PC-1 | `type MyOption<T> = Option<T>` then match with `Some`/`None` | accepted if alias transparent | `TASK-913` `transparent_alias_to_adt_canonicalizes_to_underlying_constructor_universe`; `TASK-914` `transparent_alias_scrutinee_accepts_canonical_variant_pattern_and_binds_payload`; `TASK-915` `transparent_alias_full_match_uses_canonical_result_universe_and_is_exhaustive`; `TASK-916` `transparent_alias_match_remains_accepted` | Implemented MVP for transparent aliases over ordinary ADTs |
| PC-2 | associated projection reduces to concrete ADT before match | accepted if selected/reducible | `TASK-913` `selected_associated_projection_to_adt_canonicalizes_to_constructor_universe` | Implemented MVP at the pattern canonicalization API boundary; match-surface projection coverage remains limited to selected/reducible projections that already normalize to an ordinary ADT |
| PC-3 | rigid where-bound projection as match scrutinee | blocked diagnostic | `TASK-913` `unresolved_associated_projection_returns_typed_blocked_result`; `TASK-916` `unresolved_associated_projection_returns_typed_blocked_reason_for_patterns` | Implemented MVP as a typed blocked pattern-canonicalization result |
| PC-4 | neutral type-function result as match scrutinee | blocked diagnostic | `TASK-913` `constructor_variable_application_returns_typed_blocked_result`; `TASK-915` `blocked_non_matchable_scrutinee_does_not_guess_visible_arm_constructor_universe`; `TASK-916` `primitive_scrutinee_with_visible_constructor_does_not_fabricate_missing_witness` | Partial MVP boundary coverage: neutral/non-matchable heads are blocked and exhaustiveness does not guess, but Phase 121 does not introduce source-level type-function-result runtime matching |
| PC-5 | same visible constructor name from unrelated ADT | no leakage, rejected or resolved by source identity | `TASK-914` `visible_constructor_from_unrelated_adt_is_rejected_for_different_scrutinee_adt`; `TASK-916` `same_visible_constructor_from_unrelated_adt_is_rejected_for_scrutinee_identity` and `unrelated_constructor_name_is_rejected_and_does_not_bind_payload` | Implemented MVP |
| PC-6 | existing direct ADT match/exhaustiveness tests | unchanged | `TASK-914` `direct_adt_scrutinee_still_accepts_variant_pattern`; `TASK-915` `direct_result_full_match_remains_exhaustive`; `TASK-916` `direct_adt_match_remains_accepted` | Implemented MVP non-interference |

## 9. Implementation Tasks

See [PLAN-117](../plan/PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md).
