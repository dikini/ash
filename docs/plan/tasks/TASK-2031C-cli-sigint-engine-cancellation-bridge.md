# TASK-2031C: CLI SIGINT Delivery Capability Gate

**Status:** Complete
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Type:** Bounded implementation-conformance remediation
**Depends on:** TASK-2008 V1 terminal projection and TASK-2014 run-wide cooperative control
**Blocks:** TASK-2031A/TASK-2031 workspace-gate closeout

## Description

Preserve the existing CLI-to-Engine SIGINT cancellation contract while making its process-level
evidence reliable. Investigation established that the managed test sandbox does not deliver a
programmatic SIGINT to a standalone Tokio listener, even after resetting the child disposition;
there is therefore no CLI forwarding defect to repair. The gate must prove that its host can
exercise Tokio signal delivery before it draws conclusions about the admitted `time::sleep` route.

## Existing semantic handoff

**Canonical rules:** `SEM-EFFECT-CANCEL-001`, `SEM-EFFECT-TERMINAL-001`, and
`OBS-TARGET-PROJECTION-001`.

**Declared domain:** **bounded** to Unix SIGINT delivery for the exact admitted
`fn main() -> Null { time::sleep(<non-negative Int literal>) }` CLI route. Type, Core, CPS,
admission authority, provider binding, CLI forwarding, Engine cancellation precedence, and
terminal-envelope shape are existing owned behavior; this task adds only bounded test-host
capability evidence.
TASK-2008 remains the active semantic-task-record owner for the existing terminal contract; this
task adds no semantic rule, domain, evidence classification, or route ownership.

**Run-route impact:** **prerequisite**. The producer is the test-host capability probe; the
existing TASK-2008 terminal projection remains the consumer of an actual Engine outcome, and
TASK-2032 remains the separately owned CLI/daemon parity integration owner. The capability probe
does not authorize a client-local execution route or alter an active route.

**Handoffs:** Consumes TASK-2008's process-level cancellation evidence and TASK-2014's existing
`ProductionCancellation` contract. Produces a test-only, Linux-specific proof that the harness can
deliver SIGINT to Tokio before the existing CLI/Engine assertions run. The Engine owns cancellation
vs timeout vs completion ordering; TASK-2008 owns V1 terminal projection; no runtime authority,
proof responsibility, or general signal semantics transfer to this task.

## Requirements

1. Prove, in an isolated reset-disposition child, that a Tokio SIGINT listener can receive one
   programmatic SIGINT before running the Ash cancellation controls.
2. Run the existing Ash cancellation controls unchanged when that capability is present; only an
   isolated failed capability proof may report the environment unsupported.
3. Preserve exit 130 and the exact V1 `external/execution/cancelled` envelope on stdout and via
   `--output` for the selected route.
4. Do not change CLI production behavior, Engine driver, admission tokens, provider behavior,
   timeout semantics, terminal schemas, daemon behavior, or add a direct-evaluator/client-local
   fallback.

## TDD steps

1. **RED:** Run the two existing admitted `time::sleep(10000)` SIGINT integration tests and
   reproduce normal completion in the managed sandbox after the listener-ready signal delivery.
2. **ROOT CAUSE:** Show that an isolated child using only Tokio's SIGINT listener also fails to
   receive the same programmatic signal after disposition reset.
3. **GREEN:** Add a test-only, cached capability probe; retain the full Ash assertions when the
   probe succeeds and make an unsupported host explicit when it fails.
4. **QA/review:** Run terminal, Engine run-control, workspace, formatter, Clippy, docs, and
   independent review gates.

## Completion checklist

**Completion evidence:** The audit confirms that `de4043d8` changes only the test-host capability
preflight and cancellation controls; production CLI run and Engine code are untouched. On capable
hosts the unchanged controls retain exit 130 and the exact V1 `external/execution/cancelled`
envelope on stdout and through `--output`; the managed sandbox probe is explicitly unavailable.
The workspace, formatter, and Clippy gates were freshly green during TASK-2031A closeout. This
task does not add a signal, terminal, admission, execution, or client-parity semantic claim.

- [x] The prior SIGINT exit-0 failure and isolated Tokio delivery failure are reproduced.
- [x] The test-only capability probe gates neither production behavior nor a successful-capability
      cancellation assertion.
- [x] Both stdout and output-file cancellation envelopes remain asserted with exit 130 and no
      telemetry whenever the host supports the required Tokio delivery capability.
- [x] No alternate execution, admission, terminal, or shutdown path is introduced.
- [x] Workspace Rust tests, formatter, Clippy, and docs gate pass; QA/review evidence is recorded.
