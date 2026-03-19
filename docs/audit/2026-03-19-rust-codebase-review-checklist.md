# Rust Codebase Review Checklist

## Purpose

Use this checklist to review the Rust implementation against the drift-heavy areas identified in:
- [docs/audit/2026-03-19-spec-001-018-consistency-review.md](2026-03-19-spec-001-018-consistency-review.md)
- [docs/audit/2026-03-19-task-consistency-review-non-lean.md](2026-03-19-task-consistency-review-non-lean.md)

This checklist is organized by review risk rather than by phase.

## Scope

Review targets include:
- `ash-core`
- `ash-parser`
- `ash-typeck`
- `ash-interp`
- `ash-engine`
- `ash-cli`
- `ash-repl`
- `std`

Goal of the review:
- confirm where code matches the intended model,
- identify where code follows stale task guidance instead of current spec intent,
- separate true implementation bugs from documentation drift.

## How to Use This Checklist

For each cluster:
1. Read the linked audit findings first.
2. Inspect the listed Rust files.
3. Answer the review questions in order.
4. Record whether the code aligns with:
   - current spec text,
   - task-document intent,
   - actual runtime behavior.
5. Mark each mismatch as one of:
   - spec drift,
   - task drift,
   - code bug,
   - intentional implementation divergence.

---

## 1. Baseline Review Pass: Lower-Risk Anchors

Start here before the high-risk clusters.

### Files to inspect
- [crates/ash-core/src/ast.rs](../crates/ash-core/src/ast.rs)
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs)
- [crates/ash-parser/src/parse_workflow.rs](../crates/ash-parser/src/parse_workflow.rs)
- [crates/ash-typeck/src/check_expr.rs](../crates/ash-typeck/src/check_expr.rs)

### Checklist
- [ ] Confirm core AST names still match the intended IR vocabulary from the core specs.
- [ ] Check whether parser surface forms lower cleanly into the current AST.
- [ ] Verify effect-related code still reflects the four-level lattice consistently.
- [ ] Note any foundational naming drift before reviewing higher-risk clusters.

---

## 2. Policy Cluster Review

### Why this cluster is risky
This cluster inherits known drift between policy definitions, combinators, and dynamic policies from the earlier audits.

### Files to inspect
- [crates/ash-parser/src/surface.rs](../crates/ash-parser/src/surface.rs)
- [crates/ash-parser/src/parse_workflow.rs](../crates/ash-parser/src/parse_workflow.rs)
- [crates/ash-typeck/src/policy_check.rs](../crates/ash-typeck/src/policy_check.rs)
- [crates/ash-typeck/src/check_expr.rs](../crates/ash-typeck/src/check_expr.rs)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs)

### Audit inputs
- [docs/audit/2026-03-19-spec-001-018-consistency-review.md](2026-03-19-spec-001-018-consistency-review.md)
- [docs/audit/2026-03-19-task-consistency-review-non-lean.md](2026-03-19-task-consistency-review-non-lean.md)
- [docs/plan/tasks/TASK-061-policy-definitions.md](../plan/tasks/TASK-061-policy-definitions.md)
- [docs/plan/tasks/TASK-062-policy-combinators.md](../plan/tasks/TASK-062-policy-combinators.md)
- [docs/plan/tasks/TASK-063-dynamic-policies.md](../plan/tasks/TASK-063-dynamic-policies.md)

### Checklist
- [ ] Check whether policy definitions are modeled as declarations, first-class values, or both.
- [ ] Verify whether `check` operations in code align with the policy model actually implemented.
- [ ] Inspect whether combinator support matches task expectations or was intentionally narrowed.
- [ ] Confirm SMT conflict detection behavior, especially where docs claim conflicts that may be satisfiable.
- [ ] Identify whether runtime policy evaluation paths exist only for verification or also for execution.
- [ ] Flag any places where parser/type checker/runtime use incompatible policy representations.

### Drift signatures to look for
- Struct-like policies in one layer, expression-like policies in another
- Dead or partial dynamic-policy hooks
- SMT checks that do not match documented examples
- `Policy` types that exist in type checking but not parser/runtime shape

---

## 3. REPL and CLI Cluster Review

### Why this cluster is risky
The audits found command-surface drift between REPL and CLI specs, and task docs disagree on the supported command set.

