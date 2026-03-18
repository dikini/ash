# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Common Changelog](https://common-changelog.org/).

## [Unreleased]

### Fixed
- Infinite recursion bug in `TypedSendableProvider::send()` and `BidirectionalStreamProvider::send()` methods. Both were calling themselves instead of delegating to `inner.send()`. Added proper write_schema validation and delegation to inner provider.

### Changed
- Refactored parser utilities to eliminate code duplication between `parse_set.rs` and `parse_send.rs`. Created new `parse_utils.rs` module with shared helper functions: `parse_capability_ref()`, `keyword()`, `literal_str()`, and `skip_whitespace_and_comments()`.

### Added
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
- List literal parsing for expressions: `[1, 2, 3]` or `["a", "b"]` syntax. Updated SPEC-002 to define list_literal production. Added Literal::List variant to surface AST.

### Added
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
