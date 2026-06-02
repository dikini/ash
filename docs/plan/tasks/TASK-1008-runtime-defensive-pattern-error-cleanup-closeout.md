# TASK-1008: Runtime defensive pattern error boundary and closeout

## Status: 📝 Planned

## Description

Verify runtime pattern-match failure remains defensive only, reconcile status surfaces, run broad gates, and close SPEC-076 acceptance evidence.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- 📝 TASK-1001 audit gate must complete and patch this task before implementation starts
- 📝 TASK-1002 through TASK-1007 must be complete before closeout evidence is recorded

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add or update interpreter tests showing unchecked/host-created mismatches still produce structured runtime errors, using exact live variants such as expression `LetPatternBindFailed`, workflow `PatternMatchFailed`, and `NonExhaustiveMatch` or their TASK-1001-refreshed names.
3. Add integration tests proving checked source rejects binder failures before runtime.
4. Update SPEC-076 acceptance matrix with evidence.
5. Run broad gates and independent review before promoting status.

## File Targets

- Modify: exact files to be patched by TASK-1001 audit
- Test: exact focused test target to be patched by TASK-1001 audit

## TDD / Execution Steps

1. Stop if this file still contains the fail-closed TASK-1001 verification guard.
2. Write RED tests named by TASK-1001.
3. Implement the smallest semantic change for this task only.
4. Run focused tests and required workspace checks.
5. Request independent review before marking complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - false # TASK-1001 must replace this guard with exact focused non-zero commands before TASK-1008 implementation starts
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - bash -lc "cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-126-doc.log && ! grep -i '^warning:' /tmp/ash-plan-126-doc.log"
checklist:
  - [ ] TASK-1001 replaced the fail-closed guard
  - [ ] RED tests fail before implementation and pass after implementation
  - [ ] Scope did not expand beyond SPEC-076
  - [ ] Diagnostics are asserted where required
  - [ ] Broad closeout gates pass on the final diff
  - [ ] SPEC-076/PLAN-126/PLAN-INDEX/CHANGELOG status and evidence are reconciled
```

## Dependencies for Next Task

This is the closeout task for [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later phases may cite this task's evidence but must not silently expand its scope.

## Notes

Runtime/closeout
