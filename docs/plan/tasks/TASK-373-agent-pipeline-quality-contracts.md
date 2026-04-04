# TASK-373: Agent Pipeline Quality Contracts

## Status: ✅ Complete

## Description

Upgrade `tools/agent-pipeline` so its stage prompts, artifacts, and supervisor decisions enforce superpower-style quality contracts from the outside. Keep orchestration external, but strengthen the prompt and artifact contracts so downstream stages rely on explicit evidence, reusable templates/fragments, and fail-closed review verdicts rather than agent self-attestation.

Follow-up hardening completed after review:
- review artifacts now fail-close tasks even when the reviewer exits non-zero
- QA prompts now read the repository workspace plus implementation evidence artifacts instead of referencing a nonexistent `impl/` snapshot directory
- QA fail-close blocking now requires both `qa.review` and the required `qa.md` evidence report, so a bare verdict file remains retryable instead of permanently blocking the task

## Specification Reference

- AGENTS.md: external orchestration and task-file policy
- `tools/agent-pipeline/README.md`: current stage graph and artifact model
- Installed workflow skills used as contract references:
  - `verification-before-completion`
  - `subagent-driven-development`
  - `writing-plans`
  - `write-docs-workflow`

## Dependencies

- ✅ TASK-371: Agent Pipeline Supervision Fixes
- ✅ TASK-372: Agent Pipeline Packaging Review Fixes

## Requirements

### Functional Requirements

1. Stage prompt construction must reuse shared prompt-contract fragments/templates where feasible rather than duplicating quality language across stages.
2. Design/spec/plan prompts must require explicit traceability, measurable acceptance criteria, and structured review targets.
3. The implementation stage must produce verification evidence artifacts instead of only a completion marker.
4. QA and validate must support explicit pass/fail review artifacts (`qa.verified`/`qa.review`, `validated`/`validate.review`).
5. Supervisor logic must treat review findings as blocking contract failures, not retryable execution failures.
6. Documentation must explain how the external pipeline now mirrors superpower quality contracts without adopting superpower-native orchestration.

### Contract Requirements

1. Review prompts must use a strict verdict vocabulary (`VERIFIED`, `BLOCKED`, `NEEDS_REVISION`) or equivalent fail-closed outcome language.
2. Completion claims must require fresh verification evidence in stage artifacts.
3. Traceability must be representable from `task.md` through design/spec/plan/impl/qa outputs.
4. Existing reusable patterns (review artifact pass/fail files, docs-quality checklist structure) should be reused where feasible.

## TDD Steps

1. Add failing prompt-construction tests for shared contract fragments/templates and stricter verdict language.
2. Add failing prompt tests for design/spec/plan traceability and documentation-quality requirements.
3. Add failing artifact tests for impl evidence files, QA pass/fail artifacts, and validate pass/fail artifacts.
4. Add failing supervisor tests proving `qa.review` and `validate.review` block tasks instead of retrying.
5. Implement the minimum prompt-builder, artifact, and supervisor changes to satisfy the tests.
6. Run the full `tools/agent-pipeline` pytest suite and lint checks.

## Completion Checklist

- [x] Shared prompt-contract module or equivalent reusable template layer added
- [x] Design/spec/plan prompts require structured traceability artifacts
- [x] Impl stage requires summary + verification evidence artifacts
- [x] QA supports `qa.verified` / `qa.review`
- [x] Validate supports `validated` / `validate.review`
- [x] Supervisor blocks on QA/validate review findings
- [x] `tools/agent-pipeline/tests` passes
- [x] `ruff check tools/agent-pipeline/src tools/agent-pipeline/tests` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
