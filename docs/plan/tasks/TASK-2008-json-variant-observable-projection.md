# TASK-2008: JSON `_variant` Observable Projection

**Status:** In progress — `ash run --format json` now emits canonical envelopes for entry return,
declared runtime trap, entry execution failure, unreadable input plus parse/type/entry-verification
pre-entry failures (including one bounded dry-run declaration-only source), admission rejection,
and a canonical post-admission checked-CPS `time::sleep` timeout and cancellation slice. The
bounded closed production routes also project missing admission as `external/admission/rejected`
and invalid purported checked Core/CPS as the fixed `entry_verification` pre-entry failure. The
exact admitted abortive `trap_sleep` fixture also projects its post-admission division as a V1
`trap` (exit 5). A narrow build-configuration envelope is implemented and covered by the focused
terminal suite. These envelopes carry additive `schema_version: 1`, while full
observable/differential coverage remains deferred.
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)

## Description

Reconcile CLI JSON value conversion’s exposed `_variant` field with SPEC-021’s canonical runtime
observable projection.

## Requirements

- Establish whether `_variant` is canonical, derived compatibility metadata, or prohibited storage
  leakage.
- Test normal, trap, pre-entry failure, and bounded external outcome projections.
- Version or preserve compatibility explicitly if the wire result changes.
- Keep trace/session and runtime-artifact telemetry separate from language terminal outcomes.

## TDD Steps

1. [x] Add failing JSON projection fixtures for the canonical decision.
2. [x] Add compatibility fixtures for existing consumers where needed.
3. [x] Implement the isolated conversion/envelope prototype.
4. [x] Wire and test the selected CLI terminal paths: return, trap, pre-entry failure, bounded
   admission rejection, and the accepted canonical asynchronous `time::sleep` timeout slice.
5. [x] Model and test the bounded one-shot cancellation terminal boundary; complete the remaining
   observable/differential coverage separately.

## Scoped decision and current evidence

`_variant` is **legacy compatibility metadata for direct value serialization**. It remains the
wire shape emitted by `value_to_json` for existing consumers and is not a canonical terminal
observable field. The new serialization-only `CanonicalTerminalObservable` boundary instead
projects one of four fixed envelopes:

- `return` with a canonical value payload; variants use `constructor` and `fields`, never
  `_variant`;
- `trap` with a structured reason;
- `pre_entry_failure` with a class and message; or
- `external` with only a named boundary and bounded outcome.

The envelope carries no trace, session, runtime-artifact, or instance telemetry, and carries the
additive `schema_version: 1` marker. Focused fixtures
in `crates/ash-cli/tests/task_2008_terminal_observable_projection.rs` preserve the legacy direct
value shape and exercise the serializer. The binary contract in
`crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs` verifies that `ash run --format
json` emits exactly one leakage-free envelope for:

- successful checked entry execution: `{"kind":"return","value":...}`;
- a declared `RuntimeError`: `{"kind":"trap","reason":...}`;
- an entry `EntryBootstrapError::Execution` failure (the division-by-zero fixture):
  `{"kind":"trap","reason":...}`, with a nonempty language reason and no raw bootstrap,
  engine, verification, or invalid-exit-code leakage;
- unreadable source input: `{"kind":"pre_entry_failure","class":"input","message":"entry source could not be read"}`;
- malformed source, typecheck failure, wrong entry contract, or an ordinary source file without
  `main`: `{"kind":"pre_entry_failure",...}`; and
- a declaration-only source under `ash run --dry-run --format json`: the existing
  `{"kind":"pre_entry_failure","class":"entry_verification","message":"entry file has no 'main' entry"}`
  envelope; and
- rejected host admission: `{"kind":"external","boundary":"admission","outcome":"rejected"}`; and
- a canonical `time::sleep` timeout: `{"kind":"external","boundary":"execution","outcome":"timeout"}`; and
- one-shot cancellation: `{"kind":"external","boundary":"execution","outcome":"cancelled"}`.
- the exact admitted abortive `trap_sleep` handler: `{"kind":"trap","reason":"division by zero"}`
  (exit 5), after checked-CPS admission rather than as an admission failure.

