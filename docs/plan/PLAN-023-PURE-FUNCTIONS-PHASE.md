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

**Resolution:** This phase must reconcile these models. The parser will support both:
1. **Main-workflow mode** (current): One workflow + definitions, for entry points
2. **Module mode** (new): Zero or more definitions, for library modules

This is not merely adding a variant; it changes how files are parsed and what constitutes a valid top-level.

## Prerequisite Clarifications

Before implementation begins, the following spec changes must be drafted. These resolve ambiguities that the implementation depends on:

| Task | Description | Spec |
|------|-------------|------|
| TASK-P0 | Draft SPEC-009 module grammar update: add `fn_def` to `definition` production; clarify file-as-module vs file-as-entry-point | SPEC-009 |
| TASK-P0a | **NEW:** Draft parser model update: define `ModuleFile` AST (definition collection, optional workflow) alongside `Program` | surface.rs |
| TASK-P1 | Draft SPEC-002 surface syntax update: add `fn` keyword, match/if-as-expr, panic to lexical and expression grammar | SPEC-002 |
| TASK-P2 | Draft SPEC-003 type system update: add fn typing judgment, FnType, purity judgment, effect-neutral fn calls | SPEC-003 |
| TASK-P3 | Draft SPEC-004 semantics update: add fn evaluation rules (LET, TAIL, MATCH, IF, CALL, PANIC) | SPEC-004 |

## Task Breakdown

### Track 0: Prerequisites (Spec Clarifications)

|| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
|| TASK-P0 | Update SPEC-009 grammar: add `fn_def` to `definition` production | SPEC-009 | 1 | None |
|| TASK-P0a | **PARSER MODEL:** Draft parser model update: define `ModuleFile` AST (definition collection, optional workflow) alongside `Program` | surface.rs | 2 | P0 |
|| TASK-P1 | Update SPEC-002: lexical grammar (fn keyword, panic keyword), expression grammar (match, if-as-expr, fn call) | SPEC-002 | 2 | None |
|| TASK-P2 | Update SPEC-003: fn typing judgment, FnType, generic fn instantiation, purity judgment, fn call in workflow typing | SPEC-003 | 3 | P0, P0a, P1 |
|| TASK-P3 | Update SPEC-004: fn evaluation rules, no effect tracking, fn panic propagation | SPEC-004 | 2 | P0, P1 |

### Track 1: AST and Parser Foundation

|| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
|| TASK-A0 | **Parser Model Reconciliation:** Define `FileUnit` enum with `Main(Program)` and `Module(ModuleFile)` variants; update parse entry points | SPEC-009, P0a | 4 | None |
|| TASK-A0a | Add `ModuleFile` struct: collection of definitions, optional workflow, no mandatory main workflow | surface.rs | 2 | A0 |
|| TASK-A0b | Implement mode detection or unified grammar: parser can handle both entry-point files and library modules | surface.rs | 3 | A0a |
|| TASK-A1 | Add `Fn` token kind and `"fn"` to lexer keyword table (token.rs, lexer.rs) | SPEC-027 §2 | 1 | None |
|| TASK-A2 | Add `Panic` token kind and `"panic"` to lexer keyword table | SPEC-027 §2.5 | 1 | None |
|| TASK-A3 | Add `FnDef` struct to surface.rs and `Definition::Function(FnDef)` variant | SPEC-027 §5 | 2 | A1 |
|| TASK-A4 | Add `fn` to `parse_definitions` dispatch in parse_module.rs (keyword-based arm); ensure fn works in both Program and ModuleFile contexts | SPEC-009, SPEC-027 §5 | 3 | A3, A0b |
|| TASK-A5 | Implement `parse_fn_def`: visibility, name, type params, params, return type, contract, body | SPEC-027 §2.1 | 4 | A4 |
|| TASK-A5a | **NEW:** Add function-type syntax to Type enum: `Type::Fn { params: Vec<Type>, return_type: Box<Type> }` | SPEC-027 §3.1 | 2 | A3 |
|| TASK-A5b | **NEW:** Implement `parse_fn_type` for `Fn(T, U) -> V` syntax in type annotations | SPEC-027 §3.1 | 2 | A5a |
|| TASK-A6 | Add `Expr::If` variant to surface.rs (general if-as-expression, distinct from Workflow::If and Expr::IfLet) | SPEC-027 §2.4 | 2 | None |
|| TASK-A6a | **NEW:** Add `Expr::Panic` variant for panic expressions | SPEC-027 §2.5 | 1 | A2 |
|| TASK-A6b | **NEW:** Add `Expr::Block` variant for block expressions in fn bodies | SPEC-027 §2.2 | 2 | None |
|| TASK-A7 | Implement `parse_if_expr` for fn bodies (value-producing branches with type agreement) | SPEC-027 §2.4 | 3 | A6 |
|| TASK-A8 | Implement `parse_match_expr` for fn body context (match expression producing values) | SPEC-027 §2.3 | 4 | None |
|| TASK-A9 | Implement `parse_panic_expr` (requires Expr::Panic AST node) | SPEC-027 §2.5 | 1 | A6a |
|| TASK-A10 | Implement fn body parser: sequence of let-bindings + tail expression dispatching to Expr nodes; uses Expr::Block | SPEC-027 §2.2 | 4 | A7, A8, A9, A6b |
|| TASK-A11 | Extend Expr::Call to carry an optional module qualifier (for `module::name(args)` resolution) | SPEC-027 §2.6 | 3 | None |

