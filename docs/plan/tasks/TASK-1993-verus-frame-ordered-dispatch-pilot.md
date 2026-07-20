# TASK-1993: Verus Pilot 2 — Frame-Ordered Operation Dispatch

**Status:** Planned conditionally on the TASK-1992 go decision
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1992 and the `λAsh-Effect` lookup-rule freeze

## Description

Verify that production operation dispatch selects the innermost matching handler or provider frame
according to one shared ordering relation.

## Requirements

- Target `HandlerChain::find_operation_frame` and the selection boundary consumed by `eval_raise`.
- Prove absence, bounds/match, greatest matching index, payload provenance, matching-frame
  shadowing, nonmatching-frame preservation, and equal handler/provider ordering.
- Keep provider execution, handler body evaluation, and resume behavior outside the pilot theorem.
- Add a mutation gate that fails when search order becomes outermost-first.
- Pilot one LLM lemma-repair loop whose output is accepted only after Verus checks it.

## TDD Steps

1. Strengthen inner-provider/outer-handler and inner-handler/outer-provider property fixtures.
2. Define the canonical frame-stack view and checked correspondence.
3. Add the lookup/shadowing proofs and deliberate proof-break/repair benchmark.
4. Run Verus, focused Rust, conformance, traceability, and documentation gates.

## Completion Checklist

- [ ] The lookup/shadowing theorem set passes reproducibly.
- [ ] Production dispatch uses the verified algorithm.
- [ ] Assumptions, holes, and provider/tool versions are explicit.
- [ ] LLM output is checker-validated and recorded as hybrid provenance only.
- [ ] If TASK-1992 stops expansion, this task records `conditionally skipped`, retained
  obligations, and a remediation owner rather than claiming proof completion.
