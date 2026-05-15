# TASK-898 Type Hole Audit Gate

Status: Complete
Date: 2026-05-15
Phase: Phase 119 / PLAN-115
Spec: SPEC-066

## Decision

Phase 119 MVP enables explicit source type holes only in audited explicit do-target type arguments, specifically the `do:Result<_, E> { ... }` shape.

Frozen MVP hole-position policy:

- Enabled: exactly one `_` in an explicit generalized do target type-argument spine, e.g. `do:Result<_, ParseError> { return value }`.
- The enabled hole is a value-position type argument that abstracts one constructor input so the elaborated target has effective kind `* -> *`.
- Rejected for MVP: bare higher-arity do targets such as `do:Result { ... }`, multiple do-target holes such as `Foo<_, _, E>`, holes outside explicit do-target type arguments, and holes that require type-function or associated-family output inversion.
- Separate namespace: `_` in type-function patterns remains `TypePattern::Wildcard` / `TypeFunctionPattern::Wildcard`; it is not a source type hole and must not allocate `TypeHoleId`.

This gate does not implement Rust behavior. It binds the live seams and downstream guards so TASK-899 through TASK-902 can proceed with focused, non-zero verification.

## Live seam audit

| ID | Layer | Live files / callsites | Current substrate | Phase 119 binding |
|---|---|---|---|---|
| A1 | Parser do-target surface | `crates/ash-parser/src/parse_expr.rs:31` `parse_do_block_expr`, `parse_expr.rs:73` `parse_do_target`, `parse_expr.rs:85` `parse_do_target_args`, `parse_expr.rs:112` `parse_do_type`, `crates/ash-parser/src/surface.rs:1417` `DoTarget`, `surface.rs:2020` `Type` | `DoTarget { name, args, span }` already preserves explicit target args as `Vec<Type>`. `parse_do_type` currently parses identifiers and nested `Type::Constructor` but cannot parse `_`. | TASK-900 adds parser-only hole surface under `parse_do_target_args`/`parse_do_type`, with spans. Do not enable `_` through ordinary workflow return types or general type declaration parsing in this task slice. |
| A2 | Parser ordinary type surfaces | `crates/ash-parser/src/parse_workflow.rs:727` `parse_type`, `crates/ash-parser/src/parse_type_def.rs:592` `parse_named_type`, `parse_type_def.rs:599` `parse_constructor_type`, `parse_type_def.rs:607` type-arg parser | Ordinary type parsers accept named/constructor/tuple/record/projection types. They do not currently have a hole variant. | Keep ordinary parser surfaces fail-closed for `_` until a later audit enables them. TASK-900 tests must prove `do:Result<_, E>` is the MVP enabled surface and type-function pattern `_` remains distinct. |
| A3 | Parser type-function wildcard surface | `crates/ash-parser/src/surface.rs:292` `TypePattern`, `surface.rs:315` `TypePattern::Wildcard`, `crates/ash-parser/tests/task_832_type_function_parser.rs` | Type-function equations have an existing wildcard pattern carrier with source span. | No type-hole lowering here. `_` in type-function patterns stays `TypePattern::Wildcard` and remains covered by existing/new non-interference tests. |
| A4 | Core kind/type-computation substrate | `crates/ash-core/src/kind.rs`, `crates/ash-core/src/type_ir.rs`, `crates/ash-core/src/lib.rs` | `Kind` supports `*` and arrows; `TypeComputationHeadId`, `CanonicalTypeExpr`, normal/proposition carriers, and type-function carriers exist. No `TypeHoleId`, no partial-argument carrier, and no partial-constructor expression carrier. | TASK-899 adds core-owned hole identity/source metadata and partial-application carriers beside existing canonical/normal terms. Do not encode holes as `CanonicalTypeExpr::Var` or saturated nominal constructors with fake args. |
| A5 | Typeck legacy type representation and lowering | `crates/ash-typeck/src/types.rs:16` `Type`, `types.rs:53` `Type::Constructor`, `crates/ash-typeck/src/lib.rs:207` surface `Type::Constructor` lowering, `lib.rs:1512` type walking | Legacy `Type::Constructor` assumes applied args of kind `*` and currently represents fully applied nominal/application types. | TASK-901 owns semantic elaboration/kinding of partial applications and must not route partial constructor terms through legacy saturated `Type::Constructor` without an explicit partial carrier boundary. |
| A6 | Do-target resolution boundary | `crates/ash-typeck/src/do_target.rs:41` `resolve_do_target`, `do_target.rs:47` explicit args rejected, `do_target.rs:65` bare `Result` deferred, tests in `do_target.rs:229` | Current resolver accepts only hidden `Act`, `Proc`, and `Workflow`; all explicit target args reject before dictionary selection. | TASK-902 replaces the explicit-args rejection for exactly one audited hole target shape after TASK-901 elaboration. It must keep wrong-shape diagnostics separate from missing `Monad<Result<_, E>>` evidence. |
| A7 | Type-function normalizer and no-inversion boundary | `crates/ash-typeck/src/normalizer.rs:422` non-inverting boundary docs, `normalizer.rs:1262` `TypeFunctionPattern::Wildcard`, `crates/ash-typeck/tests/task_825_non_inverting_unification_boundary.rs`, `task_838_type_function_normalizer.rs` | Type-function wildcard patterns participate in source equation matching. Normalizer/equality deliberately do not solve inputs from outputs. | TASK-901/TASK-902 must reject or defer holes under neutral type-function or associated-family output contexts rather than inverting. Type-function `_` remains pattern matching only. |
| A8 | Engine summary/transport boundary | `crates/ash-engine/src/module_loader.rs`, `crates/ash-engine/src/lib.rs`, existing summary transport tests `crates/ash-engine/tests/task_854_type_computation_summary_acceptance.rs`, `task_879_*`, `task_896_*` | Engine transports semantic summaries and asks TypeEnv/core validation to own type semantics. | No Phase 119 MVP engine semantics are required before summary-visible carriers exist. Engine remains transport-only; TASK-902 evidence may include non-interference if public summaries start carrying partial-target facts. |

