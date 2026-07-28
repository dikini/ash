# TASK-2039: REPL Canonical Engine Execution

**Status:** Planned
**Semantic task classification:** semantic-runtime-realization
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2035, TASK-2036, and TASK-2037

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
- Add finite-domain property tests for same-admitted-program REPL/`ash run` terminal parity and
  rejection; they may range only over the declared supported corpus, not generated source forms.
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
  final same-admitted-program client parity.

## TDD and activation steps

1. Activate semantic records and add failing Engine-request, normal-result, admission-rejection,
   multiline, and inspection-command no-evaluation tests.
2. Route evaluable input through parse/check/lower/admit/Engine; retain canonical terminal error
   categories and history behavior.
3. Add parity controls for the same admitted source through REPL and `ash run`.
4. Run focused CLI/Engine tests plus semantic/documentation gates.

## Completion checklist

- [ ] REPL has no AST/CPS execution call outside Engine.
- [ ] Normal evaluable input and stored entries use admitted Engine requests.
- [ ] Inspection commands do not create an alternate execution route.
- [ ] Focused evidence reports implementation/evidence/parity independently.
