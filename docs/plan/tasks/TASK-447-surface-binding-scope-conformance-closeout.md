# TASK-447: Surface Binding Scope Conformance Closeout

## Status: ⏳ Planned

## Description

Close the phase by adding end-to-end conformance coverage across `ash check`, `ash run`, and `ash trace`, refreshing the affected examples, and updating planning/reporting surfaces so the lexical-block scoping contract is frozen as implemented behavior rather than open design intent.

## Specification Reference

- [SPEC-002: Syntax](../../spec/SPEC-002-SYNTAX.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)

## Dependencies

- ⏳ [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)
- ⏳ [TASK-444: Parser And Lowering Lexical Block Normalization](TASK-444-parser-and-lowering-lexical-block-normalization.md)
- ⏳ [TASK-445: Type Checker Lexical Scope Conformance](TASK-445-type-checker-lexical-scope-conformance.md)
- ⏳ [TASK-446: Interpreter Lexical Scope And Seq Faithfulness](TASK-446-interpreter-lexical-scope-and-seq-faithfulness.md)

## Requirements

1. Add end-to-end tests proving `ash check`, `ash run`, and `ash trace` agree on lexical block scope.
2. Refresh affected examples so they demonstrate the canonical behavior.
3. Update `PLAN-INDEX.md` and `CHANGELOG.md` to mark the phase/task closeout once complete.
4. Run full verification gates required by project policy.

## TDD Steps

### Red

- There is currently no locked end-to-end conformance surface ensuring compile-time and runtime agreement for lexical block scope.

### Green

- CLI-facing commands agree on the accepted lexical-scope contract.
- Phase planning/reporting surfaces record the closeout cleanly.

## Completion Checklist

- [ ] End-to-end lexical-scope coverage exists for `ash check`, `ash run`, and `ash trace`
- [ ] Affected examples are updated or explicitly validated
- [ ] Full verification gates pass
- [ ] `PLAN-INDEX.md` and `CHANGELOG.md` record closeout
