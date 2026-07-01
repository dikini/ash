# TASK-1779: Audit and specify callable identity summaries for macro inference

## Status: ✅ Complete

## Description

Define what it means for an ordinary call expression inside a macro template to have a unique callable identity that can be used for bounded macro type inference. This task is an audit/specification task; TASK-1780 performs any implementation.

## Specification Reference

- [PLAN-174: Macro-Aware Tooling, Summary Identity, and Inference Readiness](../PLAN-174-MACRO-AWARE-TOOLING-SUMMARY-IDENTITY-AND-INFERENCE-READINESS.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095c: Surface AST, Macro Expansion, and Notation](../../spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)

## Dependencies

- ✅ TASK-1775: Macro-aware tooling audit

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Ordinary calls remain uninferred | TASK-1772 | A call like `add(x, 1)` did not prove a unique callable identity | Partial | Specify proof requirements before implementation | TASK-1780 may implement only audited positive cases |
| Imported/public callable summaries | Phase 170/173 module-loader work | Existing summaries serve runtime callability, not macro inference directly | Partial | Distinguish callable identity evidence from macro summary metadata | Audit must reject macro-as-callable conflation |

## Requirements

### Functional Requirements

1. Create `docs/audit/phase-174-callable-identity-summary-audit.md`.
2. Inventory callable summary sources in parser lowering, module-loader exports, engine imported closures, and typechecker call resolution.
3. Define a `CallableIdentityProof` decision table in prose: local function, builtin function, imported public function, overloaded/interface method, module-qualified path, macro summary, private helper, unresolved name.
4. Identify the minimum implementation hooks TASK-1780 may use without broad call-resolution rewrites.
5. Patch TASK-1780 with exact positive and negative fixtures if the initial task file is too broad.

### Property Requirements

- Macro summaries are not callable identity proofs.
- A unique callable identity must include enough type information to infer a macro result without guessing.
- Ambiguous, overloaded, private, unresolved, or module-qualified forms remain annotation-required unless the audit proves otherwise.

## TDD Steps

### Step 1: Inspect call-resolution and export surfaces

Use rust-analyzer and file reads to inspect relevant call-summary structs and functions in `ash-parser`, `ash-engine`, and `ash-typeck`.

### Step 2: Write the callable identity audit

Document each callable category, available evidence, safe inference status, and required tests.

### Step 3: Patch TASK-1780 if needed

Replace broad placeholders with exact test names and file targets based on the audit.

### Step 4: Verify the audit

Assert the audit names `TASK-1772`, `TASK-1780`, `MacroSummary`, and at least local, builtin, imported, overloaded, private, and unresolved callable categories.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - python3 -c 'from pathlib import Path; s=Path("docs/audit/phase-174-callable-identity-summary-audit.md").read_text(); assert "TASK-1772" in s and "TASK-1780" in s and "MacroSummary" in s and "local" in s and "builtin" in s and "imported" in s and "overloaded" in s and "private" in s and "unresolved" in s'
  - git diff --check
checklist:
  - [x] Callable identity audit created
  - [x] Safe positive cases and fail-closed negatives identified
  - [x] TASK-1780 patched with exact implementation gates if needed
```

## Dependencies for Next Task

TASK-1780 depends on this task's positive/negative decision table.

## Completion Evidence

- Created `docs/audit/phase-174-callable-identity-summary-audit.md` with proof categories for local, builtin, imported, overloaded, private, unresolved, and `MacroSummary` cases.
