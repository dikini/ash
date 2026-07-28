# TASK-2032: Shared Engine Execution Seam and Client Parity

**Status:** Complete integration task; target-spec implementation remains partial
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Depends on:** TASK-2031, TASK-2004, TASK-2014, and TASK-2008

**Semantic task record:** [TASK-2032](../semantic-task-records.json)

**Semantic coverage map:** [TASK-2032 shared Engine execution seam record](../SEMANTIC-RULE-COVERAGE.md#task-2032-shared-engine-execution-seam-record)

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** A separately owned daemon transport/profile/binding task must carry an admitted request and V1 terminal envelope before a selected noncanonical provider or handler route can be daemon-active.

## Description

Establish the one Engine-owned admitted-program execution seam used by all production clients, and
make the target-surface runnability matrix and CLI/daemon parity cases its durable integration
evidence. The task composes completed layer handoffs; it does not redefine Surface, Core, CPS,
handler, provider, terminal, or transport semantics.

## Handoffs

**Handoff scope:** integration of the currently admitted production routes. This task does not
expand parser acceptance, typed/Core/CPS lowering, provider behavior, frame authority, or the
terminal taxonomy. Completing this integration task does not establish target-spec parity.

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
- **Integration/proof responsibility:** TASK-2032 owns same-request normalized-terminal parity
  for the two in-process Engine client adapters, including artifact/source identity, inputs,
  bindings, deadline, and cancellation signal. Each submission receives a fresh Engine-owned
  deadline from the request's retained timeout configuration, while cancellation stays shared and
  sticky. The daemon transport cannot carry the opaque request and currently exposes status rather
  than a V1 terminal envelope, so it has separate process-level activation/rejection evidence in
  the matrix. It records timeout and cancellation using the canonical versioned envelope, not
  client-specific result types. Verus work remains optional traceability evidence.

## Semantic workflow record

The active integration record in
[semantic-task-records.json](../semantic-task-records.json) consumes
`SEM-TARGET-CORE-CPS-001`, `OBS-TARGET-PROJECTION-001`, and the TASK-2031 admission,
handler-trap, timeout, cancellation, and terminal rules. It records the Engine-seam positive,
negative, mutation, and CLI/daemon parity witnesses. Those tests do not make the currently
selected source fixtures a target executor implementation claim.

## Task-owned evidence plan

- **Positive:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-POSITIVE` requires real Engine-issued
  requests for an admitted return, including TASK-2013's exact `deep_affine_clock` checked-CPS
  `Int(107)` result, to project the versioned terminal envelope.
- **Negative:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-NEGATIVE` requires unsupported checked input
  to reject at admission before client execution.
- **Mutation:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-MUTATION` requires forged checked evidence to
  reject before frame installation or provider dispatch.
- **Adapter parity:** `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY` proves that the same admitted
  request normalizes equally through the two in-process adapters for selected pure return,
  `time::sleep` timeout and cancellation, `trap_sleep`, `deep_affine_clock`, and provider-backed
  `forward_sleep` controls. `TEST-TASK-2032-CLIENT-ADAPTER-DEADLINE-REUSE-PARITY` additionally
  proves a delayed sequential submission refreshes its Engine-owned deadline without changing the
  request's shared cancellation signal.
- **Daemon service:** `daemon_start_execute_uses_hashed_source_bytes_after_drift_check` proves
  the canonical-pure success/status route uses the shared Engine seam.
  `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`
  (`ashd_rejects_selected_noncanonical_engine_routes_before_execution`) proves the selected
  noncanonical pure, provider, and handler fixtures explicitly reject at the daemon's canonical
  entry-index boundary; no adapter-only proof marks them daemon-active.

The focused Engine and adapter contracts name only opaque Engine request/result interfaces; no test
authorizes a client-local evaluator or a row-derived frame. The matrix separates that exact
same-request evidence from actual daemon service behavior, which is active only for the
canonical-pure status route and rejects the noncanonical selected fixtures before execution. The
`ash trace` client independently proves its admitted pure-return and missing-admission terminal
projection while retaining only provenance-recorder lifecycle ownership.

## Requirements

- Remove source-shape, handler-name, and client-local semantic dispatch from production route
  selection. Unsupported input rejects at the shared admission boundary; it never selects a
  direct evaluator.
- Define one Engine request/result interface over the validated admission artifact and canonical
  terminal envelope, including versioned timeout and cancellation control.
- Create and maintain the runnability matrix with rule identity, source production, owner,
  accepted/rejected disposition, relevant handoffs, terminal cases, and parity evidence.
- Add end-to-end positive, negative, mutation, and same-request adapter-parity tests for every
  route marked adapter-active, plus process-level daemon activation or rejection evidence for each
  source route. Route-specific formatting and daemon lifecycle behavior remain outside the
  normalized-terminal comparison until the daemon protocol carries a V1 terminal envelope.
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

- [x] Every production client reaches the same Engine admitted-program interface.
- [x] No client-local or source-shape evaluator/fallback remains reachable for an admitted source.
- [x] The runnability matrix covers every target production with an executable route or explicit
      source-to-client rejection contract.
- [x] Active in-process adapter parity cases compare the same request and normalized terminal
      envelope, including versioned timeout and cancellation outcomes; daemon transport additions
      require separate service evidence before a row is marked daemon-active.
- [x] The active semantic workflow record, traceability evidence, CHANGELOG, and quality gates are
      complete and consistent.

## Completion evidence

The scoped Engine seam test suite covers positive, negative, mutation, issuer-integrity, and
property-based literal-return evidence (14 tests). The adapter suite covers return, timeout,
cancellation, handler, provider, and delayed same-request deadline-reuse parity (7 tests).
Focused daemon, terminal-envelope, trace, formatter, check, strict Clippy, semantic-record,
orientation, documentation, and traceability gates passed after independent QA and review.
