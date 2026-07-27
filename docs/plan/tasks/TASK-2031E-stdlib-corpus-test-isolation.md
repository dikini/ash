# TASK-2031E: Stdlib Corpus Test Isolation

**Status:** In progress
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Type:** Bounded test-isolation remediation
**Depends on:** TASK-543 LLM stdlib E2E validation and TASK-760 stdlib corpus baseline
**Blocks:** TASK-2031 workspace-gate closeout

## Description

Remove a workspace-test race without changing the standard library. TASK-543's LLM import tests
temporarily create four `.ash` consumers under the tracked `std/src` tree. TASK-760 correctly
enumerates that tree, so concurrent workspace test processes can observe 62 files instead of the
canonical 59. The consumer fixtures must instead live in an isolated temporary copy of the LLM
stdlib layout.

## Requirements

1. Preserve the existing import and rejection assertions.
2. Do not write temporary fixtures inside the repository's `std/src` tree.
3. Supply only the copied LLM stdlib layout required for the existing resolver behavior.
4. Keep TASK-760's canonical 59-file expectation strict; do not relax its baseline.

## TDD steps

1. **RED:** Run the workspace suite and reproduce TASK-760's `62 != 59` corpus count while the
   TASK-543 consumers are present.
2. Move only the mutable consumers to an isolated temporary fixture layout.
3. **GREEN:** Run TASK-543 and TASK-760 targets together and through the workspace suite.
4. Run formatter, Clippy, docs, and independent review gates.

## Completion checklist

- [ ] Concurrent test processes cannot add `.ash` files to `std/src`.
- [ ] Existing LLM import/rejection assertions are retained.
- [ ] TASK-760 remains strict at 59 tracked stdlib files.
- [ ] Workspace Rust tests, formatter, Clippy, and docs gate pass; QA/review evidence is recorded.
