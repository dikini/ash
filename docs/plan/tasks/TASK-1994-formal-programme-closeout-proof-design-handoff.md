# TASK-1994: Formal Programme Closeout and Proof-Design Handoff

**Status:** Planned
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1987 through TASK-1992; TASK-1993 only when authorized by the Pilot 1 go decision

## Description

Audit PLAN-202 completion requirement by requirement and package the measured evidence needed for a
separate Ash-native `spec`/`proof` design programme.

## Requirements

- Prove the canonical corpus, archive routing, deprecation packet, calculus, traceability, and Verus
  pilot sequence meet PLAN-202 acceptance gates, including a valid evidence-backed no-go/conditional
  skip outcome.
- Record unresolved gaps without relabelling them complete.
- Produce an Ash proof-design handoff covering predicate fragments, provider routing, proof scripts,
  termination/erasure, holes/trust, diagnostics, and LLM synthesis evidence.
- Do not implement Ash proof syntax in this task.

## TDD Steps

1. Build the completion matrix from PLAN-202 requirements and evidence sources.
2. Run the complete documentation, conformance, Rust, Verus, and traceability gates.
3. Remediate or explicitly defer every failed requirement.
4. Publish closeout and proof-design handoff only when the evidence matrix is complete.

## Completion Checklist

- [ ] Every PLAN-202 completion claim has direct evidence.
- [ ] The toolchain and both defined pilots have explicit go/no-go or conditional-skip outcomes and
  TCB reports.
- [ ] Remaining gaps are visible and owned.
- [ ] The next proof-design programme has an evidence-backed entry contract.
