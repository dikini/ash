# TASK-1809: Add surface computation-row parser and AST carriers

## Status: ✅ Complete

## Description

Add source-preserving parser and surface AST carriers for target computation rows. The first slice covers inline callable rows and expanded `where row { ... }` blocks without granting authority or requiring full row inference.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [NOTE-021: Row, Callable, Where, and Fact Syntax](../../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)

## Dependencies

- TASK-1807 seam audit complete.
- TASK-1808 implementation decisions recorded.

## Requirements

### Functional Requirements

1. Add parsed surface carriers for row items, row tails, inline row spans, and expanded `where row` spans in `crates/ash-parser/src/surface.rs`.
2. Extend parsing in `crates/ash-parser/src/parse_module.rs` and any type/function parsing helpers identified by TASK-1807.
3. Preserve row syntax on function declarations and callable type positions that the current parser can safely own.
4. Parse row item families needed by Phase 177: operation path, resource, role, policy, channel, process/proc, fail, evidence, group, and row tail.
5. Keep unsupported raw predicate bodies and broader fact/proof declaration bodies out of executable row carriers.
6. Add focused parser tests for compact inline rows, expanded `where row`, open rows, and representative item families.

### Property Requirements

- Parsed row carriers are source metadata and type requirements only; they do not imply provider, admission, role, or host authority.
- Parser carriers must retain enough span information for duplicate-row and tail diagnostics in TASK-1811.
- Unsupported row forms fail closed or remain parsed only as non-lowered syntax carriers if TASK-1808 explicitly allows that.

## TDD Steps

### Step 1: Write failing parser tests

Add tests under `crates/ash-parser` for:

- `fn read(path: Path) -> {PosixFs::read} String { ... }`
- `fn read(path: Path) -> String where row { PosixFs::read } { ... }`
- `fn map<A, B, r: Row>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }`
- a mixed row with `resource`, `role`, `policy`, `fail`, `evidence`, and `| r`.

### Step 2: Verify RED

Run focused parser tests and confirm they fail due to missing row carriers/parser support.

### Step 3: Implement minimal parser carriers

Add surface AST structs/enums and parser productions scoped to the passing tests.

### Step 4: Verify GREEN

Run focused parser tests, then `cargo test -p ash-parser`.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-parser
  - git diff --check
checklist:
  - [ ] Parser tests cover inline rows.
  - [ ] Parser tests cover expanded `where row`.
  - [ ] Surface row carriers preserve spans.
  - [ ] Unsupported row forms do not silently lower.
```

## Dependencies for Next Task

This task feeds TASK-1810 and TASK-1811.
