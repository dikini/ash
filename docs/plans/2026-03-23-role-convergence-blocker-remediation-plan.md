# Role Convergence Blocker Remediation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the remaining blocker-class gaps in role convergence by replacing placeholder role-obligation lowering, wiring role-definition lowering into an honest parser/core path, and canonicalizing the touched role docs/examples.

**Architecture:** First align the canonical/core role-obligation carrier with the simplified source contract, then integrate inline-module role-definition lowering into an observable parser/core path, then fix the touched docs/examples so they either match the canonical surface or are explicitly historical, and finally close with an audit/bookkeeping pass.

**Tech Stack:** Markdown specs and plans, Rust crates (`ash-core`, `ash-parser`), parser/core tests, focused repository audits, and Ash examples/docs.

---

## Planning assumptions

- Phase 35 established the flat role contract and removed `supervises` from canonical role syntax.
- The remaining blocker is semantic honesty, not a request for richer governance features.
- All implementation tasks must avoid placeholder role-obligation behavior.
- TDD applies to parser/core changes, and documentation changes must stay synchronized with the
  implementation truth.

## Sequencing rules

1. Fix the core role-obligation carrier before expanding parser/lowering claims.
2. Do not ship role-definition lowering that fabricates unconditional obligation semantics.
3. Do not leave touched docs/examples in a half-canonical state.
4. Keep runtime/process supervision language separate from role-contract language.

## Verification defaults

- `cargo test -p ash-core`
- `cargo test -p ash-parser`
- `cargo fmt --check`
- `cargo clippy -p ash-core -p ash-parser --all-targets --all-features -- -D warnings`
- focused `rg` / `grep` audits for touched docs/examples

---

### Task 1 / TASK-221: Align Core Role Obligation Carrier

**Contract:** Replace the current placeholder role-obligation lowering target with a core role-level carrier that preserves named obligation references without fabricating workflow-obligation semantics.

**Files:**

- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-core/src/lib.rs` (if exports change)
- Test: `ash-core` role metadata tests
- Modify: `CHANGELOG.md`

**Steps:**

1. Write failing core/spec-alignment tests for role metadata that preserve named obligation references.
2. Run focused RED verification.
3. Introduce the dedicated core carrier and update the canonical IR/spec wording.
4. Run focused GREEN verification.
5. Commit.

---

### Task 2 / TASK-222: Integrate Role Definition Lowering Path

**Contract:** Make inline-module `role` definitions lower through a real parser/core path that uses the new role-obligation carrier, and remove the current dead/placeholder role-lowering story.

**Files:**

- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/module.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-parser/src/lib.rs` (if a new lowering API is exported)
- Test: parser/lowering unit tests for inline-module role definitions
- Test: any module-level integration tests needed for the new path
- Modify: `CHANGELOG.md`

**Steps:**

1. Write failing tests for lowering parsed inline-module role definitions through the observable API.
2. Run focused RED verification.
3. Implement the minimal parser/core integration and remove placeholder role-obligation lowering.
4. Run focused GREEN verification.
5. Commit.

---

### Task 3 / TASK-223: Canonicalize Touched Role Docs and Examples

**Contract:** Bring the touched role docs/examples into one of two honest states: canonical-surface-aligned or explicitly historical/reference-only.

**Files:**

- Modify: `docs/TUTORIAL.md`
- Modify: `docs/SHARO_CORE_LANGUAGE.md`
- Modify: `docs/book/SUMMARY.md`
- Modify: `docs/book/appendix-a.md`
- Modify: `examples/03-policies/01-role-based.ash`
- Modify: `examples/03-policies/README.md`
- Modify: `examples/04-real-world/code-review.ash`
- Modify: `examples/04-real-world/customer-support.ash`
- Modify: `examples/code_review.ash`
- Modify: `examples/multi_agent_research.ash`
- Modify: `examples/workflows/40_tdd_workflow.ash`
- Modify: `CHANGELOG.md`

**Steps:**

1. Write the failing checklist of touched files that still overclaim canonicality or contain local inconsistencies.
2. Run the focused RED audit.
3. Update the files to match canonical syntax or clearly label them as historical/reference material; fix the undefined-role example and local README/task wording drift.
4. Run the focused GREEN audit.
5. Commit.

---

### Task 4 / TASK-224: Role Convergence Closeout Audit

**Contract:** Re-run the role-convergence review after TASK-221 through TASK-223, reconcile task bookkeeping, and record any intentional residual references as non-blocking.

**Files:**

- Modify: `docs/audit/2026-03-23-role-convergence-audit.md` or create a new closeout audit
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/tasks/TASK-221-align-core-role-obligation-carrier.md`
- Modify: `docs/plan/tasks/TASK-222-integrate-role-definition-lowering-path.md`
- Modify: `docs/plan/tasks/TASK-223-canonicalize-touched-role-docs-and-examples.md`
- Modify: `docs/plan/tasks/TASK-224-role-convergence-closeout-audit.md`
- Modify: `CHANGELOG.md`

**Steps:**

1. Write the failing closeout checklist from the blocker review.
2. Run full verification plus focused audits.
3. Record the new audit result and reconcile task/phase bookkeeping.
4. Verify GREEN with fresh evidence.
5. Commit.

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-23-role-convergence-blocker-remediation-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** — dispatch a fresh subagent per task and review between tasks.
2. **Parallel Session (separate)** — open a new session and execute with `superpowers:executing-plans`.
