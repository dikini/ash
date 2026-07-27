# TASK-2032: Shared Engine Execution Seam and Client Parity

**Status:** Planned
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Depends on:** TASK-2031, TASK-2004, TASK-2014, and TASK-2008

## Description

Establish the one Engine-owned admitted-program execution seam used by all production clients, and
make the target-surface runnability matrix and CLI/daemon parity cases its durable integration
evidence. The task composes completed layer handoffs; it does not redefine Surface, Core, CPS,
handler, provider, terminal, or transport semantics.

## Handoffs

- **Run-route impact:** `active`. This is the PLAN-203 integration owner for an admitted program
  reaching the shared Engine CPS executor and its normalized terminal envelope through both
  client surfaces.
- **Consumes:** checked CPS and admission artifacts from TASK-2004/TASK-2014, effect-calculus
  correspondence from TASK-2031, canonical terminal projection from TASK-2008, and each
  feature-owning task's declared accepted or rejected source handoff.
- **Produces:** one Engine admitted-program interface; client adapters that submit the same
  admitted request; rule-indexed CLI/daemon parity cases; and
  `docs/plan/RUNNABLE-ASH-MATRIX.md`, which records every target production as either an
  executable shared-Engine route or an explicit source-to-client rejection contract.
- **Downstream owner:** feature-realization tasks consume matrix gaps for their own layer/domain;
  TASK-2032 retains the integration cases and matrix entries that compose those handoffs.
- **Does not own:** a feature's parser, typing, Core/CPS lowering, provider implementation,
  handler semantics, terminal taxonomy, or daemon transport protocol.
- **Integration/proof responsibility:** TASK-2032 owns the active-route proof that the same
  admitted request—artifact/source identity, inputs, bindings, deadline, cancellation signal, and
  declared host configuration—normalizes to the same terminal result through CLI and daemon.
  It records timeout and cancellation using the canonical versioned envelope, not client-specific
  result types. Verus work remains optional traceability evidence.

## Requirements

- Remove source-shape, handler-name, and client-local semantic dispatch from production route
  selection. Unsupported input rejects at the shared admission boundary; it never selects a
  direct evaluator.
- Define one Engine request/result interface over the validated admission artifact and canonical
  terminal envelope, including versioned timeout and cancellation control.
- Create and maintain the runnability matrix with rule identity, source production, owner,
  accepted/rejected disposition, relevant handoffs, terminal cases, and parity evidence.
- Add end-to-end positive, negative, mutation, and CLI/daemon parity tests for every route this
  task marks active. Route-specific formatting and daemon lifecycle behavior remain outside the
  normalized-terminal comparison.
- Preserve the admission invariant: only explicit, validated frame instructions install frames;
  rows cannot grant authority or create frames.

## TDD and activation steps

1. Before implementation, add this task to the active semantic-task record scope with its
   canonical rules, coverage section, traceability edges, and evidence IDs.
2. Add failing Engine-interface and client-parity tests, including missing admission, malformed
   CPS, handler-body trap, timeout, and cancellation envelopes.
3. Add the initial runnability matrix before enabling any newly active route; update it in the
   same change as each route's source-to-terminal evidence.
4. Implement the shared adapters and remove the superseded route recognizers only when the same
   behavior is available through admitted CPS.
5. Run affected Rust tests, formatter, clippy, traceability, orientation, documentation, and
   client integration gates.

## Completion checklist

- [ ] Every production client reaches the same Engine admitted-program interface.
- [ ] No client-local or source-shape evaluator/fallback remains reachable for an admitted source.
- [ ] The runnability matrix covers every target production with an executable route or explicit
      source-to-client rejection contract.
- [ ] Active CLI/daemon parity cases compare the same request and normalized terminal envelope,
      including versioned timeout and cancellation outcomes.
- [ ] The active semantic workflow record, traceability evidence, CHANGELOG, and quality gates are
      complete and consistent.
