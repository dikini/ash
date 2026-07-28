# TASK-2034: Direct AST Retirement Audit Manifest

**Status:** Complete
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** [PLAN-204](../PLAN-204-DIRECT-AST-RETIREMENT-AUDIT-AND-CONTRACT-FREEZE.md)
**Depends on:** PLAN-203 and TASK-2033

## Description

Create `docs/plan/audits/AUDIT-204-direct-ast-retirement.{md,json}` as the frozen catalogue of
all direct-AST and non-Engine evaluator material present at the audit commit. It covers code,
tests, corpora, benchmarks, scripts, workflows, Lean artifacts, tasks, plans, specs, reference
documents, and traceability edges. It does not remove or migrate any item.

Each JSON entry has: stable ID; path; symbol or exact text locator; current role; reachability
(`run`, `daemon`, `test`, `repl`, `differential`, `none`); current-state classification
(`current`, `historical`, or `deferred_separate_project`); execution role (`executable`,
`test-only`, or `reference-only`); target rule/contract; disposition (`replace`, `delete`,
`deferred`, `historical`, or `deferred_separate_project`); Phase-205 owner or external handoff;
consumed/produced handoff; required test/proof evidence; and rationale. A `deferred` record also
names one exact `case_id`, missing obligation, and fail-closed result. A Lean
`deferred_separate_project` record instead names its external project, owner, handoff, retained
paths, and prohibited current authority. The manifest stores its repository revision and a
sorted-entry SHA-256 digest.

## Requirements

- A disposition `deferred` entry alone carries the finite-case fields: `case_id`, missing
  source-wrapper or target-spec clause, and expected fail-closed result. Those fields do not
  apply to `deferred_separate_project` entries.
- A `deferred_separate_project` Lean entry carries `external_project`, `external_owner`,
  `external_handoff`, `retained_paths`, and `prohibited_current_authority` instead. It is never a
  deletion candidate in this phase.
- Inventory the current-authority links for [SPEC-046](../../spec/SPEC-046-LEAN-REFERENCE.md),
  the differential section of [SPEC-026](../../spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md),
  [TASK-439](TASK-439-differential-conformance-harness-rust-first.md),
  [TASK-440](TASK-440-lean-reference-refresh-plan-against-current-semantic-corpus.md), and the
  [formalization boundary](../../reference/formalization-boundary.md). Classify each as deferred
  separate-project work or as stale current-Ash authority requiring relabeling/quarantine.
- Report audit findings without claiming implementation, test/proof evidence, or target-spec
  parity for the removed or deferred behavior.

## Handoffs

- **Run-route impact:** `none`.
- **Consumes:** current code/documentation inventory and PLAN-203's Engine-only route policy.
- **Produces:** a finite Rust-removal catalogue, deferred-case catalogue, and Lean separate-project
  handoff map.
- **Downstream owner:** TASK-2035 consumes missing contract clauses; TASK-2036 consumes all
  executable/current Rust entries; TASK-2037 through TASK-2041 consume their assigned entries.
  Lean entries are handed to the deferred separate Lean project, not a Phase-205 deletion task.
- **Does not own:** specification amendments, Rust implementation, semantic traceability edits,
  or deletion.
- **Integration/proof responsibility:** TASK-2041 consumes the final manifest and proves its
  current/executable entries are zero after approved historical exceptions only.

## TDD and verification steps

1. Add a failing manifest-schema/self-consistency test and a fixture covering each classification
   and disposition.
2. Inventory direct AST entry points, public exports, CPS execution APIs, CLI/daemon/test/REPL
   callers, differential corpus and oracle material, Lean artifacts, and current documentation.
   Classify each Lean entry as `deferred_separate_project` unless it is stale current-Ash authority.
3. Write the manifest and human catalogue; explicitly list every deferred finite case rather than
   describing a class of generated cases.
4. Add a deterministic digest and tests that reject duplicate IDs, unowned entries, a missing
   deferred reason, or an executable/current entry without a disposition.
5. Run the focused audit test, orientation-index self-test, docs gate, and `git diff --check`.

## Completion checklist

- [x] The manifest has a verified revision, digest, schema, and complete finite entries.
- [x] Each entry has exactly one current/historical/deferred-separate-project classification and
      exactly one disposition.
- [x] Deferred cases are finite, named, and have an explicit second-phase owner.
- [x] Lean sources/docs have a clear deferred separate-project handoff and no deletion disposition.
- [x] No entry is reported as removed, implemented, tested, proved, or at spec parity by this
      planning/audit task alone.
- [x] PLAN-INDEX, CHANGELOG, and affected orientation paths are current.

## Completion evidence

The completed AUDIT-204 manifest contains 309 explicit records with a revision and sorted-entry
digest. Its Lean records are retained as `deferred_separate_project` entries with external
handoffs; no Lean source or documentation is a deletion disposition. Independent review accepted
the catalogue and its validator as the Phase-205 handoff; this task adds no runtime implementation,
test/proof claim, or target-spec parity claim.
