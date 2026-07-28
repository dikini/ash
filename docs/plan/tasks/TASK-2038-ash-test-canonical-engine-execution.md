# TASK-2038: `ash test` Canonical Engine Execution

**Status:** Complete
**Semantic task classification:** semantic-runtime-realization
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2034, TASK-2035, TASK-2036, and TASK-2037
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2038 `ash test` canonical Engine execution](../SEMANTIC-RULE-COVERAGE.md#task-2038-ash-test-canonical-engine-execution)

## Description

Migrate authored, property, small-world, and supported synthesized contract test execution to
Engine-issued admitted source programs. Use only the source-wrapper contract from
TASK-2035. Preserve pass/fail results for every manifest-catalogued supported case; emit a stable
explicit deferred result for every catalogue entry without a source wrapper. Delete CoreExpr or
AST postcondition evaluation that exists solely as a test oracle.

## Requirements

- Construct only the TASK-2035 source-backed wrappers and submit admitted Engine requests.
- Preserve each manifest-supported pass/fail observation; emit the specified deferred result for
  every other catalogue case.
- Add catalogued-domain property tests for wrapper identity, terminal parity, and no fallback. Their
  strategies must be constrained to manifest-declared literals/case IDs and may not generate
  source forms or feature slices.
- Activation records implementation, evidence, and parity separately for each named target rule.

## Handoffs

- **Run-route impact:** `active`.
- **Consumes:** source-wrapper contract, Engine-private executor, and TASK-2034 case catalogue.
- **Produces:** Engine-only test execution, source identity/repro linkage, and deferred-case
  result records.
- **Downstream owner:** TASK-2040 deletes residual direct test evaluator material; TASK-2041 owns
  cross-client parity evidence.
- **Does not own:** unlisted test forms, a general source synthesizer, expanded target grammar, or
  a direct evaluator compatibility mode.
- **Integration/proof responsibility:** this task owns focused test-runner vs Engine terminal
  parity; TASK-2041 owns final CLI/daemon/REPL comparison.

## TDD and activation steps

1. Activate rule-scoped semantic records and add failing supported-wrapper, deferred-wrapper,
   mutation, and no-fallback tests.
2. Build source wrappers from exact source identities and literal inputs; submit only
   admitted Engine requests.
3. Replace legacy oracle metadata and update repro output to identify the source wrapper and
   admitted-program identity.
4. Prove tests reject when parsing/checking/lowering/admission fails rather than selecting AST.

## Semantic workflow record

**Canonical rules:** `CONF-SYNTH-SOURCE-WRAPPER-001` and
`CONF-ENGINE-ONLY-CLIENT-001`.

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Only the two exact TASK-2035 source identities are selected. The remaining SPEC-077 synthesized-test domain, unselected client routes, residual direct-evaluator deletion, and TASK-2041's four-client comparison remain incomplete.

**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification
partial.

**Run-route impact:** active.

**Consumes:** `TASK-2035-SYNTH-WRAPPER-001`, `TASK-2035-SHARED-ROUTE-001`,
`AUDIT-204-TEST-EXEC-002`, `AUDIT-204-DEFERRED-001` through `AUDIT-204-DEFERRED-007`, and the
TASK-2037 Engine-private executor boundary.

**Produces:** selected `ash test` Engine submissions, source identity/repro linkage, exact
deferred-case result records, and focused terminal observations for the two selected source
identities.

**Downstream owner:** TASK-2040 removes residual direct test-evaluator material; TASK-2041 owns
the same-source-contract four-client terminal comparison.

**Does not own:** a general source synthesizer, forms absent from the TASK-2035 catalogue, REPL,
daemon, or `ash run` client implementation, target grammar expansion, or a direct-evaluator
compatibility mode.

**Integration/proof responsibility:** TASK-2038 owns focused test-client to Engine terminal
observations. TASK-2041 separately compares the selected shared source contract across all four
clients.

**Next obligation:** Retain the selected Engine route while TASK-2040 removes residual direct
test-evaluator material and TASK-2041 supplies the four-client terminal comparison.

## Task-owned evidence plan

The following controls are focused runtime evidence for this partial route. They do not establish
the remaining target-spec domain or four-client parity.

- `TEST-TASK-2038-SYNTH-WRAPPER-POSITIVE`: the exact wrapper source identity reaches Engine and
  observes `Bool(true)`.
- `TEST-TASK-2038-SHARED-ROUTE-PARITY`: the exact shared source identity observes the normalized
  `Int(42)` terminal envelope through the test client.
- `TEST-TASK-2038-DEFERRED-CATALOGUE`: all seven listed unsupported shapes return their required
  deferred result.
- `TEST-TASK-2038-MUTATION-NO-FALLBACK`: an altered parse-success source shape rejects at the
  Engine admission boundary; the compatibility presentation is an explicit deferred result and
  cannot select an AST, Core, CPS, or differential fallback.
- `TEST-TASK-2038-CATALOGUE-PROPERTY`: an enumerated property ranges only over the two declared
  source IDs and their exact literal bindings, preserving the required terminal or deferred
  observation without fallback.

## Completion checklist

- [x] Every catalogued supported test case reaches the Engine executor.
- [x] Every catalogued unsupported case is an explicit deferred result.
- [x] No test-runner AST/CoreExpr/CPS evaluator remains reachable.
- [x] Focused implementation/evidence/parity records are updated without generalizing the target
      test domain.
