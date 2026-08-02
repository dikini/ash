---
id: spec.ash.module-realization
title: Ash Module Realization and Operational Semantics
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-08-02
verified_against:
  specs:
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
  plans:
    - docs/plan/PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md
---

# SPEC-103: Module Realization and Operational Semantics

**Status:** Draft target-state specification. It is normative for the target module semantic path.
It supersedes historical module/import operational claims only where this specification explicitly
amends them; existing implemented summary packets retain authority within their stated scopes.
**Scope:** This specification defines the complete language-level realization of source modules. It makes `mod name;` and `mod name { ... }` semantically equivalent after source acquisition, replaces semantic text scans with parsed `ModuleFile` traversal, and defines the module route through checking, lowering, admission, and Engine execution.
**Depends on:** SPEC-095b, SPEC-095c, SPEC-097b, SPEC-098b, SPEC-098c, SPEC-099b,
SPEC-099c, SPEC-057, SPEC-062, and PLAN-203.

## 1. Purpose and authority

A parser accepting a module declaration does not implement a module system. A module system is complete only when a declaration creates a stable module identity, the module acquires source, its declarations are checked, its public interface is available to importers, and reachable definitions lower and execute through the one Engine route.

This specification owns that rule. Its amendment boundary is narrow and clause-specific:

| Existing authority and clauses | SPEC-103 treatment |
|---|---|
| SPEC-009 §§2–6, §8, and §10 | Superseded for target structural discovery, module trees/graphs, item resolution, visibility checking, and inline-body grammar. SPEC-103 §§3–8 supply the target executable rule. Historical examples and V1-frozen context remain evidence, not target execution authority. |
| SPEC-012 §§2–5, §7, and §9 | Superseded for target import syntax-in-context, resolution, re-exports, visibility, grammar composition, and diagnostics. SPEC-103 §§3, 6, and 8 supply the target AST/interface route. Capability-specific historical material remains outside this override unless a later target rule selects it. |
| SPEC-030 §§4–6 | Superseded for child-module loading, module-file checking, and compatibility/source-collection paths. SPEC-030 §3's two-pass sibling type-registration invariant remains in force. |
| SPEC-057 §§7–14 and SPEC-062 §§5–12 | Retained as the implemented bounded substrate for ordinary-type and type-computation summaries, except neither may authorize a raw source scan, a bare/path identity, Engine-private export ownership, or a second import route for the complete module path. SPEC-103 consumes and extends their transport through explicit compatibility/version amendments; it does not recreate their type identities, closure rules, versioning, or import-order behavior. |

SPEC-103 replaces general source-text scanning and Engine-private semantic export ownership for the
module route. It does not claim that SPEC-057 has already realized that broader result.

This specification does not revive removed `workflow` source forms. An executable program still enters through an ordinary checked `fn main` under the target function-first grammar.

## Rule index

| Rule | Requirement | Primary realization owner |
|---|---|---|
| MOD-REAL-001 | Parsed declarations are the only authority for structural module edges and child identities. | TASK-2057, TASK-2058 |
| MOD-REAL-002 | File-backed and inline children become equivalent module units after source acquisition. | TASK-2059 |
| MOD-REAL-003 | Checked public interfaces are export-closed and preserve defining identities. | TASK-2060 |
| MOD-REAL-004 | Imports, qualified paths, and visibility resolve only through canonical checked interfaces. | TASK-2061 |
| MOD-REAL-005 | Resolved checked modules lower to identity- and origin-preserving Core/CPS artifacts. | TASK-2062 |
| MOD-REAL-006 | The Engine links and admits one reachable module closure and proves CLI/daemon terminal parity. | TASK-2063, TASK-2064 |

## 2. Terms

**Module key** `m` is a stable, canonical identity consisting of a crate identity and a canonical path of module segments. It is not a bare filename, a parser-local index, or an import alias.

**Module source** is one of:

```text
Source ::= File(path) | Inline(parent-key, declaration-origin)
```

**Module unit** is the parsed `ModuleFile`, expanded surface state, checked declarations, interface, lowered definitions, and source-origin evidence attached to one module key.

