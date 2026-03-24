# SPEC-021: Runtime Observable Behavior

## Status: Draft

## 1. Overview

This specification is the single normative owner for Ash runtime observable behavior.

It defines:

1. verification-visible and runtime-visible outcomes,
2. CLI and REPL observable behavior boundaries,
3. stable user-visible value display for ADTs and instance/control/monitor values, and
4. observable failure boundaries for parse, type, verification, policy, and runtime errors.

This spec is about what users and tooling may observe, not about the internal storage or execution
strategy used to realize those observations.

## 2. Observable Outcomes

### 2.1 Verification and Execution

The runtime and verification stack must make these outcomes observable:

| Outcome | Observable meaning |
|---|---|
| capability verification success | execution may proceed |
| capability verification error | tooling receives a concrete verification error |
| capability verification warning | tooling receives an observable warning distinct from success and error |
| workflow policy `Permit` | gated continuation may execute |
| workflow policy `Deny` | execution stops with observable denial |
| capability-level approval requirement | approval is observed as a distinct state, not as permit |
| capability-level transformation | the transformed value is the observable post-policy value |
| `receive` timeout with wildcard | the wildcard arm executes observably |
| `receive` timeout without wildcard | the workflow falls through observably without a synthetic error |

### 2.2 Recoverable Failure

Recoverable failures are represented explicitly as `Result<T, E>` values and are handled through
ordinary pattern matching.

`catch` is not part of the canonical observable contract. Any recoverable failure surfaced to users
or tooling must be representable as an explicit value and observed through `match` or `if let`.

### 2.3 CLI Tooling Outcomes

The CLI surface must preserve the same failure classes and observability rules:

- `ash check` surfaces parse, type, verification, and policy failures as diagnostics rather than
  collapsing them into one undifferentiated error class.
- `ash check` surfaces verification warnings as observable diagnostics rather than hiding them or
  collapsing them into success.
- `ash run` and `ash trace` surface runtime execution failures observably instead of silently
  swallowing them.
- `ash repl` surfaces interactive parse, type, and runtime failures using the same canonical
  observable categories.

## 3. CLI and REPL Boundaries

### 3.1 Normative REPL Entry Point

The only normative interactive entrypoint is:

- `ash repl`

The authoritative interactive command set is limited to:

- `:help`
- `:quit`
- `:type`
- `:ast`
- `:clear`

### 3.2 Required REPL Behavior

| REPL feature | Observable behavior |
|---|---|
| prompt | normal prompt is `ash> ` and multiline continuation prompt is `... ` |
| `:help` | lists the supported commands and aliases |
| `:quit` | exits the session |
| `:type <expr>` | prints a canonical Ash type name from the type vocabulary |
| `:ast <expr>` | prints the parsed AST representation for the expression |
| `:clear` | clears the interactive screen without mutating language state |
| multiline input | incomplete input continues until parseable or cancelled |
| history | persisted by default; disable/override behavior follows the CLI contract |

### 3.3 Tooling Failure Visibility

Tooling-visible failures must remain distinguishable at the observable level:

- parse errors
- type errors
- verification errors
- policy denials
- runtime execution errors

The contract does not permit silent fallback that hides one of these failure classes.

## 4. Value Display

### 4.1 ADT Values

Runtime values for ADT constructors are observably constructor-shaped:

- unit variants display as the constructor name
- non-unit variants display as the constructor name plus named fields

Examples:

- `None`
- `Some { value: 42 }`
- `Ok { value: "hello" }`
- `Err { error: "not found" }`

The visible shape is:

- constructor name
- named payload fields

Tooling and stdlib behavior must not depend on synthetic tag fields such as `__variant`.

### 4.2 Instance and Control Values

Runtime values expose distinct observable roles for:

- `InstanceAddr` as a communicable endpoint value
- `ControlLink` as transferable reusable control authority
- `MonitorLink` as shareable observation authority
- `Instance` as a composite containing an address plus `Option<ControlLink>` plus
  `Option<MonitorLink>`

Observable formatting may vary in punctuation, but the distinction between address and control
authority must remain visible. Monitoring authority must remain visible as a separate observable
role, not as control or messaging.

Observable control behavior follows the runtime control contract:

- `ControlLink` may be reused for non-terminal supervision operations such as health checks,
  pause, and resume while the target instance remains valid
- terminal control such as `kill` invalidates future control operations for that instance
- after terminal control, the current runtime contract retains the target as a tombstone for the
  lifetime of the owning `RuntimeState`
- later failed control attempts must surface as explicit runtime failures rather than as silent
  no-ops or hidden first-use consumption
- while that runtime state remains alive, retained tombstones must keep surfacing terminal-control
  failure rather than silently collapsing into `NotFound`

### 4.3 Monitor Views

Workflow instances may expose a monitor view via `exposes { ... }`.

- the monitor view is read-only
- it may include obligations, behaviours, and values
- it may include monitor metadata such as `monitor_count`
- only holders of `MonitorLink` may observe the exposed monitor view, subject to policy
- `MonitorLink` is shareable by default: sharing or delegating it does not consume the source
  authority, unlike control-link transfer
- monitoring does not imply control or message-send authority

### 4.4 Pattern Match Failure

When a `match` expression has no matching arm at runtime:

**Well-typed programs**: Exhaustiveness checking (SPEC-003, SPEC-020) guarantees this cannot happen
for statically-typed matches. The type checker rejects non-exhaustive pattern matches before runtime.

**Dynamic scenarios**: If a non-exhaustive match somehow executes (e.g., through dynamic code loading,
external data with unexpected shapes, or unsafe operations), the runtime signals a **pattern-match
failure** as an observable error.

**Observable error must include**:
- The scrutinee value that failed to match
- The match location (source position)
- The available match arms (for diagnosis)

**Note**: `if let` surface syntax lowers to a `match` with an explicit wildcard fallback arm (see
SPEC-001 Section 2.6). Therefore, `if let` has no separate failure mode—it always has a matching
branch.

## 5. Stdlib-Visible Guarantees

The stdlib-visible ADT surface relies on these runtime guarantees:

- `Option<T>` values are represented by `Some { value: ... }` and `None`
- `Result<T, E>` values are represented by `Ok { value: ... }` and `Err { error: ... }`
- `match` and `if let` consume those constructor-shaped runtime values consistently
- `if let` is observable as a `match` with an explicit wildcard fallback branch; it does not add a
  separate no-match failure mode
- `unwrap`, `unwrap_or`, `map`, `and`, `or`, `ok_or`, `map_err`, `and_then`, `ok`, and `err`
  operate over that constructor-shaped runtime behavior
- `split` / control-link examples relying on `Option<ControlLink>` observe `Some { value: link }`
  vs `None` semantics rather than an unrelated sentinel encoding
- monitor-view observation uses the declared exposed monitor view and does not grant control or
  messaging authority

These guarantees are about visible behavior, not about requiring one internal storage type.

## 6. Related Documents

- [SPEC-001](../SPEC-001-IR.md): IR - Core intermediate representation including `Value::Variant`
- [SPEC-002](../SPEC-002-SURFACE.md): Surface Language - Surface syntax for ADTs and match
- [SPEC-003](../SPEC-003-TYPE-SYSTEM.md): Type System - Type checking and exhaustiveness
- [SPEC-004](../SPEC-004-SEMANTICS.md): Operational Semantics - Pattern matching semantics
- [SPEC-020](../SPEC-020-ADT-TYPES.md): ADT Types - Algebraic data type specifications
