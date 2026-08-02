# TASK-2065: Module Realization Closeout

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103; PLAN-203
**Run-route impact:** none

## Semantic accounting

**Implementation:** not_implemented. **Evidence:** none. **Parity:** below_spec.
**Missing target-spec clauses:** all rule clauses not independently proven by TASK-2057–2064 remain below specification.
**Layers:** type/Core/CPS/admission-runtime `partial`; verification `not_implemented`.
**Evidence identifiers:** positive `TEST-MOD-REAL-CLOSEOUT-CORPUS`; negative `TEST-MOD-REAL-CLOSEOUT-SCANNER-GATE`; mutation `TEST-MOD-REAL-CLOSEOUT-TRACEABILITY`; parity `TEST-MOD-REAL-CLOSEOUT-CLI-DAEMON`.
**Next obligation:** no downstream phase owner; retain every unimplemented clause as an explicit deferral.

## Description

Close PLAN-207 only after every SPEC-103 clause has an owned implementation result, focused evidence, reviewed traceability, documentation status, and full repository-gate result. This task is a closeout and remediation owner, not a new semantic implementation path.

## Dependencies

- 📝 TASK-2057 through TASK-2064 — all implementation and integration handoffs.

## Requirements

1. Reconcile SPEC-103, PLAN-207, AUDIT-207, task records, semantic coverage, traceability, language reference support status, and changelog.
2. Confirm no semantic text scan, engine-private semantic export path, or direct-evaluator fallback remains reachable in the realized module route.
3. Confirm file/inline parity at source, interface, Core/CPS, admission, and CLI/daemon terminal layers.
4. Obtain independent code/spec review and map each finding to a fix, accepted explicit deferral, or contradiction in the target contract.
5. Mark a rule `implemented` only if every clause is realized; retain `partial`/`below_spec` otherwise.

## Closeout procedure

1. Read the final `MOD-REAL-*` records and compare them against every SPEC-103 invariant and conformance bullet.
2. Run the full rule-indexed corpus, mutation controls, and CLI/daemon parity case on the same admitted program.
3. Run an independent review over parser, graph, interface, binder, lowering, Engine, and documentation changes.
4. Fix blocking findings, rerun focused evidence, then rerun review.
5. Run workspace Rust, docs, orientation-index, traceability, and diff gates.
6. Update `CHANGELOG.md`, PLAN-INDEX progress, the plan status, coverage axes, and reference status only from recorded evidence.

## TDD Steps

1. Add a closeout checker that fails when a `MOD-REAL-*` clause has no implementation/evidence/parity
   axis, a planned task has no handoff record, a scanner is absent from AUDIT-207's denylist/allowlist,
   or a reference claim exceeds its evidence.
2. Run it before status edits; repair documentation and evidence until it passes.

## Completion checklist

- [ ] Every SPEC-103 clause has a traceable implementation, evidence, and parity conclusion.
- [ ] Independent code/spec review findings are resolved or explicit below-spec deferrals.
- [ ] Full Rust and documentation gates pass.
- [ ] The AUDIT-207 scanner denylist/allowlist proves no raw scanner can publish a graph, binding,
  interface, lowering, admission, or execution fact.
- [ ] Changelog, task/plan/spec indexes, coverage, traceability, and reference status agree.

## Handoffs

- **Consumes:** all PLAN-207 artifacts and TASK-2064 conformance reports.
- **Produces:** complete or explicitly partial phase status with review and verification evidence.
- **Downstream owner:** PLAN-203 retains future general runtime integration; a follow-on phase owns any deferred cycle, package, incremental-workspace, or dynamic-module work.
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
  - [ ] Every MOD-REAL rule has implementation, evidence, and parity status.
  - [ ] File/inline Core/CPS and terminal parity controls pass.
  - [ ] No raw-source semantic discovery or direct-evaluator fallback remains.
  - [ ] Independent review findings are resolved or explicitly retained as below-spec gaps.
  - [ ] PLAN-INDEX, SPEC-INDEX, coverage map, traceability, reference status, and CHANGELOG agree.
```
