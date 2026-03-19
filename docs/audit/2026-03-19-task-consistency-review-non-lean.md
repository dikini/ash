# Audit: Non-Lean Task Consistency Review

## Scope

Reviewed non-Lean task documents in [docs/plan/tasks](../plan/tasks) for consistency in preparation for reviewing the Rust implementation.

This audit used the earlier spec audit in [docs/audit/2026-03-19-spec-001-018-consistency-review.md](2026-03-19-spec-001-018-consistency-review.md) as prior context.

Constraints of this audit:
- Reporting only
- No task files changed
- Lean-related tasks excluded
- Focus on consistency between task docs, referenced specs, and [docs/plan/PLAN-INDEX.md](../plan/PLAN-INDEX.md)

Review date: 2026-03-19

## Summary

The task set is uneven.

Early core tasks are mostly aligned with the corresponding specifications and are suitable as relatively safe anchors for reviewing the Rust codebase. The highest concentration of drift appears in four clusters:
- policy tasks,
- REPL and CLI-adjacent tasks,
- stream and runtime-verification tasks,
- ADT tasks.

Several task documents also conflict with the completion conventions in [docs/plan/PLAN-INDEX.md](../plan/PLAN-INDEX.md), most visibly by remaining marked complete while retaining unchecked completion lists.

## Relationship to the Prior Spec Audit

The earlier spec audit found drift in these areas:
- module/import scope,
- policy syntax and abstraction level,
- REPL versus CLI command surface,
- `receive` syntax,
- runtime verification model details in SPEC-018.

This task audit shows that many task documents preserved or amplified those same inconsistencies rather than resolving them before implementation guidance was written.

## High-Severity Findings

### 1. ADT phase and plan indexing drift

