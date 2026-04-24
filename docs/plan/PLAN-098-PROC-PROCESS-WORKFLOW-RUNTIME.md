# PLAN-098: Proc, Process Runtime, Failure, and Workflow Boundary Implementation

> For Hermes: this is an implementation-planning packet only. Do not start runtime implementation from this plan until the referenced task file for the selected slice exists and the branch has passed the task preflight.

Goal: implement the semantic tower runtime slice introduced by SPEC-048 through SPEC-051 in dependency order, without collapsing `Act<A>`, `Proc<A>`, and `Workflow` into one runtime concept.

Architecture:
- Preserve the tower `Pure < Effectful/Act < Proc < Workflow`.
- Treat Phase 97 / SPEC-047 as the prerequisite Act substrate for expression-level effectful computation.
- Build a process runtime substrate before public `par`/`await`/`join`/`gather` behavior.
- Keep workflow governance/reporting above process execution; do not make workflow admission the first capability semantics layer.
- Keep existing `Workflow::Spawn` / `ControlLink` behavior distinct from affine `P<A>` process handles until an explicit bridge task chooses otherwise.

Tech stack: Rust 2024, ash-core, ash-parser, ash-typeck, ash-interp, ash-engine, cargo test/clippy/fmt, SPEC-047, SPEC-048, SPEC-049, SPEC-050, SPEC-051.

---

## Discovery summary

Current substrate discovered before writing this plan:
- `WorkflowId` exists in `crates/ash-core/src/provenance.rs`, but `RunId`, `ProcessId`, and `BranchId` do not yet exist.
- `RuntimeState` already owns useful async infrastructure in `crates/ash-interp/src/runtime_state.rs`: control-link registry, child workflow registry, yield/proxy registries, providers, and retained completion records.
- `ControlLinkRegistry` in `crates/ash-interp/src/control_link.rs` is reusable infrastructure, but it is workflow-instance control authority, not an affine `P<A>` result-observation handle.
- `Context` in `crates/ash-interp/src/context.rs` is a cloneable lexical/obligation/role context; it is not an identity-indexed process environment projection model.
- `ExecError` in `crates/ash-interp/src/error.rs` has no structured process failure, aggregate process failure, handle-consumed, or workflow-boundary failure variants.
- Surface/core AST and typechecker do not currently have `Proc`, `P`, `fail`, `with_error`, process `yield`, `await`, `join`, or `gather` support.
- Existing workflow/proxy `Yield` exists partially, but it is not SPEC-049 `yield : Proc<Unit>`.
- Workflow declaration syntax already carries roles/capabilities/requires/ensures, but there is no `WorkflowFailure`, `WorkflowReport`, `RunId`, admission context, requires/ensures evidence, or report commit boundary.

## Scope

This phase creates the first implementation path for:
- runtime identity vocabulary and structured lower failures,
- operational `fail` and scoped `with_error`,
- `Proc<A>` / `P<A>` type and runtime substrate,
- `Proc` core construction/sequencing combinators (`unit`, `bind`, `then`),
- process scheduling and affine handle observation,
- `yield`, `par`, `scatter`, `await`, `join`, and `gather`,
- workflow boundary outcomes and reporting over governed process execution.

This phase does not include:
- shared/replayable process handles,
- monitors/supervisors,
- multi-observer process observation,
- process mailbox/channel semantics beyond what tasks explicitly require,
- replacing existing `Workflow::Spawn` / `ControlLink` supervision semantics,
- broad workflow orchestration rewrites.

## Decision gates

### D1: Act prerequisite readiness
Default: do not start Proc runtime tasks that depend on `from_act`/effectful execution until the Phase 97 Act substrate required by that task is implemented and verified. Identity/failure-carrier tasks may proceed independently.

### D2: `Proc` and `P` type representation
Recommended default: register `Proc` and `P` as ordinary type constructors first. Introduce dedicated Rust enum variants only if typechecker implementation pressure makes generic constructors unsafe or too ambiguous.

### D3: Operational bottom representation
Recommended default: add explicit surface/core carriers for `fail` and `with_error` rather than relying on ordinary stdlib function calls, because SPEC-050 requires bottom typing and scoped dynamic handling.

### D4: Existing workflow `Yield` vs process `yield`
Recommended default: keep names semantically distinct in implementation. Existing workflow/proxy yield remains workflow-level suspension; SPEC-049 `yield : Proc<Unit>` is a process scheduler point.

