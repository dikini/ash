# TASK-2019: Project Post-Execution Invalid Entry Exit Codes Through the Canonical Terminal Envelope

**Status:** Complete
**Phase:** Implementation follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md) and the bounded terminal projection in [TASK-2008](TASK-2008-json-variant-observable-projection.md)

## Description

Complete one missing post-execution terminal-projection case: a valid, checked
`main` can return the required `Result<(), RuntimeError>` terminal value while
the leading `RuntimeError` code cannot be represented as an OS exit code.  At
present `derive_entry_exit_code` discards that value when it returns
`EntryBootstrapError::InvalidExitCode { code }`, so `ash run --format json`
cannot project the language terminal outcome and instead falls through its
generic error path.

For this task's exact fixture,
`Err { error: RuntimeError(999, "boom") }`, the source has a valid canonical
entry contract, bootstrap and evaluation succeed, and only OS exit-code
derivation fails.  JSON output must therefore be the existing versioned
canonical `trap` envelope derived from the retained terminal value, rather
than a `pre_entry_failure`, a raw invalid-exit-code diagnostic, or a new
envelope kind.  The process remains non-successful because no legal OS exit
code was derived; this task does not redesign the OS exit-code policy.

## Authoritative References

- [SPEC-021 §2.3](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md): entry
  completion is the observable boundary and a `RuntimeError(N, _)` determines
  the entry result.
- [SPEC-005, Process Exit Policy](../../spec/SPEC-005-CLI.md): `ash run`
  derives its process result from `main` completion.
- [TASK-2008](TASK-2008-json-variant-observable-projection.md): canonical
  versioned terminal envelopes and stdout/`--output` ownership.

## Scope

### In scope

- Retain the exact evaluated `ash_core::Value` in the post-execution
  invalid-exit-code error carrier, alongside the invalid numeric code.  This
  is terminal-value transport, not re-evaluation or reconstructed text.
- At the JSON-only CLI projection boundary, recognize that carrier and project
  the retained value through the existing canonical versioned `trap` envelope.
  For the bounded fixture its reason is the canonical terminal rendering of
  `RuntimeError(999, "boom")`.
- Prove the exact valid-entry route with a binary integration test: a source
  `main` with the canonical return type returns the out-of-range runtime error;
  it emits one versioned `trap` envelope and exits nonzero.
- Prove output ownership: without `--output` the one envelope is on stdout;
  with `--output <file>` the exact same envelope is in that file and stdout is
  empty.
- Preserve the terminal value through the engine-to-CLI error boundary so the
  test can distinguish retention from a generic error-string heuristic.

### Explicit exclusions

- No change to `derive_entry_exit_code`'s legal `0..=255` range, OS exit-code
  mapping, successful/runtime-error process exit behavior, text diagnostics,
  or `--format` non-JSON behavior.
- No change to entry verification, parsing, type checking, engine execution,
  entry evaluation, direct-value `_variant` serialization, or legacy workflow
  execution/dry-run paths.
- No attempt to classify engine, verification, pre-entry, admission, timeout,
  cancellation, or ordinary `EntryBootstrapError::Execution` outcomes beyond
  the existing TASK-2008 behavior.
- No telemetry, trace, monitor, provider, admission, Core, CPS, handler, or
  differential-contract change.

## Requirements and Invariants

1. **Post-execution boundary.** The task fixture must pass canonical entry
   verification and produce the `Err/RuntimeError` terminal value before
   invalid-exit-code detection.  It is not a pre-entry failure.
2. **Exact value retention.** `EntryBootstrapError::InvalidExitCode` (or an
   equivalently explicit post-execution carrier) contains both `code: 999` and
   the exact evaluated terminal `Value`; the CLI must not synthesize a value
   from `999` or parse a diagnostic string.
3. **Canonical reuse.** JSON projection reuses the existing
   `schema_version: 1`, `kind: "trap"`, `reason` terminal envelope.  It must
   not introduce a special invalid-exit-code wire class or leak bootstrap,
   engine, verification, or host-error details.
4. **Output exclusivity.** The normal JSON route emits exactly one envelope on
   stdout.  The `--output` JSON route writes that same complete envelope to the
   requested file and emits no stdout JSON/value payload.
5. **Narrow classification.** Only the carrier that proves a terminal value
   existed receives this projection.  Engine and verification failures remain
   on their established TASK-2008 paths; no broad generic-error heuristic may
   accidentally reclassify them as traps.
6. **No semantic authority.** Carrying the value affects only JSON terminal
   serialization after execution.  It does not alter the evaluated result,
   admission/provider behavior, traces, monitors, Core/CPS, or the invalid
   exit-code range rule.