The selected TASK-2014 terminal taxonomy now adds two typed Engine-to-CLI outcomes for closed
production routes. A source that parses/checks but lacks an Engine-issued validated lowering/token
projects `{"kind":"external","boundary":"admission","outcome":"rejected"}` and exits 1.
A forged, malformed, or unchecked purported sealed Core/CPS artifact projects exactly
`{"kind":"pre_entry_failure","class":"entry_verification","message":"checked Core/CPS artifact is invalid"}`
and exits 4. The classification is carried by the Engine boundary, not inferred from error text;
the focused Engine control proves a forged artifact cannot dispatch a provider. JSON is emitted
once to stdout or exclusively to `--output`, with no direct-value fallback or implementation
telemetry. The exact admitted `trap_sleep` fixture now reaches a real post-admission language trap:
its no-`resume`, identity-`done` handler clause lowers fixed `1 / 0` and emits V1 `trap` with a
recognizable nonempty division-by-zero reason (exit 5). This stdout integration evidence does not
generalize handlers, continuations, residual/open rows, or all `--output` handler-trap routes.
Forged artifacts remain invalid pre-entry evidence, not handler-trap evidence.
The focused output-file counterpart now covers that **same exact admitted** `trap_sleep` fixture
only: `ash run --format json --output terminal.json` exits 5, leaves stdout empty, and writes one
V1 `trap` envelope with its division-by-zero language reason to `terminal.json`. The test also
rejects implementation telemetry in that file. It is output-ownership evidence for the already
admitted route, not a new terminal kind, generalized handler route, broader `--output`
guarantee, or a change to TASK-2014 admission.
The near-match, still type-valid lexical `trap_sleep` candidate with `TestClock::sleep(1)` instead
has no exact validated lowering/token and is covered as `external/admission/rejected` (exit 1) on
stdout and exclusively via `--output`; it cannot be routed as the exact handler trap.
Likewise, a type-valid lexical `trap_sleep` with two checked operation clauses is structurally
ineligible for the one-clause bounded token and rejects as missing admission before the private
Core inspection/lowering bridge can run.
Conversely, a same-Engine forged exact `trap_sleep` public Core is typed as invalid checked
Core/CPS and the CLI seam writes the fixed `pre_entry_failure/entry_verification` envelope
(exit 4) exclusively to `--output`; foreign-Engine provenance remains missing admission.

- malformed `--capability-impl` build configuration:
  `{"kind":"pre_entry_failure","class":"configuration","message":"run configuration is invalid"}`.

The bounded configuration/build slice handles a malformed
`--capability-impl` selection that fails during engine construction before any source path is read. For
`--format json`, it emits exactly
`{"schema_version":1,"kind":"pre_entry_failure","class":"configuration","message":"run configuration is invalid"}`.
The class and message are deliberately coarse and stable: they must not expose the selected flag
value, host error chain, provider details, or build internals. Text output, error reporting, and
exit behavior remain unchanged. The binary contract is
[`task_2008_runtime_terminal_envelope.rs`](../../../crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs)
and passes as part of its focused 37-test terminal suite. With `--output terminal.json`, that file
owns the exact same configuration envelope and stdout is empty; this prevents the build-failure
route from falling back to a legacy direct-value JSON payload or duplicating output.

The same coarse classification now covers the source-aware configuration boundary after a readable
source has been loaded: a syntactically valid `--capability-impl binding=missing_impl` selection
whose implementation is absent from that source emits the identical JSON envelope on stdout and
then fails nonzero before parsing, entry verification, admission, or execution. Neither the
missing implementation name nor the engine's configuration-error display reaches the envelope.
This preserves a stable terminal class across pre-read engine construction and post-read source
validation; text diagnostics and exit behavior remain unchanged. The source-aware `--output`
variant writes the same exact configuration envelope to the requested file with stdout empty.
Both configuration paths therefore preserve one telemetry-free JSON envelope shape regardless of
whether the caller selects stdout or `--output`.

The same terminal projection now covers the actual execution-failure route after valid entry
bootstrap. The division-by-zero integration fixture reaches `EntryBootstrapError::Execution` and
projects a versioned `trap` envelope with a nonempty reason, either on stdout or exclusively in
the requested `--output` file. This is deliberately narrower than a claim that every bootstrap
failure is an execution trap: verification, engine, and invalid-exit-code outcomes retain their
existing classifications and do not enter this route. The focused terminal suite now passes
30/30 tests.

The same successful return envelope is written to `--output terminal.json` with stdout empty,
and the same is true for unreadable input, so the file-output route cannot silently fall back to
legacy direct-value JSON. Text behavior is unchanged: the new input envelope is JSON-only.

The legacy `value_to_json` function remains available and its `_variant` wire shape is covered as
direct-value compatibility; it does not carry a canonical-envelope version field. The terminal
envelope now has an explicit versioned migration boundary.

### Host-outcome direction and bounded timeout slice

TASK-2008 accepts one narrowly scoped canonical asynchronous host operation: the exact checked-CPS
source form `fn main() -> Null { time::sleep(<non-negative Int literal>) }`. It is not an entry
bootstrap program and does not use the legacy `Result<(), RuntimeError>` entry contract. The CLI
first parses, checks, validates the registered `time.sleep` binding, and seals a production token;
only then does it create the Engine run-control envelope. The focused binary fixture calls
`time::sleep(1500)` with `ash run --format json --timeout 1`; the post-admission control causes
the Engine driver to project the versioned external execution timeout before the provider can
complete. This is a bounded production-host-operation slice, not a general host-execution model.

This narrow route has no trace/telemetry contract and does not support `--trace`. Callers must not
interpret it as traced execution or infer a trace session, report, or telemetry from its terminal
envelope; an implementation may reject that flag combination rather than adding such behavior.

### One-shot cancellation boundary

