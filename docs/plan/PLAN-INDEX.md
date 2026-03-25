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

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-008](tasks/TASK-008-tokens.md) | Token definitions | SPEC-002 | 3 | ✅ Complete |
| [TASK-009](tasks/TASK-009-lexer.md) | Lexer with error recovery | SPEC-002 | 6 | ✅ Complete |
| [TASK-010](tasks/TASK-010-lexer-tests.md) | Lexer property tests | SPEC-002 | 4 | ✅ Complete |

### Parser

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-011](tasks/TASK-011-surface-ast.md) | Surface AST types | SPEC-002 | 4 | ✅ Complete |
| [TASK-012](tasks/TASK-012-parser-core.md) | Parser combinators (winnow) | SPEC-002 | 8 | ✅ Complete |
| [TASK-013](tasks/TASK-013-parser-workflows.md) | Workflow parsing | SPEC-002 | 6 | ✅ Complete |
| [TASK-014](tasks/TASK-014-parser-expr.md) | Expression parsing | SPEC-002 | 6 | ✅ Complete |
| [TASK-015](tasks/TASK-015-error-recovery.md) | Parser error recovery | SPEC-002 | 6 | ✅ Complete |

### Lowering

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-016](tasks/TASK-016-lowering.md) | Surface → Core lowering | SPEC-001/002 | 8 | ✅ Complete |
| [TASK-017](tasks/TASK-017-desugar.md) | Desugaring transformations | SPEC-002 | 4 | ✅ Complete |

**Phase 2 Deliverable**: `ash-parser` crate, complete parsing pipeline

## Phase 3: Type System (Weeks 5-6)

### Type Inference

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-018](tasks/TASK-018-type-representation.md) | Type enum and unification | SPEC-003 | 4 | ✅ Complete |
| [TASK-019](tasks/TASK-019-type-constraints.md) | Type constraint generation | SPEC-003 | 6 | ✅ Complete |
| [TASK-020](tasks/TASK-020-unification.md) | Unification algorithm | SPEC-003 | 6 | ✅ Complete |
| [TASK-021](tasks/TASK-021-effect-inference.md) | Effect inference | SPEC-003 | 6 | ✅ Complete |

### Validation

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-022](tasks/TASK-022-name-resolution.md) | Name resolution pass | SPEC-003 | 6 | ✅ Complete |
| [TASK-023](tasks/TASK-023-obligation-check.md) | Obligation tracking | SPEC-003 | 6 | ✅ Complete |
| [TASK-024](tasks/TASK-024-proof-obligations.md) | Proof obligation generation | SPEC-003 | 6 | ✅ Complete |
| [TASK-024b](tasks/TASK-024b-smt-integration.md) | Z3 SMT integration for conflict detection | SPEC-003 | 8 | ✅ Complete |

### Error Reporting

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-025](tasks/TASK-025-type-errors.md) | Rich type error messages | SPEC-003 | 6 | ✅ Complete |

**Phase 3 Deliverable**: `ash-typeck` crate, complete type checking

## Phase 4: Interpreter (Weeks 7-8)

### Core Runtime

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-026](tasks/TASK-026-context.md) | Runtime context and state | SPEC-004 | 4 | ✅ Complete |
| [TASK-027](tasks/TASK-027-eval-expr.md) | Expression evaluator | SPEC-004 | 6 | ✅ Complete |
| [TASK-028](tasks/TASK-028-pattern-match.md) | Pattern matching engine | SPEC-004 | 6 | ✅ Complete |
| [TASK-029](tasks/TASK-029-guards.md) | Guard evaluation | SPEC-004 | 4 | ✅ Complete |

### Workflow Execution

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-030](tasks/TASK-030-interp-epistemic.md) | OBSERVE execution | SPEC-004 | 4 | ✅ Complete |
| [TASK-031](tasks/TASK-031-interp-deliberative.md) | ORIENT/PROPOSE execution | SPEC-004 | 4 | ✅ Complete |
| [TASK-032](tasks/TASK-032-interp-evaluative.md) | DECIDE/CHECK execution | SPEC-004 | 6 | ✅ Complete |
| [TASK-033](tasks/TASK-033-interp-operational.md) | ACT/OBLIG execution | SPEC-004 | 6 | ✅ Complete |
| [TASK-034](tasks/TASK-034-control-flow.md) | Control flow execution | SPEC-004 | 6 | ✅ Complete |

### Capability System

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-035](tasks/TASK-035-capability-trait.md) | Capability provider trait | SPEC-004 | 4 | ✅ Complete |
| [TASK-036](tasks/TASK-036-policy-runtime.md) | Runtime policy evaluation | SPEC-004 | 6 | ✅ Complete |
| [TASK-037](tasks/TASK-037-async-runtime.md) | Async runtime integration | SPEC-004 | 6 | ✅ Complete |

**Phase 4 Deliverable**: `ash-interp` crate, working interpreter

## Phase 5: Provenance (Week 9)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-038](tasks/TASK-038-trace-recording.md) | Trace event recording | SPEC-001 | 4 | ✅ Complete |
| [TASK-039](tasks/TASK-039-lineage-tracking.md) | Lineage tracking | SPEC-001 | 4 | ✅ Complete |
| [TASK-040](tasks/TASK-040-audit-export.md) | Audit log export | SPEC-001 | 4 | ✅ Complete |
| [TASK-041](tasks/TASK-041-integrity.md) | Trace integrity (Merkle) | SPEC-001 | 6 | ✅ Complete |

**Phase 5 Deliverable**: `ash-provenance` crate, complete audit system

## Phase 6: CLI and Integration (Week 10)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-053](tasks/TASK-053-cli-check.md) | `ash check` command | SPEC-005 | 6 | ✅ Complete |
| [TASK-054](tasks/TASK-054-cli-run.md) | `ash run` command | SPEC-005 | 8 | ✅ Complete |
| [TASK-055](tasks/TASK-055-cli-trace.md) | `ash trace` command | SPEC-005 | 6 | ✅ Complete |
| [TASK-056](tasks/TASK-056-cli-repl.md) | `ash repl` command | SPEC-005 | 8 | ✅ Complete |
| [TASK-057](tasks/TASK-057-cli-dot.md) | `ash dot` command | SPEC-005 | 4 | ✅ Complete |
| [TASK-058](tasks/TASK-058-cli-fmt.md) | `ash fmt` command | SPEC-005 | 4 | ✅ Complete |
| [TASK-059](tasks/TASK-059-cli-lsp.md) | `ash lsp` command | SPEC-005 | 12 | ✅ Complete |
| [TASK-060](tasks/TASK-060-integration-tests.md) | End-to-end integration tests | - | 8 | ✅ Complete |

**Phase 6 Deliverable**: `ash-cli` crate with check, run, trace, repl, dot commands

