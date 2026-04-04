# Agent Pipeline Quality Contracts Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade `tools/agent-pipeline` so its externally orchestrated stage prompts and artifacts enforce the same quality discipline as the superpower workflow contracts, without adopting native superpowers orchestration inside the pipeline runtime.

**Architecture:** Keep the existing external stage graph (`design -> spec_write -> spec_verify -> plan_write -> plan_verify -> impl -> qa -> validate`), but strengthen it with reusable prompt-contract fragments, evidence-carrying artifacts, and fail-closed review verdicts. Reuse existing patterns where feasible: current pass/fail review artifacts (`spec.review`, `plan.review`), the write-docs 3-critic lens for documentation quality, and shared verification vocabulary adapted from the superpower skills. Prefer extracting reusable templates/fragments over duplicating stage-specific prompt text.

**Tech Stack:** Python 3.10+, `click`, `pytest`, filesystem state under `tools/agent-pipeline/.agents/`, prompt builders in `tools/agent-pipeline/src/agent_pipeline/`

---

### Task 1: Extract reusable prompt-contract templates and verdict vocabulary

**Objective:** Centralize the quality language shared across stage prompts so the pipeline reuses one contract vocabulary instead of duplicating bespoke strings.

**Files:**
- Create: `tools/agent-pipeline/src/agent_pipeline/prompt_contracts.py`
- Modify: `tools/agent-pipeline/src/agent_pipeline/agents.py`
- Test: `tools/agent-pipeline/tests/test_agents.py`

**Step 1: Write failing tests**

Add prompt-construction tests that assert:
- shared verdict vocabulary (`VERIFIED`, `BLOCKED`, `NEEDS_REVISION`) appears where required
- reusable quality fragments can be injected into multiple stage prompts
- stage prompts reuse common contract text rather than each embedding ad hoc wording

**Step 2: Run focused tests to verify failure**

Run: `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
Expected: FAIL because prompt construction is currently inline and does not expose reusable contract fragments.

**Step 3: Write minimal implementation**

Create a shared prompt-contract module that provides reusable fragments such as:
- review verdict instructions
- evidence-before-claim language
- traceability requirements
- verification evidence requirements
- structured report headings

Refactor `agents.py` to build stage prompts from these fragments instead of embedding all stage-specific quality language inline.

**Step 4: Re-run the focused tests**

Run: `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
Expected: PASS

### Task 2: Harden design/spec/plan prompts around traceability and reusable document-quality templates

**Objective:** Make upstream documentation stages produce structured artifacts that downstream reviews can verify mechanically instead of relying on free-form prose.

**Files:**
- Modify: `tools/agent-pipeline/src/agent_pipeline/agents.py`
- Modify: `tools/agent-pipeline/README.md`
- Test: `tools/agent-pipeline/tests/test_agents.py`

**Step 1: Write failing tests**

Add tests asserting that:
- the design prompt requires requirements, non-goals, assumptions, acceptance criteria, and traceability seeds
- the spec-write prompt reuses documentation-quality templates and explicitly requests technical/style/pedagogical checks where relevant
- the plan-write prompt requires exact file paths, per-task verification steps, and requirement-to-task traceability
- review prompts demand structured pass/fail artifacts rather than vague prose

**Step 2: Run focused tests to verify failure**

Run: `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
Expected: FAIL because current prompts only require thin document generation and loose verification text.

**Step 3: Write minimal implementation**

Update prompt builders so:
- `design.md` includes explicit requirements, non-goals, assumptions, measurable acceptance criteria, and a traceability section
- `spec.md` includes a requirement-to-spec traceability section and uses a reusable document-quality checklist derived from existing docs workflow patterns
- `plan.md` includes task/file mappings, verification commands, and requirement-to-task traceability
- `spec_verify` and `plan_verify` ask for severity-grouped findings and strict verdict headers

Document the new contract expectations in `README.md`.

**Step 4: Re-run the focused tests**

Run: `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
Expected: PASS

### Task 3: Harden implementation and downstream quality gates with evidence-carrying artifacts

