# TASK-1001: Matching semantics audit gate

## Status: ✅ Complete

## Description

Audit every live pattern-use callsite before implementation and replace downstream fail-closed verification guards with exact non-zero focused commands.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists

## Requirements

1. Map parser, lowering, typeck, engine, interp, CLI, and LSP pattern-use callsites.
2. Classify every pattern use as irrefutable binder, exhaustive eliminator, explicit complement eliminator, or explicit refutable filtering construct.
3. Audit `if let ... else` parser entrypoints in real module/function-body contexts and freeze accepted live syntax before TASK-1007 tests are written.
4. Audit current `if let` typechecking, especially whether `check_pattern` errors are propagated or silently ignored.
5. Split workflow/operational binders into source-level, lowered-only, and core-only callsites, including yield arms, receive arms, and core spawn/split patterns.
6. Record exact current diagnostics and runtime failure variants, distinguishing expression `LetPatternBindFailed`, workflow `PatternMatchFailed`, `NonExhaustiveMatch`, and any renamed live equivalents.
7. Add a RED-test map for wildcard/default over open or non-ADT scrutinees, blocked constructor coverage, impossible patterns, if-let scope/shadowing, if-let branch type mismatch, duplicate binders, nested refutable binders, list patterns, and selective receive guard/order/no-match behavior.
8. Patch TASK-1002 through TASK-1008 verification commands with exact tests before Rust implementation starts.

## File Targets

- Create: docs/plan/audits/TASK-1001-matching-semantics-audit-gate.md
- Modify: docs/plan/tasks/TASK-1002-*.md through TASK-1008-*.md

## TDD / Execution Steps

1. Read SPEC-076 and PLAN-126.
2. Inspect `ash_parser::surface` pattern-bearing nodes, expression parser entrypoints, module/function-body expression parsing, and lowering.
3. Inspect `ash-typeck` pattern, match, if-let, block-let, workflow, receive, yield-arm, and with_error code paths.
4. Inspect interpreter/runtime pattern failure paths and record exact error variants.
5. Write the audit artifact with callsite table, source/lowered/core classification, current-behavior notes, and RED test map.
6. Replace every downstream `false # TASK-1001` guard with exact focused commands.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 25
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - test -f docs/plan/audits/TASK-1001-matching-semantics-audit-gate.md
  - ! rg -n 'false # TASK-1001' docs/plan/tasks/TASK-100{2,3,4,5,6,7,8}-*.md
  - rg -n 'if-let parser entrypoints|silent pattern|yield arms|surface-level|core-only|LetPatternBindFailed|NonExhaustiveMatch|duplicate binders|branch type mismatch' docs/plan/audits/TASK-1001-matching-semantics-audit-gate.md
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Audit artifact exists and names exact live callsites
  - [x] Audit classifies source-level, lowered-only, and core-only binders
  - [x] Audit records exact runtime error variants and current if-let parser/typecheck behavior
  - [x] Downstream fail-closed guards are replaced with non-zero commands
  - [x] No downstream task remains implementation-ready without audit evidence
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

This is the hard gate for implementation. Do not code around unknown pattern-use surfaces.
