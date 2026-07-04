# TASK-1899: Contract Blame Diagnostics

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Emit structured blame diagnostics for contract violations and predicate faults.

## Requirements

1. Include boundary id, callable identity, clause kind, predicate id, source span, and blame label.
2. Include snapshot refs and observed values when policy permits.
3. Include evidence/discharge refs and stale/missing evidence reasons when relevant.
4. Preserve redaction metadata so diagnostics can explain omitted sensitive values.
5. Distinguish admission failures, operation failures, predicate falsehood, and predicate faults.

## TDD Steps

1. RED: add diagnostic snapshot tests for failed `requires`, failed `ensures`, and predicate fault.
2. RED: add observation evidence and redaction tests.
3. GREEN: extend diagnostic payloads and renderers.
4. Verify CLI JSON/text output remains structured and stable.

## Completion Checklist

- [ ] Diagnostics include blame and boundary metadata.
- [ ] Diagnostics include predicate, snapshot, and evidence references where applicable.
- [ ] Observation evidence is policy-governed and redactable.
- [ ] Failure categories are not conflated.
- [ ] CLI diagnostic output is covered by regression tests.