## Phase 7: Examples and Documentation (Week 11)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-047](tasks/TASK-047-examples.md) | Example workflow library | - | 8 | ✅ Complete |
| [TASK-048](tasks/TASK-048-tutorial.md) | User tutorial | - | 8 | ✅ Complete |
| [TASK-049](tasks/TASK-049-api-docs.md) | API documentation | - | 6 | ✅ Complete |

## Phase 8: Optimization and Polish (Week 12)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-050](tasks/TASK-050-benchmarks.md) | Criterion benchmarks | - | 6 | ✅ Complete |
| [TASK-051](tasks/TASK-051-optimizations.md) | Performance optimizations | - | 8 | ✅ Complete |
| [TASK-052](tasks/TASK-052-fuzzing.md) | Fuzzing setup | - | 6 | ✅ Complete |

## Phase 9: Advanced Policy Features (Week 13+)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-061](tasks/TASK-061-policy-definitions.md) | Policy definition syntax | SPEC-006 | 12 | ✅ Complete |
| [TASK-062](tasks/TASK-062-policy-combinators.md) | Policy combinators | SPEC-007 | 16 | ✅ Complete |
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
| 3 | 9 | 9 | ✅ Complete |
| 4 | 12 | 12 | ✅ Complete |
| 5 | 4 | 4 | ✅ Complete |
| 6 | 8 | 8 | ✅ Complete |
| 7 | 3 | 3 | ✅ Complete |
| 8 | 3 | 3 | ✅ Complete |
| 9 | 3 | 3 | ✅ Complete |
| 10 | 11 | 11 | ✅ Complete |
| 11 | 6 | 6 | ✅ Complete |
| 12 | 7 | 7 | ✅ Complete |
| 13 | 8 | 8 | ✅ Complete |
| 14 | 5 | 5 | ✅ Complete |
| 14.5 | 7 | 7 | ✅ Complete |
| 15 | 6 | 6 | ✅ Complete |
| 16 | 6 | 6 | ✅ Complete |
| 17 | 12 | 12 | ✅ Complete |
| 18 | 7 | 3 | 🟡 In Progress |
| 19 | 7 | 7 | ✅ Complete |
| 20 | 5 | 5 | ✅ Complete |
| 21 | 3 | 0 | 📝 Planned |
| 22 | 2 | 0 | 📝 Planned |
| 23 | 4 | 4 | ✅ Complete |
| 24 | 2 | 0 | 📝 Planned |
| 25 | 1 | 0 | 📝 Planned |
| 26 | 4 | 4 | ✅ Complete |
| 27 | 3 | 3 | ✅ Complete |
| 28 | 2 | 0 | 📝 Planned |
| 29 | 2 | 0 | 📝 Planned |
| 30 | 2 | 0 | 📝 Planned |
| 31 | 1 | 1 | ✅ Complete |
| 34 | 3 | 3 | ✅ Complete |
| 35 | 5 | 5 | ✅ Complete |
| 36 | 4 | 4 | ✅ Complete |

## Phase 10: Module System (Weeks 14-16)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-064](tasks/TASK-064-module-ast.md) | Module AST types (ModuleDecl) | SPEC-009 | 4 | 🟢 Complete |
| [TASK-065](tasks/TASK-065-visibility-ast.md) | Visibility AST types (pub, pub(crate)) | SPEC-009 | 4 | 🟢 Complete |
| [TASK-066](tasks/TASK-066-parse-visibility.md) | Parse visibility modifiers | SPEC-009 | 4 | 🟢 Complete |
| [TASK-067](tasks/TASK-067-parse-mod.md) | Parse module declarations | SPEC-009 | 6 | 🟢 Complete |
| [TASK-068](tasks/TASK-068-module-graph.md) | Module graph data structure | SPEC-009 | 4 | 🟢 Complete |
| [TASK-069](tasks/TASK-069-module-resolver.md) | Module resolution algorithm | SPEC-009 | 8 | 🟢 Complete |
| [TASK-070](tasks/TASK-070-visibility-check.md) | Visibility checking in typeck | SPEC-009 | 6 | 🟢 Complete |
| [TASK-084](tasks/TASK-084-use-ast.md) | Use statement AST types | SPEC-012 | 3 | 🟢 Complete |
| [TASK-085](tasks/TASK-085-parse-use.md) | Parse use statements | SPEC-012 | 4 | 🟢 Complete |
| [TASK-086](tasks/TASK-086-import-resolution.md) | Import resolution algorithm | SPEC-012 | 6 | 🟢 Complete |
| [TASK-087](tasks/TASK-087-name-binding.md) | Name binding with imports | SPEC-012 | 5 | 🟢 Complete |

**Phase 10 Deliverable**: Rust-style module system with `mod`, `pub`, `use`, and file-based resolution

## Phase 11: Embedding API (Weeks 16-18)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-071](tasks/TASK-071-engine-crate.md) | Create ash-engine crate structure | SPEC-010 | 3 | 🟢 Complete |
| [TASK-072](tasks/TASK-072-engine-parse.md) | Implement Engine::parse | SPEC-010 | 2 | 🟢 Complete |
| [TASK-073](tasks/TASK-073-engine-check.md) | Implement Engine::check | SPEC-010 | 2 | 🟢 Complete |
| [TASK-074](tasks/TASK-074-engine-execute.md) | Implement Engine::execute | SPEC-010 | 3 | 🟢 Complete |
| [TASK-075](tasks/TASK-075-engine-capabilities.md) | Standard capability providers | SPEC-010 | 6 | 🟢 Complete |
| [TASK-076](tasks/TASK-076-cli-engine.md) | Update CLI to use ash-engine | SPEC-010 | 4 | 🟢 Complete |

**Phase 11 Deliverable**: Unified `Engine` type with builder API for embedding

## Phase 12: REPL (Weeks 18-19)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-077](tasks/TASK-077-repl-crate.md) | Create ash-repl crate | SPEC-011 | 3 | ✅ Complete |
| [TASK-078](tasks/TASK-078-repl-eval.md) | Expression evaluation in REPL | SPEC-011 | 4 | ✅ Complete |
| [TASK-079](tasks/TASK-079-repl-multiline.md) | Multi-line input detection | SPEC-011 | 4 | ✅ Complete |
| [TASK-080](tasks/TASK-080-repl-commands.md) | REPL commands (:help, :type, :quit) | SPEC-011 | 3 | ✅ Complete |
| [TASK-081](tasks/TASK-081-repl-completion.md) | Tab completion | SPEC-011 | 4 | ✅ Complete |
| [TASK-082](tasks/TASK-082-repl-history.md) | Persistent history | SPEC-011 | 2 | ✅ Complete |
| [TASK-083](tasks/TASK-083-repl-errors.md) | Error display improvements | SPEC-011 | 3 | ✅ Complete |

