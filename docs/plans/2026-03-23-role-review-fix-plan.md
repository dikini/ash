# Role Review Fix Plan Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Resolve the remaining review findings around role-convergence correctness, dead-code smell, and spec compliance without widening scope beyond the touched parser/spec surface.

**Architecture:** First make inline-module capability constraints match the canonical surface grammar so role-authority metadata survives lowering honestly. Then resolve the current test-only status of the new role-lowering API by wiring it into a real parser-facing path or narrowing the exposed surface so the branch no longer overclaims end-to-end support. Finally, remove the stale role-obligation grammar residue from the canonical surface spec and run focused verification.

**Tech Stack:** Rust (`ash-parser`), Markdown specs, parser unit/integration tests, focused repository audits, cargo fmt/test/clippy.

---

## Planning assumptions

- The current branch already passed focused formatter/tests/clippy for `ash-core` and `ash-parser`.
- The remaining work is bounded to the three review findings, not a broader role-model redesign.
- The canonical source of truth for role syntax remains [docs/spec/SPEC-002-SURFACE.md](../spec/SPEC-002-SURFACE.md).
- The fix should preserve the flat role contract and avoid reopening hierarchy/supervision semantics.

## Sequencing rules

1. Fix the parser/spec mismatch before adjusting claims about the role-lowering path.
2. Keep parser changes minimal and spec-driven; do not add broader capability parsing than needed.
3. Resolve the public-API dead-code smell either by adding a real non-test consumer or by narrowing visibility/claims; do not leave the branch in the current ambiguous state.
4. Finish with verification and a focused search audit.

## Verification defaults

- `cargo fmt --check`
- `cargo test -p ash-parser`
- `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`
- focused `rg` audits for touched spec text and role-lowering API call sites

---

### Task 1 / Spec-align inline capability constraint parsing

**Files:**

- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Test: `crates/ash-parser/src/parse_module.rs` test module
- Test: `crates/ash-parser/src/lib.rs` test module

**Step 1: Write the failing tests**

Add focused tests proving that inline-module capability constraints with canonical predicate syntax and arguments are preserved through parsing and role-authority lowering.

Examples to cover:

- `where requires_mfa()`
- `where requires_region("EU")`
- at least one same-module role authority case that expects the parsed argument to survive lowering into `ash_core::Constraint`

**Step 2: Run focused RED verification**

Run: `cargo test -p ash-parser parse_module::tests::test_parse_inline_module_with_capability -- --nocapture`

Run: `cargo test -p ash-parser lib_tests::test_public_api_preserves_same_module_capability_metadata_for_role_authority -- --nocapture`

Expected: failure because `parse_constraint()` currently only supports a bare name plus optional empty parens and always drops arguments.

**Step 3: Implement the minimal parser fix**

In `crates/ash-parser/src/parse_module.rs`:

- teach `parse_constraint()` to parse canonical predicate syntax instead of the current name-only placeholder
- preserve parsed predicate arguments in `Predicate.args`
- keep the implementation narrowly scoped to the subset needed for module capability constraints used by role lowering

Prefer reusing existing expression/literal parsing helpers where practical instead of inventing a second incompatible argument grammar.

**Step 4: Update public API tests to canonical syntax**

In `crates/ash-parser/src/lib.rs` and the local parser tests:

- replace non-canonical `where requires_mfa` examples with canonical predicate syntax
- assert the argument-preservation behavior explicitly

**Step 5: Run focused GREEN verification**

Run: `cargo test -p ash-parser`

Expected: passing parser tests, including the new constraint cases.

**Step 6: Commit**

Commit message: `fix: preserve canonical constraint metadata in module role lowering`

---

### Task 2 / Resolve the test-only role-lowering API smell

**Files:**

