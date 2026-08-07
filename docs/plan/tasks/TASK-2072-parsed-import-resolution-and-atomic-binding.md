# TASK-2072: Parsed Import Resolution and Atomic Binding

**Status:** Complete
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§6-7 (`M-IMPORT-EDGE`, `M-IMPORT-CYCLE`, `M-BIND`)
**Owned rule:** MOD-REAL-004 complete parsed imports, cycles, precedence, and binding
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json)
**Semantic coverage map:** [TASK-2072](../SEMANTIC-RULE-COVERAGE.md#task-2072-parsed-import-resolution-and-atomic-binding)

## Semantic authority and axes

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** The implemented Type-layer slice resolves the admitted parsed
use/path/visibility/alias/re-export forms against TASK-2075's name-only
`CanonicalProvisionalNameView`, stages canonical bindings/edges and non-authorizing `pub use`
facts, applies deterministic local/explicit/glob precedence and ambiguity/duplicate rules,
preserves defining identity, namespace, provenance, spans, visibility, and source order, and
rejects graph mismatches, inaccessible paths, unsupported shapes, and complete ordinary/public
re-export cycles before publication. Notation dependency edges and syntax-prepass cycle authority
remain the TASK-2074 parser-stage handoff; this resolver transports notation facts without
duplicating that graph authority. Checked bodies, final export closure, complete interface
finalization, Core/CPS, Engine admission/runtime, and client parity remain downstream. The target
rule therefore remains `partial / tested / below_spec`: TASK-2073 owns final checked/export-closed
publication, TASK-2069 consumes TASK-2073 only, and TASK-2064 owns parity.
Notation dependency edges and syntax-prepass cycle authority remain the TASK-2074 parser-stage handoff; this resolver transports notation facts without duplicating that graph authority. Checked bodies, final export closure, complete interface finalization, Core/CPS, Engine admission/runtime, and client parity remain downstream.
**Layers:** Type `partial`; Core/CPS/admission-runtime `not_applicable`; verification `partial`.
**Handoff status:** Complete for the task-owned non-authorizing parsed-import/binding handoff at
`partial / tested / below_spec`. Task 9 review, focused quality gates, and handoff documentation
are complete. TASK-2073 consumes the staged bindings for finalization; TASK-2074 retains notation
edge/cycle authority; TASK-2064 owns composed parity. The workspace-wide quality gate is not a
task-owned completion blocker; current frozen-route verification is recorded by TASK-2065.
**Next obligation:** The TASK-2072 task-owned parsed-import/binding handoff is complete at `partial / tested / below_spec`; TASK-2073 consumes its staged bindings for finalization, while TASK-2074 retains notation-edge/cycle authority and TASK-2064 owns parity. No final-interface, Core/CPS, admission/runtime, or client-parity authority is added here.

## Description

Replace TASK-2068's deliberately isolated resolver/binder leaves with one complete parsed-import
realization. It consumes only the canonical provisional name view, never the internal collected
snapshot, raw-source rediscovery, or M-CHECK
private facts as import authority. It must never inspect TASK-2075's checker-internal snapshot as
import authority, and it publishes no partial plan/binding set.

## Requirements

1. Cover all target parsed import grammar forms, structural traversal, visibility, aliases,
   re-exports, groups, globs, self/super/root forms, namespaces, and source-owned spans.
2. Apply complete precedence, ambiguity, duplicate-binding, and cycle rules deterministically.
3. Stage `pub use` identity/visibility/provenance facts without finalizing their export closure.
4. Reject every failure atomically across imports and sibling modules.

## TDD Steps

1. Add red all-grammar and identity/provenance tests.
2. Add red precedence/ambiguity/duplicate, staged-`pub use`, and complete-cycle atomicity tests.
3. Add file/inline normalized binding parity, generated grammar/visibility properties, mutation
   rejection, and a name-view-only authority fence.
4. Implement after RED, run focused tests/quality gates, then promote actual evidence only.

## Delivered implementation and evidence checkpoint

The resolver now consumes only the canonical parsed graph and TASK-2075's provisional name view.
It stages all bindings, import edges, notation facts, and public-use facts locally, then publishes
one result only after graph consistency, lookup, visibility, precedence, duplicate, and cycle
validation succeeds. Structural module aliases preserve child identities; lexical `self`/`super`
paths, parent-scoped members, transitive public re-exports, typed notation keys, and source
ordering retain their canonical provenance. Parser-owned notation dependency edges remain outside
this result and are consumed from TASK-2074's expanded graph handoff.

The focused `crates/ash-typeck/tests/task_2072_parsed_import_resolution.rs` target passes 21/21,
and its focused clippy target passes with `-D warnings`. Positive evidence covers every admitted
path family, aliases, groups, globs, parent-scoped members, notation summaries, visibility
carriers, public-use staging, transitive re-exports, and normalized file/inline projection.
Negative evidence covers inaccessible lexical shadowing, namespace ambiguity, duplicate bindings,
ordinary/public re-export cycles, and unsupported/partial resolution. Mutation/property evidence
covers graph-key mismatch, generated visibility outcomes, and generated grammar/visibility anchor
preservation. These are tests, not a proof or final-interface/runtime/parity claim.

## Scope and non-goals

This task excludes provisional collection ownership, checked bodies, final interface/export closure, Core/CPS, Engine transport/admission/runtime, execution, and client parity.

## Handoffs and completion checklist

- **Consumes:** TASK-2075 `CanonicalProvisionalNameView`, TASK-2070's narrow self alias, and
  TASK-2068's preserved foundation evidence. TASK-2071 supplies only the contract.
- **Produces:** atomic resolved bindings/edges plus staged `pub use` facts, non-authorizing.
- **Downstream owner:** TASK-2073 alone validates bodies, private/public views, export closure, and
  final public-use publication; TASK-2069 consumes only that complete handoff.
- **Integration/proof:** TASK-2064 owns cross-layer/file-inline/client parity.
- [x] Every admitted grammar form, precedence/ambiguity/duplicate/cycle outcome, and staged
  public-use path has focused positive/negative/mutation/property evidence.
- [x] No staged binding authorizes a final interface, Engine route, or direct evaluator.
- [x] Task 9 review, focused quality gates, handoff documentation, and downstream ownership boundaries
      are recorded.
