# TASK-2038: `ash test` Canonical Engine Execution

**Status:** Planned
**Semantic task classification:** semantic-runtime-realization
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2034, TASK-2035, TASK-2036, and TASK-2037

## Description

Migrate authored, property, small-world, and supported synthesized contract test execution to
Engine-issued admitted source programs. Use only the finite source-wrapper contract from
TASK-2035. Preserve pass/fail results for every manifest-catalogued supported case; emit a stable
explicit deferred result for every catalogue entry without a source wrapper. Delete CoreExpr or
AST postcondition evaluation that exists solely as a test oracle.

## Requirements

- Construct only the TASK-2035 source-backed finite wrappers and submit admitted Engine requests.
- Preserve each manifest-supported pass/fail observation; emit the specified deferred result for
  every other finite catalogue case.
- Add finite-domain property tests for wrapper identity, terminal parity, and no fallback. Their
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
2. Build source wrappers from exact source identities and finite literal inputs; submit only
   admitted Engine requests.
3. Replace legacy oracle metadata and update repro output to identify the source wrapper and
   admitted-program identity.
4. Prove tests reject when parsing/checking/lowering/admission fails rather than selecting AST.

## Completion checklist

- [ ] Every catalogued supported test case reaches the Engine executor.
- [ ] Every catalogued unsupported case is an explicit finite deferred result.
- [ ] No test-runner AST/CoreExpr/CPS evaluator remains reachable.
- [ ] Focused implementation/evidence/parity records are updated without generalizing the target
      test domain.
