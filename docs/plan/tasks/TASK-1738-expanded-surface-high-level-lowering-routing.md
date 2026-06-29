# TASK-1738: Route high-level module/file lowering through expanded-surface validation

## Status: 📝 Planned

## Summary

Use the TASK-1737 audit to route high-level module/file lowering paths through expansion before Core lowering, while keeping low-level parser/test helpers available and fail-closed.

## Specification Reference

- PLAN-170: high-level expanded-surface boundary
- SPEC-098c §10-11: surface expansion before Core lowering
- PLAN-169 TASK-1734: initial expanded-surface gate

## Dependencies

- 📝 TASK-1737: Expanded-surface boundary call-site audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Lowering bypass risk | Phase 169 final review | Helper gate existed but public callers could bypass it | Audit pending | Route high-level paths identified by TASK-1737 | Positive high-level tests and negative low-level fail-closed tests |

## Requirements

1. Update only high-level module/file/workflow loading boundaries identified by TASK-1737.
   - Primary target: `Engine::check_module_file` in `crates/ash-engine/src/lib.rs` should validate the authoritative parsed `ModuleFile` through `ash_parser::lower::expand_and_lower_surface_module` or equivalent after successful full-module parsing.
   - Primary target: `module_loader::collect_module_exports` in `crates/ash-engine/src/module_loader.rs` should validate the full parsed module through the same expanded-surface gate before public callable export collection.
   - Non-target: low-level `lower_expr`, `lower_workflow`, `lower_workflow_def`, and type-metadata-only helpers remain available and fail-closed.
2. Preserve low-level `lower_expr` and related parser-local APIs for tests, but document them as post-expansion or fail-closed helpers.
3. Add regression tests showing unresolved surface-only syntax is rejected through high-level engine/module paths.
4. Add positive tests showing elaborated operator sections and local notation still lower through the routed path.
5. Avoid broad `SPEC-098c` lowering work unrelated to expansion gating.

## TDD Steps

1. Add failing high-level tests for an unresolved operator section in an engine/module-loader path.
   - Flip `crates/ash-engine/tests/task_1737_expanded_surface_boundary_audit.rs` so `check_module_file` rejects `(<*>)` in a public function body.
   - Add an import/export negative test proving a module with an unresolved public callable body fails during import/export loading instead of silently skipping the callable.
2. Add positive tests for a Phase 169-resolved section through the same high-level path.
   - Add a `check_module_file` positive case for a built-in section such as `(+)` after expansion.
   - Add a local notation positive case only if the function body is otherwise lowerable; otherwise document why local notation remains parser/lowering-only until TASK-1740.
3. Route the audited high-level path through expansion.
4. Re-run parser, typeck, and engine gates.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1734_expanded_surface_lowering_gate
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - cargo check --workspace
  - cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] High-level paths identified by TASK-1737 are routed or explicitly deferred.
  - [ ] Low-level bypasses still reject raw surface-only nodes.
  - [ ] Existing accepted syntax remains stable.
```
