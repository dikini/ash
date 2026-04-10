# PLAN-023: Pure Functions Phase

## Status: Draft

## Overview

Implement the `fn` construct and three-vertex model as defined in DESIGN-020, SPEC-027, and
SPEC-028. This phase makes the std/ pure library modules parseable and correct.

## Prerequisite

Phase 74 (Stdlib IO V1) should be complete or at least its capability surface frozen, so that
fn and IO modules can be developed without conflicts.

## Design References

- [DESIGN-020: Pure Functions and the Three-Vertex Model](../design/DESIGN-020-PURE-FUNCTIONS-THREE-VERTEX-MODEL.md)
- [SPEC-027: Pure Functions](../spec/SPEC-027-PURE-FUNCTIONS.md)
- [SPEC-028: Function Constraint System](../spec/SPEC-028-FUNCTION-CONSTRAINT-SYSTEM.md)

## Critical Issue #1: Top-Level Parser/Module Model Reconciliation

**Problem:** The current parser root (`Program { definitions, workflow }` in surface.rs) requires exactly one workflow, but the module grammar (`program ::= module_item*` in SPEC-009) allows any collection of items. With fn definitions, modules become primary units where workflows are optional.

**Resolution:** This phase must reconcile these models with a single authoritative file/root split:
1. **`ModuleFile` is the authoritative file-level parse result** for any source file: top-level definitions, module declarations, and an optional workflow.
2. **`Program` remains the entry-point model only** and is produced only when entry-point loading/validation is requested for a `ModuleFile` that has the required workflow shape.

A file is parsed once under one selected entry path; it is not simultaneously both roots. This is not merely adding a variant; it changes how files are parsed and what constitutes a valid top-level.

## Prerequisite Clarifications

Before implementation begins, the following prerequisite drafts must cover the core parser, surface syntax, type-system, and operational-semantics changes that unblock implementation. They do not by themselves complete the later documentation work for SPEC-012/SPEC-022 or any separate design-ratification follow-up called out elsewhere in this plan.

| Task | Description | Spec |
|------|-------------|------|
| TASK-P0 | Draft SPEC-009 module grammar update: add `fn_def` to `definition` production; repair the `module_item*`/workflow cardinality contradiction; clarify file-as-module vs file-as-entry-point | SPEC-009 |
| TASK-P0a | **NEW:** Draft parser model update: make `ModuleFile` the authoritative file AST (definitions, module declarations, optional workflow) and reserve `Program` for entry-point validation/loading | surface.rs |
| TASK-P1 | Draft SPEC-002 surface syntax groundwork: add `fn` and planned `panic` keyword syntax, fn/module top-level file grammar, and match/if-as-expr grammar needed by this phase | SPEC-002 |
| TASK-P2 | Draft SPEC-003 type system groundwork: add fn typing judgment, FnType, purity judgment, effect-neutral fn calls | SPEC-003 |
| TASK-P3 | Draft SPEC-004 semantics groundwork: add fn evaluation rules (LET, TAIL, MATCH, IF, CALL, PANIC) | SPEC-004 |

Panic syntax handoff note: for PLAN-023 planning and implementation sequencing, this phase follows SPEC-027 and treats `panic` as keyword syntax. DESIGN-020 previously left that as an open question; for this phase that design decision is considered resolved/ratified and any remaining design-document cleanup is tracked later rather than blocking this plan.

## Task Breakdown

