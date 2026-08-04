# TASK-2068 Canonical Provisional Module Scopes Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Add one bounded, generic Type-layer import route that resolves inherited simple
`crate::` structural paths through canonical provisional module scopes and checks structural-path
visibility before atomically staging a local alias.

**Architecture:** The type checker derives immutable, typeck-owned per-module provisional scopes
only from TASK-2067 canonical graph units and artifacts. Each scope contains direct structural
children and ordinary function declarations, retaining canonical `ModuleKey`, declaration/use
spans, origins, and visibility. The existing simple parsed-import planner adapts the one admitted
source form to a structural-edge walk over those scopes. A segment-based visibility resolver
preflights every child edge and the target function before a temporary alias is staged. The output
is opaque and non-authorizing: it is neither a final interface nor a generic export/binder result.

**Tech Stack:** Rust 2024; `ash-core`, `ash-parser`, `ash-typeck`; `proptest`; repository
semantic-accounting validators.

---

## Scope and semantic boundary

The authority is SPEC-103 §§3, 5, 6, 8, and 9: canonical module identity and structure, scoped
visibility, local aliases that retain the definition identity, `M-IMPORT-EDGE`/`M-BIND`, and
failure atomicity. This reservation is only `partial / none / below_spec`. It makes Type-layer
scope and visibility facts available for a deliberately narrow simple-import route; Core, CPS,
admission, Engine, runtime, and parity are not part of this work.

The only admitted form is an inherited ordinary import of an ordinary function through existing
structural child modules:

```ash
mod api {
    pub mod text {
        pub fn normalize(value: Int) -> Int { value }
    }
}

use crate::api::text::normalize as normalize_text;

fn entry(value: Int) -> Int { normalize_text(value) }
```

The path is resolved edge by edge as `crate` followed by canonical direct structural children and
a final ordinary function declaration. It must not be accepted because a display string happens to
match. The binding preserves the target function's defining `ModuleKey` and declaration identity;
the importer receives an alias, not a flattened declaration or final public interface.

The provisional scope representation is built solely from TASK-2067 graph units/artifacts. It
contains no file-name lookup, source-text traversal, synthetic members, re-export entries, or
finalized namespace closure. The scope of module `M` has only the graph's direct structural
children of `M` and `M`'s ordinary function declaration entries. Its constructor and contents are
kept typeck-owned and immutable after construction.

Visibility is decided with canonical `ModuleKey` crate identity and structural segments, never the
existing string visibility helper. Given defining module `D`, requesting/importing module `R`, and
where needed a canonical module `P` resolved from a `pub(in P)` path:

- inherited/private and `pub(self)` admit exactly `R == D`;
- `pub(crate)` admits only a `R` with the same canonical crate identity as `D`;
- `pub(super)` admits `D`'s structural parent and every descendant of that parent in the same
  crate, using a segment-prefix relation;
- `pub(in P)` admits `P` and every descendant of `P` in the same crate after resolving `P` to its
  canonical key and verifying the specification's allowed structural relationship; and
- `pub` admits only when the final function is public and every canonical structural edge from the
  crate root to its defining module is public. No hidden or restricted child may be bypassed.

The resolver must apply that predicate to every traversed child and then to the final function
before any alias is staged. A local ordinary function named like the requested alias is a
declaration collision and rejects the import with a declaration/use diagnostic. The local
collision rule does not authorize wider namespace conflict handling.

Out of scope: `pub use`; grouped, glob, relative, external, non-`crate`, malformed, or implicit
paths; targets other than ordinary functions; values/types/macros and every other namespace;
definition/body checking; re-exports and final interface/export closure; compatibility binders;
general import-cycle realization; all unselected import/visibility clauses; Core/CPS; Engine,
admission, runtime, CLI/daemon, and file/inline integration parity. Existing explicit-public-use
rejection remains intact. TASK-2069 cannot begin until TASK-2068 is complete. No commit is
authorized by this plan or its future implementation.

## TDD implementation tasks

### Task 1: Create the red canonical-scope contract

**Files:**

