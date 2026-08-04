# TASK-2068 Direct Primitive Re-export Root Client Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task.

**Goal:** Add one bounded Type-layer route where an inherited/private root function can call an
explicit direct public primitive re-export alias without widening generic `pub use` authority.

**Architecture:** The route is opt-in and consumes a new opaque direct-plan kind, the canonical
graph, and exact root/provider artifact snapshots. It checks the selected provider and private
root client atomically, retaining the direct fragment and a checked local alias binding that
preserves the selected public target's defining identity. The result remains a constructor-free,
non-authorizing Type handoff; it is not a final module interface or a generic binder result.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-core`, `ash-typeck`; `proptest`; repository
semantic-accounting validators.

---

## Scope and semantic boundary

The authority is SPEC-103 §6 (local aliases retain definition identity and visibility is checked
before registration), §8 (`M-BIND`/`M-CHECK`), and §9 (identity preservation, no implicit
flattening, failure atomicity, and no runtime authority).

The sole admitted source form is:

```ash
pub mod api {
    fn normalize(value: Int) -> Int { value }
    pub fn greet(value: Int) -> Int { normalize(value) }
}

pub use crate::api::greet as welcome;

fn internal_entry(value: Int) -> Int { welcome(value) }
```

`internal_entry` must be inherited/private and ordinary with a closed primitive signature. Its
local `welcome` binding preserves the defining identity, visibility, checked signature, provenance,
and use span of `api::greet`; it does not turn `greet` into a root declaration or flatten `api`.
The successful opaque result contains only the existing direct fragment, checked private root
function facts, selected provider facts, and that local alias binding.

Out of scope: root public functions; provider uses; nested modules; other definition forms;
generics or contracts; restricted visibility; non-primitive/open signatures; all other import,
path, or re-export forms; final interfaces/export closure; compatibility or generic binder
authority; Core/CPS; Engine/admission/runtime; and file/inline or CLI/daemon end-to-end parity.
The existing generic planner/binder and generic provider/client route continue to reject source
`pub use`. No commit is authorized for this plan or its future implementation.

## TDD implementation tasks

### Task 1: Create the red root-client contract

**Files:**

- Create: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_root_client.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_private_primitive_provider_helpers.rs`
- Inspect: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Inspect: `crates/ash-typeck/src/canonical_primitive_interface_fragments.rs`
- Inspect: `crates/ash-typeck/src/canonical_primitive_provider_client.rs`

1. Add an inline positive fixture for the exact form. Assert that `internal_entry` resolves
   `welcome` to `api::greet`'s defining identity, signature, provenance, and use span, while the
   fragment still exposes only the structural child and explicit public alias. Record
   `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-LOCAL-BINDING-POSITIVE`.
2. Add equivalent file and inline fixtures and compare normalized opaque public projections,
   selected-provider facts, private root facts, and local alias identity. Record
   `...-FILE-INLINE-PARITY`.
3. Add a root-local `welcome` collision fixture and a `pub fn internal_entry` fixture. Both must
   fail before a result is published, with the collision/visibility declaration anchor. Record
   `...-LOCAL-COLLISION` and `...-PUBLIC-ROOT-REJECTION`.
4. Add a malformed `internal_entry` body that calls `welcome` incompatibly; require a local-body
   diagnostic anchored at the call/use and no partial result. Add a root/provider plan-artifact
   snapshot mismatch fixture. Record `...-BODY-DIAGNOSTIC` and `...-ARTIFACT-SNAPSHOT`.
5. Add a 16-case `proptest!` that varies accepted primitive names/signatures while checking that
   only the selected target's identity can enter the local alias. Add a late-invalid-root-client
   fixture proving no staged facts publish. Record `...-PROPERTY` and `...-ATOMICITY`.
6. Add source fences: the generic planner/binder and generic provider/client checker must reject
   source `pub use`, and the dedicated route must require its distinct opaque plan kind; no
   compatibility, Core/CPS, admission, or runtime authority may occur. Record
   `...-PLAN-KIND-FENCE` and `...-AUTHORITY-FENCE`.
7. Run `cargo test -p ash-typeck --test task_2068_direct_primitive_reexport_root_client`.
   Expected: the new target fails because no root-client direct plan or checked local-alias route
   exists.

### Task 2: Introduce an unforgeable direct root-client plan

**Files:**

- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_root_client.rs`

1. Define a separately named opaque direct root-client plan and an opt-in resolver for the exact
   root `pub mod` plus explicit `pub use crate::<provider>::<primitive> as <alias>` shape. Keep
   constructors private and expose only checked accessors required by the dedicated checker.
2. Capture exact root/provider `ModuleArtifact` snapshots, selected provider/module identities,
   public-target declaration identity, explicit alias spelling/use span, and the one permitted
   private root client declaration. Reject empty/implicit/malformed plans, public root functions,
   and all extra public root forms before a plan is returned.
3. Do not alter `resolve_simple_imports`, `bind_simple_parsed_uses`, or their public-use rejection.
   Assert the generic provider/client route continues to inspect source items and rejects `pub use`
   rather than accepting the new direct-plan type through a compatibility conversion.
4. Run the focused target. Expected: plan-kind and source-route fence tests pass once the opaque
   plan is available; positive root-client checking remains red until Task 3.

### Task 3: Check the private root client atomically

**Files:**

- Modify: `crates/ash-typeck/src/canonical_primitive_interface_fragments.rs`
- Modify if required for an explicit source-route fence:
  `crates/ash-typeck/src/canonical_primitive_provider_client.rs`
- Test: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_root_client.rs`

1. Add a dedicated opaque root-client output type with no public constructor. It may carry only
   the existing `CanonicalPrimitiveInterfaceFragments`, staged checked private root functions,
   selected provider facts, and a local alias binding; no final-interface/export/general-binder
   accessor may be added.
2. Revalidate plan/graph/root/provider artifact equality before checking. Reuse the delivered
   private-helper provider preflight and only then inject the explicit `welcome` alias into the
   private root checking environment.
3. Check `internal_entry` through the existing primitive body checker. Its alias target must retain
   `api::greet`'s definition identity rather than receive a root identity. Validate private root
   visibility before binding registration and reject a colliding root-local spelling.
4. Stage every provider, fragment, root-function, and alias fact locally. Construct the opaque
   result only after the provider, alias, root body, snapshot, and authority checks all succeed.
   Any late error must return no fragment, no private root facts, and no alias binding.
5. Run `cargo test -p ash-typeck --test task_2068_direct_primitive_reexport_root_client`.
   Expected: all ten cases pass, including the 16-case property.

### Task 4: Protect existing route boundaries and quality gates

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_root_client.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_parsed_import_binder.rs`
- Inspect: `crates/ash-typeck/tests/task_2068_primitive_provider_client.rs`

1. Re-run the generic parsed-import-binder and provider/client targets to prove that source
   `pub use` remains rejected outside the dedicated direct plan.
2. Re-run the existing direct-fragment and private-helper targets to prove that their public
   projection has not widened and their atomicity fences still hold.
3. Run `cargo fmt --check` and
   `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`.
4. Request code review for opaque-plan forgery resistance, identity/visibility ordering, snapshot
   equality, collision diagnostics, staged publication, and authority containment. Address any
   blocking review finding before evidence promotion.

### Task 5: Promote only earned evidence

**Files:**

- Modify after GREEN: TASK-2068, `docs/plan/SEMANTIC-RULE-COVERAGE.md`,
  `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, PLAN-207,
  AUDIT-207, Phase 207 index text, and the modules language reference as needed.

1. Replace only `IMPL-MODULE-CANONICAL-DIRECT-PRIMITIVE-REEXPORT-ROOT-CLIENT` and the ten
   deferred local-binding nodes/edges with concrete implementation/test anchors.
2. Classify the positive, file/inline parity, and property witnesses as positive evidence; the
   collision, public-root, body-diagnostic, artifact-snapshot, plan-kind, and authority witnesses
   as negative evidence; and the late-invalid-root-client witness as mutation evidence.
3. Recompute source fingerprints for every changed Type-layer source file. Report the slice as
   `partial / tested / below_spec` only if all focused witnesses are green; say explicitly that
   tests are not a proof and do not establish final-interface or client parity.
4. Run the semantic-record, traceability, orientation, documentation-gate, and diff checks. Keep
   TASK-2068 and Phase 207 In progress, leave TASK-2069 unstarted, and do not update
   `CHANGELOG.md` for this planning or bounded-evidence work.

## Handoffs and completion boundary

This reservation consumes the canonical graph, exact artifact snapshots, a new opaque direct plan,
selected provider facts, and the explicit local alias. It would produce only a non-authorizing
Type-layer fragment/root-client handoff. TASK-2068 retains complete interfaces/imports/binder
ownership; TASK-2069 owns later lowering and Engine transport; TASK-2064 owns integrated
file/inline and CLI/daemon parity. A successful implementation of this plan remains partial and
below specification until the complete SPEC-103 rule is realized.
