# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Common Changelog](https://common-changelog.org/).

## [Unreleased]

### Added
- TASK-183 follow-up refinement for the formalization boundary. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now distinguishes the canonical semantic corpus from authoritative source/handoff contracts and historical artifacts, and [docs/spec/SPEC-021-LEAN-REFERENCE.md](docs/spec/SPEC-021-LEAN-REFERENCE.md) is explicitly marked as a legacy sketch rather than a competing current spec.
- Formalization boundary note for TASK-183. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now names the canonical Lean/Rust proof corpus, separates migration-only artifacts, and lists the initial proof and bisimulation targets for the hardened language contract.
- TASK-182 follow-up tightening for runtime observable behavior. [SPEC-011](docs/spec/SPEC-011-REPL.md) now defers REPL error rendering to [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) now treats verification warnings as observable tooling output, and [docs/reference/runtime-observable-behavior-contract.md](docs/reference/runtime-observable-behavior-contract.md) is now mechanically a handoff note rather than a second canonical owner.
- Runtime observable behavior specification for TASK-182. [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) now owns the canonical CLI/REPL observable contract, runtime verification visibility, constructor-shaped ADT display, and explicit `Result`-based recoverable failure handling.
- ADT dynamic semantics tightening for TASK-181. [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-020](docs/spec/SPEC-020-ADT-TYPES.md), [docs/reference/parser-to-core-lowering-contract.md](docs/reference/parser-to-core-lowering-contract.md), [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md), and [docs/reference/runtime-observable-behavior-contract.md](docs/reference/runtime-observable-behavior-contract.md) now define canonical constructor evaluation, constructor-shaped runtime `Variant` values, `Match` no-match behavior, and `if let` as sugar for `match` with a wildcard fallback arm. SPEC-004 now carries the normative operational semantics directly.
- Follow-up tightening for TASK-180. [SPEC-006](docs/spec/SPEC-006-POLICY-DEFINITIONS.md), [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md), [SPEC-018](docs/spec/SPEC-018-CAPABILITY-MATRIX.md), and [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md) now require named policy bindings at capability sites and define the capability-verification outcome set as a verification-time interface with explicit pre-execution incompatibility rejection for unsupported approval or transformation outcomes.
- Removal of `attempt`/`catch` from the canonical language for TASK-185. [SPEC-002](docs/spec/SPEC-002-SURFACE.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-014](docs/spec/SPEC-014-BEHAVIOURS.md), [SPEC-016](docs/spec/SPEC-016-OUTPUT.md), [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md), and [SPEC-020](docs/spec/SPEC-020-ADT-TYPES.md) now require explicit `Result` values and pattern matching for recoverable failures.
- Policy evaluation and verification semantics tightening for TASK-180. [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-006](docs/spec/SPEC-006-POLICY-DEFINITIONS.md), [SPEC-007](docs/spec/SPEC-007-POLICY-COMBINATORS.md), [SPEC-008](docs/spec/SPEC-008-DYNAMIC-POLICIES.md), [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md), [SPEC-018](docs/spec/SPEC-018-CAPABILITY-MATRIX.md), and [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md) now define one policy story from named binding through lowered `CorePolicy` to runtime `PolicyDecision`, with workflow `decide` limited to `Permit` / `Deny` and capability verification using the richer verification outcome set.
- Receive mailbox and scheduling semantics formalization for TASK-179. [SPEC-002](docs/spec/SPEC-002-SURFACE.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-013](docs/spec/SPEC-013-STREAMS.md), and [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md) now define the source-selection model, source scheduling modifier semantics, guard timing, consumption timing, global `_` fallback, and one timeout budget for `receive`.
- Phase-judgment and rejection-boundary tightening for TASK-178. [SPEC-001](docs/spec/SPEC-001-IR.md), [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), and the canonical reference docs now separate parser, lowering, type, and runtime rejection classes from contract text while leaving implementation drift in task/planning notes.
- Canonical core language and execution-neutral IR tightening for TASK-177. [SPEC-001](docs/spec/SPEC-001-IR.md), [SPEC-002](docs/spec/SPEC-002-SURFACE.md), and [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md) now state the core-language form set, surface-sugar boundary, and backend-neutral IR invariants explicitly so later Rust and Lean work can treat them as canonical contract.
- Spec-hardening design in [docs/plan/2026-03-19-spec-hardening-design.md](docs/plan/2026-03-19-spec-hardening-design.md) and implementation plan in [docs/plan/2026-03-19-spec-hardening-plan.md](docs/plan/2026-03-19-spec-hardening-plan.md). These define the documentation gate required before Rust convergence resumes, with explicit goals for unambiguous Rust/Lean implementation, execution-neutral IR, and theory-grounded semantics.
- Spec-hardening task files [TASK-177](docs/plan/tasks/TASK-177-freeze-canonical-core-language-and-ir.md) through [TASK-184](docs/plan/tasks/TASK-184-audit-spec-hardening-readiness.md). These add a new pre-alignment task track for canonical core semantics, phase judgments, `receive`, policy, ADT, observable-behavior, and formalization-boundary tightening.
- [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md) and [docs/reference/runtime-observable-behavior-contract.md](docs/reference/runtime-observable-behavior-contract.md) as the canonical type/runtime and runtime/observable handoff references (TASK-163). They freeze required type-layer outputs, runtime/verification rejection boundaries, normative REPL-observable behavior, and stdlib-visible ADT/runtime guarantees for downstream convergence work.
- [docs/reference/parser-to-core-lowering-contract.md](docs/reference/parser-to-core-lowering-contract.md) as the canonical lowering handoff for stabilized workflow, policy, `receive`, and ADT forms (TASK-162). It defines the required surface-to-core mappings, lowering-time rejection cases, and preservation rules for downstream parser/core convergence work.
- [docs/reference/surface-to-parser-contract.md](docs/reference/surface-to-parser-contract.md) as the canonical parser handoff for stabilized workflow, policy, and ADT forms (TASK-161). It fixes the accepted syntax, required surface AST outputs, legal parser rejections, and the parser-versus-later-phase boundary for downstream convergence work.
- Convergence continuation task files [TASK-161](docs/plan/tasks/TASK-161-surface-to-parser-handoff-contract.md) through [TASK-176](docs/plan/tasks/TASK-176-final-convergence-audit.md). These extend the spec-to-implementation convergence program with explicit handoff-reference, parser/lowering, type/runtime, REPL/CLI, ADT, and final-audit tasks.
- [docs/design/LANGUAGE-TERMINOLOGY.md](docs/design/LANGUAGE-TERMINOLOGY.md) as a shared language guide for project documents. It standardizes terms such as `source scheduling modifier`, `scheduler`, `InstanceAddr`, and `ControlLink`, and reserves `policy` for authorization semantics.
- Phase-A convergence task files in [docs/plan/tasks/TASK-156-canonicalize-workflow-form-contracts.md](docs/plan/tasks/TASK-156-canonicalize-workflow-form-contracts.md), [docs/plan/tasks/TASK-157-canonicalize-policy-contracts.md](docs/plan/tasks/TASK-157-canonicalize-policy-contracts.md), [docs/plan/tasks/TASK-158-canonicalize-streams-runtime-verification-contracts.md](docs/plan/tasks/TASK-158-canonicalize-streams-runtime-verification-contracts.md), [docs/plan/tasks/TASK-159-canonicalize-repl-cli-contracts.md](docs/plan/tasks/TASK-159-canonicalize-repl-cli-contracts.md), and [docs/plan/tasks/TASK-160-canonicalize-adt-contracts.md](docs/plan/tasks/TASK-160-canonicalize-adt-contracts.md). Splits the first convergence phase into concrete documentation tasks with explicit requirements, TDD-style review steps, dependencies, and non-goals.
- Spec-to-implementation convergence design in [docs/plan/2026-03-19-spec-to-implementation-convergence-design.md](docs/plan/2026-03-19-spec-to-implementation-convergence-design.md). Defines the spec-first recovery model, phase ordering, task-shaping rules, and completion criteria for bringing Rust code back into compliance.
- Spec-to-implementation convergence plan in [docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md](docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md). Breaks convergence into fresh follow-up tasks ordered from canonical spec repair through final implementation audit.
- Rust codebase review findings report in [docs/audit/2026-03-19-rust-codebase-review-findings.md](docs/audit/2026-03-19-rust-codebase-review-findings.md). Records checklist-driven implementation findings across baseline, policy, REPL/CLI, streams/runtime-verification, and ADT clusters without modifying Rust source.
- Rust codebase review checklist in [docs/audit/2026-03-19-rust-codebase-review-checklist.md](docs/audit/2026-03-19-rust-codebase-review-checklist.md). Maps audit-identified risky task clusters to concrete Rust review targets and questions.
- Non-Lean task consistency audit report in [docs/audit/2026-03-19-task-consistency-review-non-lean.md](docs/audit/2026-03-19-task-consistency-review-non-lean.md). Links task-plan drift to prior spec-audit findings to prepare for Rust code review.
- Specification consistency audit report for SPEC-001 through SPEC-018 in [docs/audit/2026-03-19-spec-001-018-consistency-review.md](docs/audit/2026-03-19-spec-001-018-consistency-review.md). Captures cross-spec inconsistencies and aligned areas without modifying the specs.

