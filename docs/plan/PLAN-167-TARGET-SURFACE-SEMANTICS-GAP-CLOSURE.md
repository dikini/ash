# PLAN-167: Target Surface and Semantics Gap Closure

## Status: ✅ Complete

## Overview

Phase 167 turns the preserved target-spec gap audit into a sequenced documentation-only
spec-hardening packet. The phase closes the gaps that currently block parser, macro, lowering, and
operational-semantics implementation work: target surface grammar drift, source-preserving surface
AST design, notation and operator sections, general surface-to-Core lowering, surface type
inference, and target operational semantics.

This phase does not implement parser, type-checker, Core, CPS, or runtime code. It produces
implementation-grade specifications and cross-spec reconciliation only. Because the work is
documentation-only, completion requires docs verification gates, not Rust build/test gates.

## Source audit

- `docs/audit/2026-06-29-target-spec-notes-gap-audit.md`

The audit records five top-level gaps:

1. `SPEC-099b` still describes Phase 159 CPS interpreter semantics instead of the current target
   language semantics.
2. The target specs lack a source-preserving surface AST and macro substrate suitable for a future
   `syn`-like library.
3. The target specs lack custom prefix/infix/suffix/mixfix notation and operator sections.
4. General expanded-surface-AST-to-Core lowering is missing.
5. Canonical fact/evidence syntax, export, and discharge interactions remain under-specified.

## Goals

- [x] Patch immediate `SPEC-095b` target grammar drift without overloading the grammar spec.
- [x] Add a companion `SPEC-095c` for surface AST, macros, notation, and operator sections.
- [x] Add a general surface-to-Core lowering spec that consumes the expanded surface AST.
- [x] Tighten surface type-inference rules affected by rows, facts/evidence, handler markers,
      operation identities, notation, and operator sections.
- [x] Rewrite or split target operational semantics so `SPEC-099b` owns current target behavior,
      not only Phase 159 interpreter behavior.
- [x] Keep `docs/spec/SPEC-INDEX.md`, `CHANGELOG.md`, and the audit cross-references current.

## Non-goals

- No parser implementation.
- No macro expander implementation.
- No Core/CPS/runtime implementation.
- No Rust test or clippy gate required by this phase.
- No final design for typed macros or binder-introducing mixfix. Those may be listed as future work.

## Phase structure

### Phase 1: Surface grammar, AST, macros, and notation

This phase makes the surface layer safe to build on. It removes stale grammar claims, adds the
source-preserving AST/macro substrate, and defines notation/operator sections as callable sugar that
is gone before Core.

Tasks:

- TASK-1709: Patch target grammar drift in `SPEC-095b`. ✅
- TASK-1710: Create `SPEC-095c` with syntax-tree layers and macro expansion boundaries. ✅
- TASK-1711: Add notation declarations and expansion rules to `SPEC-095c`. ✅
- TASK-1712: Add operator sections and callable-section typing to `SPEC-095c`. ✅
- TASK-1713: Reconcile Phase 1 cross-references and stale claims. ✅

### Phase 2: Surface-to-Core lowering and type inference

This phase defines the bridge from expanded surface syntax to Core. It should consume the Phase 1
expanded AST, not raw parse syntax.

Tasks:

- TASK-1714: Create the surface-to-Core lowering spec scaffold and pipeline invariants. ✅
- TASK-1715: Specify lowering for callables, rows, `do`, handlers, and impl operation identity. ✅
- TASK-1716: Specify lowering for facts, evidence, contracts, trace contracts, notation, and macros. ✅
- TASK-1717: Tighten surface type inference in `SPEC-097b` for the new surface/lowering rules. ✅

### Phase 3: Target operational semantics

This phase rewrites the semantics layer after the surface and lowering boundaries are stable.

Tasks:

- TASK-1718: Rewrite `SPEC-099b` scope and preserve Phase 159 interpreter semantics as context. ✅
- TASK-1719: Add target Core big-step and Core/CPS small-step semantics. ✅
- TASK-1720: Integrate contracts, providers, traces, monitors, lazy/memo timing, and closeout. ✅

## Dependency graph

```text
TASK-1709
  -> TASK-1710
      -> TASK-1711
          -> TASK-1712
              -> TASK-1713
                  -> TASK-1714
                      -> TASK-1715
                          -> TASK-1716
                              -> TASK-1717
                                  -> TASK-1718
                                      -> TASK-1719
                                          -> TASK-1720
```

The sequence is intentionally linear. Each task is small enough for focused review, and each task
leaves the document set in a coherent state for the next task.

## Verification policy

This phase is documentation-only. Each task must run docs verification rather than Rust build gates:

```bash
git diff --check
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Task-specific structural assertions may be added where useful, for example checking that a new spec
file exists, a phrase no longer appears in live normative text, or `SPEC-INDEX.md` links the new
spec.

## Acceptance criteria

- [x] `SPEC-095b` no longer teaches stale inline contract-handler syntax as target syntax.
- [x] `SPEC-095b` reconciles trace contract syntax with `SPEC-096b` or explicitly defers it.
- [x] `SPEC-095b` no longer claims a closed operator future.
- [x] `SPEC-095c` defines a source-preserving surface AST and macro/notation substrate suitable
      for a future `syn`-like library.
- [x] `SPEC-095c` defines prefix, infix, suffix, mixfix notation and binary infix operator
      sections as callable sugar.
- [x] The lowering spec consumes expanded surface AST and erases macros, notation, and sections
      before Core.
- [x] `SPEC-097b` states the surface type-inference rules needed by rows, evidence, handlers,
      operation identity, notation, and sections.
- [x] `SPEC-099b` no longer presents Phase 159 CPS-interpreter semantics as the full target
      semantics.
- [x] Target operational semantics includes provider frames, structured traps, contract checks,
      lazy/memo force timing, trace facts, and temporal monitor behavior.
- [x] `docs/spec/SPEC-INDEX.md` has read paths for target surface syntax/AST/lowering/semantics.
- [x] `CHANGELOG.md` records the spec-hardening packet.

## Closeout evidence

- Implemented on 2026-06-29.
- Documentation-only verification: `git diff --check`, `python3 tools/docs/validate_orientation_indexes.py --self-test`, and `bash scripts/check-docs-gate.sh`.
- Audit gaps A-E are closed at spec-planning level; implementation remains future work.
- Post-review remediation on 2026-06-29 reconciled the phase-local task list and removed the stale closeout-evidence template.
