# TASK-2041: Engine-Only Closeout, Documentation, Traceability, and Gate

**Status:** Complete
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2034 through TASK-2040 and TASK-2042
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Rule coverage:** [Engine-only closeout coverage](../SEMANTIC-RULE-COVERAGE.md#task-2041-engine-only-closeout)

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

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
- Add declared-corpus property tests for normalized-terminal parity over the supported
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
- **Integration/proof responsibility:** owns same-source-contract normalized-terminal comparison
  through `ash run`, daemon, `ash test`, and REPL for every declared supported shared case.

## Semantic workflow record

**Canonical rules:** `CONF-ENGINE-ONLY-CLIENT-001` and `CONF-IMPLEMENTATION-001`.

**Missing target-spec clauses:** The target Core/CPS domains remain partial; TASK-2041 compares only the declared shared source contract across four independent local Engine clients.

**Non-goals:** A shared Engine service, daemon execution for ash run or REPL, source synthesis, deferred-case implementation, Lean execution, or a runtime refinement proof.

**Next obligation:** Later target-rule realization tasks own every remaining partial/below-spec clause.

## Task-owned evidence plan

- **Positive:** `TEST-TASK-2041-FOUR-CLIENT-PARITY` compares the exact
  `TASK-2035-SHARED-ROUTE-001` source bytes through `ash run`, `ash test`, the daemon descriptor
  route, and REPL. `ash test` is exercised only through the public
  `ash_cli::test_runner::executor::run_suite` boundary defined by TASK-2038, with no new CLI or
  snapshot mechanism. `ash run` and REPL complete through their own local Engines before the test
  starts the daemon. Unavailable daemon Unix-socket support fails the evidence target rather than
  turning it into a three-client pass.
- **Negative:** `TEST-TASK-2041-ZERO-USE-GATE` rejects current manifest-listed Rust delete
  entries and relocation of the retired private test route. `TEST-TASK-2041-LEAN-BOUNDARY`
  retains the deferred separate-project and historical-prose controls.
- **Mutation:** `TEST-TASK-2041-DECLARED-CORPUS-PROPERTY` ranges only over the one declared
  shared contract and does not generate source forms.
- **Parity:** `TEST-TASK-2041-FOUR-CLIENT-PARITY` records normalized
  `CanonicalTerminalEnvelopeV1::Returned(Value::Int(42))`; this is tested evidence, not a proof
  or target-spec parity claim.

## TDD and verification steps

1. Add failing zero-use, stale-current-document, stale-traceability, and four-client parity
  controls; retain labeled historical-prose and deferred-Lean-project controls.
2. Convert the TASK-2036 gate from manifest allowlist to zero current/executable entries.
3. Update current docs/read paths and classify historical documents; repair indexes in the same
   change.
4. Run focused route tests, workspace formatting/check/clippy/tests, semantic-task and
   traceability validation, orientation-index self-test, docs gate, and `git diff --check`.

## Completion checklist

- [x] The zero-use gate proves no current/executable Rust direct evaluator or differential oracle;
      preserved Lean material is labeled deferred and has no current executable, conformance, or
      proof evidence/authority and no runtime refinement bridge.
- [x] Documentation states that the four routes share Engine implementation and contracts while each uses a separate local Engine instance.
- [x] Same source contracts have normalized-terminal parity through all four clients.
- [x] Every deferred case remains explicit and below-spec until its target clause is
      implemented.
- [x] CHANGELOG, PLAN-INDEX, coverage, traceability, and orientation indexes are current.
