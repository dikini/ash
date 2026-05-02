# TASK-792: Phase 109 Review Remediation

## Status: ✅ Complete

## Description

Fix the blocking and non-blocking findings from the independent Phase 109 review after TASK-791. This task hardens Phase 109 completion by reconciling stale corpus status, enforcing semantic-summary authority in TypeEnv and engine import/export paths, preserving stdlib semantics that were weakened during closeout, and recording focused/broad verification honestly.

## Specification Reference

- [SPEC-057](../../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [PLAN-105](../PLAN-105-UNIFIED-TYPE-MODULE-PIPELINE-SEMANTIC-SUMMARIES.md)
- [TASK-791](TASK-791-spec-a-closeout-docs-examples-verification.md)

## Dependencies

- ✅ TASK-780 through TASK-791: Phase 109 implementation and closeout.
- ✅ Independent Phase 109 review findings from the controller review pass.

## Requirements

### Functional Requirements

1. Reconcile Phase 109 status surfaces so PLAN-INDEX, PLAN-105, SPEC-057, task files, and CHANGELOG agree on the final state.
2. Fix engine import/export alias leakage so re-export aliases do not expose origin type names through callable-only imports, glob imports, or legacy `TypeDef` fallback transport.
3. Make selected representation-dependency transport identity-only for dependent public types unless the dependency's type or constructor was selected directly, so importing `Message` for a field of type `Role` does not make `Role` constructors such as `System` usable.
4. Make constructor-only imports expose only the named constructor identity plus the parent type identity needed for typing, not sibling constructors from the same enum, while allowing separate selected constructor imports for the same parent type to accumulate without erasing earlier constructor metadata.
5. Make export/import validation reject public API private-type leaks consistently at export/check boundaries, including public signatures that mention imported private or unresolved ordinary types, while treating capabilities and current builtin carrier names (`Bytes`, `Map`, `Stream`, `P`, `Act`, `Proc`, `Workflow`) as non-ordinary-type references.
6. Reject inline-module ordinary type declarations explicitly in the current engine module-file check path until inline-module summary lowering/export support exists.
7. Harden `TypeEnv::register_module_semantic_summary` so it validates summary version/visibility/exposure, rejects conflicting duplicate summaries for the same canonical identity even under different visible aliases, and does not expose constructors independently of the exported constructor metadata contract.
8. Add serde/default compatibility coverage for module semantic summary evolution where appropriate.
9. Preserve stdlib semantics that were weakened during TASK-791 closeout, or explicitly document narrowed behavior if current syntax cannot represent the old behavior safely.
10. Keep deferred DESIGN-034 features deferred: no `type fn`, sealed domains, normalization, generalized associated-family computation, or proposition solving.
11. Preserve Phase 108 workflow-summary transport.

### Regression Requirements

Add or strengthen tests for:

1. Callable-only import of a pub-use type alias does not make the origin type name visible.
2. Glob import/re-export alias cases do not leak origin type names through fallback type definitions.
3. Named imports of representation-dependent public types transport dependency identities without exposing dependency constructors as ordinary values.
4. Constructor-only imports do not expose sibling constructors from the same enum, and separate named constructor imports for the same parent type accumulate selected constructor metadata without duplicate type-summary leakage.
5. Export/import validation rejects public callable signatures and public representations that expose private, imported-private, or unresolved ordinary types; capability references and current builtin carrier types are not classified as unresolved ordinary types.
6. Inline-module ordinary type declarations fail explicitly in the current engine check path rather than being silently ignored.
7. TypeEnv rejects unsupported summary versions, private exposed summaries, and conflicting duplicate same-identity summaries under different visible aliases; compatible exposed summaries may upgrade a prior identity-only summary for the same visible canonical identity.
8. Constructor exposure follows semantic-summary constructor visibility/metadata, not only enum body reconstruction.
9. `ModuleSemanticSummary` deserializes older/minimal payloads for fields that have defaults.
10. stdlib LLM supervised-agent helper behavior preserves rejection feedback and meaningful tool-call review details within the current Ash syntax subset.

## TDD Steps

1. Write failing regression tests for the engine alias/export findings in `crates/ash-engine/tests/task_786_import_visibility_summary_rules.rs` or a new focused test file.
2. Write failing regression tests for TypeEnv summary validation in `crates/ash-typeck/tests/task_787_semantic_summary_typeenv.rs` and serde compatibility in `crates/ash-core/src/semantic_summary.rs` tests.
3. Write or adjust stdlib-focused E2E tests in `crates/ash-engine/tests/llm_stdlib_e2e_tests.rs` for preserved supervised/router helper behavior where current test infrastructure can observe it.
4. Implement the minimal Rust/Ash/docs changes to make those tests pass.
5. Run focused tests after each remediation slice.
6. Run final focused Phase 109 gates, clippy, fmt, workspace check, and broad `cargo test --all`; if broad remains failing, document exact residual scope honestly.

## Verification Steps

- [x] `git diff --check`
- [x] `cargo fmt --check`
- [x] `cargo test -p ash-core semantic_summary`
- [x] `cargo test -p ash-typeck --test task_787_semantic_summary_typeenv`
- [x] `cargo test -p ash-typeck --test task_788_interface_summary_identity`
- [x] `cargo test -p ash-engine --test task_785_modulefile_summary_exports`
- [x] `cargo test -p ash-engine --test task_786_import_visibility_summary_rules`
- [x] `cargo test -p ash-engine --test llm_stdlib_e2e_tests`
- [x] `cargo clippy -p ash-parser -p ash-core -p ash-engine -p ash-typeck --all-targets -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --all` passes, or any residual failure is explicitly classified and reconciled with completion status.
- [x] Independent sub-agent review passes after remediation.

## Completion Notes

TASK-792 remediated the post-closeout Phase 109 review findings:

- Reconciled PLAN-105, PLAN-INDEX, SPEC-057, docs/spec references, and CHANGELOG status surfaces.
- Hardened engine import/export transport so re-export aliases do not leak origin names through legacy `TypeDef` fallback, selected public type summaries carry identity/shape for representation dependencies such as `Message -> Role` without exposing dependency constructors, constructor-only imports expose only the selected constructor, cumulative selected-constructor imports merge metadata for the same parent type without losing earlier constructors, and public signatures mentioning imported private/unresolved ordinary types are rejected.
- Added regressions for aliased self-recursive types and callable re-export signature alias rewriting across separate `pub use` statements.
- Follow-up Phase 109 re-review fixed split-order `pub use` alias constructor leakage, builtin callable alias execution through the original dispatch target, and same-module `pub use` type aliases in public callable signatures.
- Hardened `TypeEnv::register_module_semantic_summary` with stronger version/duplicate/visibility/constructor validation, transactional malformed-summary behavior, compatible identity-only to exposed-summary upgrades, cumulative partial constructor-summary registration, and canonical std `Result`/`Option` compatibility.
- Kept stdlib provider-backed operations checkable by declaring parser-safe builtins while documenting that concrete capability-wrapper bodies remain deferred until a canonical stdlib `act` wrapper spelling exists; left HTTP HEAD deferred because its current provider/result shape and `head` builtin-name collision require a real runtime bridge instead of a plain `pub builtin fn` declaration.
- Deferred `io::Result<T>` rather than introducing an IO-specific ADT that would shadow the prelude/canonical `Result<T, E>` identity.
- Repaired the expected-pass capability example corpus by replacing unsupported unit literal bodies with `null`.
- Hardened the Ash CLI test-runner timeout helper so late operation completions are classified by their actual completion instant rather than by when the receiver thread wakes under broad parallel `cargo test --all` scheduling.

Verification completed with focused Phase 109 gates, follow-up alias/visibility execution regressions, clippy, workspace check, independent sub-agent review, and broad `TMPDIR=/home/dikini cargo test --all` passing after remediation.
