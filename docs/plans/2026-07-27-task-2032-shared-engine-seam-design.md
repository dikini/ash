# TASK-2032 Shared Engine Execution Seam Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make selected admitted Ash routes execute through one Engine-owned request/result seam
and compare its canonical terminal envelope through CLI and daemon clients.

**Architecture:** The Engine mints an opaque admitted-program artifact only after the existing
TASK-2004/TASK-2014 checks succeed, then creates an opaque request binding it to deadline and
cancellation control. The Engine alone dispatches the sealed artifact to its existing checked-CPS
driver and projects TASK-2008's versioned terminal envelope. CLI and daemon submit the same
request and retain only formatting or lifecycle/transport responsibilities.

**Tech Stack:** Rust 2024, `ash-engine`, `ash-cli`, Tokio, existing TASK-2004/TASK-2014 sealed
admission artifacts, and TASK-2008 terminal projection.

---

## Approved bounded design

### Engine ownership

`ash-engine` will expose documented public concepts whose constructors are restricted to `Engine`:

- `AdmittedProgram`: opaque Engine-issued provenance over a selected checked-CPS or
  checked-handler admission. It exposes no raw CPS, row, provider, or frame-construction
  authority.
- `AdmittedProgramRequest`: opaque Engine-created execution request, obtained only from its
  issuing Engine and `AdmittedProgram`; it binds the artifact to explicit deadline/cancellation
  control only after admission succeeds.
- `CanonicalTerminalEnvelopeV1`: TASK-2008's stable, versioned terminal projection and the
  normalized comparison value for return, rejection, malformed evidence, trap, timeout, and
  cancellation. It excludes CLI output paths and daemon lifecycle/transport telemetry.

The Engine selects current bounded literal/pure, `time::sleep`, `trap_sleep`, and `forward_sleep`
artifacts only from validated checked evidence and retained provenance. Neither client may use
source shape, handler names, raw CPS, or a route-specific terminal variant for semantic routing.
Uncovered artifacts reject at admission; no missing lowering or failed admission selects
`Engine::execute`, `Engine::run`, or another direct-evaluation fallback. Only validated
`FrameInstallationInstructionV1` data can install frames; rows stay non-authorizing requirements.

### Client adapters and parity

`ash run` parses/checks/configures its host, then submits the Engine-issued request and formats the
returned envelope. The daemon retains socket, instance, reload, drift, and lifecycle work, but its
execution worker submits that same request rather than reparsing into `Engine::execute`.

Parity holds only for the same admitted artifact identity, inputs, resolved bindings, host
configuration identity, deadline, and cancellation signal. It compares only
`CanonicalTerminalEnvelopeV1`; CLI formatting and daemon transport/lifecycle output are outside
the equality relation. A request retains its timeout configuration but the Engine starts a fresh
deadline for each submission; its cancellation signal remains shared and sticky across submissions.

### Initial matrix and non-goals

`docs/plan/RUNNABLE-ASH-MATRIX.md` is the route ledger. No row becomes an executable shared-client
route until the Engine-seam and client-parity RED tests turn green. This bounded task is not a
general CPS executor, parser/lowering expansion, provider implementation, handler-semantics
change, terminal-taxonomy change, or daemon-protocol redesign.

### TDD implementation sequence

1. Run `cargo test -p ash-engine --test task_2032_shared_engine_execution_seam` and confirm the
   desired Engine seam API is absent.
2. Add only the opaque Engine admission/request/result seam; preserve existing sealed admissions.
3. Run `cargo test -p ash-cli --test task_2032_shared_engine_client_parity` and make both
   adapters submit an identical request without semantic routing or direct evaluation.
4. Remove the named client-local recognizers/evaluator only after both focused tests pass.
5. Update matrix/traceability statuses only after focused tests, formatter, clippy, docs gates,
   and client integration evidence are green.
