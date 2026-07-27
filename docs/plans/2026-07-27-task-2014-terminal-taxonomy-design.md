# TASK-2014 Terminal Taxonomy Design

## Decision

TASK-2014 adopts the existing canonical terminal-observable V1 schema for the
three currently unclassified production-boundary outcomes. No new JSON fields,
schema version, telemetry, frame authority, or direct-evaluator fallback is
introduced.

| Boundary condition | Canonical V1 observable | CLI exit code |
| --- | --- | --- |
| A source entry has no validated typed lowering or no Engine-issued production token | `external { boundary: "admission", outcome: "rejected" }` | 1 |
| A purported checked Core/CPS artifact is malformed or unchecked | `pre_entry_failure { class: "entry_verification", message: "checked Core/CPS artifact is invalid" }` | 4 |
| The exact admitted abortive `trap_sleep` handler body reaches `1 / 0` | `trap { reason: "division by zero" }` | 5 |

Existing `return`, `external/execution/timeout`, and
`external/execution/cancelled` behavior and their exit codes are unchanged.

## Ownership and routing

The selected Engine route classifies the terminal outcome before it is reduced
to the existing `CliError` text path. JSON output is emitted once to stdout,
or exclusively to `--output`; text mode retains its existing diagnostics and
exit behavior. Classification must not expose engine internals, provider
details, unchecked Core text, or handler implementation details.

`external/admission/rejected` is reserved for an otherwise source-valid
request that cannot be admitted through the strict checked Core/CPS boundary.
It is distinct from `entry_verification`, which represents a malformed or
unchecked purported sealed artifact. A handler-body failure happens after
admission and is consequently a language `trap`, rather than an admission or
verification failure.

The admitted handler-trap evidence is deliberately narrow. The exact abortive
`trap_sleep` fixture (fixed `1 / 0` language trap, no `resume`, identity
`done`, and exact `TestClock::sleep(0)` application) is admitted and executed
through checked Core/CPS, then projects the nonempty language reason as V1
`trap` (exit 5). Forged artifacts remain `entry_verification`, never a
substitute for that post-admission witness. This does not establish general
handler bodies, deep affine continuation resumption, or residual/open-row
semantics.

## Constraints

- Checked Core/CPS remains the sole execution owner for admitted source.
- Every unsupported source form remains fail-closed at admission.
- Rows never authorize frames; only sealed admission instructions do.
- The CLI terminal projection remains telemetry-free and preserves existing
  `--output` ownership.
- The implementation initially covers the bounded production routes that can
  demonstrate these conditions. It makes no claim of general handler lowering
  or general Core/CPS artifact construction.

## Evidence required

Focused CLI integration tests prove the implemented admission, invalid-artifact,
and exact admitted-handler-trap mappings in JSON mode, their nonzero exit
codes, and absence of direct-evaluator fallback. Engine-level tests prove
malformed/unchecked artifacts cannot reach dispatch and the exact `trap_sleep`
fixture fails only after checked-CPS admission. The existing terminal writer
continues to own stdout/`--output` exclusivity; dedicated `--output` evidence
for this handler-trap fixture remains a follow-up. A type-valid near-match
`trap_sleep` source using `TestClock::sleep(1)` is instead tested as missing
admission on stdout and exclusively through `--output`. TASK-2004, TASK-2008,
TASK-2014, PLAN-INDEX, traceability, and CHANGELOG evidence identify the
bounded scope and remaining generalizations.
