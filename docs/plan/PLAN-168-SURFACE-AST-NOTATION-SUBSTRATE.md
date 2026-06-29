# PLAN-168: Surface AST, Notation, and Lowering Substrate

## Status: 📝 Planned

## Overview

Phase 168 introduces the first implementation substrate for the surface layer specified by Phase
167. It bridges the newly hardened target specs into live parser/lowering work without attempting to
implement the full macro system in one step.

The phase starts with an inventory of the current parser AST and lowering seams, then adds or designs
minimal carriers for source-preserving surface syntax, notation-token preservation, binary infix
operator-section boundaries, expanded-surface-AST staging, and surface-to-Core lowering inventory.
The intended result is a grounded substrate that future macro, notation, type-inference, and
operational-semantics phases can build on.

## Source specs

- `docs/spec/SPEC-095a-CURRENT-GRAMMAR.md`
- `docs/spec/SPEC-095b-TARGET-GRAMMAR.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

## Goals

- [ ] Map the current parser AST and lowering surfaces against `SPEC-095c` and `SPEC-098c`.
- [ ] Establish an implementation plan for source-preserving surface syntax carriers.
- [ ] Preserve operator-like tokens and grouped infix shape before semantic notation resolution.
- [ ] Define a fail-closed implementation boundary for binary infix operator sections.
- [ ] Create an explicit parsed-surface-AST to expanded-surface-AST boundary while keeping full
      macro expansion deferred.
- [ ] Inventory current surface-to-Core lowering and identify exact follow-on implementation tasks.
- [ ] Reconcile plan/task/changelog status and verification evidence at closeout.

## Non-goals

- No full macro expander.
- No typed macro system or hygiene-complete implementation.
- No generalized mixfix partial-application implementation.
- No operational-semantics implementation beyond preserving enough origin/lowering metadata for
  later phases.
- No attempt to make every target grammar form fully parse and lower in this phase; unsupported
  forms must fail closed with explicit diagnostics or remain honestly documented as deferred.

## Phase structure

### Phase 1: Ground the current parser and AST surface

Tasks:

- TASK-1721: Audit current parser AST and lowering seams against Phase 167 specs. 📝
- TASK-1722: Design the source-preserving surface syntax carrier slice. 📝

### Phase 2: Preserve notation-relevant surface shape

Tasks:

- TASK-1723: Preserve notation/operator token shape before resolution. 📝
- TASK-1724: Add the binary infix operator-section AST boundary or fail-closed diagnostics. 📝

### Phase 3: Stage expansion and lowering handoff

Tasks:

- TASK-1725: Introduce an expanded-surface-AST boundary without full macro expansion. 📝
- TASK-1726: Inventory and scope surface-to-Core lowering implementation seams. 📝
- TASK-1727: Close out Phase 168 with verification, status reconciliation, and follow-on plan notes. 📝

## Dependency graph

```text
TASK-1721
  -> TASK-1722
      -> TASK-1723
          -> TASK-1724
              -> TASK-1725
                  -> TASK-1726
                      -> TASK-1727
```

## Implementation constraints

- Start from live code, especially `crates/ash-parser/src/surface.rs`, parser modules under
  `crates/ash-parser/src/`, and lowering/engine seams that consume parser surface nodes.
- Keep current accepted syntax stable unless a task explicitly adds fail-closed rejection for a
  previously ambiguous target-only form.
- Preserve source spans and origin metadata at every new carrier boundary.
- Treat notation and macros as surface constructs that must not leak into Core.
- Prefer narrow, testable substrate changes over speculative architecture rewrites.

## Verification policy

Each implementation task must run focused parser/lowering tests plus formatting and relevant crate
checks. Documentation-only inventory/design tasks use docs gates plus structural assertions over the
created audit/design artifact.

Baseline commands for closeout:

```bash
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
cargo fmt --check
cargo test -p ash-parser
cargo test -p ash-engine
```

If focused implementation changes touch other crates, the task must add exact crate/test commands in
its own verification block.

## Acceptance criteria

- [ ] Current parser/lowering gaps against `SPEC-095c`/`SPEC-098c` are recorded in a durable audit
      artifact.
- [ ] Source-preserving carrier requirements are mapped to concrete Rust modules and public/private
      API boundaries.
- [ ] Notation-relevant operator token/grouping shape is either preserved by AST carriers or rejected
      explicitly until the carrier exists.
- [ ] Binary infix operator sections have an implementation boundary: parsed representation or
      fail-closed diagnostics with focused tests.
- [ ] Expanded-surface-AST staging exists as a named boundary, even if macro expansion is a no-op or
      explicit deferral in this phase.
- [ ] Surface-to-Core lowering follow-on tasks are concrete enough for implementation without
      redoing Phase 167's spec audit.
- [ ] `PLAN-INDEX.md`, this plan, task files, and `CHANGELOG.md` agree on Phase 168 status.
