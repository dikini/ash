# TASK-775: First-Class Workflow Closeout

## Status: 📝 Planned

## References

- [SPEC-056](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
- [PLAN-104](../PLAN-104-FIRST-CLASS-WORKFLOW-CARRIER.md)

## Objective

Close Phase 108 with examples, documentation reconciliation, changelog, and independent verification.

## Requirements

1. Add first-class workflow examples for `workflow::unit`/`bind`, `workflow::requires` / `workflow::ensures`, `requires:` / `ensures:` in `do:Workflow`, explicit lifts, and `[...]: Workflow`.
2. Mark deferred parallel/dynamic-admission/handle behavior honestly.
3. Update docs/spec/README.md, PLAN-INDEX.md, PLAN-104, and task statuses.
4. Update CHANGELOG.md.
5. Run full affected verification and independent subagent review.
6. Ensure no examples imply implicit Act/Proc lifts or workflow parallel semantics.
7. Include one deprecated legacy workflow declaration example only as a migration/compatibility warning example, paired with the equivalent first-class workflow expression.

## Verification

- [ ] Examples parse/check as expected or are explicitly marked reference-only.
- [ ] Contract-injection examples cover both statement-form and intrinsic-call spelling.
- [ ] Deprecated legacy declaration example emits a warning and has an equivalent first-class rewrite.
- [ ] Documentation status surfaces are reconciled.
- [ ] `cargo fmt --check` passes.
- [ ] Affected `cargo test` suites pass.
- [ ] Affected `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Independent subagent phase audit returns VERIFIED or findings are addressed.
- [ ] CHANGELOG.md updated.
