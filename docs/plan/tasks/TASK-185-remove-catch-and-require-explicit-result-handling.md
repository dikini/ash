# TASK-185: Remove `catch` and Require Explicit Result Handling

## Status: ✅ Complete

## Description

Remove `attempt`/`catch` from the canonical language. Recoverable failures must be represented
explicitly as `Result` values and handled with pattern matching.

## Specification Reference

- SPEC-002: Surface Language
- SPEC-004: Operational Semantics
- SPEC-014: Behaviours
- SPEC-016: Output Capabilities
- SPEC-017: Capability Integration
- SPEC-020: Algebraic Data Types

## Requirements

### Functional Requirements

1. Remove `attempt`/`catch` from canonical syntax, semantics, and examples
2. Make explicit `Result<T, E>` handling the canonical recoverable-failure path
3. Ensure remaining hardening tasks assume the language has no `catch` construct
4. Keep migration traceability out of normative specs and in this task record

## TDD Evidence

### Red

Before this change, the canonical language still contained `attempt`/`catch` in the surface
keyword set, operational semantics, and several behaviour/output/capability examples.

### Green

The canonical language now uses explicit `Result` values and pattern matching for recoverable
failures:

- `attempt`/`catch` are removed from SPEC-002 and SPEC-004
- behaviour, output, and capability examples now use `Result` and `match`
- SPEC-020 states that `Result<T, E>` is the canonical recoverable error-handling mechanism
- the hardening roadmap and this task record assume no canonical `catch` construct remains

## Migration Notes

The Rust implementation still has legacy `Attempt` / `catch` concepts in parser, AST, desugaring,
effect, and runtime-adjacent code paths. Those implementation details are intentionally not
documented in the canonical specs here; later Rust-alignment work must migrate them to explicit
`Result` handling.

## Files

- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/tasks/TASK-185-remove-catch-and-require-explicit-result-handling.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-014-BEHAVIOURS.md`
- Modify: `docs/spec/SPEC-016-OUTPUT.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`

## TDD Steps

### Step 1: Write the failing checklist (Red)

Check for:

- `attempt` or `catch` still present in canonical surface or semantics
- recoverable failures still described as a separate language construct
- remaining hardening tasks not yet assuming explicit `Result` handling

### Step 2: Verify RED

Expected failure conditions:

- canonical docs still mention `attempt`/`catch`

### Step 3: Implement the minimal spec fix (Green)

Replace the canonical `attempt`/`catch` story with explicit `Result` and `match`.

### Step 4: Verify GREEN

Expected pass conditions:

- canonical specs no longer define an `attempt`/`catch` construct
- recoverable handling is explicit and Result-based
- remaining task/index wording assumes no `catch`

### Step 5: Commit

```bash
git add CHANGELOG.md docs/plan/PLAN-INDEX.md docs/plan/tasks/TASK-185-remove-catch-and-require-explicit-result-handling.md docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-014-BEHAVIOURS.md docs/spec/SPEC-016-OUTPUT.md docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md docs/spec/SPEC-020-ADT-TYPES.md
git commit -m "docs: remove catch from canonical language"
```

## Completion Checklist

- [x] `attempt`/`catch` removed from canonical specs
- [x] explicit `Result` handling documented
- [x] migration notes kept in task record
- [x] `CHANGELOG.md` updated

## Non-goals

- No Rust parser or runtime code changes
- No new exception mechanism

## Dependencies

- Depends on: TASK-177, TASK-178, TASK-181, TASK-182, TASK-183, TASK-184
- Blocks: future Rust migration tasks that still reference `Attempt`/`catch`
