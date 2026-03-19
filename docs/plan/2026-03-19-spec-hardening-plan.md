# Spec Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Tighten the Ash language definition until Rust and Lean implementations can follow one unambiguous canonical contract rather than making local semantic choices.

**Architecture:** This plan inserts a documentation-only hardening gate between the current handoff-reference work and the planned Rust convergence phases. It first hardens core and phase semantics, then tightens the highest-risk semantic clusters, then adds explicit observable-behavior and formalization-boundary documents, and finally re-audits readiness before Rust alignment resumes.

**Tech Stack:** Markdown specs, reference docs, design/plan/task documents, existing convergence task files, and the current Ash spec set spanning `SPEC-001` through `SPEC-020`.

---

## Planning assumptions

- TASK-156 through TASK-163 remain valid prerequisites.
- TASK-164 through TASK-176 are still the Rust-convergence track, but they are blocked behind this hardening program.
- This hardening pass is docs-only.
- The point is to clarify canonical semantics, not to argue from current implementation behavior.

## Sequencing rules

1. No Rust-alignment task starts before the hardening audit passes.
2. No normative spec should embed implementation drift as if it were part of the language contract.
3. Every hardening task must identify which parts are canonical truth and which parts belong only in migration notes or follow-up tasks.
4. The lowered IR contract must remain compatible with both interpreter-first execution and future JIT compilation.

## Verification defaults

Use the smallest focused verification first, then widen:

- targeted contradiction/readability checks across the touched specs
- task-file red/green checklist verification
- `git diff --check`

---

### Task 1 / TASK-177: Freeze Canonical Core Language and Execution-Neutral IR

**Contract:** Define the canonical core language, identify which forms are surface sugar only, and make the lowered IR explicitly neutral between interpreter execution and later JIT compilation.

**Files:**
- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist identifying where the current core/IR contract still permits local interpretation.

**Step 2:** Verify RED by confirming at least one core-vs-sugar or interpreter-vs-JIT boundary is still implicit.

**Step 3:** Write the minimal spec changes freezing the canonical core language and execution-neutral IR contract.

**Step 4:** Verify GREEN by checking that the core/sugar split and IR neutrality are explicit.

**Step 5:** Commit.

**Non-goals:** No Rust code changes. No parser/lowering fixes yet.

---

### Task 2 / TASK-178: Normalize Phase Judgments and Rejection Boundaries

**Contract:** Define explicit parse, lowering, typing, runtime, and observable-behavior judgment boundaries and make each phase’s rejection/error ownership explicit.

**Files:**
- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/reference/surface-to-parser-contract.md`
- Modify: `docs/reference/parser-to-core-lowering-contract.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist listing still-implicit phase boundaries.

**Step 2:** Verify RED by confirming at least one rejection class still depends on prose interpretation.

**Step 3:** Write the minimal spec/reference changes separating phase-owned judgments and rejection boundaries.

**Step 4:** Verify GREEN by checking that phase ownership is explicit and non-overlapping.

**Step 5:** Commit.

**Non-goals:** No implementation changes. No new runtime features.

---

### Task 3 / TASK-179: Formalize Receive Mailbox and Scheduling Semantics

**Contract:** Make `receive` precise enough for both implementation and proof work: source scheduling modifier, mailbox scan order, guard point, consumption point, timeout behavior, and control-stream behavior.

**Files:**
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-013-STREAMS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/reference/parser-to-core-lowering-contract.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist identifying remaining ambiguity in `receive`.

**Step 2:** Verify RED by confirming at least one of source selection, arm order, guard timing, or timeout fallthrough is still under-specified.

**Step 3:** Write the minimal spec changes that fully define `receive`.

**Step 4:** Verify GREEN by checking that `receive` semantics no longer depend on local implementation choices.

**Step 5:** Commit.

**Non-goals:** No scheduler implementation work. No new scheduling modifiers beyond the documented contract.

---

### Task 4 / TASK-180: Formalize Policy Evaluation and Verification Semantics

