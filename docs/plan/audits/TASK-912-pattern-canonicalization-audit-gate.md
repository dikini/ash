# TASK-912 Pattern Canonicalization Audit Gate

Status: Complete
Date: 2026-05-17
Branch: phase-121-pattern-canon
Phase: Phase 121 / PLAN-117
Spec: SPEC-068

## Decision

Downstream implementation must introduce a pattern-specific canonicalization API instead of directly reusing `TypeEnv::canonicalize_type_for_equality`.

The equality API is close but not identical to the pattern need. It recursively expands transparent aliases and canonicalizes identity names for equality, and it can route representable terms through the SPEC-060 normalizer/definitional-equality boundary. Pattern typing and exhaustiveness need a narrower operation that answers whether a scrutinee has a concrete matchable ADT identity and constructor universe. Neutral/stuck associated projections, type-function outputs, constructor-variable applications, and unrelated same-visible-name constructors must remain blocked or rejected with pattern diagnostics rather than silently being treated as equality-equivalent.

Required API shape for TASK-913:

- add a `TypeEnv`-owned pattern canonicalization result, not a parser or engine helper;
- expand transparent aliases and selected reducible associated projections only when the result is a concrete ordinary ADT identity already known to `TypeEnv`;
- carry the canonical type identity/visible name, canonicalized type arguments, source spelling, and a blocked reason for neutral/stuck/non-matchable forms;
- expose constructor-universe lookup through the same result so pattern typing and exhaustiveness cannot drift;
- preserve the existing equality API for equality forcing points only.

Frozen SPEC-068 non-goals remain active: no broad equality adoption, no neutral inversion, no ADT runtime layout changes, no GADT/refinement pattern semantics, no type-level sealed-domain/promoted-constructor matching at runtime, and no name-only constructor equivalence.

## Live Callsite Audit