- Modify: `crates/ash-parser/src/module.rs`
- Modify: `crates/ash-parser/src/resolver.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Modify: `docs/plan/tasks/TASK-222-integrate-role-definition-lowering-path.md` (if the implementation story changes)
- Modify: `docs/audit/2026-03-23-role-convergence-closeout-audit.md` (if the audit wording changes)
- Test: module/resolver/parser integration tests as needed

**Implementation target:**
Make the role-lowering API honest in one of these two bounded ways, preferring Option A.

- **Option A (preferred):** add a real non-test parser-facing consumer for `ModuleDecl::lower_role_definitions()` so the API is no longer repo-local dead surface area.
- **Option B (fallback):** if no real consumer belongs in this phase, narrow the API/wording so the branch stops claiming broader end-to-end integration than it actually provides.

**Step 1: Write the failing usage/contract test**

Choose the option before coding, then add a test that enforces it:

- For Option A: add a non-test-facing parser facade or resolver-path test that uses parsed `ModuleDecl` data and lowered roles outside the current unit-test-only path.
- For Option B: add/adjust tests and docs to ensure the helper is not presented as a production-consumed end-to-end path.

**Step 2: Run focused RED verification**

Run the smallest targeted `ash-parser` test selection covering the chosen contract.

Expected: failure showing the current branch still leaves the API test-only or overclaimed.

**Step 3: Implement the bounded fix**

Preferred direction:

- keep `ModuleDecl` as the surface carrier
- either route a real parser-facing flow through the helper or reduce visibility/claims
- avoid dragging the entire module resolver onto the full parser unless the change remains small and obviously local

Concrete acceptance criteria:

- there is a clear reason this API exists
- the reason is enforced by code/tests rather than only by comments
- audit/task wording matches the actual implementation scope

**Step 4: Run focused GREEN verification**

Run: `cargo test -p ash-parser`

If `resolver.rs` changes materially, add a focused resolver test run or broader crate run confirming no regression.

**Step 5: Commit**

Commit message: `refactor: make module role lowering path honest`

---

### Task 3 / Remove dead role-obligation grammar residue from SPEC-002

**Files:**

- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/audit/2026-03-23-role-convergence-closeout-audit.md` (if the audit should mention this cleanup)

**Step 1: Write the failing checklist**

Record the stale grammar issue explicitly:

- `obligation_ref` remains in the role-definition grammar block even though `role_def` now uses `workflow_obligation_ref` only

**Step 2: Run RED audit**

Run: `rg -n "obligation_ref|workflow_obligation_ref" docs/spec/SPEC-002-SURFACE.md`

Expected: both the live name-only grammar and the stale deontic production still appear together.

**Step 3: Implement the minimal spec fix**

In `docs/spec/SPEC-002-SURFACE.md`:

- remove the orphaned `obligation_ref` production from the role-definition grammar block
- keep the surrounding prose that clarifies role obligations are name-only references lowering to `RoleObligationRef`
- ensure no remaining nearby text implies deontic role-obligation syntax is still canonical here

**Step 4: Run GREEN audit**

Run the same `rg` audit and manually verify the role-definition section reads consistently.

**Step 5: Commit**

Commit message: `docs: remove stale deontic role obligation grammar`

---

### Task 4 / Final verification and closeout note refresh

**Files:**

- Modify: `docs/audit/2026-03-23-role-convergence-closeout-audit.md`
- Modify: `CHANGELOG.md`

**Step 1: Re-run verification**

Run:

- `cargo fmt --check`
- `cargo test -p ash-parser`
- `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`

**Step 2: Re-run focused audits**

Run:

- `rg -n "obligation_ref|workflow_obligation_ref" docs/spec/SPEC-002-SURFACE.md`
- `rg -n "lower_role_definitions\(|role_definitions\(" crates/ash-parser/src`

**Step 3: Refresh the audit note**

Update the closeout audit so it no longer overstates the branch state and explicitly records the resolution of:

- canonical constraint parsing on the module role-lowering path
- the role-lowering API honesty/dead-code issue
- the stale spec grammar residue

**Step 4: Update changelog**

Add an `Unreleased` entry summarizing the review-driven parser/spec cleanup.

**Step 5: Commit**

Commit message: `chore: close review-driven role convergence fixes`

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-23-role-review-fix-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch a fresh subagent per task and review between tasks
2. **Parallel Session (separate)** - Open a new session with executing-plans and run the plan with checkpoints
