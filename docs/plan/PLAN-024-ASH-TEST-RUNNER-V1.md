# PLAN-024: Ash Test Runner V1

## Status: Complete (V1 / Phase 76B final remediation complete; residuals deferred)

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
| [TASK-1011](tasks/TASK-1011-phase-76b-final-remediation-and-design022-023-planning.md) | Remediate final Phase 76B review blockers, reconcile narrow-slice status, and plan DESIGN-022/023 completion follow-on work | PLAN-024, DESIGN-022/023, SPEC-077, PLAN-127 | 6-8 | 513-515 |

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

Closeout note: the current implementation supports explicit synthesized-source selection, direct
kind-directory discovery, real per-test timeout containment via isolated execution, `-- @test`
metadata parsing, minimal `std::test` authored usage, and targeted verification/smoke coverage
consistent with the v1 phase contract. TASK-513/TASK-514 close the Phase 76B structured metadata
execution slice for narrow synthesized contract/policy/obligation cases, exact finite generated
property inputs, and deterministic explicit small-world states. TASK-515 records final phase
documentation, verification, and smoke evidence. TASK-1011 remediates the final review blockers:
obligation lifecycle pass rows now evaluate explicit finite lifecycle world state, uncapped
bounded-int domains defer before range materialization, and synthesized kind/tag filters plus
fail-fast apply to structured synthesized results.

## Explicit Deferred Follow-Up Items

The following items were intentionally deferred from the closed Phase 76 surface. SPEC-077 / Phase 132 later completed the bounded DESIGN-022/023 MVP for these items while keeping arbitrary/open-domain runtime semantics outside the MVP boundary:

1. Live checked/lowered snapshot production from ordinary CLI source files
   - Phase 132 adds live checked/lowered `RunnerIntrospectionSnapshot` production for supported ordinary CLI source files
   - unsupported raw-source/open-domain paths still defer rather than passing
2. Broader synthesized contract/policy/obligation execution beyond the TASK-513 checkpoint
   - live wiring from lowered contracts, policies, and obligations into the runner snapshot
   - executable contract postcondition cases beyond the narrow structured contract `requires`
     boundary slice
   - policy execution beyond exact `TerminalEquals` allow/deny metadata and obligation execution
     beyond exact finite lifecycle metadata
3. Rich generative property testing beyond TASK-514
   - property oracles beyond the current exact finite descriptor values with narrow metadata
     `property_holds` expectations
4. Rich small-world exploration beyond TASK-514
   - broader product/list/state-machine domain descriptors
   - live wiring from lowered policy/obligation/role metadata into runner snapshots
5. Richer `std::test` surface
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
     terminal expectations, explicit finite lifecycle world states, and small-world
     derivation hints
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
executed `pass` outcomes for true synthesized or true small-world cases. TASK-513 adds
the first runner-side synthesized-case substrate, a `SuiteConfig` structured snapshot seam, narrow
structured contract `requires` boundary oracles over exact generators, narrow policy
`TerminalEquals` allow/deny oracles over exact finite domains, and narrow obligation lifecycle
oracles over explicit finite lifecycle world-state metadata. Unsupported raw-source paths and incomplete
metadata remain explicit deferred skips at the Phase 76B boundary. Phase 132 later adds user-facing
live checked snapshots for supported ordinary CLI source files. TASK-514 extends that seam with exact finite generated property
inputs and deterministic explicit small-world state enumeration, including generated input/world repro
snapshots and `--max-worlds` truncation over actual metadata worlds.

## Summary of Reasoning for Deferral

These items are deferred intentionally, not forgotten. The current Phase 76 landing establishes the
runner substrate, authored test metadata/discovery path, minimal `std::test` surface, bounded
authored property/small-world compatibility hooks, metadata-backed generated property execution,
explicit small-world state enumeration, and explicit synthesized-source selection. That is the
smallest coherent v1 slice that can be verified honestly today before TASK-515 closeout.

The deferred items all depend on upstream spec and metadata-shape improvement rather than mere local
implementation effort:
- broader synthesized contract/policy/obligation execution needs live lowered metadata for runtime
  contract targets, policy domains/oracles, and obligation lifecycle semantics; without that, the
  runner must keep unsupported paths as explicit deferred skips rather than truthful executable
  synthesized tests
- richer property generation still needs live checked metadata and richer oracle shapes beyond the
  TASK-514 exact finite descriptor slice
- richer small-world execution still needs broader domain families and live lowered metadata wiring
  beyond the TASK-514 explicit finite world slice
- a broader `std::test` surface should wait until panic semantics, runtime-facing assertions, and
  related helper contracts are stable enough to avoid committing to the wrong long-term API

In short: Phase 76 closes the execution substrate and explicit extension points, plus the narrow
Phase 76B structured snapshot execution slice. SPEC-077 and PLAN-127 subsequently track and close
the bounded Phase 132 completion work for DESIGN-022 and DESIGN-023, while richer arbitrary
open-domain/runtime-heavy semantics remain outside that MVP.

## Deliverable

A practical Ash test runner v1: CLI-integrated, panic-contained, assertion-backed by a dedicated
Ash test library surface, capable of executing authored unit/integration/e2e tests plus bounded
property/small-world tests, and able to execute the narrow structured-snapshot synthesized and
generated/small-world metadata slices only when explicitly requested. Phase 132 extends this with
live checked/lowered snapshots for supported ordinary CLI source files; raw-source/open-domain
unsupported paths remain explicit deferred compatibility rows.
