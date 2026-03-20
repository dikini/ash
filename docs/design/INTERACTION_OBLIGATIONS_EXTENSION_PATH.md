# Interaction Obligations Extension Path

Date: 2026-03-20
Status: Design note

## Purpose

This note records the intended extension path for adding reasoner-facing or interaction-facing
obligations to Ash without overloading the existing runtime-only obligation and capability model.

It exists to preserve a clear story for future work around asymmetric execution with an external
reasoner such as an LLM or agent:

- where new interaction obligations would live,
- how they would relate to existing runtime obligations,
- what the type checker would validate,
- what runtime verification would validate,
- and what runtime execution would enforce.

This note is intentionally pre-spec. It is meant to be used when developing border concerns around
projection, advisory artifacts, gates, and commit boundaries.

## Non-Goal

This note does not redefine:

- monitor views,
- `exposes { ... }`,
- runtime observability,
- workflow capability declarations,
- or the existing runtime obligation model.

Those remain separate concerns.

## Current Obligation Story

Ash currently has a runtime-oriented obligation story.

In the existing architecture:

- the type checker validates obligation shape and workflow compatibility,
- runtime verification validates obligation compatibility against a concrete runtime context,
- and runtime execution enforces the resulting accepted behavior.

That model is appropriate for obligations tied to:

- capability use,
- effect requirements,
- runtime roles,
- and named runtime obligations.

This is the model currently used by the type/runtime contract and the aggregate runtime
verification path.

## Why Interaction Obligations Are Different

Future asymmetric execution introduces a second class of concerns.

Examples include:

- projection obligations:
  what must be included or excluded from injected reasoner context
- advisory-output obligations:
  what reasoning artifact must exist before progression
- gate obligations:
  what evaluative step must succeed before a candidate action may commit
- handoff obligations:
  what runtime↔reasoner synchronization event must occur

These are not the same as:

- capability availability,
- runtime monitorability,
- or exposed workflow views.

They should therefore not be modeled as extensions of the current runtime capability declarations
or monitor surfaces.

## Extension Principle

Future interaction obligations should be introduced as a separate obligation family.

The guiding separation is:

- runtime obligations govern execution compatibility and authoritative environment requirements
- interaction obligations govern runtime↔reasoner participation and advisory-boundary discipline

This keeps the architecture aligned with the runtime/reasoner separation rules:

- runtime-only concerns stay runtime-only
- interaction-layer concerns remain explicit rather than implicit

## Suggested Layering

The intended path is to extend existing subsystems rather than create a completely separate
"soundness engine".

### 1. Type Checker

The type checker should validate static interaction-obligation soundness.

This includes questions such as:

- is the interaction obligation well-formed?
- do referenced projection targets or advisory artifact names exist?
- are gate dependencies satisfiable in principle?
- do any interaction obligations contradict one another?
- does the workflow require an impossible runtime↔reasoner boundary sequence?

This is analogous to current static checking of capability, effect, and policy shape.

### 2. Runtime Verification

Runtime verification should validate environment-dependent interaction obligations before
execution.

This includes questions such as:

- can the required injected context actually be constructed?
- does the runtime provide the required interaction boundary support?
- are required approval, gate, or projection facilities available?
- does the runtime context satisfy the workflow's interaction-obligation requirements?

This is analogous to current runtime verification of capabilities, obligations, and policy
compatibility.

### 3. Runtime Execution

Runtime execution should enforce the obligations operationally.

This includes behavior such as:

- performing projection when required,
- requiring advisory artifacts before progression,
- blocking commitment until required gates succeed,
- and preserving the separation between advisory outputs and committed state.

Execution should not have to invent meaning that was not already validated by the type checker and
runtime verifier.

## Soundness Story

The recommended terminology is:

- **interaction obligations** for the new obligation family
- **soundness checking** for the validation performed by the type checker and runtime verifier

This avoids creating a misleading category called "soundness obligations".

The story should remain:

- interaction obligations are the subject matter
- soundness checking is the validation work performed over them

If stronger formal reasoning is added later, that should be treated as an additional validation
layer, not as the primary definition of the feature.

## Minimal Introduction Path

The lowest-risk path for future work is:

1. Define the interaction-obligation family at the design/reference level.
2. Extend the type-to-runtime and interaction contracts so those obligations have an explicit
   handoff story.
3. Extend `ash-typeck` to validate static interaction-obligation well-formedness.
4. Extend runtime verification to validate environment-dependent interaction-obligation
   requirements.
5. Extend runtime/interpreter execution to enforce the admitted obligations.
6. Only then consider stronger formal soundness checks if needed.

This keeps the extension aligned with the existing architecture and avoids speculative runtime
redesign.

## Relationship to Existing Notes

This note should be read together with:

- [Runtime-Reasoner Interaction Model](RUNTIME_REASONER_INTERACTION_MODEL.md)
- [Runtime-to-Reasoner Interaction Contract](../reference/runtime-to-reasoner-interaction-contract.md)
- [Type-to-Runtime Contract](../reference/type-to-runtime-contract.md)
- [Runtime Verification Input Contract](../reference/runtime-verification-input-contract.md)

Those documents define the current runtime/reasoner boundary and the existing verification split.
This note records the intended path for extending that system with interaction obligations later.
