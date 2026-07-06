# TASK-1925: Host Boundary Seam Audit

**Status:** Complete
**Phase:** [PLAN-197: Host / FFI / Builtins](../PLAN-197-HOST-FFI-BUILTINS.md)

## Description

Audit current builtin dispatch, stdlib builtin declarations, provider registration, runtime adapter,
sandbox, and provenance seams before changing host behavior.

## Requirements

- Map builtin dispatch tables, `pub builtin fn` declarations, provider registries, runtime artifact
  construction, daemon host boundaries, sandbox checks, and provenance records to owning modules.
- Identify current host effects that can execute without explicit row/admission/sandbox evidence.
- Classify seams as current-state compatibility, target substrate, implementation gap, or legacy
  reference.
- Produce an audit artifact under `docs/plan/audits/`.

## TDD Steps

1. Add audit-focused tests or inventory assertions where a seam can be mechanically checked.
2. Create the audit document with ownership and risks.
3. Update PLAN-197 evidence with audit outcomes.

## Completion Checklist

- [x] Audit identifies builtin, provider, adapter, sandbox, and provenance owners.
- [x] Authority-bypass risks are assigned to later tasks.
- [x] Audit evidence distinguishes compatibility from target guidance.

## Evidence

- Added [AUDIT-197](../audits/AUDIT-197-host-boundary-seams.md), covering builtin dispatch,
  provider APIs, standard providers, runtime artifacts/adapters, CLI/daemon host surfaces,
  sandbox/constraint enforcement, provenance/report surfaces, and legacy `extern`/old-form
  references.
- Assigned the direct `process::run` builtin path, permissive provider-local defaults, provider API
  metadata gaps, runtime adapter registry gap, sandbox fail-open behavior, and redacted provenance
  gaps to TASK-1926 through TASK-1932.
- Classified `Act`, `Proc`, `Workflow`, `builtin fn`, and `extern` vocabulary as legacy/reference
  material unless routed through explicit Phase 197 host-boundary substrate.
