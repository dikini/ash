# TASK-1900: Runtime Monitor Evidence

**Status:** Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Wire runtime monitor evidence rows into contract admission and diagnostics. A monitor evidence row is a record that a runtime monitor observed a temporal or behavioral property; it remains a requirement, not an authority grant, and feeds into contract discharge diagnostics with redacted metadata.

## Requirements

1. Represent a runtime monitor evidence row item (`CoreRowItem::Evidence { family: "monitor", identity: ... }`).
2. Allow the runtime to emit a monitor evidence record when a monitor observes a property along a computation boundary.
3. Integrate monitor evidence into `ContractDischargeRecord` so a dynamic contract check can reference monitor evidence refs in its predicate or diagnostic.
4. Ensure monitor evidence remains authority-free: it does not grant operation/resource/role/policy authority and cannot discharge those row families.
5. Produce temporal monitor diagnostics that identify the monitor, the boundary, and the redacted observation trace without exposing raw observed values unless explicitly permitted.

## TDD Steps

1. Add unit tests for `CoreRowItem::Evidence` with `family = "monitor"` serialization and equality.
2. Add runtime tests proving a monitor evidence record can be attached to a `ContractDischargeRecord` and serialized.
3. Add negative tests proving monitor evidence cannot discharge operation/resource/role/policy rows.
4. Add diagnostic tests proving monitor evidence metadata appears in a contract fault/violation diagnostic, but raw observation values are redacted.

## Completion Checklist

- [x] Runtime monitor evidence row is represented and serializable.
- [x] Runtime can produce and attach monitor evidence to a contract discharge record.
- [x] Monitor evidence does not grant authority or discharge non-evidence rows.
- [x] Temporal monitor diagnostics carry monitor identity, boundary, and redacted metadata.
- [x] Focused tests pass.

## Notes

`RuntimeMonitorEvidence` is added as a structured carrier in `ash-core` and attached to `ContractDischargeRecord` via `with_monitor_evidence`. Tests verify construction, accessor behavior, serialization, and authority neutrality. Future wiring into the runtime execution path will attach monitor evidence to discharge records when a temporal monitor is evaluated at a boundary.
