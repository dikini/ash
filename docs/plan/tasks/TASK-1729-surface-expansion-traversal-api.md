# TASK-1729: Add reusable surface traversal for expansion diagnostics

## Status: ✅ Complete

## Summary

Replace the Phase 168 ad-hoc operator-section scan with reusable traversal helpers that can visit all
expression-bearing module, workflow, contract, policy, law, proof, capability, proxy, and inline-module
surfaces before expansion and lowering.

## Specification Reference

- PLAN-169: `docs/plan/PLAN-169-SURFACE-EXPANSION-AND-NOTATION-ELABORATION.md`
- SPEC-095c §3-4: parsed and expanded surface AST layers
- SPEC-098c §2 and §10: expanded-surface input and notation erasure

## Dependencies

- ✅ TASK-1725: Expanded-surface AST boundary
- 📝 TASK-1728: Phase 169 plan packet

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Reusable traversal API | Phase 168 review remediation | Ad-hoc traversal was sufficient for one carrier only | Yes | Implement reusable visitor/walker | Existing operator-section rejection tests still pass and new traversal coverage tests pass |

## Files

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/tests/task_1729_surface_expansion_traversal.rs`

## Requirements

1. Add traversal helpers for `ModuleFile`, `Definition`, `Expr`, `Workflow`, `Contract`, `PolicyDef`,
   law/proof surfaces, capability implementation bodies, proxy bodies, and inline module definitions.
2. Keep traversal read-only for this task; mutation/rewriting can be introduced later if needed.
3. Route `expand_surface_module` unresolved-operator-section detection through the reusable traversal.
4. Add tests that place an operator section in at least three non-function surfaces and prove the same
   boundary rejects all of them.
5. Avoid broad wildcard arms that silently skip future expression-bearing variants.

## Current state

`expand_surface_module` rejects unresolved operator sections, but the traversal is local to that
specific diagnostic pass.

## Target state

Expansion diagnostics use reusable traversal helpers that future notation, macro, and lowering checks
can share.

## TDD steps

1. Write `task_1729_surface_expansion_traversal.rs` with negative cases for contract, workflow, and
   capability/proxy/policy expression sites.
2. Refactor the existing scan into reusable traversal helpers.
3. Keep the Phase 168 focused tests passing.
4. Run workspace check if public surface enum or traversal API shape changes.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1729_surface_expansion_traversal
  - cargo test -p ash-parser --test task_1725_expanded_surface_boundary
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Traversal covers module/workflow/contract/capability expression surfaces.
  - [x] `expand_surface_module` uses the traversal API.
  - [x] No surface-only operator section reaches lowering through tested sites.
```

## Implementation evidence

Implemented in Phase 169 final diff. Verified with:

- `cargo test -p ash-parser --test task_1729_surface_expansion_traversal`
- `cargo test -p ash-parser --test task_1725_expanded_surface_boundary`
- `cargo test -p ash-parser`
- `cargo check --workspace`

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Dependencies for next task

Provides the traversal substrate used by notation declaration diagnostics and later expansion passes.
