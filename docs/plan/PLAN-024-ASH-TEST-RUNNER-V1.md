# PLAN-024: Ash Test Runner V1

## Status: Draft

## Overview

Build a first-class Ash-native test runner integrated with the Ash CLI. V1 establishes the execution substrate for authored Ash tests and explicitly synthesized metadata-driven tests, together with a dedicated Ash test library surface for assertions and test helpers.

This phase should land a fail-contained `ash test` command that can:
- discover and run authored Ash tests
- isolate failures and panics so one bad test does not stop the suite
- report results through one canonical result model
- support unit, integration, e2e, property, and small-world execution modes
- keep synthesized tests from contracts, policies, and obligations explicit, labeled, opt-in, and complementary to authored tests rather than silently replacing them

## Design Reference

- [DESIGN-021: Ash Test Runner V1](../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Problem Statement

Ash has strong Rust-side testing coverage but no canonical Ash-native test execution path. There is no `ash test` command, no shared per-test outcome model, no explicit Ash test library surface, and no agreed-on way to discover or label authored vs synthesized tests.

The phase must therefore land both:
1. a runner substrate integrated into `ash-cli`
2. an Ash test library phase for assertions and deterministic test helpers

V1 is intentionally scoped to running tests, not to the full long-term ecosystem of shrinking, coverage, mutation testing, fuzz orchestration, or proof-producing test synthesis.

## Goals

1. Add `ash test` to the CLI with human and JSON output.
2. Define one canonical test result envelope for authored and synthesized tests.
3. Provide per-test panic capture, timeout handling, and fail-contained suite execution.
4. Add a dedicated Ash test library surface for authored tests.
5. Support authored unit, integration, and e2e tests in v1.
6. Add minimal seeded property-test and bounded small-world execution paths.
7. Make contracts, policies, and obligations first-class metadata sources for synthesized tests, but only through explicit opt-in runner modes.
8. Freeze a practical file layout and metadata shape for Ash tests in the repository.

## Scope

**In Scope**:
- `ash test` CLI command and output modes
- canonical suite/test result model
- per-test panic capture and timeout isolation
- authored test discovery under explicit repository roots
- a minimal Ash test library surface for assertions and helpers
- authored unit/integration/e2e test execution
- opt-in synthesized tests from contracts, policies, and obligations
- bounded seeded property-test execution
- bounded small-world execution
- planning/bookkeeping/docs for the new phase

**Out of Scope**:
- advanced shrinking/minimization
- coverage and mutation testing
- broad automatic synthesis from all language metadata
- distributed/fuzzy/flaky orchestration features
- full long-term `std::test` design beyond the v1 minimal surface
- proof-producing or theorem-backed test synthesis

## Non-Goals

This phase does not attempt to provide:
- full shrinking/minimization comparable to mature proptest ecosystems
- mutation testing
- coverage reporting
- distributed test execution
- flaky-test quarantine or retry logic
- broad automatic synthesis from all language metadata
- a final long-term `std::test` surface beyond the minimal v1 assertion/helper substrate
- proof-producing semantic verification from tests

## Tracks

### Track 0: Phase 76B Rescope / Spec-Hardening Gate

Before the deferred Phase 76B implementation resumes, freeze the stable runner-facing
introspection contracts that make executable synthesized tests and true small-world
exploration honest.

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| [TASK-1010](tasks/TASK-1010-phase-76b-rescope-spec-hardening-packet.md) | Define stable runner-facing introspection APIs for contracts, policies, obligations, generated inputs, small-world state models, and reproducible artifacts | DESIGN-022, DESIGN-023 | 4-6 | 509-512 |

### Track 1: Runner Substrate

Build the core runner execution model and CLI integration.

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| [TASK-509](tasks/TASK-509-ash-test-runner-substrate.md) | Add `ash test` CLI surface, test discovery roots, and canonical human/JSON suite reporting | PLAN-024, DESIGN-021 | 6-8 | None |
| [TASK-510](tasks/TASK-510-test-execution-isolation-and-panic-capture.md) | Add per-test execution isolation with panic capture, timeout handling, and sealed result classification | PLAN-024, DESIGN-021 | 8-12 | 509 |

### Track 2: Ash Test Library Surface

Add the minimal Ash-side library vocabulary needed for authored tests.

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| [TASK-511](tasks/TASK-511-ash-test-library-surface.md) | Introduce a minimal Ash test library surface (`std::test` or equivalent) with assertions, panic-aware helpers, and runtime-facing test helpers | PLAN-024, DESIGN-021 | 8-12 | 509 |

### Track 3: Authored Test Metadata and Execution Model

Freeze how authored Ash tests are structured and discovered.

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| [TASK-512](tasks/TASK-512-authored-test-metadata-and-execution-model.md) | Define and implement authored test metadata syntax/structure, explicit test declaration/discovery rules, and authored unit/integration/e2e execution wiring | PLAN-024, DESIGN-021 | 10-14 | 510, 511 |

### Track 4: Synthesized Metadata-Driven Tests

Use existing language metadata as an explicit, opt-in source of synthesized tests.

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| [TASK-513](tasks/TASK-513-synthesized-tests-from-contracts-policies-and-obligations.md) | Add synthesized test planning/execution for contracts, policies, and obligations with explicit labeling and opt-in CLI controls | PLAN-024, DESIGN-021, DESIGN-022 | 10-14 | 510, 511, 1010 |

### Track 5: Property and Small-World Execution

Add bounded generative execution modes on the runner substrate.

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| [TASK-514](tasks/TASK-514-property-and-smallworld-execution.md) | Add seeded property-test execution and bounded small-world execution, including reproducible failure reporting and runner controls (`--seed`, `--max-cases`, `--max-worlds`) | PLAN-024, DESIGN-021, DESIGN-023 | 8-12 | 510, 511, 512, 1010 |

### Track 6: Phase Finalization

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| [TASK-515](tasks/TASK-515-ash-test-runner-docs-and-phase-verification.md) | Finalize docs/bookkeeping, update PLAN-INDEX/CHANGELOG, and run the v1 verification gate for the Ash test runner phase | PLAN-024, DESIGN-021, DESIGN-022/023 | 4-6 | 509-514, 1010 |

## Recommended File/Code Organization

Expected major implementation areas:

- CLI / runner orchestration:
  - `crates/ash-cli/src/commands/test.rs`
  - `crates/ash-cli/src/main.rs`
  - supporting output/report modules as needed
- runner substrate / execution model:
  - `crates/ash-engine/` or a dedicated test-runner module/crate if the split is justified during implementation
- Ash test library surface:
  - stdlib-visible test helpers under `std/src/` in a dedicated test module namespace
- authored test fixtures and examples:
  - `tests/ash/unit/`
  - `tests/ash/integration/`
  - `tests/ash/e2e/`
  - `tests/ash/property/`
  - `tests/ash/smallworld/`
- synthesized test planning:
  - runner-side code that consumes stored contract/policy/obligation metadata without pretending those tests were authored declarations

## Metadata and Discovery Contract

V1 should keep discovery explicit and file-oriented.

Recommended authored test roots:
- `tests/ash/unit/`
- `tests/ash/integration/`
- `tests/ash/e2e/`
- `tests/ash/property/`
- `tests/ash/smallworld/`

Recommended file-level metadata shape:

```ash
-- @test
-- name: option.unwrap_or returns default on None
-- kind: unit
-- tags: [stdlib, option]
-- timeout_ms: 1000
```

The metadata block controls runner behavior. The authored test body uses the Ash test library surface from TASK-511.

Synthesized tests must never be silently mixed into authored discovery. They should be surfaced separately in runner output as synthesized/contract, synthesized/policy, or synthesized/obligation cases.

## CLI Contract

V1 target CLI shape:

```text
ash test [PATH]
  --format human|json
  --filter <substring-or-pattern>
  --tag <tag>
  --kind unit|integration|e2e|property|smallworld
  --include-synthesized contracts,policies,obligations
  --only-synthesized contracts,policies,obligations
  --fail-fast
  --timeout <seconds>
  --seed <u64>
  --max-cases <n>
  --max-worlds <n>
```

Default behavior:
- run authored tests only
- do not synthesize metadata-driven tests unless explicitly requested
- continue after panic/failure unless `--fail-fast` is requested

## Acceptance Criteria

Phase 76 is complete when:

1. `ash test` exists and discovers authored tests from the agreed roots.
2. Each test executes inside a fail-contained boundary with panic capture and timeout handling.
3. The runner emits a stable human and JSON result format.
4. A minimal Ash test library surface exists and is usable from authored Ash tests.
5. Authored unit, integration, and e2e Ash tests run through the same runner/result substrate.
6. Property and small-world execution modes exist with explicit runner controls.
7. Contracts, policies, and obligations are usable as synthesized metadata-driven tests only through explicit opt-in flags.
8. Runner output preserves the distinction between authored and synthesized tests.
9. PLAN-INDEX, task records, and CHANGELOG are aligned with the landed state.

## Verification Gate

Before phase closeout:
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- targeted `ash test` smoke runs for authored tests
- targeted synthesized test smoke runs for contracts/policies/obligations

Current bookkeeping note: the current implementation now supports explicit synthesized-source
selection, direct kind-directory discovery, real per-test timeout containment via isolated execution,
`-- @test` metadata parsing, minimal `std::test` authored usage, and targeted verification/smoke
coverage consistent with the v1 phase contract.

## Explicit Deferred Follow-Up Items

The following items are intentionally deferred until after the next round of spec work improvement and should not be treated as part of the current closed Phase 76 surface:

1. True synthesized contract/policy/obligation execution
   - stable runner-facing introspection APIs for lowered contracts, policies, and obligations
   - executable synthesized cases rather than planning-level labeled records
2. Rich generative property testing
   - generated inputs from type/contract metadata rather than bounded reruns of the same authored body
3. True small-world exploration
   - bounded world/state enumeration rather than bounded reruns of the same authored body
4. Richer `std::test` surface
   - panic-aware helpers and runtime-facing helpers that depend on stronger stable runtime/spec support

## Phase 76B Rescope / Spec-Hardening Packet

TASK-1010 adds the required hardening layer before TASK-513, TASK-514, and TASK-515 may
continue as implementation work. The hardening packet defines these stable runner-facing
surfaces:

1. `RunnerIntrospectionSnapshot`
   - one checked/lowered read-only snapshot per module or suite root
   - carries contracts, policies, obligations, type-generator descriptors,
     small-world model references, source artifact identity, check summary identity,
     schema version, and unsupported/deferred rows
2. Contract metadata
   - callable identity, parameter names/types, return type, lowered `requires`, lowered
     `ensures`, runtime postconditions, generation hints, source span, and executable-case
     eligibility
3. Policy metadata
   - policy identity, bounded input domain, lowered policy reference, supported terminal
     outcomes, oracle shape, authority requirements, and materialization limits
4. Obligation metadata
   - obligation identity, scope, lifecycle model, introduction/discharge/check sites,
     terminal expectations, and small-world derivation hints
5. Type/contract-derived generated input descriptors
   - authored examples, exact finite domains, valid contract-domain representatives,
     invalid-nearby contract-domain representatives, and explicit unsupported cases
6. Small-world state model
   - deterministic finite-domain enumeration over `SmallWorldState` values, stable
     `world_index`, stable `world_id`, transition traces, and world-specific oracles
7. Reproducible artifacts
   - runner schema version, source artifact identity, check summary identity, seed,
     case/world index, generated input or world snapshot, oracle snapshot, and replay
     command

The implementation boundary is intentionally strict: raw-source pattern scans and bounded
reruns may remain as planning-level compatibility paths, but they must not produce
executed `pass` outcomes for true synthesized or true small-world cases.

## Summary of Reasoning for Deferral

These items are deferred intentionally, not forgotten. The current Phase 76 landing establishes the
runner substrate, authored test metadata/discovery path, minimal `std::test` surface, bounded
property/small-world execution hooks, and explicit synthesized-source selection. That is the smallest
coherent v1 slice that can be verified honestly today.

The deferred items all depend on upstream spec and metadata-shape improvement rather than mere local
implementation effort:
- synthesized contract/policy/obligation execution needs stable runner-facing introspection APIs for
  lowered contracts, policies, and obligation lifecycle semantics; without that, the runner can only
  produce planning-level labeled cases rather than truthful executable synthesized tests
- richer property generation needs a stable contract/type-driven input-generation story; bounded reruns
  of authored bodies are acceptable for v1, but not a substitute for true generated cases
- richer small-world execution needs a spec-backed state/world enumeration model instead of ad hoc
  bounded repetition
- a broader `std::test` surface should wait until panic semantics, runtime-facing assertions, and
  related helper contracts are stable enough to avoid committing to the wrong long-term API

In short: Phase 76 closes the execution substrate and explicit extension points, while the richer
synthesized/generative/test-library layers are deferred until the next round of spec work can define
those surfaces cleanly and prevent design debt.

## Deliverable

A practical Ash test runner v1: CLI-integrated, panic-contained, assertion-backed by a dedicated Ash test library surface, capable of executing authored unit/integration/e2e tests plus bounded property/small-world tests, and able to plan/source synthesized tests from contracts, policies, and obligations only when explicitly requested.
