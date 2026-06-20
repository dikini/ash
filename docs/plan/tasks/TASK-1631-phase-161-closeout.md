# TASK-1631: Close out Phase 161

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Close Phase 161 by reconciling status surfaces, changelog, reference docs, verification evidence, and review findings.

## Specification Reference

- [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- TASK-1620 through TASK-1630 complete or explicitly deferred with user approval.

## Requirements

### Functional Requirements

1. Re-read PLAN-161, all TASK-1620 through TASK-1630 files, SPEC-099, and SPEC-098b.
2. Run focused Phase 161 tests.
3. Run affected-crate gates.
4. Update PLAN-161 task table statuses.
5. Update PLAN-INDEX summary and Phase 161 section.
6. Update CHANGELOG.md with completed implementation details.
7. Request or perform independent review before marking complete.

### Property Requirements

- Status surfaces must agree.
- Closeout must not claim surface-to-Core lowering or full type checking.
- Any pre-existing `ash-interp` baseline failure must be classified honestly if broad gates are run.

## TDD Steps

### Step 1: Gather verification evidence

Run:

```bash
cargo test -p ash-core --test task_1620_core_ash_ast
cargo test -p ash-core --test task_1621_core_text_format
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
cargo test -p ash-core --test task_1623_core_text_parser_expressions
cargo test -p ash-core --test task_1624_core_text_serializer
cargo test -p ash-core --test task_1625_core_validator_basic
cargo test -p ash-core --test task_1626_core_validator_affine_resume
cargo test -p ash-core --test task_1627_core_to_cps_basic
cargo test -p ash-core --test task_1628_core_to_cps_effects
cargo test -p ash-core --test task_1629_core_end_to_end
cargo test -p ash-core --test task_1630_core_docs_consistency
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
```

Expected: all Phase 161 and affected `ash-core` gates pass.

### Step 2: Reconcile docs

**Files:** `docs/plan/PLAN-161-CORE-ASH-IR-FOUNDATION.md`, `docs/plan/PLAN-INDEX.md`, `docs/plan/tasks/TASK-1620-*.md` through `TASK-1630-*.md`, `CHANGELOG.md`

Update statuses only after verification evidence exists.

### Step 3: Review and commit

Request independent review focused on:

- Core/CPS boundary correctness;
- `.core` fixture format not drifting into surface syntax;
- validation/lowering responsibility split;
- stale overclaims in docs.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-core
  - cargo clippy -p ash-core --all-targets -- -D warnings
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] TASK-1620 through TASK-1630 status surfaces agree
  - [x] CHANGELOG.md updated
  - [x] PLAN-INDEX.md updated
  - [x] Independent review complete
```

## Completion Evidence

- Re-read PLAN-161, TASK-1620 through TASK-1630, TASK-1631, SPEC-099, and SPEC-098b.
- Ran all focused Phase 161 tests and affected `ash-core` gates listed above.
- Added closeout review artifact: [`PHASE-161-CLOSEOUT-REVIEW.md`](../audits/PHASE-161-CLOSEOUT-REVIEW.md).
- Reconciled PLAN-161, PLAN-INDEX, TASK-1631, and CHANGELOG status surfaces.

Verified on 2026-06-20:

```bash
cargo test -p ash-core --test task_1620_core_ash_ast
cargo test -p ash-core --test task_1621_core_text_format
cargo test -p ash-core --test task_1622_core_text_parser_atoms_values
cargo test -p ash-core --test task_1623_core_text_parser_expressions
cargo test -p ash-core --test task_1624_core_text_serializer
cargo test -p ash-core --test task_1625_core_validator_basic
cargo test -p ash-core --test task_1626_core_validator_affine_resume
cargo test -p ash-core --test task_1627_core_to_cps_basic
cargo test -p ash-core --test task_1628_core_to_cps_effects
cargo test -p ash-core --test task_1629_core_end_to_end
cargo test -p ash-core --test task_1630_core_docs_consistency
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
```