**Module interface** is the checked public projection of one module. It contains exported child-module identities, declarations, callable/type/constructor/interface/implementation summaries, macro and notation syntax-phase summaries, visibility metadata, source origins, dependencies, and a schema version. It contains no implicit authority.

**Structural edge** is a parent-to-child edge created by a `mod` declaration. Structural edges form a rooted tree within each crate. A structural cycle is an error.

**Import edge** is a dependency introduced by `use`. It does not create a child module and does not authorize source loading by raw path. A first complete realization rejects an import cycle with a diagnostic that names the cycle. A later specification may add interface-first SCC checking.

## 3. Surface rule

This specification preserves the existing `mod` and `use` spellings, and refines the target module
body so that an inline child has the same item domain as a file module. This grammar amendment is
required for source-form parity:

```ebnf
module_declaration = visibility? "mod" identifier (";" | "{" module_item* "}") ;
module_item        = use_declaration | definition | module_declaration ;
```

For a declaration in module `p`:

```text
mod n;              declares child key child(p, n) with File source
mod n { definitions } declares child key child(p, n) with Inline source
```

`visibility` controls whether the child-module identity appears in `p`'s public interface. It does not flatten child declarations into the parent. `pub mod n` exports the name `n`; it does not export every declaration in `n`. A parent re-exports a child declaration only through explicit `pub use`.

The same identifier checks, duplicate-child diagnostics, visibility grammar, and declaration-origin rules apply to both forms.

## 4. Source acquisition and parity

A file-backed child obtains text from exactly these candidates, in order:

```text
<parent-directory>/<n>.ash
<parent-directory>/<n>/mod.ash
```

The resolver parses the selected source once as `ModuleFile`. It obtains an inline child directly
from the `ModuleDecl` item list already present in its parent `ModuleFile`. After that point neither
the graph, binder, checker, lowerer, nor Engine may branch on `File` versus `Inline` except to
preserve source locations and file-loading diagnostics.

The parity invariant is:

```text
same child declarations + same parent/module environment
  => same checked interface, resolved declaration identities, Core/CPS artifacts,
     admission result, and normalized terminal result
```

The invariant does not require equal filenames, byte offsets, or diagnostic display paths. It requires the same semantic result after source acquisition.

## 5. Authoritative module pipeline

Every semantic module route uses this sequence:

```text
source root
  -> parse ModuleFile
  -> construct structural module graph from ModuleDecl AST nodes
  -> acquire and parse child module sources
  -> expand syntax-phase macros and notation in each module scope
  -> collect declarations and provisional interfaces
  -> resolve use declarations and qualified references
  -> enforce visibility
  -> typecheck declarations and finalize interfaces
  -> lower reachable checked definitions to Core
  -> lower Core to CPS
  -> construct one admitted Engine artifact
  -> execute only the selected checked entry
```

No phase may recover module declarations, ordinary declarations, exports, imports, or visibility facts by scanning source text after the authoritative `ModuleFile` exists. A temporary compatibility reader may exist only when it is named, fenced, non-authorizing, and rejects if it would disagree with parsed data. TASK-2057 owns removal or quarantine of every such path.

`ash-core` owns stable identities and shared interface carriers. `ash-parser` owns source spelling, parser AST, and spans. `ash-typeck` owns binding and semantic validation. `ash-engine` transports checked artifacts, cache keys, and admitted execution; it does not define a second module semantics.

## 6. Resolution and visibility

Let `I(m)` be the finalized interface of module `m`. A qualified module path resolves from a crate root through public child-module interface entries. A `use` declaration resolves only against finalized or provisional checked interface entries; it does not walk filenames.

For a declaration reference in module `m`, resolution order is:

```text
local lexical binding
-> same-module declaration
-> explicit import
-> glob import
-> permitted lexical parent/module path
```

Each successful binding retains the defining declaration identity. Re-exporting changes an import path, not the defining identity.

Visibility is checked before an imported declaration enters the importing environment. The rule applies uniformly to module identities and declaration identities:

```text
private       visible only in the defining module
pub           visible wherever its enclosing public path is reachable
pub(crate)    visible only to modules in the same crate
pub(super)    visible only through the permitted ancestor boundary
pub(in path)  visible only in the named descendant/ancestor region
```

An inaccessible entry is not a missing name. The diagnostic must identify the declaration, its defining module, the attempted access path, and the violated visibility boundary.

