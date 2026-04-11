# DESIGN-024: Property Generation Substrate

## Status: Draft

## Overview

Design and implement the substrate required to move Ash property testing from bounded reruns of authored test bodies into true generated-input property testing.

This note is a direct follow-up to Phase 76 and complements:
- [DESIGN-021: Ash Test Runner V1](DESIGN-021-ASH-TEST-RUNNER-V1.md)
- [DESIGN-022: Synthesized Contract / Policy / Obligation Cases](DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)

The current `ash test` runner supports:
- `property` as an explicit test kind
- `--seed` and `--max-cases` controls
- bounded repeated execution and reporting of case indices

But the current implementation does not yet generate true inputs. It reruns the same authored body in a bounded loop. This note defines the missing generator model, domain model, sampling contract, and reproducibility substrate needed for real property testing.

## Problem Statement

Property testing only becomes meaningful when the runner can execute the same behavioral claim across a varied, bounded, reproducible stream of inputs. The current runner can count iterations, but not generate distinct values or bind them into the test case in a principled way.

What is missing:
1. no canonical representation of a generated property case
2. no generator/domain model for Ash values and higher-level semantic inputs
3. no stable way to derive generators from type and contract metadata
4. no shrinking/minimization contract, even at a simple future-oriented level
5. no semantic repro artifact beyond case index and seed

Without those pieces, `property` remains a bounded repetition mode rather than a true generated-input testing substrate.

## Goals

1. Define one canonical internal model for generated property cases.
2. Support deterministic, seed-driven generation of concrete test inputs.
3. Support bounded generation with stable size/case controls.
4. Make failures reproducible from semantic case data, not only loop counters.
5. Create a generation substrate that can later be reused by synthesized contract/policy/obligation cases.

## Non-Goals

This design does not attempt to:
- land full shrinking in the first implementation slice
- cover every Ash type/domain immediately
- replace authored test declarations
- solve symbolic execution or theorem-backed case generation
- force all generation logic into the Ash surface language immediately

## Design Principles

### P1. Generator model before generator syntax

Do not start by inventing complex authored syntax. First define the internal generator/domain substrate that the runner can execute deterministically.

### P2. Deterministic seeds are mandatory

Every generated property run must be reproducible from:
- seed
- generator/domain spec
- case index
- generated case data

### P3. Type and contract metadata should guide generation, but not be conflated with it

Type information and contracts are valuable generator inputs, but they are not themselves the whole generation model.

### P4. Start with small, honest domains

Initial property generation should target:
- Bool
- small Int
- small String
- small lists/records of bounded size
- simple nominal values when constructors are known and bounded

### P5. Case data is part of the result contract

A failing property test should report the generated input case, not merely “case 17”.

## Core Design

## 1. Canonical PropertyCase Model

Introduce a runner-internal generated case model:

```text
PropertyCase {
  id,
  case_index,
  seed,
  generator_spec,
  bindings,
  size,
  notes,
}
```

### Generator spec

```text
PropertyGeneratorSpec {
  target,
  input_domains,
  derivation_source,   // explicit | type | contract | mixed
  max_size,
}
```

### Bindings

```text
GeneratedBindings {
  values,
  named_inputs,
}
```

These bindings are what the runner injects into the property case execution environment.

## 2. Generator Domain Model

The runner needs a composable finite/bounded domain vocabulary.

### Initial domain forms

```text
ValueDomain =
  - bools
  - bounded_int(min, max)
  - bounded_string(max_len, alphabet)
  - option(domain)
  - list(domain, max_len)
  - record(fields)
  - nominal(constructors, bounded_args)
```

### Why bounded domains first

Property generation in Ash should initially prioritize:
- determinism
- explainability
- reproducibility
- stable execution time

over maximum diversity.

## 3. Generation Pipeline

The runner should have one explicit case-generation pipeline:

```text
generate_property_cases(spec, seed, max_cases) -> Vec<PropertyCase>
```

### Inputs to generation

