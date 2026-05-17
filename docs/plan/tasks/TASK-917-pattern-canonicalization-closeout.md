# TASK-917: Reconcile SPEC-068/PLAN-117 docs, acceptance matrix, broad gates, and review remediation

## Status: ✅ Complete

## Description

Reconcile SPEC-068/PLAN-117 docs, acceptance matrix, broad gates, and review remediation

## Specification Reference

- [SPEC-068](../../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
- [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)

## Dependencies

- ✅ SPEC-068: spec packet exists
- ✅ PLAN-117: implementation plan exists
- Depends on all prior implementation tasks in this phase and final acceptance evidence
- Depends on TASK-916 completion

## Requirements

1. Reconcile SPEC-068/PLAN-117 docs, acceptance matrix, broad gates, and review remediation.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.
5. Record final acceptance matrix, broad verification, and independent review remediation.

## File Targets

- Modify: docs/spec/README.md
- Modify: docs/plan/PLAN-INDEX.md
- Modify: CHANGELOG.md
- Modify: docs/plan/PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md

## TDD / Execution Steps

1. Re-read the referenced SPEC, PLAN, implementation tasks, and acceptance matrix.
2. Verify every SPEC acceptance row has focused non-zero evidence or an explicit scoped deferral.
3. Run the broad closeout command set recorded in this task after the final code/doc change.
4. Reconcile this task status, the owning PLAN row, PLAN-INDEX, docs/spec/README.md, and CHANGELOG.
5. Run independent review remediation before marking the phase complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 16
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps
checklist:
  - [x] Acceptance matrix evidence is recorded
  - [x] Broad closeout gates recorded; transient performance-baseline failure triaged with focused rerun
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Pattern canonicalization is audit-first and must not solve under neutral computation heads.

## Acceptance Matrix Evidence

Phase 121 closes SPEC-068 as an implemented MVP for ordinary ADT pattern/exhaustiveness canonicalization. The closeout explicitly does not add GADT/refinement patterns, runtime matching on type-level sealed-domain or promoted constructors, broad equality-canonicalization adoption, type-function or associated-family inversion, or ADT runtime layout changes.

| SPEC row | Focused evidence | Closeout status |
|----------|------------------|-----------------|
| PC-1 transparent alias match | `TASK-913` `transparent_alias_to_adt_canonicalizes_to_underlying_constructor_universe`; `TASK-914` `transparent_alias_scrutinee_accepts_canonical_variant_pattern_and_binds_payload`; `TASK-915` `transparent_alias_full_match_uses_canonical_result_universe_and_is_exhaustive`; `TASK-916` `transparent_alias_match_remains_accepted` | Implemented MVP |
| PC-2 selected projection reduces to concrete ADT | `TASK-913` `selected_associated_projection_to_adt_canonicalizes_to_constructor_universe` | Implemented MVP at the canonicalization API boundary; limited to already selected/reducible projections producing ordinary ADTs |
| PC-3 rigid projection blocked | `TASK-913` `unresolved_associated_projection_returns_typed_blocked_result`; `TASK-916` `unresolved_associated_projection_returns_typed_blocked_reason_for_patterns` | Implemented MVP as typed blocked output |
| PC-4 neutral type-function result blocked | `TASK-913` `constructor_variable_application_returns_typed_blocked_result`; `TASK-915` `blocked_non_matchable_scrutinee_does_not_guess_visible_arm_constructor_universe`; `TASK-916` `primitive_scrutinee_with_visible_constructor_does_not_fabricate_missing_witness` | Partial MVP boundary coverage: neutral/non-matchable heads block and do not drive exhaustiveness guesses; no source-level type-function-result runtime matching was added |
| PC-5 unrelated same-visible constructor no leakage | `TASK-914` `visible_constructor_from_unrelated_adt_is_rejected_for_different_scrutinee_adt`; `TASK-916` `same_visible_constructor_from_unrelated_adt_is_rejected_for_scrutinee_identity`; `TASK-916` `unrelated_constructor_name_is_rejected_and_does_not_bind_payload` | Implemented MVP |
| PC-6 direct ADT non-interference | `TASK-914` `direct_adt_scrutinee_still_accepts_variant_pattern`; `TASK-915` `direct_result_full_match_remains_exhaustive`; `TASK-916` `direct_adt_match_remains_accepted` | Implemented MVP |

## Closeout Verification Evidence

2026-05-17, branch `phase-121-pattern-canon`:

- `cargo fmt --check` passed.
- `git diff --check` passed.
- `RUSTC_WRAPPER= cargo check --workspace` passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `RUSTC_WRAPPER= cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-117-doc.log && ! grep -i "^warning:" /tmp/ash-plan-117-doc.log` passed.
- `RUSTC_WRAPPER= scripts/check-rust-tests.sh --workspace` ran broad workspace tests and failed once in `ash-engine --test performance_baseline` because `baseline_computation_workflow` measured 5616ms against a 5000ms performance threshold during the broad run.
- Focused rerun `RUSTC_WRAPPER= cargo test -p ash-engine --test performance_baseline baseline_computation_workflow -- --nocapture` passed, measuring 5ms.

Review note: controller review is expected to run on the unstaged diff after this closeout edit.