### Track 0: Prerequisites (Spec Clarifications)

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-P0 | Update SPEC-009 grammar: add `fn_def` to `definition` production and repair the top-level workflow/module-item cardinality contradiction | SPEC-009 | 1 | None |
| TASK-P0a | **PARSER MODEL:** Draft parser model update: make `ModuleFile` the authoritative file AST (definitions, module declarations, optional workflow) and reserve `Program` for entry-point validation/loading | surface.rs | 2 | P0 |
| TASK-P1 | Update SPEC-002: lexical grammar (fn keyword, `panic` keyword syntax per SPEC-027 handoff), top-level file/module grammar, and expression grammar (match, if-as-expr, fn call) | SPEC-002 | 2 | None |
| TASK-P2 | Update SPEC-003: fn typing judgment, FnType, generic fn instantiation, purity judgment, fn call in workflow typing | SPEC-003 | 3 | P0, P0a, P1 |
| TASK-P3 | Update SPEC-004: fn evaluation rules, no effect tracking, fn panic propagation | SPEC-004 | 2 | P0, P1 |

### Track 1: AST and Parser Foundation

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-A0 | **Parser Model Reconciliation:** Define `FileUnit`/entry-point APIs so generic file parsing yields `ModuleFile`, while entry-point loading yields `Program` only after validation | SPEC-009, P0a | 4 | P0a |
| TASK-A0a | Add `ModuleFile` struct: collection of definitions, module declarations, optional workflow, no mandatory main workflow | surface.rs | 2 | A0 |
| TASK-A0b | Implement the selection rule in parser/loading code so files are parsed as `ModuleFile`, then upgraded to entry-point `Program` only on the entry-point path | surface.rs | 3 | A0a |
| TASK-A0c | **NEW:** Implement entry-point promotion/validation against SPEC-022 main-workflow rules before producing `Program` from `ModuleFile` | SPEC-022 | 2 | A0b |
| TASK-A1 | Add `Fn` token kind and `"fn"` to lexer keyword table (token.rs, lexer.rs) | SPEC-027 §2 | 1 | None |
| TASK-A2 | Add `Panic` token kind and `"panic"` to lexer keyword table | SPEC-027 §2.5 | 1 | None |
| TASK-A2a | **NEW:** Add `Match` token kind and `"match"` to lexer keyword table | SPEC-027 §2.3 | 1 | None |
| TASK-A3 | Add `FnDef` struct to surface.rs and `Definition::Function(FnDef)` variant | SPEC-027 §5 | 2 | A1 |
| TASK-A4 | Add `fn` to `parse_definitions` dispatch in parse_module.rs (keyword-based arm) for `ModuleFile` parsing; entry-point `Program` loading inherits this through `ModuleFile` validation/loading | SPEC-009, SPEC-027 §5 | 3 | A3, A0b |
| TASK-A5a | **NEW:** Add function-type syntax to Type enum using the frozen SPEC-003 shape: `Type::Fn(Vec<Type>, Box<Type>)` | SPEC-027 §3.1 | 2 | A3 |
| TASK-A5b | **NEW:** Implement `parse_fn_type` for `Fn(T, U) -> V` syntax in type annotations | SPEC-027 §3.1 | 2 | A5a |
| TASK-A5 | Implement `parse_fn_def`: visibility, name, type params, params, return type, contract, body | SPEC-027 §2.1 | 4 | A4, A5b |
| TASK-A5c | **NEW:** Normalize fn contract clause surface grammar/AST shape: decide and implement the canonical mapping for repeated vs comma-separated `requires`/`ensures` predicates before lowering | SPEC-002, SPEC-028 | 2 | A5 |
| TASK-A6 | Add `Expr::If` variant to surface.rs (general if-as-expression, distinct from Workflow::If and Expr::IfLet) | SPEC-027 §2.4 | 2 | None |
| TASK-A6a | **NEW:** Add `Expr::Panic` variant for panic expressions | SPEC-027 §2.5 | 1 | A2 |
| TASK-A6b | **NEW:** Add `Expr::Block` variant for block expressions in fn bodies | SPEC-027 §2.2 | 2 | None |
| TASK-A7 | Implement `parse_if_expr` for fn bodies (value-producing branches with type agreement and block-body support) | SPEC-027 §2.4 | 3 | A6, A6b |
| TASK-A8 | Implement `parse_match_expr` for fn body context (match expression producing values, including block-arm bodies) | SPEC-027 §2.3 | 4 | A2a, A6b |
| TASK-A9 | Implement `parse_panic_expr` (requires Expr::Panic AST node) | SPEC-027 §2.5 | 1 | A6a |
| TASK-A10 | Implement fn body parser: sequence of let-bindings + tail expression dispatching to Expr nodes; uses Expr::Block | SPEC-027 §2.2 | 4 | A7, A8, A9, A6b |
| TASK-A11 | Extend Expr::Call to carry an optional module qualifier (for `module::name(args)` resolution) | SPEC-027 §2.6 | 3 | None |