**Phase 12 Deliverable**: Interactive REPL with readline features

## Progress Tracking

| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| 1 | 7 | 7 | ✅ Complete |
| 2 | 10 | 10 | ✅ Complete |
| 3 | 9 | 9 | ✅ Complete |
| 4 | 12 | 12 | ✅ Complete |
| 5 | 4 | 4 | ✅ Complete |
| 6 | 8 | 8 | ✅ Complete |
| 7 | 3 | 3 | ✅ Complete |
| 8 | 3 | 3 | ✅ Complete |
| 9 | 3 | 3 | ✅ Complete |
| 10 | 11 | 11 | ✅ Complete |
| 11 | 6 | 6 | ✅ Complete |
| 12 | 7 | 7 | ✅ Complete |
| 13 | 8 | 8 | ✅ Complete |
| 14 | 5 | 5 | ✅ Complete |
| 14.5 | 7 | 7 | ✅ Complete |
| 15 | 6 | 6 | ✅ Complete |
| 16 | 6 | 6 | ✅ Complete |

## Phase 13: Streams and Behaviours (Weeks 20-22)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-088](tasks/TASK-088-stream-ast.md) | Stream AST types and mailbox structure | SPEC-013 | 4 | ✅ Complete |
| [TASK-089](tasks/TASK-089-stream-provider.md) | Stream provider trait and registry | SPEC-013 | 4 | ✅ Complete |
| [TASK-090](tasks/TASK-090-parse-receive.md) | Parse receive construct | SPEC-013 | 6 | ✅ Complete |
| [TASK-091](tasks/TASK-091-mailbox-impl.md) | Mailbox implementation with limits | SPEC-013 | 6 | ✅ Complete |
| [TASK-092](tasks/TASK-092-stream-execution.md) | Stream execution with pattern matching | SPEC-013 | 8 | ✅ Complete |
| [TASK-093](tasks/TASK-093-behaviour-provider.md) | Behaviour provider trait | SPEC-014 | 3 | ✅ Complete |
| [TASK-094](tasks/TASK-094-parse-observe.md) | Parse observe with constraints | SPEC-014 | 3 | ✅ Complete |
| [TASK-095](tasks/TASK-095-observe-execution.md) | Observe execution and sampling | SPEC-014 | 4 | ✅ Complete |

**Phase 13 Deliverable**: Stream processing with receive and behaviour sampling with observe

## Phase 14: Typed Providers (Week 23)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-096](tasks/TASK-096-typed-provider-wrapper.md) | Typed provider wrapper structs | SPEC-015 | 3 | ✅ Complete |
| [TASK-097](tasks/TASK-097-schema-validation.md) | Schema validation logic | SPEC-015 | 4 | ✅ Complete |
| [TASK-098](tasks/TASK-098-typed-registry.md) | Typed registry integration | SPEC-015 | 3 | ✅ Complete |
| [TASK-099](tasks/TASK-099-runtime-validation.md) | Runtime validation in providers | SPEC-015 | 3 | ✅ Complete |
| [TASK-100](tasks/TASK-100-type-error-reporting.md) | Type error reporting | SPEC-015 | 2 | ✅ Complete |

**Phase 14 Deliverable**: Runtime type safety for Rust/Ash provider boundary

## Phase 14.5: Output Capabilities (Week 23.5)

Output capabilities for writing/sending data (complement to input capabilities in Phase 13).

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-101](tasks/TASK-101-settable-provider.md) | Settable behaviour provider trait | SPEC-016 | 3 | ✅ Complete |
| [TASK-102](tasks/TASK-102-sendable-provider.md) | Sendable stream provider trait | SPEC-016 | 3 | ✅ Complete |
| [TASK-103](tasks/TASK-103-parse-set.md) | Parse set statement | SPEC-016 | 3 | ✅ Complete |
| [TASK-104](tasks/TASK-104-parse-send.md) | Parse send statement | SPEC-016 | 3 | ✅ Complete |
| [TASK-105](tasks/TASK-105-set-execution.md) | Set execution | SPEC-016 | 4 | ✅ Complete |
| [TASK-106](tasks/TASK-106-send-execution.md) | Send execution | SPEC-016 | 4 | ✅ Complete |
| [TASK-107](tasks/TASK-107-bidirectional-wrapper.md) | Bidirectional provider wrappers | SPEC-016 | 3 | ✅ Complete |

**Phase 14.5 Deliverable**: Complete output capability support (set/send) for behaviours and streams

## Phase 15: Capability Integration (Week 24)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-108](tasks/TASK-108-effect-tracking.md) | Effect tracking for all capabilities | SPEC-017 | 4 | ✅ Complete |
| [TASK-109](tasks/TASK-109-obligation-checking.md) | Obligation checking with capabilities | SPEC-017 | 4 | ✅ Complete |
| [TASK-110](tasks/TASK-110-policy-evaluation.md) | Policy evaluation for input/output | SPEC-017 | 6 | ✅ Complete |
| [TASK-111](tasks/TASK-111-provenance-tracking.md) | Provenance tracking for all capabilities | SPEC-017 | 6 | ✅ Complete |
| [TASK-112](tasks/TASK-112-capability-verification.md) | Capability declaration verification | SPEC-017 | 4 | ✅ Complete |
| [TASK-113](tasks/TASK-113-read-write-types.md) | Read/write type checking | SPEC-017 | 4 | ✅ Complete |

**Phase 15 Deliverable**: Full integration of capabilities with obligations, policies, provenance

## Phase 16: Runtime Verification (Week 25)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-114](tasks/TASK-114-capability-verifier.md) | Capability availability verifier | SPEC-018 | 4 | ✅ Complete |
| [TASK-115](tasks/TASK-115-obligation-checker.md) | Obligation satisfaction checker | SPEC-018 | 4 | ✅ Complete |
| [TASK-116](tasks/TASK-116-effect-checker.md) | Effect compatibility checker | SPEC-018 | 3 | ✅ Complete |
| [TASK-117](tasks/TASK-117-static-policy-validator.md) | Static policy validator | SPEC-018 | 4 | ✅ Complete |
| [TASK-118](tasks/TASK-118-operation-verifier.md) | Per-operation runtime verifier | SPEC-018 | 5 | ✅ Complete |
| [TASK-119](tasks/TASK-119-verification-aggregator.md) | Verification result aggregation | SPEC-018 | 3 | ✅ Complete |

**Phase 16 Deliverable**: Runtime verification of workflow-context compatibility

**Overall Progress**: 209/210 tasks complete (1 deferred)
**Remaining Tasks**: 1 deferred task (`TASK-063` dynamic policy registration)

## Phase 17: Lean Reference Implementation (Weeks 26-28)

