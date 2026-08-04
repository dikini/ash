# TASK-2068 Private Primitive Provider Helper Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Extend the bounded direct public primitive re-export fragment so selected public primitive targets may use inherited/private primitive provider helpers without exposing those helpers.

**Architecture:** Keep the existing opt-in exact root form, planner, and non-authorizing fragment
output. Narrow only the provider preflight so it checks public targets and inherited/private helper
functions atomically as implementation detail; projection still publishes only the direct public
child and explicit public re-export target.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-core`, `ash-typeck`; `proptest`; repository semantic-accounting validators.

---

## Scope and authority

The target clauses are SPEC-103 §§6--9 and TASK-2068: private declarations are usable only in
their defining provider; public interface closure excludes private facts; all provider/helper
checks must complete before fragment publication; and the structural child is never implicitly
flattened into a root binding.

The only admitted source shape remains:

```ash
pub mod api {
    fn normalize(value: Int) -> Int { value }
    pub fn greet(value: Int) -> Int { normalize(value) }
}

pub use crate::api::greet as welcome;
```

`normalize` is checked only as a provider implementation detail. The successful
`CanonicalPrimitiveInterfaceFragments` value retains `api` and `welcome`, never `normalize`.
The selected re-export target remains public. The generic simple planner/binder is untouched.

Explicit non-goals: provider `use` declarations; nested modules; non-function definitions;
generics; contracts; restricted visibility; non-primitive or open signatures; other path/import/
re-export forms; compatibility carriers; final interfaces/export closure; Core/CPS; Engine,
admission, runtime, file/inline integration parity, and CLI/daemon parity. No commit is authorized
for this plan or its future implementation.

## TDD implementation tasks

### Task 1: Make the private-helper contract red

**Files:**

- Modify: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`
- Modify before Rust: TASK-2068 accounting files named in the activation section below.

1. Add an inline positive fixture where a public primitive target calls an inherited/private
   primitive helper; assert the fragment exposes only `api` and the explicit alias. Record
   `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-POSITIVE`.
2. Add equivalent file and inline fixtures. Assert the same public child, alias identity, origin,
   signature, and no helper projection. Record
   `TEST-MOD-REAL-004-DIRECT-PRIMITIVE-REEXPORT-PRIVATE-HELPER-FILE-INLINE-PARITY`.
3. Add negative fixtures: re-exporting a private helper selects the planner's anchored private-
   target error; a private helper with a non-primitive signature rejects before publication.
   Record `...-PRIVATE-TARGET` and `...-NONPRIMITIVE`.
4. Add a late-invalid-helper fixture proving no fragment is returned after an earlier valid target;
   a 16-case helper-name/property test proving only the public target projects; and a source
   authority fence. Record `...-ATOMICITY`, `...-PROPERTY`, and `...-AUTHORITY-FENCE`.
5. Run `cargo test -p ash-typeck --test task_2068_direct_primitive_reexport_interface_fragments`.
   Expected: the new helper cases fail because provider preflight still requires every provider
   function to be public.

### Task 2: Admit checked private primitive helpers without widening publication

**Files:**

- Modify: `crates/ash-typeck/src/canonical_primitive_interface_fragments.rs`
- Test: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`

1. Preserve `resolve_direct_primitive_interface_imports` and the generic planner/binder unchanged:
   only a public selected re-export target may form an exact plan.
2. In `preflight_provider`, allow only inherited/private ordinary functions as non-exported helper
   declarations, while retaining rejection of restricted visibility, non-function definitions,
   provider uses, child modules, nested graph children, generics/contracts, and non-primitive or
   open signatures.
3. Reuse the existing primitive function/body checker for every provider function, stage every
   checked fact privately, and construct the fragment only after all selected providers pass.
4. Keep `CanonicalPrimitiveInterfaceFragments` projection unchanged: add structural children and
   planned public re-exports only; never add helper bindings or a helper constructor/accessor.
5. Run the focused target; expected: all existing and new cases pass, including 16 generated
   helper-name cases.

### Task 3: Verify atomicity, fences, and focused compatibility

**Files:**

- Test: `crates/ash-typeck/tests/task_2068_direct_primitive_reexport_interface_fragments.rs`
- Inspect only: `crates/ash-typeck/src/canonical_simple_import_planner.rs`,
  `crates/ash-typeck/src/canonical_primitive_interface_fragments.rs`

1. Re-run the direct target, the generic parsed-import binder target, and the primitive
   provider/client target to confirm no generic planner/binder or delivered provider/client
   contract widened.
2. Run `cargo fmt --check` and
   `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`.
3. Have a code-review subagent check private-fact containment, selected-target public visibility,
   atomic failure behavior, and the authority fence before evidence promotion.

### Task 4: Promote only earned helper evidence

**Files:**

- Modify after GREEN: TASK-2068, coverage, semantic record, traceability, PLAN-207, seam audit,
  and module language reference documents.

1. Replace only the seven deferred helper witness reservations with tested source anchors and
   `tested_by` edges; record focused test/function and property counts where applicable.
2. Recompute source fingerprints for changed Type-layer files.
3. Keep the helper fragment `partial / tested / below_spec`, Type `partial`, and Core/CPS/
   admission-runtime `not_applicable`; do not claim final interfaces, runtime, or parity.
4. Run the semantic-record, traceability, orientation, docs-gate, and diff checks. Do not update
   `CHANGELOG.md` or mark TASK-2068/Phase 207 complete for this bounded sub-slice.

## Activation accounting and handoffs

This plan reserves the helper slice as `partial / none / below_spec` only. It consumes the
canonical graph, exact direct-public alias plan, and bounded provider checker facts. It produces
the same constructor-free, non-authorizing `CanonicalPrimitiveInterfaceFragments` handoff; private
helper facts remain internal. TASK-2068 retains complete interface/import/binder ownership;
TASK-2069 cannot begin until TASK-2068 is complete; TASK-2064 remains the separately owned
integration/parity owner.
