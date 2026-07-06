# TASK-1916: Application Entrypoint Metadata

**Status:** ✅ Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Add application entrypoint metadata and invocation packet carriers over checked computations.

## Requirements

- Represent entrypoints as runtime/application metadata, not a new surface/Core/IR language form.
- Preserve module imports, source/check identity, callable identity, and runtime target identity.
- Report missing, ambiguous, stale, or incompatible entrypoints with structured diagnostics.
- Keep legacy `workflow` compatibility paths separate from target application entrypoint metadata.

## TDD Steps

1. Add failing tests for entrypoint metadata construction and invalid entrypoint diagnostics.
2. Implement minimal carriers and conversion from existing checked callable/module summaries.
3. Add CLI/engine fixture coverage for entrypoint selection.

## Completion Checklist

- [x] Entrypoint metadata exists over checked computations.
- [x] Invocation packets carry source/check/runtime identity.
- [x] Missing and ambiguous entrypoints fail closed.
- [x] Legacy workflow form is not required for target entrypoint selection.

## Evidence

- Added `ApplicationEntrypointMetadata`, `ApplicationEntrypointKind`,
  `ApplicationEntrypointDiagnostic`, and `ApplicationInvocationPacket` carriers in
  `ash-core::runtime_kernel`.
- Wired `RuntimeArtifactBuildRequest::new_application_entrypoint` and RuntimeKernel verified
  artifacts to carry checked-callable entrypoint metadata and invocation identity.
- Added CLI fixture coverage proving `ash run --dry-run` over target `fn main` reports
  `checked_callable` entrypoint metadata rather than legacy workflow compatibility metadata.
- Verification passed:
  - `cargo fmt --check`
  - `cargo test -p ash-core --test alpha_runtime_kernel_artifact_builder`
  - `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder`
  - `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode`
