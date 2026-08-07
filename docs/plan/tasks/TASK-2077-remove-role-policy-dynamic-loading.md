# TASK-2077: Remove Role, Policy, and Dynamic-Loading Machinery

**Status:** Complete
**Phase:** Maintenance cleanup
**Owned rule:** Dedicated Ash role/policy forms and dynamic module-loading behavior are removed;
static file/inline module realization remains the supported module path.

## Description

Delete dedicated Ash role and policy language/runtime machinery and dynamic module-loading-only
surfaces. This is a breaking cleanup with no compatibility layer. Unrelated provider data named
`Role`, host sandbox/provenance policy configuration, and contract-discharge `dynamic` states are
not part of this task.

## Requirements

1. Remove role and policy source forms, AST variants, parser/lowering paths, Core/CPS carriers,
   typechecker/finalizer/import/export branches, runtime authority/evaluator machinery, CLI/LSP
   behavior, fixtures, tests, and dedicated diagnostics.
2. Remove dynamic module acquisition, package/registry, incremental-loading, and runtime-module
   claims or APIs when they are dedicated to those behaviors.
3. Preserve static file-backed and inline module acquisition and the Phase 207 Engine/CLI/daemon
   execution route.
4. Preserve unrelated provider-role data, host sandbox/provenance configuration, and dynamic
   contract-discharge classification.
5. Remove obsolete active and historical documentation claims. Preserve and update NOTE-040 as
   the sole forward-looking composition note.
6. Update `CHANGELOG.md`, `PLAN-INDEX.md`, and orientation indexes as required by project policy.

## TDD Steps

1. Inventory dedicated symbols and classify the explicit unrelated-use allowlist.
2. Add or identify focused checks proving the static Phase 207 route remains executable.
3. Remove parser/compiler/runtime/tooling surfaces and run the smallest affected tests after each
   deletion group.
4. Remove stale documentation and run the documentation/index and absence gates.
5. Run workspace formatting, clippy, and tests.

## Completion Checklist

- [x] Dedicated role/policy syntax, AST, lowering, typechecking, Core/CPS, runtime, tooling,
      diagnostics, tests, fixtures, and documentation are removed.
- [x] Dynamic-loading-only implementation and documentation claims are removed.
- [x] Static file/inline module execution and CLI/daemon parity remain passing.
- [x] Explicit unrelated provider/host/contract uses remain passing.
- [x] NOTE-040 is updated and remains non-normative.
- [x] `CHANGELOG.md`, plan/index records, and orientation indexes are current.
- [x] `cargo fmt --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features` passes.
- [x] `cargo test --workspace` passes.
- [x] `python3 tools/docs/validate_orientation_indexes.py --self-test` passes.
- [x] `bash scripts/check-docs-gate.sh` passes.
