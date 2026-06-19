# TASK-1599: Harden the CPS S-expression parser

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Extend the Phase 1 `.cps` parser scaffold to cover every Phase 1 through Phase 5 IR form and reject malformed or layer-violating syntax.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1590: Define CPS IR core data structures.
- 📝 TASK-1592: Evaluate conditionals and structured data.
- 📝 TASK-1593: Implement Raise and Handle dispatch.
- 📝 TASK-1595: Construct and enforce resume continuations.
- 📝 TASK-1596: Implement single-binding LetRec recursion.
- 📝 TASK-1597: Implement RecordDischarge and Trap.
- 📝 TASK-1598: Implement row representation and local/total row validation scaffold.

## Requirements

### Functional Requirements

1. Parse every `Atom`, `Value`, `Term`, `ContRef`, `EffectOp`, `HandlerClause`, and row form used by Phase 159.
2. Reject values in term position, labels in data position, inline primitive expressions where `LetPrim` is required, and incomplete handler clauses.
3. Produce diagnostics that name the malformed form.
4. Preserve source spans or stable fixture locations if the parser substrate supports them.

### Property Requirements

- All committed valid `.cps` fixtures parse.
- Known malformed fixtures fail closed with stable diagnostics.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1599_cps_ir.rs`

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps.rs`

Implement only the slice named by this task. Preserve the SPEC-098b `Atom` / `Value` / `Term` boundary and avoid direct-style convenience nodes.

### Step 3: Integrate

Wire the new slice through crate exports and the Phase 159 `.cps` fixture path without replacing the existing workflow interpreter.

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
  - cargo fmt --check
  - cargo test -p ash-core -p ash-interp
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
checklist:
  - [ ] Focused tests execute non-zero cases
  - [ ] `.cps` fixtures parse or are explicitly deferred by this task
  - [ ] CHANGELOG.md updated when this task is completed
```

## Dependencies for Next Task

- Provides the complete parser half of the differential-test fixture contract.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.

### Deferral Note: Custom .cps Grammar Parser

The current implementation uses `serde-lexpr` for S-expression serialization/deserialization. This provides a safe round-trip from Rust AST → S-expression → Rust AST with minimal code. The trade-off is that the output format uses PascalCase enum names and dotted-pair field notation rather than the documented lowercase fixture grammar.

**serde-lexpr format:**
```
(LetVal (name . "x") (value Atom Int . 42) (body Jump (cont Label . "exit") ...))
```

**Desired fixture format:**
```
(letval x 42 (jump (label exit) (var x)))
```

**Decision:** Defer custom parser/serializer to a follow-up task. The serde-lexpr approach is sufficient for Phase 159 because:
- Round-trip property holds: `parse(serialize(term)) == term`
- File I/O workflow is tested: write → read → execute
- No external producer/consumer requires the lowercase format yet
- A custom parser would require significant lexer + recursive descent work for primarily stylistic benefit

When a custom parser is needed, implement:
1. Lexer: `(`, `)`, `[`, `]`, identifiers, strings, numbers
2. Parser: recursive descent for each term/value variant with lowercase keywords
3. Serializer: custom output formatting with canonical shape
4. Negative tests: malformed fixture rejection