## 7. Interface closure

A public interface is export-closed. Every public signature, row, type, constructor, interface, implementation, macro summary, notation summary, or nested module reference named in the interface must itself be publicly reachable or the export is rejected.

The interface has two views:

```text
PrivateInterface(m)  = facts available while checking m
PublicInterface(m)   = export-closed projection available to other modules
```

Private representation facts do not cross the public boundary. Public module summaries must preserve stable defining identities, visibility, dependency summaries, schema version, source anchors, and enough checked metadata for importers to validate the interface without rediscovering source text.

## 8. Module machine

This section gives the operational-style semantics for compilation and linking. It does not make modules runtime values.

A module-store entry is:

```text
Entry ::= Absent
        | Discovered(Source)
        | Parsed(ModuleFile, Source)
        | Expanded(ExpandedModule, Source)
        | Collected(ProvisionalInterface, ExpandedModule)
        | Bound(ResolvedModule, ProvisionalInterface)
        | Checked(CheckedModule, PublicInterface)
        | Lowered(CoreModule, PublicInterface)
        | Linked(CpsModule, PublicInterface)
        | Failed(ModuleDiagnostic)
```

A module-machine state is:

```text
Ξ ::= <G, S, W, D>
```

where `G` is the structural/import graph, `S` maps module keys to entries, `W` is an ordered worklist, and `D` is a diagnostic set. `->m` is one module-machine step.

### Discovery and parsing

```text
S(p) = Parsed(P, _)     d = module-declaration(P, n)     k = child(p, n)
source-of(d) = q        S(k) = Absent
--------------------------------------------------------- M-DISCOVER
<G, S, p::W, D> ->m <G + (p -struct-> k), S[k := Discovered(q)], k::W, D>
```

```text
S(k) = Discovered(File(path))     parse_file(path) = P
-------------------------------------------------- M-PARSE-FILE
<G, S, k::W, D> ->m <G, S[k := Parsed(P, File(path))], discover(k)::W, D>
```

```text
S(k) = Discovered(Inline(p, origin))     inline-items(p, origin) = items
----------------------------------------------------------------- M-PARSE-INLINE
<G, S, k::W, D> ->m <G, S[k := Parsed(module-file(items), Inline(p, origin))], discover(k)::W, D>
```

If file lookup fails, parsing fails, a child name is duplicated, or a structural edge would close a cycle, the machine produces `Failed` with a source-anchored diagnostic and does not manufacture a partial child.

### Expansion, collection, binding, and checking

```text
S(k) = Parsed(P, src)     expand_module(k, P) = E
------------------------------------------------ M-EXPAND
<G, S, k::W, D> ->m <G, S[k := Expanded(E, src)], collect(k)::W, D>

S(k) = Expanded(E, src)     collect(k, E) = PI
------------------------------------------------ M-COLLECT
<G, S, k::W, D> ->m <G, S[k := Collected(PI, E)], bind(k)::W, D>

S(k) = Collected(PI, E)     u = use-declaration(E)     target(k, u, PI, interfaces(S)) = t
---------------------------------------------------------------------------------------- M-IMPORT-EDGE
<G, S, k::W, D> ->m <G + (k -import-> t), S, collect-imports(k)::W, D>

path(G + (k -import-> t), t, k) = c
----------------------------------- M-IMPORT-CYCLE
<G, S, k::W, D> ->m <G, S[k := Failed(import-cycle(c))], W, D + import-cycle(c)>

S(k) = Collected(PI, E)     imports-resolved(k, G, interfaces(S))
---------------------------------------------------------------- M-IMPORTS-READY
<G, S, k::W, D> ->m <G, S, bind(k)::W, D>

S(k) = Collected(PI, E)     bind(k, G, interfaces(S)) = R
------------------------------------------------------- M-BIND
<G, S, k::W, D> ->m <G, S[k := Bound(R, PI)], check(k)::W, D>

S(k) = Bound(R, PI)     check(k, R, PI) = (C, I)
------------------------------------------------ M-CHECK
<G, S, k::W, D> ->m <G, S[k := Checked(C, I)], lower(k)::W, D>
```

