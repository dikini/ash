# Runtime Boundary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the runtime-first hardening work identified by the runtime-boundary steering brief without mixing in tooling or runtime-to-reasoner concerns.

**Architecture:** This phase executes only authoritative runtime work. It starts with runtime execution completeness, then makes runtime admission/rejection/commitment boundaries more explicit across engine and interpreter entry points, and finally hardens trace/provenance capture around accepted runtime progression. The plan deliberately keeps CLI, REPL, and presentation wording out of scope.

**Tech Stack:** Rust in `crates/ash-interp`, `crates/ash-engine`, `crates/ash-provenance`, and `crates/ash-macros`, plus focused Rust tests and `CHANGELOG.md`.

---

### Task 1: Runtime Action and Control-Link Execution Completeness

**Task File:**
- `docs/plan/tasks/TASK-205-implement-runtime-action-and-control-link-execution.md`

**Purpose:**
Remove the remaining stubbed or TODO-level runtime execution branches so the runtime execution path is complete on its own terms.

**Scope:**
- implement the canonical `Workflow::Act` execution path
- align control-link branches such as `Check`, `Kill`, `Pause`, `Resume`, and `CheckHealth`
- add focused interpreter tests for action execution and control-link outcomes
- keep the work runtime-only; no CLI/REPL wording or projection concerns

**Parallelization note:**
Must complete before Task 2.

### Task 2: Runtime Admission, Rejection, and Commitment Visibility

**Task File:**
- `docs/plan/tasks/TASK-206-align-runtime-admission-rejection-and-commitment-visibility.md`

**Purpose:**
Make runtime acceptance, rejection, admission, and commitment boundaries explicit and consistent across runtime entry points and helper boundaries.

**Scope:**
- align engine/interpreter entry points with explicit runtime-owned acceptance and rejection behavior
- tighten observe/set/send/receive admission and failure surfaces
- add focused tests for visible runtime boundary outcomes
- build on the canonical policy-outcome story instead of redefining it

**Parallelization note:**
Depends on Task 1 and should follow existing receive/policy convergence work where necessary.

### Task 3: Runtime Trace and Provenance Boundary Hardening

**Task File:**
- `docs/plan/tasks/TASK-207-harden-runtime-trace-and-provenance-boundaries.md`

**Purpose:**
Align runtime trace and provenance capture with accepted runtime progression and wrapper entry/exit framing.

**Scope:**
- tighten `TraceRecorder` / `TraceEvent` usage around accepted runtime effects
- align workflow wrapper entry/exit framing with runtime execution boundaries
- add focused provenance/trace tests
- keep trace/provenance runtime-owned and separate from presentation concerns

**Parallelization note:**
Depends on Tasks 205 and 206.

## Execution order

1. Complete TASK-205.
2. Complete TASK-206.
3. Complete TASK-207.

## Review rules to preserve

- Do not reopen runtime-versus-reasoner separation.
- Do not introduce CLI, REPL, or trace-presentation wording changes here.
- Keep monitorability, `exposes`, workflow observability, and provenance ownership runtime-only.
- Treat policy and capability verification as runtime-owned gates rather than new interaction-layer machinery.

## Deliverables

- complete runtime action/control-link execution
- explicit runtime admission/rejection/commitment boundaries
- hardened runtime trace/provenance capture aligned with accepted runtime progression

## Completion criteria

- runtime execution no longer relies on TODO/stub branches for the targeted surfaces
- runtime boundary behavior is explicitly test-covered
- trace/provenance capture is aligned with accepted runtime progression
- `CHANGELOG.md` and task/index state are updated
