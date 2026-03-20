# Runtime Verification Input Contract

## Status

Canonical follow-up reference for the capability-versus-obligation verification split.

## Purpose

This document freezes one specific boundary in the runtime verification model:

- workflow-declared capability use,
- obligation-backed runtime requirements,
- and aggregate verification inputs

must remain distinct.

The goal is to prevent aggregate verification from collapsing:

- "the workflow uses capability `X`"

into:

- "runtime obligations must justify capability `X`"

unless an explicit contract says that mapping exists.

## Core Distinction

Runtime verification consumes at least two different requirement classes:

### 1. Workflow Capability Declarations

These come from workflow structure and type/lowering outputs.

Examples:

- observes `sensor:temp`
- receives `queue:orders`
- sets `hvac:target`
- sends `alert:critical`

These declarations answer:

- what runtime capabilities must be available,
- in what direction,
- and with what provider/runtime shape.

They are checked against:

- capability registry,
- mailbox registry,
- provider directionality,
- and runtime effect ceilings.

### 2. Obligation-Backed Runtime Requirements

These come from runtime obligations, obligation contracts, role requirements, or other explicit
runtime verification inputs that govern whether execution is justified or required.

They answer:

- which obligations must be present,
- which roles or obligation sets must hold,
- and which required capabilities are justified by those obligations.

They are checked against:

- runtime obligations,
- role context,
- and obligation-satisfaction metadata.

## Non-Equivalence Rule

Workflow capability declarations are not, by themselves, obligation requirements.

Therefore:

- `WorkflowCapabilities` must not be treated as the canonical source of
  obligation-backed runtime requirements,
- aggregate runtime verification must accept or derive obligation requirements through a separate
  explicit input path,
- any temporary bridging logic must be treated as implementation debt, not the contract.

## Aggregate Verification Contract

Aggregate runtime verification should conceptually consume:

- workflow capability declarations,
- obligation-backed runtime requirements,
- runtime context,
- workflow effect requirements,
- policy availability / policy-outcome compatibility inputs.

In contract form:

- capability availability checks consume workflow capability declarations,
- obligation checks consume obligation-backed runtime requirements,
- aggregate verification combines those results but must not silently substitute one for the other.

## Allowed Temporary Bridging

If current implementation structure does not yet expose separate obligation-backed requirement
inputs, a temporary bridge may exist.

That bridge must be understood as:

- a stopgap to restore enforcement,
- not a semantic equation between capability declarations and obligation requirements,
- and something that later runtime convergence work must replace.

## Required Follow-Up

Before runtime execution and policy-outcome convergence are considered complete:

- the runtime/type boundary must expose a distinct aggregate-verification input for
  obligation-backed requirements,
- aggregate verification must stop deriving those requirements implicitly from
  `WorkflowCapabilities`,
- and tests must prove the two requirement classes can vary independently.

## Relationship to Other Contracts

This document refines:

- [Type-to-Runtime Contract](type-to-runtime-contract.md)

It does not replace it.

`type-to-runtime-contract.md` already distinguishes:

- required input/output capability sets
- obligation requirements and discharge shape

This document makes that separation operational for aggregate runtime verification work.