### Track 2: Name Resolution and Module Integration

**Critical Issue #4:** Function name resolution/import/export is under-specified. The plan must explicitly cover resolver/name-binding work for local fn names, imported fn names, `module::name(...)` syntax, and disambiguation between fn calls and capability dispatch.

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-B1 | Extend name resolver to handle fn definitions as module-level bindings; add `Res::Fn` to resolution result enum | SPEC-009, SPEC-027 §5 | 3 | A4 |
| TASK-B1a | **NEW:** Implement fn name binding in module scopes: local fn definitions are visible throughout the module | SPEC-009 | 2 | B1 |
| TASK-B1b | **NEW:** Implement fn import/export: `use path::fn_name` resolves to fn definitions; `pub fn` exports fn to other modules | SPEC-012 | 3 | B1 |
| TASK-B2 | Implement qualified fn call resolution: `module::name(args)` resolves to fn if target is fn definition, type error if capability | SPEC-027 §2.6, SPEC-028 §3.1 | 4 | B1b, A11 |
| TASK-B3 | Distinguish fn calls from capability calls at call sites: `name(args)` resolves to fn (including imported fn names in scope), `provider:action(args)` is always capability (workflow context only) | SPEC-027 §2.6, SPEC-028 §3.1 | 3 | B1, B1b |
| TASK-B3a | **NEW:** Implement wrong-target call diagnostics: calling a capability with function-call syntax or using capability-only syntax where a fn is required produces clear errors/hints | SPEC-027 §2.6 | 2 | B3 |
| TASK-B4 | Ensure fn definitions are properly exported/imported via `pub use` and `use` (existing import machinery extended for Definition::Function) | SPEC-009, SPEC-012 | 2 | B1b |

### Track 3: Type System and Contract Semantics

**Critical Issue #5:** Contract lowering is a major dependency. Surface contracts use `Requirement::Arithmetic { expr: Expr }` but core contracts use structured `{ var, constraint }`. The lowering boundary and required tasks must be explicit.

