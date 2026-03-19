# Runtime Observable Behavior Contract

## Status

Canonical reference for TASK-163.

## Purpose

This document freezes the runtime-visible behavior that downstream CLI, REPL, and stdlib-facing
layers may rely on from the stabilized Ash contracts.

It defines:

1. which runtime outcomes are observably required,
2. which user-facing tool behaviors are normative,
3. which ADT and instance values have stable user-visible meaning, and
4. which failures must surface as observable errors rather than silent fallback.

## Scope

This handoff covers:

- workflow/runtime verification outcomes visible to tooling
- REPL-observable behavior from `ash repl`
- CLI/REPL distinction for help, type inspection, and AST inspection
- stdlib-visible runtime guarantees for `Option`, `Result`, variants, `InstanceAddr`, and
  `ControlLink`

Out of scope:

- parser acceptance
- lowering mechanics
- private implementation details such as concrete storage layouts or internal caches

## Required Observable Runtime Outcomes

### Verification and Execution

The runtime/verification stack must make these outcomes observable to tooling:

| Outcome | Required observable meaning |
|---|---|
| capability verification success | workflow may proceed with execution |
| capability verification error | tooling receives a concrete verification error, not silent degradation |
| workflow policy `Permit` | gated continuation may execute |
| workflow policy `Deny` | execution stops with observable denial |
| capability-level approval requirement | tooling/runtime observes an approval-required state rather than treating it as permit |
| capability-level transformation | transformed value is the observable value seen after policy application |
| `receive` timeout with wildcard | wildcard arm executes observably |
| `receive` timeout without wildcard | workflow falls through observably without synthetic error |

## REPL and CLI Observable Contract

### Normative REPL Surface

The only normative interactive entrypoint is:

- `ash repl`

The authoritative interactive command set is limited to:

- `:help`
- `:quit`
- `:type`
- `:ast`
- `:clear`

No additional command is part of the stabilized contract unless it is added to both `SPEC-005`
and `SPEC-011`.

### Required Observable REPL Behavior

| REPL feature | Required behavior |
|---|---|
| prompt | normal prompt is `ash> ` and multiline continuation prompt is `... ` |
| `:help` | lists the supported commands and aliases |
| `:quit` | exits the session |
| `:type <expr>` | prints a canonical Ash type name from the type vocabulary, not an effect name and not placeholder prose |
| `:ast <expr>` | prints the parsed AST representation for the expression |
| `:clear` | clears the interactive screen; it does not mutate language state |
| multiline input | incomplete input continues until it becomes parseable or is cancelled |
| history | persisted by default; disable/override behavior follows the CLI contract from `SPEC-005` / `SPEC-011` |

The current CLI and standalone REPL implementations may diverge, but downstream convergence work
must preserve this observable contract rather than current incidental behavior.

## Value Display Contract

### ADT Values

Runtime values for ADT constructors are observably constructor-shaped:

- unit variants display as the constructor name
- non-unit variants display as constructor name plus named fields

Examples:

- `None`
- `Some { value: 42 }`
- `Ok { value: "hello" }`
- `Err { error: "not found" }`

This is the observable consequence of the canonical runtime value shape:

- constructor name
- named payload fields

Tooling and stdlib behavior must not depend on synthetic tag fields such as `__variant`.

### Instance and Control Values

Runtime values expose distinct observable roles for:

- `InstanceAddr` as a communicable endpoint value
- `ControlLink` as transferable control authority
- `Instance` as a composite containing an address plus `Option<ControlLink>`

Observable formatting may vary in punctuation, but the distinction between address and control
authority must remain visible to users and tooling.

## Stdlib-Visible Guarantees

The stdlib-visible ADT surface relies on these runtime guarantees:

- `Option<T>` values are represented by `Some { value: ... }` and `None`
- `Result<T, E>` values are represented by `Ok { value: ... }` and `Err { error: ... }`
- `match` and `if let` consume those constructor-shaped runtime values consistently
- `unwrap`, `unwrap_or`, `map`, `and`, `or`, `ok_or`, `map_err`, `and_then`, `ok`, and `err`
  operate over that constructor-shaped runtime behavior
- `split`/control-link examples relying on `Option<ControlLink>` observe `Some { value: link }`
  vs `None` semantics rather than an unrelated sentinel encoding

These guarantees are about visible behavior, not about requiring one internal storage type.

## Observable Error Boundaries

The following failures must be surfaced observably:

- parse errors
- type errors
- verification errors and warnings
- policy denials
- runtime execution errors

The following are not valid silent fallbacks:

- replacing `:type` output with an inferred effect or placeholder text
- collapsing approval-required outcomes into permit
- collapsing transformed values into original pre-policy values
- treating non-exhaustive ADT failure as an invisible default branch

## Convergence Notes

Current implementation mismatches that downstream tasks must close:

- `ash-cli` REPL currently exposes non-normative commands such as `:bindings`.
- the current CLI `:type` path prints effect information instead of canonical type names.
- the standalone `ash-repl` crate currently prints placeholder type text for `:type`.
- history defaults differ between the standalone REPL and CLI shim, but the stabilized contract is
  the `ash repl` behavior defined by `SPEC-005` and `SPEC-011`.
