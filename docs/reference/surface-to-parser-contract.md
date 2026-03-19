# Surface-to-Parser Contract

## Status

Canonical reference for TASK-161.

## Purpose

This document freezes the parser handoff for the stabilized convergence features introduced by
TASK-156 through TASK-160.

It answers four questions:

1. Which surface forms are accepted?
2. Which parsed surface AST node must be produced?
3. Which malformed forms must be rejected by the parser?
4. Which failures are owned by lowering, type checking, or runtime layers instead?

This document is the parser authority for downstream convergence tasks. When current parser code
still exposes a legacy shape, this document defines the required target shape rather than
ratifying the legacy behavior.

## Scope

This handoff covers:

- workflow forms: `check`, `decide`, `receive`
- policy declarations and policy-expression syntax
- ADT declarations, constructor expressions, variant patterns, `match`, and `if let`

Out of scope:

- lowering meaning
- type-checking validity
- runtime behavior

## Required Parser Outputs

The parser boundary produces surface AST values from `ash_parser::surface` plus ADT declaration
values from `ash_parser::parse_type_def`.

The required output vocabulary for the stabilized features is:

- `surface::Workflow::Decide`
- `surface::Workflow::Check`
- `surface::Workflow::Receive`
- `surface::ReceiveMode`
- `surface::ReceiveArm`
- `surface::StreamPattern`
- `surface::PolicyDef`
- `surface::PolicyExpr`
- `surface::Expr::Match`
- `surface::Expr::IfLet`
- `surface::Pattern::Variant`
- `parse_type_def::TypeDef`
- `parse_type_def::TypeBody`
- `parse_type_def::VariantDef`
- `parse_type_def::TypeExpr`

## Accepted Forms

### Workflow Forms

| Surface form | Required parser output | Notes |
|---|---|---|
| `decide { expr } under policy_name then workflow` | `surface::Workflow::Decide { expr, policy: Some(policy_name), then_branch, else_branch, .. }` | The parser output must preserve the explicit named policy. |
| `check obligation_ref` | `surface::Workflow::Check { target: surface::CheckTarget::Obligation(...), continuation, .. }` | `check` is obligation-only at the contract level. |
| `receive { cap:chan as pattern => workflow, ... }` | `surface::Workflow::Receive { mode: NonBlocking, arms, is_control: false, .. }` | Each arm uses `surface::StreamPattern::Binding`. |
| `receive wait { ... }` | `surface::Workflow::Receive { mode: Blocking(None), .. }` | Blocking forever. |
| `receive wait 30s { ... }` | `surface::Workflow::Receive { mode: Blocking(Some(duration)), .. }` | Timeout must be parsed structurally, not preserved as raw text. |
| `receive control { "shutdown" => workflow, _ => workflow }` | `surface::Workflow::Receive { is_control: true, arms, .. }` | Control literal arms use `surface::StreamPattern::Literal`; `_` uses `surface::StreamPattern::Wildcard`. |

### Policy Forms

| Surface form | Required parser output | Notes |
|---|---|---|
| `policy Name { field: Type, ... }` | `surface::Definition::Policy(surface::PolicyDef { ... })` | Schema declaration form. |
| `policy Name { ... } where { expr }` | `surface::PolicyDef { where_clause: Some(expr), .. }` | The `where` clause remains surface expression syntax. |
| closed policy-expression syntax from SPEC-006/007 | `surface::PolicyExpr` tree | Includes named references, instance forms, and combinator expressions. |

Named policy bindings are part of the stabilized surface contract, but this handoff treats them
as parser-owned only to the extent that the parser must preserve a named policy expression tree
for lowering. Binding-to-core meaning belongs to the lowering contract in TASK-162.

### ADT Forms

| Surface form | Required parser output | Notes |
|---|---|---|
| `type Option<T> = Some { value: T } | None;` | `parse_type_def::TypeDef { body: TypeBody::Enum(...), .. }` | Enum declarations use the canonical `TypeDef` model. |
| `type Point = { x: Int, y: Int };` | `TypeDef { body: TypeBody::Struct(...), .. }` | Struct declarations are source-level record forms. |
| `type Alias = (Int, String);` | `TypeDef { body: TypeBody::Alias(TypeExpr::Tuple(...)), .. }` | Alias declarations preserve source type expressions. |
| `Some { value: x }` in expression position | expression tree with constructor syntax preserved for later lowering/type checking | Constructor resolution is not a parser responsibility. |
| `Some { value: x }` in pattern position | `surface::Pattern::Variant { name, fields: Some(...) }` | Variant patterns are constructor-name plus named fields. |
| `None` in pattern position | `surface::Pattern::Variant { name, fields: None }` | Unit variants remain variants, not plain identifiers. |
| `match expr { ... }` | `surface::Expr::Match { scrutinee, arms, .. }` | Exhaustiveness is not decided by parsing. |
| `if let pat = expr then a else b` | `surface::Expr::IfLet { pattern, expr, then_branch, else_branch, .. }` | Sugar is preserved through parsing. |

## Legal Parser Rejections

The parser must reject malformed syntax, including:

- `decide { expr } then workflow` with no `under <policy>`
- `check` targets that are not obligation references
- `receive` arms that omit `=>`
- `receive control` arms that use stream selectors such as `cap:chan as pat`
- non-control `receive` arms that use bare string-literal message selectors
- malformed duration syntax in `receive wait <duration>`
- malformed policy declarations: missing field types, malformed `where` block syntax, or broken
  combinator syntax
- malformed ADT declarations: missing `=`, malformed variant field lists, malformed generic
  parameter syntax, or missing trailing `;`
- malformed `match` or `if let` delimiters and arm separators

Parser rejection is about syntax and unambiguous syntactic category selection. It is not about
semantic validity.

## Parser vs Later-Phase Boundary

### Parser-Owned Responsibilities

- tokenization and concrete syntax acceptance/rejection
- building the required surface AST / source type-definition nodes
- preserving explicit names, arm order, `receive` mode, control-vs-stream shape, and ADT
  constructor syntax
- distinguishing syntactic categories such as obligation-reference check targets vs invalid forms

### Later-Layer Responsibilities

The following are not parser failures:

- lowering whether a parsed policy expression is closed enough to become a named lowered policy
- lowering whether a parsed `receive` form can map into canonical core IR
- type-checking whether a `decide` expression has the required subject type
- type-checking whether a `receive` guard has type `Bool`
- type-checking constructor resolution, field-type compatibility, pattern exhaustiveness, or
  `if let` branch compatibility
- runtime or verification behavior for mailbox selection, policy evaluation, REPL display, or
  variant execution
