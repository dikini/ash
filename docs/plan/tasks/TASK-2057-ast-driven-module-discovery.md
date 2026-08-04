# TASK-2057: AST-Driven Module Discovery

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§3-5, §8 (`M-DISCOVER`, `M-PARSE-FILE`)
**Owned rule:** MOD-REAL-001
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2057 AST-driven module discovery](../SEMANTIC-RULE-COVERAGE.md#task-2057-ast-driven-module-discovery)

## Semantic accounting

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Source-anchored ModuleNotFound and CircularDependency diagnostics; canonical module identities; source-kind-independent module units and parity; checked interfaces; interface-driven imports and visibility; module-aware Core/CPS lowering; linked Engine admission; and CLI/daemon terminal parity.
**Layers:** type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Evidence identifiers:** positive `TEST-MOD-REAL-001-AST-DISCOVERY`; negative
`TEST-MOD-REAL-001-LOOKALIKE-REJECTION`; mutation `TEST-MOD-REAL-001-SCAN-NONAUTHORITY`;
parity `not_applicable`.
**Next obligation:** TASK-2059 consumes the TASK-2058 carrier for source acquisition, module units, and structural source diagnostics; TASK-2060 interfaces; TASK-2061 imports/visibility; TASK-2062 lowering; TASK-2063 admission; TASK-2064 diagnostic conformance and client parity; TASK-2065 closeout.

## Task-owned evidence

**Canonical traceability rule:** `SEM-MODULE-REALIZATION-001`, the traceability alias for
`MOD-REAL-001` in SPEC-103.

`ash_parser::discover_module_declarations` exposes parser-owned child name, visibility, source
form, declaration span, and source path from an authoritative `ModuleFile`. The resolver consumes
that handoff to create file and inline structural graph children. It reads and parses a crate root
once, and an inline declaration never probes a file child.

- **Positive:** `TEST-MOD-REAL-001-AST-DISCOVERY` is the public parser/resolver integration
  target. It checks the handoff fields, file-child edges, inline structural nodes, exact duplicate
  anchors, and the one-read root carrier.
- **Negative:** `TEST-MOD-REAL-001-LOOKALIKE-REJECTION` proves malformed module text fails in
  the parser and comments/literals cannot publish a child edge.
- **Mutation:** `TEST-MOD-REAL-001-SCAN-NONAUTHORITY` generates comment/literal lookalikes and
  proves they cannot change discovered file-child keys.
- **Parity:** not applicable. This prerequisite handoff has no paired execution relation.

The delivery is Type-only. It is tested evidence for the stated handoff, not a proof and not
complete SPEC-103 parity.

## Description

Replace semantic discovery of `mod name;` with traversal of parsed `ModuleFile`/`ModuleDecl` nodes. A raw-text scan may not create a graph edge or semantic module fact after parsing.

## Dependencies

- ✅ TASK-2056 — target rule, seam audit, and ownership packet.

## Current → target

**Current files:** `crates/ash-parser/src/resolver.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/surface.rs`.

**Delivered state:** each resolver source is parsed as a `ModuleFile`; crate metadata and root
structural discovery share that root carrier. `discover_module_declarations` is the public
AST-derived handoff, and resolver graph edges come only from it. The former line-oriented
module-declaration scanner is removed.

**Deferred target clauses:** TASK-2057 does not yet provide the source-anchored
`ModuleNotFound`/`CircularDependency` diagnostics required by SPEC-103 §8. TASK-2059 owns common
source acquisition and structural failure behavior; TASK-2064 owns the rule-indexed diagnostics
and conformance evidence.

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

- [x] Graph edges and child facts originate only from parsed declarations.
- [x] Text lookalikes cannot create a graph edge.
- [x] The resolver declaration scanner is removed and AUDIT-207 records its denylist/removal
      evidence; discovered non-resolver scanners are classified.
- [x] Focused parser/resolver tests, fmt, and clippy pass.
- [ ] Source-anchored `ModuleNotFound` and `CircularDependency` diagnostics remain deferred to
      TASK-2059/TASK-2064; this checked gap keeps the task and phase `partial`/`below_spec`.

## Handoffs

- **Consumes:** parser `ModuleFile` and `ModuleDecl` surface carriers.
- **Produces:** public `ash_parser::discover_module_declarations` records with parser-owned name,
  visibility, source form, span, and source path; parser-owned file and inline structural edges;
  and focused positive, negative, and mutation evidence.
- **Downstream owners:** TASK-2059 consumes the TASK-2058 carrier for common file/inline source
  acquisition, module units, and structural source diagnostics; TASK-2060 owns
  checked interfaces; TASK-2061 owns imports and visibility; TASK-2062 owns Core/CPS lowering;
  TASK-2063 owns linked Engine admission; TASK-2064 owns diagnostic conformance and CLI/daemon
  parity; TASK-2065 owns phase closeout.
- **Non-goals:** Canonical identity, common file/inline module units, source-anchored missing/cycle diagnostics, inline checking, import binding, visibility enforcement, summaries, lowering, admission, runtime execution, and client parity.

## Files and verification

**Files:** `crates/ash-parser/src/resolver.rs`, `crates/ash-parser/src/parse_module.rs` or existing parser APIs, parser/resolver tests.

```text
cargo test -p ash-parser resolver
cargo test -p ash-parser parse_module
cargo clippy -p ash-parser --all-targets -- -D warnings
cargo fmt --check
```