**Contract:** Tighten the semantics of named policy bindings, normalized core policy lowering, workflow `decide`, capability verification outcomes, and policy rejection/error ownership.

**Files:**
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`
- Modify: `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- Modify: `docs/spec/SPEC-008-DYNAMIC-POLICIES.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist identifying the remaining ambiguous policy semantics.

**Step 2:** Verify RED by confirming at least one workflow-vs-capability policy boundary still relies on interpretation.

**Step 3:** Write the minimal spec changes that define one proof-friendly and implementation-friendly policy story.

**Step 4:** Verify GREEN by checking that policy meaning is explicit from named binding through runtime decision domain.

**Step 5:** Commit.

**Non-goals:** No SMT or runtime implementation changes.

---

### Task 5 / TASK-181: Formalize ADT Dynamic Semantics

**Contract:** Tighten constructor evaluation, runtime value shape, pattern matching, `if let`, and exhaustiveness-facing dynamic behavior so ADT semantics are proof-shaped and implementation-shaped at once.

**Files:**
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`
- Modify: `docs/reference/parser-to-core-lowering-contract.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `docs/reference/runtime-observable-behavior-contract.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist identifying the remaining dynamic ADT ambiguity.

**Step 2:** Verify RED by confirming at least one constructor/pattern/runtime-value boundary still depends on prose interpretation.

**Step 3:** Write the minimal spec changes that define the canonical ADT dynamic semantics.

**Step 4:** Verify GREEN by checking that ADT runtime behavior is explicit enough for both Rust and Lean.

**Step 5:** Commit.

**Non-goals:** No stdlib code changes. No Rust type-checker updates yet.

---

### Task 6 / TASK-182: Add Normative Observable-Behavior Specification

**Contract:** Create one single normative specification for runtime-observable behavior covering verification outcomes, REPL/CLI-visible behavior, error visibility, and value-display semantics that matter contractually.

**Files:**
- Create: `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
- Modify: `docs/spec/SPEC-005-CLI.md`
- Modify: `docs/spec/SPEC-011-REPL.md`
- Modify: `docs/spec/SPEC-016-OUTPUT.md`
- Modify: `docs/reference/runtime-observable-behavior-contract.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist identifying where observable behavior is still split across multiple docs.

**Step 2:** Verify RED by confirming there is no single normative observable-behavior spec.

**Step 3:** Write the minimal new spec and cross-references.

**Step 4:** Verify GREEN by checking that one spec now owns the observable contract.

**Step 5:** Commit.

**Non-goals:** No CLI or REPL implementation changes.

---

### Task 7 / TASK-183: Define Formalization Boundary and Proof Targets

**Contract:** State exactly which specifications Lean should treat as normative, which artifacts are migration-only, and which target properties later proof and bisimulation work should establish.

**Files:**
- Create: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/2026-03-19-spec-hardening-design.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist identifying what a Lean implementation would still have to guess.

**Step 2:** Verify RED by confirming there is no single formalization-boundary note.

**Step 3:** Write the boundary note and target proof list.

**Step 4:** Verify GREEN by checking that the normative document set and target properties are explicit.

**Step 5:** Commit.

**Non-goals:** No Lean code yet. No theorem proofs yet.

---

### Task 8 / TASK-184: Audit Spec Hardening Readiness for Rust and Lean

**Contract:** Re-audit the tightened spec set and confirm whether Rust convergence can proceed mechanically and whether Lean formalization has a stable starting point.

**Files:**
- Create: `docs/audit/2026-03-19-spec-hardening-readiness-review.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1:** Write the failing checklist for hardening readiness.

**Step 2:** Verify RED by confirming at least one ambiguity class still exists before the hardening work is done.

**Step 3:** Write the readiness audit after TASK-177 through TASK-183 complete.

**Step 4:** Verify GREEN by checking that the audit explicitly gates Rust convergence.

**Step 5:** Commit.

**Non-goals:** No Rust implementation changes. No Lean implementation changes.