- Create: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_graph.rs`
- Inspect: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Inspect: `crates/ash-typeck/src/canonical_module_binder.rs`
- Inspect: `crates/ash-core/src/module_graph.rs`

1. Add a positive graph-derived structural-path fixture for inherited
   `use crate::<child>...::<function> as <name>`. Assert the staged alias retains the final
   function's defining `ModuleKey`, declaration identity, visibility/origin, and use span, and
   that each intermediate segment is a direct canonical child. Record
   `TEST-MOD-REAL-004-CANONICAL-STRUCTURAL-PATH-VISIBILITY`.
2. Add fixtures with an inaccessible structural child and with an inaccessible final function.
   Require an error at the first offending declaration/visibility and the importing use span, with
   no provisional binding result. Record
   `TEST-MOD-REAL-004-STRUCTURAL-PATH-INACCESSIBLE-DIAGNOSTIC`.
3. Build a table-driven set of canonical keys proving the exact private, `pub(self)`,
   `pub(crate)`, `pub(super)`, `pub(in path)`, and `pub` regions. Include sibling, descendant,
   ancestor, cross-crate, and non-public-intermediate-edge cases. Record
   `TEST-MOD-REAL-004-CANONICAL-VISIBILITY-REGIONS`.
4. Feed a same-path/topology declaration snapshot that either removes the target function or
   changes it from `pub` to private while retaining the graph artifact. Require
   `ScopeGraphMismatch` before alias preflight or publication; artifacts alone must not authorize
   a scope entry. Record
   `TEST-MOD-REAL-004-CANONICAL-SCOPE-DECLARATION-SNAPSHOT-MISMATCH`.
5. Put a public target below a non-public structural child. Require it to remain inaccessible even
   if a declaration-only public query would call the target public; the route must validate the
   full traversed structural path independently. Record
   `TEST-MOD-REAL-004-CANONICAL-PUBLIC-PATH-VISIBILITY-FENCE`.
6. Add a root/local ordinary function with the requested alias spelling. Require a collision
   diagnostic and prove no alias replaces or shadows the declaration. Record
   `TEST-MOD-REAL-004-BINDER-LOCAL-DECLARATION-COLLISION`.
7. Add equivalent inline and file-backed module fixtures. Compare normalized graph-derived scopes,
   resolved structural target identity, visibility decision, and staged binding projection; this is
   unit-level representation parity only, not CLI/daemon parity. Record
   `TEST-MOD-REAL-004-STRUCTURAL-PATH-FILE-INLINE-PARITY`.
8. Stage a valid early import followed by a late invalid structural path or visibility failure.
   Require the full provisional-scope/binding result to be absent, not partially published. Record
   `TEST-MOD-REAL-004-STRUCTURAL-PATH-ATOMICITY`.
9. Add authority-fence fixtures showing that `pub use`, groups, globs, non-`crate` paths,
   non-function targets, legacy/string visibility helpers, final interfaces, Core/CPS, admission,
   and runtime are neither accepted nor reached by this route. Record
   `TEST-MOD-REAL-004-STRUCTURAL-PATH-AUTHORITY-FENCE`.
10. Run
   `cargo test -p ash-typeck --test task_2068_canonical_provisional_module_scopes`.
   Expected: the target fails because canonical provisional scopes and structural-path visibility
   resolution do not exist.

### Task 2: Derive immutable provisional module scopes

**Files:**

- Create: `crates/ash-typeck/src/canonical_provisional_module_scopes.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Test: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`

1. Define a constructor-restricted `CanonicalProvisionalModuleScopes` and per-module scope entry
   types. Derive them only from the canonical graph's TASK-2067 module units/artifacts, storing
   direct child entries and ordinary function entries in deterministic maps keyed by their local
   spelling.
2. Retain canonical `ModuleKey`, defining/declaration identity, visibility, origin, and relevant
   declaration spans in scope entries. Do not reconstruct them from source paths or strings, add
   source-order inference, synthesize exports, or expose public constructors.
3. Validate graph/artifact consistency and the correlated declaration snapshot while building.
   Duplicate direct names, missing child targets, mismatched owner identity, impossible artifact
   relations, or a same-path/topology source snapshot that removes a function or changes its
   visibility reject as `ScopeGraphMismatch` before a scope set or binding is returned. Artifacts
   alone must not authorize entries.
4. Keep the scope set immutable and opaque outside the intended Type-layer resolver. It may expose
   read-only lookup methods needed by the simple-import planner, but no final-interface,
   export-closure, compatibility-binder, Core, or runtime authority.
5. Run the focused test target. Expected: scope-construction and identity assertions pass; path
   resolution remains red until Task 3.

### Task 3: Resolve the admitted structural path and visibility before binding

**Files:**

- Modify: `crates/ash-typeck/src/canonical_provisional_module_scopes.rs`
- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify if delegation is required: `crates/ash-typeck/src/canonical_module_binder.rs`
- Test: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`

