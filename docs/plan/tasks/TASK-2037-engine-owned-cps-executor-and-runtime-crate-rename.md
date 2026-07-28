# TASK-2037: Engine-Owned CPS Executor and Runtime-Crate Rename

**Status:** Planned
**Semantic task classification:** semantic-runtime-realization
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2035 and TASK-2036

## Description

Move checked-CPS validation and evaluation behind an Engine-private executor boundary. `Engine`
is the sole owner permitted to execute admitted checked CPS. Remove public evaluator exports and
ensure CLI, daemon, test, and REPL can submit only Engine requests/results. Rename the residual
support crate from `ash-interp` to `ash-runtime` only when it exposes no program evaluator; any
temporary migration boundary is private, manifest-listed, and deleted by TASK-2040.

## Requirements

- Preserve the checked Core → checked CPS → admission → terminal contract; moving code does not
  widen accepted CPS or add a second evaluator.
- Expose no public non-Engine CPS execution API after migration.
- Add finite-domain property tests for admitted CPS terminalization and rejection. Their strategies
  may range only over manifest-declared inputs/artifacts; they must not generate source forms,
  features, or implementation slices.
- Activation must add rule-scoped coverage/traceability and independently report implementation,
  evidence, and parity.

## Handoffs

- **Run-route impact:** `prerequisite`.
- **Consumes:** checked Core/CPS/admission contracts and TASK-2035's client contract.
- **Produces:** Engine-private executor API, non-evaluator runtime support boundary, and caller
  migration guide for TASK-2038/2039/2040/2042.
- **Downstream owner:** TASK-2038/2039/2042 use Engine submission only; TASK-2040 deletes remaining
  direct evaluator material; TASK-2041 proves API absence.
- **Does not own:** expansion of target Core/CPS semantics, new lowering, client UX, or deletion
  of the AST evaluator itself.
- **Integration/proof responsibility:** TASK-2041 owns end-state route parity and API absence;
  this task owns focused Engine admission/executor tests.

## TDD and activation steps

1. Promote the task with rule-scoped semantic record, coverage, traceability, and evidence IDs.
2. Add compile/API tests that external crates cannot invoke checked-CPS evaluation and focused
   Engine tests for admitted success, malformed CPS rejection, trap, timeout, and cancellation.
3. Move the executor, update dependencies and crate metadata, and preserve terminal projection.
4. Run focused Engine/client tests, formatter, clippy, semantic gates, and crate-rename checks.

## Completion checklist

- [ ] No public non-Engine checked-CPS executor remains.
- [ ] Engine validates and executes only admitted checked CPS.
- [ ] `ash-runtime` has no public program evaluator; its remaining support APIs are documented.
- [ ] Implementation/evidence/parity report only the exact named target rules.
