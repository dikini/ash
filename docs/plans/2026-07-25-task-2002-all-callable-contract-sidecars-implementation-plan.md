# TASK-2002 All-Callable Contract Sidecars Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Retain each local function's fully lowered `requires`/`ensures` contract sidecars on the parsed Engine entry without changing execution, authority, or admission.

**Architecture:** `ash_parser::lower::lower_fn_contract` already constructs complete predicate, discharge, blame, and runtime-postcondition artifacts. During Engine source-entry lowering, derive that artifact for every local `FnDef`, store it in a deterministic callable-name map on `EntryLoweringSidecars`, and fail parsing atomically if any contract cannot lower. The retained map is read-only diagnostic/evidence data; it does not install checks, frames, providers, monitors, or a direct-evaluator fallback.

**Tech Stack:** Rust 2024, `ash-parser` contract lowering, `ash-engine` entry lowering, Cargo tests, semantic traceability/docs gates.

---

### Task 1: Prove all-callable retention is currently absent

**Files:**
- Modify: `crates/ash-engine/tests/task_2002_do_lowering_sidecars.rs`
- Test: `crates/ash-engine/tests/task_2002_do_lowering_sidecars.rs`

**Step 1: Write the failing tests**

Add a source module with both a helper and `main`, each having supported `requires` and/or `ensures` clauses. Assert that the parsed `Entry` exposes a deterministic sidecar entry for both callable names, with complete lowered discharge records and runtime postconditions. Assert the existing row sidecar remains non-granting and no execution/admission API is invoked.

Add one invalid helper-contract source. Assert parsing/lowering fails and therefore returns no partial `Entry` sidecar map.

**Step 2: Run the focused test to verify RED**

Run: `cargo test -p ash-engine --test task_2002_do_lowering_sidecars`

Expected: the new assertions fail because `EntryLoweringSidecars` currently retains no lowered callable contracts.

### Task 2: Retain and validate all local callable contracts

**Files:**
- Modify: `crates/ash-engine/src/lib.rs:451-500`
- Modify: `crates/ash-engine/src/lib.rs:1479-1660`
- Test: `crates/ash-engine/tests/task_2002_do_lowering_sidecars.rs`

**Step 1: Add the sidecar carrier**

Add a deterministic map keyed by local callable name to `EntryLoweringSidecars`. Its values retain the existing `ash_parser::lower::LoweredFnContract` intact rather than reducing records to strings or counts.

**Step 2: Lower contracts during the local-function pass**

For every `Definition::Function`, construct `FnContractLoweringContext` from the callable name, surface parameters converted to `CoreType`, and declared return type converted to `CoreType`; call `lower_fn_contract` before publishing the `Entry`. Convert a lowering failure into the existing source-entry error path so no partial `Entry` can exist.

**Step 3: Keep scope non-authorizing**

Do not mutate `callable_row_requirements`, provider bindings, handler facts, Core terms, CPS terms, runtime monitor state, or admission code. Preserve source expansion/origin sidecars unchanged.

**Step 4: Run focused tests to verify GREEN**

Run: `cargo test -p ash-engine --test task_2002_do_lowering_sidecars`

Expected: all current and new tests pass.

### Task 3: QA, review, and evidence

**Files:**
- Modify: `docs/plan/tasks/TASK-2002-generic-do-and-lowering-sidecar-strategy.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json` (only if a new implementation/test node is warranted)
- Modify: `CHANGELOG.md`

**Step 1: QA**

Run focused engine tests and library tests, Clippy with warnings denied, format, semantic traceability, docs, and whitespace-diff gates.

**Step 2: Record bounded evidence**

Update task/index/changelog/traceability to say all local callable contract artifacts survive entry lowering, while execution, enforcement, monitors, handler/provider sidecars, and full conformance remain deferred.

**Step 3: Review**

Review that the map is deterministic, complete for every local function (including helpers), cannot expose a partial entry on an invalid contract, and has no authority/runtime effect.
