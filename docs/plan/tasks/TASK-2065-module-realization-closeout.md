# TASK-2065: Module Realization Closeout

**Status:** Complete for the frozen callable-module completion domain
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103; PLAN-203
**Run-route impact:** none
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2065](../SEMANTIC-RULE-COVERAGE.md#task-2065-module-realization-closeout)

## Semantic accounting

**Implementation:** implemented
**Evidence:** tested
**Parity:** matches_spec
**Completion scope:** Closeout evaluates the frozen callable-module route and the checked metadata
dependencies it requires. Dedicated role/policy declarations and dynamic module loading are not
part of the closeout domain; TASK-2077 removes those surfaces. Static file-backed and inline module
acquisition remains in scope.
The closeout must still account for public-declaration propagation: each public declaration needed
by an importer must retain canonical identity, origin, visibility, and checked metadata through
finalization and import transport, even when it has no standalone execution semantics.
**Layers:** type `implemented`; Core `implemented`; CPS `implemented`; admission-runtime `implemented`;
  verification `implemented`.
**Missing target-spec clauses:** None within the frozen Phase 207 closeout and evidence domain.
  Bodyless `BuiltinFn` host dispatch/runtime semantics, raw synthesized-pattern compatibility APIs,
  dynamic module loading remains an explicit follow-on exclusion.
**Recorded missing clause:** None within the frozen Phase 207 closeout domain.
**Recorded closeout boundary:** The checker is an activation/closeout control only; it creates no
runtime or admission authority. `--require-complete` is green for the frozen route.
**Evidence identifiers:** positive `TEST-MOD-REAL-CLOSEOUT-CHECKER-CONTRACT` and
`TEST-MOD-REAL-CLOSEOUT-COMPLETION-SCOPE`;
negative `TEST-MOD-REAL-CLOSEOUT-SCANNER-INVENTORY`; mutation
`TEST-MOD-REAL-CLOSEOUT-READINESS-GATE`; parity `TEST-MOD-REAL-CLOSEOUT-REFERENCE-BOUNDARY`.
The closeout checker validates the semantic axes, active-task handoff records, AUDIT-207 scanner
inventory, and the reference boundary. It reports historical partial handoffs without treating
them, host builtin runtime dispatch, or dynamic loading as frozen completion work.
**Next obligation:** None within Phase 207; retain the documented follow-on boundary for host
builtin runtime dispatch, dynamic loading, and generalized runtime features.
**Recorded next obligation:** None within Phase 207.

## Description

Close PLAN-207 only after every SPEC-103 clause has an owned implementation result, focused evidence, reviewed traceability, documentation status, and full repository-gate result. This task is a closeout and remediation owner, not a new semantic implementation path.

## Dependencies

- ✅ TASK-2057 through TASK-2069, TASK-2070 through TASK-2073, plus TASK-2064 conformance — all
  implementation and integration handoffs. TASK-2068 is the completed foundation; the frozen
  route-owner records and their evidence are complete, so this closeout record documents the
  result rather than waiting on an unfinished prerequisite.

## Requirements

1. Reconcile the in-scope callable-route portions of SPEC-103, PLAN-207, AUDIT-207, task records,
   semantic coverage, traceability, language reference support status, and changelog. Record
   dynamic module loading as explicit excluded follow-on work rather than a closeout blocker.
2. Confirm no semantic text scan, engine-private semantic export path, or direct-evaluator fallback remains reachable in the realized module route.
3. Confirm file/inline parity at source, interface, Core/CPS, admission, and CLI/daemon terminal layers.
4. Obtain independent code/spec review and map each finding to a fix, accepted explicit deferral, or contradiction in the target contract.
5. Mark a rule `implemented` only if every clause is realized; retain `partial`/`below_spec` otherwise.

## Closeout procedure

1. Read the final in-scope `MOD-REAL-*` records and compare them against the callable-route
   SPEC-103 invariants and conformance bullets. Do not require dynamic module loading.
2. Run the full rule-indexed corpus, mutation controls, and CLI/daemon parity case on the same admitted program.
3. Run an independent review over parser, graph, interface, binder, lowering, Engine, and documentation changes.
4. Fix blocking findings, rerun focused evidence, then rerun review.
5. Run workspace Rust, docs, orientation-index, traceability, and diff gates.
6. Update `CHANGELOG.md`, PLAN-INDEX progress, the plan status, coverage axes, and reference status only from recorded evidence.

## TDD Steps

1. Add a closeout checker that fails when a completion-owner `MOD-REAL-*` clause has no implementation/evidence/parity
   axis, a planned task has no handoff record, a scanner is absent from AUDIT-207's denylist/allowlist,
   or a reference claim exceeds its evidence.
2. Run it before status edits; repair documentation and evidence until it passes. Audit mode must
   remain green while reporting incomplete work, while `--require-complete` must fail until every
   completion-owner callable rule is complete.

## Completion checklist

- [x] Every in-scope callable-route SPEC-103 clause has a traceable implementation, evidence, and
  parity conclusion; dynamic module loading is recorded as excluded follow-on work.
- [x] Independent code/spec review findings are resolved or explicit below-spec deferrals.
- [x] Full Rust and documentation gates pass.
- [x] The AUDIT-207 scanner denylist/allowlist proves no raw scanner can publish a graph, binding,
  interface, lowering, admission, or execution fact.
- [x] Changelog, task/plan/spec indexes, coverage, traceability, and reference status agree.

## Handoffs

- **Consumes:** all PLAN-207 artifacts and TASK-2064 conformance reports.
- **Produces:** complete or explicitly partial phase status with review and verification evidence.
- **Downstream owner:** PLAN-203 retains future general runtime integration; a follow-on phase owns any deferred cycle, package, incremental-workspace, or dynamic-module work.
- **Explicit exclusion:** Dynamic module loading is not part of Phase 207 closeout.
- **Non-goals:** implementation shortcuts, semantic widening, or relabeling a partial rule as complete.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - git diff --check
checklist:
  - [x] Every in-scope callable-route MOD-REAL rule has implementation, evidence, and parity status.
  - [x] File/inline Core/CPS and terminal parity controls pass.
  - [x] No raw-source semantic discovery or direct-evaluator fallback remains.
  - [x] Independent review findings are resolved or explicitly retained as below-spec gaps.
  - [x] PLAN-INDEX, SPEC-INDEX, coverage map, traceability, reference status, and CHANGELOG agree.
```

The implemented checker is `tools/docs/check_phase_207_closeout.py`; its contract tests are in
`tools/docs/tests/test_phase_207_closeout.py`.
