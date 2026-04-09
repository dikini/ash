# Sequential Workflow Language Design

## Goal

Remove the `par` form from the Ash language so a single workflow denotes only sequential process execution, making sequential composition the sole workflow-internal control-flow composition contract.

## Problem

The current language admits `par` as a first-class workflow form across surface syntax, core IR, typing, and operational semantics. That makes concurrency part of the meaning of a single workflow term and forces the semantics to preserve helper-owned interleaving and aggregation boundaries inside one workflow.

That conflicts with the desired language model:

- a single workflow should be sequential and therefore strongly deterministic for fixed runtime/helper inputs;
- concurrency should be modeled as multiple workflows communicating through existing process/runtime mechanisms rather than as a structural subterm inside one workflow;
- the specification should express that design directly instead of preserving `Par` as a historical semantic artifact.

## Chosen Design

Remove `par` completely from the current language contract.

Normative consequences:

- surface syntax no longer recognizes a `par { ... }` workflow form;
- the canonical core IR no longer includes `Workflow::Par`;
- type-system and effect-composition rules no longer mention `Par`;
- big-step and small-step semantics no longer define `Par` progress, aggregation, or bounded nondeterminism for workflow-internal concurrency;
- implementation conformance and canonical semantics corpus documents no longer treat `Par` as an active language feature.

Old source files that still use `par` will fail at the normal parser boundary once the keyword and production are removed. No compatibility shim or sequential desugaring is introduced.

## Why This Option

This is the only option that matches the requested semantics cleanly.

- Keeping an internal `Par` while removing only the surface form would leave concurrency as a hidden core-language feature.
- Desugaring `par` into `seq` would silently change the meaning of existing programs while pretending they still express concurrency.
- Full removal lets the spec state one simple truth: one workflow is sequential; concurrent systems are built from multiple communicating workflows.

## Concurrency Model After Removal

Concurrent behavior remains expressible, but not as one workflow term containing parallel branches.

The intended model is:

- one workflow instance executes sequentially;
- multiple workflow instances may still run concurrently under the runtime;
- coordination happens through message-passing and supervision mechanisms such as `spawn`, `send`, `receive`, `yield`, proxy workflows, and retained completion/control surfaces.

This keeps concurrency at the process boundary instead of embedding it as a composition form within a single workflow AST.

## Migration

The migration is intentionally strict.

- Historical records remain untouched: changelog entries, completed task files, and older design/reference material that discussed `Par` remain as historical evidence.
- Current normative docs become the sole source of truth and must no longer list `Par` as part of the language.
- Examples, tests, and tutorial material that currently demonstrate `par` must be rewritten either as sequential workflows or as communicating multi-workflow/process examples.
- Generic parser rejection is acceptable for legacy `par` source; no custom deprecation diagnostic is required.

## Scope

This phase should include:

- spec updates in `docs/spec` removing `par` from normative language definition;
- parser, lowering, AST, type checking, interpreter, REPL/visualization, and engine changes that remove `Par`;
- example, tutorial, and workflow fixture updates;
- conformance/reference updates that remove active `Par` cases from the current language corpus while preserving historical documents as history;
- plan/task/changelog updates required by project policy.

This phase should not:

- erase or rewrite historical completed work records merely because they mention `Par`;
- redesign unrelated runtime concurrency primitives such as spawned workflow control or mailbox behavior;
- add a compatibility layer that preserves `par` source programs.

## Success Criteria

- no current normative spec presents `Par` as part of the language;
- no parser/lowering/typechecker/interpreter path accepts or executes `par`;
- the core AST and public workflow contracts expose only sequential workflow composition internally;
- examples and tests express concurrency through communicating workflows rather than `par`;
- the remaining language/spec corpus supports the claim that a single workflow is sequential.
