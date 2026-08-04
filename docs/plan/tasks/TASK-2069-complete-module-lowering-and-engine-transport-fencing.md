# TASK-2069: Complete Module Lowering and Engine Transport Fencing

**Status:** Planned
**Phase:** [PLAN-207](../PLAN-207-COMPLETE-MODULE-REALIZATION.md)
**Spec:** SPEC-103 §§5, 8-11 (`M-LOWER`, `M-LINK`); SPEC-098c; SPEC-099b; PLAN-203
**Owned rule:** MOD-REAL-005
**Run-route impact:** prerequisite
**Semantic task record:** None while planned. Promote this task to **In progress** and add its
record, coverage section, and traceability implementation/evidence nodes in the same activation
change; this planning record authorizes no Rust change.
**Semantic coverage map:** [Complete modules, imports, and visibility](../SEMANTIC-RULE-COVERAGE.md#complete-modules-imports-and-visibility)

## Semantic accounting

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** Complete checked definition bodies do not yet lower from a final
module interface through identity/origin-preserving Core and CPS artifacts. Engine module loaders,
export/import scanners, and path/string-keyed caches still have unfenced semantic seams instead of
receiving canonical checked artifact transport for TASK-2063 to seal and admit.
**Layers:** type `not_applicable`; Core `not_implemented`; CPS `not_implemented`;
admission-runtime `not_implemented` for non-authorizing Engine transport; verification
`not_implemented`.
**Evidence identifiers:** positive `TEST-MOD-REAL-005-FULL-DEFINITION-BODY-LOWERING` and
`TEST-MOD-REAL-005-ENGINE-CHECKED-TRANSPORT`; negative
`TEST-MOD-REAL-005-BODY-LOWERING-REJECTION`; mutation
`TEST-MOD-REAL-005-PROVENANCE-REWRITE` and
`TEST-MOD-REAL-005-SCANNER-AUTHORITY-REJECTION`; layer-parity
`TEST-MOD-REAL-005-FILE-INLINE-LOWERING-PARITY`; cache fence
`TEST-MOD-REAL-005-CANONICAL-CACHE-KEY`. These identifiers reserve required future evidence; no
test or proof exists yet.
**Next obligation:** TASK-2063 consumes the complete reachable checked Core/CPS closure and
canonical Engine transport handoff, then independently seals linking/admission. TASK-2064 owns
admitted-program and CLI/daemon terminal parity.

## Description

Replace TASK-2062's deliberately bounded, already-materialized-Core envelope with lowering of the
complete TASK-2073 checked module definition bodies. Carry canonical module/declaration identity,
origin, visibility-resolved import facts, and dependency versions through Core and CPS. At the
Engine boundary, retire a scanner where possible or fence it as an AST-agreement-only,
fail-closed, non-authorizing compatibility check, and move semantic cache identity from paths and
strings to canonical checked artifact keys. This task transports artifacts to TASK-2063; it does
not seal, admit, or execute them.

## Dependencies

- 📝 TASK-2067 — canonical structural graph and real acquired module units.
- 📝 TASK-2073 — complete checked interfaces, definition bodies, resolved bindings, and export
  closure. TASK-2070/2071/2072 are its separately owned prerequisites, not lowering authority.
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
   (`TEST-MOD-REAL-005-FULL-DEFINITION-BODY-LOWERING`).
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
   `TEST-MOD-REAL-005-CANONICAL-CACHE-KEY`).
5. Implement only after the focused tests are red, then run focused Core/typechecker/Engine tests,
   affected crate suites, strict clippy, and formatting. TASK-2063 tests begin only after this
   complete closure transport exists.

## Completion checklist

- [ ] Complete checked definition bodies lower to provenance-preserving Core and CPS artifacts
  without source rediscovery or caller-materialized Core authority.
- [ ] Equivalent file/inline modules have equal normalized Core/CPS artifact closures.
- [ ] Every audited Engine/synthesized-runner scanner is removed or fenced fail-closed and
  non-authorizing, and path/string cache identity is retired from the semantic transport route.
- [ ] TASK-2063 receives one complete, canonical-keyed, non-sealed dependency closure with
  positive, negative, mutation, scanner-fence, cache-fence, and layer-parity evidence recorded in
  the activated task record and traceability graph.
- [ ] No transport carrier admits/executes a module, creates provider/handler authority, or permits
  a direct-evaluator fallback.

## Handoffs

- **Consumes:** TASK-2073 complete checked module/interface/export-closure facts and TASK-2067 canonical
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
