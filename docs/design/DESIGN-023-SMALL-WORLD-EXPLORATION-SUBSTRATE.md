# DESIGN-023: Small-World Exploration Substrate

## Status: Draft

## Overview

Design and implement the substrate required to turn small-world testing in Ash from bounded reruns of authored test bodies into true bounded exploration over explicit world/state spaces.

This note is a follow-up to Phase 76. The current runner supports:
- `smallworld` as an explicit test kind
- `--max-worlds` runner controls
- small-world-specific result labeling and reporting

But the current implementation does not yet explore true worlds. It simply reruns the same authored body in a bounded loop. This note defines the missing world model, enumeration model, and oracle model required for genuine small-world exploration.

## Problem Statement

Many Ash behaviors are naturally finite-state or bounded-interaction problems:
- role/capability combinations
- policy outcome surfaces
- obligation lifecycle transitions
- control-link lifecycle paths
- bounded message/receive/send protocols

These are exactly the sorts of problems where exhaustive or near-exhaustive exploration over small domains is more valuable than random generation. However, the current small-world path does not model worlds explicitly and therefore cannot explore them meaningfully.

What is missing:
1. no canonical representation of an Ash test world
2. no stable enumeration semantics for bounded worlds/states
3. no way to derive worlds from structured metadata
4. no way to report counterexamples as concrete worlds rather than loop counters

## Goals

1. Define one canonical `SmallWorldState` model for runner execution.
2. Support bounded, deterministic enumeration over explicit world domains.
3. Make small-world tests executable and reproducible.
4. Preserve explicit runner controls (`--max-worlds`) while making them refer to real explored worlds.
5. Reuse the same per-test result substrate as authored and property tests.

## Non-Goals

This design does not attempt to:
- solve arbitrary large-state exploration
- replace property testing
- provide symbolic execution or model checking in this phase
- enumerate every possible Ash semantic configuration automatically
- commit to one final global exploration strategy for every subsystem

## Design Principles

### P1. World first, execution second

A small-world test must first define or derive its world space. Only then can execution and oracles be meaningful.

### P2. Explicit boundedness

A world exploration substrate must always be bounded by:
- finite domains
- explicit depth/size limits
- explicit world-count limits

### P3. Deterministic ordering

World enumeration should be stable and deterministic so failures are reproducible and comparable across runs.

### P4. Real counterexamples

If a small-world test fails, the report should name the world that failed, not merely “iteration 3”.

### P5. Build from the best bounded domains first

Not every Ash subsystem needs to be world-explored at once. Start with the domains that are naturally small and structurally well-defined.

## Core Design

## 1. Canonical small-world state model

Introduce a runner-internal world model:

```text
SmallWorldState {
  id,
  schema_version,
  world_kind,
  bindings,
  capabilities,
  roles,
  policies,
  obligations,
  mailbox,
  control_state,
  resource_state,
  transition_trace,
  oracle_refs,
}
```

TASK-1010 freezes this as the semantic contract for Phase 76B. Implementation may
choose Rust-specific names, but `--max-worlds` must eventually bound enumeration over
real `SmallWorldState` values rather than bounded reruns of one authored body.

### World kinds

Suggested initial kinds:
- value-domain world
- role/capability world
- obligation lifecycle world
- policy-context world
- protocol/message world

Not every test needs every field populated. The world model should be sparse enough to support different exploration families through one shared representation.

## 2. World Domains

A small-world test needs one or more finite domains.

### Example domain forms

```text
FiniteDomain<T> =
  - explicit_values([v1, v2, ...])
  - bounded_int(range)
  - booleans
  - bounded_list(element_domain, max_len)
  - bounded_product([domain_a, domain_b, ...])
  - bounded_state_machine(states, transitions, max_depth)
```

### Initial recommended domains

1. Bool domain
2. Small Int domain
3. Tiny list domain
4. Small enum/nominal domain where variants are known
5. Obligation lifecycle states
6. Role/capability inclusion sets

### Stable domain descriptor

The runner-facing domain descriptor should be explicit and deterministic:

```text
SmallWorldDomain {
  id,
  domain_kind,          // explicit_values | bounded_int | bool | list | product | state_machine
  value_type,
  bounds,
  ordering_policy,
  source,               // authored | obligation_metadata | policy_metadata | contract_metadata
  unsupported_reason,
}
```

If a domain has no finite descriptor, the runner may record an unsupported planning row
but must not claim true small-world execution for that source.

## 3. Enumeration Substrate

The runner needs one explicit world enumerator:

```text
enumerate_worlds(domain_spec, max_worlds) -> Vec<SmallWorldState>
```

### Requirements

- deterministic ordering
- stable truncation when `max_worlds` is hit
- enough metadata to reconstruct which domain choices produced the world
- stable `world_index` assignment starting at 1 for reported results
- stable `world_id` or digest derived from the canonical world snapshot