**Lowering Boundary:**
- **Surface (parser):** `FnDef.contract.requires` contains `Requirement::Arithmetic { expr: Expr }` (raw AST expression)
- **Lowering pass:** Extracts variable name and `ArithConstraint` from the expression
- **Core (type checker/verifier):** `Requirement::Arithmetic { var: String, constraint: ArithConstraint }` (structured)

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-C1 | Define FnType in the type system (pure function type, no effect slot, distinct from Type::Fun) | SPEC-027 §3.2 | 3 | Track 2, A5a |
| TASK-C2 | Implement fn return type inference (params explicit, return type inferred from body if omitted) | SPEC-027 §3.3 | 3 | C1 |
| TASK-C3 | Implement generic fn type instantiation at call sites (unification with argument types) | SPEC-027 §3.4 | 4 | C2 |
| TASK-C4 | Implement purity checking pass: reject Expr::Policy, Expr::CheckObligation in fn bodies; reject workflow keywords (ret, act, receive, send, spawn, etc.); resolve Expr::Call and Expr::InterfaceMethodCall callee purity | SPEC-027 §3.5 | 4 | Track 2 |
| TASK-C5 | Implement match exhaustiveness checking for fn match expressions | SPEC-027 §3.7 | 3 | A8 |
| TASK-C6 | Implement branch type agreement for Expr::If (both branches must produce same type) | SPEC-027 §2.4 | 2 | A7 |
| TASK-C6a | **NEW:** Implement one-armed `if` typing/evaluation contract: omitted-else form has `Type::Null`, implicit `null` else path, and taken then-branch must also type-check as `Type::Null` | SPEC-027 §2.4, §4.1 | 2 | A7 |
| TASK-C7 | fn contract validation: reject HasCapability/HasRole in fn requires clauses | SPEC-028 §3 | 2 | A5 |
| TASK-C7b | **NEW:** fn postcondition validation: allow only value-level ensures predicates; reject StateAssertion and non-fn contract forms in `ensures` clauses | SPEC-028 §4.2, §8 | 2 | A5 |
| TASK-C7a | **NEW:** Define precondition lowering interface: `lower_contract(surface_contract) -> core_contract` with explicit error cases | SPEC-028 §4 | 2 | C7 |
| TASK-C7c | **NEW:** Define postcondition lowering/evaluation interface for surface `ensures` clauses to runtime/core postcondition checks | SPEC-028 §4.2, §8 | 2 | C7b, A5c |
| TASK-C8 | Build surface Requirement::Arithmetic → core Requirement::Arithmetic lowering pass (extract var name and ArithConstraint from raw Expr for Stage 1 arithmetic predicates) | SPEC-028 §4, §10 | 6 | C7a, C9, C9a |
| TASK-C8a | **NEW:** Add lowering acceptance tests: verify `n >= 0` lowers to `{ var: "n", constraint: Gte(0) }`; verify `n != 0` and modulo predicates lower correctly | SPEC-028 §4, §10 | 2 | C8, C9, C9a |
| TASK-C9 | Add NotEq variant to ArithConstraint in ash-core | SPEC-028 §10 | 1 | None |
| TASK-C9a | **NEW:** Add Modulo variant to ArithConstraint in ash-core for Stage 1 fn constraints | SPEC-028 §10 | 1 | None |
| TASK-C10 | Define fn panic propagation in workflow context: fn panic becomes workflow CompletionPayload Err(RuntimeFailure) | SPEC-027 §4.3 | 2 | C4 |

### Track 4: Semantic Validation and Integration

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-D1 | Fn evaluation rules: implement fn call in the interpreter/evaluator (no effect, no trace, no provenance) | SPEC-027 §4.1 | 4 | C2, C4 |
| TASK-D1a | **NEW:** Implement fn `ensures` runtime checking at return; failing ensures raises runtime failure | SPEC-028 §8.1 | 2 | C7b, C7c, D1 |
| TASK-D2 | fn call effect neutrality in workflow composition: fn calls in workflow bodies stay effect-neutral and do not introduce a new workflow grade | SPEC-027 §3.2 | 2 | C4 |
| TASK-D3 | fn precondition propagation: workflow callers inherit fn requires as call-site checks | SPEC-028 §9 | 3 | C3, C8, D2 |

### Track 5: std/ Library Rewrite

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-E1 | Rewrite std/src/option.ash using fn syntax (tail-expression return, no ret) | DESIGN-020 D9 | 2 | Track 1, C4 |
| TASK-E2 | Rewrite std/src/result.ash using fn syntax | DESIGN-020 D9 | 2 | Track 1, C4 |
| TASK-E3 | Rewrite std/src/io/path.ash pure functions using fn syntax | DESIGN-020 D9 | 2 | Track 1, C4 |
| TASK-E4 | Rewrite stdlib_parsing.rs tests: replace string-matching with actual parser validation for fn definitions, match expressions, if-as-expr, panic | SPEC-027 | 4 | E1, E2, E3 |
| TASK-E5 | **NEW:** Add conformance tests for milestone/failure modes: imported fn calls, qualified fn success/failure, one-armed `if` null typing, recursion/panic boundary, undefined fn, and wrong `provider::action()`/`provider:::` diagnostics | SPEC-027, SPEC-028 | 4 | Track 2, Track 3, Track 4 |

