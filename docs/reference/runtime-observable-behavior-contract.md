# Runtime Observable Behavior Contract

## Status

TASK-182 handoff reference.

## Purpose

This document freezes the runtime-visible behavior handoff for TASK-182 and points to
[SPEC-021: Runtime Observable Behavior](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) as the
canonical owner of the observable contract.

It defines:

1. which runtime outcomes are observably required,
2. which user-facing tool behaviors are normative,
3. which ADT and instance values have stable user-visible meaning, and
4. which failures must surface as observable errors rather than silent fallback.

## Scope

This handoff covers:

- the observable outcome classes that SPEC-021 freezes
- the REPL and CLI boundaries that SPEC-021 owns
- the stdlib-visible value-shape guarantees that SPEC-021 freezes for ADTs and instances

Out of scope:

- parser acceptance
- lowering mechanics
- private implementation details such as concrete storage layouts or internal caches

## Handoff Summary

SPEC-021 owns the canonical observable contract. This reference preserves the migration boundary
and records the implementation drift that TASK-182 was meant to eliminate.

Current implementation notes:

- `ash-cli` and `ash repl` still expose observable behavior that must converge to SPEC-021.
- CLI `:type` handling and REPL display text are still split across implementation paths.
- runtime value formatting and error visibility must converge to the constructor-shaped display
  and explicit `Result`-based recovery model in SPEC-021.
- observable control authority must converge to the reusable-supervision model in SPEC-021 rather
  than an affine one-shot control token.
- retained terminal control targets remain observably terminal for the lifetime of the owning
  runtime state; see [Control-Link Retention Policy](control-link-retention-policy.md).

## Observable Error Boundaries

See SPEC-021 for the canonical failure classes and observable value-shape guarantees.
