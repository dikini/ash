# TASK-2051: Language Reference for Handlers, Failure, Do, and Comprehensions

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045; coordinate row terminology with TASK-2050
**Owned feature IDs:** LANG-013, LANG-014, LANG-022.

## Description

Document current source handler forms, scoped failure, direct-style do blocks, and bracket
comprehensions, including their deliberately partial lowering/admission/runtime routes.

## Requirements

- Create `docs/reference/language/effects/handlers-failure-do-and-comprehensions.md`.
- Cover source `handler`, `on`, and `handle … with` separately from Core/CPS `Raise`; source
  `raise` is excluded because it has no parser production.
- Cover source `fail payload` and `with_error { body } handle { pattern => expression; ... }` as
  current forms with parser/Core lowering and checked limitations; do not collapse them into the
  separate handler declaration mechanism or old tower failure prose.
- State that handler execution/admission is fixture-bounded, source ordered/deep-affine evidence
  is not general handler execution, and rows never install a frame.
- Distinguish ambient-do lowering from generic typed-do/comprehension paths that currently reject
  ordinary lowering; retain static-only examples where appropriate.

## Handoffs and dependencies

- **Consumes:** `parse_expr.rs`, `surface.rs`, `lower.rs`, typeck handler/do paths, Core/CPS, and
  Engine admission boundaries.
- **Evidence:** `cargo test -p ash-parser --test task_2013_handler_surface`; `cargo test -p
  ash-typeck --test task_2013_handler_core_lowering`, `--test task_1024_do_and_comprehension_stdlib_evidence`;
  `cargo test -p ash-engine --test task_2014_handler_production_admission`, `--test
  task_1024_stdlib_do_evidence`; `cargo test -p ash-parser --test task_708_fail_with_error`; `cargo
  test -p ash-typeck --test task_708_operational_bottom`.
- **Produces:** handler and do links consumed by execution/diagnostic pages.
- **Non-goals:** source raise, general/multi-shot/open-row handler execution, `do:Act`/`do:Proc`/
  `do:Workflow`, or assumed runtime parity.

## TDD and verification steps

1. Create a feature matrix that fails closed for each parsed form without lowering/admission proof.
2. Verify handler/do positive and negative routes through named parser/typeck/Engine tests.
3. Render only implementation-backed typing/transition sequents and supported EBNF.

## Completion checklist

- [ ] Source handlers, IR-only raises, and fixture-bounded runtime evidence are unambiguous.
- [ ] Do/comprehension static and lowering gaps are explicit.
- [ ] Removed forms never appear as current examples.
- [ ] Indexes, changelog, and PLAN-INDEX are updated.
