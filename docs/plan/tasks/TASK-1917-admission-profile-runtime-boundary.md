# TASK-1917: Admission Profile Runtime Boundary

**Status:** ✅ Complete
**Phase:** [PLAN-196: Application / Workflow Runtime](../PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md)

## Description

Wire admission profiles to application runtime entry boundaries without granting ambient authority.

## Requirements

- Treat admission profiles as explicit runtime-boundary inputs.
- Preserve profile identity in reports, traces, and runtime artifacts.
- Reject missing, malformed, stale, incompatible, or authority-widening profiles fail-closed.
- Ensure profile names alone do not grant capabilities, resources, roles, policies, or providers.

## TDD Steps

1. Add failing admission profile boundary tests.
2. Implement profile validation and report attachment.
3. Verify existing handler/provider and row admission checks still govern authority.

## Completion Checklist

- [x] Admission profiles attach to invocation packets and reports.
- [x] Invalid profiles fail closed with structured diagnostics.
- [x] Profile selection does not bypass provider/role/resource/policy discharge.
- [x] CLI and engine fixtures cover profile behavior.

## Evidence

- Added `ApplicationAdmissionProfile` and `ApplicationAdmissionProfileDiagnostic` carriers in
  `ash-core::runtime_kernel`, with fail-closed missing, malformed, stale, incompatible, and
  authority-widening diagnostics.
- RuntimeKernel invocation packets now carry admission profile metadata alongside entrypoint,
  source, check, and runtime target identity.
- `ash run` and `ash daemon start` attach selected alpha admission profiles as explicit
  non-authority boundary metadata in reports and per-instance artifact summaries.
- Engine and CLI fixtures prove profile selection does not grant capabilities, resources, actions,
  provider authority, roles, or policies by name.
- Verification passed:
  - `cargo fmt --check`
  - `cargo test -p ash-core --test alpha_runtime_kernel_artifact_builder`
  - `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder`
  - `cargo test -p ash-cli --test alpha_admission_profile`
  - `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode`
  - `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane`
  - `cargo clippy --all-targets --all-features`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`
