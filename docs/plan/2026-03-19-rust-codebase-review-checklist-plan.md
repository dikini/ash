# Rust Codebase Review Checklist Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a cluster-based Rust review checklist that turns the audit findings into a practical review sequence.

**Architecture:** Build one audit-facing checklist document organized by review risk rather than by phase. Use the prior audit reports as inputs, then map each risky cluster to concrete Rust files and review questions.

**Tech Stack:** Markdown documentation, existing audit reports, task docs, Rust crate layout

---

### Task 1: Draft checklist structure

**Files:**

- Create: `docs/audit/2026-03-19-rust-codebase-review-checklist.md`
- Reference: `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- Reference: `docs/audit/2026-03-19-task-consistency-review-non-lean.md`

**Step 1: Write the checklist headings**

- Add scope, baseline review, risky clusters, cross-cutting checks, and review order.

**Step 2: Verify structure against the audits**

- Confirm each risky cluster from the task audit is represented.

**Step 3: Commit**

- Commit with a docs-focused message after the checklist is complete.

### Task 2: Map clusters to review targets

**Files:**

- Modify: `docs/audit/2026-03-19-rust-codebase-review-checklist.md`
- Reference: `crates/ash-parser/src/surface.rs`
- Reference: `crates/ash-parser/src/parse_workflow.rs`
- Reference: `crates/ash-parser/src/parse_receive.rs`
- Reference: `crates/ash-parser/src/parse_type_def.rs`
- Reference: `crates/ash-typeck/src/policy_check.rs`
- Reference: `crates/ash-typeck/src/runtime_verification.rs`
- Reference: `crates/ash-typeck/src/check_pattern.rs`
- Reference: `crates/ash-typeck/src/capability_check.rs`
- Reference: `crates/ash-engine/src/providers.rs`
- Reference: `crates/ash-repl/src/lib.rs`
- Reference: `crates/ash-cli/src/commands/repl.rs`
- Reference: `crates/ash-interp/src/execute_stream.rs`
- Reference: `crates/ash-interp/src/execute_observe.rs`
- Reference: `crates/ash-interp/src/execute_set.rs`
- Reference: `crates/ash-interp/src/exec_send.rs`
- Reference: `crates/ash-interp/src/eval.rs`
- Reference: `crates/ash-interp/src/pattern.rs`
- Reference: `crates/ash-core/src/ast.rs`
- Reference: `std/src/option.ash`
- Reference: `std/src/result.ash`

**Step 1: Add concrete file targets per cluster**

- Policies
- REPL/CLI
- Streams/runtime verification
- ADTs

**Step 2: Add review questions per target**

- Spec alignment
- Task alignment
- API and runtime behavior alignment
- Test coverage gaps

### Task 3: Add review workflow notes

**Files:**

- Modify: `docs/audit/2026-03-19-rust-codebase-review-checklist.md`
- Modify: `CHANGELOG.md`

**Step 1: Add a recommended review order**

- Baseline stable clusters first, drift-heavy clusters after.

**Step 2: Add changelog entry**

- Record the new checklist document under `Unreleased`.

**Step 3: Verify output**

- Confirm the checklist file exists and the changelog entry is present.