Reference interpreter implementation in Lean 4 for specification verification.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-137](tasks/TASK-137-lean-setup.md) | Lean 4 project setup with lake | SPEC-021 | 4 | ✅ Complete |
| [TASK-138](tasks/TASK-138-lean-ast-types.md) | Core AST types (Value, Pattern, Expr) | SPEC-021 | 8 | ✅ Complete |
| [TASK-139](tasks/TASK-139-lean-environment.md) | Environment and Bindings types | SPEC-021 | 6 | ✅ Complete |
| [TASK-140](tasks/TASK-140-lean-expression-eval.md) | Expression evaluation | SPEC-021 | 12 | ✅ Complete |
| [TASK-141](tasks/TASK-141-lean-pattern-match.md) | Pattern matching engine | SPEC-021 | 12 | ✅ Complete |
| [TASK-142](tasks/TASK-142-lean-match-expr.md) | Match expression evaluation | SPEC-021 | 8 | ✅ Complete |
| [TASK-143](tasks/TASK-143-lean-if-let.md) | If-let expression evaluation | SPEC-021 | 6 | ✅ Complete |
| [TASK-144](tasks/TASK-144-lean-json-serialization.md) | JSON serialization for diff testing | SPEC-021 | 8 | ✅ Complete |
| [TASK-145](tasks/TASK-145-lean-differential-testing.md) | Differential testing framework | SPEC-021 | 10 | ✅ Complete |
| [TASK-146](tasks/TASK-146-lean-property-tests.md) | Property-based tests with Plausible | SPEC-021 | 8 | ✅ Complete |
| [TASK-147](tasks/TASK-147-lean-ci-integration.md) | CI integration for Lean | SPEC-021 | 4 | ✅ Complete |
| [TASK-148](tasks/TASK-148-lean-documentation.md) | API documentation and examples | SPEC-021 | 6 | ✅ Complete |

**Phase 17 Deliverable**: Complete Lean 4 reference interpreter with testing

## Phase 18: ADT Implementation (Weeks 29-30)

Algebraic Data Types support in the Rust implementation.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-130](tasks/TASK-130-exhaustiveness-checking.md) | Exhaustiveness checking for patterns | SPEC-020 | 16 | ✅ Complete |
| [TASK-131](tasks/TASK-131-constructor-evaluation.md) | Constructor evaluation | SPEC-020 | 8 | ✅ Complete |
| [TASK-132](tasks/TASK-132-pattern-matching-engine.md) | Pattern matching engine | SPEC-020 | 12 | ✅ Complete |
| [TASK-133](tasks/TASK-133-match-evaluation.md) | Match expression evaluation | SPEC-020 | 12 | ✅ Complete |
| [TASK-134](tasks/TASK-134-spawn-option-control-link.md) | Spawn with Option<ControlLink> | SPEC-020 | 8 | ✅ Complete |
| [TASK-135](tasks/TASK-135-control-link-transfer.md) | Control link affine transfer | SPEC-020 | 8 | ✅ Complete |
| [TASK-136](tasks/TASK-136-option-result-library.md) | Option/Result standard library | SPEC-020 | 8 | ✅ Complete |

**Phase 18 Deliverable**: ADT support with pattern matching in Rust implementation

## Phase 19: Formal Proofs (Weeks 31-36)

Formal proofs of key semantic properties in the Lean reference interpreter.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-149](tasks/TASK-149-pattern-determinism-proof.md) | Pattern match determinism proof | SPEC-004 | 12 | ✅ Complete |
| [TASK-150](tasks/TASK-150-pattern-totality-proof.md) | Pattern match totality proof | SPEC-004 | 16 | ✅ Complete |
| [TASK-151](tasks/TASK-151-constructor-purity-proof.md) | Constructor purity proof | SPEC-004 | 8 | ✅ Complete |
| [TASK-152](tasks/TASK-152-evaluation-determinism-proof.md) | Evaluation determinism proof | SPEC-004 | 12 | ✅ Complete |
| [TASK-153](tasks/TASK-153-progress-theorem.md) | Progress theorem | Type Safety | 24 | ✅ Complete |
| [TASK-154](tasks/TASK-154-preservation-theorem.md) | Preservation theorem | Type Safety | 32 | ✅ Complete |
| [TASK-155](tasks/TASK-155-type-safety-corollary.md) | Type safety corollary | Type Safety | 8 | ✅ Complete |

**Phase 19 Deliverable**: Mathematical proofs of pattern determinism, evaluation determinism, and type safety

**Note**: Phase 19 proofs use `sorry` for incomplete proofs due to Lean 4 partial function limitations. The theorems are correctly stated and the determinism proofs are complete. Full proofs require making `eval` total (fuel-based approach) - see long-term tasks.

## Phase 20: Spec Convergence (Week 37+)

Canonicalize spec contracts before downstream Rust alignment work.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-156](tasks/TASK-156-canonicalize-workflow-form-contracts.md) | Canonicalize workflow form contracts | SPEC-001/002/003/004/017/018 | 6 | ✅ Complete |
| [TASK-157](tasks/TASK-157-canonicalize-policy-contracts.md) | Canonicalize policy contracts | SPEC-003/004/006/007/008/017/018 | 6 | ✅ Complete |
| [TASK-158](tasks/TASK-158-canonicalize-streams-runtime-verification-contracts.md) | Canonicalize streams/runtime verification contracts | SPEC-004/013/014/017/018 | 6 | ✅ Complete |
| [TASK-159](tasks/TASK-159-canonicalize-repl-cli-contracts.md) | Canonicalize REPL/CLI contracts | SPEC-005/011/016 | 4 | ✅ Complete |
| [TASK-160](tasks/TASK-160-canonicalize-adt-contracts.md) | Canonicalize ADT contracts | SPEC-003/004/013/014/020 | 6 | ✅ Complete |

**Phase 20 Deliverable**: Canonicalized spec contracts for policy, workflow, streams/runtime verification, CLI/REPL, and ADT behavior

## Phase 21: Convergence Handoff Docs (Week 38)

Document explicit reference contracts between surface syntax, lowering, type checking, and runtime behavior before further implementation alignment.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-161](tasks/TASK-161-surface-to-parser-handoff-contract.md) | Surface-to-parser handoff contract | SPEC-002/013/020 | 4 | ✅ Complete |
| [TASK-162](tasks/TASK-162-parser-to-core-lowering-handoff-contract.md) | Parser-to-core lowering handoff contract | SPEC-001/002/006/013/020 | 4 | ✅ Complete |
| [TASK-163](tasks/TASK-163-type-runtime-handoff-contracts.md) | Type/runtime handoff contracts | SPEC-003/004/005/011/016 | 6 | ✅ Complete |

**Phase 21 Deliverable**: Reference contracts that freeze parser/lowering/type/runtime handoffs for convergence work

## Phase 22: Core Semantics Hardening (Week 39)

