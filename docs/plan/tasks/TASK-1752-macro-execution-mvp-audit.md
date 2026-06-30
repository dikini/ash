# TASK-1752: Audit macro execution seams and define the safe MVP subset

## Status: ✅ Complete

## Description

Audit the live parser, expansion, module-loader, lowering, and typechecker surfaces that would participate in parser-first macro execution. The output is an audit artifact that freezes the safe expression-template subset, unsupported shapes, and exact file targets before implementation begins.

## Specification Reference

- PLAN-172: `docs/plan/PLAN-172-PARSER-FIRST-MACRO-EXECUTION-MVP.md`
- `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md` §6
- `docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md` §10
- TASK-1744 audit and TASK-1750 closeout evidence

## Dependencies

- ✅ TASK-1751: Phase 172 plan packet

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Token-tree parser | Phase 171 review | Raw body was only conservative substring | No | Audit exact parser limits first | Artifact must list delimiter/string/comment behavior |
| Binder hygiene | PLAN-171 non-goals | No binder-origin model | No | Reject binder-introducing templates | Whitelist table required |
| Type-directed macro expansion | SPEC-095c future | No typed macro model | No | Keep parser-first | Audit must identify no typechecker dependency |

## Requirements

1. Inspect live code paths:
   - `crates/ash-parser/src/surface.rs`
   - `crates/ash-parser/src/parse_expr.rs`
   - `crates/ash-parser/src/parse_module.rs`
   - `crates/ash-parser/src/lower.rs`
   - `crates/ash-engine/src/module_loader.rs`
   - typechecker visitors that currently handle `Expr::MacroInvocation`.
2. Create `docs/audit/phase-172-macro-execution-mvp-audit.md`.
3. Classify every `Expr` variant as allowed, recursively allowed, rejected, or requires a later binder-hygiene phase for macro templates.
4. Decide whether only parenthesized `name!(...)` invocations execute in MVP and list bracket/brace behavior.
5. Map positive and negative tests required downstream.
6. Do not modify implementation code.

## TDD Steps

### Step 1: Inspect live seams

Use `read_file`, `search_files`, and LSP diagnostics/symbol tools where useful. Record current behavior rather than expected behavior.

### Step 2: Write audit artifact

**File:** `docs/audit/phase-172-macro-execution-mvp-audit.md`

Must include:
- current carriers and parsers;
- expansion order;
- high-level validation paths;
- safe template whitelist;
- fail-closed unsupported forms;
- task-to-file ownership table.

### Step 3: Patch downstream task details if needed

If the audit finds that a planned downstream task owns too much or too little, patch its requirements before implementation starts.

## Verification

```yaml
strictness: clean
commands:
  - test -f docs/audit/phase-172-macro-execution-mvp-audit.md
  - grep -q "Template whitelist" docs/audit/phase-172-macro-execution-mvp-audit.md
  - grep -q "Fail-closed unsupported forms" docs/audit/phase-172-macro-execution-mvp-audit.md
  - git diff --check
checklist:
  - [x] Audit artifact exists.
  - [x] Every macro execution consumer path is mapped.
  - [x] Safe template subset is explicit.
  - [x] Downstream tasks were patched if audit changed scope.
```

## Completion Evidence

Created `docs/audit/phase-172-macro-execution-mvp-audit.md`. The audit freezes the Phase 172 executable subset as local parenthesized expression-position macros only, records bracket/brace/qualified/imported/binder forms as fail-closed, classifies every current `Expr` variant for template safety, and maps task/file ownership for implementation. Verification passed:

```bash
test -f docs/audit/phase-172-macro-execution-mvp-audit.md
grep -q "Template whitelist" docs/audit/phase-172-macro-execution-mvp-audit.md
grep -q "Fail-closed unsupported forms" docs/audit/phase-172-macro-execution-mvp-audit.md
git diff --check
```

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides the scope and whitelist consumed by TASK-1753 through TASK-1758.
