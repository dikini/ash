---
id: language.reference.library.diagnostics-and-errors
title: Diagnostics and Errors
kind: feature-reference
status: partial
audience: [human, agent]
reviewed_revision: 423f603c
evidence: tested
refresh_trigger: ["crates/ash-parser/src/**", "crates/ash-typeck/src/diagnostic.rs", "crates/ash-engine/src/error.rs", "crates/ash-cli/tests/**"]
---

# Diagnostics and Errors

[Library and diagnostics](index.md) · [Standard-library modules and imports](modules-and-imports.md) ·
[Clients and terminal results](../execution/clients-terminals-and-diagnostics.md) ·
[Language reference](../index.md)

## Support

**Reviewed revision:** `423f603c`.

| Diagnostic boundary | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `ash check` parser failure | rejected | not-applicable | not-applicable | not-applicable | implemented | tested | not-applicable |
| `ash check` type/module failure | accepted | rejected-after-parse | not-applicable | not-applicable | partial | tested | below_spec |
| Engine admission rejection | accepted | checked | bounded-only | closed | partial | tested | below_spec |
| Canonical V1 terminal result | not-applicable | not-applicable | not-applicable | fixture-bounded | partial | tested | below_spec |

The primary paths are `ash-cli/src/commands/check.rs`, `ash-cli/src/error.rs`,
`ash-typeck/src/diagnostic.rs`, `ash-engine/src/{lib.rs,error.rs}`, and the terminal projection
adapters. The most direct evidence is
`crates/ash-cli/tests/check_parse_diagnostics.rs`,
`crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs`,
`crates/ash-cli/tests/task_2008_terminal_observable_projection.rs`, and
`crates/ash-cli/tests/task_2042_daemon_admitted_request_terminal_envelope_parity.rs`.

## What diagnostics are and how to use them

Ash reports errors at several stages. `ash check` first parses a path with the Engine. For an
`.ash` file, it may then try the module-file parser. Parsed source can still fail name, type, or
module checks.

`ash run` and other execution clients make a separate admission decision after parsing and
checking. If the Engine has no checked Core/CPS lowering, it rejects the program. An admitted
request then reports its
outcome using the six-case V1 terminal envelope described below.

Use the layer named by the diagnostic to decide what to fix. Do not treat a parser diagnostic as a
type error, a check success as admission success, or a terminal observation as permission to
retry through a direct evaluator.

## Observable diagnostic layers

| Layer | Current observable behavior | Evidence and limit |
|---|---|---|
| Parse | `ash check` reports a `CliError::ParseError`; its normal exit code is 2. It gives targeted migration diagnostics for selected removed spellings. | `check_parse_diagnostics.rs` exercises human and JSON migration messages. It is not a complete grammar-recovery specification. |
| Static and module checking | A parsed source may report `CliError::TypeError`; its normal check-command exit code is 1. Module-file registration errors also flow through this category. | `check.rs` runs `Engine::check_with_typeck_config` or bounded `check_module_file`. Typechecker/LSP diagnostics carry selected codes and spans, but this page does not promise every internal error a stable code. |
| Admission | The sealed Engine production boundary classifies a checked-but-unlowered source as `AdmissionRejected`; malformed/forged checked evidence becomes `InvalidCheckedArtifact` at the admitted-program seam. | Terminal-envelope tests distinguish the two. There is no evaluator fallback. |
| Admitted terminal | A shared Engine request returns a `CanonicalTerminalEnvelopeV1` observation. Clients only project or format it. | The bounded client tests cover each normalized kind, not every possible host, CLI, or provider failure. |

### Selected parser migration diagnostics

`ash check` has targeted migration messages for selected removed source forms and retired callable
notation. Their JSON form carries the code `DeprecatedSyntaxMigration`, source line/column,
context, and help. These diagnostics explain rejection; they do not preserve the removed forms as
current language syntax. The exact negative controls are in
`crates/ash-cli/tests/check_parse_diagnostics.rs`, rather than copied into this manual.

## Examples

### Admission diagnostic: checked source can still be closed

The JSON-library control described in [Standard-library modules and imports](modules-and-imports.md#examples)
uses a source that parses and checks, then receives the canonical closed-admission error instead
of executing `json::parse`. This example is a boundary, not runnable code:

```ash
use json::{parse};
fn main() -> String { parse("42") }
```

## Terminal result vocabulary

An Engine-issued admitted request can project exactly one current V1 terminal observation:

| V1 result | Meaning |
|---|---|
| `Returned(Value)` | The admitted program returned a language value. |
| `Trapped(String)` | The admitted program reached a language-level terminal trap. |
| `AdmissionRejected` | No validated production admission exists for the requested artifact. |
| `InvalidCheckedArtifact` | Checked Core/CPS provenance or artifact validation failed. |
| `TimedOut` | The Engine-owned deadline elapsed. |
| `Cancelled` | The Engine-owned cancellation control won. |

This vocabulary is the Engine/client terminal boundary. Pre-entry parse, I/O, configuration, and
ordinary type errors can instead be projected by their caller as a CLI error or bounded external
outcome. No table here converts those other errors into a universal terminal protocol.

## Syntax

Diagnostics introduce no source form. The only source syntax used on this page is ordinary
library import syntax, whose accepted direct-parser grammar is documented in
[Standard-library modules and imports](modules-and-imports.md#syntax). No EBNF fence is repeated
because the diagnostics themselves are not Ash grammar.

## Errors and limits

No source-level sequent is warranted. Diagnostics are projections of parser, checker, admission,
and Engine-terminal procedures rather than an implemented source calculus. The required ordering
is:

```text
source
  → parse or module-file diagnostic
  → static/module diagnostic
  → sealed admission decision
  → admitted-request terminal observation
  → client formatting/projection
```

Any earlier rejection stops that route. In particular, checking does not bypass admission, and a
client-side formatter cannot turn an `AdmissionRejected` result into execution authority.

## Limitations

- `ash-typeck/src/diagnostic.rs` and `ash-diagnostic` expose useful spans, severities, and selected
  codes, but the implementation does not establish a complete stable diagnostic-code taxonomy for
  all parser, typechecker, Engine, and host errors.
- CLI exit-code behavior is command-specific. The `ash check` parse/type mappings above are
  evidence for that command, not a promise for all clients.
- `AdmissionRejected` and `InvalidCheckedArtifact` are intentionally distinct. The first is
  missing validated admission; the second is invalid purported checked evidence.
- Terminal results carry no source, provider, row, frame, or admission authority and cannot be
  fed back as an execution request.
- Historical workflow/tower error vocabulary is excluded from the current language reference.

## Related evidence

- [Execution terminal results](../execution/clients-terminals-and-diagnostics.md#terminal-envelope)
- `cargo test -p ash-cli --test check_parse_diagnostics`
- `cargo test -p ash-cli --test task_2008_runtime_terminal_envelope`
- `cargo test -p ash-cli --test task_2008_terminal_observable_projection`
- `cargo test -p ash-cli --test task_2042_daemon_admitted_request_terminal_envelope_parity`
