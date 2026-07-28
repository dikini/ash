# SPEC-077: Ash Test Runner Synthesized and Small-World Completion

**Status:** Implemented MVP
**Related:** DESIGN-022, DESIGN-023, PLAN-024, PLAN-127

## Summary

This specification defines the follow-on work required to complete DESIGN-022 and DESIGN-023 after the narrow Phase 76B structured-snapshot slice. Phase 76B implemented runner-injected structured snapshots, narrow contract `requires` boundary cases, policy `TerminalEquals` allow/deny cases, explicit finite obligation lifecycle world-state oracles, exact finite generated property values, explicit finite small-world states, repro artifacts, synthesized filters/fail-fast, and bounded-int cap safety.

Phase 132 implements the bounded MVP for this specification. Ordinary `ash test` CLI source files can produce live checked/lowered `RunnerIntrospectionSnapshot` values for supported pure-function contract metadata; supported contract postconditions, policy terminals, obligation lifecycle transitions, and small-world target-output oracles execute over explicit finite metadata; richer finite domains are bounded and fail closed; raw-source compatibility scans, open domains, unsupported setup, and arbitrary Ash/runtime semantics remain deferred-skip only.

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

## 8. Engine-only exact source-wrapper contract

`CONF-SYNTH-SOURCE-WRAPPER-001` defines the selected implementation case for the
source-wrapper route. It does not alter the complete target domains in Requirements 1 through 7.
Those domains remain the target specification; an unimplemented portion is reported as `partial`
implementation and `below_spec` parity, never narrowed by a selected implementation slice.

The exact source-wrapper catalogue contains the following one row:

| Source-contract ID | Consumed audit record | Exact source | Source identity and observation |
|---|---|---|---|
| `TASK-2035-SYNTH-WRAPPER-001` | `AUDIT-204-TEST-EXEC-002` | `fn contract_target_zero() -> Int { 0 }`<br>`fn main() -> Bool { contract_target_zero() == 0 }` | source digest `sha256:71990ce4a503c89efb95340a6d7c6674a036858b8e337f8b9bc4337839ebe390`; callable `contract_target_zero`; literal input `[]`; postcondition in `main`; expected Engine terminal projection of `Bool(true)`. |

The source-contract ID and source text are exact. The audit record identifies the client material
to retire; it is not source provenance. The table is neither a template nor a grammar for
generating wrappers. A synthesized client must parse, check, lower, admit, and execute a selected
source through Engine. This selected implementation catalogue does not reject a wrapper already authorized by Requirements 1 through 7 merely because it is not selected here. A new wrapper outside those target requirements requires a target-specification amendment before implementation.

The seven enumerated unsupported shapes from AUDIT-204 remain deferred and must produce their exact
recorded result:

| Audit case ID | Required result |
|---|---|
| `test:contract_postcondition_without_executable_target_metadata` | `deferred: contract metadata lacks executable postcondition target metadata` |
| `test:contract_postcondition_without_structured_oracle_metadata` | `deferred: contract postcondition metadata is not executable` |
| `test:contract_postcondition_with_unsupported_target_kind_defers` | `deferred: unsupported contract target kind runtime_callable` |
| `test:contract_postcondition_with_missing_setup_defers` | `deferred: contract target execution setup is missing` |
| `test:contract_postcondition_explicit_finite_setup_defers` | `deferred: explicit finite setup is not executable for pure target slice` |
| `test:contract_postcondition_unsupported_body_defers` | `deferred: contract target body is not executable` |
| `test:contract_postcondition_missing_exact_input_defers` | `deferred: contract postcondition oracle lacks exact valid input representatives` |

No deferred case may be reclassified as passing through a local Core/AST predicate evaluator,
client-local CPS executor, or differential oracle. The table specifies intended behavior; it is not
test or proof evidence. TASK-2038 owns runtime realization and focused test evidence.

### Separate Lean handoff

Lean is deferred to `external:lean-reference-project`. It is neither a synthesized-test oracle nor
an execution, conformance, proof, or refinement authority for current Ash. Any future Lean work
must establish a checked refinement bridge in that separate project.

## Changelog

### 2026-06-03

- Initial draft created after Phase 76B final remediation to define the remaining DESIGN-022 and DESIGN-023 completion work.
- Phase 132 closeout promoted SPEC-077 to Implemented MVP after TASK-1012 through TASK-1018 landed and the closeout verification gate passed.
