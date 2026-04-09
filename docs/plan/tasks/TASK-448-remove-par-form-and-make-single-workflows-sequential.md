# TASK-448: Remove `par` Form And Make Single Workflows Sequential

## Status: Complete

## Description

Remove the `par` workflow form from the active Ash language so a single workflow denotes sequential process execution only. Concurrency remains expressible at the system level through multiple communicating workflows and runtime process mechanisms, but it is no longer modeled as workflow-internal `Par` composition.

## Specification Reference

- [SPEC-001: IR](../../spec/SPEC-001-IR.md)
- [SPEC-002: Surface Syntax](../../spec/SPEC-002-SURFACE.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-022: Workflow Typing](../../spec/SPEC-022-WORKFLOW-TYPING.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [SPEC-026: Implementation Conformance](../../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md)

## Dependencies

- ✅ [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)
- ✅ [TASK-444: Parser And Lowering Lexical Block Normalization](TASK-444-parser-and-lowering-lexical-block-normalization.md)
- ✅ [TASK-445: Type Checker Lexical Scope Conformance](TASK-445-type-checker-lexical-scope-conformance.md)
- ✅ [TASK-446: Interpreter Lexical Scope And Seq Faithfulness](TASK-446-interpreter-lexical-scope-and-seq-faithfulness.md)
- ✅ [TASK-447: Surface Binding Scope Conformance Closeout](TASK-447-surface-binding-scope-conformance-closeout.md)

## Requirements

1. Remove `par` from the active surface syntax, canonical workflow IR, typing rules, and operational semantics.
2. Preserve historical records of prior `Par` work in changelog/task history instead of rewriting completed historical documents.
3. Ensure legacy `par` source is rejected at the normal parser boundary; no compatibility lowering to `seq` is allowed.
4. Keep runtime/process-level concurrency features that are independent of the `par` language form.
5. Replace active examples, fixtures, tutorial text, and conformance/reference material that still present `par` as a current language feature.
6. Update `PLAN-INDEX.md` and `CHANGELOG.md` as part of the task.

## TDD Steps

### Red

- Current specs, parser, AST, type checker, interpreter, examples, and conformance/reference docs still encode `par` / `Par` as a live language feature.

### Green

- No active language surface accepts or specifies `par`.
- A single workflow is sequential in the current normative spec corpus.
- Active user-facing examples and conformance materials no longer rely on `Par`.

## Completion Checklist

- [x] Normative specs no longer include `Par` as part of the active language
- [x] Parser and lowering reject/remove `par`
- [x] Core AST, type checking, and interpreter remove `Par`
- [x] Active examples/tutorials/fixtures no longer teach `par`
- [x] Active conformance/reference docs no longer depend on `Par`
- [x] Full verification gates pass
- [x] `PLAN-INDEX.md` and `CHANGELOG.md` updated

## Implementation Notes

- The target semantics are strict: a single workflow is sequential.
- Concurrency should be explained through communicating workflows/processes rather than within one workflow term.
- Historical records mentioning `Par` remain valid as history and should not be rewritten merely to erase prior design stages.

## Completion Notes

Completed 2026-04-09 via subagent-driven development following the sequential workflow language plan.

### Commits
1. c072393 "docs(spec): fix historical marker consistency in SPEC-025"
2. 232672d "feat(parser): remove par workflow form"
3. (ash-interp removal) "refactor(core): remove Par from workflow execution model"
4. 5cac23c "docs(examples): replace par-based workflow examples"
5. b041716 "test(conformance): remove active Par corpus dependence"
6. cfb28d2 "Remove engine unit tests that assert par is valid"
7. 341877a "Fix docs: Mark par examples in SPEC-013-STREAMS.md as historical"
8. 13ce33f "Fix: Remove par from parser benchmark"
9. ee6122a "Mark normative Par text in SPEC-025 as historical"
10. 3f399e7 "Fix low issue: Remove stale par comments in source code"

### Verification
- All 5 tasks from the plan completed successfully
- All review findings addressed (2 rounds, 5 total findings)
- Workspace verification: cargo fmt, cargo check, cargo clippy, cargo test all pass
- No active references to `par` as valid language syntax remain
