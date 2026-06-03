# DESIGN-022: Synthesized Contract / Policy / Obligation Cases

## Status: Implemented MVP (Phase 132 complete)

Phase 132 completes the executable MVP for this design beyond the Phase 76B
narrow structured-snapshot substrate. Ordinary `ash test` CLI source files can
produce live checked runner snapshots, supported pure `Int` function contract
postconditions execute checked/lowered targets and `ensures` expressions,
policy and obligation rows execute through narrow stable metadata slices,
unsupported or string-only metadata stays deferred-skip only, and every executed
row carries reproducible source/check, input/world, oracle, and replay metadata.

Residual non-goals remain explicit: arbitrary open-domain generation, full
capability/Act/workflow setup, broad approval/transform policy semantics, and
unbounded or symbolic synthesis are not claimed by the MVP.

## Overview

Design and implement the substrate required to turn synthesized tests from contracts, policies, and obligations into real executable test cases rather than planning-level labeled records.

This note is a follow-up to Phase 76. The current `ash test` runner can:
- select synthesized sources explicitly (`contracts`, `policies`, `obligations`)
- label synthesized results distinctly from authored tests
- report synthesized cases in human and JSON output
- execute a narrow structured-snapshot slice when snapshots are injected through
  runner internals/tests

Before Phase 132, ordinary CLI source files did not produce live
checked/lowered snapshots, and the executable slice did not yet provide
end-to-end contract target/postcondition execution, policy domain/oracle
execution, or obligation lifecycle execution beyond narrow metadata checks. This
note defines the execution model, metadata extraction model, and oracle model
that Phase 132 implements for the bounded MVP.

## Problem Statement

The pre-Phase 132 synthesized-test implementation was intentionally conservative. It could identify that a file contained contract-, policy-, or obligation-relevant material, but it could not yet produce truthful executable cases grounded in stable structured metadata.

Phase 132 closes that bounded MVP gap by adding stable runner-facing snapshots, executable synthesized case carriers, exact finite metadata/domain descriptors, and oracles that classify pass/fail/error from evaluated target/oracle results. Unsupported metadata still reports deferred skips rather than synthesized pass rows.

## Goals

1. Define one canonical internal `SynthesizedCase` model shared by contract-, policy-, and obligation-derived cases.
2. Add stable runner-facing introspection APIs for the metadata required to build those cases.
3. Make synthesized cases executable and reproducible.
4. Preserve explicit, opt-in synthesized selection in `ash test`.
5. Keep synthesized tests visibly distinct from authored tests in planning, execution, and reporting.
6. Make the first executable synthesized cases small, honest, and bounded.

## Non-Goals

This design does not attempt to:
- replace authored tests
- auto-synthesize from every language feature
- solve full property generation from arbitrary Ash types
- solve full small-world exploration in the same phase
- commit to a final long-term theorem-backed synthesis story

## Design Principles

### P1. Structured metadata before generation

Do not generate synthesized cases from raw source text pattern matching once stable metadata APIs are available. Source scanning may remain a temporary bootstrap, but it is not the target substrate.

### P2. Execution plans, not just labels

A synthesized case must be an actual executable test plan with:
- target
- setup
- bindings
- expected outcome
- oracle
- repro metadata

If the runner cannot execute the plan, the case should remain classified as planning-only rather than pretending to pass.

### P3. Bounded first, broad later

The first executable synthesized cases should target narrow, explicit scenarios with clear semantics rather than trying to cover every possible contract/policy/obligation form at once.

### P4. Different metadata sources, one execution substrate

Contracts, policies, and obligations differ in semantics, but they should lower into one common synthesized-case execution substrate.

### P5. Honest reporting

If a synthesized case is only planned but not executed, report it as planning-level / deferred / skipped. Never stamp it `pass` merely because the runner recognized a pattern.

## Core Design

## 1. Canonical SynthesizedCase Model

Introduce a runner-internal case model:

```text
SynthesizedCase {
  id,
  source_kind,        // contract | policy | obligation
  target_kind,        // function | workflow | policy | obligation_lifecycle
  target_name,
  file_path,
  tags,
  seed,
  setup,
  inputs,
  oracle,
  expected_outcome,
  repro,
}
```

### Setup

```text
SynthesizedSetup {
  required_capabilities,
  role_context,
  policy_context,
  obligation_context,
  fixture_refs,
}
```

### Inputs

```text
SynthesizedInputs {
  bindings,
  generated_from,     // contract | fixed_example | finite_domain
  case_index,
  world_index,
}
```

### Oracle

```text
SynthesizedOracle {
  kind,
  details,
}
```

Supported oracle kinds should include:
- precondition_rejects
- postcondition_holds
- policy_allows
- policy_denies
- policy_requires_approval
- policy_transforms
- obligation_introduced
- obligation_discharged
- obligation_missing_discharge_rejected
- obligation_double_discharge_rejected