| ID | Seam | Live files / functions inspected | Current behavior | Gap | Downstream owner |
|---|---|---|---|---|---|
| A1 | TypeEnv ADT metadata and visible constructor map | `crates/ash-typeck/src/type_env.rs:3141` `TypeEnv`, `type_env.rs:3143` `ast_types`, `type_env.rs:3145` `type_info`, `type_env.rs:3147` `constructors`, `type_env.rs:15786` `lookup_constructor`, `type_env.rs:15811` `get_variant`, `type_env.rs:16908` `unfold_constructor` | ADT definitions are keyed by visible type name. Constructors are keyed by exported/visible constructor name and point to `(type name, variant index)`. `unfold_constructor` substitutes type arguments for a named `Type::Constructor`. | No API resolves constructor names against a canonical ADT identity selected from a scrutinee type. The constructor map is visible-name-first, so downstream pattern work must not treat same visible names from unrelated ADTs as equivalent. | TASK-913 / TASK-914 |
| A2 | Type registration and transparent aliases | `type_env.rs:5505` `register_type_identity`, `type_env.rs:5531` `expose_type_representation`, `type_env.rs:5557` `transparent_alias_target`, `type_env.rs:5572` `register_type`, `type_env.rs:5739` `expose_summary_type_representation` | Exposed enum representations register constructors. Struct representations with a `__alias_target` field mark transparent aliases. Imported summaries expose constructors only when summary metadata matches the parent type identity and reject duplicate exported constructor conflicts. | Alias expansion exists, but constructor lookup is not tied to alias-canonical scrutinee identity. Imported constructor identities are available in summaries but not consumed by pattern/exhaustiveness APIs. | TASK-913 / TASK-914 |
| A3 | Equality canonicalization substrate | `type_env.rs:13269` `type_identity_for_name`, `type_env.rs:13303` `canonical_constructor_name_for_equality`, `type_env.rs:13486` `canonicalize_transparent_aliases`, `type_env.rs:13557` `canonicalize_type_for_equality`, `type_env.rs:13629` `unify_types`, `type_env.rs:13644` `types_equivalent_for_equality`, `type_env.rs:13657` `definitionally_equal_types_when_canonicalizable` | Equality canonicalization recursively expands transparent aliases, canonicalizes nominal identity names, canonicalizes associated projection identities, and then optionally compares canonical IR normal forms through `Normalizer::definitional_equality`. | This behavior is broader than pattern matching needs. Pattern consumers need concrete matchability and blocked diagnostics, not a boolean equality answer or fallback unification. | TASK-913 |
| A4 | Associated projection normalization | `type_env.rs:16576` `normalize_associated_types`, `type_env.rs:16651` `resolve_interface_method_call`, `crates/ash-typeck/src/types.rs:73` `Type::Associated` | Associated types can normalize through a selected impl scheme. Rigid or unresolved projections remain represented as `Type::Associated` at ordinary type boundaries. | Pattern canonicalization may use only selected/reducible projections that produce a concrete ADT. It must not invert a projection result or guess a constructor universe for rigid/neutral forms. | TASK-913 / TASK-916 |
| A5 | Parser pattern surface | `crates/ash-parser/src/parse_pattern.rs:26` `pattern`, `parse_pattern.rs:46` `parse_variant_pattern`, `parse_pattern.rs:89` record variant construction, `parse_pattern.rs:101` tuple variant construction, `parse_pattern.rs:115` unit variant construction | Parser stores source-visible variant names and payload shape only. It does not carry type identity or constructor identity. | No parser change is needed for TASK-913 through TASK-916 unless diagnostics need extra spans later. Semantics belong in `ash-typeck`. | TASK-916 only if diagnostics need span refinements |
| A6 | Standalone pattern checker environment | `crates/ash-typeck/src/check_pattern.rs:23` local `check_pattern::TypeEnv`, `check_pattern.rs:59` `lookup_variant`, `check_pattern.rs:146` `check_pattern`, `check_pattern.rs:241` `check_variant_pattern`, `check_pattern.rs:296` `simple_type_expr_to_type` | The local pattern environment stores cloned AST type definitions only. `check_variant_pattern` first matches `expected: Type::Constructor` by visible `name.name`, then falls back to visible-name scanning when `expected` is a type variable. `simple_type_expr_to_type` punts associated type expressions with `todo!`. | Pattern checking is not connected to canonical TypeEnv identities, transparent aliases, selected associated projection reduction, or blocked neutral diagnostics. The local environment cannot own SPEC-068 semantics by itself. | TASK-913 / TASK-914 |
| A7 | Match expression integration | `crates/ash-typeck/src/check_expr.rs:2576` `pattern_type_env_from`, `check_expr.rs:2632` `resolve_enum_type_def_for_match`, `check_expr.rs:2725` `check_match`, `check_expr.rs:2747` arm binding `check_pattern` call | Exhaustiveness selects an enum by scrutinee constructor expression or first variant-name lookup. Arm binding currently calls `check_pattern` with a fresh empty pattern env and the inferred scrutinee type. | The match path must canonicalize the scrutinee type once through the TASK-913 API, pass the resulting constructor universe to both pattern checking and exhaustiveness, and avoid visible-name-only discovery from unrelated constructors. | TASK-914 / TASK-915 |
| A8 | Exhaustiveness checker | `crates/ash-typeck/src/exhaustiveness.rs:42` `PatternMatrix::new`, `exhaustiveness.rs:59` `pattern_to_cell`, `exhaustiveness.rs:88` `check_exhaustive`, `exhaustiveness.rs:98` `find_uncovered` | Exhaustiveness accepts an already selected `TypeDef`. It checks variant names/payload shape from that type definition and emits witness patterns. It has no scrutinee `Type` input and no canonicalization hook. | TASK-915 must change the entry boundary or add a wrapper so exhaustiveness consumes the same canonical ADT universe as pattern typing. Blocked/neutral scrutinee types must produce an unsupported/blocked diagnostic instead of `Covered` or name-guessed coverage. | TASK-915 / TASK-916 |
| A9 | Constructor expression and bare unit constructor typing | `crates/ash-typeck/src/check_expr.rs:104` variable fallback to `get_variant`, `check_expr.rs:2787` `check_constructor`, `check_expr.rs:2793` `get_variant`, `check_expr.rs:2830` `build_constructor_type` | Expression constructors and bare unit constructors use the current visible constructor map and return the variant parent type. | This is adjacent but not owned by Phase 121 unless pattern diagnostics need to compare expression construction with pattern matching. Do not alter constructor expression semantics in TASK-914. | Non-owner / non-interference tests |
| A10 | Pattern binding helper outside `check_expr` | `crates/ash-typeck/src/lib.rs:464` `variant_field_types`, `lib.rs:471` `unfold_constructor`, `lib.rs:485` `lookup_constructor` fallback | The helper can get fields from `expected: Type::Constructor` by unfolding, otherwise falls back to visible constructor lookup. | Same canonicalization gap as match patterns. If downstream tasks touch this helper, they must route through the pattern API and keep visible-name fallback guarded by canonical identity. | TASK-914 if touched |
| A11 | Core semantic-summary constructor identities | `crates/ash-core/src/semantic_summary.rs:561` `TypeDeclSummary`, `semantic_summary.rs:601` `ConstructorSummary`, `semantic_summary.rs:1046` `ModuleSemanticSummary`, `semantic_summary.rs:1050` `exported_constructors` | Core summaries carry constructor identity, parent type identity, exported name, payload kind, visibility, and source anchors. | Pattern/exhaustiveness currently do not consume `ConstructorSummary::id` directly. This is substrate only; do not move pattern semantics into `ash-core` or `ash-engine`. | TASK-914 / TASK-916 diagnostics as needed |
| A12 | Engine summary transport | `crates/ash-engine/src/module_loader.rs:773` selected constructor merge, `module_loader.rs:3005` summary exported constructors, `module_loader.rs:3065` TypeEnv summary registration, `crates/ash-engine/src/lib.rs:1488` TypeEnv summary registration | Engine transports and selects constructor summaries, then delegates semantics to `TypeEnv::register_module_semantic_summary`. | No Phase 121 engine semantics are required unless later tests expose summary transport non-interference. Engine must remain transport-only. | TASK-916 non-interference only |

