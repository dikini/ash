# TASK-1897: Contract Discharge Integration

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Integrate static, evidence, and dynamic contract discharge with the row admission model.

## Requirements

1. Distinguish static proof discharge, evidence discharge, dynamic runtime-check discharge, and
   explicit recoverable failure/compensation paths.
2. Ensure contract and evidence rows are discharged only by the matching discharge family.
3. Preserve residual row requirements when discharge is missing.
4. Reject attempts to use operation/resource/role/policy admission as contract evidence.
5. Preserve public summaries for imported contract discharge metadata.

## TDD Steps

1. RED: add row admission tests for static, evidence, dynamic, missing, and mismatched discharges.
2. RED: add imported callable discharge tests.
3. GREEN: wire contract/evidence discharge into row admission.
4. Verify existing operation/resource/role/policy row admission behavior is unchanged.

## Completion Checklist

- [ ] Static/evidence/dynamic discharge modes are distinct.
- [ ] Missing contract/evidence discharge fails closed or leaves a visible residual requirement.
- [ ] Non-contract authority cannot discharge contract predicates.
- [ ] Imported callable discharge metadata is preserved.
- [ ] Existing row admission tests remain green.
