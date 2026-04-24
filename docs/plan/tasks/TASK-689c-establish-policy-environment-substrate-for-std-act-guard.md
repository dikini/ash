# TASK-689C: Establish policy/environment substrate for `std::act` `guard`

## Status: ✅ Complete

## Description

Before `std/src/act.ash` can honestly replace its placeholder `guard` builtin declaration with an ordinary library implementation, Ash needs enough policy/environment surface to express the SPEC-047 `guard(policy, ma)` contract without faking runtime-only details.

This task now has two coupled goals:

1. Fix the immediate blocker that prevents honest ordinary-library `guard` implementation.
2. Pay down the broader language debt behind that blocker by making field/member access a real typed expression feature instead of a parser/runtime-only surface.

## Specification Reference

- SPEC-047 §2.5
- SPEC-047 §7
- SPEC-047 §8

## Dependencies

- 📝 TASK-689B: prerequisite task

## Requirements

### Functional Requirements

1. Implement honest expression-level record field projection typing in `ash-typeck` so `Expr::FieldAccess` is no longer hard-rejected.
2. Add focused tests covering successful record projection, nested projection, missing-field errors, and non-record-base errors.
3. Preserve honest reporting about the remaining `guard` surface gap after projection typing lands (for example, callable/member-call semantics and/or Rust-only `ActEnv` boundary).
4. If feasible within this task, add the smallest additional substrate needed to let ordinary-library `guard` typecheck honestly after projection typing.
5. Complete a design pass for the C3c boundary before exposing any runtime environment surface.
6. Keep the queue/task corpus honest about whether TASK-689 can proceed afterward.

### Property Requirements (proptest)

```rust
// Prefer focused regression/integration tests unless this task introduces
// a new broad invariant worth property coverage.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that expose the current inability to express honest ordinary-library `guard` behavior.

### Step 2: Implement (Green)

Land the substrate in explicit stages:

- C3a: general record field projection typing in `ash-typeck`
  - typecheck `Expr::FieldAccess` against `Type::Record`
  - project concrete field types
  - reject missing-field and non-record access with explicit errors
  - support nested projection chains honestly
- C3b: evaluate the callable/member-access gap exposed by SPEC-047's `env.policies.check(policy)` shape
  - determine whether projected-call support needs a dedicated `callee: Expr` call form or equivalent member-call representation
  - do not fake this with synthetic name-based calls
- C3c: resolve the policy/environment boundary needed by ordinary-library `guard`
  - complete a design pass before exposing any runtime environment surface
  - decide whether Phase 97 preserves the runtime-only `ActEnv` boundary or explicitly revises it
  - if the boundary is preserved, prefer a narrow explicit policy bridge over full environment exposure
  - spin out any broader Ash-visible environment feature into a separate spec/plan track rather than silently expanding TASK-689C

### Step 3: Integration (Green)

Verify both:

- the broad language debt payoff (`record.field`, nested projections, expression typing consistency), and
- the `guard`-specific parse/type/runtime boundary relevant to TASK-689.

### Step 4: Verification

Re-run focused checks and update docs/task status surfaces to match reality.

## Verification Steps

- [x] Focused tests capture the pre-fix field-access/typechecking blocker: `crates/ash-typeck/tests/expression_typing_soundness_test.rs` covers successful record projection, nested projection, missing-field rejection, non-record-base rejection, and projected callable invocation.
- [x] `ash-typeck` supports honest record field projection for expression typing: the landed `Expr::FieldAccess` coverage in `expression_typing_soundness_test.rs` verifies projected concrete field types.
- [x] Nested projection tests demonstrate value beyond the immediate `guard` blocker: `expression_typing_soundness_test.rs` verifies chained record projection and projected callable invocation.
- [x] Any remaining member-call or environment-surface blocker for literal SPEC-047 `guard` is documented explicitly: `std/src/act.ash` now uses the narrow `act::policy_check` bridge rather than exposing `ActEnv` as an Ash value, and `docs/notes/NOTE-006-C3C-ACTENV-EXPOSURE-DESIGN.md` records the preserved runtime-only boundary.
- [x] C3c design outcome is written down before any runtime environment exposure work begins: see `docs/notes/NOTE-006-C3C-ACTENV-EXPOSURE-DESIGN.md`.
- [x] TASK-689 status is updated honestly based on what C3a/C3b/C3c actually land: `docs/plan/tasks/TASK-689-create-stdlib-act-module.md` records TASK-689C as complete and TASK-689 as ready for closeout, with broader environment exposure deferred outside Phase 97.
- [x] `cargo fmt --check` passed during the TASK-689C implementation slice; this docs/status review did not modify Rust sources.

## Dependencies for Next Task

This task determines whether TASK-689 can finally replace all placeholder `std::act` helper declarations with ordinary library implementations.

## Notes

- Phase 97 is additive.
- `Act<A>` should remain abstract and first-class at the language level.
- Original blocker evidence was live and specific when TASK-689C was opened:
  - SPEC-047 requires `guard` to inspect policy state from an ordinary library function body.
  - `ash-typeck` previously rejected field access outright as `UnsupportedExpression` in `check_expr` (`Expr::FieldAccess`), so a spec-shaped `guard` body could not type-check honestly.
- The landed technical-debt payoff is intentionally broader than `guard`:
  - field access already existed in parser, lowered core IR, and runtime evaluation
  - expression typing has now caught up to that existing language/runtime surface
  - record projection is now supported in its own right, not merely as a `guard` special case
- Keep the staged interpretation honest:
  - C3a fixes general record projection typing debt and is expected to land regardless
  - C3b enables projected-call/member-call parsing/typechecking/runtime paths for expressions like `env.policies.check(p)`
  - C3c still determines whether literal SPEC-shaped `guard` can be expressed directly or still needs an explicit bridge/boundary clarification
- Current honest status after landing C3a/C3b/C3c:
  - record projection now typechecks in `ash-typeck`
  - projected callable invocation now parses as `FnApply` instead of degrading to a synthetic `call` name
  - projected callable invocation typechecks and evaluates when the callee is an ordinary record field holding a closure/function value
  - Phase 97 now provides a narrow `act::policy_check` bridge that preserves the runtime-only `ActEnv` boundary
  - `std::act::guard` can now be expressed as an ordinary library function without exposing `ActEnv` as an Ash value
- C3c design-pass output:
  - see `docs/notes/NOTE-006-C3C-ACTENV-EXPOSURE-DESIGN.md`
  - current recommendation is to preserve the runtime-only `ActEnv` boundary from TASK-683 / D3
  - Phase 97 should prefer a narrow explicit policy bridge for `guard` over full environment exposure
  - any broader Ash-visible environment feature should be spun out into a separate spec/plan track
- Do not fake `guard` with a Bool-only or panic-only placeholder and then call TASK-689 complete.