## TDD Steps

1. **Freeze current boundary.** Inspect `ash-engine::derive_entry_exit_code`,
   `EntryBootstrapError`, entry bootstrap result construction, and the
   `ash-cli` terminal-envelope projection/error classification ordering.
   Record that invalid exit code is post-execution but currently loses the
   terminal value.
2. **RED: retained carrier.** Add an engine-level regression around a valid
   entry terminal `Err { error: RuntimeError(999, "boom") }`.  Assert the
   invalid-exit carrier exposes `999` and the exact evaluated `Value`; this
   must fail while the carrier stores only the integer.
3. **RED: stdout projection.** Extend the focused TASK-2008 binary terminal
   suite with the same valid source under `ash run --format json`.  Assert a
   nonzero status and the exact leakage-free versioned `trap` envelope, with
   a reason derived from the retained terminal value.  Assert that the output
   is not `pre_entry_failure` and does not expose `invalid runtime exit code`.
4. **GREEN: narrow transport and projection.** Add the exact value to the
   post-execution carrier at the point where entry execution has completed;
   thread it only to the JSON terminal projection.  Reuse the existing trap
   serializer and leave range validation and non-JSON classification intact.
5. **RED/GREEN: output ownership.** Add the `--output` counterpart and require
   byte-for-byte-equivalent JSON in the output file with empty stdout.  Keep
   the existing return, execution-trap, and pre-entry output controls green.
6. **Regression and documentation.** Run focused entry/CLI tests, relevant
   terminal projection regressions, engine and CLI checks, all-target/all-
   feature Clippy with warnings denied, formatting, docs/traceability gates,
   and `git diff --check`.  On implementation completion update this task,
   `CHANGELOG.md`, `PLAN-INDEX.md`, and semantic traceability only for this
   bounded terminal-projection case.

## Expected Completion Evidence

- An engine-level test proves a valid evaluated `RuntimeError(999, "boom")`
  value survives inside the post-execution invalid-exit-code carrier.
- The CLI test proves the same source emits exactly
  `{"schema_version":1,"kind":"trap","reason":"RuntimeError(999, \\\"boom\\\")"}`
  (subject only to JSON object-key ordering) to stdout and exits nonzero.
- The `--output` control proves that exact envelope is written exclusively to
  the output file and stdout is empty.
- Existing pre-entry, execution-trap, successful return, timeout,
  cancellation, admission, and direct-value compatibility tests remain green;
  none is reclassified by string matching.

## Completion Checklist

- [x] The post-execution invalid-exit carrier retains the exact terminal
  value, not only the numeric code.
- [x] A valid `RuntimeError(999, "boom")` entry projects the existing
  versioned canonical JSON `trap` envelope and remains non-successful.
- [x] The JSON `--output` route is exclusive and byte-for-byte equivalent to
  stdout projection.
- [x] Verification, engine, pre-entry, legacy/dry-run, and non-JSON behavior
  are unchanged.
- [x] No new terminal wire kind, generic-error heuristic, runtime authority,
  or engine execution behavior is introduced.
- [x] Focused and affected regressions, Clippy, formatting, docs/traceability,
  and diff gates pass.

## Completion Evidence

`EntryBootstrapError::InvalidExitCode` now retains `terminal_value` at the
already-completed `derive_entry_exit_code` boundary. The retained value passes
only to the JSON-only CLI projection helper, which reuses the existing
versioned `trap` serializer. It neither re-evaluates the entry nor reconstructs
a language result from the numeric code. The invalid `0..=255` process-code
policy still makes this route non-successful; the helper's private boxing is
transport-only and has no runtime authority.

`crates/ash-engine/tests/entry_verification.rs` proves the exact valid-entry
`Err { error: RuntimeError(999, "boom") }` value survives in the
`InvalidExitCode { code: 999, terminal_value }` carrier. The TASK-2008
terminal integration suite proves JSON stdout projects exactly one canonical
`schema_version: 1`, `kind: "trap"` envelope with the terminal rendering as
its reason; its `--output` counterpart proves the same envelope is owned
exclusively by the requested file and stdout is empty. The suite retains its
text, pre-entry, and legacy controls, so those routes are not reclassified.

Regression evidence: entry verification 21/21, terminal envelope 19/19,
terminal projection 5/5, output ownership 17/17, affected all-target/all-
feature Clippy with warnings denied, formatting, `git diff --check`, semantic
traceability validation, and the documentation gate passed. QA and review
confirmed that this is a post-execution-only classification and that no
generic error-string heuristic, new wire kind, or engine behavior was added.
