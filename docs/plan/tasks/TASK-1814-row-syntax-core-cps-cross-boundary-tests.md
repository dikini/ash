# TASK-1814: Add parser/engine/Core/CPS cross-boundary row preservation tests

## Status: ✅ Complete

## Description

Add cross-boundary tests proving Phase 177's bounded row slice works across parser/module validation evidence and Core/CPS row carriers without claiming source-to-Core row lowering. This task focuses on integration evidence, negative authority-leakage tests, and explicitly recording the current validation-only source-to-typechecker boundary.

TASK-1807 identified the current source-row boundary as validation-only: parser rows are retained in surface syntax, but engine/typechecker lowering paths still convert callable types into rowless `Type::Fn` values. TASK-1814 must either bridge that boundary with focused summaries/Core conversion or record the remaining validation-only boundary for TASK-1815 closeout.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-098c](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- TASK-1809 through TASK-1813 complete or explicitly re-scoped.

## Requirements

### Functional Requirements

1. Add parser tests proving row syntax parses with spans and validation boundaries.
2. Add engine/module validation tests proving public summaries preserve row requirements without activating providers or handlers.
3. Add Core tests proving surface-originated row families can be represented as Core rows, or explicitly fail closed if a source-to-Core bridge remains out of scope after the audit.
4. Add Core-to-CPS tests proving supported row families survive lowering.
5. Add negative tests proving row mentions do not grant provider authority, role admission, workflow admission, host primitive access, or handler installation.
6. Add CLI/check fixture coverage if the row syntax is user-facing enough by this task.
7. Record any still-deferred cross-boundary case in TASK-1815 rather than hiding it in passing tests.

### Property Requirements

- Cross-boundary tests must cover both positive preservation within each implemented boundary and negative leakage.
- A narrow parser-only test cannot be used as evidence for Core/CPS alignment.
- A Core-only test cannot be used as evidence that surface syntax is integrated.
- Validation-only preservation must be named as a deferral rather than treated as end-to-end success.

## TDD Steps

### Step 1: Write failing cross-boundary tests

Add tests in the affected crates identified by TASK-1807. Prefer focused new files named after TASK-1814.

### Step 2: Verify RED

Run the focused cross-boundary tests and confirm they fail for real missing integration, not fixture mistakes.

### Step 3: Patch integration gaps

Patch parser summaries, module validation, surface-to-Core conversion, or Core-to-CPS bridging only as needed to satisfy the cross-boundary requirements.

### Step 4: Verify GREEN

Run focused tests plus `cargo test -p ash-parser`, `cargo test -p ash-core`, `cargo test -p ash-engine`, and any CLI fixture tests added.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - cargo test -p ash-core
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - git diff --check
checklist:
  - [x] Positive preservation tests cover parser/engine retention and independent Core/CPS carrier preservation.
  - [x] Negative leakage tests cover authority/admission/provider non-grants.
  - [x] Any still-deferred cross-boundary case is recorded for closeout.
```

## Dependencies for Next Task

This task feeds TASK-1815.

## Completion Evidence

- Added parser cross-boundary tests proving inline and expanded row carriers retain non-empty
  source spans before validation.
- Added an engine/module import test proving imported public callable signatures preserve
  source row carriers while current typechecker signature conversion remains deliberately
  rowless (`Type::Fn`), naming the validation-only source-to-typechecker boundary for closeout.
- Added a typechecker non-authority regression proving supported row mentions validate without
  creating runtime resource or capability authority provenance.
- Added Core/CPS regression coverage proving supported closed Core row families lower to
  kind-specific CPS `EffectItemKind` rows.
- Focused verification passed for `task_1814_row_cross_boundary_parser`,
  `task_1814_row_cross_boundary_engine`, `task_1814_row_cross_boundary_non_authority`, and
  `task_1814_core_cps_row_preservation`.
