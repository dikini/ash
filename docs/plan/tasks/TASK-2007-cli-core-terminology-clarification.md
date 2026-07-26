# TASK-2007: CLI Core Terminology Clarification

**Status:** Complete
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)

## Description

Remove ambiguous CLI/test-runner “Core” terminology that currently denotes `ash_core::Expr`, not
the target `core_ash::CoreExpr` or CPS calculus.

## Requirements

- Inventory user-facing, schema, diagnostic, and test labels using “Core”.
- Rename or qualify labels without changing observable result schema accidentally.
- Preserve serialization compatibility or supply a versioned migration path.
- Ensure conformance documentation names the actual semantic representation.

## TDD Steps

1. Add snapshots/schema compatibility tests for current labels.
2. Add failing clarity/metadata assertions for the selected terminology.
3. Implement label/schema migration with explicit compatibility behavior.
4. Run CLI, schema, docs, and conformance gates.

## Inventory and scoped decision

The user-facing test-runner inventory found two `target_execution` repro-artifact paths using the
legacy-compatible substrate string `ash_interp_core_expr`: synthesized contract postconditions and
synthesized small-world executions. Neither string denotes target `core_ash::CoreExpr` nor the CPS
calculus. No user-facing CLI diagnostic or result-schema label named an unqualified `Core`
representation in the audited test-runner paths.

Both paths now retain `substrate: "ash_interp_core_expr"` for existing consumers and add
`representation: "ash_core::Expr"`. The additive field is the explicit clarification; it does not
rename or version the existing substrate field. This runner metadata identifies the legacy
direct-style interpreter substrate only. It does not claim Core Ash/CPS production realization or
change the canonical terminal-observable projection owned by TASK-2008.

The semantic traceability graph intentionally has no new node for this metadata-only change:
`OBS-TARGET-PROJECTION-001` covers language terminal observables, while this task preserves
test-runner repro provenance and does not establish a new semantic realization claim.

## TDD and verification evidence

`crates/ash-cli/tests/task_2007_cli_core_terminology.rs` was RED before the implementation because
the selected contract repro output had no `representation` member. It now asserts both the
unchanged compatibility substrate and `ash_core::Expr` representation. The same representation
field is emitted from the small-world path.

Verified after the change:

- `cargo test -p ash-cli --test task_2007_cli_core_terminology -- --exact`
- `cargo fmt --check`
- `cargo clippy -p ash-cli --test task_2007_cli_core_terminology -- -D warnings`
- `git diff --check`

## Completion Checklist

- [x] Every public “Core” label identifies its representation.
- [x] No result format compatibility regression is hidden by a wording edit.
- [x] Canonical documents and CLI metadata agree.
- [x] Changelog and migration notes are complete.

## Evidence required

TASK-1988 found three distinct Core dialects; this task is terminology/refinement work, not
permission to delete any semantic implementation.
