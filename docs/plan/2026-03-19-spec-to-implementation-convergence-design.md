# Spec-to-Implementation Convergence Design

## Goal

Define a spec-first recovery path that ends with consistent Ash specifications and a Rust implementation that matches those specifications across surface syntax, AST, lowering/IR, type checking, interpreter/runtime verification, CLI/REPL, and standard library behavior.

## Context

This design is based on the following audits:
- `docs/audit/2026-03-19-spec-001-018-consistency-review.md`
- `docs/audit/2026-03-19-task-consistency-review-non-lean.md`
- `docs/audit/2026-03-19-rust-codebase-review-findings.md`

The audits show that the primary risk is layered drift. Different layers currently implement partially different models of the same feature, especially for policies, `receive`, runtime verification, REPL/CLI behavior, and ADTs.

## Decision Summary

### 1. Authority model
The recovery path is spec-first. Specifications are the authoritative contract. Existing tasks are informative only and do not constrain the new work.

### 2. Task strategy
Do not retrofit old task history. Create new follow-up tasks that each stabilize one contract boundary or one tightly coupled contract pair.

### 3. Ordering rule
Repair in dependency order:
1. Canonical specs
2. Layer handoff contracts
3. Rust implementation convergence
4. Final compliance audit

### 4. Quality rule
Every Rust implementation task must be TDD-first and must use the Rust best-practice guidance already adopted in this repository.

## Why this design

The current failures are mostly boundary failures:
- surface syntax does not always agree with surface AST,
- surface AST does not always agree with core AST or lowering,
- lowering does not always preserve the semantics assumed by type checking,
- runtime verification does not consistently define the same execution contract used by interpreter paths,
- CLI and REPL entrypoints expose different observable behavior.

A code-first repair would likely stabilize the wrong contracts. A cluster-by-cluster repair without a canonical spec pass would likely recreate contradictions between layers.

## Scope

This convergence effort covers:
- `docs/spec/SPEC-001-IR.md`
- `docs/spec/SPEC-002-SURFACE.md`
- `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- `docs/spec/SPEC-004-SEMANTICS.md`
- `docs/spec/SPEC-005-CLI.md`
- `docs/spec/SPEC-006-POLICY-DEFINITIONS.md`
- `docs/spec/SPEC-007-POLICY-COMBINATORS.md`
- `docs/spec/SPEC-008-DYNAMIC-POLICIES.md`
- `docs/spec/SPEC-011-REPL.md`
- `docs/spec/SPEC-013-STREAMS.md`
- `docs/spec/SPEC-014-BEHAVIOURS.md`
- `docs/spec/SPEC-015-TYPED-PROVIDERS.md`
- `docs/spec/SPEC-016-OUTPUT.md`
- `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- `docs/spec/SPEC-018-CAPABILITY-MATRIX.md`
- `docs/spec/SPEC-020-ADT-TYPES.md`

Primary implementation areas:
- `crates/ash-core`
- `crates/ash-parser`
- `crates/ash-typeck`
- `crates/ash-interp`
- `crates/ash-engine`
- `crates/ash-cli`
- `crates/ash-repl`
- `std`

## Phase model

### Phase A — Canonical spec repair
Freeze one internally consistent written contract for:
- workflow forms and effect vocabulary,
- policy model,
- streams and runtime verification,
- REPL/CLI behavior,
- ADT semantics.

### Phase B — Pipeline handoff contracts
Add concise reference docs or spec sections that define what each layer receives, emits, and rejects at these boundaries:
- surface syntax → parser AST,
- parser AST → core AST/lowering,
- lowering/IR → type checking,
- type checking → interpreter/runtime verification,
- runtime → CLI/REPL observables,
- language/runtime → stdlib and examples.

### Phase C — Rust convergence tasks
Implement fresh, self-contained tasks in strict dependency order:
1. syntax and parser routing,
2. surface/core AST alignment,
3. lowering/IR alignment,
4. type checking alignment,
5. interpreter/runtime verification alignment,
6. CLI/REPL unification,
7. stdlib/examples alignment.

### Phase D — Closure
Re-run both document and implementation audits. Convergence is complete only when no known drift remains in the previously audited clusters.

## Task-shaping rules

A good follow-up task must:
- stabilize exactly one contract boundary or one tightly-coupled pair,
- cite one primary spec authority,
- be small enough to verify in one focused test run,
- be large enough to produce one meaningful invariant,
- declare explicit non-goals.

A task is too large if it says “fix policies”, “stabilize ADTs”, or “finish runtime verification”.
A task is too small if it changes an isolated helper or a name without stabilizing a boundary.

## Required task template

Each follow-up task produced from this plan should include:
1. Contract being stabilized
2. Spec references
3. Rust files likely affected
4. Failing tests to add first
5. Minimal implementation target
6. Verification commands
7. Explicit non-goals

## Acceptance criteria

The convergence program is complete only when all of the following are true:
- Specs are internally consistent across the covered feature areas.
- Surface syntax, AST, lowering/IR, type checking, interpreter, runtime verification, CLI/REPL, and stdlib agree on the same feature contracts.
- `receive`, policies, REPL/CLI, runtime verification, and ADTs have end-to-end test coverage at their boundary points.
- The final audits no longer report the currently known drift classes.

## Non-goals

This design does not:
- preserve historic task numbering or completion claims,
- attempt to make older task documents authoritative,
- mix Lean follow-up into the Rust convergence sequence,
- optimize for shortest code-change path over contract correctness.
