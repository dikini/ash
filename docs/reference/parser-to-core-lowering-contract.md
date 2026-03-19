# Parser-to-Core Lowering Contract

## Status

Canonical reference for TASK-162.

## Purpose

This document freezes the lowering handoff from the parsed surface forms covered by TASK-161 into
the canonical core forms required by the stabilized Ash contract.

It defines:

1. which parsed surface nodes are eligible for lowering,
2. which canonical core forms they lower into,
3. which invalid combinations must be rejected before or during lowering, and
4. which information must be preserved across the boundary.

This document is the lowering authority for downstream convergence tasks. Existing lowering
placeholders and fallback behavior are implementation debt, not alternate semantics.

## Scope

This handoff covers:

- workflow `check`, `decide`, and `receive`
- policy bindings and policy expressions
- ADT declarations, constructor expressions, variant patterns, `match`, and `if let`

Out of scope:

- parser acceptance rules
- type-checking judgments
- runtime execution behavior

## Canonical Core Targets

The required lowering targets come from the canonical core contracts in the spec set:

- `SPEC-001` core `Workflow::{Decide, Check, Receive, Let, If, Seq, Done, ...}`
- `SPEC-001` `ReceiveMode`, `ReceiveArm`, and `ReceivePattern`
- `SPEC-001` core `Pattern`
- `SPEC-006` / `SPEC-007` canonical `CorePolicy { name, params, graph }`
- `SPEC-020` canonical ADT source model and the constructor / pattern rules derived from it

Lowering may elaborate the parsed surface tree, but it must preserve the canonical meaning listed
here instead of substituting placeholders, dummy obligations, or default policy names.

## Surface-to-Core Mappings

### Workflow Forms

| Parsed surface node | Canonical lowering target | Preservation requirements |
|---|---|---|
| `surface::Workflow::Decide { expr, policy: Some(name), then_branch, else_branch: None, .. }` | `core::Workflow::Decide { expr: lower_expr(expr), policy: name, continuation: lower_workflow(then_branch) }` | Preserve the explicit policy name; do not invent `"default"` or any fallback binding. |
| `surface::Workflow::Check { target: CheckTarget::Obligation(obl), continuation, .. }` | `core::Workflow::Check { obligation: lower_obligation(obl), continuation: lower_workflow(cont) }` | Preserve the obligation identity and continuation order. |
| `surface::Workflow::Receive { mode, arms, is_control, .. }` | `core::Workflow::Receive { mode, arms: lower_receive_arms(arms), control: is_control }` | Preserve receive mode, arm order, guard expressions, body order, and control-vs-stream selection. |

### Receive-Arm Forms

| Parsed surface node | Canonical lowering target |
|---|---|
| `surface::StreamPattern::Binding { capability, channel, pattern }` | `core::ReceivePattern::Stream { capability, channel, pattern: lower_pattern(pattern) }` |
| `surface::StreamPattern::Literal(value)` | `core::ReceivePattern::Literal(lower_literal(value))` |
| `surface::StreamPattern::Wildcard` | `core::ReceivePattern::Wildcard` |
| `surface::ReceiveArm { pattern, guard, body, .. }` | `core::ReceiveArm { pattern: lower_receive_pattern(pattern), guard: guard.map(lower_expr), body: lower_workflow(body) }` |

### Policy Forms

| Parsed surface node | Canonical lowering target | Notes |
|---|---|---|
| `surface::PolicyDef` | normalized policy schema metadata used to compile bindings into `CorePolicy` | Schema definitions are not general runtime values. |
| closed named policy binding | one `CorePolicy { name, params, graph }` | The binding name is the workflow/runtime reference identity. |
| `surface::PolicyExpr` tree | normalized `PolicyGraph` inside one named `CorePolicy` | Combinator structure may be normalized, but meaning must be preserved. |

Workflow `decide` lowers only against named lowered policies. Inline policy expressions are not a
workflow-level core form.

### ADT Forms

| Parsed surface node | Canonical lowering target | Notes |
|---|---|---|
| `parse_type_def::TypeDef` | canonical source `TypeDef` / `TypeBody` / `TypeExpr` metadata consumed by later phases | Lowering preserves the source declaration model; it does not replace it with a second spec-level shape. |
| `surface::Expr::Constructor { name, fields, .. }` | core constructor expression preserving constructor name plus lowered named fields | Constructor resolution happens against the canonical enum metadata. |
| `surface::Pattern::Variant { name, fields }` | core variant pattern preserving constructor name plus lowered field patterns | No synthetic `__variant` tags are introduced at the contract level. |
| `surface::Expr::Match { scrutinee, arms, .. }` | core `Expr::Match` with lowered scrutinee, patterns, and bodies | Arm order is preserved. |
| `surface::Expr::IfLet { pattern, expr, then_branch, else_branch, .. }` | either preserved as core `Expr::IfLet` or desugared into an equivalent core `Match` with wildcard fallback | Either choice is valid if semantics are preserved exactly. |

## Lowering-Time Rejections

Lowering must reject semantically invalid combinations that are still syntactically parsable.

Required lowering-time rejections include:

- `surface::Workflow::Decide` with `policy: None`
- any parsed legacy `decide` shape carrying an else-branch outside the stabilized contract
- `surface::Workflow::Check` with `CheckTarget::Policy(_)`
- any `receive` form whose parsed arm shape cannot map to canonical stream, literal, or wildcard
  receive patterns
- policy bindings whose final `surface::PolicyExpr` is not closed enough to compile to one named
  `CorePolicy`

Lowering rejection is appropriate when syntax is valid but the parsed surface tree cannot be
translated into a canonical core form without inventing semantics.

## Preservation Rules

Lowering must preserve:

- explicit policy names used by workflow `decide`
- source `receive` mode and arm order
- control-vs-stream receive selection
- ADT constructor names and named fields
- `match` arm order and `if let` branch meaning

Lowering may normalize:

- policy combinator trees into one `PolicyGraph`
- source type declarations into internal metadata derived from canonical `TypeDef`
- `if let` into equivalent `match` when branch semantics are unchanged

## Lowering vs Later-Phase Boundary

### Lowering Owns

- choosing the canonical core node for each eligible surface node
- rejecting parsed surface trees that have no canonical lowering target
- preserving source information needed by type checking and runtime layers

### Later Phases Own

The following are not lowering failures:

- type compatibility of `decide` subjects, `receive` guards, constructor fields, or `match` arms
- workflow-level `decide` outcome-domain restrictions; those are enforced by the type layer after a
  named policy binding is resolved
- ADT constructor-to-variant and pattern-to-enum relation failures; those are enforced by the type
  layer against the resolved enum metadata
- exhaustiveness checking for `match`
- source scheduling modifier behavior and mailbox probe order; those are runtime concerns defined
  by the receive contract, not lowering targets
- runtime evaluation of `CorePolicy`
- runtime mailbox polling or policy enforcement behavior
- user-visible CLI or REPL output
