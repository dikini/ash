# TASK-2068 Delivered Scoped `super` Grouped Ordinary-Function Imports Evidence

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Record the delivered Type-only, binding-only route for inherited grouped
ordinary-function imports whose `UsePath::Nested` base begins with exactly one `super`.

**Architecture:** Existing parser-owned `UseItem { name, alias, span }` facts are the only
member-provenance authority: every selected member and member-specific error retains that span,
never the enclosing use span. The dedicated scope-backed resolver starts from
`ModuleKey::parent()`, traverses structural children, preflights every group member and every
edge, and atomically returns an opaque plan. The private structural binder exposes only one named
projection; generic resolver and binder routes remain unchanged.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-typeck`, `proptest`; repository semantic-accounting validators.

---

## Scope and semantic boundary

Authority is SPEC-103 §§3, 5, 6, 8, and 9, including `M-IMPORT-EDGE`, `M-IMPORT-CYCLE`, and
`M-BIND`, under MOD-REAL-004 / SEM-MODULE-REALIZATION-004. This delivered M-SUPER-GROUP slice
is `partial / tested / below_spec`: Type and verification are `partial`,
Core/CPS/admission-runtime are `not_applicable`, and run-route impact is `prerequisite`.

The only admitted forms are:

```ash
use super::{parent_a, parent_b as local_b};
use super::sibling::{child_a, child_b as local_b};
```

The importer is non-root; the inherited `UsePath::Nested` base has exactly one leading `super`, no
outer alias, zero or more structural children after the canonical parent, and at least one member.
Each member is an ordinary function and uses its member `as` spelling or natural name. The route
reuses canonical scopes/snapshots, visibility and whole-public-path checks, local
collision/duplicate checks, same-module no-edge behavior, cycle detection, and atomic publishing.
It preflights a final member named `super` before child lookup, so that failure takes its member
span before private-child visibility could apply. It consumes TASK-2067 canonical graph units,
existing parser member spans, and TASK-2068 provisional scopes; it produces only Type-layer
plan/bound-set/edge facts.

Out of scope: root/repeated `super`, `self`, `crate`, unprefixed, standard-library, or external bases; simple/glob/non-nested/nested-group forms; outer aliases or empty groups; public/restricted/re-export forms; non-functions; generic resolver/binder changes; final interfaces/export closure; Core/CPS; admission/runtime/parity; and precedence. No commit is authorized.

## Delivered evidence and retained boundary

The focused `task_2068_scoped_super_grouped_ordinary_function_imports` target passes 13/13,
including its 16-case property. POSITIVE, IDENTITY, FILE-INLINE-PARITY, and PROPERTY are positive
evidence; VISIBILITY-DIAGNOSTIC, ROOT-DIAGNOSTIC, LOCAL-COLLISION, DUPLICATE-BINDING, and
AUTHORITY-FENCE are negative evidence; CYCLE-ATOMICITY is mutation evidence. The visibility matrix
includes public, crate, super, restricted, and `pub(self)` same-module zero-edge cases. Its shape
matrix covers root/repeated/final `super`, the final-`super` private-child precedence case, outer
aliases, unsupported heads/forms/use visibility, and whole-use anchoring only where no member AST
exists. A later non-function member rejects atomically at its own span, and the cycle witness uses
a real cross-module cycle.

The implementation anchor is
`crates/ash-typeck/src/canonical_structural_module_binder.rs#projects-scoped-grouped-super-ordinary-function-imports-into-bindings`
(`sha256:0bfb497fe11b17623bcb39485d2420f80d3b5c64a39ba4ad9642f148d4413a06`).
The resolver anchor is
`crates/ash-typeck/src/canonical_simple_import_planner.rs#resolves-inherited-grouped-ordinary-function-imports-through-one-super`
(`sha256:77ff8e437ada70fc1182bb52b99a4d9e56c2fe39c669ffe87258ff71d8eb021c`), and the
public export boundary in `crates/ash-typeck/src/lib.rs` is
`sha256:68775641f867d47b9f4a7af344b856eb3ec132f256659fc68bdc51444e934f86`.
`crates/ash-typeck/src/canonical_module_binder.rs` remains generic-only at
`sha256:aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6`.

## Handoffs and completion boundary

The route consumes TASK-2067 canonical graph units, parser-owned member spans, and TASK-2068
provisional scopes. It produces only a Type-layer opaque plan, binding projection, and canonical
edges; it is neither a checked final interface nor a runtime credential. Its run-route impact is
`prerequisite`: TASK-2068 owns remaining Type/interface/import/binder work, TASK-2069 owns
lowering/Engine transport, and TASK-2064 owns integration parity. It remains
`partial / tested / below_spec`; test evidence is not proof or parity evidence.

## Delivered traceability

`IMPL-MODULE-SCOPED-SUPER-GROUPED-ORDINARY-FUNCTION-IMPORTS` is implemented and exactly these
ten test witnesses are tested, all linked to `SEM-MODULE-REALIZATION-004`:

- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-POSITIVE`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-IDENTITY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-VISIBILITY-DIAGNOSTIC`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-ROOT-DIAGNOSTIC`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-LOCAL-COLLISION`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-DUPLICATE-BINDING`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-CYCLE-ATOMICITY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-FILE-INLINE-PARITY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-PROPERTY`
- `TEST-MOD-REAL-004-SCOPED-SUPER-GROUP-IMPORT-AUTHORITY-FENCE`

The trace witnesses are evidence only: they do not claim proof, a final interface, later-layer
authority, or client parity. This is historical evidence for TASK-2068's completed foundation;
Phase 207 remains In progress; TASK-2069 remains
planned. This evidence promotion does not update `CHANGELOG.md`.
