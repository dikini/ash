# TASK-1896: Evidence Row Substrate

**Status:** ✅ Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Add evidence row records for tests, laws, proofs, runtime monitors, and observation evidence so they remain requirements and records without granting authority.

## Requirements

1. Extend the row language with evidence row items that name an evidence family and an identity.
2. Define evidence records for: `test`, `law`, `proof`, `monitor`, and `observation`.
3. Keep evidence rows as requirements/records: they can require or record evidence but cannot prove authority by being mentioned.
4. Provide stable evidence identities across module boundaries.
5. Reject invalid or stale evidence forms fail-closed.

## TDD Steps

1. Add schema/unit tests for evidence row records and identities.
2. Add tests proving `by test`, law, proof, runtime monitor, and observation rows remain requirements.
3. Add tests proving invalid or stale evidence fails closed without converting to authority.
4. Add tests proving statistical/test evidence remains advisory unless the contract strategy explicitly permits dynamic check or evidence discharge.

## Completion Checklist

- [x] Evidence row item schema and record carriers defined (Phase 165 carriers reused; `CoreRowItem::Evidence` path encodes family + identity).
- [x] Evidence families (`test`, `law`, `proof`, `monitor`, `observation`) implemented in `RowAdmissionRequirement::Evidence`.
- [x] Evidence rows treated as non-authority-granting requirements in row admission.
- [x] Stable evidence identities across module boundaries (Core row path preserved).
- [x] Invalid/stale evidence rejected fail-closed.
- [x] Focused schema and admission tests pass in `crates/ash-engine/tests/task_1896_1897_evidence_contract_discharge.rs`.
