# TASK-2042: Daemon Descriptor and Terminal Envelope Parity

**Status:** Complete
**Semantic task classification:** semantic-runtime-integration
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2032, TASK-2035, TASK-2036, and TASK-2037
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2042 daemon descriptor and terminal-envelope parity](../SEMANTIC-RULE-COVERAGE.md#task-2042-daemon-descriptor-and-terminal-envelope-parity)

## Description

Extend the daemon protocol so it validates the declared wire descriptor: exact source identity and
bytes, entry, inputs, bindings, run control, and host configuration. The TASK-2035 manifest
identity and digest identify the selected source contract. Direct `ash run` receives source, not a
daemon descriptor; its local Engine retains the exact source bytes to bind that contract. The
daemon validates every wire field, while each execution path keeps a local Engine and independently
mints its process-local opaque request. The protocol never transports or reconstructs that
authority. The daemon returns the canonical normalized V1 terminal envelope for supported
descriptors.

## Requirements

- The daemon carries a submitted-program descriptor and canonical terminal envelope. Its local
  Engine mints the opaque request after parse, check, lowering, and admission; the protocol must
  not transport, reconstruct, or expose request authority, or select an AST/CPS evaluator.
- The selected normal source contract has the same normalized terminal result through direct
  `ash run` and the daemon's fully valid wire descriptor. Transport framing, daemon lifecycle,
  run-control failures, and presentation remain outside that equality.
- Malformed, stale, forged, and host-rejected wire descriptors use an Engine-owned pre-execution
  transport classification. Timed-out and pre-cancelled descriptors use the local Engine execution
  controls. Every failure is canonical and fail closed; no fallback is permitted.
- Add declared-corpus property tests for descriptor/envelope parity and mutation rejection.
  Strategies may range only over the audited supported request corpus; they must not generate source forms or
  feature slices.
- On activation, add semantic coverage/traceability and report implementation, evidence, and
  parity independently for each named target rule.

## Handoffs

- **Run-route impact:** `active`.
- **Consumes:** TASK-2032's Engine request seam, TASK-2035's target contract, TASK-2036's guard,
  TASK-2037's Engine-private executor, and TASK-2034's declared request corpus.
- **Produces:** daemon submitted-program-descriptor transport, process-local Engine request
  minting, normalized terminal-envelope response, and run/daemon same-source-contract parity evidence.
- **Downstream owner:** TASK-2040 removes residual daemon legacy calls; TASK-2041 owns final
  four-client parity and zero-use closure.
- **Does not own:** new daemon language semantics, source synthesis, admission reconstruction,
  formatting, or Lean execution.
- **Integration/proof responsibility:** owns daemon/`ash run` parity for its declared supported
  request corpus. TASK-2041 composes that evidence with `ash test` and REPL.

## TDD and activation steps

1. Promote the task and add canonical-rule coverage, traceability links, and focused evidence IDs
   before semantic Rust is staged.
2. Add failing daemon transport tests for admitted success, malformed/forged rejection, timeout,
   cancellation, and terminal-envelope normalization.
3. Add declared-corpus parity over the one normal selected source contract and mutation property
   tests over named wire-descriptor mutations.
4. Implement the transport without adding a daemon-local evaluator, a shared Engine service, or
   client-to-daemon request handles; run focused daemon/CLI/Engine
   tests, formatter, clippy, semantic-task, traceability, and documentation gates.

## Semantic workflow record

**Canonical rules:** `CONF-ENGINE-ONLY-CLIENT-001`, `SEM-EFFECT-ADMISSION-001`,
`SEM-EFFECT-TIMEOUT-001`, `SEM-EFFECT-CANCEL-001`, and `SEM-EFFECT-TERMINAL-001`.

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Only `TASK-2035-SHARED-ROUTE-001` is selected. The remaining daemon protocol domain, residual direct-evaluator deletion, and TASK-2041's four-client comparison remain incomplete.

**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime partial;
verification partial.

**Run-route impact:** active.

**Consumes:** `TASK-2035-SHARED-ROUTE-001`, `AUDIT-204-CLIENT-006`, TASK-2032's Engine request
seam, TASK-2036's no-fallback guard, and TASK-2037's Engine-private executor boundary.

**Produces:** descriptor validation, local-Engine admission and request minting, terminal-envelope
transport, and `ash run`/daemon same-source-contract parity evidence.

**Downstream owner:** TASK-2040 deletes residual daemon direct-evaluator calls. TASK-2041 owns the
four-client descriptor/envelope comparison and API-absence closeout.

**Does not own:** a shared Engine service, cross-process request handles, source synthesis,
admission reconstruction, a new daemon language, formatting, or Lean execution.

**Integration/proof responsibility:** TASK-2042 compares normalized envelopes for the selected
source contract: direct `ash run` retains the source bytes and daemon validates the full wire
descriptor before each local Engine mints its request. TASK-2041 separately composes that result
with `ash test` and REPL.

**Next obligation:** Retain the selected daemon descriptor route while TASK-2040 removes residual daemon direct-evaluator calls and TASK-2041 supplies the four-client terminal comparison.

## Task-owned evidence plan

The following controls are focused runtime evidence for this partial route. They do not establish
the remaining target-spec daemon domain or four-client parity.

- `TEST-TASK-2042-DAEMON-DESCRIPTOR-SUCCESS` (**Positive**): the shared descriptor reaches the
  daemon's local Engine and returns the V1 `Int(42)` terminal observation.
- `TEST-TASK-2042-DAEMON-DESCRIPTOR-ADMISSION-REJECTION` (**Negative**): the declared rejected
  host control uses the Engine-owned pre-execution transport terminal without a fallback.
- `TEST-TASK-2042-DAEMON-DESCRIPTOR-PRE-EXECUTION-CLASSIFICATION` (**Negative**): Engine maps
  invalid wire descriptors and host rejection before source execution.
- `TEST-TASK-2042-DAEMON-DESCRIPTOR-RUN-CONTROLS` (**Negative**): the declared zero-deadline and
  pre-cancelled controls project canonical timeout and cancellation terminals.
- `TEST-TASK-2042-DAEMON-DESCRIPTOR-MUTATION` (**Mutation**): named descriptor mutations
  (missing digest, forged identity, stale digest, nonzero deadline, and a deadline/cancellation
  combination outside the declared records) reject before execution; the strategy never generates
  source forms.
- `TEST-TASK-2042-DAEMON-DESCRIPTOR-PARITY` (**Parity**): direct `ash run` binds the manifest
  through its Engine-retained exact source bytes, while daemon validates the complete normal wire
  descriptor; their local Engines return the same normalized V1 terminal observation. Timeout,
  cancellation, and host-rejection controls are daemon-boundary evidence, not parity cases.

## Completion checklist

- [x] Daemon validates its descriptor, mints only local Engine-issued admitted requests, and returns the canonical terminal
      envelope for its selected declared scope.
- [x] The normal selected source contract has `ash run`/daemon normalized-terminal parity; daemon
      control failures have separate canonical boundary evidence.
- [x] Malformed, stale, forged, timeout, and cancellation controls fail closed without fallback.
- [x] Task-owned implementation/evidence/parity records and CHANGELOG are current.