## 2. Runner-Facing Introspection APIs

The runner needs structured metadata, not ad hoc source strings.

TASK-1010 freezes the runner-facing API shape before implementation continues. The
first implementation task may choose Rust names that fit the crate boundaries, but it
must preserve the semantic contract in this section. Raw source scanning may remain as
a compatibility fallback only for planning-level `skip` results; it must not produce
executed `pass` outcomes.

### 2.0 Introspection snapshot

The runner should consume one stable snapshot per checked module or suite root:

```text
RunnerIntrospectionSnapshot {
  schema_version,
  module_identity,
  source_artifact_id,
  check_summary_id,
  contracts: Vec<RunnerContractMetadata>,
  policies: Vec<RunnerPolicyMetadata>,
  obligations: Vec<RunnerObligationMetadata>,
  generators: Vec<TypeGeneratorDescriptor>,
  small_world_models: Vec<SmallWorldModelRef>,
  unsupported: Vec<IntrospectionUnsupportedReason>,
}
```

The snapshot is a read-only handoff from parse/check/lowering infrastructure to the
runner. It must not expose parser-private raw syntax as the runner contract, and it
must not require the runner to re-parse source text to discover executable cases.

Required snapshot invariants:

- `schema_version` changes when the JSON/output or internal case-building contract
  changes in a replay-affecting way.
- `source_artifact_id` identifies the exact source artifact or file set used to build
  the snapshot.
- `check_summary_id` identifies the checked/lowered semantic summary consumed by the
  runner.
- `unsupported` records metadata that was recognized but cannot yet be materialized
  into executable synthesized cases. Unsupported rows may plan `skip` results with an
  explicit deferred reason; they must not be reported as successful execution.

### 2.1 Contracts

Required API surface:

```text
RunnerContractMetadata {
  id,
  callable_name,
  callable_kind,             // pure_function | act_function | workflow_callable
  param_names,
  param_types,
  return_type,
  lowered_requires,
  lowered_ensures,
  runtime_postconditions,
  generation_hints,
  executable_case_kinds,
  source_span,
}
```

The runner should be able to enumerate all callable items with lowered contract boundaries that are already accepted by the typechecker/runtime boundary.

`StoredFnContract` is the current live grounding point for function contracts, but it
is not yet enough by itself. The runner-facing metadata also needs parameter and return
type shape, stable target identity, generation hints, and an explicit list of case kinds
that are executable with the current substrate.

### 2.2 Policies

Required API surface:

```text
RunnerPolicyMetadata {
  id,
  policy_name,
  input_domain,
  lowered_policy_ref,
  supported_terminal_outcomes,
  oracle_shape,
  required_authority,
  materialization_limits,
  source_span,
}
```

The key missing contract here is the input shape/domain description. The runner needs a bounded way to construct representative policy inputs.

Policy metadata must be explicit about which terminals can be tested: allow, deny,
approval, transform, or unsupported. A policy case is executable only when the runner
can materialize both a bounded input and an oracle for the expected terminal outcome.

### 2.3 Obligations

Required API surface:

```text
RunnerObligationMetadata {
  id,
  obligation_name,
  scope,                    // workflow | role | local | runtime_resource
  lifecycle_model,
  introduction_sites,
  discharge_sites,
  check_sites,
  required_closeout_behavior,
  terminal_expectations,
  small_world_derivation_hints,
  source_span,
}
```

For obligation synthesis, the runner needs explicit lifecycle metadata rather than simple text matches for `oblige` and `check`.

Obligation metadata should describe the finite lifecycle that can be tested. The first
slice should cover introduced, discharged, missing-discharge, and double-discharge
cases only when those transitions are represented by stable lowered metadata.

### 2.4 Type and contract input descriptors

Executable synthesized tests need bounded values. The runner must not infer arbitrary
Ash values from strings or debug output.

```text
TypeGeneratorDescriptor {
  id,
  target_type,
  source,              // authored_examples | finite_domain | contract_valid | contract_invalid_nearby
  values_or_strategy,
  seed_policy,
  max_cases,
  unsupported_reason,
}
```

The first implementation slice should support only descriptors that are exact and
bounded: booleans, small integer ranges, known enum/variant representatives, explicitly
authored examples, and simple contract boundary representatives. Open types, resource
values, capabilities, functions, processes, and unconstrained generics remain
unsupported unless a later task defines a finite domain for them.

### 2.5 Reproducible artifacts

Every executed synthesized case must carry enough data to replay or diagnose the same
case after a failure:

```text
ReproArtifact {
  runner_schema_version,
  source_artifact_id,
  check_summary_id,
  case_id,
  seed,
  case_index,
  world_index,
  generated_input_snapshot,
  world_snapshot,
  oracle_snapshot,
  replay_command,
}
```

