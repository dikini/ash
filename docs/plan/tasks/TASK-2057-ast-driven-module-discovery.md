# TASK-2057: AST-Driven Module Discovery

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§3-5, §8 (`M-DISCOVER`, `M-PARSE-FILE`)
**Owned rule:** MOD-REAL-001
**Run-route impact:** prerequisite

## Description

Replace semantic discovery of `mod name;` with traversal of parsed `ModuleFile`/`ModuleDecl` nodes. A raw-text scan may not create a graph edge or semantic module fact after parsing.

## Dependencies

- ✅ TASK-2056 — target rule, seam audit, and ownership packet.

## Current → target

**Current files:** `crates/ash-parser/src/resolver.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/surface.rs`.

**Current state:** the resolver reads source and identifies file modules through line-oriented matching. Parser `ModuleFile` already carries parsed module declarations.

**Target state:** resolver entry points parse each source once, obtain child declarations only from AST nodes, retain declaration spans/origins, and produce deterministic source-anchored diagnostics. Any retained scanner is a test-only disagreement detector and cannot publish a graph edge.

## Requirements

1. Add an AST-to-discovery adapter with no duplicate parsing and no string pattern matching for module declarations.
2. Preserve `pub`, restricted visibility, child name, source form, and declaration origin from the parsed carrier.
3. Diagnose malformed source through parser diagnostics before resolution.
4. Add a fail-closed test proving a comment/string/text lookalike does not create a child edge.
5. Remove or quarantine every semantic caller of the old scan; search call sites, not only the resolver definition.
6. Update AUDIT-207's scanner inventory and add the first denylist entries for module-declaration
   scanning; no unclassified production caller may remain.

## TDD Steps and evidence

1. Write parser/resolver integration tests for file declarations, `pub mod`, comments, string lookalikes, malformed declarations, duplicate children, and source span diagnostics.
2. Make the old scanner-derived path fail the new tests.
3. Implement AST-driven discovery.
4. Add proptest generation over non-declaration lines: inserting arbitrary comments/literals must not change discovered child keys.

## Completion checklist

- [ ] Graph edges and child facts originate only from parsed declarations.
- [ ] Text lookalikes cannot create a graph edge.
- [ ] Existing semantic scan callers are removed or fenced non-authorizing.
- [ ] AUDIT-207 records the resolver scan's removal/fence evidence and no new caller is unclassified.
- [ ] Focused parser/resolver tests, fmt, and clippy pass.

## Handoffs

- **Consumes:** parser `ModuleFile` and `ModuleDecl` surface carriers.
- **Produces:** AST-derived structural declarations with canonical source origins for TASK-2058 and TASK-2059.
- **Downstream owner:** TASK-2058 turns discovered declarations into stable graph identities; TASK-2059 acquires sources.
- **Non-goals:** inline checking, import binding, visibility enforcement, summaries, lowering, and runtime execution.

## Files and verification

**Files:** `crates/ash-parser/src/resolver.rs`, `crates/ash-parser/src/parse_module.rs` or existing parser APIs, parser/resolver tests.

```text
cargo test -p ash-parser resolver
cargo test -p ash-parser parse_module
cargo clippy -p ash-parser --all-targets -- -D warnings
cargo fmt --check
```
