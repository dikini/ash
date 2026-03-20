# Tooling and Surface Steering Brief

Date: 2026-03-20
Status: TASK-204 output

## Purpose

This brief synthesizes:

- [CLI and REPL Interaction Planning Review](../audit/2026-03-20-cli-and-repl-interaction-planning-review.md)
- [Trace Export and Presentation Planning Review](../audit/2026-03-20-trace-export-and-presentation-planning-review.md)
- [Tooling and Surface Implementation Planning Plan](2026-03-20-tooling-surface-implementation-planning-plan.md)

Its purpose is to define the next tooling and surface code-facing task clusters that should later
exist, without opening them yet, and to make the tooling/surface review checkpoint explicit.

## Steering Outcome

The tooling/surface phase confirms a clean separation:

- CLI, REPL, and trace presentation remain runtime-observable user-facing surfaces
- explanatory stage guidance remains a separate documentation/presentation layer
- no CLI, REPL, or trace surface should be used as a proxy for projection or hidden reasoner state

The remaining work is user-facing convergence and presentation hardening, not semantic redesign.

## Tooling and Surface Task Clusters To Open Later

### 1. REPL observable-behavior convergence cluster

This cluster should cover the places where the REPL currently drifts from the frozen observable
contract.

Expected focus:

- canonical `:type` reporting instead of placeholder output
- authoritative command/help wording
- banner, prompt, and multiline behavior consistency
- keeping the CLI-owned REPL contract and implementation aligned

This is observable-behavior convergence work, not interaction-layer work.

### 2. CLI run/trace output convergence cluster

This cluster should cover the user-visible output and reporting shape of the CLI commands that
surface execution and trace information.

Expected focus:

- `ash run` result and trace-summary messaging
- `ash trace` export confirmations, integrity acknowledgements, and output shape
- keeping observable error categories distinct
- preserving the difference between runtime authority and explanatory text

This cluster should remain tooling-level and runtime-observable.

### 3. Stage-guidance and presentation overlay cluster

This cluster should cover the explanatory wording that may later help humans read advisory, gated,
and committed stages without changing runtime semantics.

Expected focus:

- optional trace/report wording for accepted versus rejected progression
- explanatory labels in docs-facing or tooling-facing surfaces
- careful use of stage language so it remains presentation-only

This cluster must not redefine runtime truth or imply hidden reasoner context.

## Explicit Non-Goals For Later Tooling Work

The tooling/surface review does not authorize:

- new syntax
- new stage markers in the language
- projection redesign
- monitor or `exposes` reinterpretation
- using CLI, REPL, or trace output as an implicit reasoner context channel

Those remain outside tooling/surface planning.

## Review Checkpoint

Review this brief before opening any tooling or surface code-facing tasks.

The steering questions are:

1. Are the proposed tooling changes clearly presentation-level or observable-behavior convergence,
   rather than semantic changes?
2. Is the boundary between runtime-observable behavior and explanatory stage guidance still
   explicit?
3. Did any proposed tooling cluster accidentally start carrying projection or hidden reasoner-state
   meaning?

If the answer to any question is no, revise the task cluster boundary before opening tasks.

## Recommendation

The next tooling implementation-planning move should be to open a small tooling-first task set
derived from these clusters, with REPL observable-behavior convergence first.

In practical terms:

- prioritize REPL `:type` and command-surface convergence first
- keep trace-presentation and wording work separate from runtime semantics
- treat stage-guidance overlays as optional explanatory work after the observable contract is
  implemented cleanly

## Closure

The tooling/surface phase is complete.

It found no runtime/reasoner conflation in the CLI, REPL, or trace presentation surfaces.
The remaining work is user-facing convergence and presentation cleanup on top of the already-frozen
runtime-observable contracts.
