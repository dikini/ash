# TASK-1792: Reconcile specs, docs, indexes, and changelog for Phase 175

## Status: ✅ Complete

## Description

Update specs, audits, task files, PLAN-175, PLAN-INDEX, and CHANGELOG to reflect the implemented semantic identity substrate and its limits.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- 📝 TASK-1785 through TASK-1791 complete

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Update `SPEC-095c` with macro declaration identity and non-callability wording.
2. Update `SPEC-038` with semantic same-file reference behavior and unsupported cross-file limitations.
3. Update `SPEC-INDEX.md` read paths/tags if new identity guidance changes navigation.
4. Update task files, PLAN-175, PLAN-INDEX, and CHANGELOG consistently.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Inspect implemented behavior

Read final code/tests and audit artifacts before writing docs.

### Step 2: Patch specs and indexes

Update only behavior that is actually implemented.

### Step 3: Patch status surfaces

Mark tasks complete only after verification evidence exists.

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
  - [x] Specs describe implemented identity behavior only
  - [x] PLAN-INDEX and PLAN-175 status agree
  - [x] CHANGELOG has Phase 175 entries
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Completion Evidence

- Reconciled SPEC-095c, SPEC-038, PLAN-175, PLAN-INDEX, task files, and CHANGELOG for the Phase 175 implementation slice.
