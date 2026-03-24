# TASK-219: Align Runtime Role Approval Contract

## Status: ✅ Complete

## Description

Align runtime role handling with the simplified canonical contract: runtime policy and approval
paths use flat named roles, without hierarchy or inherited authority semantics.

## Specification Reference

- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Requirements

### Functional Requirements

1. Keep runtime approval decisions explicitly role-directed
2. Ensure runtime tests do not imply supervisor lookup or inherited approval semantics
3. Tighten any role-related runtime documentation/comments to match the flat role contract
4. Preserve existing approval-role behavior where it already matches the simplified model

## Files

- Modify: `crates/ash-interp/src/capability_policy.rs`
- Modify: `crates/ash-interp/src/capability_policy_runtime.rs`
- Modify: `crates/ash-interp/src/error.rs`
- Test: `crates/ash-interp/tests/policy_runtime_outcomes.rs`
- Modify: role/approval-related runtime tests as needed
- Modify: `CHANGELOG.md`

## TDD Steps

1. ✅ Add failing runtime tests that lock the direct named-role approval contract
2. ℹ️ Separate RED was not preserved because the runtime path already matched the flat-role contract; this task locked that existing behavior with focused regression coverage instead.
3. ✅ Make the minimal runtime/test updates
4. ✅ Verify GREEN with focused and broader `ash-interp` tests
5. ☐ Commit

## Completion Checklist

- [x] runtime approval-role tests lock the direct named-role contract
- [x] runtime role comments/documentation describe a flat named-role carrier
- [x] approval outcomes still preserve the explicit named role in runtime errors
- [x] focused `ash-interp` approval-role tests passed
- [x] broader `ash-interp` verification passed
- [x] `CHANGELOG.md` updated

## Non-goals

- No richer runtime role object unless required by tests/spec
- No workflow/process supervision changes
