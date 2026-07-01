# TASK-1782: Reconcile specs, docs, and indexes for Phase 174 boundaries

## Status: ✅ Complete

## Description

Update the canonical documentation surfaces so Phase 174's macro-aware tooling and callable-identity boundaries are recorded without overclaiming runtime semantics or broad macro power. This is a documentation reconciliation task after implementation tasks land.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-1775 through TASK-1781 (all complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| LSP macro UX documented only in audit | Phase 173/174 audits | Implementation not landed | After TASK-1776-1781 | Document current behavior and limits | Spec/index checks pass |
| Callable identity inference contract | TASK-1779/1780 | New bounded behavior | After TASK-1780 | Record safe cases and rejected cases | Search finds no stale TASK-1772-only wording |

## Requirements

### Functional Requirements

1. Patch relevant sections of `SPEC-095c`, `SPEC-097b`, and/or `SPEC-098c` only where Phase 174 changes normative or target behavior.
2. Update `docs/spec/SPEC-INDEX.md` if spec scope/read paths change.
3. Update `docs/plan/PLAN-174-...md` with implementation evidence for completed tasks.
4. Update `CHANGELOG.md` entries for implementation tasks under `[Unreleased]`.
5. Sweep for stale wording that says LSP macros are function-like or ordinary-call macro inference is always impossible.

### Property Requirements

- Documentation must distinguish LSP/tooling presentation from parser/typechecker/runtime semantics.
- Callable identity inference docs must list rejection boundaries as first-class behavior.
- No spec text may claim generalized mixfix, macro-by-example, imported notation activation, or runtime macro authority.

## TDD Steps

### Step 1: Search stale claims

Search docs and LSP code comments for function-like macro wording, old Phase 173 audit claims now superseded, and TASK-1772-only inference limitations.

### Step 2: Patch specs and indexes

Patch only the sections required by implemented behavior. Preserve historical audit artifacts unless adding a clear supersession note is necessary.

### Step 3: Update changelog and plan evidence

Add task-specific entries and evidence for completed Phase 174 tasks.

### Step 4: Verify docs gates

Run orientation index validation and docs gate scripts.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - git diff --check
  - cargo fmt --check
checklist:
  - [x] Relevant specs/docs reflect Phase 174 behavior
  - [x] No stale macro-as-function tooling wording remains in docs
  - [x] CHANGELOG records implementation changes
```

## Dependencies for Next Task

TASK-1783 depends on reconciled status/docs surfaces.

## Completion Evidence

- Updated SPEC-095c, SPEC-038, SPEC-INDEX, CHANGELOG, audits, and Phase 174 plan/status surfaces for the implemented boundaries.
