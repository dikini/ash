# TASK-2059: File/Inline Module-Unit Parity

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§3-5, §8 (`M-PARSE-FILE`, `M-PARSE-INLINE`)
**Owned rule:** MOD-REAL-002
**Run-route impact:** prerequisite

## Description

Construct one module-unit route for file-backed and inline modules. Source acquisition is the only permitted difference; expansion, declaration collection, checking inputs, diagnostics, and artifact identities then share one path.

## Dependencies

- 📝 TASK-2057 — AST-driven discovery.
- 📝 TASK-2058 — canonical module identity and artifact substrate.

## Current → target

**Current files:** `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/resolver.rs`, `crates/ash-engine/src/module_loader.rs`.

**Current state:** file-backed children have resolver graph support. Inline definitions remain
parser carriers with selected Engine rejection guards, and their definition-only item grammar
cannot express the existing `use` forms needed for file/inline parity. The paths do not establish
equivalent module units.

**Target state:** a `ModuleDecl` becomes a canonical module unit. File acquisition parses selected source once. Inline acquisition turns the declaration’s definition list into the same module-file/module-unit representation. Later consumers cannot inspect source kind to choose different semantics.

## Requirements

1. Amend inline parsing to accept the same existing `use`, definition, and nested-module item
   forms as a file `ModuleFile`, then implement the two source-acquisition rules with equal
   module-unit outputs.
2. Preserve source-specific spans and path diagnostics while normalizing semantic identity.
3. Remove the inline ordinary-definition rejection that exists only because no common route exists; replace it with ordinary check diagnostics.
4. Reject a missing file child, malformed inline declaration, duplicate child, and structural cycle before a partial interface is published.
5. Make macro/notation scope boundaries explicit and equal for both source forms.

## TDD Steps and evidence

1. Create paired fixtures: one file tree and one inline source with equivalent types, functions, `use`, macro, and nested child declarations.
2. Assert equal normalized module-unit snapshots and equal diagnostic classes for equivalent invalid cases.
3. Add mutation tests that alter only source acquisition details and confirm semantic output does not change.
4. Add negative tests proving parent declarations do not leak into an inline child and child declarations do not leak into the parent without exports.

## Completion checklist

- [ ] File and inline declarations accept the same module-item domain and construct one common
  module-unit representation.
- [ ] Equivalent source forms have equal normalized module-unit evidence.
- [ ] Inline ordinary definitions use ordinary diagnostics rather than unsupported-form guards.
- [ ] Focused parser/Engine tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** AST structural declarations and canonical identities.
- **Produces:** source-kind-independent module units for TASK-2060 and TASK-2061.
- **Downstream owner:** TASK-2060 owns checked interface creation; TASK-2064 owns end-to-end parity evidence.
- **Non-goals:** interface export closure, visibility, imported runtime execution, dynamic loading, or import-cycle support.

## Files and verification

**Files:** `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/resolver.rs`, `crates/ash-engine/src/module_loader.rs`, parser/engine module integration tests.

```text
cargo test -p ash-parser module
cargo test -p ash-engine --test module_file_check_tests
cargo clippy -p ash-parser -p ash-engine --all-targets -- -D warnings
cargo fmt --check
```
