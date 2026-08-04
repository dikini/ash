---
id: spec.ash.module-realization
title: Ash Module Realization and Operational Semantics
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-08-04
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
| MOD-REAL-001 | Parsed declarations are the only authority for structural module edges and child identities. | TASK-2057, TASK-2058, TASK-2071, TASK-2074 |
| MOD-REAL-002 | File-backed and inline children become equivalent module units after source acquisition. | TASK-2059, TASK-2071, TASK-2074, TASK-2075 |
| MOD-REAL-003 | Checked public interfaces are export-closed and preserve defining identities. | TASK-2060, TASK-2071, TASK-2073 |
| MOD-REAL-004 | Imports, qualified paths, and visibility resolve only through canonical provisional name views and checked interfaces. | TASK-2061, TASK-2070, TASK-2071, TASK-2072, TASK-2075 |
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

**Expanded module graph** is the parser-owned `CanonicalExpandedModuleGraph`. It consumes and owns
one `CanonicalModuleGraph`, contains exactly one shallowly expanded `ModuleBody` for every canonical
`ModuleKey`, and retains the parsed uses, source order, source anchors, and per-module expansion
origin and hygiene sidecars. It is not the Engine module loader and cannot discover source, paths,
or declarations from text.

**Collected module snapshot** is the checker-internal `CanonicalCollectedModuleSnapshot`. It may
retain expanded raw declaration and callable shapes, bodies and member spans, per-module expansion
origins and hygiene, and source ordinals. It contains no checked types or body-checking results.

**Provisional name view** is the import-facing `CanonicalProvisionalNameView`. Each entry contains
only a lookup key and visible local name, defining identity and `ModuleKey`, namespace kind,
declared visibility and exportability, origin/source anchor, and source ordinal. It contains no
signature, callable shape, body, checked type, equation, final export, or runtime-authority fact.

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
  -> collect AST-only public macro/notation summaries and syntax-import dependencies
  -> topologically expand syntax providers before consumers
  -> construct one canonical expanded module graph
  -> collect internal declaration snapshots and minimal provisional name views
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

### Syntax-only prepass and canonical expansion

Expansion performs a syntax-only parsed prepass before `M-EXPAND`. The prepass gathers public macro
and notation summaries directly from the authoritative module AST, resolves only syntax imports by
canonical `ModuleKey` and parsed `Use` spans, rejects syntax-dependency cycles, and topologically
orders providers before consumers. It creates neither general import bindings nor runtime
authority. It may not use filesystem lookup, path/source-text fallback, the Engine module loader,
or Engine path caches.

An imported notation is eligible only when the provider exposes a canonical notation summary. It
remains inactive when no such summary exists; no source spelling or loader registration may
manufacture one. A notation import names one exact normalized token/hole pattern in a
parenthesized selector:

```ash
use crate::math::(<*>);
use crate::ranges::(_ between _ and _);
```

The selector follows SPEC-095c's nonempty typed `notation_selector` grammar: whitespace/comments
normalize between atoms, and `_` is a hole only as a complete atom. The selector does not encode fixity, associativity, or precedence. It transports every eligible
public provider summary with that pattern in deterministic full-key order, where the full key is
the normalized pattern plus fixity, associativity, and precedence. Declaration patterns and import
selectors must expose structured parsed token/hole parts; diagnostic raw spelling is not semantic
authority and must not be reparsed or scanned for matching. The transported summary retains the
target callable identity, provider `ModuleKey`, declaration provenance and visibility, and consumer
`Use` span. It neither binds the callable name nor authorizes callable, type, Core/CPS, admission,
Engine, or runtime behavior. An ordinary callable import never activates notation.

The provider exports notation directly by declaring the notation `pub`, which produces its
canonical public summary. Only an inherited-visibility `use module::(pattern)` is an import in this
contract. Notation re-export is not defined: `pub use module::(pattern)` and every other visibly
qualified notation use reject as unsupported rather than importing privately or republishing a
summary. A separate future contract must define and own re-export semantics.

Notation imports have neither an `as` form nor a notation glob. Missing, private, malformed,
conflicting, or cyclic notation dependencies reject the whole expanded graph atomically. Missing
and malformed selectors retain the consumer use anchor; private or conflicting summaries also
retain every applicable provider declaration anchor; cycle diagnostics retain ordered
provider/importer edges and their use spans. Eligible summaries are activated into the consumer's
existing syntax-phase notation table, whose overlap and use-site-context rules select a compatible
full-key variant or reject deterministically. This activation preserves hole order for downstream
resolution; it does not itself add generalized mixfix use-site parsing or elaboration.

Item-generating macros are unsupported by this realization. Expansion is shallow
per keyed module: direct definitions are expanded in their owning `ModuleBody`, parsed `use`
declarations and source order are retained, and an inline child's expansion sidecars appear only
under the child's canonical key. The `CanonicalExpandedModuleGraph` owns the parsed graph and has
an exact one-to-one module-key map. Any prepass, cycle, expansion, or invariant failure rejects the
whole result atomically.

## 6. Resolution and visibility

Let `I(m)` be the finalized interface of module `m`. A qualified module path resolves from a crate root through public child-module interface entries. A `use` declaration resolves only against a finalized interface or a provisional name-view entry; it does not walk filenames.

Before checking, imports resolve only against `CanonicalProvisionalNameView`, never against the
internal collected snapshot. A declaration's canonical identity key is:

```text
(ModuleKey, declaration kind, canonical parent, origin key)
```

Its lookup key is:

```text
(namespace bucket, visible local key)
```

