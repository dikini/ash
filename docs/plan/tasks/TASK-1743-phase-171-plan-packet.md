# TASK-1743: Create the Phase 171 plan and task packet

## Status: ✅ Complete

## Description

Create the conservative Phase 171 implementation-grade packet for macro/notation hygiene, expansion-origin preservation, and notation/macro scope boundaries. This task creates the plan, task files, PLAN-INDEX registration, and changelog entry. It does not implement the hygiene substrate.

## Specification Reference

- PLAN-171: `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- Phase 170 closeout: `docs/plan/PLAN-170-EXPANDED-SURFACE-INTEGRATION-AND-NOTATION-SCOPING.md`

## Dependencies

- ✅ Phase 170: Expanded surface integration and notation scoping (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro hygiene | PLAN-170 follow-ons | No complete macro execution substrate | Partially: expansion boundary and origin sidecars exist | Plan conservative hygiene substrate now; defer full macro execution | TASK-1748 must fail-closed for macro invocation without execution |
| Imported notation propagation | TASK-1740 / Phase 170 | Summary carriers not ready | No | Keep local-only; test negative leakage | TASK-1747/TASK-1749 |
| Generalized mixfix sections | SPEC-095c / PLAN-170 | Binder/hygiene rules absent | No | Keep out of Phase 171 | TASK-1744 audit and TASK-1750 closeout |

## Requirements

1. Create `docs/plan/PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md`.
2. Create TASK-1743 through TASK-1750.
3. Register Phase 171 in `docs/plan/PLAN-INDEX.md` with progress `1/8`.
4. Add a `[Unreleased]` changelog entry for the packet.
5. Include conservative non-goals so future agents do not implement full macros by accident.
6. Include structural verification that every referenced task file exists.

## Docs-only steps

1. Inspect Phase 170 follow-on text and existing Phase 168–170 packet structure.
2. Allocate globally unique task IDs after TASK-1742.
3. Write the plan and task files.
4. Update `PLAN-INDEX.md` and `CHANGELOG.md`.
5. Run docs and structural verification.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
  - python3 -c 'from pathlib import Path; root=Path("docs/plan"); plan=(root/"PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md").read_text(); assert all(f"TASK-{i}" in plan for i in range(1743, 1751)); assert all((root/"tasks"/p).exists() for p in ["TASK-1743-phase-171-plan-packet.md","TASK-1744-hygiene-origin-scope-audit.md","TASK-1745-expansion-identity-origin-chain.md","TASK-1746-generated-identifier-hygiene-fences.md","TASK-1747-notation-macro-scope-boundaries.md","TASK-1748-fail-closed-macro-invocation-boundary.md","TASK-1749-cross-boundary-hygiene-validation.md","TASK-1750-phase-171-closeout.md"]); idx=(root/"PLAN-INDEX.md").read_text(); assert "PLAN-171-MACRO-NOTATION-HYGIENE-AND-EXPANSION-BOUNDARIES.md" in idx and "1 | 📝 Planned" in idx'
checklist:
  - [x] PLAN-171 exists.
  - [x] TASK-1743 through TASK-1750 exist.
  - [x] PLAN-INDEX registers Phase 171.
  - [x] CHANGELOG.md records the packet.
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Dependencies for Next Task

This task produces the implementation-grade handoff consumed by TASK-1744.
