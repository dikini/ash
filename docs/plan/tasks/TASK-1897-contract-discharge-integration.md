# TASK-1897: Contract Discharge Integration

**Status:** ✅ Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Integrate static, evidence, and dynamic contract discharge with row admission so contract discharge participates in row accounting without letting contract rows grant authority.

## Requirements

1. Map static, evidence, and dynamic `DischargeMode`s onto the row admission path.
2. Ensure contract discharge metadata flows through callable row metadata (NOTE-033 / PLAN-165 carriers).
3. Prevent contract row items from installing providers, selecting resources, admitting roles, or granting operation authority.
4. Make evidence rows dischargeable only when the evidence record is valid and the strategy explicitly allows it.
5. Preserve fail-closed behavior for missing or invalid discharge.

## TDD Steps

1. Add admission tests proving static discharge records do not require runtime checks.
2. Add admission tests proving evidence discharge requires valid evidence records.
3. Add admission tests proving dynamic discharge installs runtime checks.
4. Add authority-neutrality tests proving contract rows do not grant authority.

## Completion Checklist

- [x] Static/evidence/dynamic discharge modes wired to row admission via `ContractDischargeRecord` stored in `RuntimeState` and checked by `RowAdmissionCheck`.
- [x] Contract discharge metadata preserved through callable rows (engine-side `set_contract_discharge_for_callable` / `contract_discharge_record_for_callable` hooks).
- [x] Contract rows blocked from authority acquisition (`CoreRowItem::Contract` mapped to `RowAdmissionRequirement::Unsupported` fail-closed).
- [x] Evidence discharge validated fail-closed (no record == missing requirement).
- [x] Focused admission and authority-neutrality tests pass in `crates/ash-engine/tests/task_1896_1897_evidence_contract_discharge.rs`.
