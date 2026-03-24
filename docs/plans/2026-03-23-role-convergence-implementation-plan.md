# Role Convergence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bring the current parser, core, runtime-adjacent role handling, and examples into alignment with the simplified canonical role contract: authority plus obligations, with no role supervision.

**Architecture:** First remove `supervises` from canonical-facing parser/core data shapes, then implement end-to-end source role-definition parsing and lowering, then align runtime approval-role handling with the flat role contract, and finally update examples/docs and close with a focused audit. The plan deliberately avoids inventing new hierarchy semantics and keeps workflow/process supervision out of the role model.

**Tech Stack:** Markdown specs and planning docs, Rust crates (`ash-parser`, `ash-core`, `ash-interp`), parser/lowering tests, runtime policy tests, and Ash example workflows.

---

## Planning assumptions

- [TASK-216](../plan/tasks/TASK-216-canonicalize-role-contracts.md) is complete and is the normative spec baseline.
- Role convergence is intentionally narrower than a full governance redesign.
- Approval-by-role remains part of policy/runtime behavior, but role hierarchy does not.
- TDD applies to parser/core/runtime changes.

## Sequencing rules

1. Remove legacy role-shape fields before adding new parsing behavior.
2. Do not reintroduce role hierarchy through examples, helper code, or approval routing.
3. Keep runtime role handling flat unless a later spec explicitly widens it.
4. Update examples only after parser/core contracts are aligned.

## Verification defaults

Use the smallest focused verification first, then widen:

- `cargo test -p ash-parser`
- `cargo test -p ash-core`
- `cargo test -p ash-interp`
- `cargo test --all`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`

---

### Task 1 / TASK-217: Remove Legacy Role Supervision Shape

**Contract:** Remove `supervises` from canonical-facing parser/core role data structures and from the placeholder lowering path so the in-repo model matches the simplified spec.

**Files:**
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/token.rs`
- Modify: `crates/ash-parser/src/lexer.rs`
- Modify: `crates/ash-parser/src/parse_pattern.rs`
- Modify: `crates/ash-parser/src/parse_expr.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-core/src/ast.rs`
- Test: parser/core unit tests that currently assert `supervises`
- Modify: `CHANGELOG.md`

**Step 1: Write failing tests**

Add or update focused tests that assert role data structures no longer expose `supervises`.

**Step 2: Verify RED**

Run:

```bash
cargo test -p ash-parser
cargo test -p ash-core
```

Expected: failures in role-structure tests or compilation failures where `supervises` is still referenced.

**Step 3: Implement the minimal fix**

Remove the legacy field and update keyword/reserved-word handling plus placeholder lowering.

**Step 4: Verify focused GREEN**

Run:

```bash
cargo test -p ash-parser
cargo test -p ash-core
```

Expected: pass.

**Step 5: Commit**

```bash
git add crates/ash-parser/src/surface.rs crates/ash-parser/src/token.rs crates/ash-parser/src/lexer.rs crates/ash-parser/src/parse_pattern.rs crates/ash-parser/src/parse_expr.rs crates/ash-parser/src/parse_workflow.rs crates/ash-parser/src/lower.rs crates/ash-core/src/ast.rs CHANGELOG.md
git commit -m "refactor: remove legacy role supervision shape"
```

---

### Task 2 / TASK-218: Implement Source Role Definition Parsing and Lowering

**Contract:** Support source `role` definitions end-to-end through parser and lowering using the simplified authority-plus-obligations contract.