The existing generic one-shot CLI helper still races an arbitrary command execution future against
`tokio::signal::ctrl_c()` for legacy/non-production routes. That outer race is a historical
command-level boundary; it is not the control semantics of the admitted checked-CPS host-operation
slice.

For the exact admitted `time::sleep` route, the CLI completes admission first and forwards SIGINT
only as a cancellation signal to the Engine's post-admission `RunControl`. The Engine driver owns
the cancellation/deadline/provider-completion decision and drops an in-flight provider future when
control wins. This is cooperative only: it does not claim a process kill, rollback, or retained
runtime instance. Cancellation projects exit code `130` and exactly the versioned
`external/execution/cancelled` envelope. With `--format json`, it writes that envelope to stdout
unless `--output` is selected, in which case the output file owns it and stdout remains empty. A
Unix binary integration control proves that exact ownership rule for a running `time::sleep(10000)`
process interrupted by SIGINT: it exits `130`, stdout is empty, and `terminal.json` contains only
the versioned `external/execution/cancelled` envelope.

This differs from `ash daemon cancel`: daemon cancellation is a durable control-plane lifecycle
operation over a retained instance, whereas the generic one-shot helper is only a signal race for
the current command and the admitted slice is post-admission Engine cooperation. Async filesystem
work remains a legacy ordinary-module direct-value surface, so it is not used as canonical timeout
evidence. Full observable-contract/differential coverage and diagnostics evidence remain required.

### Input and configuration boundaries

Before source classification, an unreadable input path is a deterministic pre-entry boundary.
For `--format json`, it projects the stable `input` pre-entry failure above to stdout or `--output`;
the existing text diagnostic and direct-value compatibility surfaces are unchanged. The specified
malformed build-selection boundary is likewise pre-entry but occurs even earlier, before source
I/O, and emits the fixed `configuration` envelope above.

The source-aware configuration validation case is covered for both JSON stdout and `--output`.
It shares the build-selection route's exact envelope and output ownership rule, without exposing
source or engine configuration details.

### Dry-run declaration-only entry-verification boundary

A declaration-only source with no `main` under `ash run --dry-run --format json` now follows the
existing versioned `pre_entry_failure` / `entry_verification` projection. It exits nonzero and
emits exactly `entry file has no 'main' entry` to stdout, or exclusively to the requested
`--output` file with stdout empty. Text diagnostics and exit behavior are unchanged.

This is a failure projection at the entry-verification boundary, not dry-run success semantics:
it does not execute a declaration-only program, project a successful dry run, or alter legacy
ordinary direct-value JSON projection. The focused stdout and file-ownership controls are
`run_dry_run_json_projects_a_declaration_only_source_as_an_entry_verification_failure` and
`run_dry_run_json_writes_a_declaration_only_entry_verification_failure_to_output_file` in
[`task_2008_runtime_terminal_envelope.rs`](../../../crates/ash-cli/tests/task_2008_runtime_terminal_envelope.rs).

## Completion Checklist

- [x] `_variant` has one documented observable status.
- [x] Canonical terminal projection excludes implementation leakage for the four implemented CLI
  terminal paths.
- [x] Canonical terminal envelopes are explicitly versioned; legacy direct-value JSON remains
  wire-compatible.
- [x] Canonical `time::sleep` provides a deterministic bounded timeout projection without
  changing legacy direct-value JSON.
- [x] One-shot cancellation races cooperatively with command execution, exits `130`, and projects
  a versioned telemetry-free external envelope to stdout or `--output`.
- [x] Unreadable input emits a stable JSON-only `input` pre-entry failure to stdout or `--output`.
- [x] Malformed `--capability-impl` build configuration emits the specified JSON-only
  `configuration` pre-entry failure before source I/O. The focused terminal suite passes 37/37;
  formatting, Clippy, and diff checks are clean.
- [x] With `--output`, malformed build configuration writes that exact envelope to the requested
  file and leaves stdout empty.
- [x] A source-aware unknown capability implementation selection against a readable entry emits
  the same coarse JSON configuration envelope on stdout and fails before entry processing.
- [x] With `--output`, a source-aware unknown capability implementation selection writes that
  exact envelope to the requested file and leaves stdout empty.
- [x] A valid entry whose evaluation fails through `EntryBootstrapError::Execution` emits a
  versioned nonempty-reason `trap` envelope on stdout or exclusively through `--output`, without
  changing verification, engine, or invalid-exit-code classifications.
- [x] A declaration-only dry-run source without `main` reuses the versioned JSON
  `entry_verification` pre-entry-failure envelope on stdout or exclusively through `--output`;
  it does not establish dry-run success semantics.
- [x] Bounded closed production routes classify missing validated admission as JSON
  `external/admission/rejected` (exit 1), and invalid purported checked Core/CPS as fixed JSON
  `entry_verification` (exit 4), preserving exclusive `--output` ownership and no dispatch.
- [ ] CLI, observable-contract, and differential evidence covers the complete terminal boundary.
- [x] Rule traces and changelog are updated.

## Evidence required

TASK-1988’s runtime slice identifies the present mismatch. Completion needs serialized behavior
evidence, not an internal enum rename.
