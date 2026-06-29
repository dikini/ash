# TASK-1734: Add expanded-surface-to-Core lowering gate

## Status: 📝 Planned

## Summary

Introduce a high-level lowering entry point that requires `ExpandedSurfaceModule` or an equivalent
validated carrier before Core lowering, preventing parsed-surface-only notation and operator sections
from bypassing expansion.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-098c §1-2: expanded surface AST as lowering input
- SPEC-098c §10: notation/operator-section erasure before Core

## Dependencies

- 📝 TASK-1733: Operator-section elaboration

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Lowering input is expanded surface AST | SPEC-098c §1-2 | Phase 168 only named the boundary | Yes | Add high-level gate while preserving low-level helpers for tests | Tests prove module lowering calls expansion first |
| Full surface-to-Core lowering | SPEC-098c overall | Requires many later packets | No | Gate only; keep unsupported forms explicit | Lowering inventory remains honest |

## Files

- `crates/ash-parser/src/lower.rs`
- `crates/ash-parser/src/surface.rs`
- engine or module-loader call sites that lower parser modules, if any
- `crates/ash-parser/tests/task_1734_expanded_surface_lowering_gate.rs`

## Requirements

1. Add a high-level lowering function that accepts an `ExpandedSurfaceModule` or runs expansion before
   lowering a parsed module.
2. Keep low-level `lower_expr` available for focused expression tests, but document that it is not the
   module boundary.
3. Add tests proving unresolved surface-only nodes are rejected at the expansion gate before Core.
4. Add tests proving elaborated operator sections can proceed through the high-level gate.
5. Update call sites conservatively; do not rewrite unrelated lowering semantics.

## Current state

`lower_expr` rejects raw operator sections, but high-level lowering boundaries do not yet uniformly
require an expanded module carrier.

## Target state

Module-level lowering has an explicit expansion gate. Parsed-surface-only syntax cannot accidentally
bypass notation expansion.

## TDD steps

1. Add a failing test that attempts high-level lowering with an unresolved section in a module surface.
2. Add a positive test for a resolved/elaborated section after TASK-1733.
3. Implement or route through the expanded module gate.
4. Update docs/comments around low-level helpers.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1734_expanded_surface_lowering_gate
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [ ] High-level lowering requires expansion.
  - [ ] Low-level helpers remain honest and fail closed.
  - [ ] Existing engine/module-loading tests still pass.
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for next task

Provides the final implementation seam that Phase 169 closeout must verify.
