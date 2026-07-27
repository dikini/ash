# TASK-2031F: Stdlib Admission Contract Reconciliation

**Status:** Complete
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Type:** Bounded test-contract remediation
**Depends on:** TASK-2003 bounded `PureAnf` lowering and TASK-2004/TASK-2014 Path-B closed admission
**Blocks:** TASK-2031 workspace-gate closeout

## Description

Align three stdlib callable integration assertions with the current checked Core-to-CPS bridge
domain. `string::concat`, `list::len`, and `predicate::is_int` correctly typecheck and then reject
at the existing closed admission boundary, but their test constant still names an obsolete
atomic-only bridge message.

## Semantic rule and bounded domain

**Canonical owners:** `SEM-TARGET-CORE-CPS-001`, TASK-2003 bounded typed `PureAnf`, and
TASK-2004/TASK-2014 Path-B closed admission.

**Declared domain:** **bounded** to three existing stdlib callable negative assertions. It changes
neither Type → Core → CPS lowering, admission authority, runtime behavior, terminal projection,
nor CLI/daemon parity; it records existing negative evidence only.

**Handoffs:** Consumes TASK-2003/TASK-2004/TASK-2014's existing closed-admission result. Produces
corrected stdlib negative evidence only. **Run-route impact:** **none**. TASK-2032 remains the
separately owned client-parity integration owner; no runtime or proof responsibility transfers.

## Requirements

1. Preserve parse/check success and closed admission rejection for all three existing controls.
2. Assert the current generic PureAnf bridge-domain message exactly.
3. Do not alter production lowering, admission, stdlib behavior, Engine execution, or terminal routes.

## TDD steps

1. **RED:** Run `module_resolution`; demonstrate the three stale-message failures.
2. **GREEN:** Replace only the test-local obsolete expected message.
3. Run focused, workspace, formatter, Clippy, docs, and review gates.

## Completion checklist

**Completion evidence:** `module_resolution` passed 17/17. The `string::concat`, `list::len`,
and `predicate::is_int` controls retain parse/check success followed by the exact shared current
PureAnf bridge-domain diagnostic. Historical change `de4043d8` changes only the test-local
expected-message constant. This task changes no production lowering, admission, stdlib, Engine,
terminal, or client-parity behavior.

- [x] All three controls retain their existing parse/check and closed-admission assertions.
- [x] No production semantics change.
- [x] Workspace gates and review evidence are clean.