### Track 6: Documentation Finalization

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-F1 | Finalize SPEC-002, SPEC-003, SPEC-004, SPEC-009, SPEC-012, SPEC-022, SPEC-027, and SPEC-028 updates required by this phase | All | 2 | All |
| TASK-F2 | Update docs/spec/README.md if needed | - | 0.5 | F1 |
| TASK-F3 | Update CHANGELOG.md | - | 1 | All |
| TASK-F4 | Final verification: cargo test, cargo clippy, cargo fmt --check, zero warnings | - | 2 | All |

## Milestone Definition

**Critical Issue #6:** Milestone/acceptance criteria must require verification for parsing `Fn(...) -> ...`, qualified fn call resolution success/failure, fn/capability ambiguity rejection, lowering contract expressions, and recursion/panic boundary behavior.

**Phase complete when:**

### Parser/AST (Track 1)
1. `fn` is a recognized keyword in the lexer
2. `fn` definitions parse as `Definition::Function(FnDef)` at module level
3. **AC:** Parser accepts `fn foo(n: Int) -> Int { n + 1 }` and produces correct AST
4. **AC:** Parser accepts `Fn(Int) -> Int` type syntax in parameter and return type positions
5. **AC:** Parser recognizes `match` as fn-body syntax through explicit lexer/token support and produces the correct match-expression AST
6. **AC:** Parser rejects `fn` inside fn bodies (fn cannot be nested)
7. fn bodies parse match expressions, if-as-expressions, and panic as Expr nodes
8. **AC:** `Expr::If`, `Expr::Panic`, `Expr::Block` AST variants exist and are produced by parser
9. **AC:** Entry-point promotion from `ModuleFile` to `Program` enforces SPEC-022 main-workflow validity rules

### Name Resolution (Track 2)
10. `module::name(args)` resolves to fn when target is a fn definition
11. **AC:** `module::name(args)` produces type error when target is a capability (not a fn)
12. **AC:** Test: `use path::fn_name` imports fn; calling imported fn works
13. **AC:** Test: `pub fn` exports fn; importing module can call it
14. **AC:** Wrong-target call diagnostics are clear: calling a capability with function-call syntax or using capability-only syntax where a fn is required produces actionable errors/hints

### Type System (Track 3)
15. fn bodies use tail-expression return; `ret` is rejected in fn bodies
16. Purity checking rejects Expr::Policy, Expr::CheckObligation, and all workflow keywords in fn bodies
17. **AC:** Test purity violation detection: `fn bad() { act io:println("hello") }` is rejected
18. fn calls in workflow bodies are effect-neutral and do not introduce a new workflow grade
19. **AC:** Type checker correctly classifies `Fn(Int) -> Int` as distinct from effectful function types
20. **AC:** One-armed `if` in fn bodies is accepted only when the taken branch also has `Type::Null`; implicit else path is `null`

### Contract Lowering (Track 3 - Critical Issue #5)
21. **AC:** Contract lowering tests pass: `requires: n >= 0` lowers to `{ var: "n", constraint: Gte(0) }`
22. **AC:** Stage 1 arithmetic lowering handles `n != 0` and modulo predicates correctly
23. **AC:** Repeated and comma-separated `requires`/`ensures` clause forms normalize to the same canonical contract AST/lowering path
24. **AC:** Contract lowering rejects non-arithmetic constraints in fn contracts: `requires: HasCapability(Fs)` is rejected
25. **AC:** `ensures` clauses reject StateAssertion and other non-value fn postcondition forms
26. **AC:** Stage 1 arithmetic vocabulary includes both `NotEq` and `Modulo`

