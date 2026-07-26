# Effect-Row Provider Binding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make cross-module effect-row transport fail closed by separating immutable provider
identity from visible bindings, sanitizing every named/glob/public import closure, and versioning
both summaries and cache keys.

**Architecture:** `ash-core` owns the provider-identity, binding, opaque-boundary, version, and
cache-key contract. `ash-engine` selects and re-exports effect-row summaries exclusively through
one sanitizer. `ash-typeck` validates and registers the sanitized binding without granting runtime
authority. Older or incomplete summaries/caches are rejected at the consumer boundary.

**Tech Stack:** Rust 2024; `ash-core` semantic summaries; `ash-engine` module loader; `ash-typeck`
type environment; focused Rust tests; Cargo and documentation validation gates.

**Execution status:** Tasks 1–6 are complete in TASK-2025. The delivered scope is the V7
provider/binding summary contract, shared sanitizer, conflict preflight, TypeEnv transactional
registration, and process-local semantic-cache safeguards; runtime provider/handler authority is
not part of this plan's implementation.

---

## Preconditions

- Read [the approved design](2026-07-24-effect-row-provider-binding-design.md) and
  [TASK-2025](../plan/tasks/TASK-2025-effect-row-provider-binding-identity-and-sanitization.md).
- Activate rust-analyzer for the active worktree and trace every effect-row import/re-export
  selection path before editing it.
- Preserve the current non-authority meaning of effect rows. Do not combine this work with
  provider dispatch, handler execution, or residual-row implementation.

## Task 1: Define the core provider/binding contract test-first

**Files:**
- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify: `crates/ash-core/src/lib.rs`
- Test: `crates/ash-core/src/semantic_summary.rs` (`#[cfg(test)]` module)

1. Write failing core tests for a provider identity rooted in its declaring module and declaration;
   assert that an alias changes only a binding name, while the provider identity remains equal.
2. Write failing serialization tests showing that an inaccessible dependency produces only an
   opaque classification. Assert the JSON contains none of its private name, path, source anchor,
   row text, signature, or provider identity.
3. Add explicit core types equivalent to `EffectRowProviderIdentity`,
   `EffectRowVisibleBinding`, and `OpaqueInaccessibleDependency`. Keep provider-owned facts and
   visible binding facts in separate fields; do not overload `exported_name` as identity.
4. Replace the named, serializable inaccessible-dependency transport with the opaque contract.
   Update derives and public exports only as required by consumers.
5. Run `cargo test -p ash-core semantic_summary` and `cargo fmt --check`.

## Task 2: Version and validate summaries and semantic cache keys

**Files:**
- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify: `crates/ash-typeck/src/type_env/mod.rs`
- Modify: `crates/ash-typeck/src/error.rs`
- Test: `crates/ash-core/src/semantic_summary.rs`
- Test: `crates/ash-typeck/src/type_env/tests.rs`

1. Add failing tests that a summary carrying provider-binding payload under V1–V6 is rejected,
   that a new-version summary lacking required binding/closure fields is rejected, and that an
   unknown future version remains unsupported.
2. Introduce one new named summary version and validation errors for missing/invalid
   provider-binding payloads. Do not use serde defaults to turn legacy summaries into valid
   provider-binding summaries.
3. Extend `semantic_cache_key()` to include provider identities, visible bindings/exposure,
   sanitizer schema version, public-closure digest, and opaque status without private content.
4. Map validation failures to a deterministic type-environment error with a public-boundary
   diagnostic only. Run `cargo test -p ash-core semantic_summary` and
   `cargo test -p ash-typeck type_env`.

## Task 3: Centralize sanitizing closure in the module loader

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs`
- Test: `crates/ash-engine/src/module_loader/tests.rs`

1. Add failing named-import, glob-import, and `pub use` tests using one provider and one facade.
   Each must resolve the same provider identity while retaining its own visible binding.
2. Replace per-path selected-effect-row copying with one sanitizer that accepts a provider summary,
   selected binding, and exposure mode, then returns either a complete sanitized closure or an
   opaque failure.
3. Traverse dependencies by provider identity, not by rewritten local names. Preserve public
   declaration order only after identity-based closure selection has completed.
4. Reject a selected row whose closure reaches an inaccessible dependency before creating an
   imported summary, re-export payload, or cache entry. Assert diagnostics and JSON do not contain
   the private fixture name.
5. Run `cargo test -p ash-engine module_loader` and `cargo fmt --check`.

## Task 4: Reject binding conflicts deterministically at registration

**Files:**
- Modify: `crates/ash-engine/src/module_loader.rs`
- Modify: `crates/ash-typeck/src/type_env/imported_summaries_and_domains.rs`
- Modify: `crates/ash-typeck/src/type_env/mod.rs`
- Test: `crates/ash-engine/src/module_loader/tests.rs`
- Test: `crates/ash-typeck/src/type_env/tests.rs`

1. Add failing tests for two imports that expose one visible name from different providers, and for
   one provider with incompatible sanitized closure content. Execute each test in both source
   orders and require the same rejection class.
2. Make loader dedup/merge keys use provider identity plus visible binding and closure digest;
   remove any last-import-wins behavior for effect-row bindings.
3. Make type-environment registration reject a second incompatible binding before it can replace
   a prior entry. Do not register an opaque dependency as a usable row.
4. Run focused engine/typechecker tests and verify existing TASK-2001 named-import row controls
   still pass.

## Task 5: Prove cache invalidation and no-leak behavior end-to-end

**Files:**
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/module_loader.rs`
- Test: `crates/ash-engine/src/module_loader/tests.rs`
- Test: `crates/ash-engine/tests/task_2025_effect_row_provider_binding.rs` (new)

1. Write failing integration tests that seed a stale V1–V6 or mismatched-digest semantic summary
   cache entry and require a miss/rejection rather than binding registration.
2. Test that cache keys differ for different provider identities and closure digests, but not only
   because a facade chose a different visible alias where the provider/closure contract is the
   same. Include the visible binding/exposure dimension where it affects selection semantics.
3. Test that serialized cache/summary output and public diagnostics never contain a private
   dependency fixture token.
4. Implement only the cache-schema and registration coverage needed for these tests. Run
   `cargo test -p ash-engine task_2025_effect_row_provider_binding`.

## Task 6: Regressions, documents, and verification

**Files:**
- Modify: `docs/plan/tasks/TASK-2025-effect-row-provider-binding-identity-and-sanitization.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

1. Keep the symbolic `ImplType::operation(args)` controls, TASK-2001 row-cycle/newtype controls,
   TASK-2013/2024 private handler inspection, and direct stringy `invoke` rejection as regressions;
   none may become a dispatch or authority path.
2. Record exact completed evidence and any intentionally unresolved general row semantics in
   TASK-2025, the plan index, and changelog. Update semantic traceability only if an authoritative
   requirement/test mapping changes.
3. Run:

   ```bash
   cargo fmt --check
   cargo test --workspace
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   python3 tools/docs/validate_orientation_indexes.py --self-test
   bash scripts/check-docs-gate.sh
   python3 tools/docs/validate_semantic_traceability.py --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json
   git diff --check
   ```

4. Request code review. Commit only with explicit user authorization and a matching
   Common-Changelog entry.
