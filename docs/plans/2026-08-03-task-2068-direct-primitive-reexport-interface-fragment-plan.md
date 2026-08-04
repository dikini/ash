# TASK-2068 Direct Primitive Re-export Interface Fragment Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Deliver one bounded, non-authorizing Type-layer fragment that validates a root's direct public primitive re-export and produces an export-closed canonical interface fragment.

**Architecture:** The fragment consumes the parser-owned `CanonicalModuleGraph` and an exact resolved public-simple-import plan. It admits only a root and plan-selected direct primitive provider leaves, validates `pub mod api` plus `pub use crate::api::greet as welcome`, and stages an immutable structural-child projection and explicit re-export bindings before returning either the whole fragment or an anchored error. It reuses the existing primitive provider/client checking only as checked-fact validation; it must not convert that result, the planner, or this fragment into a final interface, compatibility carrier, or runtime authority.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-core`, `ash-typeck`; `proptest`; repository semantic-accounting and documentation validators.

---

## Scope, authority, and activation gate

The authoritative target clauses are TASK-2068 requirement 6 and SPEC-103 §§3, 6--8:

- `pub mod api` exposes the direct child identity `api`, not every declaration in `api`.
- An explicit `pub use crate::api::greet as welcome` is the sole route that makes the direct child declaration visible in the root's public fragment.
- The public structural path and the target declaration must both be public before registration; defining identity, declaration span, acquisition provenance, checked primitive signature, and use span are retained.
- The result is transactional: no fragment exists after a topology, visibility, plan/artifact, signature, collision, or late validation failure.

The admitted domain is only a root plus plan-selected direct provider leaves. Each selected provider is a direct public child of the root, has no imports or children, and contains ordinary primitive functions only. The root may expose the child through `pub mod` and must explicitly re-export one or more selected direct public primitive functions through exact crate-root simple paths; an empty public-use plan fails closed. No other public root definition is in this fragment, even if one valid re-export exists, and a root alias may not collide with the public child spelling. The positive seed is:

```ash
pub mod api {
    pub fn greet() -> String { "hello" }
}

pub use crate::api::greet as welcome;
```

Everything outside that seed remains excluded: every other namespace, path/import form, visibility form, re-export topology, non-direct child, compatibility carrier, Core/CPS, Engine, admission, runtime, and file/inline or CLI/daemon parity.

Before any Rust implementation, update the active TASK-2068 accounting to name this exact bounded rule and retain `partial / tested / below_spec`. Update the TASK-2068 task record, `SEMANTIC-RULE-COVERAGE.md`, `semantic-task-records.json`, and `SEMANTIC-TRACEABILITY.json` before publishing implementation/evidence claims. When the sub-slice is actually activated and completed, reconcile the same rule in the Phase 207 plan and index, AUDIT-207, the module language reference, and `CHANGELOG.md` as well.

This plan is planning material only. It supplies no implementation, test, proof, or parity evidence and must not change any semantic axis by itself.

## Proposed public boundary

Add the following narrow checker API:

```rust
pub fn check_direct_primitive_interface_fragments(
    graph: &CanonicalModuleGraph,
    plan: &CanonicalResolvedSimpleImports,
) -> Result<CanonicalPrimitiveInterfaceFragments, CanonicalPrimitiveInterfaceError>
```

`CanonicalPrimitiveInterfaceFragments` has private fields and read-only accessors. It retains:

- the exact root artifact/key;
- public direct structural children keyed by their public child spelling and canonical child key;
- explicit public re-export bindings keyed by their root-visible alias; and
- for each binding, the immutable defining identity, defining declaration span, acquisition provenance, checked primitive signature, and parsed `use` span.

It must expose `api` as a public structural child and `welcome` as an explicit binding to
`api::greet`; it must not synthesize a root binding named `greet`. It is neither
`PublicModuleInterface` nor `CanonicalPublicFunctionInterface`, has no public constructor or
`Default`, and grants no import/binder/admission/runtime authority.

`CanonicalPrimitiveInterfaceError` must preserve the existing plan/artifact and checked-provider
diagnostic context or wrap it without losing its relevant parser anchor. It also needs anchored
failures for a non-public structural path, private target, non-primitive target, empty public-use
plan, public root shape outside the exact fragment, and root-visible name collision, including
`pub mod api` plus `pub use crate::api::greet as api`. Tests assert variants and retained fields
rather than rendered error text.

## Planned files

- Create: `crates/ash-typeck/src/canonical_primitive_interface_fragments.rs`
- Create: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`
- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
  - keep `resolve_simple_parsed_imports` fail-closed for its existing inherited-only contract;
  - add a separately named, opt-in direct-public-primitive re-export planning entry point that
    yields an exact `CanonicalResolvedSimpleImports` snapshot only for the admitted root/direct
    form; share only private target-resolution mechanics.