### Track 2: Name Resolution and Module Integration

**Critical Issue #4:** Function name resolution/import/export is under-specified. The plan must explicitly cover resolver/name-binding work for local fn names, imported fn names, `module::name(...)` syntax, and disambiguation between fn calls and capability dispatch.

|| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
|| TASK-B1 | Extend name resolver to handle fn definitions as module-level bindings; add `Res::Fn` to resolution result enum | SPEC-009, SPEC-027 §5 | 3 | A4 |
|| TASK-B1a | **NEW:** Implement fn name binding in module scopes: local fn definitions are visible throughout the module | SPEC-009 | 2 | B1 |
|| TASK-B1b | **NEW:** Implement fn import/export: `use path::fn_name` resolves to fn definitions; `pub fn` exports fn to other modules | SPEC-012 | 3 | B1 |
|| TASK-B2 | Implement qualified fn call resolution: `module::name(args)` resolves to fn if target is fn definition, type error if capability | SPEC-027 §2.6, SPEC-028 §3.1 | 4 | B1b, A11 |
|| TASK-B3 | Distinguish fn calls from capability calls at call sites: `name(args)` resolves to fn, `provider:action(args)` is always capability (workflow context only) | SPEC-027 §2.6, SPEC-028 §3.1 | 3 | B1 |
|| TASK-B3a | **NEW:** Implement fn/capability ambiguity detection and error reporting: when `name` could be either, use context (call syntax) to decide; error if ambiguous | SPEC-027 §2.6 | 2 | B3 |
|| TASK-B4 | Ensure fn definitions are properly exported/imported via `pub use` and `use` (existing import machinery extended for Definition::Function) | SPEC-009, SPEC-012 | 2 | B1b |

### Track 3: Type System and Contract Semantics

**Critical Issue #5:** Contract lowering is a major dependency. Surface contracts use `Requirement::Arithmetic { expr: Expr }` but core contracts use structured `{ var, constraint }`. The lowering boundary and required tasks must be explicit.

**Lowering Boundary:**
- **Surface (parser):** `FnDef.contract.requires` contains `Requirement::Arithmetic { expr: Expr }` (raw AST expression)
- **Lowering pass:** Extracts variable name and `ArithConstraint` from the expression
- **Core (type checker/verifier):** `Requirement::Arithmetic { var: String, constraint: ArithConstraint }` (structured)

|| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
|| TASK-C1 | Define FnType in the type system (pure function type, no effect slot, distinct from Type::Fun) | SPEC-027 §3.2 | 3 | Track 2, A5a |
|| TASK-C2 | Implement fn return type inference (params explicit, return type inferred from body if omitted) | SPEC-027 §3.3 | 3 | C1 |
|| TASK-C3 | Implement generic fn type instantiation at call sites (unification with argument types) | SPEC-027 §3.4 | 4 | C2 |
|| TASK-C4 | Implement purity checking pass: reject Expr::Policy, Expr::CheckObligation in fn bodies; reject workflow keywords (ret, act, receive, send, spawn, etc.); resolve Expr::Call callee purity | SPEC-027 §3.5 | 4 | Track 2 |
|| TASK-C5 | Implement match exhaustiveness checking for fn match expressions | SPEC-027 §3.7 | 3 | A8 |
|| TASK-C6 | Implement branch type agreement for Expr::If (both branches must produce same type) | SPEC-027 §2.4 | 2 | A7 |
|| TASK-C7 | fn contract validation: reject HasCapability/HasRole in fn requires clauses | SPEC-028 §3 | 2 | A5 |
|| TASK-C7a | **NEW:** Define contract lowering interface: `lower_contract(surface_contract) -> core_contract` with explicit error cases | SPEC-028 §4 | 2 | C7 |
|| TASK-C8 | Build surface Requirement::Arithmetic → core Requirement::Arithmetic lowering pass (extract var name and ArithConstraint from raw Expr); handles compound predicates (&&, \|\|) | SPEC-028 §4 | 6 | C7a |
|| TASK-C8a | **NEW:** Add lowering acceptance tests: verify `n >= 0` lowers to `{ var: "n", constraint: Gte(0) }`; verify `n != 0` lowers correctly | SPEC-028 §4, §10 | 2 | C8 |
|| TASK-C9 | Add NotEq variant to ArithConstraint in ash-core | SPEC-028 §10 | 1 | None |
|| TASK-C10 | Define fn panic propagation in workflow context: fn panic becomes workflow CompletionPayload Err(RuntimeFailure) | SPEC-027 §4.3 | 2 | C4 |

