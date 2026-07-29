# TASK-2048: Language Reference for Ordinary Types, Callables, and Interfaces

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-008, LANG-009, LANG-020.

## Description

Document source-visible ordinary data/type/newtype/callable forms, generic and kind annotations,
interfaces, and implementations without conflating their static evidence with an executable route.

## Requirements

- Create `docs/reference/language/types/index.md`, `data-newtypes-and-callables.md`, and
  `generics-kinds-interfaces-and-impls.md`.
- Verify exact spellings, visibility, arity/kind constraints, constructor/pattern use, and callable
  arrow rules from live parser/typeck paths.
- Identify selected Engine/library execution evidence separately from accepted type declarations.
- Document `capability Name` only as a source type spelling: trace its type lowering and selected
  entry binding validation, cross-link TASK-2050's authority page, and distinguish it from
  excluded top-level capability declarations and non-granting authority metadata.
- Exclude historical `Fn(...)` and tower callable arrows; do not substitute target syntax where
  the parser rejects it.

## Handoffs and dependencies

- **Consumes:** parser type definitions, `surface.rs::Type`, `ash-typeck`, and module summary
  paths.
- **Evidence:** `cargo test -p ash-parser --test task_782_modulefile_type_surface`, `--test
  task_960_reserved_callable_arrows`, `--test task_910_hkt_diagnostics_surface`; `cargo test -p
  ash-typeck --test task_959_pure_closure_arrow`; `cargo test -p ash-parser --test stdlib_parsing
  test_runtime_args_usage_surface`; selected Engine stdlib constraint tests.
- **Produces:** terminology and links consumed by TASK-2049 through TASK-2051.
- **Non-goals:** `dtype`, type-level evaluation pages, arbitrary interface-method runtime,
  historical tower forms, or inferred public API from internal enum variants.

## TDD and verification steps

1. Build a declaration-to-parser/typeck/Engine evidence table before page writing.
2. Verify accepted and removed callable spellings with positive/negative parser fixtures.
3. Render EBNF and only evidence-backed typing sequents.

## Completion checklist

- [ ] Type pages distinguish ordinary source types from type-level computation.
- [ ] Callable and interface claims cite current tests.
- [ ] Static-only and runtime-bounded examples are labelled.
- [ ] Removed forms never appear as current examples; indexes/changelog/PLAN-INDEX are updated.
