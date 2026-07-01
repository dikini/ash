# TASK-1784: Create the Phase 175 semantic-identity plan packet

## Status: ✅ Complete

## Description

Create and register the Phase 175 planning packet for name-resolution-backed semantic identity for macros and tooling. This is documentation/planning only and does not implement Rust behavior.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- ✅ Phase 174 closeout and push to origin/main (complete)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Create the PLAN-175 document with scope, non-goals, decision gates, tasks, and verification baseline.
2. Create TASK-1784 through TASK-1793 task files with dependencies, requirements, dispatch metadata, and verification commands.
3. Register Phase 175 in PLAN-INDEX progress and detail sections.
4. Add a CHANGELOG.md planning entry under [Unreleased].

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Inspect current planning state

Read PLAN-174, PLAN-INDEX, CHANGELOG, and current TASK-178x allocation before assigning globally unique task IDs.

### Step 2: Write planning artifacts

Create the Phase 175 plan and task files with conservative identity-first scope.

### Step 3: Register planning surfaces

Update PLAN-INDEX and CHANGELOG after all task files exist.

### Step 4: Verify structure

Run structural checks that every task link resolves and PLAN-INDEX/CHANGELOG mention Phase 175.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Plan file exists
  - [x] Task files TASK-1784 through TASK-1793 exist
  - [x] PLAN-INDEX row and phase section exist
  - [x] CHANGELOG entry exists
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Completion Evidence

Created the Phase 175 planning packet, task files, PLAN-INDEX entries, and CHANGELOG planning entry. Implementation tasks remain planned.