**Files:**
- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Modify: `crates/ash-parser/src/resolver.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Test: new parser tests for role definitions
- Test: lowering tests for role definitions
- Modify: `CHANGELOG.md`

**Step 1: Write failing parser/lowering tests**

Add focused tests for parsing a role definition with authority and obligations and lowering it into the corresponding core representation.

**Step 2: Verify RED**

Run:

```bash
cargo test -p ash-parser role
```

Expected: fail because inline/module definition parsing still skips full role-definition parsing.

**Step 3: Implement the minimal fix**

Add role-definition parsing in `parse_module`, preserve the simplified shape in the surface AST, and lower it consistently.

**Step 4: Verify focused GREEN**

Run:

```bash
cargo test -p ash-parser
```

Expected: pass.

**Step 5: Commit**

```bash
git add crates/ash-parser/src/parse_module.rs crates/ash-parser/src/lib.rs crates/ash-parser/src/resolver.rs crates/ash-parser/src/surface.rs crates/ash-parser/src/lower.rs CHANGELOG.md
git commit -m "feat: parse and lower role definitions"
```

---

### Task 3 / TASK-219: Align Runtime Approval-Role Handling with the Flat Role Contract

**Contract:** Keep runtime role handling intentionally flat: role identity is sufficient for policy evaluation and approval routing, with no hierarchy or inherited authority assumptions.

**Files:**
- Modify: `crates/ash-interp/src/capability_policy.rs`
- Modify: `crates/ash-interp/src/capability_policy_runtime.rs`
- Modify: `crates/ash-interp/src/error.rs`
- Test: `crates/ash-interp/tests/policy_runtime_outcomes.rs`
- Test: any role/approval-focused runtime tests
- Modify: `CHANGELOG.md`

**Step 1: Write failing tests or assertions**

Add focused runtime-policy tests that lock in direct approval-by-role behavior without any hierarchy lookup.

**Step 2: Verify RED**

Run:

```bash
cargo test -p ash-interp policy_runtime_outcomes -- --nocapture
```

Expected: fail if runtime code or tests still imply richer role semantics than direct named-role approval.

**Step 3: Implement the minimal fix**

Tighten runtime role docs/types/tests so approval routing remains flat and explicit.

**Step 4: Verify focused GREEN**

Run:

```bash
cargo test -p ash-interp --test policy_runtime_outcomes -- --nocapture
```

Expected: pass.

**Step 5: Commit**

```bash
git add crates/ash-interp/src/capability_policy.rs crates/ash-interp/src/capability_policy_runtime.rs crates/ash-interp/src/error.rs crates/ash-interp/tests/policy_runtime_outcomes.rs CHANGELOG.md
git commit -m "test: lock flat approval role contract"
```

---

### Task 4 / TASK-220: Align Examples, Docs, and Residual Role References

**Contract:** Remove stale `supervises` usage from examples and residual docs, then close with a focused role-convergence audit.

**Files:**
- Modify: `examples/03-policies/01-role-based.ash`
- Modify: `examples/code_review.ash`
- Modify: `examples/multi_agent_research.ash`
- Modify: `examples/workflows/40_tdd_workflow.ash`
- Modify: `examples/04-real-world/customer-support.ash`
- Modify: `examples/04-real-world/code-review.ash`
- Modify: any remaining docs that still present `supervises` as canonical
- Create or modify: focused audit note if needed
- Modify: `CHANGELOG.md`

**Step 1: Write the failing checklist**

List every example/doc that still presents `supervises` as part of the role contract.

**Step 2: Verify RED**

Run a focused search:

```bash
grep -R "supervises" examples docs/spec docs/reference
```

Expected: remaining canonical/example hits.

**Step 3: Implement the minimal fix**

Remove stale `supervises` examples, update wording, and record any residual non-canonical references that are intentionally historical.

**Step 4: Verify GREEN**

Run:

```bash
grep -R "supervises" examples docs/spec docs/reference
cargo test --all
```

Expected: only intentional historical/planning references remain, and tests pass.

**Step 5: Commit**

```bash
git add examples docs CHANGELOG.md
git commit -m "docs: align examples with simplified role contract"
```

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-03-23-role-convergence-implementation-plan.md`.

Two execution options:

1. **Subagent-Driven (this session)** — dispatch a fresh subagent per task and review between tasks.
2. **Parallel Session (separate)** — open a new session and execute with `superpowers:executing-plans`.
