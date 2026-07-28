# TASK-2042: Daemon Admitted Request and Terminal Envelope Parity

**Status:** Planned
**Semantic task classification:** semantic-runtime-integration
**Phase:** [PLAN-205](../PLAN-205-ENGINE-ONLY-EXECUTION-CUTOVER.md)
**Depends on:** TASK-2032, TASK-2035, TASK-2036, and TASK-2037

## Description

Extend the daemon protocol so a daemon client submits the same opaque Engine-issued admitted
request as `ash run`, rather than reparsing/evaluating source locally or exposing a daemon-specific
runtime. The daemon returns the canonical normalized terminal envelope for supported requests.
The finite supported request set, identities, inputs, bindings, deadline, cancellation signal,
and host configuration are those authorized by TASK-2035 and recorded by TASK-2034.

## Requirements

- The daemon carries an admitted request identity and canonical terminal envelope; it must not
  reconstruct admission authority or select an AST/CPS evaluator.
- The same admitted supported request has the same normalized terminal result through `ash run`
  and daemon. Transport framing, daemon lifecycle, and presentation remain outside that equality.
- Unsupported, malformed, stale, forged, cancelled, or timed-out requests fail at the canonical
  Engine/admission/terminal boundary; no fallback is permitted.
- Add finite-domain property tests for request/envelope parity and mutation rejection. Strategies
  may range only over the audited supported request corpus; they must not generate source forms or
  feature slices.
- On activation, add semantic coverage/traceability and report implementation, evidence, and
  parity independently for each named target rule.

## Handoffs

- **Run-route impact:** `active`.
- **Consumes:** TASK-2032's Engine request seam, TASK-2035's target contract, TASK-2036's guard,
  TASK-2037's Engine-private executor, and TASK-2034's finite request catalogue.
- **Produces:** daemon admitted-request transport, normalized terminal-envelope response, and
  run/daemon same-request parity evidence.
- **Downstream owner:** TASK-2040 removes residual daemon legacy calls; TASK-2041 owns final
  four-client parity and zero-use closure.
- **Does not own:** new daemon language semantics, source synthesis, admission reconstruction,
  formatting, or Lean execution.
- **Integration/proof responsibility:** owns daemon/`ash run` parity for its finite supported
  request set. TASK-2041 composes that evidence with `ash test` and REPL.

## TDD and activation steps

1. Promote the task and add canonical-rule coverage, traceability links, and focused evidence IDs
   before semantic Rust is staged.
2. Add failing daemon transport tests for admitted success, malformed/forged rejection, timeout,
   cancellation, and terminal-envelope normalization.
3. Add finite-domain parity and mutation property tests against the same Engine-issued request.
4. Implement the transport without adding a daemon-local evaluator; run focused daemon/CLI/Engine
   tests, formatter, clippy, semantic-task, traceability, and documentation gates.

## Completion checklist

- [ ] Daemon executes only Engine-issued admitted requests and returns the canonical terminal
      envelope for its selected finite domain.
- [ ] Same-request `ash run`/daemon normalized-terminal parity passes for success and required
      failure outcomes.
- [ ] Malformed, stale, forged, timeout, and cancellation controls fail closed without fallback.
- [ ] Task-owned implementation/evidence/parity records and CHANGELOG are current.
