# TASK-1927: Provider Authoring API

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Define a provider authoring API that makes authority, operation surfaces, constraints, resources,
effects, and provenance visible at registration time.

## Requirements

- Add provider declaration/building APIs for operation surfaces, effect levels, constraints,
  resource usage, sandbox policy, and provenance policy.
- Validate provider metadata before provider registration can satisfy row admission.
- Preserve compatibility for existing provider implementations through explicit adapters or
  migration shims.
- Fail closed for overbroad, missing, stale, or authority-widening provider metadata.

## TDD Steps

1. Add failing provider authoring tests for valid and invalid metadata.
2. Implement the authoring API and validation.
3. Migrate existing built-in providers to the validated authoring path.

## Completion Checklist

- [x] Providers declare operation surfaces and effect levels.
- [x] Providers declare constraints, resources, sandbox policy, and provenance policy.
- [x] Invalid provider metadata fails closed.
- [x] Existing providers continue to work through explicit metadata.

## Evidence

- Added provider authoring metadata carriers in `ash-core::capability`:
  `ProviderAuthoringMetadata`, `ProviderOperationMetadata`, `ProviderMetadataError`, and
  `validate_provider_authoring_metadata`.
- Extended `CapabilityProvider` with `provider_metadata()`. Existing custom providers receive an
  explicit compatibility shim; standard host providers override the method with operation-level
  metadata.
- Added explicit metadata for stdio, filesystem, HTTP, time, process, MCP, and LLM providers,
  including operation surfaces, effects, row requirements, constraints, resources, sandbox policies,
  and provenance policies.
- Runtime host capability binding admission now validates provider metadata and rejects admitted
  rows not declared by explicit provider metadata.
- Added TASK-1927 tests:
  - `cargo test -p ash-core --test task_1927_provider_authoring_metadata`
  - `cargo test -p ash-engine --test task_1927_provider_authoring_api`
  - `cargo test -p ash-interp --test task_1927_provider_authoring_admission`
- Verified targeted custom-provider compatibility:
  - `cargo test -p ash-engine --test e2e_capability_provider_tests test_provider_with_empty_name -- --exact`
  - `cargo test -p ash-engine --lib tests::test_builder_custom_provider_chaining -- --exact`
  - `cargo test -p ash-engine --lib tests::test_builder_custom_provider_overrides_builtin -- --exact`
  - `cargo test -p ash-engine --lib tests::test_builder_multiple_custom_providers -- --exact`
  - `cargo test -p ash-engine --lib tests::test_builder_stdio_fs_custom_together -- --exact`
  - `cargo test -p ash-engine --lib tests::test_builder_complex_chaining_order_without_http -- --exact`
- Verified affected-package gates:
  - `cargo fmt --all -- --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`
  - `cargo check -p ash-core -p ash-interp -p ash-engine`
  - `cargo clippy -p ash-core -p ash-interp -p ash-engine --all-targets --all-features`
  - `cargo test -p ash-core -p ash-interp -p ash-engine`
