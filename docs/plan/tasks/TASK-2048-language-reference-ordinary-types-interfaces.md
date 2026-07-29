# TASK-2048: Language Reference for Ordinary Types, Callables, and Interfaces

**Status:** Complete
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-008, LANG-009, LANG-020.
**Semantic task classification:** non-semantic-workflow-enforcement

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

## Verification evidence

- Re-audited parser acceptance in `parse_type_def.rs`, `parse_module.rs`, and
  `parse_module::module_file`; re-audited static/lowering boundaries in `ash-typeck`,
  `lower.rs`, and `ash-engine/src/entry.rs`.
- The evidence matrix separates declaration/parser acceptance from the normal Engine
  parse/check boundary and from the selected entry verifier. It makes no general type/newtype,
  callable, interface, or implementation execution claim.

  | Documentation example or claim | Parser/static/lowering evidence | Admission/runtime status | Boundary note |
  |---|---|---|---|
  | ordinary variant `type` declaration | TASK-782 parser surface; `lower.rs` type summary route | closed | declaration route only |
  | local `newtype OrderId = OrderId(Int)` | TASK-2001 local/imported nominal checker tests | closed | normal Engine parse/check only |
  | `(Int, String) -> Bool` | TASK-957 current callable parser test | closed | parser type spelling only |
  | pure closure `|x: Int| -> x + 1` | TASK-959 typechecker test | closed | expression typechecking only |
  | `capability Args` parameter | runtime parser test; surface type lowering; entry verification | fixture-bounded | entry verification/binding only |
  | `Functor<F : * -> *>` and `Monad<Option>` evidence | TASK-910 parser/typechecker tests; Engine summary/import tests | closed | no interface dispatch evidence |

- Passed `cargo test -p ash-parser --test task_782_modulefile_type_surface --test
  task_957_callable_type_parser --test task_960_reserved_callable_arrows --test
  task_910_hkt_diagnostics_surface --test phase201_removed_syntax` (42 passed), and
  `cargo test -p ash-parser --test stdlib_parsing test_runtime_args_usage_surface` (1 passed;
  88 filtered).
- Passed `cargo test -p ash-typeck --test task_959_pure_closure_arrow --test
  task_1971_generic_signature_type_params --test task_910_hkt_acceptance_matrix --test
  task_906_hkt_fail_closed --test closed_world_interfaces_task_422 --test
  task_2001_local_newtype_identity` (46 passed).
- Passed `cargo test -p ash-engine --test entry_verification --test
  task_2001_local_nominal_newtype_checking --test task_2001_nominal_newtype_match_patterns --test
  task_1021_std_algebra_namespace_and_interfaces --test
  task_1041_interface_constraint_summary_transport --test task_1865_surface_fn_main_entry`
  (50 passed). The final test is negative admission evidence for the type-rich composite fixture.
- Rendered the two EBNF fences and two `sequent` fences using the railroad and sequent-md APIs;
  their diagnostics were empty. The sequents are explicitly bounded static/lowering rules, never
  authority or runtime semantics.
- Passed `python3 tools/docs/validate_orientation_indexes.py --self-test`,
  `bash scripts/check-docs-gate.sh`, and `git diff --check` after this task's edits.

## Completion checklist

- [x] Type pages distinguish ordinary source types from type-level computation.
- [x] Callable and interface claims cite current tests.
- [x] Static-only and runtime-bounded examples are labelled.
- [x] Removed forms never appear as current examples; indexes/changelog/PLAN-INDEX are updated.
