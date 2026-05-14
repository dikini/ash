# TASK-892 Promoted Constructor Audit Gate

Status: Complete
Date: 2026-05-14
Phase: Phase 118 / PLAN-114
Spec: SPEC-065

## Decision

Phase 118 uses explicit named data-kind declarations:

```ash
pub data kind Nat from type Nat;
data kind PrivateNat from type Nat;
```

The promoted kind name is explicit and separate from the source ADT name, even when it is spelled the same. The MVP promotes every constructor of the source ADT in source order. Constructor visibility is bounded by both the data-kind declaration visibility and the source ADT visibility; private promoted identities must not be exported through public type-computation summaries. Recursive promotion is accepted only for direct fields that name the same promoted source ADT after TypeEnv validation; arbitrary runtime field types remain rejected.

This syntax avoids introducing an attribute grammar before the parser has a general attribute substrate and keeps promotion opt-in without making ordinary ADT constructors type-level by default.

## Live seam audit

| Layer | Live files / callsites | Current substrate | Phase 118 requirement |
|---|---|---|---|
| Parser module definitions | `crates/ash-parser/src/lib.rs`, `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_type_def.rs`, `crates/ash-parser/tests/task_874_proposition_surface.rs` | `Definition::Type`, `Definition::SealedDomain`, raw `Type::Constructor`, `TypePattern::Constructor`; no data-kind definition | Add `Definition::DataKind`, parser for `[pub] data kind <Kind> from type <Adt>;`, parser tests, no typeck semantics in parser |
| Core type IR | `crates/ash-core/src/type_ir.rs`, `crates/ash-core/src/semantic_summary.rs`, `crates/ash-core/src/lib.rs` | Sealed-domain ids/apps exist separately from nominal apps; propositions and type-function results carry `DomainConstructorApp`; no promoted data ids/apps | Add `PromotedDataKindId`, `PromotedConstructorId`, promoted app variants beside sealed-domain app variants, and summary carriers |
| TypeEnv / kinding | `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/lib.rs`, existing tests `task_798_*`, `task_799_*`, `task_838_*`, `task_840_*` | Type declarations, sealed domains, type functions, propositions, associated families are registered; kind/domain constraints exist | Register promoted kinds from source ADTs; derive constructor kind from unit/tuple/record payload; reject unsupported fields; expose lookup APIs |
| Normalizer / propositions | `crates/ash-typeck/src/normalizer.rs`, `crates/ash-typeck/src/solver.rs`, `crates/ash-typeck/src/constraint_checking.rs`, tests `task_824_*`, `task_876_*`, `task_882_*` | Normal forms and proposition terms distinguish sealed domain constructor apps from nominal apps | Normalize promoted constructor apps as a distinct closed head family; prove disjointness/non-interference with sealed-domain and nominal/runtime ADT heads |
| Engine summaries | `crates/ash-engine/src/module_loader.rs`, `crates/ash-engine/src/lib.rs`, tests `task_854_*`, `task_867_*`, `task_879_*` | Engine transports type-computation and proposition summaries; validation remains in typeck | Transport public promoted kind/constructor metadata only; do not make engine own kinding semantics |
| Runtime ADTs / patterns | `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/lower.rs`, `crates/ash-interp`, parser/runtime ADT tests | Runtime constructor expressions/patterns use ordinary ADT payload carriers | No runtime layout or pattern semantics change; promoted constructors must remain type-level only |

## Downstream focused verification commands

TASK-893 parser surface:

```bash
cargo test -p ash-parser --test task_893_promoted_constructor_parser_surface
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-894 core identities/summaries:

```bash
cargo test -p ash-core --test task_894_promoted_constructor_identities
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-895 TypeEnv registration/kinding:

```bash
cargo test -p ash-typeck --test task_895_promoted_constructor_registration
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-896 normalizer/proposition/non-interference:

```bash
cargo test -p ash-typeck --test task_896_promoted_constructor_integration
cargo test -p ash-engine --test task_896_promoted_constructor_non_interference
cargo fmt --check
git diff --check
cargo check --workspace
```

## Non-goals / traps

- Do not silently promote every ordinary ADT constructor.
- Do not encode promoted constructors as ordinary `Type::Constructor` / nominal apps once they cross semantic boundaries.
- Do not reuse sealed-domain marker ids for promoted runtime ADT constructors.
- Do not add term-level singleton reflection, dependent pattern matching, GADTs, or proof values.
- Do not modify runtime ADT construction, runtime pattern matching, capability dispatch, `do` typing, or interface resolution.
- Do not let private promoted constructors leak through public summaries, type functions, or propositions.

## TASK-892 verification

- Audit artifact: this file.
- Downstream fail-closed guards patched in TASK-893 through TASK-896.
- Baseline `cargo check --workspace` passed in the dedicated worktree before implementation.
