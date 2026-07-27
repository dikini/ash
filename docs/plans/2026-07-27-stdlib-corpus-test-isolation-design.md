# Stdlib Corpus Test Isolation Design

**Goal:** Eliminate the test-process race between LLM import fixtures and the strict standard
library corpus baseline.

**Architecture:** Keep the canonical `std/src` tree read-only during tests. For the four TASK-543
import cases, make a temporary directory containing a copied `llm/` module layout and write the
consumer there. This preserves the resolver's `use llm::…` path while preventing a concurrent
TASK-760 enumeration from seeing transient repository files.

## Alternatives considered

1. Loosen TASK-760's expected file count. Rejected: it would hide real corpus drift.
2. Serialize the whole workspace. Rejected: it hides mutable-test pollution rather than removing
   it and imposes an unrelated global cost.
3. Continue writing then deleting `std/src` fixture files. Rejected: cross-process enumeration is
   inherently racy.
4. Temporary copied LLM module layout. Selected: maintains the same resolver contract without
   mutating repository state.

## TDD plan

1. Preserve the reproduced `62 != 59` workspace failure and passing isolated TASK-760 baseline.
2. Extract test-only fixture-copy/write helpers and retain all four original assertions.
3. Verify both targets and the workspace gate.