### D5: Existing `ControlLink` vs `P<A>`
Recommended default: do not reuse `ControlLink` as `P<A>`. A `ControlLink` is reusable workflow supervision/control authority; `P<A>` is affine result-observation authority.

### D6: Workflow report API rollout
Recommended default: add a new reported/admitted engine API alongside existing `ExecResult<Value>` APIs. Keep existing APIs as compatibility wrappers until report semantics are stable.

---

## Track A: Substrate and lower failure model

### TASK-705
Semantic tower runtime preflight and Phase 97 dependency check.

### TASK-706
Add runtime identity and failure carrier types: `RunId`, `ProcessId`, internal `BranchId`, process lifecycle state, operational failure, process failure, aggregate failure, and workflow boundary carrier skeletons.

### TASK-707
Register `Proc` and `P` type constructors and add typechecker/lowering guardrails without enabling process operations yet.

### TASK-718
Implement the basic SPEC-048 `Proc` library combinators `unit`, `bind`, and `then` without enabling child process creation or handle observation.

### TASK-708
Implement operational `fail` and scoped `with_error` across parser, AST, typechecker, lowering, and interpreter failure propagation.

---

## Track B: Process runtime substrate and primitive observation

### TASK-709
Introduce the process registry and component-wise child environment projection API in `RuntimeState` / interpreter runtime.

### TASK-710
Implement affine `P<A>` runtime handles and single-handle `await` observation.

### TASK-711
Implement process scheduler `yield : Proc<Unit>` as a cooperative scheduling point distinct from workflow/proxy yield.

---

## Track C: Process concurrency library

### TASK-712
Implement `par` and `scatter` all-or-none child admission returning process handles before child failures affect the parent.

### TASK-713
Implement wait-for-all `join` and `gather` with ordered successes and aggregate child failure preservation.

---

## Track D: Workflow boundary semantics

### TASK-714
Define workflow boundary outcome, failure, report, and admission-context types.

### TASK-715
Implement workflow admission checks, requires evidence, and ensures evidence plumbing without performing completion-time ensures evaluation.

### TASK-716
Implement workflow boundary completion: ensures evaluation, obligation checks, lower failure reinterpretation, and in-memory report construction.

### TASK-717
Run cross-layer conformance validation for SPEC-048 through SPEC-051 and close documentation/changelog drift.

---

## Recommended execution order

1. TASK-705 — preflight current branch, Phase 97 status, and stale syntax risks.
2. TASK-706 — identity and structured failure carriers.
3. TASK-714 — workflow boundary outcome/report carriers (may run after TASK-706 in parallel with process-runtime substrate work).
4. TASK-707 — `Proc`/`P` type constructor registration.
5. TASK-718 — `Proc` core `unit`/`bind`/`then` library combinators.
6. TASK-708 — `fail` and `with_error` operational-bottom substrate.
7. TASK-709 — process registry and environment projection.
8. TASK-710 — affine handles and `await`.
9. TASK-711 — process `yield`.
10. TASK-712 — `par`/`scatter` admission and child process creation.
11. TASK-713 — `join`/`gather` wait-for-all aggregation.
12. TASK-715 — workflow admission and requires evidence, with ensures evidence plumbing only.
13. TASK-716 — workflow completion, ensures evaluation, reporting, and failure reinterpretation.
14. TASK-717 — cross-layer conformance closeout.

## Verification gates

After each implementation task:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- targeted task-specific `cargo test` commands listed in the task file
- update `CHANGELOG.md` for implementation/tooling/docs-policy changes

Phase-close verification:
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- cross-layer parser/typechecker/interpreter/engine tests covering SPEC-048 through SPEC-051
- doc drift audit against `docs/spec/SPEC-048-PROC-LIBRARY.md`, `SPEC-049`, `SPEC-050`, and `SPEC-051`

## Expected deliverable

A runtime substrate where:
- operational bottom is structured and scoped,
- `Proc` has its basic `unit`/`bind`/`then` construction and sequencing surface,
- processes have stable identities and affine observation handles,
- `par`/`scatter` start child processes all-or-none,
- `await`/`join`/`gather` preserve child identities on failure,
- workflow admission/completion produces a reportable boundary outcome,
- existing workflow/control-link semantics remain compatible until explicitly bridged.
