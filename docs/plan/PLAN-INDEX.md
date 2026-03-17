# Ash Implementation Plan

## Overview

This document indexes all implementation tasks for Ash, organized by phase. Each task follows TDD methodology with property-based testing.

## Task Completion Criteria

Every task is considered **complete** only when:

1. ✅ **All tests pass** - Unit tests, integration tests, and property tests
2. ✅ **Property tests extensive** - Using proptest with meaningful invariants
3. ✅ **Code review** - Self-review for:
   - Opportunities to simplify
   - Code smell removal
   - Spec drift check (verify against SPEC documents)
4. ✅ **Rust tooling**:
   - `cargo fmt` passes
   - `cargo clippy` passes with no warnings
   - `cargo doc` generates clean documentation
5. ✅ **Documentation** updated:
   - Module-level docs
   - Function-level docs for public API
   - CHANGELOG.md entry

## Phase 1: Foundation (Weeks 1-2)

### Core Types and Data Structures

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-001](tasks/TASK-001-effect-lattice.md) | Effect lattice with property tests | SPEC-001 | 4 | ✅ Complete |
| [TASK-002](tasks/TASK-002-value-system.md) | Value enum with serialization | SPEC-001 | 4 | ✅ Complete |
| [TASK-003](tasks/TASK-003-workflow-ast.md) | Core Workflow AST types | SPEC-001 | 6 | ✅ Complete |
| [TASK-004](tasks/TASK-004-provenance.md) | Provenance and trace types | SPEC-001 | 4 | ✅ Complete |
| [TASK-005](tasks/TASK-005-patterns.md) | Pattern matching system | SPEC-001 | 6 | ✅ Complete |

### Testing Infrastructure

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-006](tasks/TASK-006-arbitrary-impls.md) | proptest Arbitrary implementations | - | 6 | ✅ Complete |
| [TASK-007](tasks/TASK-007-test-harness.md) | Shared testing utilities | - | 4 | ✅ Complete |

**Phase 1 Deliverable**: `ash-core` crate with complete IR

## Phase 2: Parser (Weeks 3-4)

### Lexer

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-008](tasks/TASK-008-tokens.md) | Token definitions | SPEC-002 | 3 |
| [TASK-009](tasks/TASK-009-lexer.md) | Lexer with error recovery | SPEC-002 | 6 |
| [TASK-010](tasks/TASK-010-lexer-tests.md) | Lexer property tests | SPEC-002 | 4 |

### Parser

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-011](tasks/TASK-011-surface-ast.md) | Surface AST types | SPEC-002 | 4 |
| [TASK-012](tasks/TASK-012-parser-core.md) | Parser combinators (winnow) | SPEC-002 | 8 |
| [TASK-013](tasks/TASK-013-parser-workflows.md) | Workflow parsing | SPEC-002 | 6 |
| [TASK-014](tasks/TASK-014-parser-expr.md) | Expression parsing | SPEC-002 | 6 |
| [TASK-015](tasks/TASK-015-error-recovery.md) | Parser error recovery | SPEC-002 | 6 |

### Lowering

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-016](tasks/TASK-016-lowering.md) | Surface → Core lowering | SPEC-001/002 | 8 |
| [TASK-017](tasks/TASK-017-desugar.md) | Desugaring transformations | SPEC-002 | 4 |

**Phase 2 Deliverable**: `ash-parser` crate, complete parsing pipeline

## Phase 3: Type System (Weeks 5-6)

### Type Inference

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-018](tasks/TASK-018-type-representation.md) | Type enum and unification | SPEC-003 | 4 |
| [TASK-019](tasks/TASK-019-type-constraints.md) | Type constraint generation | SPEC-003 | 6 |
| [TASK-020](tasks/TASK-020-unification.md) | Unification algorithm | SPEC-003 | 6 |
| [TASK-021](tasks/TASK-021-effect-inference.md) | Effect inference | SPEC-003 | 6 |

### Validation

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-022](tasks/TASK-022-name-resolution.md) | Name resolution pass | SPEC-003 | 6 |
| [TASK-023](tasks/TASK-023-obligation-check.md) | Obligation tracking | SPEC-003 | 6 |
| [TASK-024](tasks/TASK-024-proof-obligations.md) | Proof obligation generation | SPEC-003 | 6 |
| [TASK-024b](tasks/TASK-024b-smt-integration.md) | Z3 SMT integration for conflict detection | SPEC-003 | 8 |

### Error Reporting

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-025](tasks/TASK-025-type-errors.md) | Rich type error messages | SPEC-003 | 6 |

**Phase 3 Deliverable**: `ash-typeck` crate, complete type checking

## Phase 4: Interpreter (Weeks 7-8)

### Core Runtime

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-026](tasks/TASK-026-context.md) | Runtime context and state | SPEC-004 | 4 |
| [TASK-027](tasks/TASK-027-eval-expr.md) | Expression evaluator | SPEC-004 | 6 |
| [TASK-028](tasks/TASK-028-pattern-match.md) | Pattern matching engine | SPEC-004 | 6 |
| [TASK-029](tasks/TASK-029-guards.md) | Guard evaluation | SPEC-004 | 4 |

### Workflow Execution

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------|
| [TASK-030](tasks/TASK-030-interp-epistemic.md) | OBSERVE execution | SPEC-004 | 4 |
| [TASK-031](tasks/TASK-031-interp-deliberative.md) | ORIENT/PROPOSE execution | SPEC-004 | 4 |
| [TASK-032](tasks/TASK-032-interp-evaluative.md) | DECIDE/CHECK execution | SPEC-004 | 6 |
| [TASK-033](tasks/TASK-033-interp-operational.md) | ACT/OBLIG execution | SPEC-004 | 6 |
| [TASK-034](tasks/TASK-034-control-flow.md) | Control flow execution | SPEC-004 | 6 |

