# TASK-1723: Preserve notation/operator token shape before resolution

## Status: ✅ Completed

## Summary

Implement or stage the minimal parser substrate needed to preserve notation-relevant operator tokens
and grouping before semantic notation resolution. This is not full user-defined notation; it is the
shape-preservation layer that prevents premature erasure.

## Specification Reference

- PLAN-168: `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md` §7-§11
- TASK-1722 carrier design

## Dependencies

- 📝 TASK-1722: Source-preserving surface carrier design

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| User-defined prefix/infix/suffix/mixfix notation | SPEC-095c §7-§10 | Requires notation table and expansion | Partial | Preserve raw shape now; defer semantic resolution | Parser tests show raw/grouped shape is preserved or rejected explicitly |

## Files

- `crates/ash-parser/src/surface.rs`
- `crates/ash-parser/src/lexer.rs`
- Relevant expression/parser modules identified by TASK-1721
- `crates/ash-parser/tests/task_1723_notation_token_preservation.rs`

## Requirements

1. Preserve operator-like tokens and grouping in parser-visible carriers before any notation
   resolution.
2. Do not interpret user-defined notation semantically in this task.
3. Existing accepted syntax must remain accepted unless a precise ambiguity is intentionally changed
   to a fail-closed diagnostic.
4. Add focused parser tests for representative operator-like token/grouping cases.
5. Document any remaining unsupported token classes as explicit follow-up, not silent erasure.

## TDD Steps

### Step 1: Write tests (Red)

Create `crates/ash-parser/tests/task_1723_notation_token_preservation.rs` with focused tests for the
current parser boundary discovered by TASK-1721. Tests should prove either:

- operator/grouping shape survives in the new carrier; or
- unsupported target notation forms fail closed with an explicit diagnostic.

### Step 2: Implement (Green)

Patch the parser surface carriers and parser modules needed for the minimal shape-preservation slice.
Do not add notation resolution.

### Step 3: Integrate

Update exports or downstream adapters only as needed to compile and preserve existing behavior.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1723_notation_token_preservation
  - cargo test -p ash-parser
  - cargo fmt --check
  - git diff --check
  - bash scripts/check-docs-gate.sh
checklist:
  - [ ] Focused parser tests execute and pass.
  - [ ] Operator-like token/grouping shape is preserved or explicitly rejected.
  - [ ] No full notation-resolution behavior is introduced.
```

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the token/grouping substrate consumed by TASK-1724.
