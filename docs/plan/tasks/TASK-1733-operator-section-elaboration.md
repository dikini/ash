# TASK-1733: Elaborate binary operator sections to callable surface forms

## Status: ✅ Complete

## Summary

Elaborate binary operator sections `(+), (x +), (+ x)` from parsed-surface `Expr::OperatorSection` into
ordinary callable surface expressions after built-in or local notation resolution, preserving source
spans on generated forms and fail-closed behavior for unresolved or unsupported operators. Full
origin sidecar threading remains deferred to the later source-origin metadata packet.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c §8-9: operator sections and typing/authority invariants
- SPEC-098c §10: macro, notation, and operator-section erasure

## Dependencies

- ✅ TASK-1729: Reusable expansion traversal
- ✅ TASK-1732: Local notation-table resolution diagnostics

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Binary operator-section elaboration | SPEC-095c §8 / Phase 168 | Sections were only preserved/fail-closed | Yes | Elaborate built-in/local binary sections | Lowering no longer sees resolved binary sections |
| General mixfix sections | SPEC-095c §8 | Needs binder/mixfix partial-application model | No | Keep rejected/deferred | Negative test for `(_ + _)` or equivalent unsupported shape |

## Files

- `crates/ash-parser/src/surface.rs`
- expansion module or helper introduced by TASK-1732
- `crates/ash-parser/src/lower.rs`
- `crates/ash-parser/tests/task_1733_operator_section_elaboration.rs`

## Requirements

1. Resolve bare, left, and right binary sections against built-in operators and local notation targets.
2. Elaborate to ordinary callable surface expressions or closures that the existing lowerer/typechecker
   can consume.
3. Preserve source spans on generated nodes; full origin sidecar threading is deferred.
4. Preserve latent authority by relying on the resolved callable's later type/row checking.
5. Keep unresolved or unsupported sections fail-closed before Core lowering.

## Current state

`Expr::OperatorSection` parses and is rejected by expansion/lowering if unresolved.

## Target state

Resolved binary sections are removed from expanded surface syntax; unresolved sections still fail
closed with diagnostics.

## TDD steps

1. Add tests for built-in bare, left, and right sections elaborating before lowering.
2. Add tests for local notation-target sections elaborating to the declared callable path.
3. Add negative tests for unresolved operators and generalized mixfix placeholder syntax.
4. Implement elaboration in the expansion boundary.
5. Prove `lower_expr` is still fail-closed for any raw section that bypasses expansion.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1733_operator_section_elaboration
  - cargo test -p ash-parser --test task_1724_operator_section_boundary
  - cargo test -p ash-parser --test task_1725_expanded_surface_boundary
  - cargo test -p ash-typeck
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Bare/left/right binary sections elaborate.
  - [x] Unresolved sections still fail closed.
  - [x] Generated forms preserve section source spans; full origin sidecars are explicitly deferred.
```

## Implementation evidence

Implemented in Phase 169 final diff. Verified with:

- `cargo test -p ash-parser --test task_1733_operator_section_elaboration`
- `cargo test -p ash-parser --test task_1724_operator_section_boundary`
- `cargo test -p ash-parser --test task_1725_expanded_surface_boundary`
- `cargo test -p ash-typeck`
- `cargo check --workspace`

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for next task

Produces expanded-surface expressions that the lowering gate can accept without raw sections.