### Semantics (Track 4)
26. fn panic in workflow context produces RuntimeFailure CompletionPayload
27. **AC:** Workflow-side fn preconditions must be provable at call sites from the current typing context; unprovable preconditions reject the workflow call
28. **AC:** Test recursion: `fn factorial(n: Int) -> Int { if n <= 1 then 1 else n * factorial(n - 1) }` evaluates correctly
29. **AC:** Test panic boundary: fn that panics propagates panic to calling workflow
30. **AC:** `ensures` clauses are checked at fn return and failing ensures raise runtime failure

### std/ Library (Track 5)
31. All fn definitions in std/src/option.ash, std/src/result.ash, std/src/io/path.ash parse successfully
32. stdlib_parsing.rs tests validate constructs through the parser (not string-matching)
33. **AC:** Imported fn call tests, qualified call success/failure tests, workflow precondition-call-site proof tests, undefined-fn diagnostics, wrong-target call diagnostics, and wrong `provider::action()`/`provider:::` diagnostics are automated and passing

### Documentation (Track 6)
34. SPEC-002, SPEC-003, SPEC-004, SPEC-009, SPEC-012, SPEC-022, SPEC-027, and SPEC-028 updated with fn language and workflow/import/contract rules
35. Zero clippy warnings, zero compilation warnings, all tests pass
36. CHANGELOG.md updated

**Failure Mode Tests Required:**
- Parsing `Fn(` with missing closing paren produces helpful error
- Calling undefined fn `foo()` produces "unknown function" error
- One-armed `if` with non-`null` then branch is rejected with helpful `Type::Null` diagnostic
- Calling capability with `::` syntax (`provider::action()`) produces "use : for capabilities" error
- Calling a capability with function-call syntax or using capability-only syntax where a fn is required produces a clear wrong-target diagnostic

**Not in scope for this phase:**
- Workflow implementing capability (`implements Cap` clause) -- future phase
- Proxy collapse into workflow -- future phase
- String constraints (SPEC-028 Stage 2) -- future phase
- Z3 compile-time proving for fn contracts (SPEC-028 Stage 3) -- future phase
- Dependent constraints (SPEC-028 Stage 4) -- future phase
- IO capability syntax cleanup (multi-action capabilities, colon separator fixes) -- separate phase

## Estimated Total Effort

**Critical Issue #7:** Task granularity was too optimistic. Parser/resolver/type tasks have been split to match real implementation order.

147.5 hours across 61 tasks across 7 tracks (Tracks 0-6; revised from 95 hours / 30 tasks).

Revised estimates account for:
- Parser model reconciliation (A0-A0b): +9 hours
- Function type syntax (A5a-A5b): +4 hours  
- AST variants for fn expressions (A6a-A6b): +3 hours
- Fn name binding/import/export (B1a-B1b): +5 hours
- Fn/capability ambiguity (B3a): +2 hours
- Contract lowering interface and tests (C7a, C8a): +4 hours
- Contract lowering complexity (C8): +2 hours (was 4, now 6)

## Dependency Graph

```
Track 0 (Prerequisites):
  P0 → P0a
  P0, P0a, P1 → P2
  P0, P1 → P3

Track 1 (AST):
  P0a → A0 → A0a → A0b → A0c
  A1 → A3 → A5a → A5b
  A0b, A3 → A4 → A5 → A5c
  A2 → A6a → A9
  A2a → A8
  A6, A6b → A7
  A2a, A6b → A8
  A7, A8, A9, A6b → A10
  A11 independent

Track 2 (Names):
  A4 → B1 → B1a → B1b → B4
  B1, B1b → B3 → B3a
  B1b, A11 → B2

Track 3 (Types):
  Track 2, A5a → C1 → C2 → C3
  Track 2 → C4 → C10
  A8 → C5
  A7 → C6 → C6a
  A5 → C7 → C7a
  A5 → C7b → C7c
  C7a, C9, C9a → C8 → C8a
  C9, C9a independent

Track 4 (Semantics):
  C2, C4 → D1 → D1a
  C4 → D2
  C3, C8, D2 → D3

Track 5 (std/):
  Track 1, C4 → E1, E2, E3 → E4
  Track 2, Track 3, Track 4 → E5

Track 6 (Docs):
  All → F1 → F2
  All → F3
  All → F4
```