A property case may derive its generator spec from:
1. explicit authored generator metadata
2. type metadata
3. lowered contract metadata
4. mixed type + contract narrowing

### Recommendation for first implementation

Support explicit and type-based generation first.
Then add contract-guided narrowing.

## 4. Relationship to Contracts

Contracts are not the whole generator substrate, but they are extremely valuable guidance.

### Requires clauses

Use preconditions to:
- filter generated candidates
- define positive domains
- derive negative boundary cases for dedicated contract tests

### Ensures clauses

Use postconditions as an oracle source, not a generator source.

This keeps roles separate:
- generation chooses inputs
- contracts constrain inputs and judge outputs

## 5. Execution Model

A generated property test should execute as:
1. derive/gather generator spec
2. generate `max_cases` cases deterministically from seed
3. inject bindings for each case
4. execute property body
5. evaluate oracle/assertions
6. stop on first failure unless configured otherwise
7. emit `PropertyCase` repro data

## 6. Result and Repro Model

A failing property result should carry:

```text
PropertyFailureRepro {
  seed,
  case_index,
  bindings,
  generator_spec,
  oracle,
}
```

This is stronger than today’s runner-level reporting because it captures the actual generated case.

## 7. Shrinking / Minimization Boundary

Do not block initial implementation on full shrinking.

But define the future hook now:

```text
shrink_case(generator_spec, failing_case) -> Iterator<PropertyCase>
```

Recommended initial stance:
- first implementation: no shrinking, just report the case
- second implementation: manual/simple structural shrinkers for bounded ints, bools, lists

## 8. Sources of Generator Specs

### 8.1 Explicit authored metadata

Example conceptual direction:
- generator metadata declares bounded domains for inputs

This is the most controlled, least ambiguous starting point.

### 8.2 Type-driven generation

The runner should later be able to derive bounded generators from known Ash types.

Examples:
- `Bool` -> `{true, false}` or sampled bools
- `Int` -> bounded small integer range
- `Option<T>` -> `None` plus bounded `Some(T)`

### 8.3 Contract-guided generation

Once lowered contracts are exposed cleanly:
- `requires x > 0` can narrow the generated Int domain
- `requires len(xs) < 3` can narrow the list size domain

This is the ideal bridge between property testing and synthesized contract execution.

## 9. Recommended First Implementation Slice

### Stage A: Internal case model + bounded primitive generators

Land:
- `PropertyCase`
- `PropertyGeneratorSpec`
- Bool / small Int / small String domains
- deterministic seed-driven generation
- semantic repro output

### Stage B: Binding injection into authored property tests

Land:
- explicit authored property cases that consume generated bindings
- property result reporting with actual case data

### Stage C: Type-driven domain derivation

Land:
- derive simple bounded generators from primitive/container types

### Stage D: Contract-guided narrowing

Land:
- use lowered `requires` metadata to constrain generated cases

This order is recommended because it turns property testing into a real generation substrate without waiting on the full contract/policy/obligation synthesis stack.

## 10. Open Questions

1. Should explicit generator metadata live in test headers, dedicated property blocks, or separate fixture files?
2. How should generated bindings map into authored Ash test code: implicit names, explicit parameters, or a designated environment object?
3. Should contract-driven narrowing happen before or after basic type-based case generation?
4. How much generated-case data should be surfaced in human output versus JSON only?

## Acceptance Criteria

This design note is realized when:
1. `property` execution uses generated cases, not just bounded reruns
2. the runner can emit semantic repro data for failing property cases
3. seed + case generation are deterministic and reproducible
4. the first bounded domains (Bool/Int/String/etc.) are implemented end-to-end
5. the substrate is usable later by synthesized contract/policy/obligation execution

## Recommendation

Start with a narrow generation core: primitive bounded domains plus deterministic case generation and semantic repro data. Then layer explicit authored property binding injection, then type-driven generation, then contract-guided narrowing. This keeps the substrate honest and reusable instead of prematurely embedding too much semantics into surface syntax.