Tighten the canonical core language, execution-neutral IR contract, and per-phase judgment
boundaries before Rust-alignment work resumes.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-177](tasks/TASK-177-freeze-canonical-core-language-and-ir.md) | Freeze canonical core language and execution-neutral IR | SPEC-001/002/004 | 8 | ✅ Complete |
| [TASK-178](tasks/TASK-178-normalize-phase-judgments-and-rejection-boundaries.md) | Normalize phase judgments and rejection boundaries | SPEC-001/003/004 | 8 | ✅ Complete |

**Phase 22 Deliverable**: A canonical core contract with explicit phase-owned rejection boundaries

## Phase 23: Interaction Semantics Hardening (Week 40)

Tighten the highest-risk dynamic language semantics that still permit local implementation choice.
The canonical language no longer includes `attempt`/`catch`; recoverable failures are handled with
explicit `Result` values and pattern matching.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-179](tasks/TASK-179-formalize-receive-mailbox-and-scheduling-semantics.md) | Formalize `receive` mailbox and scheduling semantics | SPEC-002/004/013/017 | 8 | ✅ Complete |
| [TASK-180](tasks/TASK-180-formalize-policy-evaluation-and-verification-semantics.md) | Formalize policy evaluation and verification semantics | SPEC-003/004/006/007/008/017/018 | 8 | ✅ Complete |
| [TASK-185](tasks/TASK-185-remove-catch-and-require-explicit-result-handling.md) | Remove `catch` and require explicit `Result` handling | SPEC-002/004/014/016/017/020 | 6 | ✅ Complete |
| [TASK-181](tasks/TASK-181-formalize-adt-dynamic-semantics.md) | Formalize ADT dynamic semantics | SPEC-003/004/020 | 8 | ✅ Complete |

**Phase 23 Deliverable**: Proof-shaped and implementation-shaped semantics for `receive`, policy evaluation, explicit `Result`-based recovery, and ADTs

## Phase 24: Observable and Formalization Contracts (Week 41)

Define the single observable-behavior authority and the formalization boundary for future Lean work.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-182](tasks/TASK-182-add-runtime-observable-behavior-spec.md) | Add runtime observable behavior spec | SPEC-005/011/016/021 | 6 | ✅ Complete |
| [TASK-183](tasks/TASK-183-define-formalization-boundary-and-proof-targets.md) | Define formalization boundary and proof targets | SPEC-001/003/004/020/021 | 6 | ✅ Complete |

**Phase 24 Deliverable**: One normative observable-behavior spec and one explicit Lean formalization boundary

## Phase 25: Spec Hardening Audit (Week 42)

Audit whether the hardened spec set is ready to drive Rust and Lean implementations mechanically.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-184](tasks/TASK-184-audit-spec-hardening-readiness.md) | Audit spec hardening readiness | All hardened contracts | 6 | ✅ Complete |

**Phase 25 Deliverable**: Explicit readiness gate for mechanical Rust convergence and stable Lean modeling

## Monitoring Authority Gate (Week 43)

Define the monitor authority surface, exposed workflow views, and monitorability boundaries before
parser and runtime convergence resumes.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-186](tasks/TASK-186-monitor-authority-and-exposed-workflow-view.md) | Define monitor authority and exposed workflow views | SPEC-002/017/020/021 | 6 | ✅ Complete |

**Gate Deliverable**: Explicit monitor authority and exposed workflow views for later Rust convergence

## Runtime-Reasoner Design Review Gate (Week 44)

Freeze the runtime-only versus runtime-to-reasoner separation rules, audit the current canonical
docs against those rules, and synthesize the resulting spec-delta program before further language
and runtime contract revision resumes.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-187](tasks/TASK-187-freeze-runtime-reasoner-separation-rules.md) | Freeze runtime versus reasoner separation rules | Design note / SPEC-001 / SPEC-004 | 4 | ✅ Complete |
| [TASK-188](tasks/TASK-188-audit-runtime-and-verification-specs-for-reasoner-boundaries.md) | Audit runtime and verification specs for reasoner boundaries | SPEC-001/004/017/018 | 6 | ✅ Complete |
| [TASK-189](tasks/TASK-189-audit-surface-and-observability-docs-for-reasoner-boundaries.md) | Audit surface and observability docs for reasoner boundaries | SPEC-002/021 | 6 | ✅ Complete |
| [TASK-190](tasks/TASK-190-synthesize-runtime-reasoner-spec-delta-program.md) | Synthesize runtime-reasoner spec delta program | Design note / SPEC-001/002/004/017/018/021 | 6 | ✅ Complete |

**Gate Deliverable**: Frozen separation rules, completed audits, and one ordered spec-delta program that preserves runtime-only concerns while defining the review path for interaction-layer contracts

## Runtime-Reasoner Spec Follow-Up Phase (Week 45)

Complete the docs-only follow-up work required by the runtime-reasoner delta program before
planning any implementation convergence against the new interaction-facing material.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-191](tasks/TASK-191-define-runtime-to-reasoner-interaction-contract.md) | Define runtime-to-reasoner interaction contract | Design note / SPEC-004 | 6 | ✅ Complete |
| [TASK-192](tasks/TASK-192-add-runtime-authority-framing-to-spec-004.md) | Add runtime-authority framing to `SPEC-004` | SPEC-004 | 4 | ✅ Complete |
| [TASK-193](tasks/TASK-193-tighten-projection-and-monitorability-terminology.md) | Tighten projection and monitorability terminology | Design / reference | 4 | ✅ Complete |
| [TASK-194](tasks/TASK-194-define-human-facing-surface-guidance-boundary.md) | Define human-facing surface guidance boundary | SPEC-002 / reference | 5 | ✅ Complete |
| [TASK-195](tasks/TASK-195-synthesize-runtime-reasoner-spec-handoff.md) | Synthesize runtime-reasoner spec handoff | Follow-up docs corpus | 4 | ✅ Complete |

**Phase Deliverable**: One interaction contract, one minimal runtime-framing update, one terminology pass, one surface-guidance boundary note, and one implementation-readiness handoff with runtime-only protections preserved

## Runtime-Reasoner Implementation Planning Phase (Week 46)

Review the existing convergence queue against the new runtime-reasoner docs corpus and produce a
revised convergence map before opening any new code-facing tasks.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-196](tasks/TASK-196-audit-planned-convergence-tasks-against-runtime-reasoner-specs.md) | Audit planned convergence tasks against runtime-reasoner specs | Handoff / existing task corpus | 6 | ✅ Complete |
| [TASK-197](tasks/TASK-197-define-runtime-reasoner-implementation-planning-surface.md) | Define runtime-reasoner implementation-planning surface | Interaction / handoff docs | 5 | ✅ Complete |
| [TASK-198](tasks/TASK-198-synthesize-revised-runtime-reasoner-convergence-map.md) | Synthesize revised runtime-reasoner convergence map | Planning outputs | 5 | ✅ Complete |