1. Add an opt-in adapter from the existing inherited simple parsed-import route to the provisional
   scopes. It accepts exactly `use crate::<structural-child>...::<ordinary-function> as <name>`;
   retain the current rejection of `pub use` and all excluded forms.
2. Walk each intermediate segment by direct-child lookup from the canonical crate root. Do not use
   the legacy string visibility helper or a text-path equality comparison. Preserve each edge's
   canonical child key, visibility, declaration span, and origin for diagnostics.
3. Implement the visibility predicate over crate identity and `ModuleKey` segment-prefix/
   parent relations. Resolve `pub(in path)` to a canonical key before applying its descendant
   region, and reject a malformed or disallowed path rather than falling back to textual matching.
   For `pub`, verify the whole structural path from the crate root through the defining module and
   the final function's public status. A declaration-only public query is never sufficient for
   this route; a public target beneath a non-public child remains inaccessible.
4. Preflight all path segments, the final function target, visibility, and the local ordinary
   declaration collision before staging a binding. Create aliases only in a temporary deterministic
   map; publish the opaque planned binding set after every selected import succeeds. On any error,
   return no partially built scope/binding output.
5. The alias must retain the function's definition identity; it must not become an importer/root
   declaration, re-export, or final interface fact. Keep all definition/body checks and generic
   cycle realization out of this resolver.
6. Run the focused target. Expected: all nine reserved witnesses pass, including the declaration
   snapshot mismatch, public-path fence, and visibility
   regions, diagnostic anchors, file/inline unit projection parity, and atomicity fence.

### Task 4: Protect prior route boundaries and quality gates

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_canonical_provisional_module_scopes.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_parsed_import_binder.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_primitive_provider_client.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_root_client.rs`

1. Re-run the generic parsed-import binder to confirm source `pub use` remains rejected and that
   no compatibility conversion makes the provisional scope output a final binder result.
2. Re-run the existing direct-fragment, private-helper, provider/client, and root-client targets
   to prove their public projections and authority fences have not widened.
3. Run `cargo fmt --check`,
   `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`, and the affected
   `ash-typeck` test suite.
4. Request code review for graph-only construction, canonical identity preservation, each
   visibility region, diagnostic ownership, staged atomic publication, collision handling, and
   containment from final-interface/runtime authority. Address blocking findings before evidence
   promotion.

### Task 5: Promote only earned evidence

**Files:**

- Modify after GREEN: TASK-2068, `docs/plan/SEMANTIC-RULE-COVERAGE.md`,
  `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, PLAN-207,
  AUDIT-207, Phase 207 index text, and the modules language reference as needed.

1. Replace only `IMPL-MODULE-CANONICAL-PROVISIONAL-MODULE-SCOPES` and the seven deferred
   structural-path test nodes/edges with concrete source and test anchors.
2. Classify the positive structural path, visibility regions, and file/inline unit projection
   witnesses as positive evidence; inaccessible paths, collision, and authority fences as negative
   evidence; and the late-invalid-path witness as mutation evidence.
3. Recompute source fingerprints for every changed Type-layer source file. Report this fragment as
   `partial / tested / below_spec` only after every focused witness is green. Tests are not a
   proof and do not establish a final module interface or client-execution parity.
4. Run the semantic-record, traceability, orientation, documentation-gate, and diff checks. Keep
   TASK-2068 and Phase 207 In progress, leave TASK-2069 unstarted, and do not update
   `CHANGELOG.md` for this planning or bounded-evidence work.

## Handoffs and completion boundary

This reservation consumes only TASK-2067 canonical module graph units/artifacts and the existing
inherited simple parsed-import shape. It would produce an immutable, non-authorizing Type-layer
provisional-scope set plus staged local alias/import-edge facts. TASK-2068 retains the complete
interfaces/imports/binder rule; TASK-2069 owns later lowering and Engine transport; TASK-2064 owns
separately authorized end-to-end client parity. Completing the planned fragment remains `partial`
and `below_spec` until all of SPEC-103's module realization rule is implemented.