### Changed
- Tightened TASK-177 core-contract wording so SPEC-001 scopes the runtime form set precisely, SPEC-002 treats optional binding and implicit `done` as surface sugar, and SPEC-004 gives explicit expression-level semantics for `Constructor` and `Match`. The core-language contract now separates canonical truth from surface convenience without widening runtime meaning to unrelated type-level contracts.
- `SPEC-001`, `SPEC-002`, and `SPEC-004` now separate canonical core truth from surface sugar and implementation convenience. The canonical IR contract is explicitly backend-neutral, so future interpreter and JIT implementations must preserve the same meaning rather than discover it locally.
- Reordered the convergence roadmap in [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) so a new spec-hardening gate now precedes Rust alignment phases. Parser/lowering, type/runtime, REPL/CLI, ADT, and final convergence work remain planned, but only after the language definition is tightened for mechanical Rust and Lean implementation.
- Tightened the workflow-declaration grammar in SPEC-002 so `observes` names `behaviour_ref` rather than a generic capability list. The grammar now preserves the existing semantic split between read-only behaviour inputs and separately declared write authority.
- Clarified workflow input declarations, `receive` scheduling terminology, and workflow communication/link wording across SPEC-002, SPEC-013, SPEC-014, SPEC-017, SPEC-018, and SPEC-020. The docs now distinguish `observes` from `receives`, reserve `policy` for authorization semantics, use `source scheduling modifier` for `receive` source selection, and define control-link transfer as consume-on-success.
- Canonicalized the ADT contract across SPEC-003, SPEC-004, SPEC-013, SPEC-014, and SPEC-020 (TASK-160). ADT declarations now use one `TypeDef`/`TypeExpr` source model, runtime variants store only constructor names plus fields, pattern and exhaustiveness rules share that same enum model, and the required Option/Result helper surface is explicitly narrowed.
- Canonicalized the REPL and CLI contract across SPEC-005, SPEC-011, and SPEC-016 (TASK-159). `ash repl` is now the sole normative REPL entrypoint, the REPL command set is limited to `:help`, `:quit`, `:type`, `:ast`, and `:clear`, and REPL display output is explicitly separated from workflow output capabilities.
- Canonicalized the stream and runtime-verification contract across SPEC-004, SPEC-013, SPEC-014, SPEC-017, and SPEC-018 (TASK-158). `receive` modes, control-arm behavior, declaration requirements, runtime-context responsibilities, and verification outcomes now share one end-to-end contract.
- Canonicalized the policy contract across SPEC-003, SPEC-004, SPEC-006, SPEC-007, SPEC-008, SPEC-017, and SPEC-018 (TASK-157). Policies now have one continuous story from named declaration and combinator expression through lowered core policy representation, type-checking constraints, and runtime `PolicyDecision` outcomes.
- Expanded [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) with logical post-Phase-20 convergence phases. The remaining convergence work is now split into docs-only handoff phases, implementation-alignment phases, and a final audit phase rather than living only inside the convergence plan document.

