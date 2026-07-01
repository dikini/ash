# TASK-1785: Audit macro/callable identity surfaces and current name-resolution seams

## Status: 📝 Planned

## Description

Audit parser, engine/module-loader, LSP, and summary consumers to locate every current identity carrier and every name-resolution seam that Phase 175 might use. This task produces the ownership map before implementation.

## Specification Reference

- [PLAN-175: Name-Resolution-Backed Semantic Identity for Macros and Tooling](../PLAN-175-NAME-RESOLUTION-BACKED-SEMANTIC-IDENTITY-FOR-MACROS-AND-TOOLING.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-038: Language Server](../../spec/SPEC-038-LANGUAGE-SERVER.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)

## Dependencies

- ✅ TASK-1784: Phase 175 plan packet

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-only LSP references | Phase 174 closeout | No resolved semantic identity substrate | This task contributes to substrate | Replace only when resolved identity is proven | Same-name macro/function negative tests |
| Cross-file macro navigation | Phase 174 non-goal | No imported macro identity contract or workspace graph | Summary identity can be designed; graph still absent | Prepare summary-identity boundary only | Unsupported cases return honest no-result |

## Requirements

### Functional Requirements

1. Create `docs/audit/phase-175-semantic-identity-surface-audit.md`.
2. Inventory `Definition::Macro`, `MacroSummary`, `MacroSummaryKey`, `CallableTypeSummary`, LSP symbol entries, goto/reference helpers, and module-loader macro summary transport.
3. Classify surfaces as declaration identity, use-site identity, imported-summary identity, cache identity, or runtime callable identity.
4. Patch TASK-1786 through TASK-1791 if the audit changes the implementation order or scope.

### Property Requirements

- Macro identity must remain syntax-phase metadata.
- Macro identity must not imply runtime callability or callable export authority.
- Any reference/navigation behavior must fail closed when identity is ambiguous or unavailable.

## TDD Steps

### Step 1: Inspect code surfaces

Use read_file/search_files/LSP tools for parser, LSP, and engine identity carriers.

### Step 2: Write audit artifact

Record findings, risk table, and owner task mapping in `docs/audit/phase-175-semantic-identity-surface-audit.md`.

### Step 3: Patch downstream tasks if needed

Adjust task requirements before code work begins if current code contradicts the plan.

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
  - python3 -c 'from pathlib import Path; s=Path("docs/audit/phase-175-semantic-identity-surface-audit.md").read_text(); assert "MacroSummary" in s and "CallableTypeSummary" in s and "goto.rs" in s and "references" in s'
  - git diff --check
checklist:
  - [ ] Audit artifact created
  - [ ] Identity surfaces classified
  - [ ] Downstream task ownership checked
```

## Dependencies for Next Task

This task feeds the following Phase 175 tasks according to the dependency table in PLAN-175.

## Notes

Do not implement identity carriers in this audit task.
