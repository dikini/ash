# TASK-550: End-to-end validation and CHANGELOG update

**Plan Reference:** PLAN-027 (LLM Stdlb Usability Remediation)
**Spec Reference:** SPEC-029
**Status:** Done
**Depends on:** TASK-545, TASK-546, TASK-547, TASK-548, TASK-549

## Description

Final validation that the LLM stdlib is usable end-to-end. Verify all PLAN-027
success criteria are met, add an integration test that exercises the full path
from .ash source through engine parsing with llm type resolution, and update
CHANGELOG/PLAN-INDEX.

## Success Criteria (from PLAN-027)

1. `count_pub_fn_snippets(prompt.ash)` >= 23 (up from 7) -- **27/27**
2. `ash check std/src/llm/types.ash` reports 0 type errors -- **tested**
3. `use llm::Role; workflow main { done }` resolves -- **tested**
4. `render_template`, `is_final`, `append_response`, `append_tool_result` present -- **tested**
5. No fn in llm/ calls a workflow (three-vertex) -- **tested**
6. End-to-end test: build+run an LLM workflow from pure .ash code -- **this task**

## TDD Steps

1. Red: Write integration test parsing an .ash file that uses multiple llm types
   and functions through the engine.
2. Red: Write test asserting SPEC-029 sections are substantively covered.
3. Green: Fix any remaining issues.
4. Green: Update CHANGELOG.md, PLAN-INDEX.md.
5. Verify: Full `cargo test` passes.

## Files

- Add: `crates/ash-engine/tests/llm_e2e_usability_tests.rs`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`
