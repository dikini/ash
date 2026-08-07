# TASK-2069: Complete Module Lowering and Engine Transport Fencing

**Status:** Complete for the frozen callable-module completion domain
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5, 8-11 (`M-LOWER`, `M-LINK`); SPEC-098c; SPEC-099b; PLAN-203
**Owned rule:** MOD-REAL-005
**Run-route impact:** prerequisite
**Semantic task record:** [semantic-task-records.json](../semantic-task-records.json); the activation adds
the task-owned coverage section and planned traceability nodes before any Rust change.
**Semantic coverage map:** [TASK-2069 record](../SEMANTIC-RULE-COVERAGE.md#task-2069-complete-module-lowering-and-engine-transport-fencing)

## Semantic accounting

**Implementation:** implemented
**Evidence:** tested
**Parity:** matches_spec
**Completion scope:** This task must complete lowering and transport for the frozen callable-module
route and the checked type metadata dependencies it needs. Role semantics; policy instances,
enforcement, persistence, inheritance, or authority; and runtime behavior for roles/policies are
out of scope. Role/policy transport is only a non-authorizing compatibility fence and is excluded
from completion criteria.
All public declarations consumed by that route must nevertheless propagate through the resolved
import closure with canonical identity, origin, visibility, and checked metadata intact. Only
callable declarations and the metadata required by their checked dependencies need lowering or
Engine transport; other public declarations remain importable non-authorizing metadata.
**Missing target-spec clauses:** None within the frozen ordinary-callable lowering and
non-authorizing transport domain. Bodyless `BuiltinFn` declarations are propagated as checked
signature/identity/visibility metadata only; host builtin dispatch/runtime semantics are outside
this phase. Raw synthesized-pattern compatibility APIs are explicitly outside the frozen
completion domain.
**Layers:** type `implemented`; Core `implemented`; CPS `implemented`;
admission-runtime `not_applicable` for this non-authorizing transport handoff; verification
`implemented`.
**Evidence identifiers:** positive `TEST-MOD-REAL-005-FULL-DEFINITION-BODY-LOWERING`,
`TEST-MOD-REAL-005-FULL-DEFINITION-BODY-CLOSURE`, and
`TEST-MOD-REAL-005-FINALIZED-BODY-AUTHORITY`, and
`TEST-MOD-REAL-005-PARAMETERIZED-BODY-LOWERING`, and
`TEST-MOD-REAL-005-IMPORTED-CALL-LOWERING`, and
`TEST-MOD-REAL-005-IMPORTED-BUILTIN-CALLABLE-TRANSPORT`, and
`TEST-MOD-REAL-005-HANDLER-BODY-LOWERING`, and
`TEST-MOD-REAL-005-PARENT-SCOPED-CALLABLE-FENCE`, and
`TEST-MOD-REAL-005-REACHABLE-DEPENDENCY-CLOSURE`, and
`TEST-MOD-REAL-005-SELECTED-ENTRY-CLOSURE`, and
`TEST-MOD-REAL-005-SELECTED-ENTRY-SELECTION-ATOMICITY`, and
`TEST-MOD-REAL-005-ROUTE-CLOSURE-METADATA-CARRIER`, and
`TEST-MOD-REAL-005-PRIMITIVE-EXPRESSION-LOWERING`, and
`TEST-MOD-REAL-005-MODULO-LOWERING`, and
`TEST-MOD-REAL-005-RECORD-FIELD-CALL-LOWERING`, and
`TEST-MOD-REAL-005-RECORD-FIELD-CALL-ENGINE-ROUTE`, and
`TEST-MOD-REAL-005-NESTED-RECORD-FIELD-CALL-LOWERING`, and
`TEST-MOD-REAL-005-NESTED-RECORD-FIELD-CALL-ENGINE-ROUTE`, and
`TEST-MOD-REAL-005-RECORD-FIELD-EXPRESSION-CALL-LOWERING`, and
`TEST-MOD-REAL-005-RECORD-FIELD-EXPRESSION-CALL-ENGINE-ROUTE`, and
`TEST-MOD-REAL-005-DECLARATION-ORDER-LOWERING`, and
`TEST-MOD-REAL-005-DECLARATION-ORDER-ENGINE-ROUTE`, and
`TEST-MOD-REAL-005-LET-EXPRESSION-LOWERING`, and
`TEST-MOD-REAL-005-STRUCTURAL-FIELD-LOWERING`, and
`TEST-MOD-REAL-005-MATCH-SCRUTINE-CALL-LOWERING`, and
`TEST-MOD-REAL-005-IF-LET-LOWERING`, and
`TEST-MOD-REAL-005-SHORT-CIRCUIT-BOOLEAN-LOWERING`, and
`TEST-MOD-REAL-005-SHORT-CIRCUIT-BOOLEAN-LET-LOWERING`, and
`TEST-MOD-REAL-005-SHORT-CIRCUIT-BOOLEAN-IF-LET-LOWERING`, and
`TEST-MOD-REAL-005-SHORT-CIRCUIT-BOOLEAN-ARGUMENT-LOWERING`, and
`TEST-MOD-REAL-005-SHORT-CIRCUIT-BOOLEAN-MATCH-ARGUMENT-LOWERING`, and
`TEST-MOD-REAL-005-SHORT-CIRCUIT-NESTED-ARGUMENT-LOWERING`, and
`TEST-MOD-REAL-005-SHORT-CIRCUIT-RECORD-LOWERING`, and
`TEST-MOD-REAL-005-NESTED-SHORT-CIRCUIT-BOOLEAN-LOWERING`, and
`TEST-MOD-REAL-005-IMPORT-TRANSPORT`, and
`TEST-MOD-REAL-005-REPRESENTABLE-IMPORT-TRANSPORT`, and
`TEST-MOD-REAL-005-CONSTRUCTOR-IMPORT-TRANSPORT`, and
`TEST-MOD-REAL-005-PARENT-SCOPED-VISIBILITY-IMPORT-TRANSPORT`, and
`TEST-MOD-REAL-005-PUBLIC-CALLABLE-REEXPORT-TRANSPORT`, and
`TEST-MOD-REAL-005-STRUCTURAL-REEXPORT-IDENTITY`, and
`TEST-MOD-REAL-005-REEXPORT-VISIBILITY-TRANSPORT`, and
`TEST-MOD-REAL-005-PUBLISHED-STRUCTURAL-CHILD-IMPORT`, and
`TEST-MOD-REAL-005-TYPE-FUNCTION-IMPORT-TRANSPORT`, and
`TEST-MOD-REAL-005-ROLE-POLICY-METADATA-STUB-TRANSPORT` (compatibility-only
non-authorizing metadata evidence, not a completion criterion), and
`TEST-MOD-REAL-005-IMPLEMENTATION-METADATA-TRANSPORT`, and
`TEST-MOD-REAL-005-BODYLESS-CONSTRUCTOR-FENCE`, and
`TEST-MOD-REAL-005-CHECKED-INTERFACE-CLOSURE`, and
`TEST-MOD-REAL-005-TYPED-INTERFACE-IDENTITY`, and
`TEST-MOD-REAL-005-LOWERED-CLOSURE-HANDOFF`, and
`TEST-MOD-REAL-005-ENGINE-CANONICAL-SOURCE-ROUTE`, and
`TEST-MOD-REAL-005-ORDINARY-ROOT-CANONICAL-ROUTE`, and
`TEST-MOD-REAL-005-NEUTRAL-NONROOT-SELECTION`, and
`TEST-MOD-REAL-005-SUPPLIED-ROOT-SOURCE-AUTHORITY`, and
`TEST-MOD-REAL-005-ENGINE-CHECKED-TRANSPORT`, and
`TEST-MOD-REAL-005-SAME-MODULE-LOCAL-CALL-CLOSURE`; negative
`TEST-MOD-REAL-005-BODY-LOWERING-REJECTION`,
`TEST-MOD-REAL-005-PARSEABLE-CALLABLE-LOWERING-FAIL-CLOSED`, and
`TEST-MOD-REAL-005-PARSEABLE-CALLABLE-IMPORT-FAIL-CLOSED`, and
`TEST-MOD-REAL-005-SELECTED-ENTRY-PARENT-SCOPED-REJECTION`,
`TEST-MOD-REAL-005-UNPUBLISHED-STRUCTURAL-CHILD-IMPORT`, and
`TEST-MOD-REAL-005-SINGLE-BODY-PARENT-SCOPED-REJECTION`, and
`TEST-MOD-REAL-005-SELECTED-ENTRY-CALLABLE-CYCLE-FENCE`; mutation
`TEST-MOD-REAL-005-PROVENANCE-REWRITE` and
`TEST-MOD-REAL-005-CLOSURE-ATOMICITY` and
`TEST-MOD-REAL-005-EXTRA-INTERFACE-CLOSURE-REJECTION` and
`TEST-MOD-REAL-005-SCANNER-AUTHORITY-REJECTION` and
`TEST-MOD-REAL-005-LOCAL-IMPORT-NAMESPACE-KIND-FENCE` and
`TEST-MOD-REAL-005-REEXPORT-DEFINING-IDENTITY-FENCE` and
`TEST-MOD-REAL-005-PUBLIC-FUNCTION-COUNT-AUTHORITY` and
`TEST-MOD-REAL-005-UNDECLARED-IMPORT-DEPENDENCY-FENCE` and
`TEST-MOD-REAL-005-UNDECLARED-EXPORT-DEPENDENCY-FENCE` and
`TEST-MOD-REAL-005-PARSER-IMPORT-PREAMBLE` and
`TEST-MOD-REAL-005-MODULE-PARSE-FAIL-CLOSED` and
`TEST-MOD-REAL-005-PARSER-FAILURE-FALLBACK-AUTHORITY` and
`TEST-MOD-REAL-005-BUILTIN-STDLIB-COMPATIBILITY`,
`TEST-MOD-REAL-005-LEGACY-ENTRY-PRELUDE-COMPATIBILITY`, and
`TEST-MOD-REAL-005-ORDINARY-MISSING-ENTRY-COMPATIBILITY`; layer-parity
`TEST-MOD-REAL-005-FILE-INLINE-LOWERING-PARITY`; cache fence
`TEST-MOD-REAL-005-CANONICAL-CACHE-KEY` and
`TEST-MOD-REAL-005-LEGACY-PATH-CACHE-KEY`. The Engine transport identifier and checked-lowering
closure handoff are now tested by the focused 17/17 carrier/scanner target; an additional
module-transport unit witness rejects a forged non-callable namespace kind on a same-module
callable identity; the focused lowerer target
now passes 48/48, including primitive and modulo
arithmetic, local literal `let` bindings, representable type-import and type-function projection, bodyless constructor transport, explicit root-entry plus metadata-only child transport selection, and bounded
file/inline normalized Core/CPS parity, bodyless imported builtin callable transport, short-circuit
boolean values in `let` initializers, nested short-circuit boolean operands, and bounded
one-clause source-handler Core/CPS lowering, and transitive reachable dependency snapshots. Public function counting and the root import/module/export
walk now use expanded/parser-owned AST data; compatibility import parsing remains restricted to
unrepresentable versioned or parser-failure inputs. Normal metadata parsing now consumes the
complete parser-owned module file first; the metadata stripper remains only as an explicit
parser-failure fallback. Regressions prove nested inline public callables and visibility re-exports
cannot be flattened into parent facts. The synthesized-runner metadata preprocessor now preserves
parser-owned module structure, while raw pattern compatibility APIs remain deferred and
non-authorizing. Parser-failure metadata fallback is explicitly marked non-authorizing:
ordinary loading rejects fallback export records before binding, and equivalent display-path
cache hits cannot elevate them. The only ordinary-loader exception is the configured built-in
stdlib, plus byte-identical copies of its files for compatibility layouts. The parser-backed
entry-prelude and missing-entry witnesses also preserve the established bare entry diagnostics
and sealed provider/handler slices; these records remain
non-authorizing and cannot enter the canonical module closure. The remaining compatibility cache now
canonicalizes equivalent display paths and retains no duplicate path-keyed semantic records. The
canonical checked-transport cache now rejects duplicate roots and never uses source paths as identity. The
provenance-rewrite mutation is now covered by pairing finalization facts with a different expanded
source origin and rejecting before artifact creation.
The first production slices now cover a finalized checker-owned ordinary-function body and an
atomic per-function closure carrier, transports representable callable/structural resolved-import
identity and origin facts into Core/CPS, and validation of a canonical-keyed Engine closure for
interface/Core/CPS identity, schema, structural/dependency completeness, failed entries, import
and export identity/origin, duplicate keys, reachability, and deterministic ordering. These
carriers now retain the checked public-interface schema and reachable dependency snapshot through
Core and CPS, remain non-authorizing, and still leave remaining namespace/dependency wiring,
compatibility import/export fallback beyond the ordinary-loader fence, raw synthesized-pattern
compatibility APIs, and parity deferred. A
checked public interface-closure projection now
derives canonical artifacts, finalized public exports, parsed-import dependency identities, and
checked typed identities atomically. Typed identities link only `Type`, `Constructor`, `Interface`,
and `EffectRow` bindings; roles and policies remain metadata-only generic stubs. Metadata-only
modules now receive a neutral checked carrier without a selected callable entry, while namespaces
without a lossless Core interface binding remain explicit rejection cases. Public implementation summaries
now use a generic, namespace-separated Core interface binding with defining identity and origin;
implementation members remain parent-scoped, non-callable, and non-authorizing. Role and policy
declarations use dedicated generic metadata bindings in the Core/CPS carrier; they remain
non-callable and carry no typed identity, persistence, admission, or runtime authority. It now retains finalized ordinary-callable
parameter names and checked signatures for Core value environments, lowers resolved imported
callable applications through the same checked Core/CPS bridge (including call operands nested
inside a checked primitive expression), and exposes an explicit selected-entry
closure handoff so one canonical module key cannot receive multiple transport artifacts. Complete
definition lowering now also carries the selected entry and private sibling callables as local
closure entries, so
same-module aliases resolve through the Engine-owned linker without manufacturing a dependency edge
or publishing private bindings.
The route-specific lowering handoff now retains every checked callable body as a local
non-authorizing entry and adds a neutral carrier for an unselected structural child, so Engine no
longer reconstructs that selection policy locally; it preflights the complete selection map before
lowering any body.
The lowerer now accepts bodyless imported builtins as callable metadata and carries checker-retained
facts for a bounded one-clause handler through checked Core `Handle`/CPS lowering; this does not
install provider or runtime authority. Resolved imports now also retain exact source visibility
classes as non-authorizing metadata, and Engine transport admits only same-crate `pub(crate)`,
parent/descendant `pub(super)`, or canonical `pub(in crate...)` callable artifacts that are
present in the checked local closure. Non-root selected callable bodies are validated with their
finalized parameter bindings before linking, while the root remains a closed term;
public interface exports remain public-only and metadata namespaces remain excluded.
**Non-goals:** Engine-sealed linked admission, runtime execution, policy persistence or authority, filesystem/text-scan authority, source rediscovery, direct-evaluator fallback, dynamic imports, runtime module values, or CLI/daemon parity.
**Next obligation:** None within Phase 207. The non-authorizing carrier remains separate from
TASK-2063 sealing/admission; raw synthesized-pattern helpers remain compatibility-only deferred
rows and are not part of the frozen module domain.

The implementation narrative below is retained as historical slice evidence. Statements that
remaining namespace/dependency wiring, parity, or compatibility transport were deferred describe
the pre-closeout handoff boundary; they do not reopen TASK-2069 within the frozen domain.

**Historical activation checkpoint:** TASK-2069 was the active prerequisite owner for MOD-REAL-005.
The current task status is complete for the frozen domain. The
first implementation slice is a red complete-body lowering/transport contract; this activation
does not authorize Engine admission, runtime execution, policy persistence, or a direct evaluator.
The focused `task_2069_complete_module_lowering` target now passes 48/48: activation, positive
provenance-preserving body lowering, finalized-body authority, per-function closure lowering,
selected root-entry closure lowering, metadata-only child closure transport, and missing-selection atomicity,
parent-scoped selected-entry rejection,
primitive arithmetic, local literal `let` lowering, structural record field projection, callable match scrutinees, exact selected-entry key closure, representable type-import and type-function transport, public callable re-export transport, bodyless imported builtin callable transport, bounded one-clause handler Core/CPS lowering, transitive reachable dependency snapshots, file/inline normalized closure parity, resolved-import identity/origin
transport, checked public-interface closure projection, atomic closure rejection, unsupported-body rejection,
parent-scoped implementation members remaining outside standalone entry lowering,
unsupported-import rejection, missing-definition rejection, and provenance-rewrite rejection.
Definition-backed Engine closures also resolve multiple imported callable identities from a
provider's checked per-function local entries instead of requiring every import to equal the
provider's selected standalone entry.
The route-specific lowerer now supplies the neutral carrier for an unselected structural child
alongside those local callable bodies, keeping the selection/transport seam in the checked
handoff.
The entry-oriented closure handoff also retains its selected callable as a local checker-lowered
entry, so same-module aliases cannot be mistaken for missing public exports and reach the Engine
callable-cycle fence.
The Engine unit route witness now executes both file-backed and inline structural children through
`canonical_module_closure_from_source` → checked admission → checked CPS, rather than constructing
the linked closure manually.
Structural children containing only checked handlers now retain those handler bodies as local
non-authorizing callable entries alongside a neutral non-selected module carrier; only the root
requires a selected executable entry.
Parseable ordinary roots without structural children also traverse the same canonical source,
finalization, Core/CPS, Engine-linked, and admission route instead of returning to the legacy
single-entry path.
The focused `task_2069_module_transport_fencing` target passes 17/17 for the canonical Engine
transport/cache slice, including rejection of imported and externally re-exported targets absent
from the owning interface's declared dependency snapshot, public structural-child import identity
and visibility fencing, cyclic dependency snapshots, expanded-AST function-count fencing, and
parser-owned import preamble. It also accepts same-module and cross-module imports of public
non-callable role/policy metadata bindings when the checked public interface carries those
identities; restricted visibility exceptions remain callable-only.
Root import/export structure, visibility re-export readers, imported-interface readers, and normal
metadata parsing are now AST-driven; parser-failure metadata fallback remains available only to
compatibility inspection and is rejected before ordinary-loader binding, while raw synthesized-
pattern compatibility APIs and remaining parity witnesses remain deferred.

## Delivered parser-owned authority fence

The Engine loader and synthesized metadata reader now use the complete parser-owned module result for normal metadata collection,
root public functions, builtin functions, capabilities, child-module declarations, public-use
resolution, visibility/import diagnostics, and synthesized-runner metadata parsing. For these
audited readers, raw source snippets remain reachable only after
authoritative parsing fails, including the explicitly compatibility-only versioned-import route.
They do not override parser-owned facts. The focused regressions
`task_2069_nested_inline_public_callable_is_not_flattened_by_source_scan` and
`task_2069_visibility_reexport_reader_ignores_nested_inline_pub_use`, and
`synthesized_metadata_parser_preserves_parser_owned_inline_module_structure` prove that nested
inline facts do not enter parent export/visibility sets and that the runner retains module
structure. Existing versioned-import compatibility tests remain green. Roles and policies remain
metadata-only stubs and gain no authority, persistence, admission, or runtime behavior; the
focused lowerer witness only verifies that these remnants do not block unrelated callable
transport.

## Description

Replace TASK-2062's deliberately bounded, already-materialized-Core envelope with lowering of the
complete TASK-2073 checked module definition bodies. Carry canonical module/declaration identity,
origin, visibility-resolved import facts, and dependency versions through Core and CPS. At the
Engine boundary, retire a scanner where possible or fence it as an AST-agreement-only,
fail-closed, non-authorizing compatibility check, and move semantic cache identity from paths and
strings to canonical checked artifact keys. This task transports artifacts to TASK-2063; it does
not seal, admit, or execute them.

## Dependencies

- ✅ TASK-2067 — canonical structural graph and real acquired module units.
- ✅ TASK-2073 — complete checked interfaces, definition bodies, resolved bindings, and export
  closure. TASK-2070/2071/2074/2075/2072 are its separately owned prerequisites, not lowering
  authority.
- ✅ TASK-2062 — bounded provenance-carrier lessons and the existing checked Core-to-CPS bridge;
  its public carrier is not sufficient input for this task or for Engine admission.

## Requirements

1. Lower every reachable supported definition body from TASK-2073's checked module facts rather
   than from caller-materialized `RawCoreProgram`, source rediscovery, a legacy graph, raw public
   interface, or Engine loader text. Unsupported target forms must reject at the checked/lowering
   boundary; no fallback evaluator may be selected.
2. Produce per-module Core and CPS artifacts through the selected checked lowering bridges while
   retaining exact `ModuleKey`, source origin, final-interface schema/dependency version, resolved
   binding defining identity/origin, and entry/dependency closure facts needed by TASK-2063.
3. Make equivalent file/inline checked modules produce equal normalized Core/CPS artifacts. Source
   form may affect diagnostic/source provenance only; it may not select a different lowering or
   transport route.
4. Replace each audited Engine-side semantic input—leading import prelude, metadata stripping,
   source export/import snippets, `collect_module_exports`, and path/string-keyed module
   cache/walking—with the checked artifact transport. If immediate removal is impossible, the
   compatibility reader must compare only against parsed/checked data, fail closed on disagreement,
   remain explicitly denylisted, and have no authority to publish graph, binding, interface,
   lowering, admission, or execution facts.
5. Apply the same non-authority fence to the synthesized-runner metadata preprocessor while it
   remains reachable: it must consume or compare the canonical module-unit/interface carrier and
   remain introspection-only. This is transport hardening, not a new CLI runtime route.
6. Key Engine module caches and transport requests by canonical checked artifact identity, never a
   filesystem/path string. A renamed/display-path-equivalent source must not create a distinct
   semantic module artifact, and a forged key/version/origin must reject before TASK-2063.
7. Hand TASK-2063 one complete, non-sealed checked Core/CPS dependency closure. Do not mint an
   admission token, provider/handler frame, executable request, or client terminal result.

## TDD steps and reserved evidence

1. Add failing full-body lowering tests over checked multi-module definitions and all resolved
   imports; verify the produced Core/CPS closure carries exact identity/origin/version facts
   (`TEST-MOD-REAL-005-FULL-DEFINITION-BODY-LOWERING`,
   `TEST-MOD-REAL-005-PARAMETERIZED-BODY-LOWERING`,
   `TEST-MOD-REAL-005-IMPORTED-CALL-LOWERING`,
   `TEST-MOD-REAL-005-SELECTED-ENTRY-CLOSURE`,
   `TEST-MOD-REAL-005-SELECTED-ENTRY-SELECTION-ATOMICITY`,
   `TEST-MOD-REAL-005-ROUTE-CLOSURE-METADATA-CARRIER`,
   `TEST-MOD-REAL-005-SELECTED-ENTRY-PARENT-SCOPED-REJECTION`,
   `TEST-MOD-REAL-005-SINGLE-BODY-PARENT-SCOPED-REJECTION`,
   `TEST-MOD-REAL-005-PRIMITIVE-EXPRESSION-LOWERING`,
   `TEST-MOD-REAL-005-LET-EXPRESSION-LOWERING`,
   `TEST-MOD-REAL-005-STRUCTURAL-FIELD-LOWERING`,
   `TEST-MOD-REAL-005-MATCH-SCRUTINE-CALL-LOWERING`,
   `TEST-MOD-REAL-005-REPRESENTABLE-IMPORT-TRANSPORT`,
   `TEST-MOD-REAL-005-CONSTRUCTOR-IMPORT-TRANSPORT`,
   `TEST-MOD-REAL-005-PARENT-SCOPED-VISIBILITY-IMPORT-TRANSPORT`,
   `TEST-MOD-REAL-005-PUBLIC-CALLABLE-REEXPORT-TRANSPORT`,
   `TEST-MOD-REAL-005-CHECKED-INTERFACE-CLOSURE`,
   `TEST-MOD-REAL-005-TYPED-INTERFACE-IDENTITY`).
2. Add a negative incomplete/unsupported/failed-definition case and a provenance-rewrite mutation;
   assert failure before a Core/CPS artifact or Engine transport request can publish
   (`TEST-MOD-REAL-005-BODY-LOWERING-REJECTION`,
   `TEST-MOD-REAL-005-PROVENANCE-REWRITE`).
3. Add paired file/inline checked trees and compare normalized Core/CPS artifact closures
   (`TEST-MOD-REAL-005-FILE-INLINE-LOWERING-PARITY`).
4. Add scanner and cache mutations that inject a text-only export/import fact, disagreement, path
   substitution, or forged cache key. Assert all fenced readers reject or remain
   non-authorizing and that only canonical checked artifacts reach the Engine boundary
   (`TEST-MOD-REAL-005-SCANNER-AUTHORITY-REJECTION`,
   `TEST-MOD-REAL-005-LOCAL-IMPORT-NAMESPACE-KIND-FENCE`,
   `TEST-MOD-REAL-005-VISIBILITY-IMPORT-FENCE`,
   `TEST-MOD-REAL-005-PUBLISHED-STRUCTURAL-CHILD-IMPORT`,
   `TEST-MOD-REAL-005-UNPUBLISHED-STRUCTURAL-CHILD-IMPORT`,
   `TEST-MOD-REAL-005-SYNTHESIZED-METADATA-FENCE`,
   `TEST-MOD-REAL-005-PARSER-IMPORT-PREAMBLE`,
   `TEST-MOD-REAL-005-MODULE-PARSE-FAIL-CLOSED`,
   `TEST-MOD-REAL-005-PARSER-FAILURE-FALLBACK-AUTHORITY`,
   `TEST-MOD-REAL-005-CANONICAL-CACHE-KEY`,
   `TEST-MOD-REAL-005-LEGACY-PATH-CACHE-KEY`).
   Normal metadata collection must use the parser-owned module result; the stripper may remain only
   as a denylisted parser-failure compatibility fallback.
5. Implement only after the focused tests are red, then run focused Core/typechecker/Engine tests,
   affected crate suites, strict clippy, and formatting. TASK-2063 tests begin only after this
   complete closure transport exists.

## Completion checklist

- [x] Complete checked definition bodies lower to provenance-preserving Core and CPS artifacts
  without source rediscovery or caller-materialized Core authority.
- [x] Equivalent file/inline modules have equal normalized Core/CPS artifact closures.
- [x] Every audited Engine/synthesized-runner scanner is removed or fenced fail-closed and
  non-authorizing, and path/string cache identity is retired from the semantic transport route.
- [x] TASK-2063 receives one complete, canonical-keyed, non-sealed dependency closure with
  positive, negative, mutation, scanner-fence, cache-fence, and layer-parity evidence recorded in
  the activated task record and traceability graph.
- [x] No transport carrier admits/executes a module, creates provider/handler authority, or permits
  a direct-evaluator fallback.

## Handoffs

- **Consumes:** TASK-2073 complete checked module/interface/export-closure facts, after TASK-2074
  expansion, TASK-2075 collection, and TASK-2072 binding, plus TASK-2067 canonical
  source/unit/graph provenance. TASK-2062's bounded artifacts are comparison/migration evidence,
  not authority.
- **Produces:** complete reachable checked Core/CPS artifact closures and Engine transport/cache
  facts keyed by canonical identity. The transport is expressly non-sealed and non-authorizing.
- **Downstream owner:** TASK-2063 validates the closure again, mints the separate Engine-sealed
  linked/admission request, and rejects all incomplete/stale/forged/failed artifacts. TASK-2064
  alone compares one admitted real program through CLI and daemon.
- **Integration/proof responsibility:** TASK-2069 owns source-to-Core-to-CPS and scanner/cache
  fence evidence. TASK-2063 owns link/admission rejection evidence; TASK-2064 owns final
  file/inline and client normalized-terminal parity.
- **Run-route impact:** `prerequisite`. It removes/fences alternate semantic inputs but cannot
  activate an Engine or client route before TASK-2063 seals a request.
- **Non-goals:** New language syntax, parser/source acquisition, interface/binder semantics,
  dynamic imports/packages, runtime module values, import-cycle initialization, Engine linking or
  admission, execution, provider/handler frame authority, direct evaluation, or CLI/daemon
  terminal parity.

## Candidate files and verification

**Candidate source/test paths on activation:** `crates/ash-typeck/src/module_core_cps_lowering.rs`,
`crates/ash-core/src/module_lowering.rs`, `crates/ash-engine/src/{module_loader.rs,entry.rs}`,
Engine cache/transport modules, and focused Core/typechecker/Engine integration tests.

```text
cargo test -p ash-typeck --test task_2069_complete_module_lowering
cargo test -p ash-engine --test task_2069_module_transport_fencing
cargo test -p ash-core
cargo test -p ash-typeck
cargo test -p ash-engine
cargo clippy -p ash-core -p ash-typeck -p ash-engine --all-targets -- -D warnings
cargo fmt --check
git diff --check
```
