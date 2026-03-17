# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Common Changelog](https://common-changelog.org/).

## [Unreleased]

### Changed
- Updated dependencies to latest versions: winnow 0.5.40 → 0.6.26, pulldown-cmark 0.9.6 → 0.13.1, thiserror 1.0.69 → 2.0.18, colored 2.1 → 3.1.1. Fixed winnow API migration (PResult → ModalResult, Located → LocatingSlice) and pulldown-cmark breaking changes (TagEnd::CodeBlock, CodeBlockKind).

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
- Policy combinators implementation with 12 AST variants: Var, And, Or, Not, Implies, Sequential, Concurrent, ForAll, Exists, MethodCall, Call (TASK-062)
- Policy expression parser with support for infix operators (&, |, !, >>), method chaining (.and(), .or(), .retry()), and quantifiers (forall, exists) (TASK-062)
- Policy type checker with 21 tests: type inference, validation, method signatures, context bindings (TASK-062)
- Policy normalization passes: flatten nested and/or, eliminate double negation, constant folding preparation (TASK-062)
- 12 surface AST tests for PolicyExpr variants: construction, span extraction, variant coverage (TASK-062)

### Changed

### Deprecated

### Removed

### Fixed

### Security
