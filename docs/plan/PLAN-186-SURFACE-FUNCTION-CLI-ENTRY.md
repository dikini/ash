# PLAN-186: Surface Function CLI Entry

**Status:** Complete
**Depends on:** Phase 185 Surface Function Language.
**Specs/notes:** `SPEC-095b`, `SPEC-098c`, `SPEC-099b`, `SPEC-100`, and `PLAN-185`.

## Goal

Make the command-line user path match the target surface language: ordinary `fn main` sources are runnable and checkable entry files, and legacy `workflow` entry handling remains compatibility/runtime-profile routing rather than the default language shape.

## Scope

This phase closes the first CLI-facing gap after Phase 185:

- `ash run --dry-run path.ash` must parse and typecheck a source whose entry is `fn main` and which contains no `workflow` block;
- ordinary `ash run path.ash` must keep executing that same `fn main` source through the engine path;
- diagnostics and comments should describe entries as Ash source/function entries where practical, without pretending that workflow syntax is the core language path;
- function-first sources must not emit legacy workflow warnings because of internal runtime adapters;
- specs and indexes should point CLI entry work at the surface-function path.

## Non-Goals

- No removal of legacy `workflow` source compatibility.
- No broad CLI terminology rewrite outside the touched run/check path.
- No new runtime-profile semantics.
- No row inference beyond existing explicit row support.

## Tasks

| Task | Description | Status |
|------|-------------|--------|
| [TASK-1870](tasks/TASK-1870-surface-function-cli-plan-packet.md) | Create the Phase 186 plan packet | Complete |
| [TASK-1871](tasks/TASK-1871-cli-entry-boundary-audit.md) | Audit CLI run/check entry boundaries | Complete |
| [TASK-1872](tasks/TASK-1872-run-dry-run-fn-main-entry.md) | Make `ash run --dry-run` accept function-first entry sources | Complete |
| [TASK-1873](tasks/TASK-1873-cli-entry-spec-reconciliation.md) | Reconcile CLI entry specs and indexes | Complete |
| [TASK-1874](tasks/TASK-1874-surface-function-cli-closeout.md) | Close out Phase 186 | Complete |
| [TASK-1875](tasks/TASK-1875-synthetic-entry-warning-cleanup.md) | Suppress legacy workflow warnings for synthetic `fn main` adapters | Complete |
| [TASK-1876](tasks/TASK-1876-surface-constructor-field-execution.md) | Execute function-first sources with named constructor field projection | Complete |

## Verification Evidence

- RED: `cargo test -p ash-cli fn_main_entry` failed before implementation because `test_dry_run_valid_fn_main_entry` reported `'main' has wrong return type` with expected `Result<(), RuntimeError>` and found `<missing>`.
- GREEN: `cargo test -p ash-cli fn_main_entry` passed with 2/2 selected tests after routing ordinary dry-run sources through `engine.parse_file(path)` and `engine.check`.
- Continuation RED: `cargo test -p ash-cli test_dry_run_fn_main_with_module_declaration_is_checked` failed because dry-run printed `Dry run successful` without checking a `fn main` source that also contained a module-level `policy` declaration.
- Continuation GREEN: the same focused test passed after `is_module_only_source` stopped classifying token streams containing `fn main` as module-only.
- Missing-entry RED: `cargo test -p ash-cli test_dry_run_module_without_entry_is_rejected` failed because dry-run printed `Dry run successful` for a declaration-only module with no `fn main`.
- Missing-entry GREEN: the same focused test passed after dry-run began failing declaration-only modules with `entry file has no fn main or workflow`.
- Warning RED: `cargo test -p ash-cli ash_check_function_first_entry_does_not_emit_legacy_workflow_warning` failed because `ash check` surfaced `DeprecatedLegacyWorkflowDeclaration` for a function-first source whose workflow carrier was synthetic.
- Warning GREEN: `cargo test -p ash-cli --test task_778_legacy_workflow_warning` passed with 4/4 tests after engine program parsing tracked whether the entry workflow was user-authored or synthesized from `fn main`.
- Constructor-field RED: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry fn_main_source_composes_records_adts_match_calls_and_do_without_workflow` failed because execution reported `type mismatch: expected record, got Variant { name: "UserPayload", fields: [("name", String("Ada")), ("age", Int(41))] }`.
- Constructor-field GREEN: the same focused engine regression passed after named constructor payload field projection was accepted by the interpreter.
- Constructor-field CLI probe: `ash check`, `ash run --dry-run`, and `ash run` passed for the rich function-first fixture, with execution returning `41`.

## Acceptance Criteria

- [x] A `.ash` file with `fn main() -> Int { do { return 42; } }` and no `workflow` block passes `ash run --dry-run`.
- [x] The same file executes through `ash run` and produces the function result through the existing engine path.
- [x] CLI dry-run no longer routes function-first entry files through runtime-entry workflow verification or unchecked module-only short-circuiting.
- [x] CLI dry-run rejects declaration-only modules with no runnable entry; use `ash check` for module validation.
- [x] Function-first sources do not emit legacy workflow deprecation warnings from internal runtime adapters.
- [x] Function-first sources with named constructor payload field projection execute through `ash run`.
- [x] Documentation and indexes describe this as a continuation of function-first target entry support, not as a new workflow source path.
- [x] Changelog and task evidence record RED/GREEN and verification commands.
