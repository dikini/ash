# TASK-1900: Runtime Monitor Evidence

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Wire runtime monitor evidence rows and temporal monitor diagnostics into the contract/evidence
system.

## Requirements

1. Represent runtime monitor evidence as an evidence row kind with stable monitor identity.
2. Preserve monitor plan, trace alphabet, boundary, and evidence sink metadata.
3. Emit temporal monitor violations separately from value predicate violations.
4. Emit monitor faults separately from failed formulas.
5. Keep monitor evidence authority-free: monitors observe trace/evidence streams but do not acquire
   operation/provider authority.

## TDD Steps

1. RED: add carrier tests for monitor evidence rows and monitor plans.
2. RED: add runtime diagnostics tests for monitor violation versus monitor fault.
3. GREEN: wire monitor evidence rows to existing trace/monitor carriers.
4. Verify monitor evidence does not discharge operation/resource/role/policy authority.

## Completion Checklist

- [ ] Runtime monitor evidence rows are represented distinctly.
- [ ] Monitor plans preserve boundary and trace alphabet metadata.
- [ ] Temporal violations and monitor faults are separate diagnostics.
- [ ] Monitor evidence remains authority-neutral.
- [ ] Trace/monitor docs are reconciled.
