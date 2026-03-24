# Role Convergence Review Follow-up Design

## Goal

Resolve the remaining review finding in the inline-module honesty work while also reconciling the nearby non-blocking review follow-ups, without widening scope into a broader parser redesign or role-model change.

## Context

A fresh review of the current worktree found one remaining blocker in the inline-module parser:

- unsupported canonical inline module items are rejected only when they appear as the immediate next top-level token;
- after parser recovery enters `skip_unknown_definition()`, recovery only resynchronizes on `role` and `capability`;
- this still permits a shape where an unsupported canonical item such as `workflow`, `policy`, `datatype`, or `pub capability` can be consumed as skipped text after an earlier unknown item.

That behavior conflicts with the current `SPEC-009` requirement that unsupported canonical inline items must be rejected explicitly rather than silently skipped.

The same review also found two adjacent non-blocking issues:

1. the closeout and task wording now overstates the branch state in a few places;
2. the test-only role-lowering helpers are now visibility-narrowed correctly, but the surrounding wording should stay explicit that this remains a test-only crate-internal path.

## Constraints

- Keep the flat role contract unchanged.
- Keep the current test-only inline-module role-lowering path unless a real non-test consumer already exists locally.
- Avoid a broad parser refactor in this follow-up.
- Prefer focused regression tests and honest wording over speculative API expansion.

## Options Considered

### Option A: Targeted parser honesty fix plus wording cleanup (recommended)

Adjust inline-module recovery so it stops before unsupported canonical items and lets the explicit rejection path fire. Then update the affected audit/task wording to match the resulting implementation state.

**Pros**

- Fixes the real blocker directly.
- Keeps scope bounded to the reviewed issue.
- Preserves current passing role-lowering coverage.
- Minimizes regression risk.

**Cons**

- Leaves the larger local mini-parser duplication in place.
- Does not materially simplify parser structure.

### Option B: Broader parser consolidation

Fix the blocker and also refactor `parse_module.rs` to reuse more shared parser entry points for inline definitions.

**Pros**

- Better long-term maintainability.
- Reduces grammar duplication.

**Cons**

- Much larger scope.
- Higher chance of destabilizing unrelated parsing behavior.
- Not required to close the current blocker.

### Option C: Wording-only downgrade

Do not change code; weaken the audit/task wording to match current parser behavior.

**Pros**

- Minimal effort.

**Cons**

- Leaves the correctness/spec-compliance bug in place.
- Does not satisfy the review request.

## Recommended Design

### 1. Fix parser recovery honesty

In `crates/ash-parser/src/parse_module.rs`:

- keep explicit top-level rejection for unsupported canonical inline items;
- update recovery so that once skipping begins, resynchronization also stops before unsupported canonical item starts, not just before `role` and `capability`;
- preserve recovery for genuinely unknown non-canonical items;
- add regression tests covering mixed sequences such as unknown item → unsupported canonical item → supported item.

This keeps the current parser strategy but removes the remaining silent-skip path.

### 2. Keep role-lowering surface bounded and honest

Do not widen the module role-lowering API. The current state should remain:

- `RoleLoweringError` and related helpers in `crates/ash-parser/src/lower.rs` stay test-only;
- `ModuleDecl::lower_role_definitions()` remains a test-only crate-internal helper;
- nearby docs/tasks should describe this as a maintained regression-covered test-only path, not a general parser-facing lowering API.

### 3. Reconcile wording with implementation truth

Update the touched docs/tasks so they match the post-fix state precisely:

- `docs/audit/2026-03-23-role-convergence-closeout-audit.md`
- `docs/plan/tasks/TASK-218-implement-source-role-definition-parsing-and-lowering.md`
- any nearby task/audit wording that still references placeholder lowering or overclaims closeout status.

### 4. Leave broader parser cleanup out of scope

The mini-parser duplication in `parse_module.rs` remains a non-blocking maintainability issue. It should be recorded as future cleanup rather than bundled into this follow-up.

## Acceptance Criteria

This design is successful when:

1. unsupported canonical inline items cannot be silently skipped, even after recovery from an earlier unknown item;
2. focused regression tests cover the reviewed failure mode;
3. `ash-parser` formatting, tests, and clippy all pass;
4. audit/task wording no longer overstates implementation state;
5. the test-only role-lowering path remains clearly documented as such.