The enumerator must be pure with respect to the same domain descriptor and seed. If a
future strategy uses random sampling, that belongs to property testing unless the sampled
set is first materialized as an explicit finite world list.

### Reporting

When a failure happens, the runner should report:
- world index
- world id
- world data summary
- repro metadata

## 4. Oracle Model

Small-world execution needs more than repetition; it needs a well-defined oracle per explored world.

```text
SmallWorldOracle {
  kind,
  expected,
}
```

Possible oracle kinds:
- output_equals
- output_matches
- policy_terminal_equals
- obligation_state_equals
- role_capability_set_equals
- control_state_equals
- execution_rejects

## 5. First Recommended Exploration Targets

### 5.1 Obligation lifecycle worlds

Best first candidate.

Why:
- finite set of meaningful lifecycle states
- clear success/failure criteria
- already conceptually close to state-machine testing

Canonical worlds might cover:
- introduced but not discharged
- introduced then discharged
- double discharge attempt
- branch-specific discharge outcomes

### 5.2 Role/capability worlds

Next best candidate.

Why:
- finite combinations of a small number of capabilities/roles
- useful for authority checks and role composition validation

### 5.3 Policy-context worlds

Then:
- small explicit subject/resource/context fields
- bounded allow/deny/approval/transform spaces

### 5.4 Protocol/message worlds

Later:
- tiny mailbox contents
- small receive/send cases
- bounded protocol traces

## 6. Sources of World Definitions

Worlds should come from explicit bounded sources.

Recommended sources, in order:
1. explicit authored small-world metadata
2. structured metadata extraction from obligations / roles / policies
3. future generated domains from stable type/contract metadata

Do not infer rich world spaces from raw source strings if a structured source is available.

## 7. Execution Model

For each enumerated world:
1. materialize execution/setup context
2. apply world bindings/state
3. execute target test/workflow/case
4. evaluate oracle
5. emit canonical result with world metadata

The result model should preserve:
- `world_index`
- `world_summary`
- `world_repro`

## 8. Reproducibility

Every failing small-world case should emit:

```text
SmallWorldRepro {
  runner_schema_version,
  source_artifact_id,
  check_summary_id,
  case_id,
  seed,
  case_index,
  world_kind,
  world_index,
  world_id,
  world_snapshot,
  transition_trace,
  oracle_snapshot,
  target,
  replay_command,
}
```

This is stronger than today’s runner-level counter because it records the explored world itself.

The repro artifact should use the same `ReproArtifact` family as synthesized contract,
policy, and obligation cases from DESIGN-022. A failure report that contains only a loop
counter is not sufficient for Phase 76B completion.

## 8.1 Stable small-world model references

The cross-design handoff from DESIGN-022 uses `SmallWorldModelRef`:

```text
SmallWorldModelRef {
  id,
  model_kind,           // obligation_lifecycle | role_capability | policy_context | protocol_message
  domain_refs,
  transition_refs,
  oracle_refs,
  max_depth,
  max_worlds_default,
}
```

For the first implementation slice, an obligation lifecycle model is the preferred
target because it has a naturally finite transition space and a clear terminal oracle.
Policy-context models must wait until policy metadata exposes bounded input domains.

## 9. Relationship to Property Testing

Property testing and small-world testing should stay distinct.

Property testing
- samples generated inputs
- usually stochastic/seeded
- good for broad behavioral fuzzing over bounded generators

Small-world testing
- explores explicit finite worlds/states
- deterministic and enumerative
- good for protocols, lifecycle models, policy/role/obligation combinations

They may share infrastructure, but they should not be collapsed into one concept.

## 10. Suggested Implementation Order

### Stage A: World model + explicit finite domains

Land:
- `SmallWorldState` representation
- deterministic enumerator for explicit finite domains
- result/repro metadata

### Stage B: Obligation lifecycle exploration

Land first executable real small-world slice here.

### Stage C: Role/capability exploration

Land bounded inclusion/composition worlds.

### Stage D: Policy-context exploration

Land bounded policy worlds once policy-domain shape is stabilized.

## Open Questions

1. Should world definitions live entirely runner-side, or can Ash source files declare finite domains directly?
2. How much of a world should be visible in JSON output vs. summarized?
3. Which state carriers should be first-class in the world model: mailbox, obligations, role authority, policy context, control state?
4. Should obligation and policy small-world exploration share a common finite-domain DSL later?

## Acceptance Criteria

This design note is realized when:
1. the runner has a real world model and deterministic enumerator
2. `--max-worlds` bounds actual explored worlds, not just reruns
3. at least one real obligation or role/capability exploration slice is implemented end-to-end
4. failing small-world cases report the actual world, not merely a loop counter

## Recommendation

Start with obligation lifecycle worlds. They are the smallest, clearest, and most naturally finite Ash domain for genuine small-world exploration. Once that substrate is stable, extend it to role/capability worlds and then policy-context worlds.