### Track 4: Semantic Validation and Integration

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-D1 | Fn evaluation rules: implement fn call in the interpreter/evaluator (no effect, no trace, no provenance) | SPEC-027 §4.1 | 4 | C2, C4 |
| TASK-D2 | fn call effect neutrality in workflow composition: fn calls in workflow bodies are classified as Epistemic | SPEC-027 §3.2 | 2 | C4 |
| TASK-D3 | fn precondition propagation: workflow callers inherit fn requires as call-site checks | SPEC-028 §9 | 3 | C8, D2 |

### Track 5: std/ Library Rewrite

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-E1 | Rewrite std/src/option.ash using fn syntax (tail-expression return, no ret) | DESIGN-020 D9 | 2 | Track 1, C4 |
| TASK-E2 | Rewrite std/src/result.ash using fn syntax | DESIGN-020 D9 | 2 | Track 1, C4 |
| TASK-E3 | Rewrite std/src/io/path.ash pure functions using fn syntax | DESIGN-020 D9 | 2 | Track 1, C4 |
| TASK-E4 | Rewrite stdlib_parsing.rs tests: replace string-matching with actual parser validation for fn definitions, match expressions, if-as-expr, panic | SPEC-027 | 4 | E1, E2, E3 |

### Track 6: Documentation Finalization

| Task | Description | Spec | Est. Hours | Dependencies |
|------|-------------|------|------------|--------------|
| TASK-F1 | Finalize SPEC-009, SPEC-002, SPEC-003, SPEC-004 drafts (from P0-P3) | All | 2 | All |
| TASK-F2 | Update docs/spec/README.md if needed | - | 0.5 | F1 |
| TASK-F3 | Update CHANGELOG.md | - | 1 | All |
| TASK-F4 | Final verification: cargo test, cargo clippy, cargo fmt, zero warnings | - | 2 | All |

## Milestone Definition

**Critical Issue #6:** Milestone/acceptance criteria must require verification for parsing `Fn(...) -> ...`, qualified fn call resolution success/failure, fn/capability ambiguity rejection, lowering contract expressions, and recursion/panic boundary behavior.

**Phase complete when:**

### Parser/AST (Track 1)
1. `fn` is a recognized keyword in the lexer
2. `fn` definitions parse as `Definition::Function(FnDef)` at module level
3. **AC:** Parser accepts `fn foo(n: Int) -> Int { n + 1 }` and produces correct AST
4. **AC:** Parser accepts `Fn(Int) -> Int` type syntax in parameter and return type positions
5. **AC:** Parser rejects `fn` inside fn bodies (fn cannot be nested)
6. fn bodies parse match expressions, if-as-expressions, and panic as Expr nodes
7. **AC:** `Expr::If`, `Expr::Panic`, `Expr::Block` AST variants exist and are produced by parser

### Name Resolution (Track 2)
8. `module::name(args)` resolves to fn when target is a fn definition
9. **AC:** `module::name(args)` produces type error when target is a capability (not a fn)
10. **AC:** Test: `use path::fn_name` imports fn; calling imported fn works
11. **AC:** Test: `pub fn` exports fn; importing module can call it
12. **AC:** Fn/capability ambiguity detection: when a name could be both, context (call syntax) determines resolution; ambiguous cases produce clear errors

### Type System (Track 3)
13. fn bodies use tail-expression return; `ret` is rejected in fn bodies
14. Purity checking rejects Expr::Policy, Expr::CheckObligation, and all workflow keywords in fn bodies
15. **AC:** Test purity violation detection: `fn bad() { act io:println("hello") }` is rejected
16. fn calls in workflow bodies are effect-neutral (Epistemic classification)
17. **AC:** Type checker correctly classifies `Fn(Int) -> Int` as distinct from effectful function types

