# TASK-2041: Engine-Only Closeout, Documentation, Traceability, and Gate

**Status:** Planned
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2034 through TASK-2040 and TASK-2042

## Description

Close the Engine-only cutover against the frozen audit manifest. Convert the transition guard to
a zero-current-use gate, remove stale current terminology from specifications, plans, tasks,
references, CLI help, and traceability, and classify retained history explicitly. Update the
runnability matrix and task/coverage/traceability records only for completed Phase-205 evidence.
No record may claim a proof or target-spec parity not established by a verified artifact or tests.

## Requirements

- Convert only manifest-listed transition allowances to zero current/executable Rust evaluator use.
- Preserve Lean sources/docs with their deferred separate-project handoff and reject claims that
  they are a current Ash executable, conformance, or proof evidence/authority route. Retained
  Lean material has no runtime refinement bridge; a later separate project must establish one.
- Add finite-domain property tests for normalized-terminal parity over the declared supported
  shared corpus; do not generate source forms, features, or slices.
- Publish implementation/evidence/parity only from the task-owned focused and integration results.

## Handoffs

- **Run-route impact:** `active`.
- **Consumes:** all Phase-204/205 handoffs, final manifest, and focused client evidence.
- **Produces:** zero-use gate, current documentation, historical routing, and final runnability
  matrix/evidence record.
- **Downstream owner:** later target-rule realization tasks own any remaining `partial`/
  `below_spec` feature gaps.
- **Does not own:** new semantics, deferred-case implementation, or proof beyond its exact bridge.
- **Integration/proof responsibility:** owns same-admitted-program normalized-terminal comparison
  through `ash run`, daemon, `ash test`, and REPL for every finite supported shared case.

## TDD and verification steps

1. Add failing zero-use, stale-current-document, stale-traceability, and four-client parity
  controls; retain labeled historical-prose and deferred-Lean-project controls.
2. Convert the TASK-2036 gate from manifest allowlist to zero current/executable entries.
3. Update current docs/read paths and classify historical documents; repair indexes in the same
   change.
4. Run focused route tests, workspace formatting/check/clippy/tests, semantic-task and
   traceability validation, orientation-index self-test, docs gate, and `git diff --check`.

## Completion checklist

- [ ] The zero-use gate proves no current/executable Rust direct evaluator or differential oracle;
      preserved Lean material is labeled deferred and has no current executable, conformance, or
      proof evidence/authority and no runtime refinement bridge.
- [ ] Documentation states one canonical Engine executor for run, daemon, test, and REPL.
- [ ] Same admitted supported programs have normalized-terminal parity through all four clients.
- [ ] Every deferred case remains finite, explicit, and below-spec until its target clause is
      implemented.
- [ ] CHANGELOG, PLAN-INDEX, coverage, traceability, and orientation indexes are current.