### Files to inspect
- [crates/ash-repl/src/lib.rs](../crates/ash-repl/src/lib.rs)
- [crates/ash-repl/src/main.rs](../crates/ash-repl/src/main.rs)
- [crates/ash-cli/src/commands/repl.rs](../crates/ash-cli/src/commands/repl.rs)
- [crates/ash-engine/src/lib.rs](../crates/ash-engine/src/lib.rs)
- [crates/ash-engine/src/parse.rs](../crates/ash-engine/src/parse.rs)
- [crates/ash-engine/src/check.rs](../crates/ash-engine/src/check.rs)
- [crates/ash-engine/src/execute.rs](../crates/ash-engine/src/execute.rs)
- [crates/ash-engine/src/providers.rs](../crates/ash-engine/src/providers.rs)

### Audit inputs
- [docs/plan/tasks/TASK-056-cli-repl.md](../plan/tasks/TASK-056-cli-repl.md)
- [docs/plan/tasks/TASK-080-repl-commands.md](../plan/tasks/TASK-080-repl-commands.md)
- [docs/spec/SPEC-005-CLI.md](../spec/SPEC-005-CLI.md)
- [docs/spec/SPEC-011-REPL.md](../spec/SPEC-011-REPL.md)

### Checklist
- [ ] List the commands actually implemented in `ash-repl` and compare them with both REPL task docs.
- [ ] Confirm whether `ash-cli repl` delegates to `ash-repl` or implements a separate command surface.
- [ ] Check whether command names use `:ast`, `:parse`, `:effect`, `:dot`, `:load`, or a subset.
- [ ] Verify whether displayed type names follow actual `Type` names or task/spec example names.
- [ ] Inspect any expression-wrapping logic for stale inline `action { ... }` syntax assumptions.
- [ ] Check history/config/startup option handling against both CLI and REPL task expectations.

### Drift signatures to look for
- Duplicate REPL implementations with different command sets
- Type display using `Number` while the rest of the code uses `Int`
- Command handlers that do not match documented behavior
- Provider/effect assumptions leaking into REPL evaluation paths

---

## 4. Streams and Runtime Verification Cluster Review

### Why this cluster is risky
This cluster carries forward the unstable `receive` surface and the incomplete runtime-verification model from SPEC-018.

### Files to inspect
- [crates/ash-parser/src/parse_receive.rs](../crates/ash-parser/src/parse_receive.rs)
- [crates/ash-interp/src/execute_stream.rs](../crates/ash-interp/src/execute_stream.rs)
- [crates/ash-interp/src/stream.rs](../crates/ash-interp/src/stream.rs)
- [crates/ash-interp/src/execute_observe.rs](../crates/ash-interp/src/execute_observe.rs)
- [crates/ash-interp/src/execute_set.rs](../crates/ash-interp/src/execute_set.rs)
- [crates/ash-interp/src/exec_send.rs](../crates/ash-interp/src/exec_send.rs)
- [crates/ash-typeck/src/capability_check.rs](../crates/ash-typeck/src/capability_check.rs)
- [crates/ash-typeck/src/capability_typecheck.rs](../crates/ash-typeck/src/capability_typecheck.rs)
- [crates/ash-typeck/src/obligation_checker.rs](../crates/ash-typeck/src/obligation_checker.rs)
- [crates/ash-typeck/src/runtime_verification.rs](../crates/ash-typeck/src/runtime_verification.rs)

### Audit inputs
- [docs/plan/tasks/TASK-090-parse-receive.md](../plan/tasks/TASK-090-parse-receive.md)
- [docs/plan/tasks/TASK-112-capability-verification.md](../plan/tasks/TASK-112-capability-verification.md)
- [docs/plan/tasks/TASK-115-obligation-checker.md](../plan/tasks/TASK-115-obligation-checker.md)
- [docs/plan/tasks/TASK-118-operation-verifier.md](../plan/tasks/TASK-118-operation-verifier.md)
- [docs/spec/SPEC-013-STREAMS.md](../spec/SPEC-013-STREAMS.md)
- [docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md](../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md)

### Checklist
- [ ] Confirm the exact `receive` syntax the parser accepts and compare it with all documented variants.
- [ ] Check whether control-stream behavior is first-class, ad hoc, or absent.
- [ ] Verify whether declaration checking for `observe`/`receive`/`set`/`send` is compile-time only or duplicated at runtime.
- [ ] Inspect whether runtime verification has a single coherent `RuntimeContext` model.
- [ ] Check whether missing obligations are warnings, errors, or split by undocumented criticality.
- [ ] Verify whether per-operation denial returns an error or a result enum variant.
- [ ] Confirm rate-limit, approval, and transformation handling paths against actual code behavior.

### Drift signatures to look for
- Multiple incompatible `RuntimeContext` assumptions in one module
- `NotSettable` / `NotWritable` style naming mismatches in code paths
- `receive control` support in parser but not execution
- Divergence between declaration verification and runtime verification rules

---

