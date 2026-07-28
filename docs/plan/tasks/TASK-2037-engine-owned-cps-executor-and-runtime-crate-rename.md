# TASK-2037: Engine-Owned CPS Executor and Runtime-Crate Rename

**Status:** Complete
**Semantic task classification:** semantic-runtime-realization
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2035 and TASK-2036
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Rule coverage:** [TASK-2037 Engine-owned CPS executor boundary](../SEMANTIC-RULE-COVERAGE.md#task-2037-engine-owned-cps-executor-boundary)

## Description

Move checked-CPS validation and evaluation behind an Engine-private executor boundary. `Engine`
is the sole owner permitted to execute admitted checked CPS. Remove public evaluator exports and
ensure CLI, daemon, test, and REPL can submit only Engine requests/results. Retain `ash-interp`
as the explicitly documented residual support crate while TASK-2040-owned direct-AST material
remains; TASK-2040 owns the later `ash-runtime` rename. Retained AUDIT-204 differential tests may
move into Engine-private test modules only to remove their public execution API; the frozen audit
and TASK-2040 deletion ownership do not change.

## Requirements

- Preserve the checked Core → checked CPS → admission → terminal contract; moving code does not
  widen accepted CPS or add a second evaluator.
- Expose no public non-Engine CPS execution API after migration.
- Expose no public differential execution API while its TASK-2040-owned retained corpus remains.
- Add enumerated-domain property tests for admitted CPS terminalization and rejection. Their strategies
  may range only over manifest-declared inputs/artifacts; they must not generate source forms,
  features, or implementation slices.
- Activation must add rule-scoped coverage/traceability and independently report implementation,
  evidence, and parity.

## Handoffs

- **Run-route impact:** `prerequisite`.
- **Consumes:** checked Core/CPS/admission contracts and TASK-2035's client contract.
- **Produces:** Engine-private executor and differential test boundaries, documented residual
  `ash-interp` support boundary, and caller migration guide for TASK-2038/2039/2040/2042.
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

## Semantic workflow record

**Canonical rules:** `SEM-TARGET-CORE-CPS-001`, `SEM-EFFECT-ADMISSION-001`,
`OBS-TARGET-PROJECTION-001`, `CONF-ENGINE-ONLY-CLIENT-001`, `SEM-CPS-TRAP-001`,
`SEM-EFFECT-TIMEOUT-001`, and `SEM-EFFECT-CANCEL-001`.

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Selected client routes, full target Core/CPS domains, deletion of direct-AST and differential material, and TASK-2041's four-client terminal comparison remain incomplete.

**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime partial;
verification partial.

**Run-route impact:** prerequisite.

**Consumes:** TASK-2035's Engine-only client contract; `AUDIT-204-CPS-001` through
`AUDIT-204-CPS-008`; checked Core/CPS artifacts; and Engine admission provenance.

**Produces:** the Engine-private checked-CPS executor boundary, migrated private CPS regression
coverage, and private Engine test placement for retained AUDIT-204 differential material. That
placement removes public invocation only; TASK-2040 retains the frozen-audit deletion ownership.
It does not activate a client route or rename the residual support crate.

**Downstream owner:** TASK-2038, TASK-2039, TASK-2040, and TASK-2042 consume this boundary;
TASK-2041 owns integration proof and API-absence closeout.

**Does not own:** Test-runner, REPL, daemon, or ash run client-route implementation. Deletion of direct-AST evaluation, the Rust differential stack, or Lean material. Renaming ash-interp while TASK-2040-owned AST material remains. Transferring TASK-2040 deletion ownership when retained audit-listed differential tests move into Engine-private test modules.

**Integration/proof responsibility:** This task owns focused executor-boundary controls only.
TASK-2041 owns the same-source-contract four-client normalized-terminal comparison and no-public-
API closeout.

**Next obligation:** TASK-2038, TASK-2039, TASK-2042, and TASK-2040 must consume the Engine-private executor boundary; TASK-2041 must prove API absence and four-client normalized-terminal parity.

## Task-owned evidence plan

The focused controls are green: positive admitted `42` terminalization
(`TEST-TASK-2037-ENGINE-OWNED-CPS-POSITIVE`), negative external-CPS boundary
(`TEST-TASK-2037-ENGINE-OWNED-CPS-NEGATIVE`), and mutation forged-artifact rejection
(`TEST-TASK-2037-ENGINE-OWNED-CPS-MUTATION`). Admitted trap
(`TEST-TASK-2037-ENGINE-OWNED-CPS-TRAP`), zero-timeout
(`TEST-TASK-2037-ENGINE-OWNED-CPS-TIMEOUT`), and pre-cancellation
(`TEST-TASK-2037-ENGINE-OWNED-CPS-CANCELLATION`) each project a canonical terminal envelope.
These controls establish a prerequisite boundary only; client and reference-executor parity are
not applicable here.

## Completion checklist

- [x] No public non-Engine checked-CPS executor remains.
- [x] Engine validates and executes only admitted checked CPS.
- [x] `ash-interp` remains a documented residual support crate while TASK-2040-owned AST material
      remains; TASK-2040 owns the later `ash-runtime` rename.
- [x] Implementation/evidence/parity report only the exact named target rules.