**Phase Deliverable**: One impact audit of the current convergence queue, one implementation-planning surface note, and one revised convergence map for later code-facing task creation

## Runtime Boundary Implementation Planning Phase (Week 47)

Plan the authoritative runtime-boundary follow-up work separately from tooling and surface work,
then stop at a steering brief before opening any runtime code-facing tasks.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-199](tasks/TASK-199-audit-runtime-execution-boundaries-for-interaction-planning.md) | Audit runtime execution boundaries for interaction planning | SPEC-004 / interaction contract | 6 | ✅ Complete |
| [TASK-200](tasks/TASK-200-audit-runtime-trace-and-provenance-surfaces.md) | Audit runtime trace and provenance surfaces | SPEC-004 / planning surface | 5 | ✅ Complete |
| [TASK-201](tasks/TASK-201-synthesize-runtime-boundary-steering-brief.md) | Synthesize runtime boundary steering brief | Runtime-boundary audit outputs | 5 | ✅ Complete |

**Phase Deliverable**: Two runtime-boundary audits and one steering brief that identifies later runtime code-facing task clusters without opening them

## Tooling and Surface Implementation Planning Phase (Week 48)

Plan the CLI, REPL, trace-presentation, and explanatory surface follow-up work separately from the
authoritative runtime-boundary work, then stop at a steering brief before opening any tooling or
surface code-facing tasks.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-202](tasks/TASK-202-audit-cli-and-repl-surfaces-for-interaction-planning.md) | Audit CLI and REPL surfaces for interaction planning | SPEC-005/011 / runtime-observable contract | 6 | ✅ Complete |
| [TASK-203](tasks/TASK-203-audit-trace-export-and-presentation-surfaces.md) | Audit trace export and presentation surfaces | SPEC-005/016 / runtime-observable contract | 5 | ✅ Complete |
| [TASK-204](tasks/TASK-204-synthesize-tooling-and-surface-steering-brief.md) | Synthesize tooling and surface steering brief | Tooling/surface audit outputs | 5 | ✅ Complete |

**Phase Deliverable**: Two tooling/surface audits and one steering brief that identifies later user-facing task clusters without opening them

These two planning phases are additive review gates for later task creation. They do not change the
existing impact-review result that [TASK-164](tasks/TASK-164-route-receive-through-main-parser.md)
through [TASK-171](tasks/TASK-171-align-runtime-policy-outcomes.md) remain unchanged and
[TASK-172](tasks/TASK-172-unify-repl-implementation.md) and
[TASK-173](tasks/TASK-173-implement-repl-type-reporting.md) only need in-place reference updates.

## Runtime Boundary Implementation Phase (Week 49)

Implement the runtime-first hardening work identified by the runtime-boundary steering brief before
expanding user-facing tooling follow-up.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-205](tasks/TASK-205-implement-runtime-action-and-control-link-execution.md) | Implement runtime action and control-link execution | SPEC-004/017/018 | 10 | ✅ Complete |
| [TASK-206](tasks/TASK-206-align-runtime-admission-rejection-and-commitment-visibility.md) | Align runtime admission, rejection, and commitment visibility | SPEC-004/017/018/021 | 8 | ✅ Complete |
| [TASK-207](tasks/TASK-207-harden-runtime-trace-and-provenance-boundaries.md) | Harden runtime trace and provenance boundaries | SPEC-001/004/021 | 8 | ✅ Complete |

**Phase Deliverable**: Completed runtime execution branches, explicit runtime boundary behavior, and hardened trace/provenance capture aligned with accepted runtime progression

Execution note: this phase is downstream from Phase 28. Treat
[TASK-205](tasks/TASK-205-implement-runtime-action-and-control-link-execution.md),
[TASK-206](tasks/TASK-206-align-runtime-admission-rejection-and-commitment-visibility.md), and
[TASK-207](tasks/TASK-207-harden-runtime-trace-and-provenance-boundaries.md) as runtime hardening
work that begins only after [TASK-170](tasks/TASK-170-implement-end-to-end-receive-execution.md)
and [TASK-171](tasks/TASK-171-align-runtime-policy-outcomes.md) are complete.
Execution note: [TASK-211](tasks/TASK-211-revise-control-link-authority-contract.md) is a
documentation gate for this phase and must complete before
[TASK-205](tasks/TASK-205-implement-runtime-action-and-control-link-execution.md).
Execution note: [TASK-205](tasks/TASK-205-implement-runtime-action-and-control-link-execution.md)
uses a transitional shared control-link registry so transferred links remain valid across
executions; [TASK-206](tasks/TASK-206-align-runtime-admission-rejection-and-commitment-visibility.md)
must replace that fallback with explicit runtime-owned lifecycle state. The current implementation
retains terminated instances as tombstones; the long-term retention and cleanup design was later
frozen by [TASK-212](tasks/TASK-212-design-control-link-retention-policy.md).

Execution note: [TASK-207](tasks/TASK-207-harden-runtime-trace-and-provenance-boundaries.md)
lands as a runtime-only provenance session API plus wrapper-caller convergence. It hardens
workflow entry/exit framing without reclassifying CLI or macro surfaces as anything other than
runtime observability.

## Tooling Observable Convergence Extension (Week 50)

Finish the minimum-risk user-facing convergence work identified by the tooling/surface steering
brief by building on the existing REPL tasks and one new CLI output-alignment task.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-208](tasks/TASK-208-align-cli-run-and-trace-observable-output.md) | Align CLI run and trace observable output | SPEC-005/011/021 | 8 | ✅ Complete |

**Extension Deliverable**: Shared REPL authority and canonical `:type` reporting via [TASK-172](tasks/TASK-172-unify-repl-implementation.md) / [TASK-173](tasks/TASK-173-implement-repl-type-reporting.md), plus CLI `run` / `trace` output aligned with the observable contract via TASK-208

Execution note: this extension is downstream from Phase 29. Execute
[TASK-172](tasks/TASK-172-unify-repl-implementation.md), then
[TASK-173](tasks/TASK-173-implement-repl-type-reporting.md), then
[TASK-208](tasks/TASK-208-align-cli-run-and-trace-observable-output.md).

The presentation-only stage-guidance overlay remains intentionally deferred until the observable
contract is implemented cleanly.

## Phase 26: Parser and Lowering Convergence (Week 44)

These implementation phases are blocked until the monitoring authority gate confirms that the specification is
unambiguous enough to drive Rust work mechanically.

