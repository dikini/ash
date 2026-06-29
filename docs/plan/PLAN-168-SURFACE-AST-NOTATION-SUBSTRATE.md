# PLAN-168: Surface AST, Notation, and Lowering Substrate

## Status: ✅ Completed

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

- [x] Map the current parser AST and lowering surfaces against `SPEC-095c` and `SPEC-098c`.
- [x] Establish an implementation plan for source-preserving surface syntax carriers.
- [x] Preserve operator-like tokens and grouped infix shape before semantic notation resolution.
- [x] Define a fail-closed implementation boundary for binary infix operator sections.
- [x] Create an explicit parsed-surface-AST to expanded-surface-AST boundary while keeping full
      macro expansion deferred.
- [x] Inventory current surface-to-Core lowering and identify exact follow-on implementation tasks.
- [x] Reconcile plan/task/changelog status and verification evidence at closeout.

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

- TASK-1721: Audit current parser AST and lowering seams against Phase 167 specs. ✅
- TASK-1722: Design the source-preserving surface syntax carrier slice. ✅

### Phase 2: Preserve notation-relevant surface shape

Tasks:

- TASK-1723: Preserve notation/operator token shape before resolution. ✅
- TASK-1724: Add the binary infix operator-section AST boundary or fail-closed diagnostics. ✅

### Phase 3: Stage expansion and lowering handoff

Tasks:

- TASK-1725: Introduce an expanded-surface-AST boundary without full macro expansion. ✅
- TASK-1726: Inventory and scope surface-to-Core lowering implementation seams. ✅
- TASK-1727: Close out Phase 168 with verification, status reconciliation, and follow-on plan notes. ✅

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

- [x] Current parser/lowering gaps against `SPEC-095c`/`SPEC-098c` are recorded in a durable audit
      artifact.
- [x] Source-preserving carrier requirements are mapped to concrete Rust modules and public/private
      API boundaries.
- [x] Notation-relevant operator token/grouping shape is either preserved by AST carriers or rejected
      explicitly until the carrier exists.
- [x] Binary infix operator sections have an implementation boundary: parsed representation or
      fail-closed diagnostics with focused tests.
- [x] Expanded-surface-AST staging exists as a named boundary, even if macro expansion is a no-op or
      explicit deferral in this phase.
- [x] Surface-to-Core lowering follow-on tasks are concrete enough for implementation without
      redoing Phase 167's spec audit.
- [x] `PLAN-INDEX.md`, this plan, task files, and `CHANGELOG.md` agree on Phase 168 status.

## Closeout evidence

TASK-1727 completed Phase 168 with the following implementation artifacts:

- `docs/audit/phase-168-surface-ast-lowering-inventory.md` records the parser AST and lowering gap
  audit from TASK-1721.
- `docs/design/phase-168-source-preserving-surface-carriers.md` records the source-preserving carrier
  design from TASK-1722.
- `RawOperatorToken`, `OperatorSection`, `OperatorSectionKind`, `SurfaceOrigin`,
  `ParsedSurfaceModule`, and `ExpandedSurfaceModule` were added in `crates/ash-parser/src/surface.rs`.
- `Expr::OperatorSection` plus parser support for `(+), (x +), (+ x)` were added in
  `crates/ash-parser/src/parse_expr.rs`.
- `expand_surface_module` now names the parsed-surface to expanded-surface boundary and rejects
  unresolved operator sections across expression-bearing module surfaces instead of checking only
  function/impl/proof bodies.
- `lower_expr` rejects unresolved operator sections instead of leaking surface-only notation into
  Core.
- `docs/audit/phase-168-surface-to-core-lowering-inventory.md` records the `SPEC-098c` lowering-family
  matrix and next-packet ordering.

Focused verification during implementation:

```bash
cargo test -p ash-parser --test task_1723_notation_token_preservation --test task_1724_operator_section_boundary --test task_1725_expanded_surface_boundary
cargo test -p ash-parser
cargo test -p ash-typeck
cargo test -p ash-engine
cargo fmt --check
cargo clippy -p ash-parser -p ash-typeck -p ash-engine --all-targets --all-features -- -D warnings
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Independent review remediation addressed:

- underscore-leading identifiers in parenthesized expressions such as `(_foo)` no longer hard-error
  as placeholder sections;
- expanded-surface operator-section detection now traverses contracts, capability implementation
  bodies, policies, laws, proof strategy expressions, workflow headers/bodies, proxy bodies, inline
  modules, and nested expression forms before Core lowering.

Deferred honestly:

- Full macro expansion and hygiene.
- General user-defined notation declarations and type-directed notation resolution.
- Generalized mixfix sections such as `(_ + _)`.
- Full `SPEC-098c` surface-to-Core lowering; next owners are listed in
  `docs/audit/phase-168-surface-to-core-lowering-inventory.md`.
