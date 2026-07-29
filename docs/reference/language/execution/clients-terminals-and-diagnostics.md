# Clients and Terminal Results

[Execution index](index.md) · [Entry and admission](entry-lowering-and-admission.md) ·
[Effects and authority](../effects/index.md) · [Source of truth](../source-of-truth.md)

## Status and evidence

**Reviewed revision:** `423f603c`.

| Topic | Grammar | Static | Lowering | Admission/runtime | Implementation | Evidence | Parity |
|---|---|---|---|---|---|---|---|
| `ash run` admitted submission | not-applicable | not-applicable | not-applicable | fixture-bounded | partial | tested | below_spec |
| `ash test` selected source catalogue | not-applicable | not-applicable | not-applicable | fixture-bounded | partial | tested | below_spec |
| REPL source submission | not-applicable | not-applicable | not-applicable | fixture-bounded | partial | tested | below_spec |
| Daemon submitted descriptor | not-applicable | not-applicable | not-applicable | fixture-bounded | partial | tested | below_spec |
| Canonical V1 terminal envelope | not-applicable | not-applicable | not-applicable | admitted-executed | partial | tested | below_spec |

The shared terminal type is `ash_engine::CanonicalTerminalEnvelopeV1` in
`crates/ash-engine/src/error.rs`. The client adapters are
`ash-cli/src/commands/{run,daemon}.rs` and `ash-repl/src/{lib,session}.rs`.
Focused evidence includes `task_2032_shared_engine_client_parity.rs`,
`task_2038_ash_test_canonical_engine_execution.rs`,
`task_2039_repl_canonical_engine_execution.rs`,
`task_2042_daemon_admitted_request_terminal_envelope_parity.rs`, and
`task_2008_runtime_terminal_envelope.rs`.

## What clients may do

The clients format and submit a result; they do not choose language evaluation semantics.
`ash run` creates a local Engine, parses/checks the selected source, asks the Engine to admit it,
then submits the Engine-issued request through `execute_admitted_program`. Its
`submit_admitted_program` adapter has no parser, source selector, Core/CPS, row, provider, or
frame behavior.

The REPL takes the same approach for a complete submitted source: it asks its local Engine to
admit the parsed entry, mints a local request, and renders the returned terminal. Its inspection
commands are not execution routes, and an unadmitted source is returned as an Engine rejection
rather than a locally evaluated value.

`ash test` only has evidence for its declared selected source catalogue. Its synthesized and
metadata-only cases defer when they lack an executable Engine route; test metadata is not a local
oracle evaluator. The daemon accepts a validated submitted descriptor, constructs a separate
daemon-local Engine and request, and executes that local request. It does not transmit an opaque
request, provider binding, frame, or admission token over the Unix socket.

Client options, rows, imports, descriptor fields, and terminal values are non-authorizing
transport/configuration data. The Engine owns admission and dispatch; a client cannot make a
rejected source executable by changing its formatter or selecting a profile.

## Selected parity witness

The strongest current cross-client witness is intentionally tiny:

```ash
fn main() -> Int { 42 }
```

The Unix-socket daemon test uses that exact source together with its fixed source identity and
SHA-256 digest. It compares the daemon's terminal with `ash run`'s terminal and expects a V1
return carrying `42`. The surrounding TASK-2032, TASK-2038, and TASK-2039 controls keep the
selected CLI adapter, `ash test`, and REPL routes on the Engine-issued request seam for their
declared source catalogues.

This is **not** a general four-client parity theorem. In particular, it does not establish that
arbitrary sources, arbitrary descriptors, all runtime profiles, host providers, terminal
formatters, or target-language features agree across clients. The daemon test is Unix-only and
skips when the environment cannot bind the test Unix socket; it is a bounded integration witness,
not a transport fallback.

## Terminal envelope

For an Engine-issued admitted request, the shared dispatcher returns one of these six terminal
observations:

| V1 observation | Meaning at the Engine seam | Current client boundary |
|---|---|---|
| `Returned(Value)` | The admitted program produced a language value. | A client may format/project the value; it may not execute it again. |
| `Trapped(String)` | The admitted program reached a language-level terminal trap. | `ash run` emits its terminal observable and reports failure. |
| `AdmissionRejected` | No validated production admission exists for the requested artifact. | A sealed missing-admission classification; not a prompt for fallback evaluation. |
| `InvalidCheckedArtifact` | Checked Core/CPS provenance or artifact verification failed. | Projected as a pre-entry verification failure by the CLI's JSON boundary. |
| `TimedOut` | The Engine-owned request deadline won. | Projected as a terminal timeout; deadline ownership remains with the Engine request. |
| `Cancelled` | The Engine-owned cooperative cancellation control won. | Projected as a terminal cancellation; cancellation does not grant execution authority. |

The envelope carries only the normalized terminal observation. It carries no source text,
checked Core/CPS evidence, rows, provider bindings, frame authority, or right to create another
execution route. The `ash run --format json` route mechanically maps it to a public terminal
observable; a verified canonical `Result` entry has an additional exit-code projection only after
the Engine returns its terminal value. That projection does not parse, admit, or re-execute the
entry.

The bounded controls cover returned values, a sealed handler trap, missing admission, invalid
checked artifacts, deadline expiry, and pre-cancelled execution. They do not define a complete
diagnostic or exit-code contract for every parser/checker/runtime failure; the limitations and
diagnostic inventory is owned by TASK-2053.

## Client flow

```text
client-local source or validated descriptor
  → client-local Engine parse/check as applicable
  → Engine admission
  → Engine-issued request
  → execute_admitted_program
  → CanonicalTerminalEnvelopeV1
  → client formatting or terminal projection
```

The daemon differs only in where that chain begins: it validates a descriptor, then reconstructs
and mints the request inside its own Engine. No arrow in this diagram authorizes a direct AST
evaluator, a non-Engine CPS executor, a client-installed handler frame, or a route selected from
source spelling.

## Diagnostics and boundaries

- `AdmissionRejected` and `InvalidCheckedArtifact` are distinct normalized outcomes. A parsed or
  checked entry lacking sealed lowering is the former; malformed or forged purported checked
  evidence is the latter.
- A host profile that rejects a validated descriptor stops before the daemon mints its local
  request. A profile name is not source authority.
- Reusing an admitted request refreshes its Engine-owned deadline for each submission while
  retaining the request's cancellation state. This evidence is limited to the selected request
  controls.
- The in-process CLI/daemon adapter comparison in TASK-2032 uses an opaque request from one
  issuing Engine. The Unix-socket daemon comparison instead uses a descriptor and a separate
  Engine. Neither relationship is an API for third-party request transport.
- There is no client-side fallback when admission fails, and no current-language workflow/tower
  entry syntax behind these paths.

## Related evidence

- [TASK-2052](../../../plan/tasks/TASK-2052-language-reference-entry-engine-clients-terminals.md)
- [TASK-2032 shared Engine client seam](../../../plan/tasks/TASK-2032-shared-engine-execution-seam-and-client-parity.md)
- [TASK-2038 `ash test` boundary](../../../plan/tasks/TASK-2038-ash-test-canonical-engine-execution.md)
- [TASK-2039 REPL boundary](../../../plan/tasks/TASK-2039-repl-canonical-engine-execution.md)
- [TASK-2042 daemon descriptor parity](../../../plan/tasks/TASK-2042-daemon-admitted-request-terminal-envelope-parity.md)
- `cargo test -p ash-cli --test task_2008_runtime_terminal_envelope`
- `cargo test -p ash-cli --test task_2042_daemon_admitted_request_terminal_envelope_parity`
