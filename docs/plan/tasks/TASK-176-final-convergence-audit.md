# TASK-176: Final Convergence Audit

## Status: 📝 Planned

## Description

Run and publish the final convergence audit that closes the spec-to-implementation alignment
program.

This task rechecks the original drift classes against the updated specs, handoff references,
and affected Rust crates.

## Specification Reference

- All stabilized specs and reference contracts produced by the convergence program

## Audit Reference

- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`
- `docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md`

## Requirements

### Functional Requirements

1. Re-run the original drift classes as the acceptance checklist
2. Publish a final audit report stating which drift classes are closed
3. Run full repository verification for the changed convergence clusters
4. Record any remaining gaps explicitly rather than leaving them implicit

## Files

- Modify: `docs/audit/`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Use the original drift classes as the final acceptance checklist.

### Step 2: Verify RED

Confirm the current state has not yet been re-audited against the full convergence target.

### Step 3: Implement the final audit report (Green)

Record which drift classes are now closed and which, if any, remain.

### Step 4: Verify GREEN

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

Run any relevant repository gate scripts for changed clusters.

Expected: pass.

### Step 5: Commit

```bash
git add docs/audit CHANGELOG.md
git commit -m "docs: publish final convergence audit"
```

## Completion Checklist

- [ ] final acceptance checklist written
- [ ] final convergence audit report published
- [ ] full verification run
- [ ] remaining drift classes explicitly recorded if any remain
- [ ] `CHANGELOG.md` updated

## Non-goals

- No feature expansion beyond convergence work
- No new roadmap work unrelated to convergence closure

## Dependencies

- Depends on: TASK-171, TASK-173, TASK-175, TASK-207, TASK-208
- Blocks: convergence closeout
