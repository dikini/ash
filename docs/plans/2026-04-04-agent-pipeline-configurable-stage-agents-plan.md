# Agent Pipeline Configurable Stage Agents Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the agents used for each `tools/agent-pipeline` stage configurable without changing the external stage graph or weakening existing prompt/artifact contracts.

**Architecture:** Keep the existing pipeline stages and prompt builders, but move stage-to-agent selection out of the hard-coded `STAGE_AGENTS` constant and into a validated runtime configuration layer. Reuse the current `AgentType` enum and default mapping as the baseline template, then allow CLI/environment-driven overrides to replace specific stage assignments. The supervisor, CLI, and spawner should resolve the same mapping so status, execution, and packaging remain aligned.

**Tech Stack:** Python 3.10+, `click`, `pytest`, file-based state in `tools/agent-pipeline/.agents/`

---

### Task 1: Add validated stage-agent configuration support to the spawner

**Objective:** Replace the fixed class-level stage mapping with a configurable, validated mapping while preserving the current defaults.

**Files:**
- Modify: `tools/agent-pipeline/src/agent_pipeline/agents.py`
- Test: `tools/agent-pipeline/tests/test_agents.py`

**Step 1: Write failing tests**

Add tests that assert:
- `AgentSpawner` still uses the current default stage-agent mapping when no override is provided
- a partial override can replace selected stages without redefining all stages
- invalid stage names are rejected
- invalid agent names are rejected
- the resolved mapping is used by `get_agent_type()` and command preparation

**Step 2: Run focused tests to verify they fail**

Run: `PYTHONPATH=src python -m pytest tests/test_agents.py -q`
Expected: FAIL because stage-agent selection is currently hard-coded in `STAGE_AGENTS`.

**Step 3: Write minimal implementation**

Refactor `AgentSpawner` to:
- keep a reusable default mapping template
- accept a runtime override mapping
- validate and normalize overrides against `Stage` and `AgentType`
- expose the resolved mapping to downstream callers

**Step 4: Re-run focused tests**

Run: `PYTHONPATH=src python -m pytest tests/test_agents.py -q`
Expected: PASS

### Task 2: Plumb configuration through CLI and supervisor entry points

**Objective:** Make the configurable mapping usable from the orchestrator surface, not just the spawner constructor.

**Files:**
- Modify: `tools/agent-pipeline/src/agent_pipeline/cli.py`
- Modify: `tools/agent-pipeline/src/agent_pipeline/supervisor.py`
- Test: `tools/agent-pipeline/tests/test_cli.py`
- Test: `tools/agent-pipeline/tests/test_supervisor.py`

**Step 1: Write failing tests**

Add tests that assert:
- CLI can accept a stage-agent mapping override input
- supervisor receives the same resolved mapping used by CLI helpers
- status/control flows still work with a non-default stage-agent mapping
- configured mappings can be supplied through an environment/config-friendly surface

**Step 2: Run focused tests to verify they fail**

Run: `PYTHONPATH=src python -m pytest tests/test_cli.py tests/test_supervisor.py -q`
Expected: FAIL because CLI and supervisor currently instantiate `AgentSpawner` with only base-dir/workspace settings.

**Step 3: Write minimal implementation**

Add a small configuration surface, preferably reusing existing CLI/env resolution patterns, for example:
- CLI option and/or env var for a JSON stage-agent mapping
- shared parsing helper used by CLI and supervisor construction

Keep the default behavior unchanged when no mapping is provided.

**Step 4: Re-run focused tests**

Run: `PYTHONPATH=src python -m pytest tests/test_cli.py tests/test_supervisor.py -q`
Expected: PASS

### Task 3: Update docs and packaged usage guidance

**Objective:** Document the configurable stage-agent mapping as a supported contract.

**Files:**
- Modify: `tools/agent-pipeline/README.md`
- Modify: `CHANGELOG.md`
- Review: packaged deployment docs/scripts if the chosen configuration surface affects them

**Step 1: Update README**

Document:
- the default stage-agent mapping
- how to override selected stages
- accepted agent names
- how invalid mappings fail
- that prompt/artifact contracts stay unchanged even when stage agents are reassigned

**Step 2: Update changelog**

Add an `Unreleased` entry describing configurable stage-agent selection.

### Task 4: Final verification and closeout

**Objective:** Verify the configurable stage-agent feature end to end.

**Files:**
- Modify: `docs/plan/tasks/TASK-374-agent-pipeline-configurable-stage-agents.md`
- Review: `tools/agent-pipeline/src/agent_pipeline/*.py`
- Review: `tools/agent-pipeline/tests/*.py`

**Step 1: Run full verification**

Run:
- `cd tools/agent-pipeline && PYTHONPATH=src python -m pytest -q`
- `cd tools/agent-pipeline && python -m ruff check src tests`

Expected: all tests pass, no lint errors.

**Step 2: Review against requirements**

Confirm:
- default behavior unchanged
- per-stage overrides work
- invalid mappings fail clearly
- supervisor and CLI use the same resolved mapping
- prompt/artifact quality contracts remain intact

**Step 3: Mark task complete**

Update TASK-374 after verification passes.
