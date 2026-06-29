# TASK-1737: Audit expanded-surface boundary and direct-lowering call sites

## Status: ✅ Complete

## Summary

Audit all parser, lowerer, engine, module-loader, LSP, and typecheck call paths that can lower or consume parsed surface syntax without first passing through the expanded-surface boundary.

## Specification Reference

- PLAN-170: expanded-surface integration audit
- SPEC-098c §10-11: macro/notation erasure and lowering interface
- PLAN-169 TASK-1734: expanded-surface lowering gate

## Dependencies

- ✅ TASK-1736: Phase 170 packet created

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full high-level lowering enforcement | TASK-1734 review | Phase 169 added helper gates but did not prove every high-level path uses them | Yes | Audit and classify every call site before routing | Audit artifact with exact file/function list |

## Requirements

1. Inventory every public function in `crates/ash-parser/src/lower.rs` that accepts surface AST types.
2. Inventory engine/module-loader call sites that parse and lower module files or workflow definitions.
3. Classify each call site as high-level boundary, low-level/test helper, parser-local helper, or deferred.
4. Record whether raw `Expr::OperatorSection`, notation declarations, and future surface-only nodes can bypass expansion.
5. Produce an audit artifact under `docs/audit/phase-170-expanded-surface-boundary-audit.md`.
6. Patch TASK-1738 with exact target functions/tests based on the audit if needed.

## TDD Steps

1. Add audit-only assertions or focused tests that demonstrate at least one high-level path requiring routing.
2. Write the audit artifact with exact file/function rows.
3. Run existing Phase 169 boundary tests to establish baseline behavior.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1734_expanded_surface_lowering_gate
  - cargo test -p ash-engine
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] All public lowering APIs are classified.
  - [x] Engine/module-loader paths are classified.
  - [x] TASK-1738 has concrete targets or a documented no-op rationale.
```

## Closeout evidence

- Audit artifact: `docs/audit/phase-170-expanded-surface-boundary-audit.md`.
- Audit proof test: `crates/ash-engine/tests/task_1737_expanded_surface_boundary_audit.rs`.
- Fresh verification:
  - `cargo test -p ash-engine --test task_1737_expanded_surface_boundary_audit -- --nocapture`
  - `cargo test -p ash-parser --test task_1734_expanded_surface_lowering_gate`
  - `cargo test -p ash-engine`
  - `cargo check --workspace`
  - `cargo fmt --check`
  - `git diff --check`
