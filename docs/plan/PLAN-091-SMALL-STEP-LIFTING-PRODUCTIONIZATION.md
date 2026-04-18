# PLAN-091: Small-Step and Statement-Lifting Productionization

> For Hermes: use subagent-driven-development for implementation tasks. Treat prototype behavior as insufficient unless it is explicitly documented, runtime-reachable, and independently verified.

Goal: convert the integrated TASK-604/TASK-605 prototype branch into a production-quality substrate by completing `Workflow::Call`, hardening the lifting contract, replacing heuristic effect classification, and adding rollout-grade verification evidence.

Architecture:
- Keep the current integrated branch as the implementation substrate.
- Productionize in sequenced tracks: runtime completion first, then lifting semantics hardening, then rollout/perf/evidence.
- Prefer conservative, explicit contracts over silent broadening of semantics.

Tech stack: Rust 2024, ash-core, ash-parser, ash-typeck, ash-interp, ash-engine, cargo test/clippy/fmt, design docs DESIGN-027 and DESIGN-028, SPEC-001, SPEC-002, SPEC-025.

---

## Scope

This plan assumes the current branch already contains:
- compressed small-step IR types and lowering
- small-step interpreter module and tests
- parser support for `|>`
- ANF lifting pass integrated into lowering
- partial application support in typechecker/runtime
- workspace-green prototype integration

This plan closes the gap between "integrated prototype" and "production-quality feature set".

## Decision Gates

### D1: Rollout mode
Recommended default: merge behind a feature flag or clearly documented runtime-selection boundary until parity evidence is complete.

### D2: Lifting contract shape
Recommended default: conservative contract. If a workflow position cannot honestly host synthetic bindings, preserve the original expression and let downstream diagnostics/typechecking report the user-facing error.

### D3: Effect classification source of truth
Recommended default: remove parser-local heuristic effect truth (`EFFECTFUL_NAMES`) and replace it with an explicit classification source shared with later pipeline stages.

---

## Track A: Runtime completion and parity

### TASK-606
Implement production-quality `Workflow::Call` execution across explicit runtime registration, typechecking visibility for registered callable workflows, and big-step/small-step execution.

### TASK-611
Extend parser/engine/typechecker surface representation so ordinary source files can declare local helper workflows and register them as real workflow-call targets.

### TASK-607
Close the remaining small-step runtime gaps and add a parity corpus proving observable agreement with the big-step interpreter on the supported workflow surface.

---

## Track B: Lifting contract hardening

### TASK-608
Formalize and implement the non-panicking lifting contract for unsupported workflow positions, add regression coverage, and ensure diagnostics stay user-facing and honest.

### TASK-609
Replace heuristic effect detection in the lifting pass with a production-quality effect classification path aligned with name resolution/typechecking/runtime truth.

---

## Track C: Rollout and evidence

### TASK-610
Add production rollout controls and evidence: feature-flag/runtime-selection policy, benchmark/perf harnesses, changelog/spec alignment, and production-readiness verification commands in CI-relevant form.

---

## Recommended execution order

1. TASK-606 — runtime execution of `Workflow::Call` for explicitly registered callable workflows
2. TASK-611 — local helper workflow surface and registration
3. TASK-608 — freeze the conservative lifting contract and remove remaining panic paths
4. TASK-607 — parity corpus and remaining small-step runtime completion
5. TASK-609 — de-heuristicize effect classification
6. TASK-610 — rollout, benchmarks, evidence, docs closeout

## Verification gates

After each task:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- targeted task-specific `cargo test` commands

Phase-close verification:
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- benchmark/report artifacts for production-readiness claims

## Expected deliverable

A branch that no longer advertises prototype semantics for these features:
- `Workflow::Call` is executable, not stubbed
- unsupported lifting positions preserve diagnostic honesty instead of panicking
- effect classification is not parser-local heuristic truth
- small-step/runtime behavior has explicit parity evidence and rollout policy