Align parser dispatch, AST shape, and lowering behavior with the frozen contracts.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-164](tasks/TASK-164-route-receive-through-main-parser.md) | Route `receive` through main parser | SPEC-002/013 | 4 | ✅ Complete |
| [TASK-165](tasks/TASK-165-align-check-decide-ast-contracts.md) | Align `check` and `decide` AST contracts | SPEC-001/002 | 6 | ✅ Complete |
| [TASK-166](tasks/TASK-166-replace-placeholder-policy-lowering.md) | Replace placeholder policy lowering | SPEC-001/006/007 | 6 | ✅ Complete |
| [TASK-167](tasks/TASK-167-lower-receive-into-canonical-core-form.md) | Lower `receive` into canonical core form | SPEC-001/013 | 6 | ✅ Complete |

**Phase 26 Deliverable**: Parser and lowering layers aligned with the hardened canonical workflow, policy, and `receive` contracts

## Phase 27: Type and Verification Convergence (Week 45)

Bring type checking and runtime verification context into line with the frozen contracts.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-168](tasks/TASK-168-align-type-checking-for-policies-and-receive.md) | Align type checking for policies and `receive` | SPEC-003/006/013/017 | 8 | ✅ Complete |
| [TASK-169](tasks/TASK-169-unify-runtime-verification-context-and-obligation-enforcement.md) | Unify runtime verification context and obligation enforcement | SPEC-017/018 | 6 | ✅ Complete |
| [TASK-209](tasks/TASK-209-separate-runtime-verification-input-classes.md) | Separate runtime verification input classes | SPEC-017/018 | 4 | ✅ Complete |

**Phase 27 Deliverable**: Type and verification layers enforce the hardened canonical policy and stream contracts without conflating capability declarations and obligation-backed runtime requirements

## Phase 28: Runtime Convergence (Week 46)

Complete runtime alignment for `receive` execution and policy outcomes.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-170](tasks/TASK-170-implement-end-to-end-receive-execution.md) | Implement end-to-end `receive` execution | SPEC-004/013/017 | 8 | ✅ Complete |
| [TASK-171](tasks/TASK-171-align-runtime-policy-outcomes.md) | Align runtime policy outcomes | SPEC-017/018 | 6 | ✅ Complete |

**Phase 28 Deliverable**: Runtime behavior aligned with hardened canonical `receive` and policy-outcome contracts

Execution note: Phase 28 remains the upstream runtime convergence work. The later runtime-boundary
implementation phase extends this runtime path and should not begin before Phase 28 is complete.
Execution note: [TASK-209](tasks/TASK-209-separate-runtime-verification-input-classes.md) is a gating follow-up from Phase 27 and must complete before [TASK-170](tasks/TASK-170-implement-end-to-end-receive-execution.md) and [TASK-171](tasks/TASK-171-align-runtime-policy-outcomes.md).

## Phase 29: REPL and CLI Convergence (Week 47)

Align the implementation of REPL and CLI behavior with the frozen command and output contracts.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-172](tasks/TASK-172-unify-repl-implementation.md) | Unify REPL implementation | SPEC-005/011/016 | 8 | ✅ Complete |
| [TASK-173](tasks/TASK-173-implement-repl-type-reporting.md) | Implement REPL type reporting | SPEC-003/005/011 | 6 | ✅ Complete |

**Phase 29 Deliverable**: One authoritative REPL implementation with canonical type reporting

Execution note: Phase 29 is also the front half of the later tooling observable convergence
extension. Complete [TASK-172](tasks/TASK-172-unify-repl-implementation.md) and
[TASK-173](tasks/TASK-173-implement-repl-type-reporting.md) before
[TASK-208](tasks/TASK-208-align-cli-run-and-trace-observable-output.md).

## Phase 30: ADT Convergence (Week 48)

Align ADT implementation layers and user-visible stdlib surface with the canonical ADT contract.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-174](tasks/TASK-174-align-adt-type-value-and-pattern-contracts.md) | Align ADT type, value, and pattern contracts | SPEC-003/004/020 | 10 | ✅ Complete |
| [TASK-175](tasks/TASK-175-align-adt-stdlib-and-example-surface.md) | Align ADT stdlib and example surface | SPEC-020 | 6 | ✅ Complete |

**Phase 30 Deliverable**: Canonical ADT contracts implemented from parser/runtime through stdlib surface

## Phase 31: Final Convergence Audit (Week 49)

Re-audit specs and implementation to close the convergence program.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-176](tasks/TASK-176-final-convergence-audit.md) | Final convergence audit | All convergence contracts | 6 | ✅ Complete |

**Phase 31 Deliverable**: Final audit report and explicit convergence status for the completed Rust/spec implementation path

Closeout note: the main Rust/spec convergence path is complete. The final audit originally left
[TASK-212](tasks/TASK-212-design-control-link-retention-policy.md) and a small set of residual
spec-only findings as explicit follow-ups; those later closed through TASK-212 and Phase 34 rather
than being left as hidden convergence drift.

Execution note: final convergence closeout now depends on the downstream runtime-boundary and
tooling observable convergence work as well as the original convergence phases.

## Phase 32: CI Hygiene

Clear repository-level warnings that still break the enforced local and CI quality gates.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-210](tasks/TASK-210-clear-workspace-clippy-warnings.md) | Clear workspace clippy warnings | SPEC-021 | 1 | ✅ Complete |

**Phase 32 Deliverable**: Clean workspace clippy gate for the currently merged codebase

## Phase 33: Control Authority Contract Revision

Freeze the reusable-control semantics for `ControlLink` before the next runtime hardening batch
implements supervision behavior.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-211](tasks/TASK-211-revise-control-link-authority-contract.md) | Revise control-link authority contract | SPEC-002/004/020/021 | 4 | ✅ Complete |
| [TASK-212](tasks/TASK-212-design-control-link-retention-policy.md) | Design control-link retention policy | SPEC-004/021 | 3 | ✅ Complete |

**Phase 33 Deliverable**: Canonical docs updated so runtime supervision uses reusable control
authority rather than affine one-shot control, and terminal control retention is frozen as
runtime-state-owned tombstone visibility rather than hidden background cleanup.

## Phase 34: Residual Spec-Audit Follow-up

Close the explicit spec-only documentation debt that remained after the final convergence audit.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-213](tasks/TASK-213-reconcile-module-and-import-spec-scope.md) | Reconcile module and import spec scope | SPEC-009/012 | 3 | ✅ Complete |
| [TASK-214](tasks/TASK-214-fix-residual-policy-and-typed-provider-spec-drift.md) | Fix residual policy and typed-provider spec drift | SPEC-007/010/015/016 | 4 | ✅ Complete |
| [TASK-215](tasks/TASK-215-normalize-residual-spec-hygiene.md) | Normalize residual spec hygiene | Affected specs | 3 | ✅ Complete |

**Phase 34 Deliverable**: Residual spec-only findings from the final convergence audit are closed
or explicitly reclassified without reopening the completed implementation convergence path.

