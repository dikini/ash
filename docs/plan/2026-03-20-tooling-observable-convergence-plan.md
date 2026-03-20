# Tooling Observable Convergence Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the minimum-risk tooling and user-facing convergence work required by the tooling/surface steering brief.

**Architecture:** This phase builds on the already-planned REPL tasks and adds one CLI observable-output task. It first unifies REPL authority, then fixes canonical REPL `:type` reporting, and finally aligns `ash run` / `ash trace` output with the frozen observable contract. The optional stage-guidance overlay is deliberately deferred until the observable contract is implemented cleanly.

**Tech Stack:** Rust in `crates/ash-repl`, `crates/ash-cli`, and `crates/ash-engine`, focused CLI/REPL tests, and `CHANGELOG.md`.

---

### Task 1: Unify REPL Implementation

**Task File:**
- `docs/plan/tasks/TASK-172-unify-repl-implementation.md`

**Purpose:**
Make both REPL entrypoints delegate to one canonical implementation so the observable surface has one authority.

**Parallelization note:**
Must complete before Task 2.

### Task 2: Implement REPL Type Reporting

**Task File:**
- `docs/plan/tasks/TASK-173-implement-repl-type-reporting.md`

**Purpose:**
Replace placeholder `:type` output with canonical inferred-type reporting from the parse/type-check pipeline.

**Parallelization note:**
Depends on Task 1.

### Task 3: Align CLI Run and Trace Observable Output

**Task File:**
- `docs/plan/tasks/TASK-208-align-cli-run-and-trace-observable-output.md`

**Purpose:**
Align `ash run` and `ash trace` result, trace-summary, export, and error output with the frozen runtime-observable behavior contract.

**Scope:**
- tighten user-visible `run` output and trace-summary messaging
- tighten `trace` export confirmations, integrity acknowledgements, and observable error surfaces
- add focused CLI tests for observable output categories
- keep all work presentation-level and runtime-observable rather than semantic redesign

**Parallelization note:**
Depends on Task 2 so the tooling surface converges from REPL authority to REPL typing to CLI output.

## Deferred Follow-up

The presentation-only stage-guidance overlay is intentionally not opened in this batch.
It is optional explanatory work that should be reconsidered only after:

- REPL authority is unified,
- canonical `:type` output is implemented,
- and CLI observable output matches the frozen contract.

## Execution order

1. Complete TASK-172.
2. Complete TASK-173.
3. Complete TASK-208.

## Review rules to preserve

- Do not change runtime semantics through tooling wording.
- Do not add syntax or stage markers.
- Keep explanatory stage guidance separate from monitorability and runtime observability.
- Keep CLI/REPL surfaces from becoming implicit reasoner-context channels.

## Deliverables

- one authoritative REPL implementation
- canonical REPL `:type` output
- CLI `run` / `trace` output aligned with the observable contract

## Completion criteria

- REPL and CLI output match the frozen observable contract closely enough for focused tests
- user-facing wording stays presentation-level
- `CHANGELOG.md` and task/index state are updated
