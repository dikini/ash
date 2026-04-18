# TASK-606: Workflow::Call Runtime Completion

## Status: ✅ Complete

## Description

Complete runtime `Workflow::Call` execution across explicit runtime registration, typechecking visibility for registered callable workflows, and big-step/small-step execution paths. This task intentionally excludes the larger surface/program representation changes needed for local helper workflow declarations in ordinary source files; those land in TASK-611.

## Specification Reference

- `docs/design/DESIGN-027-SMALL-STEP-IR-COMPRESSION.md`
- `docs/spec/SPEC-001-IR.md`
- `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`

## Dependencies

- ✅ TASK-604: Small-Step IR Compression Prototype
- ✅ TASK-605: Statement Lifting and Pipe Operator Prototype

## Requirements

### Functional Requirements

1. Big-step interpreter executes `Workflow::Call` instead of returning a stub error.
2. Small-step interpreter executes `Stmt::Call` instead of returning a stub error.
3. Runtime/engine expose an explicit callable-workflow registration path with enough metadata to bind arguments honestly.
4. Typechecker can see registered callable workflow signatures at call sites.
5. Resolution behavior is explicit and test-covered for:
   - registered callable workflow call
   - unknown workflow target
   - argument arity mismatch

## TDD Steps

1. Add failing end-to-end tests for registered callable workflow calls.
2. Add failing tests for runtime execution of `Workflow::Call` in big-step and small-step paths.
3. Implement callable-workflow registration/typechecker plumbing.
4. Implement big-step and small-step `Workflow::Call` execution.
5. Add failure-path tests for unresolved targets and argument mismatch.

## Verification Steps

- [ ] `cargo test -p ash-engine workflow_call -- --nocapture`
- [ ] `cargo test -p ash-interp workflow_call -- --nocapture`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo fmt --check`

## Notes

Production-quality here means `Workflow::Call` is no longer a placeholder in any advertised execution path.