Execution note: [TASK-213](tasks/TASK-213-reconcile-module-and-import-spec-scope.md),
[TASK-214](tasks/TASK-214-fix-residual-policy-and-typed-provider-spec-drift.md), and
[TASK-215](tasks/TASK-215-normalize-residual-spec-hygiene.md) are complete. The final audit’s
residual spec-only findings are now closed, and [TASK-212](tasks/TASK-212-design-control-link-retention-policy.md)
later closed the remaining control-link retention follow-up in this area.

## Phase 35: Role Contract Simplification and Convergence

Align the canonical role contracts with the simplified authority-plus-obligations model, then
reopen the minimum implementation work needed to remove legacy role-supervision residue and
support source role definitions end to end.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-216](tasks/TASK-216-canonicalize-role-contracts.md) | Canonicalize role contracts | SPEC-001/002/017/018 | 4 | ✅ Complete |
| [TASK-217](tasks/TASK-217-remove-legacy-role-supervision-shape.md) | Remove legacy role supervision shape | SPEC-001/002 | 6 | ✅ Complete |
| [TASK-218](tasks/TASK-218-implement-source-role-definition-parsing-and-lowering.md) | Implement source role definition parsing and lowering | SPEC-001/002 | 8 | ✅ Complete |
| [TASK-219](tasks/TASK-219-align-runtime-role-approval-contract.md) | Align runtime role approval contract | SPEC-017/018 | 6 | ✅ Complete |
| [TASK-220](tasks/TASK-220-audit-role-convergence-and-align-examples.md) | Audit role convergence and align examples | Affected specs/examples | 4 | ✅ Complete |

**Phase 35 Deliverable**: Canonical role contracts no longer encode supervision, and the remaining
implementation work is split into focused parser/core, parser/lowering, runtime-approval, and
example/audit tasks.

## Phase 36: Role Convergence Blocker Remediation

Resolve the remaining blocker-class gaps from the Phase 35 review: remove placeholder role
obligation lowering, make role-definition lowering participate in an honest end-to-end parser/core
path, and reconcile touched docs/examples with the canonical surface contract.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-221](tasks/TASK-221-align-core-role-obligation-carrier.md) | Align core role obligation carrier | SPEC-001/002 | 6 | ✅ Complete |
| [TASK-222](tasks/TASK-222-integrate-role-definition-lowering-path.md) | Integrate role definition lowering path | SPEC-001/002/009 | 8 | ✅ Complete |
| [TASK-223](tasks/TASK-223-canonicalize-touched-role-docs-and-examples.md) | Canonicalize touched role docs and examples | SPEC-002/017/018 | 6 | ✅ Complete |
| [TASK-224](tasks/TASK-224-role-convergence-closeout-audit.md) | Role convergence closeout audit | Affected specs/examples | 4 | ✅ Complete |
| [TASK-225](tasks/TASK-225-inline-module-role-honesty-fix.md) | Inline module role honesty fix | SPEC-002/009 | 3 | ✅ Complete |

**Phase 36 Deliverable**: Complete. Role-definition support no longer relies on placeholder
obligation semantics, touched docs/examples stop overstating convergence, the inline-module parser
rejects unsupported canonical items honestly even after recovery, and the branch now carries a
focused closeout audit for the remaining intentional historical/process-supervision references.

## Phase 37: Workflow Typing with Constraints

Implement workflow contracts with Hoare-style pre/post-conditions, linear obligation tracking, and requirement checking.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-226](tasks/TASK-226-workflow-contracts-ast.md) | Workflow contracts AST extensions | SPEC-022 | 8 | ✅ Complete |
| [TASK-227](tasks/TASK-227-type-check-obligations.md) | Type check obligations as linear resources | SPEC-022 | 10 | ✅ Complete |
| [TASK-228](tasks/TASK-228-requirement-checking.md) | Requirement checking at call sites | SPEC-022 | 8 | ✅ Complete |
| [TASK-229](tasks/TASK-229-audit-trail-integration.md) | Audit trail for obligation checks | SPEC-022 | 6 | ✅ Complete |
| [TASK-230](tasks/TASK-230-parser-updates.md) | Parser updates for contract syntax | SPEC-022 | 8 | ✅ Complete |
| [TASK-231](tasks/TASK-231-integration-tests.md) | End-to-end integration tests | SPEC-022 | 6 | ✅ Complete |
| [TASK-232](tasks/TASK-232-canonicalize-spec-022.md) | Canonicalize SPEC-022 workflow typing | SPEC-022 | 4 | ✅ Complete |

**Phase 37 Deliverable**: Complete. Workflow contracts with requires/ensures clauses, linear
obligation tracking (oblige/check), requirement checking with capabilities/roles, and audit trail
integration. SPEC-022 canonicalized in docs/spec/.

---

## Future Phases: Governance and Collaboration

See [PHASES-38-43-ROADMAP.md](PHASES-38-43-ROADMAP.md) for detailed dependency graph and planning.

### Phase 38: Capability Definition Specification

**Goal:** Revise SPEC-017 to add capability definition parsing requirements.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-233](tasks/TASK-233-SPEC-017-CAPABILITY-PARSING.md) | SPEC-017 revision: capability parsing | SPEC-017 | 8 | 📋 Ready |

### Phase 39: Capability Definition Implementation

**Goal:** Implement parser support for capability definitions in `.ash` files.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-234](tasks/TASK-234-CAPABILITY-PARSER-IMPL.md) | Implement capability definition parser | SPEC-017 | 20 | ✅ Complete |

### Phase 40: Role Runtime Semantics

**Goal:** Specify and implement role authority and obligation enforcement.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-235](tasks/TASK-235-SPEC-019-ROLE-SEMANTICS.md) | SPEC-019: role runtime semantics | SPEC-019 | 12 | 📋 Ready |
| [TASK-236](tasks/TASK-236-ROLE-RUNTIME-IMPL.md) | Implement role runtime enforcement | SPEC-019 | 30 | ⏳ Blocked on TASK-235 |

### Decision Point: Obligation Syntax

**Goal:** Decide on obligation syntax direction.

| Task | Description | Type | Status |
|------|-------------|------|--------|
| [DECISION-237](tasks/TASK-237-OBLIGATION-SYNTAX-DECISION.md) | Obligation syntax: local vs role-bound | Decision | 📋 Ready |

### Phase 41-42: Proxy Workflows

**Goal:** Enable human-AI collaboration via proxy workflows.

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-238](tasks/TASK-238-SPEC-023-PROXY-WORKFLOWS.md) | SPEC-023: proxy workflows | SPEC-023 | 16 | 📋 Ready |
| [TASK-239](tasks/TASK-239-PROXY-WORKFLOW-IMPL.md) | Implement proxy workflow runtime | SPEC-023 | 50 | ⏳ Blocked on TASK-238 |

**Note:** No release is currently planned for these phases. Work can proceed according to dependency constraints and priorities.
