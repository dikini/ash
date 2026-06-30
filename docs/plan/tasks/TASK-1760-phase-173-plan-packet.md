# TASK-1760: Create the Phase 173 macro-system expansion plan packet

## Status: ✅ Complete

## Description

Create and register PLAN-173 plus TASK-1760 through TASK-1773. This is documentation/planning work only; it must not implement macro semantics.

## Specification Reference

- [PLAN-173: Macro Summaries, Token Trees, Hygienic Binders, and Typed Macros](../PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)

## Dependencies

- ✅ TASK-1759: Phase 172 closeout (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Imported/exported macro activation | PLAN-172 non-goals | No macro summary carriers | Partially: local macro MVP exists | Implement in Phase 173 via explicit summaries | Positive import/export tests plus negative callable leakage tests |
| Token-tree / bracket / brace macro parsing | PLAN-172 non-goals | Raw carriers only, no token-tree parser | Partially: macro invocation carriers exist | Implement delimiter-preserving carriers before execution | Parser span/delimiter tests and fail-closed unsupported-shape tests |
| Binder-introducing macros | PLAN-172 non-goals | No binder hygiene metadata | Partially: origin chains and generated-name fences exist | Implement only after binder hygiene metadata model | Capture-resistance tests in both directions |
| Typed macro checking / inference | PLAN-172 non-goals | No typed macro summaries | No | Implement after spec and typed signature carriers | Typecheck diagnostics before expansion/Core acceptance |

## Requirements

### Functional Requirements

- [x] PLAN-173 exists and references every task
- [x] All task files render from column 0
- [x] PLAN-INDEX records Phase 173 as 1/14 complete
- [x] CHANGELOG records the planning packet

### Property Requirements

- Macro metadata must remain syntax-phase metadata and must not grant rows, authority, contracts, failures, proof evidence, or runtime provider effects.
- Unsupported or ambiguous macro shapes must fail before Core lowering and before public export acceptance.
- Positive visibility claims require matching negative leakage tests.

## TDD Steps

### Step 1: Inspect current state / write failing evidence

**Files:**
- `docs/plan/PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md`
- `docs/plan/PLAN-INDEX.md`
- `CHANGELOG.md`
- `docs/plan/tasks/TASK-1760-*.md through TASK-1773-*.md`

Current state must be measured from live code before editing. For implementation tasks, write failing parser/engine/typeck tests that demonstrate the exact missing behavior or boundary leak.

### Step 2: Implement or document the minimal scoped change

Keep the task scoped to its listed deliverable. Do not pull later Phase 173 tasks forward unless this task explicitly owns their carriers and tests.

### Step 3: Integrate downstream consumers

Update parser, engine/module-loader, typechecker, lowering, and LSP-facing consumers only as required by this task's public carrier changes.

### Step 4: Verify

Run the commands listed below and record exact evidence in this task file before marking it complete.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - python3 - <<'PY'
from pathlib import Path
ids=range(1760,1774)
missing=[i for i in ids if not list(Path("docs/plan/tasks").glob(f"TASK-{i}-*.md"))]
assert not missing, missing
plan=Path("docs/plan/PLAN-173-MACRO-SUMMARIES-TOKEN-TREES-HYGIENIC-BINDERS-TYPED-MACROS.md").read_text()
for i in ids: assert f"TASK-{i}" in plan
PY
  - git diff --check
checklist:
- [x] PLAN-173 exists and references every task
- [x] All task files render from column 0
- [x] PLAN-INDEX records Phase 173 as 1/14 complete
- [x] CHANGELOG records the planning packet
```

## Completion Evidence

This task is complete because this planning slice created PLAN-173, TASK-1760 through TASK-1773, the PLAN-INDEX Phase 173 row/section, and the CHANGELOG planning entry.

## Notes

Keep Phase 173 conservative. If this task discovers that its scope requires a broader macro system than specified, stop and patch PLAN-173/TASK-1761 with a split recommendation before implementation continues.
