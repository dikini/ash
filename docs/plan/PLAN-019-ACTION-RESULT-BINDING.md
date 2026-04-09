# PLAN-019: Action Result Binding and Continuation

## Status: 📝 Planned

## Overview

Implement DESIGN-019 to extend `Workflow::Act` with result binding and continuation so capability
actions can produce values that flow back into the workflow. This plan adds `result_name` and
`continuation` to the core Act node, extends the surface language with `then` and `as` continuation
forms, adds `let name = <cap-call>` sugar, and updates the interpreter to execute continuations.

## Design Reference

- [DESIGN-019: Action Result Binding and Continuation](../design/DESIGN-019-ACTION-RESULT-BINDING.md)

## Goals

1. Extend core `Workflow::Act` with `result_name: Option<Name>` and `continuation: Box<Workflow>`
2. Migrate all existing bare `Act` nodes to `continuation: Done, result_name: None`
3. Add surface `then` and `as` continuation forms to the parser
4. Add `let <name> = <cap-call>` sugar recognized at parse time in `let_stmt()`
5. Update interpreter execution to bind result and execute continuation
6. Update specs (SPEC-001, SPEC-002, SPEC-004, SPEC-025) for the new contract
7. Full test suite green

## Scope

**In Scope**:
- Core `Workflow::Act` shape change
- Surface AST extension for Act (`result_name`, `continuation`)
- Parser support for `then` and `as` continuation keywords in act context (extending existing `as` pattern from observe/orient/propose)
- Parser-level recognition of `let <name> = <operational-call>` sugar in `let_stmt()` via `action_ref()` try-before-`expr()` (not deferred to lowering)
- Lowering of new surface forms to extended core Act
- Interpreter execution of Act continuation with result binding
- Migration of all existing Act construction sites
- Spec updates
- Documentation and CHANGELOG

**Out of Scope**:
- Typed return values or effect-level constraints on results
- Changes to `observe`, `set`, `send`, or other workflow variants
- Streaming/async action results
- Provider trait changes

## Phases

### Phase 1: Core AST Change and Migration (ash-core, ash-parser, ash-interp)

**Goal**: Change `Workflow::Act` to include `result_name` and `continuation`, migrate all
construction sites across the workspace, and verify the workspace compiles with existing
bare-act semantics preserved.

**Tasks**:
- [TASK-486](tasks/TASK-486-core-act-continuation-shape.md): Update core `Workflow::Act` with `result_name` and `continuation`, migrate all workspace construction sites
- [TASK-487](tasks/TASK-487-surface-act-continuation.md): Extend surface AST `Act` with `result_name` and `continuation`, update lowering
- [TASK-488](tasks/TASK-488-parser-act-then-as.md): Add parser support for `act ... then`, `act ... as`, and `let <name> = <cap-call>` sugar

**Deliverable**: Core and surface ASTs support Act continuation. All existing bare-act forms compile with identical semantics. New surface forms parse and lower correctly.

**Estimated Effort**: 6 hours

---

### Phase 2: Interpreter Execution and Binding (ash-interp)

**Goal**: Update the interpreter to execute the Act continuation, bind `result_name` into the
execution context, and return the continuation's result.

**Tasks**:
- [TASK-489](tasks/TASK-489-interpreter-act-continuation.md): Update interpreter ACT execution to bind result and execute continuation
- [TASK-490](tasks/TASK-490-act-continuation-integration-tests.md): Write integration tests for `act ... then`, `act ... as`, and `let = cap-call` forms

**Deliverable**: Interpreter correctly executes all three Act continuation forms with proper
result binding and continuation semantics.

**Estimated Effort**: 4 hours

---

### Phase 3: Spec Updates and Documentation

**Goal**: Update specs, docs, examples, and CHANGELOG to reflect the new Act continuation contract.

**Tasks**:
- [TASK-491](tasks/TASK-491-spec-act-continuation-updates.md): Update SPEC-001, SPEC-002, SPEC-004, SPEC-025 for Act continuation semantics
- [TASK-492](tasks/TASK-492-act-continuation-docs-and-verification.md): Update docs, examples, CHANGELOG, and run final verification

**Deliverable**: Specs and documentation aligned with the implemented Act continuation feature.

**Estimated Effort**: 3 hours

---

## Critical Path

```
TASK-486 (Core Act shape)
    ↓
TASK-487 (Surface AST + lowering)
    ↓
TASK-488 (Parser then/as/sugar)
    ↓
TASK-489 (Interpreter execution)
    ↓
TASK-490 (Integration tests)
    ↓
TASK-491 (Spec updates)
    ↓
TASK-492 (Docs + verification)
```

Parallel paths:
- TASK-487 and TASK-488 are sequential (surface AST must exist before parser changes)
- TASK-491 can start once TASK-489 lands (specs can be updated in parallel with integration tests)

---

## Dependencies

**External Dependencies**:
- None

**Internal Dependencies**:
- Phase 2 depends on Phase 1
- Phase 3 depends on Phase 2

---

## Risks

### Risk 1: Migration of Act Construction Sites Misses Edge Cases

**Probability**: Medium
**Impact**: High
**Mitigation**: TASK-486 explicitly audits all `Workflow::Act` construction sites via `rg` before
and after the change. Property tests verify semantic preservation.

### Risk 2: Parser Ambiguity Between `let name = expr` and `let name = <cap-call>`

**Probability**: Low
**Impact**: Medium
**Mitigation**: The surface AST has `OperationalTarget` / `ActionRef` to distinguish operational calls
from general expressions. The parser handles this in `let_stmt()` by trying `action_ref()` first
via lookahead before falling back to `expr()`. This is documented explicitly in TASK-488 and
DESIGN-019 Decision 4. The recognition happens at parse time, not at lowering time.

### Risk 3: Continuation Context Leakage

**Probability**: Low
**Impact**: Medium
**Mitigation**: `result_name` follows the same scoping rules as `Let`. The existing lexical-scope
contract (established in Phase 68) applies.

---

## Success Criteria

1. **Structural parity**: `Act` has `result_name` and `continuation`, structurally equal to `Let`.
2. **Surface forms**: All three forms (`then`, `as`, `let = cap-call`) parse, lower, and execute.
3. **Backwards compatible**: Existing bare `act` forms compile and run identically.
4. **Specs aligned**: SPEC-001, SPEC-002, SPEC-004, SPEC-025 updated.
5. **Full gate green**: `cargo test -p ash-core -p ash-parser -p ash-interp -p ash-cli`, `cargo clippy`, `cargo fmt --check`, `cargo doc` all pass. Known pre-existing ash-engine failures are excluded.

---

## Timeline

| Phase | Duration | Start Date | End Date |
|-------|----------|------------|----------|
| Phase 1 | 1 day | TBD | TBD |
| Phase 2 | 0.5 day | TBD | TBD |
| Phase 3 | 0.5 day | TBD | TBD |
| **Total** | **2 days** | TBD | TBD |

---

*Document Version: 1.0*
*Status: Planned*
*Author: hermes*
*Date: 2026-04-09*