### Capability System

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-035](tasks/TASK-035-capability-trait.md) | Capability provider trait | SPEC-004 | 4 |
| [TASK-036](tasks/TASK-036-policy-runtime.md) | Runtime policy evaluation | SPEC-004 | 6 |
| [TASK-037](tasks/TASK-037-async-runtime.md) | Async runtime integration | SPEC-004 | 6 |

**Phase 4 Deliverable**: `ash-interp` crate, working interpreter

## Phase 5: Provenance (Week 9)

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-038](tasks/TASK-038-trace-recording.md) | Trace event recording | SPEC-001 | 4 |
| [TASK-039](tasks/TASK-039-lineage-tracking.md) | Lineage tracking | SPEC-001 | 4 |
| [TASK-040](tasks/TASK-040-audit-export.md) | Audit log export | SPEC-001 | 4 |
| [TASK-041](tasks/TASK-041-integrity.md) | Trace integrity (Merkle) | SPEC-001 | 6 |

**Phase 5 Deliverable**: `ash-provenance` crate, complete audit system

## Phase 6: CLI and Integration (Week 10)

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-053](tasks/TASK-053-cli-check.md) | `ash check` command | SPEC-005 | 6 |
| [TASK-054](tasks/TASK-054-cli-run.md) | `ash run` command | SPEC-005 | 8 |
| [TASK-055](tasks/TASK-055-cli-trace.md) | `ash trace` command | SPEC-005 | 6 |
| [TASK-056](tasks/TASK-056-cli-repl.md) | `ash repl` command | SPEC-005 | 8 |
| [TASK-057](tasks/TASK-057-cli-dot.md) | `ash dot` command | SPEC-005 | 4 |
| [TASK-058](tasks/TASK-058-cli-fmt.md) | `ash fmt` command | SPEC-005 | 4 |
| [TASK-059](tasks/TASK-059-cli-lsp.md) | `ash lsp` command | SPEC-005 | 12 |
| [TASK-060](tasks/TASK-060-integration-tests.md) | End-to-end integration tests | - | 8 |

**Phase 6 Deliverable**: `ash-cli` crate with check, run, trace, repl, dot commands

## Phase 7: Examples and Documentation (Week 11)

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-047](tasks/TASK-047-examples.md) | Example workflow library | - | 8 |
| [TASK-048](tasks/TASK-048-tutorial.md) | User tutorial | - | 8 |
| [TASK-049](tasks/TASK-049-api-docs.md) | API documentation | - | 6 |

## Phase 8: Optimization and Polish (Week 12)

| Task | Description | Spec | Est. Hours |
|------|-------------|------|------------|
| [TASK-050](tasks/TASK-050-benchmarks.md) | Criterion benchmarks | - | 6 |
| [TASK-051](tasks/TASK-051-optimizations.md) | Performance optimizations | - | 8 |
| [TASK-052](tasks/TASK-052-fuzzing.md) | Fuzzing setup | - | 6 |

## Phase 9: Advanced Policy Features (Week 13+)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-061](tasks/TASK-061-policy-definitions.md) | Policy definition syntax | SPEC-006 | 12 | 🔴 Not Started |
| [TASK-062](tasks/TASK-062-policy-combinators.md) | Policy combinators | SPEC-007 | 16 | 🔴 Not Started |
| [TASK-063](tasks/TASK-063-dynamic-policies.md) | Dynamic policy registration | SPEC-008 | 40 | ⏸️ Deferred |

**Phase 9 Deliverable**: User-defined policies with compile-time conflict detection

## Total Effort Estimate

- **Tasks**: 59 (56 complete, 3 planned)
- **Estimated Hours**: ~424 hours (including Phase 9)
- **Calendar Time**: 12 weeks (single developer)
- **Team of 3**: ~4 weeks with parallel work

## Dependency Graph

```
Phase 1 (Core)
    │
    ├──→ Phase 2 (Parser)
    │       │
    │       └──→ Phase 3 (Typeck)
    │               │
    │               └──→ Phase 4 (Interp)
    │                       │
    │                       ├──→ Phase 5 (Provenance)
    │                       │       │
    │                       │       └──→ Phase 6 (CLI)
    │                       │               │
    │                       │               └──→ Phase 7 (Docs)
    │                       │
    └──→ Phase 5 can start after Phase 1
```

## Running the Plan

1. Pick next uncompleted task from current phase
2. Create feature branch: `git checkout -b task/XXX-short-name`
3. Follow TDD: Write tests → Make them pass → Refactor
4. Complete task checklist
5. Self-review and tooling checks
6. Commit: `git commit -m "TASK-XXX: Description"`
7. Move to next task

## Progress Tracking

Update this section as tasks complete:

| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| 1 | 7 | 7 | ✅ Complete |
| 2 | 10 | 10 | ✅ Complete |
| 3 | 9 | 8 | ✅ Complete |
| 4 | 12 | 12 | ✅ Complete |
| 5 | 4 | 4 | ✅ Complete |
| 6 | 8 | 8 | ✅ Complete |
| 7 | 3 | 3 | ✅ Complete |
| 8 | 3 | 3 | ✅ Complete |
| 9 | 3 | 0 | 🟡 Planned |

**Overall Progress**: 56/59 tasks (95%)