For generated inputs and worlds, the snapshot should be canonical and stable enough for
JSON output. If a value cannot be rendered faithfully, the case must either include a
digest plus a local artifact path or remain planning-only.

## 3. Source-Specific Case Synthesis

## 3.1 Contract-Derived Cases

First executable slice:
- positive precondition case
- negative precondition case
- postcondition check case

Examples:
- `requires x > 0`
  - generate one valid case: `x = 1`
  - generate one invalid boundary case: `x = 0`
- `ensures result > x`
  - execute on valid inputs and check postcondition oracle

Recommended v1 generated-case strategy for contracts:
- use narrow arithmetic boundary generation first
- avoid full arbitrary type generation initially
- prefer exact, reproducible hand-selected representatives from lowered constraints

## 3.2 Policy-Derived Cases

First executable slice:
- one allow case if policy can permit
- one deny case if policy can deny
- one approval/transform case only when the lowered policy explicitly supports those terminals

Policy synthesis should not guess a domain. It needs a bounded domain provider.

Recommended initial domain sources:
- explicit finite domains attached to policy-relevant fields
- simple canonical literals for Bool / small Int / small Enum-like nominal forms
- future metadata hooks for policy test domains

## 3.3 Obligation-Derived Cases

First executable slice:
- introduced obligation exists
- discharged obligation closes correctly
- missing discharge fails when required
- double discharge is rejected

This is naturally a bounded lifecycle/state-machine problem and should lower cleanly into a small number of canonical cases.

## 4. Execution Model

A synthesized case must execute through the same suite/result substrate as authored tests.

Execution steps:
1. materialize setup context
2. bind inputs or instantiate finite world state
3. execute target under the runner
4. evaluate oracle
5. emit canonical `TestResult`

### Reporting rules

- executed synthesized success -> `pass`
- executed synthesized contract/policy/obligation failure -> `fail`
- unable to materialize executable case due to missing substrate -> `skip` with explicit deferred reason
- infrastructure failure -> `error`

## 5. Reproducibility

Every executed synthesized case must emit a `ReproArtifact` from section 2.5. Older
`SynthesizedRepro`-style fields such as target name, source kind, bindings, world, and
oracle are not a separate schema; they are represented inside `case_id`,
`generated_input_snapshot`, `world_snapshot`, and `oracle_snapshot`.

This is especially important once generated contract cases and finite policy/obligation domains are introduced.

## 6. Recommended Implementation Order

### Stage A: Introspection substrate

1. Expose lowered function contract metadata to the runner in a stable form.
2. Expose runner-facing policy metadata with explicit terminal outcomes.
3. Expose obligation lifecycle metadata needed by the runner.

### Stage B: Executable contract synthesis

Start here. Contracts are the best first slice because they already have the strongest lowered structure.

Land:
- arithmetic precondition positive/negative representatives
- arithmetic postcondition checks
- reproducible contract-derived execution cases

### Stage C: Executable obligation synthesis

Then land:
- introduction/discharge/missing/double-discharge lifecycle cases

### Stage D: Executable policy synthesis

Finally land:
- bounded policy-domain execution cases for allow/deny/approval/transform

This order is recommended because:
- contracts have the clearest lowered substrate
- obligations have finite lifecycle semantics
- policies need the most domain-shape design

## 7. Open Design Questions

1. Where should bounded test domains live for policy synthesis?
   - inline metadata on policies?
   - runner-side fixtures?
   - a future std/test metadata library?

2. Should contract synthesis generate Ash fixtures or execute against internal engine/runtime bindings directly?

3. Should obligation synthesis target workflow snippets, lowered IR, or runtime lifecycle contracts directly?

4. How much of the synthesized-case model should be persisted in JSON output versus kept internal?

## Acceptance Criteria

This design note is realized when:
1. the runner can enumerate structured synthesizable targets for contracts, policies, and obligations
2. ordinary CLI source files produce live checked/lowered snapshots without relying on raw-source pass rows
3. contract-derived synthesized target and postcondition cases are executable end-to-end for supported metadata
4. policy and obligation synthesized cases execute supported domains/lifecycles rather than remaining metadata-only placeholders
5. synthesized `pass` means an executed oracle passed, not merely that planning succeeded
6. synthesized output remains explicit, labeled, reproducible, and opt-in

Phase 76B satisfied only the narrow structured-snapshot subset: injected
snapshots could execute finite metadata-backed contract `requires`, policy
terminal, and obligation lifecycle world-oracle cases. Phase 132 / SPEC-077 /
PLAN-127 complete the bounded MVP acceptance criteria above while preserving the
non-goals for arbitrary/open-domain generation and full runtime-heavy semantics.

## Recommendation

Implement synthesized execution in narrow vertical slices. Do not try to make all three metadata sources fully executable at once. Start with contract-derived cases, then obligation lifecycle cases, then policy-domain cases. The prerequisite is a stable runner-facing metadata extraction layer, not more CLI flags.
