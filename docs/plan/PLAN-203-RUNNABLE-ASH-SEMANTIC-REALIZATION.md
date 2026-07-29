---
id: plan.203.runnable-ash-semantic-realization
title: Runnable Ash Semantic Realization
kind: plan
status: active
authority: planning
owner: language-semantics
last_verified: 2026-07-27
---

# PLAN-203: Runnable Ash Semantic Realization

## Purpose

Realize target Ash through one production path:

```text
Surface Ash → checked Core → checked CPS → Engine CPS executor → terminal envelope
```

`ash run`, `ash test`, and REPL each use their own local Engine instance. They do not communicate
with the daemon. The daemon executes submitted descriptors through its own local Engine instance
and manages long-running programs. These routes share Engine implementation and contracts, but
there is no Engine service. No client defines another evaluator, lowers source through a separate
semantic route, or reconstructs admission authority. A source program without validated lowering
rejects at admission; it never falls back to a legacy direct evaluator.

## Relationship to the formal semantics programme

PLAN-202 owns canonical semantic authority, the λAsh calculus suite, traceability, and experimental
assurance work. PLAN-203 owns realization of that authority in the executable pipeline. The
calculi live beside CPS:

```text
Surface → Core → CPS → Engine executor
                   │
                   └─ mathematical semantics: λAsh-CPS₀, then λAsh-Effect and later extensions
```

`λAsh-CPS₀` explains CPS control. `λAsh-Effect` is its complete future conservative extension for
effectful CPS control; it must correspond to the same target operational rules and Engine state
view. Neither calculus is a second IR, lowering stage, or execution owner.

## Programme controls

### Layer ownership and integration

Existing tasks retain their layer ownership. Surface, Core, CPS, admission/runtime,
terminal-projection, and conformance work compose through named handoffs. PLAN-203 owns the
integration gate; it does not make an upstream task responsible for a downstream implementation
layer.

The target Ash specification defines the complete feature domain. Each task and rule report records
implementation, evidence, and parity independently. A completed handoff or integration task does
not establish target-spec parity: the feature remains `partial` and `below_spec` until its complete
target rule is realized. New behavior outside a target rule requires a specification update before
implementation.

Every new or materially revised PLAN-203 task must state:

- **Run-route impact:** `none`, `prerequisite`, or `active`.
- **Consumes** and **Produces** handoffs, including stable rule/artifact identities.
- **Downstream owner** for intentionally unowned layers.
- **Integration responsibility:** the PLAN-203 integration task/case that will provide proof or
  test evidence for the stated path, or an explicit reason why no execution route is involved.

An `active` route has its required integration test evidence only when its admitted program reaches
the local Engine executor for each client and the same normalized terminal result is tested through
both CLI and daemon clients. A `none` or `prerequisite` task can complete when it provides its
declared handoff.
Neither task completion nor an integration test turns a feature into an implemented target rule.

A PLAN-203 task marked **Planned** is an activation backlog item, not an active semantic-task
record. Its activation change must promote the task to **In progress** and add the semantic-task
record, coverage section, traceability links, and focused verification evidence before any semantic
Rust implementation is staged.

### Admission and client parity

The Engine admission artifact is the sole authority to construct provider or handler frames. It
contains checked CPS, source anchors, concrete operation identities, checked handler facts,
resolved provider bindings, and separately authorized frame instructions. Rows remain requirements
and never install frames.

CLI/daemon parity compares the normalized terminal result for the same source contract. Direct
clients retain the source bytes in their local Engine; daemon validates source identity and digest,
entry, inputs, bindings, controls, and host configuration from its wire descriptor before minting
its local request. Formatting, transport framing, and daemon lifecycle are outside that comparison.

### Semantic conformance and assurance ledger

Every executable rule has a rule-indexed conformance case through the shared executor. The
semantic traceability graph remains the assurance ledger: implementation, test, and proof links
are separate. Tests provide confidence in realized behavior; proofs identify their theorem and
refinement scope. Future proof work is recorded as an explicit `PROOF-*` disposition or proof node.

Verus is experimental and non-blocking. A task may label an obligation **candidate** or **pilot**;
the graph records it as deferred until a verified proof artifact exists. Only a verified artifact
may be reported as proved, and a model proof is not production-runtime proof without a checked
refinement bridge. Unselected or deferred obligations never block executable realization. The
initial pilots should concentrate on frame authorization, innermost lookup, affine resume use, and
terminal projection.

## Milestones

1. **Semantic alignment (TASK-2030).** Correct the Surface → Core → CPS reading path and
   establish the programme controls.
2. **Effectful CPS correspondence (TASK-2031).** Define λAsh-Effect syntax, configurations, judgments,
   transitions, examples, and operational/Rust correspondence before its executor expansion.
3. **Shared execution seam (TASK-2032).** Expose one Engine-owned admitted-program interface
   consumed by CLI and daemon clients; remove source-shape and handler-name dispatch as semantic
   routing, and maintain the runnability matrix and parity cases.
4. **Feature realization.** Carry each specified Surface → Core → CPS family through admission,
   execution, terminal projection, and its owning integration case.
5. **Release integration.** Maintain a target-surface runnability matrix. Every production is either
   accepted, checked, lowered, admitted, executed, terminalized, and client-parity tested, or it is
   explicitly removed with a source-to-client rejection contract.

## Initial task alignment

- TASK-2001 and TASK-2002 produce source/type/lowering facts.
- TASK-2003 and TASK-2004 own control/CPS and production-boundary handoffs.
- TASK-2013 produces checked handler facts; TASK-2014 owns admission, authorized frames, and CPS
  execution; TASK-2008 owns terminal projection.
- TASK-2005 and TASK-439 remain conformance evidence owners. They do not define a second runtime.
- [TASK-2031](tasks/TASK-2031-lambda-ash-effect-correspondence.md) defines λAsh-Effect and its
  CPS/operational/Rust correspondence contract before expanding generic effectful execution.
- [TASK-2032](tasks/TASK-2032-shared-engine-execution-seam-and-client-parity.md) owns the shared
  Engine execution seam, CLI/daemon parity cases, and the target-surface runnability matrix.

## Completion evidence

PLAN-203 is complete when the target-surface runnability matrix has no unspecified execution path,
all executable entry points share the Engine path, CLI/daemon parity cases cover applicable
terminal outcomes, no direct-evaluator fallback remains reachable, and traceability records both
realized behavior and any intentionally deferred proof obligations.
