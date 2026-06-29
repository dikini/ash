# TASK-1732: Build minimal local notation-table resolution diagnostics

## Status: ✅ Complete

## Summary

Build a minimal active notation table for local module declarations and emit deterministic duplicate,
precedence, associativity, and malformed-target diagnostics before type inference. Imported/exported
notation propagation remains deferred unless the task proves the existing summary carriers can support
it without overclaiming.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c §7 and §10: notation declarations and active notation tables
- SPEC-098c §11: type inference interface for notation target resolution

## Dependencies

- ✅ TASK-1730: Notation declaration parser and AST
- ✅ TASK-1731: Built-in raw operator token preservation

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Active notation table | SPEC-095c §10 | Needed parsed declarations first | Local only | Implement local table and diagnostics | Duplicate/conflict tests pass |
| Import/export notation propagation | SPEC-095c §10 | Needs module-summary carrier design | Unknown | Implement only if carriers suffice; otherwise record explicit deferral | Audit note or tests prove no false imported-notation claim |

## Files

- `crates/ash-parser/src/surface.rs`
- new parser/expansion module if appropriate, for example `crates/ash-parser/src/expand.rs`
- `crates/ash-parser/tests/task_1732_local_notation_table_resolution.rs`
- optional docs note if import/export propagation is deferred

## Requirements

1. Collect local notation declarations into a deterministic table keyed by fixity and pattern/operator.
2. Reject duplicate declarations with stable diagnostics that include both declaration spans when
   available.
3. Reject precedence/associativity conflicts for the same operator spelling.
4. Preserve target callable paths without typechecking them in this task.
5. Document imported/exported notation as deferred if not implemented.

## Scope note

Notation tables are local to one module definition scope. Inline-module notation declarations are
validated and used only while expanding that inline module; they do not leak into the parent module,
and parent notation declarations do not implicitly propagate into inline modules. Imported/exported
notation propagation remains deferred.

## Current state

There is no active notation table; operator sections only preserve raw operator tokens and fail closed.

## Target state

Expansion can ask a local notation table whether a binary operator spelling has a local target and can
report conflicts before type inference.

## TDD steps

1. Add tests for duplicate local notation declarations.
2. Add tests for conflicting associativity/precedence declarations.
3. Add a positive test that a local infix declaration resolves to a callable path in the table.
4. Implement table collection and diagnostics.
5. If imported notation is not implemented, add an explicit deferral artifact or diagnostic text.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1732_local_notation_table_resolution
  - cargo test -p ash-parser
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Local notation table is deterministic.
  - [x] Duplicate/conflict diagnostics are stable.
  - [x] Imported/exported notation status is honest.
```

## Implementation evidence

Implemented in Phase 169 final diff. Verified with:

- `cargo test -p ash-parser --test task_1732_local_notation_table_resolution`
- `cargo test -p ash-parser`
- `cargo check --workspace`

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Provides operator-target lookup for binary operator-section elaboration.