Duplicates reject within a collision bucket. The minimum buckets are: structural module;
type/domain (`ResourceType`, `Type`, `Newtype`, and `SealedDomain` names); type computation;
promoted kind; value/callable/eligible constructors (`Function`, `Handler`, `BuiltinFn`, and
runtime constructors); interface; row name (`EffectAlias` and `EffectGroup`); proposition; macro;
notation; policy; role; implementation registry; and evidence. The same spelling may occur across
different buckets unless the referenced syntax context cannot select one bucket, in which case the
reference is ambiguous and rejects. Nested members collide only within their canonical parent.
Implementation coherence is decided from overlap of the full canonical interface application, not
from a local spelling.

`ModuleDecl` is structural. A macro's lookup key is its name. A notation's lookup key contains its
normalized pattern, fixity, associativity, and precedence and follows the notation-overlap rules. `Capability` is
removed target syntax and rejects during complete collection. Ordinary data and newtype
constructors become value entries when visible; sealed-domain constructors remain parent-scoped
and are not standalone values; promoted constructors remain parent-scoped and type-level.
Interface and implementation members are parent-scoped. Macro-generated identifiers are hygienic,
not source-spellable import keys.

`Policy` and `Role` occupy their own namespaces and may be named imports when visible. Module
`Law` and `Proof` declarations occupy the evidence namespace and may be imported only when
explicitly visible. `Impl` entries remain checker-internal and never enter the provisional name
view. `ResourceType` shares the type/domain collision bucket; `TypeFn` remains a distinct
type-computation bucket. Functions, handlers, builtins, and eligible value constructors share the
value/callable bucket. Policy, role, law, and proof collection requires the parser AST to retain a
declared visibility carrier; a missing carrier is an implementation prerequisite, never permission
to assume public or inherited visibility.

The complete parsed declaration domain maps as follows; collection is exhaustive and uses no
wildcard fallback:

| Parsed form | Collection decision |
|---|---|
| `ModuleDecl` | Structural-module bucket and canonical child identity. |
| `Notation` | Notation bucket; normalized pattern, fixity, and precedence key. |
| `Macro` | Macro bucket; named syntax summary with hygiene retained. |
| `Capability` | Reject as removed target syntax. |
| `ResourceType`, `Type`, `Newtype`, `SealedDomain` | Type/domain bucket; constructors follow the parent and promotion rules above. |
| `EffectAlias`, `EffectGroup` | Row-name bucket. |
| `DataKind` | Promoted-kind bucket. |
| `TypeFn` | Type-computation bucket. |
| `PropositionPredicate` | Proposition bucket. |
| `Policy` | Policy bucket; importable only with retained declared visibility. |
| `Role` | Role bucket; importable only with retained declared visibility. |
| `Interface` | Interface bucket; members are parent-scoped. |
| `Impl` | Implementation registry; checker-internal only. |
| `Function`, `Handler`, `BuiltinFn` | Value/callable bucket. |
| `Law`, `Proof` | Evidence bucket; importable only with retained explicit visibility. |

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
        | SyntaxReady(SyntaxSummary, ModuleFile, Source)
        | Expanded(ExpandedModule, Source)
        | Collected(CanonicalCollectedModuleSnapshot, CanonicalProvisionalNameView, ExpandedModule)
        | Bound(ResolvedModule, CanonicalCollectedModuleSnapshot)
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
syntax_prepass(G, parsed(S)) = (Y, order)     k ready in order
---------------------------------------------------------------- M-SYNTAX-PREPASS
<G, S, k::W, D> ->m <G, S[k := SyntaxReady(Y(k), P(k), src(k))], expand(k)::W, D>

S(k) = SyntaxReady(Y, P, src)     expand_module(k, P, Y) = E
------------------------------------------------------------ M-EXPAND
<G, S, k::W, D> ->m <G, S[k := Expanded(E, src)], collect(k)::W, D>

S(k) = Expanded(E, src)     collect(k, E) = (CS, PV)
---------------------------------------------------- M-COLLECT
<G, S, k::W, D> ->m <G, S[k := Collected(CS, PV, E)], bind(k)::W, D>

S(k) = Collected(CS, PV, E)     u = use-declaration(E)     target(k, u, PV, views(S)) = t
------------------------------------------------------------------------------------------- M-IMPORT-EDGE
<G, S, k::W, D> ->m <G + (k -import-> t), S, collect-imports(k)::W, D>

path(G + (k -import-> t), t, k) = c
----------------------------------- M-IMPORT-CYCLE
<G, S, k::W, D> ->m <G, S[k := Failed(import-cycle(c))], W, D + import-cycle(c)>

S(k) = Collected(CS, PV, E)     imports-resolved(k, G, views(S))
---------------------------------------------------------------- M-IMPORTS-READY
<G, S, k::W, D> ->m <G, S, bind(k)::W, D>

S(k) = Collected(CS, PV, E)     bind(k, G, views(S)) = R
---------------------------------------------------------- M-BIND
<G, S, k::W, D> ->m <G, S[k := Bound(R, CS)], check(k)::W, D>

S(k) = Bound(R, CS)     check(k, R, CS) = (C, I)
------------------------------------------------ M-CHECK
<G, S, k::W, D> ->m <G, S[k := Checked(C, I)], lower(k)::W, D>
```

`M-IMPORT-EDGE` traverses only expanded parsed `use` nodes. `target` resolves through structural
module identities and `CanonicalProvisionalNameView`; it never reads a filesystem path, source
text, callable shape, signature, body, equation, or checked type. The internal
`CanonicalCollectedModuleSnapshot` is passed separately to `M-CHECK` and cannot be used as import
authority. `CanonicalProvisionalModuleScopes` remains a compatibility projection for the bounded
TASK-2068/TASK-2070 routes, not the complete collector or a final interface. Complete revalidation
rejects name, kind, visibility, signature, body, order, or expansion-sidecar drift before either
collected view publishes. `M-IMPORT-CYCLE` fails the entire dependency
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
