# TASK-181: Formalize ADT Dynamic Semantics

## Status: 📝 Planned

## Description

Tighten ADT constructor evaluation, runtime value shape, pattern matching, `if let`, and
exhaustiveness-facing dynamic behavior so the language definition supports both Rust and Lean
implementations without local semantic choice.

## Specification Reference

- SPEC-003: Type System
- SPEC-004: Operational Semantics
- SPEC-020: ADT Types

## Requirements

### Functional Requirements

1. Define canonical runtime constructor/value behavior
2. Define one canonical dynamic pattern-matching story
3. Define `if let` dynamic behavior precisely
4. Keep ADT semantics aligned with the canonical source enum model

## Files

- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`
- Modify: `docs/reference/parser-to-core-lowering-contract.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `docs/reference/runtime-observable-behavior-contract.md`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for ambiguity in:
- constructor evaluation,
- variant value shape,
- pattern-match dynamic behavior,
- `if let`,
- exhaustiveness-facing runtime assumptions.

### Step 2: Verify RED

Expected failure conditions:
- at least one ADT dynamic boundary still depends on prose interpretation.

### Step 3: Implement the minimal spec fix (Green)

Tighten only ADT dynamic semantics.

### Step 4: Verify GREEN

Expected pass conditions:
- ADT runtime and pattern semantics are explicit enough for mechanical implementation.

### Step 5: Commit

```bash
git add docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-020-ADT-TYPES.md docs/reference/parser-to-core-lowering-contract.md docs/reference/type-to-runtime-contract.md docs/reference/runtime-observable-behavior-contract.md CHANGELOG.md
git commit -m "docs: formalize adt dynamic semantics"
```

## Completion Checklist

- [ ] constructor/runtime value semantics documented
- [ ] pattern and `if let` dynamic semantics documented
- [ ] ADT runtime/source-model alignment documented
- [ ] `CHANGELOG.md` updated

## Non-goals

- No Rust type-checker changes
- No stdlib code updates

## Dependencies

- Depends on: TASK-177, TASK-178
- Blocks: TASK-174, TASK-175, TASK-184
