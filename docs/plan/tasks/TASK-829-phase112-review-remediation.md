# TASK-829: Phase 112 Review Remediation

## Status: 📝 Planned

## Description

Reserve a post-closeout remediation slice for independent review findings before Phase 112 is considered ready for downstream SPEC-E work.

## Specification Reference

- [SPEC-060](../../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [PLAN-108](../PLAN-108-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
- [DESIGN-034](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md) §16.4
- [SPEC-058](../../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [SPEC-059](../../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

## Dependencies

- ✅ [TASK-816](TASK-816-spec-d-spec-plan-packet.md)
- 📝 [TASK-828](TASK-828-spec-d-closeout-docs-and-verification.md) (planned predecessor)

## Dispatch

```
agent: hermes
provider: openai-codex
model: gpt-5.5
profile: default
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Objective

Reserve a post-closeout remediation slice for independent review findings before Phase 112 is considered ready for downstream SPEC-E work.

## Requirements

1. Run independent review of SPEC-060 implementation, tests, diagnostics, and status surfaces.
2. Fix any blocker/high findings in code or docs.
3. Re-run focused and broad verification affected by review findings.
4. Update TASK-829 with exact review findings and remediation evidence.
5. Only mark Phase 112 complete when review findings are closed.

## Files

- Modify files identified by independent review findings
- Modify: `docs/plan/tasks/TASK-829-phase112-review-remediation.md` with evidence

## TDD Steps

1. Write the audit/docs first; no Rust files change in this task.
2. Verify every claim against live files.
3. Re-read for scope creep before marking complete.

## Verification

```
strictness: clean
commands:
  - cargo test --all
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
  - cargo doc --workspace --no-deps
checklist:
  - [ ] Independent review completed
  - [ ] All blocker/high findings fixed or honestly deferred
  - [ ] Focused and broad gates rerun after fixes
  - [ ] Phase 112 statuses reconciled
```

## Notes

Task type: Review/Hardening. Estimated effort: 6 hours. Keep the slice compilable and do not widen beyond SPEC-060 scope.
