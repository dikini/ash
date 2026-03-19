# Type-to-Runtime Contract

## Status

Canonical reference for TASK-163.

## Purpose

This document freezes the handoff from type checking and static verification into runtime and
verification-time execution for the stabilized convergence features.

It defines:

1. what information the type layer must provide or validate before runtime,
2. which runtime/verification components consume that information,
3. which states must be rejected before execution, and
4. which failures remain runtime concerns rather than type-checking failures.

## Scope

This handoff covers:

- workflow effect and obligation outputs
- capability/runtime verification prerequisites
- workflow `decide` / policy-decision expectations
- `receive` typing prerequisites
- ADT constructor, pattern, and exhaustiveness prerequisites

Out of scope:

- parser and lowering shape
- REPL/CLI rendering details
- transport- or provider-specific runtime internals

## Required Type-Layer Outputs

The type layer must establish, directly or indirectly, the following runtime-relevant facts:

| Required fact | Why runtime needs it |
|---|---|
| workflow effect classification | runtime verification must compare workflow effect against runtime effect ceilings |
| required input/output capability sets | runtime verification must check capability availability and directionality |
| obligation requirements and discharge shape | runtime verification must compare workflow obligations against runtime obligations |
| named policy references resolved to canonical lowered-policy identities | workflow `decide` and capability verification must not consume anonymous or unresolved policy expressions |
| decision-domain restrictions for workflow `decide` | workflow-level runtime `decide` only admits `Permit` / `Deny` outcomes |
| `receive` guard well-typed as `Bool` | runtime must not execute untyped guard logic |
| ADT constructor and variant-pattern resolution against canonical enum metadata | runtime pattern matching and constructor evaluation need one shared enum model |
| `match` exhaustiveness success for required exhaustive sites | runtime must not rely on impossible fallback semantics for exhaustive ADT matches |

The current Rust surface for these outputs spans:

- `ash_typeck::TypeCheckResult`
- `ash_typeck::effect::infer_effect`
- `ash_typeck::runtime_verification::{VerificationResult, RuntimeContext, PolicyDecisionType}`
- ADT metadata and checks in `ash_typeck::{type_env, check_pattern, exhaustiveness}`

These existing APIs are implementation surfaces, not alternate contracts. Downstream convergence
tasks may reshape them, but they must preserve the facts listed above.

## Required Runtime Consumers

### Runtime Verification

Runtime verification consumes type-derived workflow facts plus runtime context:

- capability requirements
- effect ceiling requirements
- obligation requirements
- named lowered-policy availability
- policy decision domain compatibility

The canonical verification-time runtime context is the one described by `SPEC-018`:

- obligations
- policy registry
- capability registry
- mailbox registry
- scheduler
- approval queue
- provenance sink
- max effect
- role

### Workflow Runtime

The workflow runtime consumes the following already-typed assumptions:

- `decide` subjects are of the policy’s required subject type
- `receive` guards are boolean
- constructor expressions have fields compatible with the resolved constructor
- variant patterns refer to real constructors on the resolved enum type
- exhaustive `match` sites do not need synthetic runtime fallback behavior

## Rejected States at the Type-to-Runtime Boundary

The following states must be rejected before runtime execution:

- unresolved named policy references for workflow `decide`
- workflow `decide` sites whose resolved policy can lower to outcomes outside `{Permit, Deny}`
- non-boolean `receive` guards
- unknown ADT constructors or variant patterns
- constructor field mismatches against resolved enum metadata
- non-exhaustive ADT `match` where the contract requires exhaustiveness
- workflow effect requirements above the declared or verified maximum permitted effect

These are boundary failures: runtime must not be asked to “figure them out” from raw surface
syntax or unvalidated metadata.

## Runtime-Time Rejections

The following remain runtime or verification-time concerns, not type-checking failures:

- missing runtime capabilities
- non-readable / non-writable / non-sendable / non-receivable providers
- missing runtime obligations or role mismatches
- runtime policy denials, approval requirements, or transformations
- absent mailbox messages, timeout behavior, or scheduler outcomes
- provider-level input/output type mismatches caused by actual runtime values

Type checking proves or constrains shapes; runtime enforces availability, environment, and actual
execution outcomes.

## Policy and Verification Contract

The type-to-runtime boundary for policies is:

1. source policy expressions are validated as policy expressions,
2. named bindings are required to lower to one canonical lowered policy identity,
3. workflow `decide` references only that lowered policy name,
4. runtime consumes normalized policy decisions rather than source `PolicyExpr`.

Workflow runtime may observe only `Permit` or `Deny` at workflow `decide` sites.
Capability-verification runtime may additionally consume approval or transformation outcomes.

## ADT Contract

The type-to-runtime boundary for ADTs is:

1. source declarations remain anchored in canonical `TypeDef` / `TypeBody` / `VariantDef`
   metadata,
2. constructor typing and variant-pattern typing use the same resolved enum metadata,
3. runtime constructor evaluation and pattern matching operate over the same constructor names and
   named fields,
4. exhaustiveness reasoning is done over constructors of the resolved enum type, not synthetic
   tags.

The runtime therefore consumes resolved enum metadata and variant names/fields, not ad hoc
record-tag encodings as a contract surface.

## Convergence Notes

Current implementation gaps that must not be mistaken for contract:

- `TypeCheckResult` currently does not surface the full set of inferred expression types needed by
  REPL `:type` behavior.
- runtime-verification code still uses legacy `PolicyDecisionType::RequiresApproval` naming while
  the stabilized spec vocabulary prefers normalized `PolicyDecision`.
- implementation APIs may store ADT approximations internally, but runtime-visible behavior must
  still conform to the canonical enum metadata story.