## Target Ownership and Implementation Guidance

TASK-913 owns the new pattern-specific TypeEnv API. The first proof should be a focused `ash-typeck` test binary named `task_913_pattern_canonicalization_api`. It should prove transparent aliases canonicalize to the same ADT identity, selected reducible projections can produce a concrete ADT result, and rigid/neutral forms return a typed blocked result rather than equality success. Do not make callers use `canonicalize_type_for_equality` directly.

TASK-914 owns pattern constructor resolution. It should update `check_pattern` and the match arm path so constructor lookup is relative to the canonical ADT universe returned by TASK-913. Positive alias-equivalent constructor lookup must be paired with a negative same-visible-name unrelated-ADT test.

TASK-915 owns exhaustiveness. It should replace `resolve_enum_type_def_for_match` name-guessing with the TASK-913 canonical constructor universe and ensure `check_exhaustive` or its wrapper consumes the same universe as pattern typing.

TASK-916 owns diagnostics and leakage coverage. It should add stable diagnostics for blocked neutral/projection forms, wrong canonical identity despite same visible constructor name, and non-interference for existing direct ADT matches. It may include engine summary non-interference only if TASK-914 or TASK-915 touches imported-summary behavior.

## Downstream Focused Verification Commands

TASK-913 pattern canonicalization API:

```bash
cargo test -p ash-typeck --test task_913_pattern_canonicalization_api -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-914 alias-aware constructor resolution:

```bash
cargo test -p ash-typeck --test task_914_alias_aware_constructor_resolution -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-915 exhaustiveness over canonical constructor universe:

```bash
cargo test -p ash-typeck --test task_915_exhaustiveness_canonical_constructor_universe -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-916 diagnostics and negative leakage:

```bash
cargo test -p ash-typeck --test task_916_pattern_canonicalization_diagnostics -- --nocapture
cargo test -p ash-typeck --test task_916_pattern_canonicalization_negative_leakage -- --nocapture
cargo fmt --check
git diff --check
cargo check --workspace
```

## Fail-Closed Expectations

- Reject or block neutral/stuck associated projections instead of guessing a constructor universe.
- Reject constructor-variable applications and type-function outputs as matchable ADTs unless a later spec defines that runtime pattern surface.
- Reject same-visible-name constructors when their canonical ADT identity differs.
- Keep existing direct ADT matching behavior unchanged.
- Keep ADT runtime layout unchanged.
- Keep parser pattern names as raw surface names; do not encode semantic identity in parser nodes for this phase.
- Keep engine summary transport semantics unchanged.

## TASK-912 Verification

Required gate commands:

```bash
cargo fmt --check
test -f docs/plan/audits/TASK-912-pattern-canonicalization-audit-gate.md
git diff --check
! rg -n 'false # TASK-91[3-6]' docs/plan/tasks/TASK-91{3,4,5,6}-*.md
```

This task does not implement Rust production behavior for TASK-913 through TASK-916.
