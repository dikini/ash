# TASK-1725: Introduce an expanded-surface-AST boundary without full macro expansion

## Status: ✅ Completed

## Summary

Create an explicit boundary between parsed surface AST and expanded surface AST. The boundary may be a
no-op or limited adapter in this phase, but it must be named, typed, tested, and honest about macro
and notation deferrals.

## Specification Reference

- PLAN-168: `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md` §3-§6
- SPEC-098c: `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md` §2

## Dependencies

- 📝 TASK-1724: Operator-section boundary

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro expansion and hygiene | SPEC-095c §6 | Requires macro system design | No | Introduce typed boundary only | Boundary returns explicit no-op/deferred status for macro calls |
| Notation resolution | SPEC-095c §7-§12 | Requires notation table/type integration | Partial | Boundary names the stage; semantic resolution deferred | Tests prove notation is not lowered into Core silently |

## Files

- `crates/ash-parser/src/surface.rs`
- Candidate expansion module identified by TASK-1722
- `crates/ash-parser/tests/task_1725_expanded_surface_boundary.rs`

## Requirements

1. Introduce a named parsed-surface to expanded-surface boundary.
2. Preserve source origin metadata through the boundary.
3. Make macro calls and unresolved notation explicit: no silent success that pretends expansion
   happened.
4. Keep existing parser/check flows working by using the boundary only where safe or by providing an
   adapter that is not yet wired into production lowering.
5. Add tests for ordinary no-op passage and explicit deferred macro/notation cases.

## TDD Steps

### Step 1: Write tests (Red)

Create `crates/ash-parser/tests/task_1725_expanded_surface_boundary.rs` proving the boundary exists,
preserves spans/origins, and reports deferred macro/notation cases honestly.

### Step 2: Implement (Green)

Add the minimal boundary carrier/function/module. Keep semantic expansion deferred unless already
supported by existing code.

### Step 3: Integration

Wire only the safe no-op/adaptor path needed by downstream tests. Do not force full lowerer migration
inside this task.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1725_expanded_surface_boundary
  - cargo test -p ash-parser
  - cargo fmt --check
  - git diff --check
  - bash scripts/check-docs-gate.sh
checklist:
  - [ ] Expanded-surface-AST boundary exists.
  - [ ] Macro/notation deferrals are explicit.
  - [ ] Existing parser behavior remains stable.
```

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the staged surface boundary consumed by TASK-1726.