### Fixed
- Restored `ash-cli` compatibility with boxed `Value::List` and `Value::Record` constructors, and moved binary command tests into an integration harness so `cargo test -p ash-cli` passes again on the workflow-contracts branch.

### Changed
- Canonicalized the spec contracts for `check`, `decide`, and `receive` across SPEC-001, SPEC-002, SPEC-003, SPEC-004, SPEC-017, and SPEC-018 (TASK-156). `check` is now obligation-only, `decide` always names an explicit policy, and `receive` is documented as an epistemic mailbox-input form with one authoritative surface grammar.

### Added
- TASK-183 follow-up refinement for the formalization boundary. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now distinguishes the canonical semantic corpus from authoritative source/handoff contracts and historical artifacts, and [docs/spec/SPEC-021-LEAN-REFERENCE.md](docs/spec/SPEC-021-LEAN-REFERENCE.md) is explicitly marked as a legacy sketch rather than a competing current spec.
- Formal proofs for semantic properties (Phase 19, TASK-149 through TASK-155):
  - `Ash/Proofs/Pattern.lean` - Pattern match determinism and totality proofs
  - `Ash/Proofs/Pure.lean` - Constructor purity proof (effect system)
  - `Ash/Proofs/Determinism.lean` - Expression evaluation determinism proof
  - `Ash/Proofs/Progress.lean` - Progress theorem (well-typed programs don't get stuck)
  - `Ash/Proofs/Preservation.lean` - Preservation theorem (types preserved during evaluation)
  - `Ash/Proofs/TypeSafety.lean` - Type safety corollary combining progress and preservation
  - `Ash/Types/Basic.lean` - Core type system definitions (`Ty` inductive)
  - `Ash/Types/WellTyped.lean` - Well-typed relation for expressions
  - Helper lemmas: `merge_envs_assoc`, `env_lookup_bind_eq`, `join_epistemic_left`, etc.
  - **Note**: Some theorems use `sorry` due to Lean 4 partial function limitations
- Effect tracking for receive capability (TASK-108). Complete effect tracking for all capabilities:
  - Added `Workflow::Receive` variant to surface AST for pattern matching on incoming messages
  - Added `ReceiveMode` enum (NonBlocking, Blocking with optional timeout)
  - Added `StreamPattern` enum (Wildcard, Literal, Binding) for receive arm patterns
  - Added `ReceiveArm` struct (pattern, guard, body, span)
  - Implemented effect computation: receive is `Epistemic` (read-only consumption) per SPEC-017
  - Effect properly joins with all arm body effects: `arms.iter().map(|arm| arm.body.effect()).fold(Epistemic, join)`
  - Added 7 property tests for receive effect tracking (empty, blocking, epistemic body, operational body, multiple arms, control receive)
  - Updated desugar passes (sequencing, optional bindings, nested blocks) to handle Receive
  - Updated lowering with placeholder for future core IR support
  - Verified compliance with SPEC-017 Section 2.1: receive → Epistemic effect
- Option and Result standard library (TASK-136). Core standard library modules:
  - `std/src/option.ash` - Option<T> type with Some/None variants
  - `std/src/result.ash` - Result<T, E> type with Ok/Err variants
  - Helper functions: is_some, is_none, is_ok, is_err, unwrap, unwrap_or, unwrap_err
  - Transformation functions: map, map_err, and_then, and, or, ok_or, ok, err
  - `std/src/prelude.ash` - Auto-imported types and functions
  - `std/src/lib.ash` - Main library exports
  - `std/README.md` - Standard library documentation
  - Integration tests verifying stdlib files parse correctly
- Spawn returns Instance with Option<ControlLink> (TASK-134). Updated spawn expression to return a composite type that can be split into InstanceAddr and Option<ControlLink>:
  - Added `Instance`, `InstanceAddr`, and `ControlLink` types to `ash-core` value module
  - Added `Value::Instance`, `Value::InstanceAddr`, `Value::ControlLink` variants for runtime representation
  - Added `Expr::Spawn { workflow_type, init }` expression for spawning workflows
  - Added `Expr::Split` expression to decompose Instance into (InstanceAddr, ControlLink)
  - Added `Workflow::Spawn` and `Workflow::Split` workflow variants
  - Implemented evaluation logic in `ash-interp` for spawn (creates Instance with unique ID) and split (returns tuple)
  - Added visualization support for new workflow variants
  - Full test coverage for spawn/split evaluation and instance value display
- Affine control link transfer semantics (TASK-135). Runtime tracking for control link consumption:
  - `ControlLinkRegistry` for tracking link availability vs consumed state
  - `ControlLinkError` for invalid link usage (AlreadyConsumed, NotFound, InvalidInstance)
  - `acquire()` method for consuming links with exactly-once semantics
  - `verify_unused()` for checking link availability without consuming
  - `consume()` for explicit consumption, `is_consumed()` for state checking
  - Support for kill, pause, resume, check_health supervision operations
  - Workflow variants: Kill, Pause, Resume, CheckHealth for supervision
- Match and if-let expression evaluation (TASK-133). Interpreter support for match expressions:
  - `Expr::Match` evaluation with pattern matching and arm selection
  - `Expr::IfLet` evaluation as sugar for match
  - Integration with pattern matching engine for variable binding
  - Proper error handling for non-exhaustive matches
  - Full test coverage for all match forms
- Pattern matching engine (TASK-132). Core pattern matching implementation in `crates/ash-interp/src/pattern.rs`:
  - `Value::Variant` type added to `ash-core` for representing variant values
  - `Pattern::Variant` pattern matching with field extraction
  - Support for unit variants: `Pattern::Variant { name: "None", fields: None }`
  - Support for variants with fields: `Pattern::Variant { name: "Some", fields: Some([("value", var)]) }`
  - Nested variant pattern matching (variants containing tuples, records, etc.)
  - Full test coverage for variant matching including negative cases
- Constructor evaluation for ADTs (TASK-131). Interpreter support for evaluating constructor expressions like `Some { value: 42 }`:
  - `Value::Variant` type in `ash-core` with constructor name and field values
  - `Expr::Constructor` evaluation in `ash-interp/src/eval.rs`
  - Helper methods: `Value::variant()` and `Value::unit_variant()` for creating variants
  - Support for nested constructors, expressions in fields, and variable references
  - Full test coverage for Option, Result, and custom ADT constructors

### Fixed
- Dead code review: 5 `#[allow(dead_code)]` items audited, 2 duplicate `ws()` functions identified for removal
- Code review issues from Phase 17 (P0, P1, P2 priority):
  - **Critical (P0)**: Fixed `unwrap()` abuse in parsers (`parse_pattern.rs`, `parse_expr.rs`) using `is_some_and()`
  - **Critical (P0)**: Removed unnecessary `Box::new` + immediate dereference pattern in `lower.rs`
  - **High (P0)**: Added `#[must_use]` to all public constructors and pure functions in `exhaustiveness.rs`, `instantiate.rs`, `type_env.rs`
  - **High (P1)**: Boxed large `Value` enum variants (`List`, `Record`, `Variant`, `Instance`) to reduce memory footprint
  - **High (P1)**: Removed broken ternary expression parsing from `parse_expr.rs`
  - **Medium (P2)**: Added `HashMap::with_capacity()` hints where collection size is known
  - **Medium (P2)**: Optimized pattern matching to avoid temporary HashMap allocation
  - **Low (P2)**: Removed dead code/comments from parser files
  - **Low (P2)**: Fixed float literal lowering to truncate to Int instead of returning Null
- Type definition duplication between `ash-core` and `ash-typeck`. Unified `TypeDef` types by using AST types from `ash_core::ast` in `type_env.rs` with conversion functions.
- Inefficient TypeEnv creation in pattern checking. Added static `EMPTY_ENV` with `OnceLock` to avoid repeated allocations.
- Keyword lookup performance. Replaced O(n) `matches!` pattern with O(1) `HashSet` lookup using `OnceLock` for lazy initialization.
- Magic string for variant tag. Extracted `"__variant"` to `const VARIANT_TAG` constant.
- Visibility enum completeness. Added `Crate` variant to `Visibility` enum.
- Unsafe `unwrap()` usage in parser. Replaced with `is_some_and()` pattern.
- Error message formatting. Changed to lowercase per Rust conventions.

### Added
- TASK-183 follow-up refinement for the formalization boundary. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now distinguishes the canonical semantic corpus from authoritative source/handoff contracts and historical artifacts, and [docs/spec/SPEC-021-LEAN-REFERENCE.md](docs/spec/SPEC-021-LEAN-REFERENCE.md) is explicitly marked as a legacy sketch rather than a competing current spec.
- Match and if-let expression evaluation (TASK-133). Pattern matching in the interpreter:
  - `eval_match()` function for evaluating `Expr::Match` with multiple arms
  - `eval_if_let()` function for evaluating `Expr::IfLet` expressions
  - Pattern matching using existing `match_pattern()` engine
  - Variable bindings scoped to match arm bodies via `Context::extend()`
  - `NonExhaustiveMatch` error when no arm matches
  - Support for all pattern types: literal, variable, wildcard, tuple, record, list
  - First matching arm wins semantics
  - If-let desugars to match with pattern/then/else branches
- Generic type instantiation (TASK-129). Type parameter substitution for ADTs:
  - `instantiate(def, args)` function for substituting type parameters with concrete types
  - `Substitution::from_pairs()` method for creating substitutions from type variable pairs
  - `InstantiateError::ArityMismatch` for wrong number of type arguments
  - Support for instantiating enums, structs, and type aliases
  - Recursive substitution in nested types (tuples, records, constructors)
  - Full test coverage for single and multi-parameter type definitions
- Type check patterns for match expressions (TASK-128). Pattern type checking in `crates/ash-typeck/src/check_pattern.rs`:
  - `check_pattern(env, pattern, expected)` function for checking patterns against expected types
  - `Bindings` type: `HashMap<String, Type>` for pattern variable bindings
  - Support for `Pattern::Wildcard` - matches any type with no bindings
  - Support for `Pattern::Variable` - binds variable to expected type
  - Support for `Pattern::Literal` - checks literal type compatibility
  - Support for `Pattern::Variant` - checks variant patterns against sum types
  - Support for `Pattern::Tuple` - checks element count and types
  - Support for `Pattern::Record` - checks field names and types
  - Support for `Pattern::List` - checks element patterns and rest bindings
  - New error types: `PatternMismatch`, `UnknownVariant`, `PatternArityMismatch`, `InvalidPattern`
  - `TypeEnv` for managing type definitions and variable scopes during pattern checking
  - Full test coverage for all pattern types including nested patterns
- Type check constructors for ADTs (TASK-127). Type checking for constructor expressions like `Some { value: 42 }`:
  - `TypeEnv` struct to track type definitions and constructor mappings
  - `register_type(def: TypeDef)` to add type definitions
  - `lookup_constructor(name)` to find constructor's type and variant index
  - `lookup_type(name)` to retrieve type definitions
  - `add_builtin_types()` to register Option and Result types
  - `check_expr` function with `Expr::Constructor` case for expression type checking
  - Error types: `UnknownConstructor`, `MissingField`, `UnknownField`
  - Full test coverage for Option and Result constructors
- Parse type definitions (TASK-124). Parser for ADT type definitions in `ash-parser`:
  - `parse_type_def` module with `TypeDef`, `TypeBody`, `VariantDef`, `Visibility`, and `TypeExpr` types
  - Support for enums: `type Status = Pending | Processing | Completed;`
  - Support for struct types: `type Point = { x: Int, y: Int };`
  - Support for type aliases: `type Name = String;`
  - Support for generics: `type Option<T> = Some { value: T } | None;`
  - Support for visibility: `pub type Result<T, E> = Ok { value: T } | Err { error: E };`
  - Full test coverage for all type definition forms
- AST Extensions for Algebraic Data Types (TASK-120). Foundation for Phase 17 ADT implementation:
  - `Pattern::Variant` for enum variant pattern matching
  - `Expr::Constructor` for ADT value construction
  - `Expr::Match` for pattern matching expressions
  - `Expr::IfLet` for if-let syntactic sugar
  - `MatchArm` struct representing match arms
  - `TypeDef`, `TypeBody`, `VariantDef` for type definitions
  - `Visibility` enum for visibility modifiers (pub, crate, private)
  - `TypeExpr` for surface syntax type expressions
  - `Type::Instance`, `Type::InstanceAddr`, `Type::ControlLink` for spawn/control link support
- Stream iteration over registered streams. Added `StreamRegistry::iter()` method to iterate over all registered providers, `StreamContext::iter_providers()` to iterate over typed providers, and `StreamContext::try_recv_any()` to receive from any available stream (non-blocking). Updated `wait_for_message()` in `execute_stream.rs` to poll all registered streams using `try_recv_any()` instead of busy-waiting.

### Fixed
- Infinite recursion bug in `TypedSendableProvider::send()` and `BidirectionalStreamProvider::send()` methods. Both were calling themselves instead of delegating to `inner.send()`. Added proper write_schema validation and delegation to inner provider.

### Changed
- Refactored parser utilities to eliminate code duplication between `parse_set.rs` and `parse_send.rs`. Created new `parse_utils.rs` module with shared helper functions: `parse_capability_ref()`, `keyword()`, `literal_str()`, and `skip_whitespace_and_comments()`.

### Added
- TASK-183 follow-up refinement for the formalization boundary. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now distinguishes the canonical semantic corpus from authoritative source/handoff contracts and historical artifacts, and [docs/spec/SPEC-021-LEAN-REFERENCE.md](docs/spec/SPEC-021-LEAN-REFERENCE.md) is explicitly marked as a legacy sketch rather than a competing current spec.
- Set statement execution for output behaviours (TASK-105). New `execute_set` module in `ash-interp` with `execute_set(capability, channel, value, behaviour_ctx)` async function for setting values on writable channels. Integrates with `BehaviourContext` to lookup settable providers, validates values before setting, and returns `ExecError::CapabilityNotAvailable` or `ExecError::ValidationFailed` on errors. Added `Workflow::Set` variant to AST with `capability`, `channel`, and `value` fields. Extended `execute_workflow` with new `execute_workflow_with_behaviour` function that accepts `BehaviourContext` for set statement support.
- Parse send statement for output streams (TASK-104). New `parse_send` module in `ash-parser` with `SendExpr` struct for parsing `send capability:channel expr` syntax. Similar to `parse_set` but without the `=` sign. Supports variables, string literals, and function calls for structured values.
- Parse set statement for output behaviours (TASK-103). New `parse_set` module in `ash-parser` with `SetExpr` struct for parsing `set capability:channel = expr` syntax. Supports simple values, function calls for structured values, and expressions.
- Sendable Stream Provider Trait (TASK-102). Output capability support for writable streams:
  - `SendableStreamProvider` trait extending `StreamProvider` with `send(&self, value: Value)` async method
  - `would_block(&self) -> bool` for backpressure detection (default: false)
  - `flush(&self)` async for buffered sends (default: no-op)
  - `TypedSendableProvider` wrapper with `write_schema` validation before sending values
  - `MockSendableProvider` for testing with `sent_values()` and `sent_count()` inspection
  - `SendableRegistry` for managing sendable providers by capability/channel
  - `StreamContext` extension with `register_sendable()`, `get_sendable()`, and `send()` methods
- Settable Behaviour Provider Trait (TASK-101). Output capability support for writable channels:
  - `SettableBehaviourProvider` trait extending `BehaviourProvider` with `set(&self, value: Value)` async method and optional `validate(&self, value: &Value)` for pre-checks
  - `TypedSettableProvider` wrapper with `write_schema` validation before setting values
  - `MockSettableProvider` for testing with configurable validators
  - `SettableRegistry` for managing settable providers by capability/channel
  - `BehaviourContext` extension with `register_settable()`, `get_settable()`, and `set()` methods
  - `ValidationError` enum with variants for invalid values, out of range, and format errors
  - `ExecError::ValidationFailed` variant for validation failure reporting
- Bidirectional Provider Wrappers (TASK-107). Combine input/output capabilities for unified providers:
  - `BidirectionalBehaviour` trait combining `sample()` and `set()` operations for internal implementations
  - `BidirectionalBehaviourProvider` wrapper implementing both `BehaviourProvider` and `SettableBehaviourProvider` with separate `read_schema` and `write_schema` validation
  - `MockBidirectionalProvider` for testing with read/write operation tracking via `read_count()` and `write_count()`
  - `BidirectionalStream` trait combining `recv()`/`try_recv()` and `send()` operations for internal implementations
  - `BidirectionalStreamProvider` wrapper implementing both `StreamProvider` and `SendableStreamProvider` with separate read/write schema validation
  - `MockBidirectionalStream` for testing with `push()` for receive queue and `sent_values()`/`sent_count()` for sent values inspection
- Phase 16: Runtime Verification (TASK-114 to TASK-119). Comprehensive runtime verification framework:
  - Capability availability verifier (TASK-114). New `CapabilityVerifier` checks all required capabilities are available with correct modes (observable, settable, sendable, receivable).
  - Obligation satisfaction checker (TASK-115). New `RuntimeObligationChecker` verifies role requirements and obligation presence at runtime.
  - Effect compatibility checker (TASK-116). New `EffectChecker` ensures workflow effect level is within runtime bounds.
  - Static policy validator (TASK-117). New `StaticPolicyValidator` detects always-denied operations and approval requirements pre-execution.
  - Per-operation runtime verifier (TASK-118). New `OperationVerifier` with async `verify()` for checking capability availability, mode support, policy evaluation, and rate limiting.
  - Verification aggregator (TASK-119). New `VerificationAggregator` combines all verifiers into unified `VerificationResult` with `can_execute()` determination.
- Phase 15: Capability Integration (TASK-108 to TASK-113). Full integration of capabilities with obligations, policies, provenance, and type safety:
  - Effect tracking for all capability operations (TASK-108). Added `Workflow::effect()` method that computes total effect by joining operation effects (Observe/Receive=Epistemic, Set/Send=Operational).
  - Obligation checking with capabilities (TASK-109). New `ObligationChecker` verifies workflows have required input/output capabilities and sufficient effect levels.
  - Policy evaluation for input/output (TASK-110). New `CapabilityPolicyEvaluator` with support for Permit, Deny, RequireApproval, and Transform decisions.
  - Provenance tracking for all capabilities (TASK-111). New `CapabilityProvenanceTracker` records all capability operations with event types, values, and policy decisions.
  - Capability declaration verification (TASK-112). New `CapabilityChecker` framework for verifying workflows use declared capabilities.
  - Read/write type checking (TASK-113). New `CapabilitySchemaRegistry` validates input/output values against provider schemas with separate read/write types.
- Phase 14: Typed Providers (TASK-096 to TASK-100). Runtime type safety for Rust/Ash provider boundary:
  - `TypedBehaviourProvider` and `TypedStreamProvider` wrapper structs carrying type schemas (TASK-096)
  - Schema validation logic with `Type::matches()` and `Type::validate()` methods (TASK-097)
  - Typed registry integration - `BehaviourRegistry` and `StreamRegistry` now store typed providers with schema lookup via `get_schema()` (TASK-098)
  - Runtime validation in providers - sample/recv operations validate values against schemas (TASK-099)
  - Enhanced type error reporting with `ExecError::TypeMismatch` and path tracking (TASK-100)
- Shared capability types module (ash-core). New `capability.rs` consolidates `Direction`, `RoleName`, `RequiredCapabilities`, and `WorkflowCapabilities` to eliminate duplication across crates.
- Phase 13: Streams and Behaviours (TASK-088 to TASK-095). Complete stream processing and behaviour sampling implementation:
  - Stream AST types: `StreamRef`, `Receive`, `ReceiveMode`, `Mailbox` with overflow strategies (TASK-088)
  - Stream provider trait with `StreamRegistry` and `StreamContext` for async stream operations (TASK-089)
  - Parse receive construct with guards, timeouts, and control streams (TASK-090)
  - Mailbox implementation with size limits and overflow strategies (DropOldest, DropNewest, Error) (TASK-091)
  - Stream execution with pattern matching, guard evaluation, blocking/non-blocking modes (TASK-092)
  - Behaviour provider trait with `BehaviourRegistry` and `BehaviourContext` for sampling (TASK-093)
  - Parse observe construct with constraints (TASK-094)
  - Observe execution with sampling and pattern binding (TASK-095) New `execute_observe` module in `ash-interp` provides `execute_observe()` and `execute_changed()` functions. `execute_observe()` samples behaviour providers with constraints, matches patterns against sampled values, and binds variables. `execute_changed()` detects value changes since last sample. Includes 6 comprehensive async tests and proper error handling for missing providers and pattern match failures.
- Stream execution with pattern matching and guards (TASK-092). New `execute_stream` module in `ash-interp` provides `execute_receive` function supporting non-blocking/blocking/timeout modes, pattern matching with destructuring, guard clause evaluation, and control stream handling. Includes 10 comprehensive async tests.
- Interactive REPL (Phase 12, TASK-077 to TASK-083). New `ash-repl` crate with rustyline integration provides expression evaluation, multi-line input detection, commands (:help, :quit, :type, :ast, :clear), tab completion for keywords, persistent history, and syntax error highlighting with helpful suggestions.
- Embedding API for ash-engine crate (Phase 11, TASK-071 to TASK-076). Unified Engine type with Parse→Check→Execute lifecycle, builder pattern (EngineBuilder), thread-safe workflow storage, and capability provider traits. CLI integration complete with 160 tests passing.

### Changed
- Updated dependencies to latest versions: winnow 0.5.40 → 0.6.26, pulldown-cmark 0.9.6 → 0.13.1, thiserror 1.0.69 → 2.0.18, colored 2.1 → 3.1.1. Fixed winnow API migration (PResult → ModalResult, Located → LocatingSlice) and pulldown-cmark breaking changes (TagEnd::CodeBlock, CodeBlockKind).
- Fixed all clippy warnings (66+ style and correctness warnings). Removed redundant pattern matching, fixed `#[must_use]` attributes, added `#[allow]` annotations for intentional patterns.
- Fixed test failures: updated forall/exists tests to use non-keyword identifiers; removed method_chain test (feature not in spec); fixed error_recovery test assertion.
- **Breaking**: Z3/SMT is now a mandatory dependency (removed `smt` feature flag). Policy conflict detection is always enabled for security-critical workflows. System must have Z3 C library installed.

### Added
- TASK-183 follow-up refinement for the formalization boundary. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now distinguishes the canonical semantic corpus from authoritative source/handoff contracts and historical artifacts, and [docs/spec/SPEC-021-LEAN-REFERENCE.md](docs/spec/SPEC-021-LEAN-REFERENCE.md) is explicitly marked as a legacy sketch rather than a competing current spec.
- List literal parsing for expressions: `[1, 2, 3]` or `["a", "b"]` syntax. Updated SPEC-002 to define list_literal production. Added Literal::List variant to surface AST.

### Added
- TASK-183 follow-up refinement for the formalization boundary. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now distinguishes the canonical semantic corpus from authoritative source/handoff contracts and historical artifacts, and [docs/spec/SPEC-021-LEAN-REFERENCE.md](docs/spec/SPEC-021-LEAN-REFERENCE.md) is explicitly marked as a legacy sketch rather than a competing current spec.
- Initial project structure with workspace and 9 crates (ash-core, ash-macros, ash-parser, ash-typeck, ash-interp, ash-provenance, ash-cli, ash-lint, ash-doc-tests)
- Effect lattice implementation with 4 levels: Epistemic, Deliberative, Evaluative, Operational (TASK-001)
- Comprehensive property tests for Effect lattice: associativity, commutativity, idempotence, absorption, identity (18 property tests)
- Value system with 9 variants: Int, String, Bool, Null, Time, Ref, List, Record, Cap (TASK-002)
- Value serialization/deserialization with JSON roundtrip property tests (17 property tests)
- Core AST definitions for workflow language (SPEC-001)
- AST visualization module generating Graphviz DOT output
- Comprehensive development tooling: git hooks, sccache, insta, proptest
- CI/CD plan with 6 workflow types and initial ci-fast.yml implementation
- Documentation: 5 specification documents, architecture document, CLI specification
- Custom lint tool (ash-lint) for Ash-specific rules
- Doc-test extractor for testing code examples in specifications
- Fuzz testing infrastructure with cargo-fuzz (ash-fuzz crate)
- Benchmark suite with Criterion (ash-bench crate)
- Procedural macros for Effectful and Provenance derive
- Serde Serialize/Deserialize support for all AST types: Workflow, Pattern, Expr, Guard, etc. (TASK-003)
- List pattern variant for prefix matching with optional rest binding: `List(Vec<Pattern>, Option<Name>)` (TASK-003)
- Pattern helper methods: `bindings()` to collect variable names, `is_refutable()` to check match exhaustiveness (TASK-003)
- Comprehensive AST tests: workflow construction, pattern bindings, serde roundtrip (TASK-003)
- Provenance tracking types: WorkflowId, Provenance, TraceEvent, Decision with fork lineage (TASK-004)
- Provenance tests: lineage accumulation, uniqueness, serde roundtrip (TASK-004)
- Pattern matching system with 6 variants: Variable, Tuple, Record, List, Wildcard, Literal (TASK-005)
- Pattern helper methods: bindings() for collecting variables, is_refutable() for exhaustiveness (TASK-005)
- Property testing strategies: arb_effect, arb_value, arb_pattern, arb_name, arb_expr (TASK-006)
- Proptest helpers tests: binding uniqueness, value roundtrip, name validation (TASK-006)
- Test helpers module: WorkflowBuilder, test_capability, var, lit, var_expr utilities (TASK-007)
- 13 test helper tests for builders and utilities (TASK-007)
- Token definitions with 50+ variants: keywords, literals, operators, delimiters (TASK-008)
- Span tracking for source locations with line/column/byte offset (TASK-008)
- LexError types with thiserror for unexpected chars, unterminated strings, invalid numbers (TASK-008)
- Lexer implementation with streaming tokenization, comments, error recovery (TASK-009)
- 16 lexer tests for keywords, identifiers, literals, operators, spans, recovery (TASK-009)
- 23 lexer property tests: identifiers, literals, spans, error recovery, stress tests (TASK-010)
- Workflow parser with 18 tests: observe, act, let, if, for, par, etc. (TASK-013)
- Expression parser with 22 tests: precedence climbing, literals, binary ops (TASK-014)
- Error recovery with 12 tests: synchronization, recovery strategies (TASK-015)
- Surface to Core lowering with 17 tests: workflow, expr, pattern lowering (TASK-016)
- Desugaring with 17 tests: sequencing, optional bindings, nested blocks (TASK-017)
- Lexer property tests: 18 proptest-based tests for identifiers, literals, spans, error recovery, and stress testing (TASK-010)
- Surface AST types for parser: Program, Definition, Workflow, Expr, Pattern, and supporting types with full span tracking (TASK-011)
- 49 surface AST tests: construction tests for all major types, span extraction tests, and variant coverage (TASK-011)
- Parser core using winnow: ParseInput with Stream impl, ParseError with span tracking, basic combinators (TASK-012)
- 25 parser core tests: ParseInput Stream operations, ParseError formatting, whitespace/alphanumeric/keyword combinators (TASK-012)
- CLI implementation with 5 commands: check, run, trace, repl, dot (TASK-053 to TASK-057)
- check command with --all, --strict, --format flags for type checking workflows
- run command with --input, --output, --trace flags for workflow execution
- trace command with provenance capture and JSON/NDJSON/CSV export formats
- repl command with rustyline integration, :help, :type, :bindings commands
- dot command for Graphviz DOT output generation
- 23 CLI tests for argument parsing, command execution, and help output
- Example workflows: 12 examples across 4 categories (basics, control-flow, policies, real-world) (TASK-047)
- Examples README with overview, quick start, and learning path
- Basics examples: hello-world, variables, expressions, observe pattern
- Control flow examples: conditionals, foreach, parallel, sequential
- Policy examples: role-based and time-based access control
- Real-world examples: customer support and code review workflows
- Comprehensive tutorial covering installation through real-world examples (TASK-048)
- API documentation for all crates: ash-core, ash-parser, ash-typeck, ash-interp, ash-provenance, ash-cli (TASK-049)
- Core benchmarks: effect operations, value operations, pattern matching (TASK-050)
- Parser benchmarks: simple, complex, and nested workflow parsing
- Interpreter benchmarks: workflow construction, expression evaluation, traversal
- Serialization benchmarks: JSON roundtrip for workflows and values
- Optimization documentation: performance characteristics and tuning guide (TASK-051)
- Parser fuzzing target for validating input handling (TASK-052)
- Type checker fuzzing target for crash detection
- Module resolution algorithm (TASK-069). Implemented `ModuleResolver` with file system abstraction trait for testability, supporting Rust-style module resolution (`mod foo;` → `foo.ash` or `foo/mod.ash`). Includes circular dependency detection, proper error handling with `ResolveError`, and `MockFs` for testing. 19 comprehensive tests covering single files, nested modules, directory modules, and circular dependencies.
- Policy combinators implementation with 12 AST variants: Var, And, Or, Not, Implies, Sequential, Concurrent, ForAll, Exists, MethodCall, Call (TASK-062)
- Policy expression parser with support for infix operators (&, |, !, >>), method chaining (.and(), .or(), .retry()), and quantifiers (forall, exists) (TASK-062)
- Policy type checker with 21 tests: type inference, validation, method signatures, context bindings (TASK-062)
- Policy normalization passes: flatten nested and/or, eliminate double negation, constant folding preparation (TASK-062)
- 12 surface AST tests for PolicyExpr variants: construction, span extraction, variant coverage (TASK-062)
- Visibility checking for type checker (TASK-070). Implemented `VisibilityChecker` with `check_access` method for validating item accessibility across module boundaries. Supports all visibility variants: `pub`, `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path)`. Includes `VisibilityError` enum with `PrivateItem` and `MissingContext` error variants. 17 comprehensive tests covering all visibility scenarios.
- ash-engine crate with unified Engine type for embedding (TASK-071). Created new crate with `Engine` struct providing unified interface for Parse → Check → Execute workflow. Engine implements `Send + Sync` for thread safety. Builder pattern via `EngineBuilder` with fluent API for capability configuration. 39 tests covering engine creation, configuration, and error handling.
- Engine::parse and Engine::parse_file methods (TASK-072). Implemented source string and file path parsing with automatic lowering from surface AST to core IR. 29 comprehensive tests including valid workflows, invalid syntax, file I/O, and property tests for error preservation.
- Engine::check method for type checking (TASK-073). Integrated with ash_typeck to validate workflows. Creates wrapper type carrying surface workflow for type checker compatibility. Added `ret` keyword support across parser, lexer, surface AST, lowering, and type checking. 28 tests covering type checking scenarios.
- Engine::execute, run, and run_file methods (TASK-074). Async execution methods providing full pipeline (parse → check → execute) and individual execution. Integrated with ash_interp for workflow interpretation. 32 tests including async behavior, concurrent execution, and error handling.
- Standard capability providers (TASK-075). Implemented `StdioProvider` (print, println, read_line) and `FsProvider` (read_file, write_file, exists) with `CapabilityProvider` trait. Builder methods `with_stdio_capabilities()` and `with_fs_capabilities()` on EngineBuilder. 28 tests covering provider behavior and trait implementations.
- CLI integration with ash-engine (TASK-076). Updated ash-cli to use Engine API instead of direct crate dependencies. `ash run` command now uses Engine::run_file with stdio/fs capabilities. `ash check` command uses Engine::parse + Engine::check. All 23 CLI tests pass with new implementation.

### Changed

### Deprecated

### Removed

### Fixed

### Security