**Deep branch of the dependency graph:** P0 → P0a → A0 → A0a → A0b → A4 → A5 → C7 → C7a → C8 → D3.

**Dependency note:** This line highlights the contract-lowering branch into D3 only. D3 still also depends on D2 (as shown in the Track 4 table/graph), so D3 does not bypass the separate C4 → D2 path.

**Final verification milestone:** F4 remains phase-wide and `All`-gated, so it follows completion of every required track rather than a direct D3 → F4 edge.

**Note:** D1 (fn evaluation rules) still runs in parallel to the contract-lowering / precondition-propagation branch and is not on the dependency branch shown above.

**Key Dependency Changes:**
- Parser model (A0-A0b) is now foundational; all file-parsing depends on it
- Function type (A5a) is required before type checking (C1)
- Fn import/export (B1b) is required before qualified call resolution (B2)
- Contract lowering interface (C7a) is a prerequisite for lowering implementation (C8)

## Normative Delta vs Existing Specs

The following changes to existing specs are required (enumerated per the review recommendation):

### SPEC-002 (Surface Language)
- Add `fn` to lexical keyword table
- Add `panic` to lexical keyword table
- Add `match` to lexical keyword table
- Add `fn_def` to module-level definition alternatives
- Add fn-body block grammar with required tail expression
- Add top-level file/module grammar aligned with `ModuleFile` and entry-point `Program`
- Add match expression grammar to expression section
- Add if-as-expression grammar (distinct from workflow if_stmt)
- Add omitted-else `if` / `null` typing surface note
- Add panic expression grammar
- Add fn call syntax: `name(args)`, `module::name(args)`
- Note: `::` for qualified fn calls, `:` for capability calls only
### SPEC-003 (Type System)
- Add fn typing judgment: Γ ⊢ fn f(x:τ) { body } : FnType
- Add FnType: `(τ*) -> τ` (no effect slot)
- Add purity judgment over Expr: Γ ⊢ e : pure | impure | resolve
- Add generic fn instantiation rule
- Add fn call in workflow context rule (effect-neutral)
- Add omitted-else `if` rule with `Type::Null`
- Add `Expr::InterfaceMethodCall` purity-resolution rule in fn bodies
- Add recursion is allowed (no termination constraint)

### SPEC-004 (Operational Semantics)
- Add fn evaluation rules: LET, TAIL, MATCH, IF-TRUE, IF-FALSE, CALL, PANIC
- Add one-armed `if` semantics with implicit `null` else branch
- Define FnResult ::= Value | Panic (no Effect/Trace/Provenance/Obligations)
- Define fn panic propagation in workflow context

### SPEC-009 (Module System)
- Add `fn_def` to `definition` production rule
- Make `ModuleFile` include module declarations plus optional workflow
- Clarify `ModuleFile` vs entry-point `Program` split
- Add `pub fn` visibility/export rules
- Add `use` import rules for fn definitions
- Add `module::name` resolution in expression context

### SPEC-012 (Imports)
- **Update required:** Document `use path::fn_name` for importing functions
- **Update required:** Document `pub fn` export visibility for functions
- **Update required:** Clarify that `use` can import any definition kind including fn
- Note: The resolver changes (TASK-B1b, B4) implement the binding behavior; SPEC-012 documents the user-facing rules

### SPEC-022 (Workflow Typing)
- Add fn contract subset rule: fn contracts exclude HasCapability, HasRole, obligations
- Add fn precondition propagation at workflow call sites

### SPEC-028 (Function Constraint System)
- Add explicit fn postcondition/`ensures` validation rules
- Add runtime `ensures` checking requirements at fn return
- Add Stage 1 `Modulo` alongside `NotEq` in `ArithConstraint`