**Objective:** Make implementation, QA, and final validation stages carry explicit verification evidence and fail-closed review outputs.

**Files:**
- Modify: `tools/agent-pipeline/src/agent_pipeline/agents.py`
- Test: `tools/agent-pipeline/tests/test_agents.py`
- Test: `tools/agent-pipeline/tests/test_supervisor.py`

**Step 1: Write failing tests**

Add tests asserting that:
- the impl prompt requires `impl.summary.md`, `impl.verification.md`, and `impl.complete`
- the QA prompt requires distinct sections for spec compliance, code quality, verification evidence, blocking findings, and final verdict
- the validate prompt supports both success and fail artifacts (`validated` or `validate.review`)
- QA produces pass/fail artifacts (`qa.verified` or `qa.review`) instead of only `qa.md`

**Step 2: Run focused tests to verify failure**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
- `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`

Expected: FAIL because current downstream prompts only require `impl.complete`, `qa.md`, and `validated`, and supervisor logic only blocks review artifacts for spec/plan verification.

**Step 3: Write minimal implementation**

Update prompt builders so:
- `impl` requires summary + evidence artifacts and forbids completion claims without fresh verification evidence
- `qa` emits `qa.verified` or `qa.review` plus a structured `qa.md`
- `validate` emits `validated` or `validate.review`

Extend artifact expectations accordingly.

**Step 4: Re-run the focused tests**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
- `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`

Expected: PASS

### Task 4: Distinguish retryable execution failures from blocking review findings

**Objective:** Preserve the current external orchestrator while making review failures block tasks instead of triggering meaningless retries.

**Files:**
- Modify: `tools/agent-pipeline/src/agent_pipeline/agents.py`
- Modify: `tools/agent-pipeline/src/agent_pipeline/supervisor.py`
- Test: `tools/agent-pipeline/tests/test_supervisor.py`

**Step 1: Write failing tests**

Add tests asserting that:
- `qa.review` blocks the task instead of scheduling another retry of the same QA stage
- `validate.review` blocks the task instead of retrying validate
- missing required artifacts or non-zero process exit still follow retry logic
- successful review artifacts (`qa.verified`, `validated`) advance the stage machine normally

**Step 2: Run focused tests to verify failure**

Run: `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`
Expected: FAIL because current block-on-review behavior only exists for `spec.review` and `plan.review`.

**Step 3: Write minimal implementation**

Refactor supervisor completion handling so review artifacts are classified as non-retryable contract failures while execution failures remain retryable. Reuse the existing spec/plan review blocking pattern for QA and validate instead of inventing a separate control path.

**Step 4: Re-run the focused tests**

Run: `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`
Expected: PASS

### Task 5: Final verification, documentation, and closeout

**Objective:** Verify the upgraded quality-contract pipeline end to end and document how the external orchestrator now mirrors superpower quality gates without adopting superpower-native runtime orchestration.

**Files:**
- Modify: `tools/agent-pipeline/README.md`
- Modify: `CHANGELOG.md`
- Review: `tools/agent-pipeline/src/agent_pipeline/*.py`
- Review: `tools/agent-pipeline/tests/*.py`

**Step 1: Update documentation**

Document:
- reusable prompt-contract fragments/templates
- required artifacts per stage
- fail-closed review behavior
- retryable vs blocking failures
- explicit statement that the orchestrator remains external while quality contracts align with the superpower workflow

**Step 2: Run focused verification**

Run:
- `python -m pytest tools/agent-pipeline/tests -q`
- `python -m ruff check tools/agent-pipeline/src tools/agent-pipeline/tests`

Expected: all tests pass, no lint errors.

**Step 3: Review against requirements**

Confirm:
- external orchestration remains unchanged
- shared templates/fragments are reused instead of duplicating contract language
- implementation claims require evidence artifacts
- QA and validate are fail-closed on review findings
- review blockers no longer consume retries

**Step 4: Update changelog**

Add an `Unreleased` entry summarizing the new prompt contracts, artifact gates, and review-blocking behavior.