### Contract Lowering (Track 3 - Critical Issue #5)
18. **AC:** Contract lowering tests pass: `requires: n >= 0` lowers to `{ var: "n", constraint: Gte(0) }`
19. **AC:** Contract lowering handles compound predicates: `requires: n >= 0 && n < 100` lowers correctly
20. **AC:** Contract lowering rejects non-arithmetic constraints in fn contracts: `requires: HasCapability(Fs)` is rejected

### Semantics (Track 4)
21. fn panic in workflow context produces RuntimeFailure CompletionPayload
22. **AC:** Test recursion: `fn factorial(n: Int) -> Int { if n <= 1 then 1 else n * factorial(n - 1) }` evaluates correctly
23. **AC:** Test panic boundary: fn that panics propagates panic to calling workflow

### std/ Library (Track 5)
24. All fn definitions in std/src/option.ash, std/src/result.ash, std/src/io/path.ash parse successfully
25. stdlib_parsing.rs tests validate constructs through the parser (not string-matching)

### Documentation (Track 6)
26. SPEC-002, SPEC-003, SPEC-004, SPEC-009 updated with fn language
27. Zero clippy warnings, zero compilation warnings, all tests pass
28. CHANGELOG.md updated

**Failure Mode Tests Required:**
- Parsing `Fn(` with missing closing paren produces helpful error
- Calling undefined fn `foo()` produces "unknown function" error
- Calling capability with `::` syntax (`provider::action()`) produces "use : for capabilities" error
- Fn/capability name collision produces ambiguity error with helpful resolution hint

**Not in scope for this phase:**
- Workflow implementing capability (`implements Cap` clause) -- future phase
- Proxy collapse into workflow -- future phase
- String constraints (SPEC-028 Stage 2) -- future phase
- Z3 compile-time proving for fn contracts (SPEC-028 Stage 3) -- future phase
- Dependent constraints (SPEC-028 Stage 4) -- future phase
- IO capability syntax cleanup (multi-action capabilities, colon separator fixes) -- separate phase

## Estimated Total Effort

**Critical Issue #7:** Task granularity was too optimistic. Parser/resolver/type tasks have been split to match real implementation order.

~120 hours across 40 tasks across 6 tracks (revised from 95 hours / 30 tasks).

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
Track 0 (Prerequisites): P0, P0a, P1 → P2, P3

Track 1 (AST):
  A0 → A0a → A0b
  A1 → A3 → A4 → A5
  A2 → A6a
  A5 → A5a → A5b
  A6 → A7
  A6b, A6a → A10
  A8, A9 independent
  A11 independent

Track 2 (Names):
  A4 → B1 → B1a → B1b → B4
  B1 → B3 → B3a
  B1b, A11 → B2

Track 3 (Types):
  Track 2, A5a → C1 → C2 → C3
  Track 2 → C4 → C10
  A8 → C5
  A7 → C6
  A5 → C7 → C7a → C8 → C8a
  C9 independent
  C8, D2 → D3

Track 4 (Semantics):
  C2, C4 → D1
  C4 → D2

Track 5 (std/):
  Track 1, C4 → E1, E2, E3 → E4

Track 6 (Docs):
  All → F1 → F2, F3 → F4
```

**Critical path:** A0 → A0a → A0b → A1 → A3 → A4 → B1 → B1a → B1b → C7 → C7a → C8 → C8a → D2 → D3 → E1 → E4 → F4

**Note:** D1 (fn evaluation rules) runs in parallel to D2/D3 but is not on the critical path for the std/ library work.

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
- Add `fn_def` to module-level definition alternatives
- Add match expression grammar to expression section
- Add if-as-expression grammar (distinct from workflow if_stmt)
- Add fn call syntax: `name(args)`, `module::name(args)`
- Note: `::` for qualified fn calls, `:` for capability calls only

### SPEC-003 (Type System)
- Add fn typing judgment: Γ ⊢ fn f(x:τ) { body } : FnType
- Add FnType: `(τ*) -> τ` (no effect slot)
- Add purity judgment over Expr: Γ ⊢ e : pure | impure | resolve
- Add generic fn instantiation rule
- Add fn call in workflow context rule (effect-neutral)
- Add recursion is allowed (no termination constraint)

### SPEC-004 (Operational Semantics)
- Add fn evaluation rules: LET, TAIL, MATCH, IF-TRUE, IF-FALSE, CALL, PANIC
- Define FnResult ::= Value | Panic (no Effect/Trace/Provenance/Obligations)
- Define fn panic propagation in workflow context

### SPEC-009 (Module System)
- Add `fn_def` to `definition` production rule
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