## Enabled and disabled positions

Enabled in Phase 119 MVP:

```ash
do:Result<_, ParseError> {
    return value
}
```

Disabled or separate in Phase 119 MVP:

```ash
do:Result { return value }          // bare higher-arity target: reject with hole hint
do:Foo<_, _, E> { return value }    // multiple holes: reject
workflow f() -> Result<_, E> { }    // ordinary return type hole: not enabled by this audit
type Alias = Result<_, E>;          // ordinary type alias hole: not enabled by this audit
type fn F(xs: List) -> Type {       // pattern wildcard is separate
    case F(_) = Int;
}
```

## Downstream focused verification commands

TASK-899 core carriers:

```bash
cargo test -p ash-core --test task_899_type_hole_partial_application_carriers
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-900 parser surface:

```bash
cargo test -p ash-parser --test task_900_type_hole_surface
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-901 TypeEnv partial-constructor kinding:

```bash
cargo test -p ash-typeck --test task_901_partial_constructor_kinding
cargo fmt --check
git diff --check
cargo check --workspace
```

TASK-902 do-target integration:

```bash
cargo test -p ash-typeck --test task_902_do_target_partial_application
cargo fmt --check
git diff --check
cargo check --workspace
```

## Non-goals / traps

- Do not implement implicit currying of bare constructors.
- Do not solve holes by inverting type functions or associated families.
- Do not treat type-function pattern `_` as a source type hole.
- Do not use legacy saturated nominal constructor carriers with fake arguments to represent partial applications.
- Do not add arbitrary type lambdas, HKT interface binders, or general Monad dictionary resolution in TASK-899 through TASK-902.
- Do not move type semantics into `ash-engine`; engine remains summary transport only.

## TASK-898 verification

- Audit artifact: this file.
- Downstream fail-closed guards patched in TASK-899 through TASK-902.
- Required gate commands for TASK-898: `cargo fmt --check`, `test -f docs/plan/audits/TASK-898-type-hole-audit-gate.md`, and `git diff --check`.
