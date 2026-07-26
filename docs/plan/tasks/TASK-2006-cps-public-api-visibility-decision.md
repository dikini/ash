# TASK-2006: CPS Public API Visibility Decision

**Status:** Complete
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)
**Depends on:** TASK-2004

## Description

Decide the supported visibility of `ash_core::cps` and CPS evaluator entrypoints, which currently
have no non-test Rust consumers in the audited workspace.

## Requirements

- Audit workspace and external compatibility surfaces before changing visibility.
- Choose stable public API, crate-private prototype, or removal only with replacement/absence proof.
- Preserve checked versus unchecked input safety boundaries and diagnostics.
- Document private machinery so it cannot be mistaken for canonical authority.

## TDD Steps

1. Add API/reachability fixtures and compatibility checks for the chosen boundary.
2. Add behavior tests for checked and malformed input handling.
3. Apply visibility/API changes only after a decision record.
4. Run public API, Core/CPS, docs, and downstream checks.

## Scoped decision and evidence

The selected visibility is **retained public compatibility API**: the Rust visibility boundary is
stable for the current compatibility surface, while its semantic/execution status remains
prototype-only. `ash_core::cps` and `ash_interp::cps` remain exported, including the checked
entrypoints and the explicitly trusted-IR unchecked entrypoints. This does **not** promote CPS
carriers or evaluator behavior to canonical Ash semantics or production execution.
TASK-2004 continues to own the retained-private production-boundary rationale, and TASK-2005
continues to own any direct-runtime/Core/CPS parity claim.

The workspace audit found no production Rust evaluator consumer outside the CPS implementation:
the Core lowerer, checker, and S-expression helpers consume CPS carriers, while evaluator
entrypoints are otherwise used by validation and tests. That absence does not justify an API
removal. Both `ash-core` and `ash-interp` are version `0.1.0` packages with no `publish = false`
manifest restriction, and the historical lazy/memo plan and TASK-1664 explicitly call
`eval_checked`, `eval_unchecked`, and `eval_term` existing public entrypoints. CPS carriers also
derive serde and underpin debug/fixture compatibility. Reducing visibility could therefore break
an unpublished package, source-archive, or Git consumer not visible in the workspace.

The external audit found no remote tags and no GitHub release records at the audit time. That is
not an absence proof: the registry endpoint could not be queried (HTTP 403), and neither release
metadata nor workspace search reveals unpublished or Git-dependent consumers. Accordingly, this
task neither removes nor narrows an exported API and makes no claim that no external consumers
exist.

`crates/ash-interp/tests/task_2006_cps_public_api_boundary.rs` is a downstream integration-test
fixture: it imports only the public CPS carrier/evaluator paths, compares checked and unchecked
evaluation only for a known well-formed trusted term, requires a malformed term to fail at the
checked validation boundary with `CpsRunError::Validation`, and compiles/observes the public
terminal projection. It deliberately never feeds malformed IR to `eval_unchecked`.

## Completion Checklist

- [x] Public surface is intentional and evidence-backed as a compatibility/prototype API.
- [x] No API is removed solely because current workspace search is empty.
- [x] Checked/unchecked semantics remain explicit and tested.
- [x] Canonical ownership and changelog are updated.

## TDD and verification evidence

The downstream fixture initially failed to compile when it compared
`Result<Atom, CpsRunError>` directly with `Result<Atom, CpsError>`, confirming that the checked
and unchecked paths intentionally carry different error boundaries. The fixture now compares only
their successful trusted-term values and separately asserts the checked validation diagnostic.

Verified after the fixture was corrected:

- `cargo test -p ash-interp --test task_2006_cps_public_api_boundary` — 3 passed.
- `cargo fmt --check`
- `cargo clippy -p ash-interp --test task_2006_cps_public_api_boundary -- -D warnings`
- `git diff --check`

## Evidence required

TASK-1988 found `eval_checked` consumers only in module/validation/tests. That is insufficient to
delete an exported API without compatibility and replacement evidence.