- [docs/plan/tasks/TASK-120-ast-extensions.md](../plan/tasks/TASK-120-ast-extensions.md#L12-L21)
- [docs/plan/PLAN-INDEX.md](../plan/PLAN-INDEX.md#L390-L423)
- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L528-L551)

`TASK-120` still frames the retrofit as Phase 17, while the plan index places ADT work in Phase 18. More importantly, `PLAN-INDEX` lists TASK-130 through TASK-136 for the ADT phase, leaving TASK-120 through TASK-129 outside the indexed phase sequence even though SPEC-020’s implementation phases expect that work.

### 2. TASK-120 contradicts TASK-121

- [docs/plan/tasks/TASK-120-ast-extensions.md](../plan/tasks/TASK-120-ast-extensions.md#L12-L21)
- [docs/plan/tasks/TASK-121-adt-core-types.md](../plan/tasks/TASK-121-adt-core-types.md#L20-L27)

`TASK-120` says TASK-121 only extended the `Type` enum and left AST nodes undone. `TASK-121` says it also included `TypeDef` and unification work. Both cannot be true.

### 3. ADT type-definition model drifts from SPEC-020 and across tasks

- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L75-L96)
- [docs/plan/tasks/TASK-120-ast-extensions.md](../plan/tasks/TASK-120-ast-extensions.md#L120-L150)
- [docs/plan/tasks/TASK-124-parse-type-definitions.md](../plan/tasks/TASK-124-parse-type-definitions.md#L258-L292)
- [docs/plan/tasks/TASK-127-type-check-constructors.md](../plan/tasks/TASK-127-type-check-constructors.md#L79-L100)
- [docs/plan/tasks/TASK-129-generic-instantiation.md](../plan/tasks/TASK-129-generic-instantiation.md#L70-L92)

SPEC-020 defines `TypeDef` and `TypeBody` directly over `Type`, but the ADT task chain shifts to a `TypeExpr`-based model. Later tasks depend on variants such as `TypeExpr::Var(...)` that are not defined coherently in the preceding task chain.

### 4. ADT runtime value shape drifts after TASK-122

- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L311-L319)
- [docs/plan/tasks/TASK-122-adt-runtime-values.md](../plan/tasks/TASK-122-adt-runtime-values.md#L136-L144)
- [docs/plan/tasks/TASK-131-constructor-evaluation.md](../plan/tasks/TASK-131-constructor-evaluation.md#L63-L65)
- [docs/plan/tasks/TASK-132-pattern-matching-engine.md](../plan/tasks/TASK-132-pattern-matching-engine.md#L92-L118)
- [docs/plan/tasks/TASK-133-match-evaluation.md](../plan/tasks/TASK-133-match-evaluation.md#L51-L53)

SPEC-020 and TASK-122 use `typ` / `variant` fields. Later interpreter tasks switch to `type_name` / `variant_name`, creating inconsistency inside the same task cluster.

### 5. Exhaustiveness checking is split inconsistently across TASK-128 and TASK-130

- [docs/plan/tasks/TASK-128-type-check-patterns.md](../plan/tasks/TASK-128-type-check-patterns.md#L160-L180)
- [docs/plan/tasks/TASK-130-exhaustiveness-checking.md](../plan/tasks/TASK-130-exhaustiveness-checking.md#L40-L70)
- [docs/plan/PLAN-INDEX.md](../plan/PLAN-INDEX.md#L417-L423)

`TASK-128` already introduces `Coverage` and `check_exhaustive`, while `TASK-130` defines the same work again as a separate task. That weakens the phase/task boundaries in the plan index.

## Medium-Severity Findings

### 6. REPL task docs disagree with each other and reflect known CLI/REPL spec drift

- [docs/plan/tasks/TASK-056-cli-repl.md](../plan/tasks/TASK-056-cli-repl.md#L84-L92)
- [docs/plan/tasks/TASK-080-repl-commands.md](../plan/tasks/TASK-080-repl-commands.md#L1-L16)
- [docs/spec/SPEC-011-REPL.md](../spec/SPEC-011-REPL.md#L72-L90)

`TASK-056` includes commands such as `:effect`, `:parse`, `:dot`, and `:load`, while `TASK-080` focuses on `:help`, `:quit`, `:type`, `:ast`, and `:clear`. The task docs encode the same mismatch already visible between the REPL and CLI specs.

### 7. TASK-080 uses syntax that already drifted from the surface grammar

- [docs/plan/tasks/TASK-080-repl-commands.md](../plan/tasks/TASK-080-repl-commands.md#L76-L81)
- [docs/spec/SPEC-002-SURFACE.md](../spec/SPEC-002-SURFACE.md#L107-L125)

The sample implementation wraps expressions using an inline `action { effect: ...; body: ... }` shape that does not match the canonical workflow grammar in SPEC-002.

### 8. TASK-075 preserves provider-effect drift from the embedding spec

- [docs/plan/tasks/TASK-075-engine-capabilities.md](../plan/tasks/TASK-075-engine-capabilities.md#L11-L24)
- [docs/spec/SPEC-010-EMBEDDING.md](../spec/SPEC-010-EMBEDDING.md#L84-L113)
- [docs/spec/SPEC-003-TYPE-SYSTEM.md](../spec/SPEC-003-TYPE-SYSTEM.md#L49-L109)

The task models provider effects at the provider level rather than per operation, matching the drift already identified between the embedding API and the finer-grained type/effect model.

### 9. TASK-090 adopts unstable `receive` syntax directly from the drifting specs

- [docs/plan/tasks/TASK-090-parse-receive.md](../plan/tasks/TASK-090-parse-receive.md#L11-L21)
- [docs/spec/SPEC-013-STREAMS.md](../spec/SPEC-013-STREAMS.md#L58-L99)

The task explicitly includes parsing `receive control`, so it inherits the same unstable `receive` surface that the spec audit flagged.

### 10. SPEC-018 runtime verification drift is reflected in TASK-115 and TASK-118

- [docs/plan/tasks/TASK-115-obligation-checker.md](../plan/tasks/TASK-115-obligation-checker.md#L83-L98)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L57-L64)
- [docs/plan/tasks/TASK-118-operation-verifier.md](../plan/tasks/TASK-118-operation-verifier.md#L7-L24)
- [docs/spec/SPEC-018-CAPABILITY-MATRIX.md](../spec/SPEC-018-CAPABILITY-MATRIX.md#L239-L260)

`TASK-115` adds critical-versus-optional obligation behavior that is not modeled that way in SPEC-018. `TASK-118` cites the wrong section and changes denial behavior into `OperationResult::Denied` instead of the error-based form shown in SPEC-018.

### 11. Policy task docs carry forward policy-model inconsistencies

- [docs/plan/tasks/TASK-061-policy-definitions.md](../plan/tasks/TASK-061-policy-definitions.md#L11-L75)
- [docs/plan/tasks/TASK-062-policy-combinators.md](../plan/tasks/TASK-062-policy-combinators.md#L13-L91)
- [docs/plan/tasks/TASK-063-dynamic-policies.md](../plan/tasks/TASK-063-dynamic-policies.md#L34-L62)

The policy tasks reinforce the same abstraction drift seen in SPEC-006 through SPEC-008:
- struct-like policy definitions,
- first-class compositional `PolicyExpr` forms,
- runtime-loaded `Policy` values,
without a consistently reconciled model.

## Low-Severity and Systemic Findings

### 12. Completion-status drift versus PLAN-INDEX conventions

- [docs/plan/PLAN-INDEX.md](../plan/PLAN-INDEX.md#L9-L18)
- [docs/plan/tasks/TASK-061-policy-definitions.md](../plan/tasks/TASK-061-policy-definitions.md#L223-L240)
- [docs/plan/tasks/TASK-075-engine-capabilities.md](../plan/tasks/TASK-075-engine-capabilities.md#L76-L84)
- [docs/plan/tasks/TASK-080-repl-commands.md](../plan/tasks/TASK-080-repl-commands.md#L84-L91)
- [docs/plan/tasks/TASK-120-ast-extensions.md](../plan/tasks/TASK-120-ast-extensions.md#L374-L394)

Many tasks are marked complete while their own completion checklists remain unchecked. This appears systemic rather than isolated.

### 13. TASK-135 conflicts with the linked control-link design note

- [docs/plan/tasks/TASK-135-control-link-transfer.md](../plan/tasks/TASK-135-control-link-transfer.md#L15-L28)
- [docs/design/CONTROL_LINK_TRANSFER.md](../design/CONTROL_LINK_TRANSFER.md#L14-L24)

The task models transfer by extracting `link` from `Some`, while the design note models transfer by sending `w_ctrl` itself and leaving it as `None` afterward.

### 14. TASK-136 is narrower than SPEC-020 and assumes import/prelude behavior not defined there

- [docs/plan/tasks/TASK-136-option-result-library.md](../plan/tasks/TASK-136-option-result-library.md#L210-L230)
- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L422-L462)

The task’s library helper set is smaller than the one specified in SPEC-020 and assumes `use` / `pub use`-driven prelude behavior as part of the approach.

## Consistent Task Clusters

### Early core tasks are comparatively stable

Representative aligned tasks:
- [docs/plan/tasks/TASK-001-effect-lattice.md](../plan/tasks/TASK-001-effect-lattice.md#L1-L25)
- [docs/plan/tasks/TASK-003-workflow-ast.md](../plan/tasks/TASK-003-workflow-ast.md#L1-L25)
- [docs/plan/tasks/TASK-004-provenance.md](../plan/tasks/TASK-004-provenance.md#L1-L25)

These remain closely aligned with SPEC-001 and look suitable as lower-risk starting points when reviewing the Rust implementation.

### Module/import task chain is internally coherent

Representative aligned tasks:
- [docs/plan/tasks/TASK-064-module-ast.md](../plan/tasks/TASK-064-module-ast.md#L1-L16)
- [docs/plan/tasks/TASK-067-parse-mod.md](../plan/tasks/TASK-067-parse-mod.md#L1-L18)
- [docs/plan/tasks/TASK-086-import-resolution.md](../plan/tasks/TASK-086-import-resolution.md#L1-L18)

Even though the specs themselves drift about import scope, the task chain for modules and imports is internally structured and understandable.

### TASK-122 is a strong match for the ADT runtime value model

- [docs/plan/tasks/TASK-122-adt-runtime-values.md](../plan/tasks/TASK-122-adt-runtime-values.md#L136-L144)
- [docs/spec/SPEC-020-ADT-TYPES.md](../spec/SPEC-020-ADT-TYPES.md#L311-L319)

This task is one of the cleanest ADT tasks relative to the spec.

## Implications for Rust Code Review

For the upcoming Rust codebase review, the highest-risk clusters are:
1. policy tasks TASK-061 through TASK-063,
2. REPL tasks TASK-056 and TASK-080,
3. stream and runtime-verification tasks TASK-090, TASK-115, and TASK-118,
4. ADT tasks TASK-120 through TASK-136.

A practical review order would be:
- start with early core tasks as baseline anchors,
- then review module/import work,
- then concentrate effort on the four drift-heavy clusters above.

This ordering should reduce time spent untangling documentation drift while checking the Rust implementation for real spec/task divergence.

## Conclusion

The non-Lean task set is not uniformly reliable as implementation guidance. Some clusters are solid, but several later task groups preserve known spec inconsistencies or introduce new contradictions between tasks, the plan index, and referenced design notes.

No task files were modified as part of this audit.
