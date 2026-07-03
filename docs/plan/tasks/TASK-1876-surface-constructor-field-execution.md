# TASK-1876: Surface Constructor Field Execution

**Status:** Complete
**Plan:** [PLAN-186](../PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md)

## Description

Close the runtime gap found while verifying function-first CLI entries with ordinary surface records, ADTs, pattern matching, calls, and `do` sequencing.

## Requirements

- A function-first `fn main` source with named constructor payloads must execute through the ordinary engine path.
- Field projection on a named constructor payload must return the matching payload field when the surface typechecker accepts the projection.
- Synchronous and async expression evaluation must use the same projection behavior.
- Preserve existing record projection, list-index projection, missing-field diagnostics, and type-mismatch behavior.

## TDD Steps

1. RED: Extend the Phase 185/186 surface entry regression so the rich records/ADTs/match/call/`do` fixture executes and returns its projected field value.
2. GREEN: Teach interpreter field projection to accept named `Value::Variant` payload fields in both sync and async evaluation.
3. REGRESSION: Re-run focused engine, CLI, formatting, clippy, docs, and CLI probe checks.

## Completion Checklist

- [x] RED captured and recorded.
- [x] GREEN captured and recorded.
- [x] Focused engine and CLI regressions pass.
- [x] CHANGELOG.md and Phase 186 evidence updated.

## Evidence

- RED: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry fn_main_source_composes_records_adts_match_calls_and_do_without_workflow` failed because execution reported `type mismatch: expected record, got Variant { name: "UserPayload", fields: [("name", String("Ada")), ("age", Int(41))] }`.
- GREEN: the same focused test passed after interpreter field projection accepted named constructor payload fields through the same helper used by sync and async expression evaluation.
- REGRESSION: `cargo run -q -p ash-cli -- check`, `cargo run -q -p ash-cli -- run --dry-run`, and `cargo run -q -p ash-cli -- run` on the rich function-first fixture passed, with execution returning `41`.
