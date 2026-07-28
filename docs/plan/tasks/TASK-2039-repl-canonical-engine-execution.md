# TASK-2039: REPL Canonical Engine Execution

**Status:** Complete
**Semantic task classification:** semantic-runtime-realization
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2035, TASK-2036, and TASK-2037
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2039 REPL canonical Engine execution](../SEMANTIC-RULE-COVERAGE.md#task-2039-repl-canonical-engine-execution)

## Description

Make `ash repl` an Engine client for normal expression/module evaluation and stored-entry
execution. Each evaluable REPL submission becomes a source-derived admitted request under
SPEC-011; prompt/history/multiline handling remain client concerns. `:help`, `:type`, and `:ast`
remain inspection commands and may not execute through an alternate evaluator. Unsupported
session shapes reject/defer according to TASK-2035 rather than falling back to AST evaluation.

## Requirements

- Normal evaluation and stored-entry execution submit source-derived admitted requests to Engine.
- Prompt/history/multiline handling and inspection commands remain client behavior and cannot
  expose an alternate evaluator.
- Add declared-corpus property tests for focused same-source-contract REPL–Engine
  normalized-terminal comparison and rejection; they may range only over the declared supported
  corpus, not generated source forms.
- Activation records implementation, evidence, and parity separately for the named REPL/runtime
  rules.

## Handoffs

- **Run-route impact:** `active`.
- **Consumes:** SPEC-011 amendment and Engine-private executor boundary.
- **Produces:** Engine-submitted REPL request route and normalized terminal rendering contract.
- **Downstream owner:** TASK-2040 deletes REPL direct evaluator calls; TASK-2041 owns four-client
  parity and final documentation.
- **Does not own:** a new REPL language, persistent evaluation beyond specified session state, or
  expansion of the source-wrapper domain.
- **Integration/proof responsibility:** this task owns focused REPL/Engine parity; TASK-2041 owns
  final same-source-contract client parity.

## TDD and activation steps

1. Activate semantic records and add failing Engine-request, normal-result, admission-rejection,
   multiline, and inspection-command no-evaluation tests.
2. Route evaluable input through parse/check/lower/admit/Engine; retain canonical terminal error
   categories and history behavior.
3. Add a focused normalized-terminal comparison for the same source contract through REPL and
   Engine.
4. Run focused CLI/Engine tests plus semantic/documentation gates.

## Semantic workflow record

**Canonical rules:** `OBS-REPL-ENGINE-CLIENT-001` and
`CONF-ENGINE-ONLY-CLIENT-001`.

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Only the two exact TASK-2035 REPL source identities are selected. Stored-session shapes beyond the selected controls, remaining SPEC-011 submission forms, residual direct-evaluator deletion, daemon and ash run transport, and TASK-2041's four-client comparison remain incomplete.

**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification
partial.

**Run-route impact:** active.

**Consumes:** `TASK-2035-REPL-ROUTE-001`, `TASK-2035-REPL-ROUTE-002`,
`TASK-2035-SHARED-ROUTE-001`, `AUDIT-204-REPL-001`, `AUDIT-204-REPL-002`, and the TASK-2037
Engine-private executor boundary.

**Produces:** source-derived admitted REPL requests, normalized terminal rendering for the selected
controls, and focused REPL-to-Engine terminal observations.

**Downstream owner:** TASK-2040 deletes residual REPL direct-evaluator calls; TASK-2041 owns the
same-source-contract four-client terminal comparison.

**Does not own:** A new REPL language, persistent evaluation beyond the specified session state, target grammar expansion, daemon or ash run transport, or a direct-evaluator compatibility mode.

**Does not own:** TASK-2041's four-client same-source-contract terminal comparison.

**Integration/proof responsibility:** TASK-2039 owns focused REPL-to-Engine observations for the
selected controls. TASK-2041 separately compares the shared source contract across all four
clients.

**Next obligation:** Retain the selected Engine route while TASK-2040 removes residual REPL
direct-evaluator calls and TASK-2041 supplies the four-client terminal comparison.

## Task-owned evidence plan

The following controls are focused runtime evidence for this partial route. They do not establish
the remaining target-spec domain or four-client parity.

- `TEST-TASK-2039-REPL-ENGINE-POSITIVE` (**Positive**): each selected normal source submission reaches the
  admitted Engine request path and observes its required value.
- `TEST-TASK-2039-REPL-ADMISSION-REJECTION` (**Negative**): an unannotated full source without a
  checked admission artifact rejects without a local fallback.
- `TEST-TASK-2039-REPL-MULTILINE` (**Positive**): the selected multiline source remains a client input concern
  and reaches its local admitted Engine route after completion.
- `TEST-TASK-2039-REPL-INSPECTION-NO-EVALUATION` (**Negative**): `:help`, `:type`, and `:ast` retain inspection
  behavior without selecting an execution route.
- `TEST-TASK-2039-REPL-DECLARED-CORPUS-PROPERTY` (**Mutation**): the declared source corpus preserves its exact
  value and Engine terminal observation without generating a new source form.
- `TEST-TASK-2039-REPL-SHARED-ROUTE-PARITY` (**Parity**): the shared `Int(42)` source observes the same
  normalized terminal envelope through REPL and Engine. This is not TASK-2041's four-client
  evidence.

## Completion checklist

- [x] REPL has no AST/CPS execution call outside Engine.
- [x] Normal evaluable input and stored entries use admitted Engine requests.
- [x] Inspection commands do not create an alternate execution route.
- [x] Focused evidence reports implementation/evidence/parity independently.
