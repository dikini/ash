# TASK-1892: Contract Evidence Seam Audit

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Audit the live implementation seams before adding target-surface contract and evidence behavior.

## Requirements

1. Map existing Core predicate sidecars, runtime check plans, contract diagnostics, and evidence
   records from PLAN-165.
2. Map parser, typechecker, engine, Core, CPS, and CLI paths for ordinary target `fn` sources.
3. Identify where row admission currently handles operation/resource/role/policy/evidence/failure
   requirements.
4. Record current gaps and exact files for subsequent tasks.

## TDD Steps

1. Write an audit artifact under `docs/plan/audits/` with file ownership and implementation risks.
2. Add RED probes or ignored fixtures only where they clarify current failure modes.
3. Confirm the audit names the first task that will turn each gap into tests.

## Completion Checklist

- [ ] Audit artifact created.
- [ ] Contract/evidence carriers mapped.
- [ ] Row admission boundaries mapped.
- [ ] Parser/typechecker/Core/runtime gaps assigned to later tasks.
