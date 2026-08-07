# Module Source Authority Fence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fence legacy Engine source scanners so parser-owned module definitions are the only normal source of exported module facts.

**Architecture:** `collect_module_exports` already obtains an optional parser-owned `ModuleBody`. The implementation will route all root export publication through that body when present and retain existing scanners only for the explicit parser-failure compatibility path. Callable visibility checks will use the same parser-owned definitions. This is a non-authorizing transport/loader change; TASK-2063 remains the sealing boundary and roles/policies remain stubs.

**Tech Stack:** Rust 2024, `ash-parser` surface/module AST, `ash-engine` module loader, Cargo tests/clippy/fmt, Common Changelog and semantic traceability JSON.

---

### Task 1: Add red scanner-authority regressions

**Files:**
- Modify: `crates/ash-engine/src/module_loader/tests.rs`
- Modify: `crates/ash-engine/tests/task_2069_module_transport_fencing.rs`

**Step 1: Write the failing test**

Add a module-loader regression with a root module containing an inline child
whose public function is not re-exported. Assert the parent `ModuleExports` has
no callable with the child function name and the child module owns it. Add a
typed-definition fixture containing root and nested public builtin/capability
declarations and assert only root declarations are published.

**Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p ash-engine --lib module_loader::tests --no-default-features -- nested_inline_public_callable
```

Expected: FAIL because the raw braced scanner currently flattens the nested
public callable into the parent export map.

### Task 2: Route normal exports through parser-owned definitions

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs`
- Modify: `crates/ash-engine/src/module_loader/callable_exports.rs`

**Step 1: Implement typed builtin conversion**

Factor builtin callable construction so a parsed `BuiltinFnDef` can become an
`ImportedCallableExport` without reparsing a source snippet.

**Step 2: Gate capability, builtin, and public-function publication**

When `authoritative_body` is `Some`, iterate its typed definitions for public
capabilities, public builtins, and public functions. Do not execute the raw
capability/builtin/function scanners in that branch. Keep the existing scanner
loops only in the `None` compatibility branch.

**Step 3: Gate callable signature visibility scans**

Use parser-owned public function/builtin definitions for normal inputs. Preserve
the existing snippet scanner only when authoritative parsing fails, and document
that path as compatibility-only.

### Task 3: Run focused red/green verification

**Files:** None.

**Step 1: Run the focused loader tests**

```bash
cargo test -p ash-engine --lib module_loader::tests
cargo test -p ash-engine --test task_2069_module_transport_fencing
```

Expected: PASS, including nested-module isolation, existing versioned-import
fallback tests, and the new scanner-fence regressions.

**Step 2: Run affected crate checks**

```bash
cargo test -p ash-engine
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Expected: all commands succeed without warnings or formatting changes.

### Task 4: Update semantic evidence and changelog

**Files:**
- Modify: `docs/plan/tasks/TASK-2069-complete-module-lowering-and-engine-transport-fencing.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

Record the scanner-authority regression as tested evidence, retain the explicit
compatibility fallback gap, and state that roles/policies remain metadata-only.
Add one Common Changelog entry under `[Unreleased]` referencing TASK-2069.

### Task 5: Run repository completion gates

**Files:** None.

Run:

```bash
cargo test --workspace --no-fail-fast
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

Expected: workspace tests, orientation-index validation, and docs gate pass.