- Modify: `crates/ash-typeck/src/canonical_primitive_provider_client.rs`
  - factor only the checked direct-provider facts or validation helper needed by the fragment;
    preserve the existing provider/client API, topology diagnostics, and authority fence.
- Modify: `crates/ash-typeck/src/lib.rs` to declare and re-export only the proposed fragment API
  and its non-authorizing output/error types.
- Modify on activation/completion only: `docs/plan/tasks/TASK-2068-final-interfaces-parsed-imports-and-binder-integration.md`, `docs/plan/SEMANTIC-RULE-COVERAGE.md`, `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`, `docs/plan/PLAN-INDEX.md`, `docs/plan/audits/AUDIT-207-module-realization-seams.md`, `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`, and `CHANGELOG.md`.

Do not modify `ash-core`, `ash-engine`, CLI, daemon, Core/CPS lowering, or compatibility-carrier
code for this increment.

## Task 1: Activate the rule and make the test harness red

**Files:**

- Create: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`
- Modify on activation: the TASK-2068 accounting files listed above.

**Step 1: Record the rule boundary before Rust changes.**

Add a TASK-2068 sub-slice with its consumed handoffs (canonical graph plus exact public-simple
plan), produced handoff (non-authorizing fragment only), downstream owner (remaining TASK-2068
interface/binder work), integration owner (TASK-2064), and `prerequisite` run-route impact. Record
all three axes as `partial / none / below_spec` until the focused evidence exists. Do not add a
final-interface, Engine, or parity claim.

**Step 2: Add the failing focused test fixture builder and twelve rule tests.**

Use canonical inline and file fixture helpers in the style of
`task_2068_primitive_provider_client.rs`. The test file must declare these exact cases:

1. `direct_public_primitive_reexport_builds_export_closed_fragments_without_flattening`;
2. `nonpublic_direct_structural_path_rejects_before_fragment_publication`;
3. `private_direct_reexport_target_rejects_with_its_declaration_anchor`;
4. `nonprimitive_direct_reexport_target_is_rejected_before_publication`;
5. `root_public_reexport_name_collision_rejects_before_fragment_publication`;
6. `plan_from_a_same_key_different_artifact_is_rejected_before_fragment_checking`;
7. `empty_public_reexport_plan_rejects_before_fragment_publication`;
8. `public_root_definition_outside_the_exact_fragment_rejects_before_publication`;
9. `public_child_identity_and_reexport_alias_collision_rejects_with_structural_and_use_anchors`;
10. `generated_direct_public_aliases_preserve_identity_provenance_signature_and_use_span`;
11. `late_invalid_public_reexport_rejects_atomically`; and
12. `direct_primitive_interface_fragment_checker_has_no_compatibility_or_runtime_authority`.

The positive case must assert all of the following: `api` is present as the public structural
child; `welcome` maps to the defining identity for `api::greet`; its declaration span, origin,
primitive signature, and `pub use` span match parser/checked facts; and no root-visible `greet`
binding was fabricated. Configure the property test for 16 generated valid identifier aliases.

**Step 3: Run the new target to confirm RED.**

Run:

```text
cargo test -p ash-typeck --test task_2068_direct_primitive_reexport_interface_fragments
```

Expected: compilation fails because the direct-public planner entry point and
`check_direct_primitive_interface_fragments` API do not exist. Do not weaken a test to fit the
current inherited-only planner.

## Task 2: Add a fail-closed direct-public planning mode

**Files:**

- Modify: `crates/ash-typeck/src/canonical_simple_import_planner.rs`
- Modify: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`

**Step 1: Add planner-level RED assertions for public path and target admission.**

Build an exact public-use plan for the positive fixture. Then assert that planning rejects:

- `mod api` (not `pub mod api`) followed by `pub use crate::api::greet as welcome`, anchored at
  the inaccessible structural declaration/path;
- `pub mod api` with private `fn greet`, anchored at `greet`'s declaration rather than reported as
  a missing name; and
- any non-simple, non-crate-root, non-public, non-direct, grouped, glob, qualified, or restricted
  visibility spelling.

Run the focused target and confirm these tests are RED before extending planner behavior.

**Step 2: Implement the smallest separate planning entry point.**

Add a named direct-public-primitive re-export planner (for example,
`resolve_direct_primitive_interface_imports`) rather than widening
`resolve_simple_parsed_imports`. It may reuse private provisional-declaration and exact-artifact
collection, but must separately require:

- importing module is the graph root;
- use visibility is exactly `pub`;
- path is exactly `crate::<direct-public-child>::<ordinary-function>`;
- the child is a graph-owned direct child declared `pub mod` in the root; and
- the selected target is public and its defining identity/origin/declaration span are retained.

The returned `CanonicalResolvedSimpleImports` must still match the exact root and every graph
artifact. It must not make public re-exports available to the old generic planner, binder, or
compatibility paths.

**Step 3: Run the planner assertions to GREEN.**

Run the focused target. Expected: positive plan construction works, while every public-path and
private-target rejection is anchored and no plan is returned.

## Task 3: Implement the staged interface fragment checker

**Files:**

- Create: `crates/ash-typeck/src/canonical_primitive_interface_fragments.rs`
- Modify as needed: `crates/ash-typeck/src/canonical_primitive_provider_client.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`

**Step 1: Keep the remaining checker cases RED.**

After Task 2, the checker-specific cases must still fail for the missing API: positive output
shape/provenance, non-primitive selected target, root-visible collision, same-key artifact
mismatch, generated alias property, late-error atomicity, and authority fence.

**Step 2: Define immutable fragment/error types and the exact API.**

Implement private-field `CanonicalPrimitiveInterfaceFragments`, a structural-child fact, and a
public re-export binding fact with read-only accessors. Add
`CanonicalPrimitiveInterfaceError` and the exact proposed function signature. No constructor,
mutable registry, global cache, or conversion to a full interface is permitted.

**Step 3: Validate, stage, then publish once.**

First reject a mismatched plan/artifact snapshot. Next reuse or narrowly factor the existing
direct-provider check so every selected provider is a direct primitive leaf and every planned edge
revalidates against a checked public provider. Validate the root's parsed `pub mod` declaration
and graph child relation, then validate each `pub use` against the exact planned public path.
Stage structural children and explicit bindings in deterministic maps. Detect root-visible alias
collisions before insertion. Only construct `CanonicalPrimitiveInterfaceFragments` after all
providers, structural paths, targets, signatures, and aliases validate.

Do not infer `greet` from `pub mod api`; add `welcome` only from its matching explicit `pub use`.
Do not publish selected checked provider/client facts as a general import/binder credential.

**Step 4: Run the checker cases to GREEN.**

Run:

```text
cargo test -p ash-typeck --test task_2068_direct_primitive_reexport_interface_fragments
```

Expected: all twelve tests pass, including the 16-case generated-alias property. The late-error
fixture contains an earlier valid re-export followed by an invalid one; it must return only an
error, with no fragment available to inspect.

## Task 4: Preserve fences and record only earned evidence

**Files:**

- Modify: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`
- Modify on successful implementation: every activation/completion document listed in Planned files.

**Step 1: Make the authority fence precise.**

The source-level fence must reject imports or symbols that would use legacy/TASK-2060/TASK-2061/
TASK-2066 compatibility carriers, `PublicModuleInterface`, `RawCoreProgram`, `CoreExpr`,
`CpsProgram`, Engine admission/runtime paths, or a direct evaluator. It must also reject a public
constructor or `Default` for the fragment result. The allowed dependencies are parser graph facts,
the narrow planner, primitive checked facts, and ordinary `Type`/provenance data only.

**Step 2: Record semantic evidence after GREEN, not before.**

Add rule-specific positive, negative, property, mutation, and fence IDs to TASK-2068's evidence
record and traceability graph, including source fingerprints and `tested_by` edges. State exactly
that this delivers a direct primitive public structural-path/re-export fragment; retain
`partial / tested / below_spec`, no proof, and no full export/interface/import/parity claim.
Update Phase 207, the seam audit, language reference, and `CHANGELOG.md` only with that bounded
claim.

**Step 3: Run focused quality and documentation gates.**

Run:

```text
cargo fmt --check
cargo test -p ash-typeck --test task_2068_direct_primitive_reexport_interface_fragments
cargo test -p ash-typeck --test task_2068_primitive_provider_client
cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
python3 tools/docs/validate_semantic_task_records.py --root . --manifest docs/plan/semantic-task-records.json
python3 tools/docs/validate_semantic_traceability.py --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

Expected: all commands pass. The existing provider/client target remains green, proving that the
new fragment has not widened or replaced its narrower direct-client contract.

## Commit policy

No commit is made while creating this plan, and this plan deliberately contains no `git add` or
`git commit` execution step. Repository policy requires explicit user authorization before direct
git mutation. An implementer may prepare the documented changes and validation evidence, then must
request that authorization before staging or committing.
