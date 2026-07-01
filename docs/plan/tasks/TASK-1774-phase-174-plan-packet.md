# TASK-1774: Create the Phase 174 plan packet

## Status: ✅ Complete

## Description

Create and register the Phase 174 planning packet for macro-aware tooling, summary identity, and inference readiness. This task is documentation/planning only; it does not implement Rust behavior.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)

## Dependencies

- ✅ Phase 173 closeout (complete)
- ✅ Post-Phase-173 deferred work audit (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| LSP macro UX/cache debt | `docs/audit/phase-173-macro-system-expansion-audit.md` | LSP presented macros as function-like and parse summaries omitted macro details | Yes: Phase 173 carriers exist | Plan Phase 174 around tooling honesty and cache identity | TASK-1775/TASK-1776 audit and model |
| Ordinary-call macro inference | TASK-1772 | No proven callable identity substrate | Partially: typed macro carriers exist, callable identity not proven | Plan audit first, then bounded inference only for unique identities | TASK-1779/TASK-1780 |
| Imported notation propagation | PLAN-170 follow-on | No notation summary carriers | No: distinct from macro summaries | Keep deferred | Future notation-summary phase |

## Requirements

### Functional Requirements

- [x] Create `PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md`.
- [x] Create TASK-1774 through TASK-1783 task files.
- [x] Register Phase 174 in `PLAN-INDEX.md`.
- [x] Add a `CHANGELOG.md` entry under `[Unreleased]`.

### Property Requirements

- The planning packet must not claim Phase 174 implementation is complete.
- The task range must be globally unique and linkable from PLAN-INDEX.
- The plan must keep macro metadata syntax-phase-only and avoid runtime callable overclaims.

## TDD Steps

### Step 1: Inspect current state

Read `PLAN-INDEX.md`, `CHANGELOG.md`, Phase 173 closeout docs, and relevant LSP/parser surfaces before assigning task IDs.

### Step 2: Write planning artifacts

Create the Phase 174 plan and task files with concrete current-to-target transitions, dependencies, dispatch metadata, and verification commands.

### Step 3: Register planning surfaces

Update `PLAN-INDEX.md` and `CHANGELOG.md` only after task files exist.

### Step 4: Verify structure

Run scoped structural checks over the new plan/task files, `PLAN-INDEX.md`, and `CHANGELOG.md`.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 10
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Plan file exists
  - [x] Task files TASK-1774 through TASK-1783 exist
  - [x] PLAN-INDEX row and phase section exist
  - [x] CHANGELOG entry exists
```

## Completion Evidence

Created the Phase 174 planning packet and task files, registered the phase in PLAN-INDEX, and added an Unreleased changelog entry. TASK-1774 is complete because it is the planning-packet creation task; implementation tasks remain planned.

## Dependencies for Next Task

This task outputs the registered Phase 174 plan used by TASK-1775.
