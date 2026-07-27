# Runnable Ash Semantic Realization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Align Ash documentation, tasks, and gates around one Surface → Core → CPS → Engine
execution path shared by CLI and daemon clients.

**Architecture:** PLAN-203 is the integration programme. Existing tasks retain layer/domain
ownership and publish handoffs; PLAN-203 owns route composition and CLI/daemon parity. The λAsh
calculi formally explain CPS transitions and guide conformance/Verus work without adding another
runtime path.

**Tech Stack:** Markdown specifications and plans, semantic traceability graph, documentation
validation scripts, Rust integration tests in later realization tasks.

---

### Task 1: Establish the programme and its task record

**Files:**
- Create: `docs/plan/PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md`
- Create: `docs/plan/tasks/TASK-2030-runnable-ash-semantic-realization-programme.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

1. Write the programme definition: one Engine executor, CLI/daemon client parity, integration
   gates, feature handoffs, a runnability matrix, and optional proof pilots.
2. Link TASK-2030 and PLAN-203 from PLAN-INDEX without changing historical task ownership.
3. Run `bash scripts/check-docs-gate.sh` and confirm new links resolve.

### Task 2: Align canonical and formal-semantics reading paths

**Files:**
- Modify: `docs/spec/ASH-CPS-CALCULUS.md`
- Modify: `docs/spec/CANONICAL-CORE.md`
- Modify: `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/plan/PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md`

1. Replace any Surface → λAsh execution diagram with Surface → Core → CPS and a CPS-to-calculus
   semantic relation.
2. State that `λAsh-Effect` is the next complete conservative CPS extension and must correspond to
   the target operational semantics and Engine state view.
3. Correct the selected production architecture to closed admission with no direct-evaluator
   fallback; retain current partial realization as evidence, not alternate authority.
4. State that Verus pilots are selected assurance evidence, while deferred proof obligations are
   visible but non-blocking.
5. Run `bash scripts/check-docs-gate.sh`.

### Task 3: Integrate the programme with agent workflow and traceability

**Files:**
- Modify: `AGENTS.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/spec/SPEC-INDEX.md`

1. Require PLAN-203 tasks to declare run-route impact, consumed/produced handoffs, downstream
   ownership, and integration responsibility.
2. Define proof-ledger use through existing semantic traceability dispositions; prohibit an
   unproved pilot from blocking execution delivery or being reported as verified.
3. Add the executable-realization reading path without changing any current rule-family facts.
4. Run `python3 tools/docs/validate_orientation_indexes.py --self-test`,
   `bash scripts/check-docs-gate.sh`, and `git diff --check`.

### Task 4: Prepare the first realization task

**Files:**
- Create: `docs/plan/tasks/TASK-2031-lambda-ash-effect-correspondence.md`
- Modify when activated: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify when activated: `docs/spec/SEMANTIC-TRACEABILITY.json`

1. TASK-2031 is now planned. Before activating implementation, add it to the semantic-task record
   scope with a full λAsh-Effect grammar, configuration, judgments, transitions, examples, and
   CPS/operational/Rust correspondence contract.
2. Add rule-indexed conformance tests before the executor implementation work starts.
3. Keep Verus obligations as `candidate`, `pilot`, or `deferred` evidence until a selected proof is
   actually verified.
4. Commit each completed task with its CHANGELOG and verification evidence.

### Task 5: Prepare the shared execution and parity integration task

**Files:**
- Create: `docs/plan/tasks/TASK-2032-shared-engine-execution-seam-and-client-parity.md`
- Create when activated: `docs/plan/RUNNABLE-ASH-MATRIX.md`
- Modify when activated: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify when activated: `docs/spec/SEMANTIC-TRACEABILITY.json`

1. TASK-2032 is now planned as the active-route integration owner. Before activating it, add the
   task to the semantic-task record scope and create the initial rule-indexed runnability matrix.
2. Write failing shared-Engine and CLI/daemon parity tests before replacing any route recognizer.
3. Make timeout and cancellation compare through the canonical versioned terminal envelope.
4. Commit each completed task with its CHANGELOG and verification evidence.
