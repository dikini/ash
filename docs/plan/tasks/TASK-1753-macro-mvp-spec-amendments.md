# TASK-1753: Amend macro specs for parser-first expression macro MVP

## Status: ✅ Complete

## Description

Amend SPEC-095c and SPEC-098c to describe the Phase 172 executable macro MVP in implementation-grade terms. The specs must state the supported declaration/invocation syntax, expansion order, local-only scope, allowed template subset, and fail-closed behavior without implying full macro expansion.

## Specification Reference

- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md`
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md`
- `docs/spec/SPEC-INDEX.md`
- TASK-1752 audit artifact

## Dependencies

- ✅ TASK-1751: Phase 172 plan packet
- ✅ TASK-1752: Macro execution seam audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full macro expansion | SPEC-095c | Not implemented | Partially | Specify expression macro MVP only | Spec text must list non-goals |
| Core macro lowering | SPEC-098c | Core must not see macros | No | Keep Core macro-free | SPEC-098c must require pre-Core expansion/rejection |
| Imported macros | SPEC-095c import/export | No summary carriers | No | Local-only | Negative tests in TASK-1758 |

## Requirements

1. Add a Phase 172 subsection to SPEC-095c §6 describing parser-first expression macros.
2. State the tentative grammar:
   - `MacroDecl ::= visibility? "macro" name "(" ParamList? ")" "=>" Expr ";"`
   - executable invocation subset: `name!(ExprList?)`.
3. State that bracket/brace macro invocations remain fail-closed unless later tasks implement token-tree parsing.
4. State that macro template execution is syntax-only and authority-neutral.
5. Amend SPEC-098c §10 so Core lowering receives no macro declarations or invocations; unsupported macro constructs reject before Core.
6. Update SPEC-INDEX tags/read paths if needed.
7. Update CHANGELOG.md.

## TDD Steps

### Step 1: Spec edits

Patch SPEC-095c and SPEC-098c with normative MVP constraints.

### Step 2: Stale-claim sweep

Search specs/plans for wording that implies full macro execution or token-tree parsing and narrow it.

### Step 3: Index/changelog

Update SPEC-INDEX and CHANGELOG.

## Verification

```yaml
strictness: clean
commands:
  - grep -q "parser-first expression macro" docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
  - grep -q "MacroDecl" docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
  - grep -q "before Core" docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Specs state supported MVP syntax.
  - [x] Specs state fail-closed unsupported forms.
  - [x] Specs do not claim full macro expansion.
  - [x] CHANGELOG.md updated.
```

## Completion Evidence

Patched `SPEC-095c` with the Phase 172 parser-first expression macro MVP syntax and constraints, patched `SPEC-098c` to keep Core lowering macro-free while allowing supported local macro expansion before Core, updated `SPEC-INDEX.md`, and added a changelog entry. Verification passed:

```bash
grep -q "parser-first expression macro" docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
grep -q "MacroDecl" docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
grep -q "before Core" docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for Next Task

Normative syntax and boundary contract consumed by TASK-1754+.