## 5. ADT Cluster Review

### Why this cluster is risky
The task audit found the heaviest task-level drift in ADT planning, including phase drift, type-model drift, duplicated exhaustiveness work, and runtime value shape mismatch.

### Files to inspect
- [crates/ash-core/src/ast.rs](../crates/ash-core/src/ast.rs)
- [crates/ash-parser/src/parse_type_def.rs](../crates/ash-parser/src/parse_type_def.rs)
- [crates/ash-typeck/src/check_pattern.rs](../crates/ash-typeck/src/check_pattern.rs)
- [crates/ash-typeck/src/check_expr.rs](../crates/ash-typeck/src/check_expr.rs)
- [crates/ash-interp/src/eval.rs](../crates/ash-interp/src/eval.rs)
- [crates/ash-interp/src/pattern.rs](../crates/ash-interp/src/pattern.rs)
- [std/src/option.ash](../std/src/option.ash)
- [std/src/result.ash](../std/src/result.ash)

### Audit inputs
- [docs/plan/tasks/TASK-120-ast-extensions.md](../plan/tasks/TASK-120-ast-extensions.md)
- [docs/plan/tasks/TASK-121-adt-core-types.md](../plan/tasks/TASK-121-adt-core-types.md)
- [docs/plan/tasks/TASK-122-adt-runtime-values.md](../plan/tasks/TASK-122-adt-runtime-values.md)
- [docs/plan/tasks/TASK-128-type-check-patterns.md](../plan/tasks/TASK-128-type-check-patterns.md)
- [docs/plan/tasks/TASK-130-exhaustiveness-checking.md](../plan/tasks/TASK-130-exhaustiveness-checking.md)
- [docs/plan/tasks/TASK-131-constructor-evaluation.md](../plan/tasks/TASK-131-constructor-evaluation.md)
- [docs/plan/tasks/TASK-132-pattern-matching-engine.md](../plan/tasks/TASK-132-pattern-matching-engine.md)
- [docs/plan/tasks/TASK-133-match-evaluation.md](../plan/tasks/TASK-133-match-evaluation.md)
- [docs/plan/tasks/TASK-135-control-link-transfer.md](../plan/tasks/TASK-135-control-link-transfer.md)
- [docs/plan/tasks/TASK-136-option-result-library.md](../plan/tasks/TASK-136-option-result-library.md)
- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md)

### Checklist
- [ ] Verify the actual AST/type-definition model used in code and compare it with both SPEC-020 and the ADT task chain.
- [ ] Check whether runtime values use one consistent field shape for ADT instances and variants.
- [ ] Inspect whether constructor typing and pattern typing share the same type representation.
- [ ] Determine whether exhaustiveness is implemented once or duplicated across modules.
- [ ] Verify match evaluation, pattern matching, and constructor evaluation use the same runtime value conventions.
- [ ] Review control-link handling separately from ADT basics; confirm whether transfer semantics follow the design note or later task interpretation.
- [ ] Compare `std` Option/Result helpers with the documented library surface and note omissions versus intentional narrowing.

### Drift signatures to look for
- `Type`, `TypeExpr`, and runtime-value models that do not compose cleanly
- Variant field names changing across parser/type checker/interpreter
- Exhaustiveness logic implemented in more than one place
- Control-link semantics bolted onto ADT work without a stable interface

---

## 6. Cross-Cutting Verification Items

Apply these checks in every cluster.

### Checklist
- [ ] Does the crate-level public API match the task doc, the spec, or neither?
- [ ] Are naming conventions consistent across parser, type checker, interpreter, and CLI layers?
- [ ] Do tests encode current behavior or stale documented behavior?
- [ ] Are error names and messages stable across compile-time and runtime layers?
- [ ] Is there any code path that clearly implements an abandoned or contradictory task assumption?
- [ ] Are task docs marked complete despite code obviously reflecting only partial implementation?

---

## 7. Recommended Review Order

1. Baseline anchors in `ash-core`, `ash-parser`, and `ash-typeck`
2. Module/import work as a relatively coherent intermediate pass
3. Policy cluster
4. REPL and CLI cluster
5. Streams and runtime-verification cluster
6. ADT cluster last, because it has the densest task drift

## 8. Review Output Template

Use this template while reviewing code:

- **Cluster:**
- **Files reviewed:**
- **Spec/task references checked:**
- **Matches current intent:** yes / partial / no
- **Primary issue type:** spec drift / task drift / code bug / intentional divergence
- **Notes:**
- **Follow-up action:**

## Conclusion

This checklist is meant to reduce false positives during Rust review by separating stable anchors from the clusters most affected by spec/task drift.
