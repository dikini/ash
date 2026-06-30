# TASK-1757: Preserve macro expansion origin and hygiene metadata through notation expansion

## Status: ✅ Complete

## Description

Harden Phase 172 macro products so macro expansion origins compose correctly with notation/operator-section expansion origins and generated helper names remain non-source-capturing. This task focuses on metadata integrity rather than adding new macro syntax.

## Specification Reference

- PLAN-172 D5
- SPEC-095c §11 desugaring invariants
- TASK-1745/TASK-1746 origin and generated identifier hygiene
- TASK-1756 macro expansion

## Dependencies

- ✅ TASK-1756: Expression-template macro expansion

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Full def-site/call-site hygiene | PLAN-171 | No binder model | No | Preserve origin chains and generated-name fences only | Tests inspect sidecars and capture negatives |
| Core provenance schema changes | PLAN-171 | Too broad | No | Keep surface-side sidecars | No Core schema diffs |
| Macro-generated binders | PLAN-172 | Binder hygiene absent | No | Reject | Negative tests |

## Requirements

1. Verify macro expansion products get stable `ExpansionId` sidecars and `SurfaceOrigin::MacroExpansion` origin data.
2. Verify notation/operator-section products generated inside macro expansions record the macro expansion as parent origin.
3. Verify macro expansion does not create source-spellable generated helper names.
4. Add negative tests for source bindings attempting to capture generated helpers after macro+notation expansion.
5. Reject non-parameter free variables in executable macro templates so macro products cannot silently capture call-site bindings.
6. Preserve macro-to-macro parent origins when a macro expands to another local macro invocation.
7. Keep all metadata surface-side; do not change Core provenance/runtime trace schemas.

## TDD Steps

### Step 1: Origin/hygiene tests RED

**Files:**
- `crates/ash-parser/tests/task_1757_macro_origin_hygiene.rs`

Test cases:
1. Macro expansion origin sidecar records call span and stable expansion id.
2. Macro output that triggers operator-section expansion yields child origin with macro parent.
3. Source identifiers cannot capture generated operator-section params produced inside macro expansion.
4. Macro template attempting binder introduction is rejected rather than pretending hygiene exists.
5. Nested macro expansion records the outer macro expansion as parent metadata.
6. Free template variables reject instead of resolving at the call site.

### Step 2: Patch metadata threading

Patch expansion traversal only where tests expose missing parent-origin or generated-name propagation.

### Step 3: Verify no Core changes

Run parser focused tests and `cargo check --workspace`.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1757_macro_origin_hygiene -- --nocapture
  - cargo test -p ash-parser
  - cargo check --workspace
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Macro origin sidecars are present.
  - [x] Nested notation/operator origins preserve macro parent.
  - [x] Nested macro origins preserve macro parent.
  - [x] Free template variables reject rather than capture call-site bindings.
  - [x] Generated helper names remain source-inaccessible.
  - [x] No Core/runtime provenance schema changes.
  - [x] CHANGELOG.md updated.
```

## Completion Evidence

Macro expansion now threads the active local notation table and parent macro origin into the expansion pass. Nested macro invocations preserve macro-to-macro parent origins, and notation/operator sections produced by macro templates are immediately elaborated with the macro expansion origin as parent metadata. Executable macro templates also reject non-parameter free variables instead of allowing call-site capture. This keeps nested macro/notation/operator origins chained to `SurfaceOrigin::MacroExpansion` without changing Core/runtime provenance schemas.

Added `crates/ash-parser/tests/task_1757_macro_origin_hygiene.rs` covering macro origin sidecars, nested macro origin parentage, notation origins generated inside macro products with macro parents, generated helper-name fencing/capture resistance, free-template-variable rejection, and fail-closed rejection of unsupported operational-bottom templates.

Verification passed:

```bash
cargo test -p ash-parser --test task_1756_expression_macro_expansion -- --nocapture
cargo test -p ash-parser --test task_1757_macro_origin_hygiene -- --nocapture
cargo check --workspace
cargo fmt --check
git diff --check
```

Focused TASK-1757 evidence: 6 tests passed.

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 30
toolsets: [terminal, file]
```

## Dependencies for Next Task

Provides metadata guarantees for cross-boundary validation in TASK-1758.
