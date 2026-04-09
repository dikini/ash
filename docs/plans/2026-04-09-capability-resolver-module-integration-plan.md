# Capability Resolver Module Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the bridge capability resolver with module/import-owned symbolic capability resolution.

**Architecture:** Capability declarations and imports provide canonical `(provider, action)` target
metadata. The module/import pipeline constructs one capability-resolution context that is consumed
by lowering and type checking; std capability symbols enter through the same path; explicit
`provider:action(...)` remains direct.

**Tech Stack:** Rust 2024, `ash-parser`, `ash-typeck`, module graph/import resolver, active specs.

---

### Task 1: Freeze the Spec Contract

**Files:**
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-009-MODULES.md`
- Modify: `docs/spec/SPEC-012-IMPORTS.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Reference: `docs/plan/tasks/TASK-471-spec-module-owned-capability-resolution.md`

**Step 1:** Write failing doc-review notes for every bridge-specific claim that is still described as final semantics.

**Step 2:** Update the spec text so the final intended contract is module-owned symbolic resolution and the bridge remains explicitly transitional until later tasks complete.

**Step 3:** Re-read the spec set to confirm `provider:action(...)` remains direct and symbolic module-qualified names remain module-path based.

### Task 2: Add Capability Symbol Export Metadata

**Files:**
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/module.rs`
- Modify: `crates/ash-parser/src/import_resolver.rs`
- Test: module/import resolver tests in `crates/ash-parser/tests/`
- Reference: `docs/plan/tasks/TASK-472-capability-symbol-export-metadata.md`

**Step 1:** Write failing tests for module-local capability declarations not exposing enough metadata for symbolic operational resolution.

**Step 2:** Add explicit capability symbol export metadata that preserves symbolic name, canonical target pair, and visibility.

**Step 3:** Run targeted parser/import tests and confirm they pass.

### Task 3: Resolve Imported Capability Symbols

**Files:**
- Modify: `crates/ash-parser/src/import_resolver.rs`
- Test: import/multi-crate tests in `crates/ash-parser/tests/`
- Reference: `docs/plan/tasks/TASK-473-imported-capability-symbol-bindings.md`

**Step 1:** Write failing tests for imported, aliased, re-exported, and qualified capability symbols.

**Step 2:** Extend import resolution so those bindings preserve canonical `(provider, action)` targets.

**Step 3:** Run the focused import-resolution tests and confirm they pass.

### Task 4: Build the Shared Resolver Context

**Files:**
- Modify: `crates/ash-parser/src/resolver.rs`
- Modify: `crates/ash-parser/src/import_resolver.rs`
- Modify: any shared resolver context module introduced by the task
- Reference: `docs/plan/tasks/TASK-474-capability-resolution-context-pipeline.md`

**Step 1:** Write a failing integration test showing lowering/type checking still require local resolver construction.

**Step 2:** Define and thread a shared capability-resolution context through the compile-time pipeline.

**Step 3:** Run targeted integration tests and confirm the context is available to downstream consumers.

### Task 5: Migrate Lowering

**Files:**
- Modify: `crates/ash-parser/src/lower.rs`
- Remove/trim: `crates/ash-parser/src/capability_resolver.rs` if it is no longer the right ownership site
- Test: lowering tests in `crates/ash-parser/src/lower.rs` and related parser tests
- Reference: `docs/plan/tasks/TASK-475-lowering-module-owned-capability-resolution.md`

**Step 1:** Write failing tests proving lowering still depends on local built-in mappings.

**Step 2:** Replace local bridge construction with the shared context.

**Step 3:** Run targeted lowering tests and confirm unresolved symbolic names fail explicitly.

### Task 6: Migrate Type Checking and Capability Checking

**Files:**
- Modify: `crates/ash-typeck/src/names.rs`
- Modify: `crates/ash-typeck/src/capability_check.rs`
- Test: `crates/ash-typeck/src/capability_check.rs`, `crates/ash-typeck/tests/`
- Reference: `docs/plan/tasks/TASK-476-typecheck-module-owned-capability-resolution.md`

**Step 1:** Write failing tests for symbolic/imported/qualified ACT checks using a shared context.

**Step 2:** Remove local built-in resolver construction and consume the pipeline-owned context.

**Step 3:** Run `cargo test -p ash-typeck --lib` and any focused integration tests.

### Task 7: Remove the Bridge for Std Capability Symbols

**Files:**
- Modify: std capability source/bootstrap paths used by the module resolver
- Modify: `crates/ash-parser/src/resolver.rs`
- Modify: `crates/ash-parser/src/import_resolver.rs`
- Reference: `docs/plan/tasks/TASK-477-stdlib-capability-bootstrap-and-bridge-removal.md`

**Step 1:** Write failing tests showing std symbolic names still depend on built-in bridge tables.

**Step 2:** Source std capability metadata from the module pipeline instead.

**Step 3:** Remove obsolete built-in bridge code and rerun targeted parser/typecheck tests.

### Task 8: Docs and Verification

**Files:**
- Modify: `docs/spec/` files touched by the bridge wording
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Reference: `docs/plan/tasks/TASK-478-module-owned-capability-resolution-docs.md`
- Reference: `docs/plan/tasks/TASK-479-module-owned-capability-resolution-verification.md`

**Step 1:** Remove bridge-status wording only after the code no longer uses the bridge.

**Step 2:** Run:

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --lib
cargo doc --no-deps --workspace
```

**Step 3:** Update `PLAN-INDEX.md` and `CHANGELOG.md` with the actual outcome.
