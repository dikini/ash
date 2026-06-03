# SPEC-077: Ash Test Runner Synthesized and Small-World Completion

**Status:** Draft
**Related:** DESIGN-022, DESIGN-023, PLAN-024, PLAN-127

## Summary

This specification defines the follow-on work required to complete DESIGN-022 and DESIGN-023 after the narrow Phase 76B structured-snapshot slice. Phase 76B implemented runner-injected structured snapshots, narrow contract `requires` boundary cases, policy `TerminalEquals` allow/deny cases, explicit finite obligation lifecycle world-state oracles, exact finite generated property values, explicit finite small-world states, repro artifacts, synthesized filters/fail-fast, and bounded-int cap safety.

Full completion remains open. Ordinary `ash test` CLI source files still do not produce live checked/lowered `RunnerIntrospectionSnapshot` values, contract postconditions do not yet execute real targets end to end, and policy/obligation/small-world execution is limited to narrow metadata-backed finite cases.

## Requirements

### 1. Live Snapshot Production

`ash test` must build a checked/lowered `RunnerIntrospectionSnapshot` from ordinary CLI source files and suite roots before synthesized execution. Raw-source scans may remain compatibility discovery only and must emit deferred skip rows, never pass rows.

The snapshot producer must include source artifact identity, check summary identity, schema version, contracts, policies, obligations, generator descriptors, small-world domains, and unsupported rows.

### 2. Contract Target Execution

Synthesized contract cases must be able to execute real checked targets for supported pure functions, act functions where capability setup is explicit, and workflow callables where admission/setup is finite and supported.

Supported contract oracles must include:
- precondition boundary acceptance/rejection
- postcondition `ensures` checks over actual target results
- runtime postcondition hooks where metadata exposes a stable oracle

Unsupported target kinds, missing setup, open domains, and unrenderable values must defer.

### 3. Policy Domain and Oracle Execution

Policy synthesized cases must execute over explicit bounded domains from checked policy metadata. Supported terminals should grow from allow/deny to approval and transform only when lowered policy metadata exposes exact finite inputs and stable terminal oracle values.

Policy execution must preserve required authority metadata and fail closed when authority setup is missing.

### 4. Obligation Lifecycle Execution

Obligation synthesized cases must move beyond metadata-only terminal-state equality into real lifecycle execution when lowered obligation metadata exposes introduction, discharge, check, rejection, and closeout semantics.

Supported slices must include introduced, discharged, missing-discharge rejected, and double-discharge rejected. Pass requires an evaluated lifecycle world or runtime-backed lifecycle execution, not metadata presence.

### 5. Small-World Execution

Small-world execution must materialize deterministic finite worlds and execute Ash targets against each world. `--max-worlds` must bound actual world materialization and execution.

Supported domains should grow in this order:
- explicit states and explicit values
- bool and safely capped bounded integers
- bounded products
- bounded lists
- role/capability inclusion sets
- obligation lifecycle state machines
- policy-context worlds

Uncapped generated domains must defer before materialization.

### 6. CLI Integration

The CLI must route ordinary source files through checked snapshot production for `--include-synthesized` and `--only-synthesized`, while preserving authored test behavior. Filters, source selection, fail-fast, seed, max-cases, max-worlds, timeout, and JSON/human output must apply consistently to synthesized and small-world rows.

### 7. Reproducibility and Verification

Every executed generated or world case must include a `ReproArtifact` with source/check identity, seed, case/world index, generated input or world snapshot, oracle snapshot, and replay command.

Verification must include focused RED/GREEN tests for each new slice plus broad `ash-cli` runner gates, workspace check/clippy, and JSON output assertions.

## Non-Goals

- Symbolic execution, proof-producing synthesis, and unbounded model checking.
- Automatic arbitrary-value generation for open resources, capabilities, functions, processes, or unconstrained generics.
- Hosted/distributed test orchestration.

## Implementation Tasks

- TASK-1012: Live checked/lowered runner snapshot production.
- TASK-1013: End-to-end synthesized contract target and postcondition execution.
- TASK-1014: Policy domain and terminal oracle execution.
- TASK-1015: Runtime-backed obligation lifecycle execution.
- TASK-1016: Small-world materialization and Ash target execution.
- TASK-1017: Richer finite domains and CLI integration hardening.
- TASK-1018: Completion closeout, broad verification, and design promotion.

## Changelog

### 2026-06-03

- Initial draft created after Phase 76B final remediation to define the remaining DESIGN-022 and DESIGN-023 completion work.
