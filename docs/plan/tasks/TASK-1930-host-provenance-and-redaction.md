# TASK-1930: Host Provenance And Redaction

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Attach provenance, trace, report, and redaction evidence to every host boundary crossing.

## Requirements

- Emit provenance for successful host calls, failed host calls, denied sandbox attempts, timeout,
  cancellation, and malformed boundary metadata.
- Redact secrets from arguments, environment values, URLs, headers, process command details, and
  provider-specific payloads.
- Connect host boundary evidence to application runtime reports and monitor evidence.
- Ensure reports and traces never mutate authority.

## TDD Steps

1. Add failing provenance/redaction tests for success, failure, denial, timeout, and cancellation.
2. Implement host boundary evidence carriers and redaction policy.
3. Integrate evidence with runtime reports and traces.

## Completion Checklist

- [x] Host boundary evidence exists for success, failure, and denial.
- [x] Secrets are redacted from host reports and traces.
- [x] Evidence links to adapter/provider/hook identity.
- [x] Reports/traces remain authority-neutral.

## Evidence

- Added `HostBoundaryEvidenceId`, `HostBoundaryOutcome`, and `HostBoundaryEvidence` carriers for
  authority-neutral host boundary reporting.
- Admitted host provider projections now record redacted evidence for successful calls, provider
  failures, and sandbox denials.
- Host boundary evidence records provider/operation identity, optional adapter identity,
  sandbox/provenance policy identity, outcome, redacted subject, and bounded diagnostics without raw
  argument values or provider-error payloads.
- Host boundary crossings emit redacted `TraceFactKind::Operation` facts and monitor evidence
  without mutating authority.
- Added TASK-1930 test:
  - `cargo test -p ash-interp --test task_1930_host_provenance_redaction`
- Verified affected gates:
  - `cargo fmt --all -- --check`
  - `python3 tools/docs/validate_orientation_indexes.py --self-test`
  - `bash scripts/check-docs-gate.sh`
  - `git diff --check`
  - `cargo test -p ash-interp --test task_1929_host_sandbox_policy`
  - `cargo check -p ash-core -p ash-interp`
  - `cargo clippy -p ash-core -p ash-interp --all-targets --all-features`
