# Inline Module Role Honesty Fix Plan Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the remaining role-convergence review blocker by making inline-module parsing honest and spec-aligned, then narrow or justify the remaining role-lowering API surface.

**Architecture:** First fix the inline-module parser so canonical module items are either parsed through existing parsers or rejected explicitly instead of being skipped silently. Then make the role-lowering helper surface match its real usage, and finally refresh audit/bookkeeping text after fresh verification.

**Tech Stack:** Rust (`ash-parser`), existing parser modules (`parse_module`, `parse_workflow`, `parse_type_def`), Markdown specs/audits, and focused `cargo fmt` / `cargo test` / `cargo clippy` verification.

---

## Planning assumptions

- The current blocker is limited to `ash-parser` honesty/spec-compliance, not a broader role-model redesign.
- The flat role contract introduced by TASK-216 through TASK-224 is otherwise sound.
- Existing parser modules should be reused where possible instead of adding a second ad-hoc grammar.
- Rust API hygiene should follow the existing crate reality: no public or exported helper should exist without a non-test consumer.

## Sequencing rules

1. Fix silent inline-module item skipping before updating any closeout wording.
2. Reuse existing parser entry points where practical instead of extending the local mini-parser further.
3. Prefer narrowing unused API surface over inventing a fake production consumer.
4. Finish with evidence-backed verification and audit text refresh.

## Verification defaults

- `cargo fmt --check`
- `cargo test -p ash-parser`
- `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`
- focused audits with `rg` over `parse_module.rs`, `lower.rs`, `module.rs`, `docs/spec/SPEC-009-MODULES.md`, `docs/audit/2026-03-23-role-convergence-closeout-audit.md`, and `CHANGELOG.md`

---

### Task 1 / Make inline-module item handling honest and spec-aligned

**Files:**

- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Test: `crates/ash-parser/src/parse_module.rs` test module
- Test: `crates/ash-parser/src/lib.rs` test module

**Step 1: Write the failing tests**

Add focused parser tests for inline modules that currently get skipped silently:

- a `workflow` item inside `mod governance { ... }`
- a `datatype` / type-definition item inside `mod governance { ... }` if existing type-definition support is available through current parser modules
- a still-unsupported item that must now fail explicitly instead of being skipped

Also replace the current "after unknown braced item" tests with behavior that proves items are no longer discarded silently.

**Step 2: Run focused RED verification**

Run:

```bash
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_role_after_unknown_braced_item -- --nocapture
cargo test -p ash-parser parse_module::tests::test_parse_inline_module_capability_and_role_after_unknown_braced_item -- --nocapture
```

Expected: current tests pass for the wrong reason because `workflow` is skipped; the new tests should fail until parser behavior is corrected.

**Step 3: Implement the minimal parser fix**

In `crates/ash-parser/src/parse_module.rs`:

- extend `parse_definitions()` to stop treating canonical inline items as unknown text
- reuse existing parser entry points for any item type already supported elsewhere in the crate
- ensure any canonical item that still is not supported fails explicitly instead of falling through `skip_unknown_definition()`
- keep `skip_unknown_definition()` only for genuine recovery cases, not canonical top-level items recognized by `SPEC-009`

Prefer reusing existing identifier, workflow, and type-definition parsing utilities instead of growing the local duplicate grammar further.

**Step 4: Add the new passing assertions**

In the touched tests, assert one of these outcomes per item type:

- parsed definitions are preserved in `ModuleDecl::definitions()` when support exists
- unsupported canonical inline items produce a parse error immediately rather than being skipped

**Step 5: Run focused GREEN verification**

Run:

```bash
cargo test -p ash-parser parse_module::tests -- --nocapture
cargo test -p ash-parser lib_tests::test_module_decl_lowers_inline_module_roles_after_parse -- --nocapture
```

Expected: inline-module item tests pass and same-module role lowering still works.

**Step 6: Commit**

```bash
git add crates/ash-parser/src/parse_module.rs crates/ash-parser/src/lib.rs
git commit -m "fix: stop silently skipping inline module items"
```

---

### Task 2 / Make the role-lowering API surface honest

**Files:**

- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-parser/src/module.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Test: any `ash-parser` unit tests covering module role lowering

**Step 1: Write the failing contract check**

Add or adjust tests and visibility assertions so the crate no longer exports a role-lowering helper that has no non-test consumer.

The preferred contract is:

- crate-level lowering helpers used only by `ModuleDecl` tests become crate-private rather than exported as part of the parser surface
- test-only `ModuleDecl` helpers remain clearly test-only

**Step 2: Run focused RED verification**

Run:

```bash
cargo test -p ash-parser module::tests::test_inline_module_lower_role_definitions_uses_core_role_carrier -- --nocapture
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
```

Expected: the code still passes functionally, but the contract change is not yet reflected in visibility and lint cleanup.

**Step 3: Implement the minimal API cleanup**

In `crates/ash-parser/src/lower.rs`, `module.rs`, and `lib.rs`:

- narrow `lower_module_role_definitions()` to the smallest justified visibility
- stop re-exporting it publicly if no supported non-test caller exists
- remove stale lint suppression such as `#[allow(dead_code)]` from helpers that are now actively used
- keep the `&[Definition]`-based APIs and other recent ownership improvements intact

**Step 4: Run focused GREEN verification**

Run:

```bash
cargo test -p ash-parser
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
```

Expected: parser tests still pass, and the helper surface now matches actual usage.

**Step 5: Commit**

```bash
git add crates/ash-parser/src/lower.rs crates/ash-parser/src/module.rs crates/ash-parser/src/lib.rs
git commit -m "refactor: narrow module role lowering surface"
```

---

### Task 3 / Refresh spec and audit bookkeeping after the parser fix

**Files:**

- Modify: `docs/spec/SPEC-009-MODULES.md`
- Modify: `docs/audit/2026-03-23-role-convergence-closeout-audit.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing checklist**

Record the text that is currently ahead of the implementation:

- closeout audit says no blocker-class issues remain
- changelog implies unsupported canonical inline definitions are all rejected explicitly
- module spec and parser behavior need to be rechecked together after Task 1

**Step 2: Run RED audit**

Run:

```bash
rg -n "No blocker-class|unsupported canonical inline definitions|module_item\*|workflow_def|datatype_def" docs/audit/2026-03-23-role-convergence-closeout-audit.md CHANGELOG.md docs/spec/SPEC-009-MODULES.md
```

Expected: current wording still reflects the pre-fix review state.

**Step 3: Implement the documentation fix**

Update the touched docs so they describe the actual post-fix state precisely:

- if Task 1 adds parser support for additional inline items, record that support explicitly
- if some canonical inline items still remain unsupported, state that they are rejected explicitly rather than skipped silently
- only restore a full closeout claim once the parser/spec mismatch is resolved

**Step 4: Run GREEN verification**

Run:

```bash
cargo fmt --check
cargo test -p ash-parser
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
rg -n "No blocker-class|unsupported canonical inline definitions|module_item\*|workflow_def|datatype_def" docs/audit/2026-03-23-role-convergence-closeout-audit.md CHANGELOG.md docs/spec/SPEC-009-MODULES.md
```

Expected: verification passes and wording matches the actual implementation boundary.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-009-MODULES.md docs/audit/2026-03-23-role-convergence-closeout-audit.md CHANGELOG.md
git commit -m "docs: reconcile inline module closeout wording"
```

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-24-inline-module-role-honesty-fix-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Parallel Session (separate)** - Open a new session with executing-plans, batch execution with checkpoints
