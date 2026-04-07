# TASK-427: SPEC-025 Faithful Closeout and Corpus Alignment

## Status: 📝 Planned

## Description

Apply the faithful rewrite/audit results to `SPEC-025` and align the surrounding planning/reporting corpus so `SPEC-025` becomes the stable docs/spec home for the accepted small-step semantic contract. This task packages the final closeout after TASK-424 through TASK-426 have frozen the faithfulness contract and compatibility audit.

This remains docs/spec work only.

## Specification Reference

- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)

## Dependencies

- ✅ [TASK-424: SPEC-025 Faithfulness and Compatibility Contract](TASK-424-spec-025-faithfulness-and-compatibility-contract.md)
- ✅ [TASK-425: SPEC-025 Rule-Schema and Helper-Boundary Consolidation](TASK-425-spec-025-rule-schema-and-helper-boundary-consolidation.md)
- 📝 [TASK-426: SPEC-025 Big-Step and Runtime Compatibility Audit](TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md)

## Requirements

### Functional Requirements

1. Revise `SPEC-025` so it reflects the approved faithful wording from TASK-424 through TASK-426.
2. Keep `SPEC-025` explicitly compatible with:
   - accepted `MCE-005`,
   - accepted `SPEC-004`,
   - current implementation evidence from `MCE-006`.
3. Update nearby planning/reporting/index surfaces so they reference `SPEC-025` as the docs/spec home for the accepted small-step contract where appropriate.
4. Add a concise `CHANGELOG.md` entry for the faithful closeout pass.
5. Keep all wording honest about what remains partial in the implementation.

### Non-Functional Requirements

1. Keep scope limited to docs/spec/reporting alignment.
2. Use repo-relative links throughout.
3. Do not reopen Phase 61 or Phase 63 decisions.
4. Mark complete only if the surrounding corpus reads coherently after the rewrite.

## TDD Evidence

### Red

Before this task:
- the rewrite/audit decisions may exist, but the final corpus still needs one closeout pass.
- nearby planning/reporting surfaces may still treat SPEC-025 as only an initial distillation.

### Green

This task is complete when:
- SPEC-025 reflects the faithful contract and audit results,
- nearby corpus surfaces are aligned,
- changelog/reporting surfaces record the promotion clearly.

## Files

- Create: `docs/plan/tasks/TASK-427-spec-025-faithful-closeout-and-corpus-alignment.md`
- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [ ] TASK-427 task file created
- [ ] SPEC-025 revised with faithful audited wording
- [ ] nearby planning/reporting surfaces aligned
- [ ] CHANGELOG updated
- [ ] corpus reads coherently after rewrite

## Notes

Important constraints:
- Do not silently widen semantic scope.
- Do not silently strengthen runtime claims beyond MCE-006 evidence.
- Keep SPEC-025 as the docs/spec surface, with MCE-005 and MCE-006 remaining the accepted design/evidence backplanes.
