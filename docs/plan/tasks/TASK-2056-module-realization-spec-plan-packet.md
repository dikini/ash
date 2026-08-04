# TASK-2056: Module Realization Spec and Plan Packet

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** [SPEC-103](../../spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md)
**Audit:** [AUDIT-207](../audits/AUDIT-207-module-realization-seams.md)
**Semantic task classification:** planning-only; no production semantic implementation

## Semantic accounting

**Canonical rules:** MOD-REAL-001 through MOD-REAL-006 (planning ownership only).
**Implementation:** not_implemented. **Evidence:** tested (documentation/index gates). **Parity:** below_spec.
**Missing target-spec clauses:** all implementation, lowering, admission, and client-parity clauses remain for TASK-2057–2069.
**Layers:** type/core/CPS/admission-runtime `not_applicable`; verification `partial` (packet and documentation gates only).
**Evidence identifiers:** positive `DOC-PLAN-207-PACKET`; negative `DOC-PLAN-207-INDEX-REJECTION`; mutation `DOC-PLAN-207-LINK-CHECK`; parity `not_applicable`.
**Next obligation:** TASK-2057 starts the AST-driven realization chain; no packet handoff claims module implementation.

## Description

Create the normative module realization contract, live seam audit, phase plan, task records, coverage-map rows, and orientation entries before any module Rust implementation begins.

## Requirements

- Define file-backed and inline module parity after source acquisition.
- Define AST authority, stable identity, export closure, visibility, module-machine transitions, Engine-only execution, and client parity.
- Inventory the live parser, resolver, graph, binder, summary, and Engine seams without treating legacy scans as semantic authority.
- Give every SPEC-103 rule a task owner and a named integration owner.

## Handoffs

- **Consumes:** SPEC-095b/095c/097b/098c/099b, PLAN-203, and current module implementation seams.
- **Produces:** SPEC-103, AUDIT-207, PLAN-207, TASK-2057 through TASK-2069, `MOD-REAL-*` coverage rows, indexes, and changelog routing. TASK-2067 through TASK-2069 repair the later-discovered unnamed ownership gaps without activating implementation.
- **Downstream owner:** TASK-2057 starts implementation from parsed module declarations.
- **Non-goals:** parser, resolver, typechecker, Core, CPS, Engine, CLI, or daemon behavior changes.

## Documentation verification

1. Check every new task ID has one task file and one PLAN-207 table entry.
2. Check SPEC-103 names the complete target rule and does not revive historical workflow syntax.
3. Check PLAN-207 assigns Type → Core → CPS → admission/runtime ownership and run-route impact.
4. Run the orientation-index validator, docs gate, and `git diff --check`.

## TDD Steps

1. Add a structural packet check before creating indexes: it must fail if a planned task has no
   record, a task lacks a PLAN-207 link, or a rule lacks an owner.
2. Create the packet until the check passes, then run the repository documentation gates.

## Completion checklist

- [x] SPEC-103 defines the operational-style module machine and file/inline parity invariant.
- [x] AUDIT-207 identifies all current semantic text-scan seams.
- [x] PLAN-207 and all fourteen task records exist with dependency closure.
- [x] PLAN, spec, semantic coverage, indexes, and changelog agree.
- [x] Documentation validation is recorded below.

## Dispatch

```text
agent: hermes
reasoning: high
toolsets: [terminal, file]
```

## Verification

```text
strictness: clean
commands:
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - git diff --check
```

## Verification evidence

2026-08-02 packet validation passed:

- `python3 tools/docs/validate_orientation_indexes.py --self-test` — passed.
- `bash scripts/check-docs-gate.sh` — passed; 1,820 changed-Markdown links and semantic traceability validated.
- `git diff --check` — passed.
- The initial 2026-08-02 structural packet check found TASK-2056 through TASK-2065. The
  2026-08-03 ownership repair extends that inventory through TASK-2069: all fourteen task files
  are linked from PLAN-207, and SPEC-103 still contains all six `MOD-REAL-*` rule IDs.
