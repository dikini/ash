# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Common Changelog](https://common-changelog.org/).

## [Unreleased]

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

### Changed

### Deprecated

### Removed

### Fixed

### Security
