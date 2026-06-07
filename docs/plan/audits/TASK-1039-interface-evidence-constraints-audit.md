# TASK-1039 audit: interface evidence constraints

## Scope

This audit freezes the live parser, TypeEnv, summary/import, stdlib, and focused-test seams for SPEC-080 before implementation.

## Parser seams

- Surface carrier: `crates/ash-parser/src/surface.rs`
  - `InterfaceDef` currently has `visibility`, `name`, `type_params`, `associated_types`, `methods`, and `span`.
  - `ImplDef` already has `where_bounds: Vec<WhereBound>`.
  - `WhereBound` currently stores only `param: Name`, `bound: Name`, and `span`, matching the existing `T: Interface` subset.
- Parser entry point: `crates/ash-parser/src/parse_module.rs`
  - `parse_interface_definition` parses optional interface parameters and immediately expects `{`; there is no interface `where` tail today.
  - `parse_impl_definition` parses an optional `where` tail via `parse_where_bounds` before `{`.
  - `parse_where_bounds` accepts comma-separated `identifier : identifier`; it rejects equality/disequality/named predicates/object-style extension forms by not parsing them at this site.
- TASK-1040 should add an interface-owned constraint carrier rather than reusing `ImplDef::where_bounds` semantically. Reusing the same concrete `WhereBound` shape is acceptable only if the field name on `InterfaceDef` makes ownership clear, e.g. `evidence_constraints: Vec<WhereBound>`.
- TASK-1040 parser RED/GREEN target:
  - `crates/ash-parser/tests/task_1040_interface_constraint_surface.rs`
  - command: `RUSTC_WRAPPER= cargo test -p ash-parser --test task_1040_interface_constraint_surface -- --nocapture`
- TASK-1040 required parser cases:
  - accepts `interface Monad<M : * -> *> where M: Applicative { ... }` and preserves exactly one evidence constraint.
  - accepts comma-separated constraints, e.g. `where T: Functor, T: Foldable`.
  - rejects `where T == U`, `where T != U`, `where NonEmpty<T>`, `where T: Applicative + Monad`, `interface Monad<M> : Applicative<M> { ... }`, and `extends` syntax.

## TypeEnv/evidence seams

- Carrier: `crates/ash-typeck/src/type_env.rs`
  - `InterfaceInfo` currently stores name, visibility, type parameter names/kinds, associated types, and methods. It has no interface-level evidence constraints.
  - `ImplScheme` stores `where_bounds: Vec<WhereBound>` for impl schemes.
  - `TypeEnv::type_var_interface_bounds` tracks in-scope generic interface evidence by type variable.
- Interface registration:
  - `TypeEnv::register_interface` is the right place to validate that constraint subjects name parameters from the same interface and that referenced interfaces exist.
  - `register_interface` currently inserts a temporary `InterfaceInfo` before method conversion, then reinserts final info. TASK-1042 must preserve transactional cleanup on constraint-validation or method-conversion failure.
- Impl registration:
  - `TypeEnv::register_impl` lowers interface head arguments through `lower_interface_evidence_args` and validates impl `where` bounds around lines 16324-16358.
  - Concrete impl evidence is recorded by `record_concrete_impl_interface_assumption` only after method and associated-type checks.
  - TASK-1042 should verify required evidence before pushing the new `ImplScheme` and before recording the concrete impl assumption, so failed `Monad<Option>` without `Applicative<Option>` leaves no partial evidence.
- Generic entailment:
  - Existing generic facts enter via `bind_type_var_interface_bound` and `record_type_var_interface_bound_assumption`.
  - TASK-1043 should expand/check required evidence from constrained bounds directionally: `M: Monad` may satisfy `M: Applicative`, but `M: Applicative` must not satisfy `M: Monad`.
  - Do not add proof search, blanket impl synthesis, or method inheritance.
- TASK-1042 focused target:
  - `crates/ash-typeck/tests/task_1042_interface_constraint_registration.rs`
  - command: `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1042_interface_constraint_registration -- --nocapture`
- TASK-1043 focused target:
  - `crates/ash-typeck/tests/task_1043_interface_constraint_entailment.rs`
  - command: `RUSTC_WRAPPER= cargo test -p ash-typeck --test task_1043_interface_constraint_entailment -- --nocapture`

## Summary/import seams

- Engine import path uses `crates/ash-engine/src/module_loader.rs` to collect `ModuleSemanticSummary` and `crates/ash-engine/src/lib.rs::register_imported_semantic_summaries` to register imported summaries into `TypeEnv`.
- Existing summaries transport interface identities and associated-family metadata, but no interface-level evidence constraints.
- TASK-1041 must explicitly decide whether SPEC-080 constraints need semantic summary transport for stdlib/imported interfaces. Because TASK-1044 requires final `std::algebra` import-path enforcement, a local-only TypeEnv field is insufficient unless the engine re-registers imported interface definitions from source before checking users.
- TASK-1041 focused target:
  - engine/typeck summary tests as needed, preferably `crates/ash-engine/tests/task_1041_interface_constraint_summary_transport.rs` if summary transport is added.
  - command if added: `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1041_interface_constraint_summary_transport -- --nocapture`
  - if no summary change is needed, TASK-1041 must contain an explicit audit artifact explaining the final import path that preserves constraints.

## Stdlib/final path seams

- Current stdlib files:
  - `std/src/algebra/applicative.ash`: `pub interface Applicative<F : * -> *> { ... }`
  - `std/src/algebra/monad.ash`: `pub interface Monad<M : * -> *> { ... }`
- TASK-1044 should migrate only after parser/typechecker/import paths enforce constraints:
  - `pub interface Monad<M : * -> *> where M: Applicative { ... }`
- TASK-1044 focused target:
  - `crates/ash-engine/tests/task_1044_stdlib_monad_constraint.rs`
  - command: `RUSTC_WRAPPER= cargo test -p ash-engine --test task_1044_stdlib_monad_constraint -- --nocapture`

## Phase gates

Run these before closeout after the implementation tasks complete:

```bash
cargo fmt --check
git diff --check
RUSTC_WRAPPER= cargo check --workspace
RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTC_WRAPPER= cargo test --workspace
```

Focused cargo commands must run named test binaries or otherwise prove non-zero test execution; do not accept a zero-test filtered success as evidence.
