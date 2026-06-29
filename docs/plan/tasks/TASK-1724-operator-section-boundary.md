# TASK-1724: Add the binary infix operator-section AST boundary or fail-closed diagnostics

## Status: ✅ Completed

## Summary

Add an honest implementation boundary for binary infix operator sections. The phase may either parse
operator sections into a source-preserving AST carrier or reject them explicitly until notation
resolution exists, but it must not silently accept and erase them.

## Specification Reference

- PLAN-168: `docs/plan/PLAN-168-SURFACE-AST-NOTATION-SUBSTRATE.md`
- SPEC-095c: `docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md` §11
- TASK-1723 notation token preservation

## Dependencies

- 📝 TASK-1723: Notation/operator token preservation

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Generalized mixfix sections | SPEC-095c §11 | Binder/mixfix semantics unresolved | No | Keep deferred | Negative tests reject generalized mixfix sections or leave them unparsed |
| Binary infix sections | SPEC-095c §11 | Needed by target surface spec | Partial | Add AST boundary or fail-closed diagnostic now | Focused tests cover bare, left, right, and non-goal forms |

## Files

- `crates/ash-parser/src/surface.rs`
- Relevant expression/parser modules identified by TASK-1721
- `crates/ash-parser/tests/task_1724_operator_section_boundary.rs`

## Requirements

1. Cover the four binary infix section forms from `SPEC-095c`:
   - `a <op> b`
   - `(a <op>)`
   - `(<op> b)`
   - `(<op>)`
2. If sections are represented, include `Bare`, `Left`, and `Right` shape and source spans.
3. If sections are not represented yet, reject them with clear diagnostics and no fallback parse as
   ordinary parenthesized expressions.
4. Explicitly reject or defer generalized mixfix sections.
5. Keep callable typing and row semantics out of this task unless a later implementation task owns
   type inference.

## TDD Steps

### Step 1: Write tests (Red)

Create `crates/ash-parser/tests/task_1724_operator_section_boundary.rs` covering bare, left, right,
full infix, and generalized-mixfix non-goal forms.

### Step 2: Implement (Green)

Implement the parser boundary chosen by the carrier design: AST representation or fail-closed
rejection. Do not silently desugar to function calls in the parser.

### Step 3: Integration

Update parser exports/downstream match statements only as required by the chosen boundary.

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-parser --test task_1724_operator_section_boundary
  - cargo test -p ash-parser
  - cargo fmt --check
  - git diff --check
  - bash scripts/check-docs-gate.sh
checklist:
  - [ ] Bare, left, and right section cases are represented or rejected explicitly.
  - [ ] Generalized mixfix sections remain deferred/fail-closed.
  - [ ] No parser path silently erases section shape.
```

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 25
toolsets: [terminal, file]
```

## Dependencies for next task

Produces the operator-section boundary consumed by TASK-1725.
