# TASK-470: Final Verification for Capability Call Dispatch Split

## Status: Planned

## Description

Run and close out the verification slice for DESIGN-016 / PLAN-016 after spec, parser, resolver,
runtime, and engine migration work lands.

## Specification Reference

- [DESIGN-016: Capability Call Dispatch Split and Operational Call Sugar](../../design/DESIGN-016-CAPABILITY-CALL-DISPATCH.md)
- [PLAN-016: Capability Call Dispatch Split and Operational Call Sugar](../PLAN-016-CAPABILITY-CALL-DISPATCH.md)

## Dependencies

- ✅ [TASK-469](TASK-469-capability-call-docs-and-examples.md)

## Requirements

1. Run focused and cross-crate tests for parser, lowering, resolver, interpreter, engine, and CLI
   surfaces affected by split dispatch and operational call sugar.
2. Run formatting, clippy, and docs checks for the affected workspace slice.
3. Verify spec/docs/examples are aligned with the landed behavior.
4. Update plan/index/changelog status to reflect closeout.
5. Verify that the final canonical behavior can be recovered from `docs/spec/` alone, without
   treating design/plan docs as the semantic authority.

## TDD Steps

### Red

- Workspace may still contain stale flat-name assumptions after the migration.

### Green

- Verification gates pass for the affected workspace slice and closeout docs are accurate.

## Implementation Notes

- This task must explicitly check that the active `docs/spec/` files describe the landed behavior.
- Do not mark the phase complete if code passes but `docs/spec/` still describes the old overloaded
  ACT model.

## Completion Checklist

- [ ] focused and cross-crate tests pass
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets --all-features` passes
- [ ] `cargo doc --no-deps` for affected crates passes
- [ ] active specs/docs/examples checked for stale flat-name wording
- [ ] `PLAN-INDEX.md` and `CHANGELOG.md` updated for closeout
- [ ] `docs/spec/` confirmed as the final canonical authority for the completed phase