`M-IMPORT-EDGE` traverses only expanded parsed `use` nodes. `target` resolves through structural
module identities and the minimal provisional public view needed to name a target; it never reads
a filesystem path or source text. A provisional view contains names, defining module identities,
visibility declarations, and source anchors only; it contains no type/callable facts and cannot be
published to an importer as a `PublicInterface`. `M-IMPORT-CYCLE` fails the entire dependency
closure atomically before `M-BIND` can publish a binding. `bind` rejects an unresolved or
inaccessible import, ambiguity, duplicate binding, or any failed dependency. `check` validates
export closure and does not publish `I(k)` if checking fails.

### Lowering, linking, and entry execution

```text
S(k) = Checked(C, I)     lower_to_core(C) = K
------------------------------------------- M-LOWER
<G, S, k::W, D> ->m <G, S[k := Lowered(K, I)], link(k)::W, D>

S(k) = Lowered(K, I)     lower_to_cps(K) = Q
--------------------------------------------- M-LINK
<G, S, k::W, D> ->m <G, S[k := Linked(Q, I)], W, D>
```

An entry may be admitted only after every reachable dependency needed by its checked Core/CPS artifact is `Linked`. The Engine consumes the linked artifact under PLAN-203. It must reject incomplete, forged, stale, or failed module entries; it must not select a direct evaluator or a source-text import fallback.

## 9. Required properties

1. **AST authority:** no semantic graph edge or declaration fact comes from a text scan when an authoritative `ModuleFile` is available.
2. **Source parity:** file-backed and inline modules with equivalent declarations produce equivalent checked interfaces and lowered artifacts.
3. **Identity preservation:** aliases and re-exports preserve defining module and declaration identities.
4. **No implicit flattening:** child exports enter a parent only through an explicit re-export.
5. **Visibility before registration:** inaccessible declarations never enter an importing scope.
6. **Order independence:** declaration and import order do not change successful outcomes where dependency order does not matter.
7. **Failure atomicity:** an invalid module does not publish a partial public interface.
8. **No runtime authority:** module interfaces, imports, rows, and graph membership do not install provider or handler authority.
9. **One execution route:** a selected entry and its module dependencies execute only through the checked Core → CPS → Engine path.
10. **Client parity:** CLI and daemon compare the same admitted program, inputs, bindings, and normalized terminal result.

## 10. Scope and non-goals

This phase realizes ordinary modules, imports, visibility, checked interfaces, and entry linking.
It adds no new lexical `mod` or `use` form, but it does amend the inline module item grammar so an
inline child may contain the existing `use` and nested-module forms. It does not add packages, a
registry, lazy loading, hot reload, user-visible runtime module values, dynamic imports,
generalized incremental workspace indexing, or import-cycle initialization.

The first realization rejects structural and import cycles. It does not promise cross-module recursive initialization. Macro and notation expansion remains syntax-phase only; the phase must preserve their existing identity and hygiene contracts without making them runtime callable.

## 11. Conformance

Conformance requires positive, negative, mutation, and parity evidence for each rule family. At minimum:

- `mod child;` resolves `child.ash` then `child/mod.ash` from parsed declarations.
- inline and file-backed modules with the same declarations have equal normalized interfaces and Core/CPS artifacts.
- missing children, duplicate children, structural cycles, import cycles, ambiguous imports, private access, and invalid re-exports reject with anchored diagnostics.
- `pub mod` exposes only the child identity; explicit `pub use` is required to flatten a declaration.
- imports of functions, types, constructors, interfaces, macros, and notation preserve defining identities and respect their phase boundaries.
- the Engine rejects missing module artifacts and never falls back to direct evaluation.
- one admitted multi-module `fn main` program yields the same terminal result through CLI and daemon.

## 12. Implementation and evidence status

**Implementation:** not implemented as a complete rule.

**Evidence:** none for the complete rule. Existing parser and graph tests cover bounded fragments only.

**Parity:** below spec. The current code accepts module syntax and has separate file-resolution, summary, import, and Engine paths, but it does not provide the unified parity-preserving realization required here.

**Plan:** PLAN-207 owns implementation. Its task records and the `MOD-REAL-*` rows in `SEMANTIC-RULE-COVERAGE.md` must track Type → Core → CPS → admission → runtime ownership separately.