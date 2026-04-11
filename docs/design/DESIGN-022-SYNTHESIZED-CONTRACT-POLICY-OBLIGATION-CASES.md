# DESIGN-022: Synthesized Contract / Policy / Obligation Cases

## Status: Draft

## Overview

Design and implement the substrate required to turn synthesized tests from contracts, policies, and obligations into real executable test cases rather than planning-level labeled records.

This note is a follow-up to Phase 76. The current `ash test` runner can:
- select synthesized sources explicitly (`contracts`, `policies`, `obligations`)
- label synthesized results distinctly from authored tests
- report synthesized cases in human and JSON output

However, the current implementation still produces planning-level synthesized records rather than true end-to-end executable cases. This note defines the missing execution model, metadata extraction model, and oracle model needed to close that gap.

## Problem Statement

The current synthesized-test implementation is intentionally conservative. It can identify that a file contains contract-, policy-, or obligation-relevant material, but it cannot yet produce truthful executable cases grounded in stable structured metadata.

Today, the main missing pieces are:
1. no stable runner-facing introspection API for lowered contracts, policy definitions, or obligation lifecycle metadata
2. no canonical internal representation for synthesized executable cases
3. no principled way to generate inputs or worlds from the extracted metadata
4. no stable set of oracles for judging synthesized outcomes as pass/fail/error rather than mere planned cases

Without those pieces, the runner can only produce labels such as:
- synthesized/contract/...
- synthesized/policy/...
- synthesized/obligation/...

but not actually execute trustworthy synthesized tests.

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

### 2.1 Contracts

Required API surface:

```text
SynthesizableContractTarget {
  callable_name,
  callable_kind,      // function | workflow-callable
  params,
  lowered_requires,
  lowered_ensures,
  source_span,
}
```

The runner should be able to enumerate all callable items with lowered contract boundaries that are already accepted by the typechecker/runtime boundary.

### 2.2 Policies

Required API surface:

```text
SynthesizablePolicyTarget {
  policy_name,
  input_shape,
  lowered_policy,
  supported_terminal_outcomes,
  source_span,
}
```

The key missing contract here is the input shape/domain description. The runner needs a bounded way to construct representative policy inputs.

### 2.3 Obligations

Required API surface:

```text
SynthesizableObligationTarget {
  obligation_name,
  lifecycle_kind,
  introduction_sites,
  discharge_sites,
  required_closeout_behavior,
  source_span,
}
```

For obligation synthesis, the runner needs explicit lifecycle metadata rather than simple text matches for `oblige` and `check`.

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

Every synthesized case must emit repro data.

```text
SynthesizedRepro {
  target_name,
  source_kind,
  seed,
  bindings,
  world,
  oracle,
}
```

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
2. at least contract-derived synthesized cases are executable end-to-end
3. synthesized `pass` means an executed oracle passed, not merely that planning succeeded
4. policy/obligation synthesized cases stop being planning-only placeholders as their required metadata APIs land
5. synthesized output remains explicit, labeled, reproducible, and opt-in

## Recommendation

Implement synthesized execution in narrow vertical slices. Do not try to make all three metadata sources fully executable at once. Start with contract-derived cases, then obligation lifecycle cases, then policy-domain cases. The prerequisite is a stable runner-facing metadata extraction layer, not more CLI flags.
