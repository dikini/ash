# TASK-2014 Terminal Taxonomy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Route the approved missing-admission and invalid-checked-Core/CPS outcomes through the existing V1 CLI terminal envelope without reopening direct evaluation. The follow-on exact admitted abortive `trap_sleep` fixture also projects its post-admission language division as V1 `trap` (exit 5); it does not generalize handler semantics.

**Architecture:** Preserve the Engine's sealed admission boundary, but carry a typed bounded production-terminal classification from each relevant Engine boundary to the CLI rather than recovering it from error text. The CLI maps that classification to the approved existing V1 observable and its existing `CliError` exit class; all other error behavior stays unchanged.

**Tech Stack:** Rust 2024, Tokio, `thiserror`, `assert_cmd`, `serde_json`, existing Ash Engine/CLI integration tests.

---

### Task 1: Establish typed boundary classifications with RED tests

**Files:**
- Modify: `crates/ash-engine/tests/task_2014_checked_cps_admission.rs`
- Modify: `crates/ash-engine/src/lib.rs`

**Step 1: Write failing tests**

Add one test for each implemented sealed pre-execution boundary:

```rust
#[test]
fn unsupported_source_lowering_is_a_missing_production_admission() {
    // A checked but unsupported source must expose MissingAdmission, never a
    // direct-evaluator result or a malformed-Core classification.
}

#[test]
fn invalid_sealed_checked_cps_is_an_entry_verification_failure() {
    // A malformed/unchecked purported artifact cannot reach dispatch and is
    // classified InvalidCheckedCoreCps.
}

```

Use only Engine-owned admissions and deliberately invalid constructed evidence already accepted by the current test harness. Do not expose a public token constructor for tests.

**Step 2: Run the focused tests to verify RED**

Run:

```bash
cargo test -p ash-engine --test task_2014_checked_cps_admission --test task_2014_handler_production_admission -- --nocapture
```

Expected: compilation/test failure because the typed production-terminal classification does not yet exist.

**Step 3: Implement the smallest typed classification seam**

In `crates/ash-engine/src/lib.rs`, add a documented non-stringly production-boundary error/terminal classification that distinguishes exactly:

```rust
enum ProductionTerminalClassification {
    MissingAdmission,
    InvalidCheckedCoreCps,
}
```

Use it only at the sealed admission/driver boundaries. Preserve `EngineError` as the detailed internal cause where needed, do not widen public token construction, and do not use message matching to select a classification.

**Step 4: Run the focused tests to verify GREEN**

Run the Task 1 command again.

Expected: all focused Engine tests pass; unsupported source still does not execute directly; malformed Core/CPS never reaches a provider or handler. Do not add a fake handler-trap test: forged evidence is invalid Core/CPS, not a post-admission trap.

### Task 2: Project classified outcomes through the CLI terminal envelope

**Files:**
- Modify: `crates/ash-cli/src/commands/run.rs`
- Modify: `crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs`
- Modify: `crates/ash-cli/src/value_convert.rs` only if a helper avoids repeated construction; do not change its V1 schema.

**Step 1: Write failing binary tests**

Add exact JSON integration tests for:

```rust
// unsupported production lowering
json!({"schema_version": 1, "kind": "external", "boundary": "admission", "outcome": "rejected"})

// CLI-reachable invalid checked artifact fixture or an Engine-to-CLI test seam
json!({"schema_version": 1, "kind": "pre_entry_failure", "class": "entry_verification", "message": "checked Core/CPS artifact is invalid"})

```

For each JSON case assert failure status, no implementation telemetry, and one `--output` ownership control. Assert text mode keeps diagnostics rather than emitting JSON.

**Step 2: Run the terminal suite to verify RED**

Run:

```bash
cargo test -p ash-cli --test task_2008_runtime_terminal_envelope -- --nocapture
```

Expected: the newly added cases fail because the route currently maps through generic Engine/type errors or has no selected terminal projection.

**Step 3: Implement route-local projection**

In `crates/ash-cli/src/commands/run.rs`, consume the typed Engine classification before generic `classify_engine_error` conversion. Map exactly:

```rust
MissingAdmission => External { boundary: "admission", outcome: "rejected" }
InvalidCheckedCoreCps => PreEntryFailure { class: "entry_verification", message: "checked Core/CPS artifact is invalid" }
```

Return the existing `CliError` class that yields exits 1 and 4 respectively. Keep JSON emission exclusively in `emit_terminal_observable`; do not write an envelope twice and do not expose internals. The completed follow-on exact `trap_sleep` route admits its no-`resume`, identity-`done` `1 / 0` clause through checked Core/CPS and uses that same writer to project V1 `trap` (exit 5). It is not a general handler route.

**Step 4: Run the terminal suite to verify GREEN**

Run the Task 2 command again.

Expected: all existing 30 terminal controls and the new cases pass.

### Task 3: Record scope and traceability

**Files:**
- Modify: `docs/plan/tasks/TASK-2004-core-cps-production-boundary-decision.md`
- Modify: `docs/plan/tasks/TASK-2008-json-variant-observable-projection.md`
- Modify: `docs/plan/tasks/TASK-2014-source-handler-runtime-boundary-decision.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `CHANGELOG.md`

**Step 1: Add failing traceability/doc gate evidence if required**

Update the changed implementation fingerprints and add exact test anchors. Retain explicit exclusions for general handler lowering, arbitrary Core construction, and any direct-runtime fallback.

**Step 2: Run documentation gates**

Run:

```bash
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Expected: clean orientation and traceability validation.

### Task 4: Verify and review

**Files:**
- Verify only.

**Step 1: Run focused quality gates**

```bash
cargo fmt --all --check
git diff --check
cargo clippy -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings
cargo test -p ash-engine --test task_2014_checked_cps_admission --test task_2014_handler_production_admission
cargo test -p ash-cli --test task_2008_runtime_terminal_envelope
```

**Step 2: Run the repository pre-commit gate before any commit**

```bash
git diff --cached --check
git status --short
```

Do not commit without separate user authorization. Request a spec-compliance review followed by a Rust/code-quality review before reporting completion.
